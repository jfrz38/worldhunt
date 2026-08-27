use crate::{raster::RasterData, source::GeometryKind, validation::ValidatedWorldData};
use flate2::{Compression, GzBuilder};
use prost::Message;
use std::{fs, path::Path};

const EXTENT: u32 = 4096;
const NEUTRAL_LAND: u16 = u16::MAX;
const ZOOM_ZERO: [(u32, u32); 1] = [(0, 0)];
const ZOOM_ONE: [(u32, u32); 4] = [(0, 0), (1, 0), (0, 1), (1, 1)];

#[derive(Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(Clone, PartialEq, Message)]
struct Tile {
    #[prost(message, repeated, tag = "3")]
    layers: Vec<Layer>,
}

#[derive(Clone, PartialEq, Message)]
struct Layer {
    #[prost(string, required, tag = "1")]
    name: String,
    #[prost(message, repeated, tag = "2")]
    features: Vec<Feature>,
    #[prost(string, repeated, tag = "3")]
    keys: Vec<String>,
    #[prost(message, repeated, tag = "4")]
    values: Vec<Value>,
    #[prost(uint32, optional, tag = "5")]
    extent: Option<u32>,
    #[prost(uint32, required, tag = "15")]
    version: u32,
}

#[derive(Clone, PartialEq, Message)]
struct Feature {
    #[prost(uint32, repeated, packed = "true", tag = "2")]
    tags: Vec<u32>,
    #[prost(uint32, repeated, packed = "true", tag = "4")]
    geometry: Vec<u32>,
}

#[derive(Clone, PartialEq, Message)]
struct Value {
    #[prost(uint64, optional, tag = "5")]
    uint_value: Option<u64>,
}

pub(crate) fn generate_assets(
    repository_root: &Path,
    validated: &ValidatedWorldData,
    raster: &RasterData,
    check: bool,
) -> Result<usize, String> {
    let root = repository_root.join("assets/country-map-v1");
    let mut generated = 0;
    for (zoom, positions) in [(0, ZOOM_ZERO.as_slice()), (1, ZOOM_ONE.as_slice())] {
        for &(x, y) in positions {
            let bytes = encode_tile(validated, zoom, x, y)?;
            let path = root.join(format!("{zoom}_{x}_{y}.pbf.gz"));
            write_or_check(&path, &bytes, check)?;
            generated += bytes.len();
        }
    }
    let anchors = encode_anchors(raster);
    write_or_check(&root.join("anchors-v1.bin"), &anchors, check)?;
    Ok(generated + anchors.len())
}

fn encode_tile(
    validated: &ValidatedWorldData,
    zoom: u32,
    tile_x: u32,
    tile_y: u32,
) -> Result<Vec<u8>, String> {
    let mut features = Vec::new();
    for feature in &validated.source.features {
        let mapped = validated.catalog.countries.iter().any(|country| {
            country.source_records.iter().any(|selector| {
                feature.properties.iso3.as_deref() == Some(selector.iso3.as_str())
                    && feature.properties.name == selector.name
            })
        });
        if !mapped {
            features.extend(features_for_geometry(
                feature.geometry.as_ref(),
                validated.catalog.countries.len() as u32,
                zoom,
                tile_x,
                tile_y,
            ));
        }
    }
    for (country_id, country) in validated.catalog.countries.iter().enumerate() {
        for feature in &validated.source.features {
            if !country.source_records.iter().any(|selector| {
                feature.properties.iso3.as_deref() == Some(selector.iso3.as_str())
                    && feature.properties.name == selector.name
            }) {
                continue;
            }
            features.extend(features_for_geometry(
                feature.geometry.as_ref(),
                country_id as u32,
                zoom,
                tile_x,
                tile_y,
            ));
        }
    }
    let tile = Tile {
        layers: vec![Layer {
            name: "country".to_owned(),
            features,
            keys: vec!["country_id".to_owned()],
            values: (0..validated.catalog.countries.len())
                .map(|id| Value {
                    uint_value: Some(id as u64),
                })
                .chain(std::iter::once(Value {
                    uint_value: Some(u64::from(NEUTRAL_LAND)),
                }))
                .collect(),
            extent: Some(EXTENT),
            version: 2,
        }],
    };
    let mut encoded = Vec::new();
    tile.encode(&mut encoded)
        .map_err(|error| format!("could not encode country tile: {error}"))?;
    let mut gzip = GzBuilder::new()
        .mtime(0)
        .write(Vec::new(), Compression::best());
    use std::io::Write;
    gzip.write_all(&encoded)
        .map_err(|error| format!("could not compress country tile: {error}"))?;
    gzip.finish()
        .map_err(|error| format!("could not finish country tile: {error}"))
}

