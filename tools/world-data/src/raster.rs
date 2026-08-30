use crate::{
    catalog::Catalog,
    source::{Feature, GeometryKind, SourceRecordKey},
    validation::ValidatedWorldData,
};
use std::collections::{HashMap, VecDeque};

pub(crate) const WIDTH: u16 = 720;
pub(crate) const HEIGHT: u16 = 300;
pub(crate) const WATER: u16 = u16::MAX;
pub(crate) const NEUTRAL_LAND: u16 = u16::MAX - 1;
const NORTH_LATITUDE: f64 = 90.0;
const SOUTH_LATITUDE: f64 = -60.0;
const LATITUDE_SPAN: f64 = NORTH_LATITUDE - SOUTH_LATITUDE;

pub(crate) struct RasterData {
    pub(crate) cells: Vec<u16>,
    pub(crate) borders: Vec<u8>,
    pub(crate) anchors: Vec<(u16, u16)>,
}

struct Shape<'a> {
    id: u16,
    feature: &'a Feature,
    min_lon: f64,
    max_lon: f64,
    min_lat: f64,
    max_lat: f64,
    scanlines: Vec<Vec<f64>>,
}

pub(crate) fn rasterize(data: &ValidatedWorldData) -> Result<RasterData, String> {
    let shapes = shapes_for(&data.catalog, data);
    let candidates = candidate_index(&shapes);
    let mut cells = vec![WATER; usize::from(WIDTH) * usize::from(HEIGHT)];
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let mut counts: HashMap<u16, u8> = HashMap::new();
            // Four fixed sub-pixel samples preserve narrow territory deterministically.
            for (offset_x, offset_y, sample_row) in [
                (0.25, 0.25, usize::from(y) * 2),
                (0.75, 0.25, usize::from(y) * 2),
                (0.25, 0.75, usize::from(y) * 2 + 1),
                (0.75, 0.75, usize::from(y) * 2 + 1),
            ] {
                let lon = -180.0 + (f64::from(x) + offset_x) * 360.0 / f64::from(WIDTH);
                let lat =
                    NORTH_LATITUDE - (f64::from(y) + offset_y) * LATITUDE_SPAN / f64::from(HEIGHT);
                let bucket = bucket_for(lon, lat);
                if let Some(id) = candidates[bucket]
                    .iter()
                    .map(|index| &shapes[*index])
                    .filter(|shape| {
                        shape.may_contain(lon, lat) && shape.contains_sample(sample_row, lon)
                    })
                    .map(|shape| shape.id)
                    .min()
                {
                    *counts.entry(id).or_default() += 1;
                }
            }
            if let Some((id, _)) = counts
                .into_iter()
                .max_by_key(|(id, count)| (*count, std::cmp::Reverse(*id)))
            {
                cells[usize::from(y) * usize::from(WIDTH) + usize::from(x)] = id;
            }
        }
    }
    let anchors = (0..data.catalog.countries.len())
        .map(|id| {
            source_anchor(id as u16, &shapes, &cells)
                .or_else(|| find_anchor(id as u16, &cells))
                .ok_or_else(|| format!("country index {id} has no visual anchor"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let borders = border_mask(&cells);
    Ok(RasterData {
        cells,
        borders,
        anchors,
    })
}

const BUCKET_WIDTH: usize = 36;
const BUCKET_HEIGHT: usize = 18;

fn candidate_index(shapes: &[Shape<'_>]) -> Vec<Vec<usize>> {
    let mut buckets = vec![Vec::new(); BUCKET_WIDTH * BUCKET_HEIGHT];
    for (index, shape) in shapes.iter().enumerate() {
        let min_y = ((90.0 - shape.max_lat) / 10.0).floor().clamp(0.0, 17.0) as usize;
        let max_y = ((90.0 - shape.min_lat) / 10.0).floor().clamp(0.0, 17.0) as usize;
        let (min_x, max_x) = if shape.max_lon - shape.min_lon > 180.0 {
            (0, 35)
        } else {
            (
                ((shape.min_lon + 180.0) / 10.0).floor().clamp(0.0, 35.0) as usize,
                ((shape.max_lon + 180.0) / 10.0).floor().clamp(0.0, 35.0) as usize,
            )
        };
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                buckets[y * BUCKET_WIDTH + x].push(index);
            }
        }
    }
    buckets
}

fn bucket_for(lon: f64, lat: f64) -> usize {
    let x = ((lon + 180.0) / 10.0).floor().clamp(0.0, 35.0) as usize;
    let y = ((90.0 - lat) / 10.0).floor().clamp(0.0, 17.0) as usize;
    y * BUCKET_WIDTH + x
}

fn shapes_for<'a>(catalog: &'a Catalog, data: &'a ValidatedWorldData) -> Vec<Shape<'a>> {
    let index = data.source.index_by_selector();
    let mut mapped = HashMap::new();
    for (id, country) in catalog.countries.iter().enumerate() {
        for selector in &country.source_records {
            mapped.insert(
                SourceRecordKey::new(&selector.iso3, &selector.name),
                id as u16,
            );
        }
    }
    let mut shapes = Vec::new();
    for (key, features) in index {
        let id = mapped.get(&key).copied().unwrap_or(NEUTRAL_LAND);
        for feature in features {
            if matches!(
                feature.geometry,
                Some(GeometryKind::Polygon(_) | GeometryKind::MultiPolygon(_))
            ) {
                shapes.push(Shape::new(id, feature));
            }
        }
    }
    shapes
}

impl Shape<'_> {
    fn new(id: u16, feature: &Feature) -> Shape<'_> {
        let (min_lon, max_lon, min_lat, max_lat) = bounds(feature);
        let scanlines = (0..usize::from(HEIGHT) * 2)
            .map(|row| {
                let lat =
                    NORTH_LATITUDE - (row as f64 + 0.5) * LATITUDE_SPAN / (f64::from(HEIGHT) * 2.0);
                feature
                    .geometry
                    .as_ref()
                    .into_iter()
                    .flat_map(GeometryKind::polygons)
                    .flat_map(|polygon| polygon.iter())
                    .flat_map(|ring| ring_intersections(ring, lat))
                    .collect()
            })
            .collect();
        Shape {
            id,
            feature,
            min_lon,
            max_lon,
            min_lat,
            max_lat,
            scanlines,
        }
    }

    fn may_contain(&self, lon: f64, lat: f64) -> bool {
        lat >= self.min_lat
            && lat <= self.max_lat
            && (self.max_lon - self.min_lon > 180.0 || (lon >= self.min_lon && lon <= self.max_lon))
    }

    fn contains_sample(&self, sample_row: usize, lon: f64) -> bool {
        self.scanlines[sample_row]
            .iter()
            .fold(false, |inside, crossing| {
                inside ^ (lon < unwrap_longitude(*crossing, lon))
            })
    }
}

