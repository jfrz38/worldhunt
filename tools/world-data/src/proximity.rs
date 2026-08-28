//! Build-time territorial proximity data encoded into the version-2 world asset.

use crate::{source::SourceRecordKey, validation::ValidatedWorldData};
use geo::{Coord, Intersects, LineString, MapCoords, Polygon};
use geographiclib_rs::{DirectGeodesic, Geodesic, InverseGeodesic};
use rstar::{RTree, primitives::GeomWithData};

const COUNTRY_COUNT: usize = 196;
const MAX_SAMPLE_SPACING_M: f64 = 5_000.0;
const CONSERVATIVE_EARTH_RADIUS_M: f64 = 6_400_000.0;
const WGS84_SEMI_MAJOR_AXIS_M: f64 = 6_378_137.0;
const WGS84_FLATTENING: f64 = 1.0 / 298.257_223_563;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ProximityMatrices {
    country_count: usize,
    distances_km: Vec<u16>,
    adjacency: Vec<bool>,
    boundary_point_count: usize,
}

#[derive(Clone, Copy, Debug)]
struct Point {
    longitude: f64,
    latitude: f64,
}

#[derive(Clone)]
struct TerritorialPolygon {
    polygon: Polygon<f64>,
    longitude_center: f64,
    bounds: Bounds,
}

#[derive(Clone, Copy)]
struct Bounds {
    minimum_longitude: f64,
    maximum_longitude: f64,
    minimum_latitude: f64,
    maximum_latitude: f64,
}

struct CountryGeometry {
    name: String,
    polygons: Vec<TerritorialPolygon>,
    boundary_points: Vec<Point>,
    index: RTree<GeomWithData<[f64; 3], Point>>,
}

pub(crate) fn generate(data: &ValidatedWorldData) -> Result<ProximityMatrices, String> {
    if data.catalog.countries.len() != COUNTRY_COUNT {
        return Err(format!(
            "proximity generation requires {COUNTRY_COUNT} catalog countries, found {}",
            data.catalog.countries.len()
        ));
    }
    let countries = collect_countries(data)?;
    let mut matrices = ProximityMatrices {
        country_count: countries.len(),
        distances_km: vec![0; countries.len() * countries.len()],
        adjacency: vec![false; countries.len() * countries.len()],
        boundary_point_count: countries
            .iter()
            .map(|country| country.boundary_points.len())
            .sum(),
    };
    for first in 0..countries.len() {
        for second in first + 1..countries.len() {
            let (adjacent, distance_km) = pair_proximity(&countries[first], &countries[second])?;
            let first_index = matrix_index(countries.len(), first, second)?;
            let second_index = matrix_index(countries.len(), second, first)?;
            matrices.distances_km[first_index] = distance_km;
            matrices.distances_km[second_index] = distance_km;
            matrices.adjacency[first_index] = adjacent;
            matrices.adjacency[second_index] = adjacent;
        }
    }
    validate(&matrices)?;
    Ok(matrices)
}

impl ProximityMatrices {
    pub(crate) fn country_count(&self) -> usize {
        self.country_count
    }

    pub(crate) fn distances_km(&self) -> &[u16] {
        &self.distances_km
    }

    pub(crate) fn adjacency(&self) -> &[bool] {
        &self.adjacency
    }

    pub(crate) fn distance_km(&self, first: usize, second: usize) -> Option<u16> {
        matrix_index(self.country_count, first, second)
            .ok()
            .and_then(|index| self.distances_km.get(index).copied())
    }

    pub(crate) fn are_adjacent(&self, first: usize, second: usize) -> Option<bool> {
        matrix_index(self.country_count, first, second)
            .ok()
            .and_then(|index| self.adjacency.get(index).copied())
    }

    pub(crate) fn adjacent_pair_count(&self) -> usize {
        (0..self.country_count)
            .flat_map(|first| (first + 1..self.country_count).map(move |second| (first, second)))
            .filter(|(first, second)| self.are_adjacent(*first, *second) == Some(true))
            .count()
    }

    pub(crate) fn maximum_distance_km(&self) -> u16 {
        self.distances_km.iter().copied().max().unwrap_or(0)
    }

    pub(crate) fn boundary_point_count(&self) -> usize {
        self.boundary_point_count
    }
}