fn features_for_geometry(
    geometry: Option<&GeometryKind>,
    value_index: u32,
    zoom: u32,
    tile_x: u32,
    tile_y: u32,
) -> Vec<Feature> {
    geometry
        .into_iter()
        .flat_map(GeometryKind::polygons)
        .filter_map(|polygon| {
            let rings = polygon
                .iter()
                .filter_map(|ring| quantized_ring(ring, zoom, tile_x, tile_y))
                .collect::<Vec<_>>();
            (!rings.is_empty()).then(|| Feature {
                tags: vec![0, value_index],
                geometry: encode_polygon(&rings),
            })
        })
        .collect()
}

fn quantized_ring(
    ring: &[Vec<f64>],
    zoom: u32,
    tile_x: u32,
    tile_y: u32,
) -> Option<Vec<(i32, i32)>> {
    let mut projected = ring
        .iter()
        .map(|position| project(position[0], position[1]))
        .collect::<Vec<_>>();
    unwrap_longitudes(&mut projected);
    let tiles = 2.0_f64.powi(zoom as i32);
    let minimum = Point {
        x: f64::from(tile_x) / tiles,
        y: f64::from(tile_y) / tiles,
    };
    let maximum = Point {
        x: f64::from(tile_x + 1) / tiles,
        y: f64::from(tile_y + 1) / tiles,
    };
    [-1.0, 0.0, 1.0].into_iter().find_map(|offset| {
        let shifted = projected
            .iter()
            .map(|point| Point {
                x: point.x + offset,
                y: point.y,
            })
            .collect::<Vec<_>>();
        let clipped = clip_ring(&shifted, minimum, maximum);
        let mut points = clipped
            .into_iter()
            .map(|point| {
                (
                    ((point.x - minimum.x) * tiles * f64::from(EXTENT))
                        .round()
                        .clamp(0.0, f64::from(EXTENT)) as i32,
                    ((point.y - minimum.y) * tiles * f64::from(EXTENT))
                        .round()
                        .clamp(0.0, f64::from(EXTENT)) as i32,
                )
            })
            .collect::<Vec<_>>();
        points.dedup();
        if points.first() == points.last() {
            points.pop();
        }
        (points.len() >= 3).then_some(points)
    })
}

fn project(longitude: f64, latitude: f64) -> Point {
    let latitude = latitude.clamp(-85.051_128_78, 85.051_128_78).to_radians();
    Point {
        x: (longitude + 180.0) / 360.0,
        y: (1.0 - (latitude.tan() + latitude.cos().recip()).ln() / std::f64::consts::PI) / 2.0,
    }
}

fn unwrap_longitudes(points: &mut [Point]) {
    for index in 1..points.len() {
        while points[index].x - points[index - 1].x > 0.5 {
            points[index].x -= 1.0;
        }
        while points[index].x - points[index - 1].x < -0.5 {
            points[index].x += 1.0;
        }
    }
}

fn clip_ring(points: &[Point], minimum: Point, maximum: Point) -> Vec<Point> {
    let mut output = points.to_vec();
    for edge in [0, 1, 2, 3] {
        let input = std::mem::take(&mut output);
        if input.is_empty() {
            return input;
        }
        let mut previous = *input.last().expect("input is not empty");
        for current in input {
            let previous_inside = inside(previous, edge, minimum, maximum);
            let current_inside = inside(current, edge, minimum, maximum);
            if current_inside != previous_inside {
                output.push(intersection(previous, current, edge, minimum, maximum));
            }
            if current_inside {
                output.push(current);
            }
            previous = current;
        }
    }
    output
}

