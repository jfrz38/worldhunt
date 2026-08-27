use crate::{
    catalog::Catalog,
    raster::{self, HEIGHT, NEUTRAL_LAND, WATER, WIDTH},
    validation,
};
use std::{
    fs,
    io::{IsTerminal, stdout},
    path::Path,
};
use terminal_size::{Height, Width, terminal_size};

const MAGIC: [u8; 4] = *b"WHMP";
const VERSION: u16 = 1;
const HEADER_LENGTH: usize = 32;

pub fn generate_asset(repository_root: &Path, check: bool) -> Result<String, String> {
    let validated = validation::load_validated_repository(repository_root)?;
    let raster = raster::rasterize(&validated)?;
    let bytes = encode(
        &raster.cells,
        &raster.borders,
        &raster.anchors,
        validated.catalog.countries.len() as u16,
    );
    let path = repository_root.join("assets/world-v1.bin");
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
            "{} is current ({} bytes)",
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
            "generated {} ({} bytes, {}x{} raster)",
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

    Ok(render_tile_dots(&dots, dot_width, preview_columns, preview_rows, color))
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

fn render_preview(decoded: &DecodedAsset, columns: usize, rows: usize, color: bool) -> String {
    let rows = rows.saturating_sub(1).max(1);
    let map_width = usize::from(decoded.width) * 2;
    let map_height = usize::from(decoded.height);
    let preview_rows = rows.min(columns * map_height / map_width).max(1);
    let preview_columns = (preview_rows * map_width / map_height).min(columns).max(1);
    let mut preview = String::new();
    for row in 0..preview_rows {
        for column in 0..preview_columns {
            if color {
                let id = preview_cell(decoded, column, row, preview_columns, preview_rows);
                let background = preview_fill_color(id);
                let glyph = braille_glyph(preview_braille_mask(
                    decoded,
                    column,
                    row,
                    preview_columns,
                    preview_rows,
                ));
                preview.push_str(&format!(
                    "\x1b[38;2;{};{};{};48;2;{};{};{}m{}",
                    BORDER_COLOR.0,
                    BORDER_COLOR.1,
                    BORDER_COLOR.2,
                    background.0,
                    background.1,
                    background.2,
                    glyph,
                ));
            } else {
                let top = preview_cell(decoded, column, row * 2, preview_columns, preview_rows * 2);
                let bottom = preview_cell(
                    decoded,
                    column,
                    row * 2 + 1,
                    preview_columns,
                    preview_rows * 2,
                );
                preview.push(preview_glyph(top, bottom));
            }
        }
        if color {
            preview.push_str("\x1b[0m");
        }
        preview.push('\n');
    }
    preview
}

fn render_vector_preview(
    decoded: &DecodedAsset,
    source: &crate::source::FeatureCollection,
    catalog: &Catalog,
    columns: usize,
    rows: usize,
    color: bool,
) -> String {
    let rows = rows.saturating_sub(1).max(1);
    let map_width = usize::from(decoded.width) * 2;
    let map_height = usize::from(decoded.height);
    let preview_rows = rows.min(columns * map_height / map_width).max(1);
    let preview_columns = (preview_rows * map_width / map_height).min(columns).max(1);
    let dots = vector_fill(source, catalog, preview_columns * 2, preview_rows * 4);
    let mut preview = String::new();

    for row in 0..preview_rows {
        for column in 0..preview_columns {
            render_vector_cell(&mut preview, &dots, preview_columns * 2, column, row, color);
        }
        if color {
            preview.push_str("\x1b[0m");
        }
        preview.push('\n');
    }
    preview
}

fn preview_half_block(top: u16, bottom: u16) -> String {
    let foreground = preview_fill_color(top);
    let background = preview_fill_color(bottom);
    format!(
        "\x1b[38;2;{};{};{};48;2;{};{};{}m▀",
        foreground.0, foreground.1, foreground.2, background.0, background.1, background.2,
    )
}

fn preview_average_color(top: u16, bottom: u16) -> (u8, u8, u8) {
    let top = preview_fill_color(top);
    let bottom = preview_fill_color(bottom);
    (
        ((u16::from(top.0) + u16::from(bottom.0)) / 2) as u8,
        ((u16::from(top.1) + u16::from(bottom.1)) / 2) as u8,
        ((u16::from(top.2) + u16::from(bottom.2)) / 2) as u8,
    )
}

fn render_vector_cell(
    preview: &mut String,
    dots: &[u16],
    dot_width: usize,
    column: usize,
    row: usize,
    color: bool,
) {
    let mut counts = std::collections::HashMap::new();
    let mut values = [WATER; 8];
    for dot_y in 0..4 {
        for dot_x in 0..2 {
            let index = dot_y * 2 + dot_x;
            let value = dots[(row * 4 + dot_y) * dot_width + column * 2 + dot_x];
            values[index] = value;
            *counts.entry(value).or_insert(0_usize) += 1;
        }
    }
    let background = counts
        .iter()
        .max_by_key(|(id, count)| (**count, std::cmp::Reverse(**id)))
        .map(|(id, _)| *id)
        .expect("Braille cell contains dots");
    let mut mask = 0;
    let mut foreground = background;
    let mut foreground_count = 0;
    for dot_y in 0..4 {
        for dot_x in 0..2 {
            let index = dot_y * 2 + dot_x;
            let value = values[index];
            if preview_fill_color(value) != preview_fill_color(background) {
                let count = counts[&value];
                if count > foreground_count {
                    foreground = value;
                    foreground_count = count;
                }
                if value == foreground {
                    mask |= BRAILLE_DOTS[dot_y][dot_x];
                }
            }
        }
    }
    let political_border = political_border_mask(&values);
    if mask == 0 && political_border != 0 {
        mask = political_border;
        foreground = WATER;
    }

    if mask == 0 {
        let top = dominant_pair(values[0], values[1]);
        let bottom = dominant_pair(values[6], values[7]);
        if color {
            preview.push_str(&preview_half_block(top, bottom));
        } else {
            preview.push(preview_glyph(top, bottom));
        }
    } else if color {
        let foreground = if foreground == WATER {
            BORDER_COLOR
        } else {
            preview_fill_color(foreground)
        };
        let background = preview_fill_color(background);
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

fn dominant_pair(left: u16, right: u16) -> u16 {
    left.min(right)
}

fn political_border_mask(values: &[u16; 8]) -> u8 {
    let mut mask = 0;
    for dot_y in 0..4 {
        for dot_x in 0..2 {
            let index = dot_y * 2 + dot_x;
            let value = values[index];
            if value == WATER {
                continue;
            }
            let different_right = dot_x == 0 && values[index + 1] != value && values[index + 1] != WATER;
            let different_down = dot_y < 3 && values[index + 2] != value && values[index + 2] != WATER;
            if different_right || different_down {
                mask |= BRAILLE_DOTS[dot_y][dot_x];
            }
        }
    }
    mask
}

fn vector_fill(
    source: &crate::source::FeatureCollection,
    catalog: &Catalog,
    dot_width: usize,
    dot_height: usize,
) -> Vec<u16> {
    let mut dots = vec![WATER; dot_width * dot_height];
    for feature in &source.features {
        let Some(geometry) = &feature.geometry else {
            continue;
        };
        let id = feature_id(feature, catalog);
        for polygon in geometry.polygons() {
            rasterize_polygon(&mut dots, dot_width, dot_height, polygon, id);
        }
    }
    dots
}

fn feature_id(feature: &crate::source::Feature, catalog: &Catalog) -> u16 {
    let Some(iso3) = &feature.properties.iso3 else {
        return WATER;
    };
    catalog
        .countries
        .iter()
        .position(|country| {
            country.source_records.iter().any(|selector| {
                selector.iso3 == *iso3 && selector.name == feature.properties.name
            })
        })
        .map(|index| index as u16)
        .unwrap_or(NEUTRAL_LAND)
}

fn rasterize_polygon(
    dots: &mut [u16],
    dot_width: usize,
    dot_height: usize,
    polygon: &[Vec<Vec<f64>>],
    id: u16,
) {
    let rings: Vec<_> = polygon.iter().map(|ring| project_ring(ring)).collect();
    for shift in [-360.0, 0.0, 360.0] {
        let projected: Vec<_> = rings
            .iter()
            .map(|ring| {
                ring.iter()
                    .map(|(longitude, latitude)| {
                        (
                            (longitude + shift + 180.0) / 360.0 * dot_width as f64,
                            (90.0 - latitude) / 150.0 * dot_height as f64,
                        )
                    })
                    .collect()
            })
            .collect();
        rasterize_projected_polygon(dots, dot_width, dot_height, &projected, id);
    }
}

fn rasterize_projected_polygon(
    dots: &mut [u16],
    dot_width: usize,
    dot_height: usize,
    rings: &[Vec<(f64, f64)>],
    id: u16,
) {
    let Some(outer) = rings.first() else {
        return;
    };
    if polygon_area(outer) < 2.0 {
        return;
    }
    let min_y = rings
        .iter()
        .flatten()
        .map(|(_, y)| *y)
        .fold(f64::INFINITY, f64::min)
        .floor()
        .max(0.0) as usize;
    let max_y = rings
        .iter()
        .flatten()
        .map(|(_, y)| *y)
        .fold(f64::NEG_INFINITY, f64::max)
        .ceil()
        .min(dot_height as f64) as usize;
    for y in min_y..max_y {
        let mut intersections = rings
            .iter()
            .flat_map(|ring| ring_intersections(ring, y as f64 + 0.5))
            .collect::<Vec<_>>();
        intersections.sort_by(f64::total_cmp);
        for pair in intersections.chunks_exact(2) {
            let start = (pair[0] - 0.5).ceil().max(0.0) as usize;
            let end = (pair[1] - 0.5).ceil().min(dot_width as f64) as usize;
            for x in start..end {
                let dot = &mut dots[y * dot_width + x];
                *dot = (*dot).min(id);
            }
        }
    }
}

fn project_ring(ring: &[Vec<f64>]) -> Vec<(f64, f64)> {
    let mut longitude = ring[0][0];
    let mut points = vec![(longitude, ring[0][1])];
    for position in &ring[1..] {
        let mut next_longitude = position[0];
        while next_longitude - longitude > 180.0 {
            next_longitude -= 360.0;
        }
        while longitude - next_longitude > 180.0 {
            next_longitude += 360.0;
        }
        longitude = next_longitude;
        points.push((longitude, position[1]));
    }
    points
}

fn polygon_area(ring: &[(f64, f64)]) -> f64 {
    ring.windows(2)
        .map(|edge| edge[0].0 * edge[1].1 - edge[1].0 * edge[0].1)
        .sum::<f64>()
        .abs()
        / 2.0
}

fn ring_intersections(ring: &[(f64, f64)], scanline: f64) -> Vec<f64> {
    ring.windows(2)
        .filter_map(|edge| {
            let ((x0, y0), (x1, y1)) = (edge[0], edge[1]);
            ((y0 > scanline) != (y1 > scanline))
                .then(|| x0 + (scanline - y0) * (x1 - x0) / (y1 - y0))
        })
        .collect()
}

fn preview_cell(
    decoded: &DecodedAsset,
    column: usize,
    row: usize,
    columns: usize,
    rows: usize,
) -> u16 {
    let width = usize::from(decoded.width);
    let x_start = column * width / columns;
    let x_end = ((column + 1) * width / columns).max(x_start + 1);
    let y_start = row * usize::from(decoded.height) / rows;
    let y_end = ((row + 1) * usize::from(decoded.height) / rows).max(y_start + 1);
    let mut coverage = std::collections::HashMap::new();
    for y in y_start..y_end {
        for x in x_start..x_end {
            *coverage
                .entry(decoded.cells[y * width + x])
                .or_insert(0_usize) += 1;
        }
    }
    coverage
        .into_iter()
        .max_by_key(|(id, count)| (*count, std::cmp::Reverse(*id)))
        .map(|(id, _)| id)
        .expect("preview sample always contains a source cell")
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

fn preview_braille_mask(
    decoded: &DecodedAsset,
    column: usize,
    row: usize,
    columns: usize,
    rows: usize,
) -> u8 {
    const DOTS: [[u8; 2]; 4] = [[0x01, 0x08], [0x02, 0x10], [0x04, 0x20], [0x40, 0x80]];
    let width = usize::from(decoded.width);
    let height = usize::from(decoded.height);
    let mut mask = 0;
    for (dot_y, dots) in DOTS.iter().enumerate() {
        for (dot_x, bit) in dots.iter().enumerate() {
            let x = ((column * 2 + dot_x) * width + columns) / (columns * 2);
            let y = ((row * 4 + dot_y) * height + rows * 2) / (rows * 4);
            let x = x.min(width - 1);
            let y = y.min(height - 1);
            if decoded.borders[y * width + x] != 0 {
                mask |= *bit;
            }
        }
    }
    mask
}

fn braille_glyph(mask: u8) -> char {
    char::from_u32(0x2800 + u32::from(mask)).expect("Braille mask is valid Unicode")
}

fn preview_glyph(top: u16, bottom: u16) -> char {
    if top == WATER && bottom != WATER {
        return '▄';
    }
    if top != WATER && bottom == WATER {
        return '▀';
    }
    let id = if top == WATER { bottom } else { top };
    if id == WATER {
        ' '
    } else if id == NEUTRAL_LAND {
        '░'
    } else {
        '▓'
    }
}

pub(crate) struct DecodedAsset {
    pub(crate) width: u16,
    pub(crate) height: u16,
    pub(crate) cells: Vec<u16>,
    pub(crate) borders: Vec<u8>,
}

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
    let expected_cells = usize::from(width)
        .checked_mul(usize::from(height))
        .and_then(|count| count.checked_mul(2))
        .ok_or("asset dimensions overflow")?;
    if cell_length != expected_cells
        || border_length != expected_cells / 2
        || anchor_length != usize::from(country_count) * 4
    {
        return Err("asset section lengths are inconsistent".to_owned());
    }
    let total = HEADER_LENGTH
        .checked_add(cell_length)
        .and_then(|n| n.checked_add(border_length))
        .and_then(|n| n.checked_add(anchor_length))
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
    if country_count
        != validation::load_validated_repository(repository_root)?
            .catalog
            .countries
            .len() as u16
    {
        return Err("asset country count does not match the catalog".to_owned());
    }
    Ok(DecodedAsset {
        width,
        height,
        cells,
        borders,
    })
}

fn encode(cells: &[u16], borders: &[u8], anchors: &[(u16, u16)], country_count: u16) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(HEADER_LENGTH + cells.len() * 3 + anchors.len() * 4);
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
    bytes.extend([0; 4]);
    for cell in cells {
        put_u16(&mut bytes, *cell);
    }
    bytes.extend(borders);
    for (x, y) in anchors {
        put_u16(&mut bytes, *x);
        put_u16(&mut bytes, *y);
    }
    bytes
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
mod tests {
    use super::{
        DecodedAsset, braille_glyph, decode, encode, preview_braille_mask, render_preview,
    };
    use crate::raster::{HEIGHT, NEUTRAL_LAND, WATER, WIDTH};
    use std::path::Path;
    #[test]
    fn rejects_an_invalid_magic() {
        assert!(decode(b"bad", Path::new(".")).is_err());
    }
    #[test]
    fn encodes_the_expected_header_size() {
        assert_eq!(
            encode(
                &vec![0; usize::from(WIDTH) * usize::from(HEIGHT)],
                &vec![0; usize::from(WIDTH) * usize::from(HEIGHT)],
                &[(0, 0)],
                1
            )
            .len(),
            32 + usize::from(WIDTH) * usize::from(HEIGHT) * 3 + 4
        );
    }
    #[test]
    fn renders_a_colored_responsive_unicode_preview() {
        let decoded = DecodedAsset {
            width: 4,
            height: 2,
            cells: vec![WATER, NEUTRAL_LAND, 0, 0, WATER, NEUTRAL_LAND, 0, 0],
            borders: vec![0, 0, 1, 0, 0, 0, 0, 0],
        };
        let color = render_preview(&decoded, 8, 3, true);
        assert!(
            color
                .chars()
                .any(|glyph| (0x2800..=0x28ff).contains(&u32::from(glyph)))
        );
        assert!(color.contains("\x1b[38;2;"));
        assert_eq!(color.lines().count(), 2);
        let monochrome = render_preview(&decoded, 8, 3, false);
        assert!(monochrome.contains('░'));
        assert!(!monochrome.contains("\x1b["));
    }

    #[test]
    fn encodes_border_samples_as_braille_dots() {
        let decoded = DecodedAsset {
            width: 2,
            height: 4,
            cells: vec![0; 8],
            borders: vec![1, 0, 0, 0, 0, 0, 0, 1],
        };

        let mask = preview_braille_mask(&decoded, 0, 0, 1, 1);
        assert_eq!(mask, 0x81);
        assert_eq!(u32::from(braille_glyph(mask)), 0x2881);
    }

    #[test]
    fn renders_both_halves_in_monochrome() {
        let decoded = DecodedAsset {
            width: 1,
            height: 2,
            cells: vec![WATER, 0],
            borders: vec![0, 0],
        };

        assert_eq!(render_preview(&decoded, 2, 2, false), "▄\n");
    }

    #[test]
    fn committed_asset_uses_the_cropped_dimensions() {
        let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .expect("world-data should remain below the repository root");
        let decoded = decode(
            include_bytes!("../../../assets/world-v1.bin"),
            repository_root,
        )
        .expect("committed asset should decode");
        assert_eq!((decoded.width, decoded.height), (WIDTH, HEIGHT));
    }
}
