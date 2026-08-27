use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
};

use super::mvt::{self, Tile};

const BRAILLE_DOTS: [[u8; 2]; 4] = [[0x01, 0x08], [0x02, 0x10], [0x04, 0x20], [0x40, 0x80]];
const WATER: u8 = 1;
const BORDER: u8 = 2;
const NORTH: f64 = 0.0307;
const SOUTH: f64 = 0.6887;
const LAND_COLOR: Color = Color::Rgb(93, 105, 112);
const WATER_COLOR: Color = Color::Rgb(8, 24, 42);
const BORDER_COLOR: Color = Color::Rgb(37, 48, 56);

pub(super) struct Map {
    zoom: f64,
    center_x: f64,
    center_y: f64,
    zoom_zero: Tile,
    zoom_one: [Tile; 4],
}

impl Map {
    pub(super) fn load() -> Result<Self, String> {
        Ok(Self {
            zoom: 0.0,
            center_x: 0.5,
            center_y: (NORTH + SOUTH) / 2.0,
            zoom_zero: mvt::decode(include_bytes!("../../../data/source/openstreetmap/0_0_0.pbf.gz"))?,
            zoom_one: [
                mvt::decode(include_bytes!("../../../data/source/openstreetmap/1_0_0.pbf.gz"))?,
                mvt::decode(include_bytes!("../../../data/source/openstreetmap/1_1_0.pbf.gz"))?,
                mvt::decode(include_bytes!("../../../data/source/openstreetmap/1_0_1.pbf.gz"))?,
                mvt::decode(include_bytes!("../../../data/source/openstreetmap/1_1_1.pbf.gz"))?,
            ],
        })
    }

    pub(super) fn zoom_in(&mut self) {
        self.zoom = (self.zoom + 0.25).min(1.99);
    }

    pub(super) fn zoom_out(&mut self) {
        self.zoom = (self.zoom - 0.25).max(0.0);
    }

    pub(super) fn pan(&mut self, horizontal: f64, vertical: f64) {
        let step = 0.08 / 2.0_f64.powf(self.zoom);
        self.center_x = (self.center_x + horizontal * step).rem_euclid(1.0);
        self.center_y = (self.center_y + vertical * step).clamp(NORTH, SOUTH);
    }

    pub(super) fn render(&self, area: Rect, buffer: &mut Buffer) {
        if area.width < 2 || area.height < 2 {
            return;
        }
        let columns = usize::from(area.width);
        let rows = usize::from(area.height.saturating_sub(1));
        let dot_width = columns * 2;
        let dot_height = rows * 4;
        let mut dots = vec![0_u8; dot_width * dot_height];
        let scale = (dot_width as f64).min(dot_height as f64 / (SOUTH - NORTH)) * 2.0_f64.powf(self.zoom);

        for (tile, tile_x, tile_y, zoom) in self.active_tiles() {
            draw_tile(&mut dots, dot_width, dot_height, tile, tile_x, tile_y, zoom, self.center_x, self.center_y, scale);
        }
        render_dots(&dots, dot_width, area, rows, buffer);
        render_status(area, buffer, self.zoom);
    }

    fn active_tiles(&self) -> Vec<(&Tile, u32, u32, u32)> {
        if self.zoom < 1.0 {
            vec![(&self.zoom_zero, 0, 0, 0)]
        } else {
            vec![
                (&self.zoom_one[0], 0, 0, 1),
                (&self.zoom_one[1], 1, 0, 1),
                (&self.zoom_one[2], 0, 1, 1),
                (&self.zoom_one[3], 1, 1, 1),
            ]
        }
    }
}

fn draw_tile(dots: &mut [u8], width: usize, height: usize, tile: &Tile, tile_x: u32, tile_y: u32, zoom: u32, center_x: f64, center_y: f64, scale: f64) {
    for layer in &tile.layers {
        let extent = layer.extent.unwrap_or(4096);
        if layer.name == "water" {
            for feature in &layer.features {
                let rings = mvt::decode_geometry(&feature.geometry);
                fill_polygon(dots, width, height, extent, &rings, tile_x, tile_y, zoom, center_x, center_y, scale);
            }
        }
    }
    for layer in &tile.layers {
        if layer.name != "boundary" {
            continue;
        }
        let extent = layer.extent.unwrap_or(4096);
        for feature in &layer.features {
            if mvt::unsigned_property(layer, feature, "admin_level") != Some(2)
                || mvt::boolean_property(layer, feature, "maritime") == Some(true)
            {
                continue;
            }
            for path in mvt::decode_geometry(&feature.geometry) {
                let path = project_path(&path, extent, tile_x, tile_y, zoom, width, height, center_x, center_y, scale);
                for edge in path.windows(2) {
                    draw_line(dots, width, height, edge[0], edge[1], BORDER);
                }
            }
        }
    }
}

