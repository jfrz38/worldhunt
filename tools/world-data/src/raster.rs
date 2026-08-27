use crate::{
    catalog::Catalog,
    source::{Feature, GeometryKind, SourceRecordKey},
    validation::ValidatedWorldData,
};
use std::collections::HashMap;

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
            find_anchor(id as u16, &cells)
                .or_else(|| source_anchor(id as u16, &shapes))
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
    for start in 0..ring.len() - 1 {
        let end = start + 1;
        let (x1, y1) = (unwrap_longitude(ring[start][0], lon), ring[start][1]);
        let (x2, y2) = (unwrap_longitude(ring[end][0], lon), ring[end][1]);
        if (y1 > lat) != (y2 > lat) && lon < (x2 - x1) * (lat - y1) / (y2 - y1) + x1 {
            inside = !inside;
        }
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
    let mut best = None;
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let index = usize::from(y) * usize::from(WIDTH) + usize::from(x);
            if cells[index] == id {
                // A cell surrounded by its own territory makes the marker less likely to obscure a border.
                let score = [x.saturating_sub(1), x + 1]
                    .into_iter()
                    .filter(|neighbor| {
                        *neighbor < WIDTH
                            && cells[usize::from(y) * usize::from(WIDTH) + usize::from(*neighbor)]
                                == id
                    })
                    .count()
                    + [y.saturating_sub(1), y + 1]
                        .into_iter()
                        .filter(|neighbor| {
                            *neighbor < HEIGHT
                                && cells
                                    [usize::from(*neighbor) * usize::from(WIDTH) + usize::from(x)]
                                    == id
                        })
                        .count();
                if best.is_none_or(|(_, _, best_score)| score > best_score) {
                    best = Some((x, y, score));
                }
            }
        }
    }
    best.map(|(x, y, _)| (x, y))
}

fn source_anchor(id: u16, shapes: &[Shape<'_>]) -> Option<(u16, u16)> {
    for shape in shapes.iter().filter(|shape| shape.id == id) {
        for rings in shape.feature.geometry.as_ref()?.polygons() {
            let outer = rings.first()?;
            let (min_lon, max_lon, min_lat, max_lat) = outer.iter().fold(
                (180.0_f64, -180.0_f64, 90.0_f64, -90.0_f64),
                |bounds, point| {
                    (
                        bounds.0.min(point[0]),
                        bounds.1.max(point[0]),
                        bounds.2.min(point[1]),
                        bounds.3.max(point[1]),
                    )
                },
            );
            for row in 0..16 {
                for column in 0..16 {
                    let lon = min_lon + (f64::from(column) + 0.5) * (max_lon - min_lon) / 16.0;
                    let lat = min_lat + (f64::from(row) + 0.5) * (max_lat - min_lat) / 16.0;
                    if contains_polygon(rings, lon, lat) {
                        return Some(project(lon, lat));
                    }
                }
            }
        }
    }
    None
}

fn project(lon: f64, lat: f64) -> (u16, u16) {
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
    use super::{WATER, contains_polygon, mark_transition, point_in_ring};
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
}