fn collect_countries(data: &ValidatedWorldData) -> Result<Vec<CountryGeometry>, String> {
    let source_index = data.source.index_by_selector();
    data.catalog
        .countries
        .iter()
        .map(|country| {
            let mut polygons = Vec::new();
            let mut boundary_points = Vec::new();
            for selector in &country.source_records {
                let key = SourceRecordKey::new(&selector.iso3, &selector.name);
                let feature = source_index
                    .get(&key)
                    .and_then(|features| features.first())
                    .ok_or_else(|| format!("{} has no validated source geometry", country.iso3))?;
                let geometry = feature
                    .geometry
                    .as_ref()
                    .ok_or_else(|| format!("{} has no validated source geometry", country.iso3))?;
                for rings in geometry.polygons() {
                    polygons.push(normalize_polygon(rings)?);
                    for ring in rings {
                        boundary_points.extend(densify_ring(ring)?);
                    }
                }
            }
            if polygons.is_empty() || boundary_points.is_empty() {
                return Err(format!("{} has no territorial boundary", country.iso3));
            }
            let index = RTree::bulk_load(
                boundary_points
                    .iter()
                    .copied()
                    .map(|point| GeomWithData::new(ecef(point), point))
                    .collect(),
            );
            Ok(CountryGeometry {
                name: country.name.clone(),
                polygons,
                boundary_points,
                index,
            })
        })
        .collect()
}

fn normalize_polygon(rings: &[Vec<Vec<f64>>]) -> Result<TerritorialPolygon, String> {
    let exterior = unwrap_ring(rings.first().ok_or("polygon has no exterior ring")?, None)?;
    let longitude_center =
        exterior.iter().map(|point| point.x).sum::<f64>() / exterior.len() as f64;
    let interiors = rings
        .iter()
        .skip(1)
        .map(|ring| unwrap_ring(ring, Some(longitude_center)))
        .collect::<Result<Vec<_>, _>>()?;
    let bounds = Bounds {
        minimum_longitude: exterior
            .iter()
            .map(|point| point.x)
            .fold(f64::INFINITY, f64::min),
        maximum_longitude: exterior
            .iter()
            .map(|point| point.x)
            .fold(f64::NEG_INFINITY, f64::max),
        minimum_latitude: exterior
            .iter()
            .map(|point| point.y)
            .fold(f64::INFINITY, f64::min),
        maximum_latitude: exterior
            .iter()
            .map(|point| point.y)
            .fold(f64::NEG_INFINITY, f64::max),
    };
    Ok(TerritorialPolygon {
        polygon: Polygon::new(
            LineString::new(exterior),
            interiors.into_iter().map(LineString::new).collect(),
        ),
        longitude_center,
        bounds,
    })
}

fn unwrap_ring(ring: &[Vec<f64>], reference: Option<f64>) -> Result<Vec<Coord<f64>>, String> {
    let first = ring.first().ok_or("polygon ring is empty")?;
    let mut longitude = first[0];
    let mut coordinates = Vec::with_capacity(ring.len());
    for point in ring {
        longitude = unwrap_longitude(point[0], longitude);
        coordinates.push(Coord {
            x: longitude,
            y: point[1],
        });
    }
    if let Some(reference) = reference {
        let center =
            coordinates.iter().map(|point| point.x).sum::<f64>() / coordinates.len() as f64;
        let shift = ((reference - center) / 360.0).round() * 360.0;
        for coordinate in &mut coordinates {
            coordinate.x += shift;
        }
    }
    Ok(coordinates)
}

fn unwrap_longitude(value: f64, reference: f64) -> f64 {
    let mut value = value;
    while value - reference > 180.0 {
        value -= 360.0;
    }
    while value - reference < -180.0 {
        value += 360.0;
    }
    value
}

fn pair_proximity(
    first: &CountryGeometry,
    second: &CountryGeometry,
) -> Result<(bool, u16), String> {
    if territories_intersect(first, second) {
        return Ok((true, 0));
    }
    let distance_m = minimum_boundary_distance(first, second)?;
    if !distance_m.is_finite() || distance_m < 0.0 {
        return Err(format!(
            "{} and {} produced an invalid territorial distance",
            first.name, second.name
        ));
    }
    let distance_km = (distance_m / 1_000.0).round();
    if distance_km > f64::from(u16::MAX) {
        return Err(format!(
            "{} and {} exceed the encodable territorial distance range",
            first.name, second.name
        ));
    }
    Ok((false, distance_km as u16))
}

fn territories_intersect(first: &CountryGeometry, second: &CountryGeometry) -> bool {
    first.polygons.iter().any(|left| {
        second.polygons.iter().any(|right| {
            let base_shift = ((left.longitude_center - right.longitude_center) / 360.0).round();
            (-1..=1).any(|offset| {
                let shift = (base_shift + f64::from(offset)) * 360.0;
                if !left.bounds.overlaps(right.bounds.shifted(shift)) {
                    return false;
                }
                let translated = right.polygon.map_coords(|coordinate| Coord {
                    x: coordinate.x + shift,
                    y: coordinate.y,
                });
                left.polygon.intersects(&translated)
            })
        })
    })
}