fn bounds(feature: &Feature) -> (f64, f64, f64, f64) {
    let mut bounds = (180.0_f64, -180.0_f64, 90.0_f64, -90.0_f64);
    for polygon in feature
        .geometry
        .as_ref()
        .into_iter()
        .flat_map(GeometryKind::polygons)
    {
        for ring in polygon {
            for point in ring {
                bounds.0 = bounds.0.min(point[0]);
                bounds.1 = bounds.1.max(point[0]);
                bounds.2 = bounds.2.min(point[1]);
                bounds.3 = bounds.3.max(point[1]);
            }
        }
    }
    bounds
}

fn contains_polygon(rings: &[Vec<Vec<f64>>], lon: f64, lat: f64) -> bool {
    rings
        .first()
        .is_some_and(|outer| point_in_ring(outer, lon, lat))
        && rings
            .iter()
            .skip(1)
            .all(|hole| !point_in_ring(hole, lon, lat))
}

fn point_in_ring(ring: &[Vec<f64>], lon: f64, lat: f64) -> bool {
    let mut inside = false;
    let mut previous = (unwrap_longitude(ring[0][0], lon), ring[0][1]);
    for point in ring.iter().skip(1) {
        // Keep consecutive vertices together before comparing them with the query.
        let current = (unwrap_longitude(point[0], previous.0), point[1]);
        let (x1, y1) = previous;
        let (x2, y2) = current;
        if (y1 > lat) != (y2 > lat) && lon < (x2 - x1) * (lat - y1) / (y2 - y1) + x1 {
            inside = !inside;
        }
        previous = current;
    }
    inside
}

fn ring_intersections(ring: &[Vec<f64>], lat: f64) -> impl Iterator<Item = f64> + '_ {
    ring.windows(2).filter_map(move |edge| {
        let (x1, y1) = (edge[0][0], edge[0][1]);
        let (x2, y2) = (unwrap_longitude(edge[1][0], x1), edge[1][1]);
        ((y1 > lat) != (y2 > lat)).then(|| {
            let crossing = (x2 - x1) * (lat - y1) / (y2 - y1) + x1;
            unwrap_longitude(crossing, 0.0)
        })
    })
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