fn fill_polygon(dots: &mut [u8], width: usize, height: usize, extent: u32, rings: &[Vec<(i32, i32)>], tile_x: u32, tile_y: u32, zoom: u32, center_x: f64, center_y: f64, scale: f64) {
    let rings: Vec<_> = rings.iter().map(|ring| project_path(ring, extent, tile_x, tile_y, zoom, width, height, center_x, center_y, scale)).filter(|ring| ring.len() >= 3).collect();
    for y in 0..height {
        let mut intersections = rings.iter().flat_map(|ring| ring_intersections(ring, y as f64 + 0.5)).collect::<Vec<_>>();
        intersections.sort_by(f64::total_cmp);
        for pair in intersections.chunks_exact(2) {
            let start = pair[0].ceil().max(0.0) as usize;
            let end = pair[1].ceil().min(width as f64) as usize;
            for x in start..end {
                dots[y * width + x] = WATER;
            }
        }
    }
}

fn project_path(path: &[(i32, i32)], extent: u32, tile_x: u32, tile_y: u32, zoom: u32, width: usize, height: usize, center_x: f64, center_y: f64, scale: f64) -> Vec<(i32, i32)> {
    let tiles = 2.0_f64.powi(zoom as i32);
    let extent = f64::from(extent);
    path.iter().map(|&(x, y)| {
        let world_x = (f64::from(tile_x) + f64::from(x) / extent) / tiles;
        let world_y = (f64::from(tile_y) + f64::from(y) / extent) / tiles;
        ((width as f64 / 2.0 + (world_x - center_x) * scale).round() as i32, (height as f64 / 2.0 + (world_y - center_y) * scale).round() as i32)
    }).collect()
}

fn ring_intersections(ring: &[(i32, i32)], scanline: f64) -> Vec<f64> {
    ring.windows(2).filter_map(|edge| {
        let ((x0, y0), (x1, y1)) = (edge[0], edge[1]);
        ((f64::from(y0) > scanline) != (f64::from(y1) > scanline)).then(|| f64::from(x0) + (scanline - f64::from(y0)) * f64::from(x1 - x0) / f64::from(y1 - y0))
    }).collect()
}

fn draw_line(dots: &mut [u8], width: usize, height: usize, from: (i32, i32), to: (i32, i32), color: u8) {
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
        if x == x1 && y == y1 { return; }
        let doubled = error * 2;
        if doubled >= dy { error += dy; x += sx; }
        if doubled <= dx { error += dx; y += sy; }
    }
}

fn render_dots(dots: &[u8], dot_width: usize, area: Rect, rows: usize, buffer: &mut Buffer) {
    for row in 0..rows {
        for column in 0..usize::from(area.width) {
            let mut mask = 0;
            let mut water = 0;
            let mut border = 0;
            for dot_y in 0..4 {
                for dot_x in 0..2 {
                    let value = dots[(row * 4 + dot_y) * dot_width + column * 2 + dot_x];
                    if value != 0 { mask |= BRAILLE_DOTS[dot_y][dot_x]; }
                    water += usize::from(value == WATER);
                    border += usize::from(value == BORDER);
                }
            }
            let foreground = if border > water { BORDER_COLOR } else { WATER_COLOR };
            buffer[(area.x + column as u16, area.y + row as u16)]
                .set_symbol(&braille(mask))
                .set_style(Style::default().fg(foreground).bg(LAND_COLOR));
        }
    }
}

fn render_status(area: Rect, buffer: &mut Buffer, zoom: f64) {
    let status = format!("  Zoom {zoom:.2}  |  +/- zoom  Arrows/hjkl pan  Esc quit");
    for (index, character) in status.chars().take(usize::from(area.width)).enumerate() {
        buffer[(area.x + index as u16, area.y + area.height - 1)]
            .set_symbol(&character.to_string())
            .set_style(Style::default().fg(Color::Gray).bg(Color::Black));
    }
}

fn braille(mask: u8) -> String {
    char::from_u32(0x2800 + u32::from(mask)).expect("valid Braille mask").to_string()
}
