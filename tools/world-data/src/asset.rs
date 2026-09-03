use crate::{
    raster::{self, HEIGHT, NEUTRAL_LAND, WATER, WIDTH},
    validation,
};
use std::{
    fs,
    io::{IsTerminal, stdout},
    path::Path,
    time::Instant,
};
use terminal_size::{Height, Width, terminal_size};

const MAGIC: [u8; 4] = *b"WHMP";
const VERSION: u16 = 2;
const HEADER_LENGTH: usize = 36;

pub fn generate_asset(repository_root: &Path, check: bool) -> Result<String, String> {
    let validated = validation::load_validated_repository(repository_root)?;
    let started = Instant::now();
    let proximity = crate::proximity::generate(&validated)?;
    let proximity_summary = format!(
        "{} adjacent pairs, {} km maximum distance, {} boundary samples in {} ms",
        proximity.adjacent_pair_count(),
        proximity.maximum_distance_km(),
        proximity.boundary_point_count(),
        started.elapsed().as_millis()
    );
    let raster = raster::rasterize(&validated)?;
    if proximity.country_count() != validated.catalog.countries.len() {
        return Err("proximity matrix country count does not match the catalog".to_owned());
    }
    let bytes = encode(
        &raster.cells,
        &raster.borders,
        &raster.anchors,
        validated.catalog.countries.len() as u16,
        proximity.distances_km(),
        proximity.adjacency(),
    )?;
    let country_tiles_length =
        crate::country_tiles::generate_assets(repository_root, &validated, &raster, check)?;
    let (details_length, island_count) =
        crate::details::generate_asset(repository_root, &validated, check)?;
    let path = repository_root.join("assets/world-v2.bin");
    if check {
        let committed = fs::read(&path).map_err(|error| {
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
        Ok(format!(
            "{} and country tiles/map details are current ({} + {country_tiles_length} bytes, {island_count} Canary Island polygons; {proximity_summary})",
            path.display(),
            bytes.len()
        ))
    } else {
        fs::create_dir_all(path.parent().expect("asset has a parent"))
            .map_err(|error| format!("could not create assets directory: {error}"))?;
        fs::write(&path, &bytes).map_err(|error| {
            format!(
                "{}: could not write generated asset: {error}",
                path.display()
            )
        })?;
        Ok(format!(
            "generated {} and country tiles/map details ({} + {country_tiles_length} + {details_length} bytes, {}x{} raster; {proximity_summary})",
            path.display(),
            bytes.len(),
            WIDTH,
            HEIGHT
        ))
    }
}

pub fn verify_asset(repository_root: &Path, bytes: &[u8]) -> Result<(), String> {
    decode(bytes, repository_root).map(|_| ())
}

pub fn preview_asset(repository_root: &Path) -> Result<String, String> {
    let (columns, rows) = preview_size();
    render_zoom_zero_preview(
        repository_root,
        columns,
        rows,
        stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none(),
    )
}

fn preview_size() -> (usize, usize) {
    terminal_size()
        .map(|(Width(columns), Height(rows))| (usize::from(columns), usize::from(rows)))
        .or_else(|| {
            let columns = std::env::var("COLUMNS").ok()?.parse().ok()?;
            let rows = std::env::var("LINES").ok()?.parse().ok()?;
            Some((columns, rows))
        })
        .unwrap_or((120, 30))
}

fn render_zoom_zero_preview(
    repository_root: &Path,
    columns: usize,
    rows: usize,
    color: bool,
) -> Result<String, String> {
    let bytes = fs::read(repository_root.join("data/source/openstreetmap/0_0_0.pbf.gz"))
        .map_err(|error| format!("could not read zoom-0 vector tile: {error}"))?;
    let tile = crate::mvt::decode(&bytes)?;
    let rows = rows.saturating_sub(1).max(1);
    let preview_rows = rows;
    let preview_columns = columns;
    let dot_width = preview_columns * 2;
    let dot_height = preview_rows * 4;
    let mut dots = vec![0_u8; dot_width * dot_height];

    for layer in &tile.layers {
        let extent = layer.extent.unwrap_or(4096);
        if layer.name == "water" {
            for feature in &layer.features {
                let rings = crate::mvt::decode_geometry(&feature.geometry);
                fill_tile_polygon(&mut dots, dot_width, dot_height, extent, &rings, 1);
            }
        }
    }
    for layer in &tile.layers {
        if layer.name != "boundary" {
            continue;
        }
        let extent = layer.extent.unwrap_or(4096);
        for feature in &layer.features {
            for path in crate::mvt::decode_geometry(&feature.geometry) {
                draw_tile_path(&mut dots, dot_width, dot_height, extent, &path, 2);
            }
        }
    }

    Ok(render_tile_dots(
        &dots,
        dot_width,
        preview_columns,
        preview_rows,
        color,
    ))
}

fn fill_tile_polygon(
    dots: &mut [u8],
    dot_width: usize,
    dot_height: usize,
    extent: u32,
    rings: &[Vec<(i32, i32)>],
    color: u8,
) {
    let rings: Vec<_> = rings
        .iter()
        .map(|ring| project_tile_path(ring, extent, dot_width, dot_height))
        .filter(|ring| ring.len() >= 3)
        .collect();
    for y in 0..dot_height {
        let mut intersections = rings
            .iter()
            .flat_map(|ring| tile_ring_intersections(ring, y as f64 + 0.5))
            .collect::<Vec<_>>();
        intersections.sort_by(f64::total_cmp);
        for pair in intersections.chunks_exact(2) {
            let start = pair[0].ceil().max(0.0) as usize;
            let end = pair[1].ceil().min(dot_width as f64) as usize;
            for x in start..end {
                dots[y * dot_width + x] = color;
            }
        }
    }
}

fn draw_tile_path(
    dots: &mut [u8],
    dot_width: usize,
    dot_height: usize,
    extent: u32,
    path: &[(i32, i32)],
    color: u8,
) {
    let path = project_tile_path(path, extent, dot_width, dot_height);
    for edge in path.windows(2) {
        draw_tile_line(dots, dot_width, dot_height, edge[0], edge[1], color);
    }
}

fn project_tile_path(
    path: &[(i32, i32)],
    extent: u32,
    dot_width: usize,
    dot_height: usize,
) -> Vec<(i32, i32)> {
    let extent = f64::from(extent);
    let north = web_mercator_y(84.0);
    let south = web_mercator_y(-56.0);
    let scale = (dot_width as f64 / extent).min(dot_height as f64 / ((south - north) * extent));
    let x_offset = dot_width as f64 / 2.0 - extent * scale / 2.0;
    let y_offset = dot_height as f64 / 2.0 - (south - north) * extent * scale / 2.0;
    path.iter()
        .map(|&(x, y)| {
            (
                (x_offset + f64::from(x) * scale).round() as i32,
                (y_offset + (f64::from(y) - north * extent) * scale).round() as i32,
            )
        })
        .collect()
}

fn web_mercator_y(latitude: f64) -> f64 {
    let latitude = latitude.to_radians();
    (1.0 - (latitude.tan() + latitude.cos().recip()).ln() / std::f64::consts::PI) / 2.0
}

fn tile_ring_intersections(ring: &[(i32, i32)], scanline: f64) -> Vec<f64> {
    ring.windows(2)
        .filter_map(|edge| {
            let ((x0, y0), (x1, y1)) = (edge[0], edge[1]);
            ((f64::from(y0) > scanline) != (f64::from(y1) > scanline)).then(|| {
                f64::from(x0) + (scanline - f64::from(y0)) * f64::from(x1 - x0) / f64::from(y1 - y0)
            })
        })
        .collect()
}

fn draw_tile_line(
    dots: &mut [u8],
    width: usize,
    height: usize,
    from: (i32, i32),
    to: (i32, i32),
    color: u8,
) {
    let (mut x, mut y) = from;
    let (x1, y1) = to;
    let dx = (x1 - x).abs();
    let dy = -(y1 - y).abs();
    let sx = if x < x1 { 1 } else { -1 };
    let sy = if y < y1 { 1 } else { -1 };
    let mut error = dx + dy;
    loop {
        if x >= 0 && y >= 0 && (x as usize) < width && (y as usize) < height {
            dots[y as usize * width + x as usize] = color;
        }
        if x == x1 && y == y1 {
            return;
        }
        let doubled = error * 2;
        if doubled >= dy {
            error += dy;
            x += sx;
        }
        if doubled <= dx {
            error += dx;
            y += sy;
        }
    }
}

fn render_tile_dots(
    dots: &[u8],
    dot_width: usize,
    columns: usize,
    rows: usize,
    color: bool,
) -> String {
    let mut preview = String::new();
    for row in 0..rows {
        for column in 0..columns {
            let mut mask = 0;
            let mut water_dots = 0;
            let mut border_dots = 0;
            for dot_y in 0..4 {
                for dot_x in 0..2 {
                    let value = dots[(row * 4 + dot_y) * dot_width + column * 2 + dot_x];
                    if value != 0 {
                        mask |= BRAILLE_DOTS[dot_y][dot_x];
                    }
                    water_dots += usize::from(value == 1);
                    border_dots += usize::from(value == 2);
                }
            }
            if color {
                let foreground = if border_dots > water_dots {
                    BORDER_COLOR
                } else {
                    preview_fill_color(WATER)
                };
                let background = preview_fill_color(NEUTRAL_LAND);
                preview.push_str(&format!(
                    "\x1b[38;2;{};{};{};48;2;{};{};{}m{}",
                    foreground.0,
                    foreground.1,
                    foreground.2,
                    background.0,
                    background.1,
                    background.2,
                    braille_glyph(mask),
                ));
            } else {
                preview.push(braille_glyph(mask));
            }
        }
        if color {
            preview.push_str("\x1b[0m");
        }
        preview.push('\n');
    }
    preview
}

const BORDER_COLOR: (u8, u8, u8) = (37, 48, 56);
const BRAILLE_DOTS: [[u8; 2]; 4] = [[0x01, 0x08], [0x02, 0x10], [0x04, 0x20], [0x40, 0x80]];

fn preview_fill_color(id: u16) -> (u8, u8, u8) {
    if id == WATER {
        (8, 24, 42)
    } else if id == NEUTRAL_LAND {
        (93, 105, 112)
    } else {
        (142, 151, 157)
    }
}

fn braille_glyph(mask: u8) -> char {
    char::from_u32(0x2800 + u32::from(mask)).expect("Braille mask is valid Unicode")
}

#[cfg(test)]
pub(crate) struct DecodedAsset {
    pub(crate) width: u16,
    pub(crate) height: u16,
    pub(crate) country_count: u16,
    pub(crate) distances_km: Vec<u16>,
    pub(crate) adjacency: Vec<bool>,
}

#[cfg(not(test))]
pub(crate) struct DecodedAsset;

pub(crate) fn decode(bytes: &[u8], repository_root: &Path) -> Result<DecodedAsset, String> {
    if bytes.len() < HEADER_LENGTH || bytes[0..4] != MAGIC {
        return Err("asset has invalid magic or is truncated".to_owned());
    }
    let version = read_u16(bytes, 4)?;
    if version != VERSION {
        return Err(format!("asset version {version} is unsupported"));
    }
    let width = read_u16(bytes, 6)?;
    let height = read_u16(bytes, 8)?;
    let country_count = read_u16(bytes, 10)?;
    if width == 0 || height == 0 || country_count == 0 {
        return Err("asset has zero dimensions or country count".to_owned());
    }
    if read_u16(bytes, 12)? != WATER || read_u16(bytes, 14)? != NEUTRAL_LAND {
        return Err("asset reserved identifiers are invalid".to_owned());
    }
    let cell_length = read_u32(bytes, 16)? as usize;
    let border_length = read_u32(bytes, 20)? as usize;
    let anchor_length = read_u32(bytes, 24)? as usize;
    let distance_length = read_u32(bytes, 28)? as usize;
    let adjacency_length = read_u32(bytes, 32)? as usize;
    let expected_cells = usize::from(width)
        .checked_mul(usize::from(height))
        .and_then(|count| count.checked_mul(2))
        .ok_or("asset dimensions overflow")?;
    let matrix_entries = usize::from(country_count)
        .checked_mul(usize::from(country_count))
        .ok_or("asset country count overflows")?;
    if cell_length != expected_cells
        || border_length != expected_cells / 2
        || anchor_length != usize::from(country_count) * 4
        || distance_length != matrix_entries * 2
        || adjacency_length != matrix_entries
    {
        return Err("asset section lengths are inconsistent".to_owned());
    }
    let total = HEADER_LENGTH
        .checked_add(cell_length)
        .and_then(|n| n.checked_add(border_length))
        .and_then(|n| n.checked_add(anchor_length))
        .and_then(|n| n.checked_add(distance_length))
        .and_then(|n| n.checked_add(adjacency_length))
        .ok_or("asset length overflow")?;
    if bytes.len() != total {
        return Err("asset length does not match its header".to_owned());
    }
    let cells = (0..cell_length / 2)
        .map(|index| read_u16(bytes, HEADER_LENGTH + index * 2))
        .collect::<Result<Vec<_>, _>>()?;
    if cells
        .iter()
        .any(|id| *id != WATER && *id != NEUTRAL_LAND && *id >= country_count)
    {
        return Err("asset contains an unknown raster identifier".to_owned());
    }
    let borders =
        bytes[HEADER_LENGTH + cell_length..HEADER_LENGTH + cell_length + border_length].to_vec();
    if borders.iter().any(|value| *value > 1) {
        return Err("asset contains an invalid border value".to_owned());
    }
    let anchor_start = HEADER_LENGTH + cell_length + border_length;
    let _anchors = (0..usize::from(country_count))
        .map(|index| {
            let x = read_u16(bytes, anchor_start + index * 4)?;
            let y = read_u16(bytes, anchor_start + index * 4 + 2)?;
            if x >= width || y >= height {
                return Err("asset anchor is out of range".to_owned());
            }
            Ok((x, y))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let distance_start = anchor_start + anchor_length;
    let distances_km = (0..matrix_entries)
        .map(|index| read_u16(bytes, distance_start + index * 2))
        .collect::<Result<Vec<_>, _>>()?;
    let adjacency_start = distance_start + distance_length;
    let adjacency = bytes[adjacency_start..adjacency_start + adjacency_length]
        .iter()
        .map(|value| match value {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err("asset contains an invalid adjacency value".to_owned()),
        })
        .collect::<Result<Vec<_>, _>>()?;
    validate_proximity(&distances_km, &adjacency, usize::from(country_count))?;
    if country_count
        != validation::load_validated_repository(repository_root)?
            .catalog
            .countries
            .len() as u16
    {
        return Err("asset country count does not match the catalog".to_owned());
    }
    #[cfg(test)]
    {
        Ok(DecodedAsset {
            width,
            height,
            country_count,
            distances_km,
            adjacency,
        })
    }
    #[cfg(not(test))]
    {
        let _ = (cells, borders, distances_km, adjacency);
        Ok(DecodedAsset)
    }
}

fn encode(
    cells: &[u16],
    borders: &[u8],
    anchors: &[(u16, u16)],
    country_count: u16,
    distances_km: &[u16],
    adjacency: &[bool],
) -> Result<Vec<u8>, String> {
    let matrix_entries = usize::from(country_count)
        .checked_mul(usize::from(country_count))
        .ok_or("asset country count overflows")?;
    if distances_km.len() != matrix_entries || adjacency.len() != matrix_entries {
        return Err("proximity matrix dimensions do not match the country count".to_owned());
    }
    validate_proximity(distances_km, adjacency, usize::from(country_count))?;
    let mut bytes = Vec::with_capacity(
        HEADER_LENGTH + cells.len() * 3 + anchors.len() * 4 + distances_km.len() * 3,
    );
    bytes.extend(MAGIC);
    put_u16(&mut bytes, VERSION);
    put_u16(&mut bytes, WIDTH);
    put_u16(&mut bytes, HEIGHT);
    put_u16(&mut bytes, country_count);
    put_u16(&mut bytes, WATER);
    put_u16(&mut bytes, NEUTRAL_LAND);
    put_u32(&mut bytes, (cells.len() * 2) as u32);
    put_u32(&mut bytes, borders.len() as u32);
    put_u32(&mut bytes, (anchors.len() * 4) as u32);
    put_u32(&mut bytes, (distances_km.len() * 2) as u32);
    put_u32(&mut bytes, adjacency.len() as u32);
    for cell in cells {
        put_u16(&mut bytes, *cell);
    }
    bytes.extend(borders);
    for (x, y) in anchors {
        put_u16(&mut bytes, *x);
        put_u16(&mut bytes, *y);
    }
    for distance_km in distances_km {
        put_u16(&mut bytes, *distance_km);
    }
    bytes.extend(adjacency.iter().map(|value| u8::from(*value)));
    Ok(bytes)
}

fn validate_proximity(
    distances_km: &[u16],
    adjacency: &[bool],
    country_count: usize,
) -> Result<(), String> {
    let expected_length = country_count
        .checked_mul(country_count)
        .ok_or("asset country count overflows")?;
    if distances_km.len() != expected_length || adjacency.len() != expected_length {
        return Err("asset proximity matrix dimensions are invalid".to_owned());
    }
    for first in 0..country_count {
        let diagonal = first * country_count + first;
        if distances_km[diagonal] != 0 || adjacency[diagonal] {
            return Err("asset proximity matrix diagonal is invalid".to_owned());
        }
        for second in first + 1..country_count {
            let forward = first * country_count + second;
            let reverse = second * country_count + first;
            if distances_km[forward] != distances_km[reverse]
                || adjacency[forward] != adjacency[reverse]
            {
                return Err("asset proximity matrices are not symmetric".to_owned());
            }
            if adjacency[forward] && distances_km[forward] != 0 {
                return Err("asset adjacent countries must have zero separation".to_owned());
            }
        }
    }
    Ok(())
}
fn put_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend(value.to_le_bytes());
}
fn put_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend(value.to_le_bytes());
}
fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, String> {
    bytes
        .get(offset..offset + 2)
        .and_then(|part| part.try_into().ok())
        .map(u16::from_le_bytes)
        .ok_or_else(|| "asset is truncated".to_owned())
}
fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    bytes
        .get(offset..offset + 4)
        .and_then(|part| part.try_into().ok())
        .map(u32::from_le_bytes)
        .ok_or_else(|| "asset is truncated".to_owned())
}

#[cfg(test)]
mod tests;