fn find_anchor(id: u16, cells: &[u16]) -> Option<(u16, u16)> {
    let component = largest_component(id, cells)?;
    let angle = std::f64::consts::TAU;
    let mean_x = component
        .iter()
        .map(|(x, _)| {
            let phase = f64::from(*x) * angle / f64::from(WIDTH);
            (phase.sin(), phase.cos())
        })
        .fold((0.0, 0.0), |(sin, cos), (next_sin, next_cos)| {
            (sin + next_sin, cos + next_cos)
        });
    let mean_x = mean_x.0.atan2(mean_x.1).rem_euclid(angle) * f64::from(WIDTH) / angle;
    let mean_y = component.iter().map(|(_, y)| f64::from(*y)).sum::<f64>() / component.len() as f64;

    component
        .into_iter()
        .min_by(|(left_x, left_y), (right_x, right_y)| {
            anchor_distance(*left_x, *left_y, mean_x, mean_y)
                .total_cmp(&anchor_distance(*right_x, *right_y, mean_x, mean_y))
        })
}

fn largest_component(id: u16, cells: &[u16]) -> Option<Vec<(u16, u16)>> {
    let mut visited = vec![false; cells.len()];
    let mut largest = Vec::new();
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let index = usize::from(y) * usize::from(WIDTH) + usize::from(x);
            if visited[index] || cells[index] != id {
                continue;
            }
            let mut component = Vec::new();
            let mut queue = VecDeque::from([(x, y)]);
            visited[index] = true;
            while let Some((current_x, current_y)) = queue.pop_front() {
                component.push((current_x, current_y));
                let mut neighbors = vec![
                    ((current_x + WIDTH - 1) % WIDTH, current_y),
                    ((current_x + 1) % WIDTH, current_y),
                ];
                if current_y > 0 {
                    neighbors.push((current_x, current_y - 1));
                }
                if current_y + 1 < HEIGHT {
                    neighbors.push((current_x, current_y + 1));
                }
                for (neighbor_x, neighbor_y) in neighbors {
                    let neighbor_index =
                        usize::from(neighbor_y) * usize::from(WIDTH) + usize::from(neighbor_x);
                    if !visited[neighbor_index] && cells[neighbor_index] == id {
                        visited[neighbor_index] = true;
                        queue.push_back((neighbor_x, neighbor_y));
                    }
                }
            }
            if component.len() > largest.len() {
                largest = component;
            }
        }
    }
    (!largest.is_empty()).then_some(largest)
}

fn anchor_distance(x: u16, y: u16, mean_x: f64, mean_y: f64) -> f64 {
    let horizontal = (f64::from(x) - mean_x).abs();
    let horizontal = horizontal.min(f64::from(WIDTH) - horizontal);
    horizontal.powi(2) + (f64::from(y) - mean_y).powi(2)
}

