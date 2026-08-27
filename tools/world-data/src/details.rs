use crate::{source::Feature, validation::ValidatedWorldData};
use std::{collections::HashSet, fs, path::Path};

const MAGIC: [u8; 4] = *b"WHDL";
const VERSION: u16 = 1;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
struct Point {
    longitude: i32,
    latitude: i32,
}

pub(crate) fn generate_asset(repository_root: &Path, validated: &ValidatedWorldData, check: bool) -> Result<(usize, usize), String> {
    let bytes = encode(validated)?;
    let path = repository_root.join("assets/map-details-v1.bin");
    if check {
        let committed = fs::read(&path)
            .map_err(|error| format!("{}: could not read generated asset: {error}", path.display()))?;
        if committed != bytes {
            return Err(format!("{} is stale; run cargo run -p world-data -- generate", path.display()));
        }
    } else {
        fs::create_dir_all(path.parent().expect("asset has a parent"))
            .map_err(|error| format!("could not create assets directory: {error}"))?;
        fs::write(&path, &bytes)
            .map_err(|error| format!("{}: could not write generated asset: {error}", path.display()))?;
    }
    Ok((bytes.len(), 7))
}

fn encode(validated: &ValidatedWorldData) -> Result<Vec<u8>, String> {
    let spain = feature_for(validated, "ESP")?;
    let morocco = feature_for(validated, "MAR")?;
    let western_sahara = feature_for(validated, "ESH")?;
    let islands = outer_rings(spain)
        .into_iter()
        .filter(|ring| is_canary_island(ring))
        .collect::<Vec<_>>();
    if islands.len() != 7 {
        return Err(format!("expected 7 Canary Island polygons, found {}", islands.len()));
    }
    let border = shared_border(morocco, western_sahara)?;

    let mut bytes = Vec::new();
    bytes.extend(MAGIC);
    write_u16(&mut bytes, VERSION);
    write_u16(&mut bytes, islands.len() as u16);
    for island in islands {
        write_points(&mut bytes, &island)?;
    }
    write_points(&mut bytes, &border)?;
    Ok(bytes)
}

fn feature_for<'a>(validated: &'a ValidatedWorldData, iso3: &str) -> Result<&'a Feature, String> {
    validated.source.features.iter().find(|feature| feature.properties.iso3.as_deref() == Some(iso3))
        .ok_or_else(|| format!("source snapshot has no {iso3} feature"))
}

fn outer_rings(feature: &Feature) -> Vec<Vec<Point>> {
    let Some(geometry) = &feature.geometry else { return Vec::new() };
    geometry.polygons().into_iter().filter_map(|polygon| polygon.first()).map(|ring| points(ring)).collect()
}

fn points(ring: &[Vec<f64>]) -> Vec<Point> {
    ring.iter().map(|position| Point {
        longitude: (position[0] * 1_000_000.0).round() as i32,
        latitude: (position[1] * 1_000_000.0).round() as i32,
    }).collect()
}

fn is_canary_island(ring: &[Point]) -> bool {
    let (min_longitude, max_longitude, min_latitude, max_latitude) = ring.iter().fold(
        (i32::MAX, i32::MIN, i32::MAX, i32::MIN),
        |(min_lon, max_lon, min_lat, max_lat), point| (
            min_lon.min(point.longitude), max_lon.max(point.longitude), min_lat.min(point.latitude), max_lat.max(point.latitude),
        ),
    );
    min_longitude >= -19_000_000
        && max_longitude <= -13_000_000
        && min_latitude >= 27_000_000
        && max_latitude <= 30_000_000
}

fn shared_border(morocco: &Feature, western_sahara: &Feature) -> Result<Vec<Point>, String> {
    let moroccan_edges = outer_rings(morocco).into_iter().flat_map(|ring| ring.windows(2).map(|edge| normalized_edge(edge[0], edge[1])).collect::<Vec<_>>()).collect::<HashSet<_>>();
    let mut border = Vec::new();
    for ring in outer_rings(western_sahara) {
        for edge in ring.windows(2) {
            if !moroccan_edges.contains(&normalized_edge(edge[0], edge[1])) {
                continue;
            }
            if border.is_empty() {
                border.extend(edge);
            } else if border.last() == Some(&edge[0]) {
                border.push(edge[1]);
            } else if border.last() == Some(&edge[1]) {
                border.push(edge[0]);
            } else {
                return Err("Western Sahara shared border is not a single continuous path".to_owned());
            }
        }
    }
    if border.len() != 15 {
        return Err(format!("expected 15 Western Sahara shared-border points, found {}", border.len()));
    }
    Ok(border)
}

fn normalized_edge(first: Point, second: Point) -> (Point, Point) {
    if first.longitude < second.longitude || (first.longitude == second.longitude && first.latitude <= second.latitude) {
        (first, second)
    } else {
        (second, first)
    }
}

fn write_points(bytes: &mut Vec<u8>, points: &[Point]) -> Result<(), String> {
    let last = points.last().ok_or("empty geographic ring")?;
    let points = (points.len() > 1 && points.first() == Some(last)).then(|| &points[..points.len() - 1]).unwrap_or(points);
    let count = u16::try_from(points.len()).map_err(|_| "geographic path has too many points")?;
    write_u16(bytes, count);
    for point in points {
        bytes.extend(point.longitude.to_le_bytes());
        bytes.extend(point.latitude.to_le_bytes());
    }
    Ok(())
}

fn write_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend(value.to_le_bytes());
}