fn inside(point: Point, edge: u8, minimum: Point, maximum: Point) -> bool {
    match edge {
        0 => point.x >= minimum.x,
        1 => point.x <= maximum.x,
        2 => point.y >= minimum.y,
        _ => point.y <= maximum.y,
    }
}

fn intersection(from: Point, to: Point, edge: u8, minimum: Point, maximum: Point) -> Point {
    if edge < 2 {
        let x = if edge == 0 { minimum.x } else { maximum.x };
        let fraction = (x - from.x) / (to.x - from.x);
        Point {
            x,
            y: from.y + fraction * (to.y - from.y),
        }
    } else {
        let y = if edge == 2 { minimum.y } else { maximum.y };
        let fraction = (y - from.y) / (to.y - from.y);
        Point {
            x: from.x + fraction * (to.x - from.x),
            y,
        }
    }
}

fn encode_polygon(rings: &[Vec<(i32, i32)>]) -> Vec<u32> {
    let mut commands = Vec::new();
    let (mut previous_x, mut previous_y) = (0, 0);
    for ring in rings {
        let &(first_x, first_y) = ring.first().expect("ring is not empty");
        commands.push(1 << 3 | 1);
        commands.push(zigzag(first_x - previous_x));
        commands.push(zigzag(first_y - previous_y));
        previous_x = first_x;
        previous_y = first_y;
        commands.push(((ring.len() as u32 - 1) << 3) | 2);
        for &(x, y) in &ring[1..] {
            commands.push(zigzag(x - previous_x));
            commands.push(zigzag(y - previous_y));
            previous_x = x;
            previous_y = y;
        }
        commands.push(1 << 3 | 7);
    }
    commands
}

fn zigzag(value: i32) -> u32 {
    ((value << 1) ^ (value >> 31)) as u32
}

fn encode_anchors(raster: &RasterData) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(8 + raster.anchors.len() * 8);
    bytes.extend(b"WHCA");
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend((raster.anchors.len() as u16).to_le_bytes());
    for &(x, y) in &raster.anchors {
        let longitude = -180.0 + (f64::from(x) + 0.5) * 360.0 / 720.0;
        let latitude = 90.0 - (f64::from(y) + 0.5) * 150.0 / 300.0;
        let point = project(longitude, latitude);
        bytes.extend(((point.x * 1_000_000_000.0).round() as u32).to_le_bytes());
        bytes.extend(((point.y * 1_000_000_000.0).round() as u32).to_le_bytes());
    }
    bytes
}

fn write_or_check(path: &Path, bytes: &[u8], check: bool) -> Result<(), String> {
    if check {
        let committed = fs::read(path).map_err(|error| {
            format!(
                "{}: could not read generated asset: {error}",
                path.display()
            )
        })?;
        if committed != bytes {
            return Err(format!(
                "{} is stale; run cargo run -p world-data -- generate",
                path.display()
            ));
        }
        return Ok(());
    }
    fs::create_dir_all(path.parent().expect("asset has a parent"))
        .map_err(|error| format!("could not create asset directory: {error}"))?;
    fs::write(path, bytes)
        .map_err(|error| format!("{}: could not write asset: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::encode_tile;
    use crate::validation::load_validated_repository;
    use std::path::Path;

    #[test]
    fn generates_a_country_layer_for_each_zoom_level() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .expect("world-data should remain below the repository root");
        let validated = load_validated_repository(root).expect("committed data should validate");

        for (zoom, x, y) in [(0, 0, 0), (1, 0, 0), (1, 1, 0), (1, 0, 1), (1, 1, 1)] {
            let bytes = encode_tile(&validated, zoom, x, y).expect("tile should encode");
            let tile = crate::mvt::decode(&bytes).expect("tile should decode");

            assert_eq!(tile.layers.len(), 1);
            assert_eq!(tile.layers[0].name, "country");
            assert!(!tile.layers[0].features.is_empty());
        }
    }
}