fn source_anchor(id: u16, shapes: &[Shape<'_>], cells: &[u16]) -> Option<(u16, u16)> {
    shapes
        .iter()
        .filter(|shape| shape.id == id)
        .flat_map(|shape| shape.feature.geometry.as_ref().into_iter())
        .flat_map(GeometryKind::polygons)
        .max_by(|left, right| polygon_area(left).total_cmp(&polygon_area(right)))
        .and_then(polygon_anchor)
        .and_then(|candidate| anchor_in_raster(id, candidate, cells))
}

fn anchor_in_raster(id: u16, candidate: (u16, u16), cells: &[u16]) -> Option<(u16, u16)> {
    let index = usize::from(candidate.1) * usize::from(WIDTH) + usize::from(candidate.0);
    if cells.get(index) == Some(&id) {
        return Some(candidate);
    }
    cells
        .iter()
        .enumerate()
        .filter(|(_, cell)| **cell == id)
        .map(|(index, _)| {
            (
                (index % usize::from(WIDTH)) as u16,
                (index / usize::from(WIDTH)) as u16,
            )
        })
        .min_by(|left, right| {
            anchor_distance(
                left.0,
                left.1,
                f64::from(candidate.0),
                f64::from(candidate.1),
            )
            .total_cmp(&anchor_distance(
                right.0,
                right.1,
                f64::from(candidate.0),
                f64::from(candidate.1),
            ))
        })
        .or(Some(candidate))
}

fn polygon_area(rings: &[Vec<Vec<f64>>]) -> f64 {
    let Some(outer) = rings.first() else {
        return 0.0;
    };
    let Some(reference) = outer.first() else {
        return 0.0;
    };
    outer
        .iter()
        .zip(outer.iter().cycle().skip(1))
        .take(outer.len())
        .map(|(from, to)| {
            let from_x = unwrap_longitude(from[0], reference[0]);
            let to_x = unwrap_longitude(to[0], reference[0]);
            from_x * to[1] - to_x * from[1]
        })
        .sum::<f64>()
        .abs()
        / 2.0
}

fn polygon_anchor(rings: &[Vec<Vec<f64>>]) -> Option<(u16, u16)> {
    let outer = rings.first()?;
    let reference = outer.first()?[0];
    let (min_lon, max_lon, min_lat, max_lat) = outer.iter().fold(
        (180.0_f64, -180.0_f64, 90.0_f64, -90.0_f64),
        |bounds, point| {
            let longitude = unwrap_longitude(point[0], reference);
            (
                bounds.0.min(longitude),
                bounds.1.max(longitude),
                bounds.2.min(point[1]),
                bounds.3.max(point[1]),
            )
        },
    );
    let center = ((min_lon + max_lon) / 2.0, (min_lat + max_lat) / 2.0);
    (0..32)
        .flat_map(|row| (0..32).map(move |column| (row, column)))
        .map(|(row, column)| {
            let longitude = min_lon + (f64::from(column) + 0.5) * (max_lon - min_lon) / 32.0;
            let latitude = min_lat + (f64::from(row) + 0.5) * (max_lat - min_lat) / 32.0;
            (longitude, latitude)
        })
        .filter(|&(longitude, latitude)| contains_polygon(rings, longitude, latitude))
        .min_by(|left, right| {
            let left_distance = (left.0 - center.0).powi(2) + (left.1 - center.1).powi(2);
            let right_distance = (right.0 - center.0).powi(2) + (right.1 - center.1).powi(2);
            left_distance.total_cmp(&right_distance)
        })
        .map(|(longitude, latitude)| project(longitude, latitude))
}

fn project(lon: f64, lat: f64) -> (u16, u16) {
    let lon = (lon + 180.0).rem_euclid(360.0) - 180.0;
    (
        ((lon + 180.0) * f64::from(WIDTH) / 360.0)
            .floor()
            .clamp(0.0, f64::from(WIDTH - 1)) as u16,
        ((NORTH_LATITUDE - lat) * f64::from(HEIGHT) / LATITUDE_SPAN)
            .floor()
            .clamp(0.0, f64::from(HEIGHT - 1)) as u16,
    )
}

fn border_mask(cells: &[u16]) -> Vec<u8> {
    let mut borders = vec![0; cells.len()];
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let index = usize::from(y) * usize::from(WIDTH) + usize::from(x);
            let right_index = usize::from(y) * usize::from(WIDTH) + usize::from((x + 1) % WIDTH);
            let id = cells[index];
            let right = cells[right_index];
            let down = if y + 1 < HEIGHT {
                cells[usize::from(y + 1) * usize::from(WIDTH) + usize::from(x)]
            } else {
                WATER
            };
            mark_transition(&mut borders, index, right_index, id, right);
            if y + 1 < HEIGHT {
                mark_transition(&mut borders, index, index + usize::from(WIDTH), id, down);
            }
        }
    }
    borders
}

fn mark_transition(borders: &mut [u8], first: usize, second: usize, first_id: u16, second_id: u16) {
    if first_id == second_id {
        return;
    }
    // Keep a one-cell line on land; country-country boundaries use a stable side.
    let index = match (first_id == WATER, second_id == WATER) {
        (true, false) => second,
        (false, true) => first,
        (false, false) => {
            if first_id < second_id {
                first
            } else {
                second
            }
        }
        (true, true) => return,
    };
    borders[index] = 1;
}

#[cfg(test)]
mod tests {
    use super::{
        HEIGHT, WATER, WIDTH, anchor_in_raster, contains_polygon, find_anchor, mark_transition,
        point_in_ring, project, rasterize,
    };
    use crate::validation::load_validated_repository;
    use std::path::Path;
    #[test]
    fn recognizes_an_outer_ring_and_a_hole() {
        let polygon = vec![
            vec![
                vec![0.0, 0.0],
                vec![4.0, 0.0],
                vec![4.0, 4.0],
                vec![0.0, 4.0],
                vec![0.0, 0.0],
            ],
            vec![
                vec![1.0, 1.0],
                vec![3.0, 1.0],
                vec![3.0, 3.0],
                vec![1.0, 3.0],
                vec![1.0, 1.0],
            ],
        ];
        assert!(contains_polygon(&polygon, 0.5, 0.5));
        assert!(!contains_polygon(&polygon, 2.0, 2.0));
        assert!(point_in_ring(&polygon[0], 3.5, 3.5));
    }