impl Bounds {
    fn shifted(self, longitude: f64) -> Self {
        Self {
            minimum_longitude: self.minimum_longitude + longitude,
            maximum_longitude: self.maximum_longitude + longitude,
            ..self
        }
    }

    fn overlaps(self, other: Self) -> bool {
        self.minimum_longitude <= other.maximum_longitude
            && self.maximum_longitude >= other.minimum_longitude
            && self.minimum_latitude <= other.maximum_latitude
            && self.maximum_latitude >= other.minimum_latitude
    }
}

fn densify_ring(ring: &[Vec<f64>]) -> Result<Vec<Point>, String> {
    let geodesic = Geodesic::wgs84();
    let mut points = Vec::new();
    for edge in ring.windows(2) {
        let start = Point {
            longitude: edge[0][0],
            latitude: edge[0][1],
        };
        let end = Point {
            longitude: edge[1][0],
            latitude: edge[1][1],
        };
        let segments = (conservative_segment_length_m(start, end) / MAX_SAMPLE_SPACING_M)
            .ceil()
            .max(1.0) as usize;
        if segments == 1 {
            points.push(start);
            continue;
        }
        let (distance_m, azimuth): (f64, f64) =
            geodesic.inverse(start.latitude, start.longitude, end.latitude, end.longitude);
        if !distance_m.is_finite() || !azimuth.is_finite() {
            return Err("boundary segment has an invalid geodesic measurement".to_owned());
        }
        for step in 0..segments {
            let distance = distance_m * step as f64 / segments as f64;
            let (latitude, longitude): (f64, f64) =
                geodesic.direct(start.latitude, start.longitude, azimuth, distance);
            points.push(Point {
                longitude,
                latitude,
            });
        }
    }
    Ok(points)
}

fn conservative_segment_length_m(first: Point, second: Point) -> f64 {
    let latitude_delta = (second.latitude - first.latitude).to_radians();
    let longitude_delta = unwrap_longitude(second.longitude, first.longitude).to_radians()
        - first.longitude.to_radians();
    let latitude = latitude_delta / 2.0;
    let longitude = longitude_delta / 2.0;
    let haversine = latitude.sin().powi(2)
        + first.latitude.to_radians().cos()
            * second.latitude.to_radians().cos()
            * longitude.sin().powi(2);
    CONSERVATIVE_EARTH_RADIUS_M * 2.0 * haversine.sqrt().asin()
}

fn minimum_boundary_distance(
    first: &CountryGeometry,
    second: &CountryGeometry,
) -> Result<f64, String> {
    let (queries, indexed) = if first.boundary_points.len() <= second.boundary_points.len() {
        (&first.boundary_points, &second.index)
    } else {
        (&second.boundary_points, &first.index)
    };
    let mut candidates = Vec::new();
    for point in queries {
        let query = ecef(*point);
        let nearest = indexed
            .nearest_neighbor(query)
            .ok_or_else(|| "territorial distance has no boundary candidates".to_owned())?;
        candidates.push((
            squared_chord_distance(query, ecef(nearest.data)),
            *point,
            nearest.data,
        ));
    }
    candidates.sort_by(|left, right| left.0.total_cmp(&right.0));
    candidates.truncate(8);
    candidates
        .into_iter()
        .map(|(_, first, second)| inverse_distance_m(first, second))
        .reduce(f64::min)
        .ok_or_else(|| "territorial distance has no boundary candidates".to_owned())
}

fn squared_chord_distance(first: [f64; 3], second: [f64; 3]) -> f64 {
    first
        .into_iter()
        .zip(second)
        .map(|(first, second)| (first - second).powi(2))
        .sum()
}

fn inverse_distance_m(first: Point, second: Point) -> f64 {
    let semi_minor_axis = WGS84_SEMI_MAJOR_AXIS_M * (1.0 - WGS84_FLATTENING);
    let reduced_first = ((1.0 - WGS84_FLATTENING) * first.latitude.to_radians().tan()).atan();
    let reduced_second = ((1.0 - WGS84_FLATTENING) * second.latitude.to_radians().tan()).atan();
    let (sin_first, cos_first) = reduced_first.sin_cos();
    let (sin_second, cos_second) = reduced_second.sin_cos();
    let longitude = (second.longitude - first.longitude).to_radians();
    let mut lambda = longitude;
    for _ in 0..100 {
        let (sin_lambda, cos_lambda) = lambda.sin_cos();
        let sin_sigma = ((cos_second * sin_lambda).powi(2)
            + (cos_first * sin_second - sin_first * cos_second * cos_lambda).powi(2))
        .sqrt();
        if sin_sigma == 0.0 {
            return 0.0;
        }
        let cos_sigma = sin_first * sin_second + cos_first * cos_second * cos_lambda;
        let sigma = sin_sigma.atan2(cos_sigma);
        let sin_alpha = cos_first * cos_second * sin_lambda / sin_sigma;
        let cos_squared_alpha = 1.0 - sin_alpha.powi(2);
        let cos_two_sigma_m = if cos_squared_alpha == 0.0 {
            0.0
        } else {
            cos_sigma - 2.0 * sin_first * sin_second / cos_squared_alpha
        };
        let correction = WGS84_FLATTENING / 16.0
            * cos_squared_alpha
            * (4.0 + WGS84_FLATTENING * (4.0 - 3.0 * cos_squared_alpha));
        let next = longitude
            + (1.0 - correction)
                * WGS84_FLATTENING
                * sin_alpha
                * (sigma
                    + correction
                        * sin_sigma
                        * (cos_two_sigma_m
                            + correction * cos_sigma * (-1.0 + 2.0 * cos_two_sigma_m.powi(2))));
        if (next - lambda).abs() < 1e-12 {
            let squared_u = cos_squared_alpha
                * (WGS84_SEMI_MAJOR_AXIS_M.powi(2) - semi_minor_axis.powi(2))
                / semi_minor_axis.powi(2);
            let coefficient_a = 1.0
                + squared_u / 16_384.0
                    * (4_096.0 + squared_u * (-768.0 + squared_u * (320.0 - 175.0 * squared_u)));
            let coefficient_b = squared_u / 1_024.0
                * (256.0 + squared_u * (-128.0 + squared_u * (74.0 - 47.0 * squared_u)));
            let delta_sigma = coefficient_b
                * sin_sigma
                * (cos_two_sigma_m
                    + coefficient_b / 4.0
                        * (cos_sigma * (-1.0 + 2.0 * cos_two_sigma_m.powi(2))
                            - coefficient_b / 6.0
                                * cos_two_sigma_m
                                * (-3.0 + 4.0 * sin_sigma.powi(2))
                                * (-3.0 + 4.0 * cos_two_sigma_m.powi(2))));
            return semi_minor_axis * coefficient_a * (sigma - delta_sigma);
        }
        lambda = next;
    }
    let geodesic = Geodesic::wgs84();
    geodesic.inverse(
        first.latitude,
        first.longitude,
        second.latitude,
        second.longitude,
    )
}

fn ecef(point: Point) -> [f64; 3] {
    let latitude = point.latitude.to_radians();
    let longitude = point.longitude.to_radians();
    let eccentricity_squared = WGS84_FLATTENING * (2.0 - WGS84_FLATTENING);
    let radius =
        WGS84_SEMI_MAJOR_AXIS_M / (1.0 - eccentricity_squared * latitude.sin().powi(2)).sqrt();
    [
        radius * latitude.cos() * longitude.cos(),
        radius * latitude.cos() * longitude.sin(),
        radius * (1.0 - eccentricity_squared) * latitude.sin(),
    ]
}

fn matrix_index(dimension: usize, row: usize, column: usize) -> Result<usize, String> {
    if row >= dimension || column >= dimension {
        return Err("proximity matrix index is out of range".to_owned());
    }
    row.checked_mul(dimension)
        .and_then(|offset| offset.checked_add(column))
        .ok_or_else(|| "proximity matrix index overflows".to_owned())
}

fn validate(matrices: &ProximityMatrices) -> Result<(), String> {
    if matrices.country_count != COUNTRY_COUNT {
        return Err("proximity matrix country count is invalid".to_owned());
    }
    let expected_length = matrices.country_count * matrices.country_count;
    if matrices.distances_km.len() != expected_length || matrices.adjacency.len() != expected_length
    {
        return Err("proximity matrix dimensions are invalid".to_owned());
    }
    for row in 0..matrices.country_count {
        if matrices.distance_km(row, row) != Some(0)
            || matrices.are_adjacent(row, row) != Some(false)
        {
            return Err("proximity matrix diagonal is invalid".to_owned());
        }
        for column in row + 1..matrices.country_count {
            if matrices.distance_km(row, column) != matrices.distance_km(column, row)
                || matrices.are_adjacent(row, column) != matrices.are_adjacent(column, row)
            {
                return Err("proximity matrices are not symmetric".to_owned());
            }
            if matrices.are_adjacent(row, column) == Some(true)
                && matrices.distance_km(row, column) != Some(0)
            {
                return Err("adjacent territories must have zero separation".to_owned());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