    #[test]
    fn marks_each_territory_transition_once() {
        let mut borders = vec![0; 4];
        mark_transition(&mut borders, 0, 1, 0, 1);
        mark_transition(&mut borders, 2, 3, WATER, 0);

        assert_eq!(borders, vec![1, 0, 0, 1]);
    }

    #[test]
    fn recognizes_a_polygon_across_the_antimeridian() {
        let ring = vec![
            vec![179.0, 0.0],
            vec![-179.0, 0.0],
            vec![-179.0, 2.0],
            vec![179.0, 2.0],
            vec![179.0, 0.0],
        ];

        assert!(point_in_ring(&ring, 179.5, 1.0));
        assert!(point_in_ring(&ring, -179.5, 1.0));
        assert!(!point_in_ring(&ring, 0.0, 1.0));
    }

    #[test]
    fn projects_wrapped_longitudes_to_the_same_cell() {
        assert_eq!(project(181.0, 0.0), project(-179.0, 0.0));
    }

    #[test]
    fn moves_a_source_anchor_to_the_nearest_owned_raster_cell() {
        let mut cells = vec![WATER; usize::from(WIDTH) * usize::from(HEIGHT)];
        cells[100 * usize::from(WIDTH) + 11] = 7;

        assert_eq!(anchor_in_raster(7, (10, 100), &cells), Some((11, 100)));
    }

    #[test]
    fn anchors_the_largest_connected_component() {
        let mut cells = vec![WATER; usize::from(WIDTH) * usize::from(HEIGHT)];
        for x in 10_u16..15 {
            cells[usize::from(100_u16) * usize::from(WIDTH) + usize::from(x)] = 7;
        }
        for x in 300_u16..302 {
            for y in 100_u16..102 {
                cells[usize::from(y) * usize::from(WIDTH) + usize::from(x)] = 7;
            }
        }

        assert!(matches!(find_anchor(7, &cells), Some((11 | 12, 100))));
    }

    #[test]
    fn treats_components_across_the_antimeridian_as_connected() {
        let mut cells = vec![WATER; usize::from(WIDTH) * usize::from(HEIGHT)];
        cells[usize::from(100_u16) * usize::from(WIDTH)] = 7;
        cells[usize::from(100_u16) * usize::from(WIDTH) + usize::from(WIDTH - 1)] = 7;

        assert!(matches!(find_anchor(7, &cells), Some((0 | 719, 100))));
    }

    #[test]
    fn anchors_russia_in_its_mainland() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .expect("repository root exists");
        let validated = load_validated_repository(root).expect("committed data is valid");
        let russia = validated
            .catalog
            .countries
            .iter()
            .position(|country| country.iso3 == "RUS")
            .expect("Russia is catalogued") as u16;
        let (x, y) = rasterize(&validated)
            .expect("rasterizes committed data")
            .anchors[usize::from(russia)];

        assert!(
            (500..=650).contains(&x),
            "unexpected Russia longitude cell {x}"
        );
        assert!(
            (35..=80).contains(&y),
            "unexpected Russia latitude cell {y}"
        );
    }

    #[test]
    fn anchors_pacific_countries_in_their_source_regions() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .expect("repository root exists");
        let validated = load_validated_repository(root).expect("committed data is valid");
        let anchors = rasterize(&validated)
            .expect("rasterizes committed data")
            .anchors;

        for (iso3, expected_x, expected_y) in [
            ("NZL", 660..=719, 220..=299),
            ("FJI", 680..=719, 180..=240),
            ("WSM", 0..=50, 180..=240),
        ] {
            let country = validated
                .catalog
                .countries
                .iter()
                .position(|country| country.iso3 == iso3)
                .expect("country is catalogued");
            let (x, y) = anchors[country];
            assert!(
                expected_x.contains(&x),
                "unexpected {iso3} longitude cell {x}"
            );
            assert!(
                expected_y.contains(&y),
                "unexpected {iso3} latitude cell {y}"
            );
        }
    }
}
