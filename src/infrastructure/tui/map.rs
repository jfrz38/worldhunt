use ratatui::{buffer::Buffer, layout::Rect, style::Style};

use crate::domain::Guess;

use super::{
    layout,
    mvt::{self, Tile},
    theme::Theme,
};

mod countries;
use countries::CountryOverlay;

const BRAILLE_DOTS: [[u8; 2]; 4] = [[0x01, 0x08], [0x02, 0x10], [0x04, 0x20], [0x40, 0x80]];
const WATER: u8 = 1;
const BORDER: u8 = 2;
const NORTH: f64 = 0.0307;
const SOUTH: f64 = 0.6887;
const NEUTRAL_LAND: u16 = u16::MAX - 1;
const DETAIL_ZOOM: f64 = 0.5;
const INITIAL_ZOOM: f64 = 1.0;
const SPAIN_CENTER_X: f64 = 0.489_711_666_7;
const SPAIN_CENTER_Y: f64 = 0.377_063_141_2;
const OSM_ATTRIBUTION: &str = "OpenStreetMap contributors";

#[derive(Clone, Copy)]
struct Viewport {
    width: usize,
    height: usize,
    center_x: f64,
    center_y: f64,
    scale: f64,
}

#[derive(Clone, Copy)]
struct TilePosition {
    x: u32,
    y: u32,
    zoom: u32,
}

pub(super) struct Map {
    zoom: f64,
    center_x: f64,
    center_y: f64,
    zoom_zero: Tile,
    zoom_one: [Tile; 4],
    countries: CountryOverlay,
    details: MapDetails,
}

/// Immutable presentation data derived from the domain's accepted guesses.
pub(super) struct MapState {
    country_styles: Vec<Option<Style>>,
}

impl MapState {
    pub(super) fn from_guesses(guesses: &[Guess], country_count: usize, theme: Theme) -> Self {
        let mut country_styles = vec![None; country_count];
        for guess in guesses {
            if let Some(style) = country_styles.get_mut(usize::from(guess.country().value())) {
                *style = Some(theme.guessed_land(guess.clue()));
            }
        }
        Self { country_styles }
    }
}

impl Map {
    pub(super) fn load() -> Result<Self, String> {
        Ok(Self {
            zoom: INITIAL_ZOOM,
            center_x: SPAIN_CENTER_X,
            center_y: SPAIN_CENTER_Y,
            zoom_zero: mvt::decode(include_bytes!(
                "../../../data/source/openstreetmap/0_0_0.pbf.gz"
            ))?,
            zoom_one: [
                mvt::decode(include_bytes!(
                    "../../../data/source/openstreetmap/1_0_0.pbf.gz"
                ))?,
                mvt::decode(include_bytes!(
                    "../../../data/source/openstreetmap/1_1_0.pbf.gz"
                ))?,
                mvt::decode(include_bytes!(
                    "../../../data/source/openstreetmap/1_0_1.pbf.gz"
                ))?,
                mvt::decode(include_bytes!(
                    "../../../data/source/openstreetmap/1_1_1.pbf.gz"
                ))?,
            ],
            countries: CountryOverlay::load()?,
            details: MapDetails::decode(include_bytes!("../../../assets/map-details-v1.bin"))?,
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
        let theme = Theme::from_environment();
        self.render_with_guesses(area, buffer, &[], theme);
    }

    pub(super) fn render_with_guesses(
        &self,
        area: Rect,
        buffer: &mut Buffer,
        guesses: &[Guess],
        theme: Theme,
    ) {
        let state = MapState::from_guesses(guesses, self.countries.country_count(), theme);
        self.render_with_state(area, buffer, &state, theme);
    }

    fn render_with_state(&self, area: Rect, buffer: &mut Buffer, state: &MapState, theme: Theme) {
        let Some(map_area) = layout::map_area(area) else {
            render_resize_message(area, buffer);
            return;
        };
        let columns = usize::from(map_area.width);
        let rows = usize::from(map_area.height);
        let dot_width = columns * 2;
        let dot_height = rows * 4;
        let mut dots = vec![0_u8; dot_width * dot_height];
        let mut countries = vec![u16::MAX; dot_width * dot_height];
        let scale =
            (dot_width as f64).min(dot_height as f64 / (SOUTH - NORTH)) * 2.0_f64.powf(self.zoom);
        let viewport = Viewport {
            width: dot_width,
            height: dot_height,
            center_x: self.center_x,
            center_y: self.center_y,
            scale,
        };

        self.countries.draw(&mut countries, viewport, self.zoom);

        for (tile, tile_x, tile_y, zoom) in self.active_tiles() {
            draw_tile(
                &mut dots,
                tile,
                TilePosition {
                    x: tile_x,
                    y: tile_y,
                    zoom,
                },
                viewport,
            );
        }
        if self.zoom >= DETAIL_ZOOM {
            self.details.draw(
                &mut dots,
                dot_width,
                dot_height,
                self.center_x,
                self.center_y,
                scale,
            );
        }
        render_dots(&dots, &countries, state, theme, dot_width, map_area, buffer);
        self.render_anchors(&countries, state, theme, viewport, map_area, buffer);
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

struct MapDetails {
    islands: Vec<Vec<(f64, f64)>>,
    western_sahara_border: Vec<(f64, f64)>,
}

impl MapDetails {
    fn decode(bytes: &[u8]) -> Result<Self, String> {
        if bytes.get(..4) != Some(b"WHDL") || read_u16(bytes, 4)? != 1 {
            return Err("map detail asset has an invalid header".to_owned());
        }
        let island_count = usize::from(read_u16(bytes, 6)?);
        let mut offset = 8;
        let mut islands = Vec::with_capacity(island_count);
        for _ in 0..island_count {
            islands.push(read_points(bytes, &mut offset)?);
        }
        let western_sahara_border = read_points(bytes, &mut offset)?;
        if offset != bytes.len() || islands.len() != 7 || western_sahara_border.len() != 15 {
            return Err("map detail asset has unexpected geometry".to_owned());
        }
        Ok(Self {
            islands,
            western_sahara_border,
        })
    }

    fn draw(
        &self,
        dots: &mut [u8],
        width: usize,
        height: usize,
        center_x: f64,
        center_y: f64,
        scale: f64,
    ) {
        for island in &self.islands {
            fill_geographic_land(dots, width, height, island, center_x, center_y, scale);
        }
        draw_geographic_path(
            dots,
            width,
            height,
            &self.western_sahara_border,
            center_x,
            center_y,
            scale,
        );
    }
}

fn fill_geographic_land(
    dots: &mut [u8],
    width: usize,
    height: usize,
    polygon: &[(f64, f64)],
    center_x: f64,
    center_y: f64,
    scale: f64,
) {
    let mut ring = polygon
        .iter()
        .copied()
        .map(|point| project_geographic_point(point, width, height, center_x, center_y, scale))
        .collect::<Vec<_>>();
    let Some(&first) = ring.first() else { return };
    ring.push(first);

    let Some(rows) = visible_rows(&[ring.as_slice()], width, height) else {
        return;
    };
    for y in rows {
        let mut intersections = ring_intersections(&ring, y as f64 + 0.5);
        intersections.sort_by(f64::total_cmp);
        for pair in intersections.chunks_exact(2) {
            let start = pair[0].floor().max(0.0) as usize;
            let end = pair[1].ceil().min(width as f64) as usize;
            for x in start..end {
                clear_water(dots, width, x as i32, y as i32);
            }
        }
    }
    for edge in ring.windows(2) {
        clear_water_line(dots, width, edge[0], edge[1]);
    }
}

fn draw_geographic_path(
    dots: &mut [u8],
    width: usize,
    height: usize,
    points: &[(f64, f64)],
    center_x: f64,
    center_y: f64,
    scale: f64,
) {
    for edge in points.windows(2) {
        let from = project_geographic_point(edge[0], width, height, center_x, center_y, scale);
        let to = project_geographic_point(edge[1], width, height, center_x, center_y, scale);
        draw_line(dots, width, height, from, to, BORDER);
    }
}

fn project_geographic_point(
    (longitude, latitude): (f64, f64),
    width: usize,
    height: usize,
    center_x: f64,
    center_y: f64,
    scale: f64,
) -> (i32, i32) {
    let world_x = (longitude + 180.0) / 360.0;
    let latitude = latitude.to_radians();
    let world_y =
        (1.0 - (latitude.tan() + latitude.cos().recip()).ln() / std::f64::consts::PI) / 2.0;
    (
        (width as f64 / 2.0 + (world_x - center_x) * scale).round() as i32,
        (height as f64 / 2.0 + (world_y - center_y) * scale).round() as i32,
    )
}

fn clear_water_line(dots: &mut [u8], width: usize, from: (i32, i32), to: (i32, i32)) {
    let (mut x, mut y) = from;
    let (x1, y1) = to;
    let dx = (x1 - x).abs();
    let dy = -(y1 - y).abs();
    let sx = if x < x1 { 1 } else { -1 };
    let sy = if y < y1 { 1 } else { -1 };
    let mut error = dx + dy;
    loop {
        clear_water(dots, width, x, y);
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

fn clear_water(dots: &mut [u8], width: usize, x: i32, y: i32) {
    if x >= 0 && y >= 0 && (x as usize) < width && (y as usize) < dots.len() / width {
        let dot = &mut dots[y as usize * width + x as usize];
        if *dot == WATER {
            *dot = 0;
        }
    }
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, String> {
    bytes
        .get(offset..offset + 2)
        .and_then(|value| value.try_into().ok())
        .map(u16::from_le_bytes)
        .ok_or_else(|| "map detail asset is truncated".to_owned())
}

fn read_i32(bytes: &[u8], offset: usize) -> Result<i32, String> {
    bytes
        .get(offset..offset + 4)
        .and_then(|value| value.try_into().ok())
        .map(i32::from_le_bytes)
        .ok_or_else(|| "map detail asset is truncated".to_owned())
}

fn read_points(bytes: &[u8], offset: &mut usize) -> Result<Vec<(f64, f64)>, String> {
    let count = usize::from(read_u16(bytes, *offset)?);
    *offset += 2;
    let mut points = Vec::with_capacity(count);
    for _ in 0..count {
        let longitude = f64::from(read_i32(bytes, *offset)?) / 1_000_000.0;
        let latitude = f64::from(read_i32(bytes, *offset + 4)?) / 1_000_000.0;
        *offset += 8;
        points.push((longitude, latitude));
    }
    Ok(points)
}

fn draw_tile(dots: &mut [u8], tile: &Tile, position: TilePosition, viewport: Viewport) {
    for layer in &tile.layers {
        let extent = layer.extent.unwrap_or(4096);
        if layer.name == "water" {
            for feature in &layer.features {
                let rings = mvt::decode_geometry(&feature.geometry);
                fill_polygon(dots, extent, &rings, position, viewport);
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
                let path = project_path(&path, extent, position, viewport);
                for edge in path.windows(2) {
                    draw_line(
                        dots,
                        viewport.width,
                        viewport.height,
                        edge[0],
                        edge[1],
                        BORDER,
                    );
                }
            }
        }
    }
}

fn fill_polygon(
    dots: &mut [u8],
    extent: u32,
    rings: &[Vec<(i32, i32)>],
    position: TilePosition,
    viewport: Viewport,
) {
    let rings: Vec<_> = rings
        .iter()
        .map(|ring| project_path(ring, extent, position, viewport))
        .filter(|ring| ring.len() >= 3)
        .collect();
    let Some(rows) = visible_rows(&rings, viewport.width, viewport.height) else {
        return;
    };
    for y in rows {
        let mut intersections = rings
            .iter()
            .flat_map(|ring| ring_intersections(ring, y as f64 + 0.5))
            .collect::<Vec<_>>();
        intersections.sort_by(f64::total_cmp);
        for pair in intersections.chunks_exact(2) {
            let start = pair[0].ceil().max(0.0) as usize;
            let end = pair[1].ceil().min(viewport.width as f64) as usize;
            for x in start..end {
                dots[y * viewport.width + x] = WATER;
            }
        }
    }
}

fn fill_country_polygon(
    countries: &mut [u16],
    extent: u32,
    rings: &[Vec<(i32, i32)>],
    country_id: u16,
    position: TilePosition,
    viewport: Viewport,
) {
    let rings: Vec<_> = rings
        .iter()
        .map(|ring| project_path(ring, extent, position, viewport))
        .filter(|ring| ring.len() >= 3)
        .collect();
    let Some(rows) = visible_rows(&rings, viewport.width, viewport.height) else {
        return;
    };
    for y in rows {
        let mut intersections = rings
            .iter()
            .flat_map(|ring| ring_intersections(ring, y as f64 + 0.5))
            .collect::<Vec<_>>();
        intersections.sort_by(f64::total_cmp);
        for pair in intersections.chunks_exact(2) {
            let start = pair[0].ceil().max(0.0) as usize;
            let end = pair[1].ceil().min(viewport.width as f64) as usize;
            for x in start..end {
                let pixel = &mut countries[y * viewport.width + x];
                if *pixel == u16::MAX || country_id != u16::MAX {
                    *pixel = (*pixel).min(country_id);
                }
            }
        }
    }
}

fn project_path(
    path: &[(i32, i32)],
    extent: u32,
    position: TilePosition,
    viewport: Viewport,
) -> Vec<(i32, i32)> {
    let tiles = 2.0_f64.powi(position.zoom as i32);
    let extent = f64::from(extent);
    path.iter()
        .map(|&(x, y)| {
            let world_x = (f64::from(position.x) + f64::from(x) / extent) / tiles;
            let world_y = (f64::from(position.y) + f64::from(y) / extent) / tiles;
            (
                (viewport.width as f64 / 2.0 + (world_x - viewport.center_x) * viewport.scale)
                    .round() as i32,
                (viewport.height as f64 / 2.0 + (world_y - viewport.center_y) * viewport.scale)
                    .round() as i32,
            )
        })
        .collect()
}

fn visible_rows(
    rings: &[impl AsRef<[(i32, i32)]>],
    width: usize,
    height: usize,
) -> Option<std::ops::Range<usize>> {
    let mut min_x = i32::MAX;
    let mut max_x = i32::MIN;
    let mut min_y = i32::MAX;
    let mut max_y = i32::MIN;

    for ring in rings {
        for &(x, y) in ring.as_ref() {
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }
    }
    if min_x == i32::MAX
        || max_x < 0
        || min_x >= width as i32
        || max_y < 0
        || min_y >= height as i32
    {
        return None;
    }

    let start = min_y.max(0) as usize;
    let end = (max_y.saturating_add(1).max(0) as usize).min(height);
    (start < end).then_some(start..end)
}

fn ring_intersections(ring: &[(i32, i32)], scanline: f64) -> Vec<f64> {
    ring.windows(2)
        .filter_map(|edge| {
            let ((x0, y0), (x1, y1)) = (edge[0], edge[1]);
            ((f64::from(y0) > scanline) != (f64::from(y1) > scanline)).then(|| {
                f64::from(x0) + (scanline - f64::from(y0)) * f64::from(x1 - x0) / f64::from(y1 - y0)
            })
        })
        .collect()
}

fn draw_line(
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

fn render_dots(
    dots: &[u8],
    countries: &[u16],
    state: &MapState,
    theme: Theme,
    dot_width: usize,
    area: Rect,
    buffer: &mut Buffer,
) {
    for row in 0..usize::from(area.height) {
        for column in 0..usize::from(area.width) {
            let mut mask = 0;
            let mut water = 0;
            let mut border = 0;
            for dot_y in 0..4 {
                for dot_x in 0..2 {
                    let value = dots[(row * 4 + dot_y) * dot_width + column * 2 + dot_x];
                    if value != 0 {
                        mask |= BRAILLE_DOTS[dot_y][dot_x];
                    }
                    water += usize::from(value == WATER);
                    border += usize::from(value == BORDER);
                }
            }
            let foreground = if border > water {
                theme.border()
            } else {
                theme.water()
            };
            let country = dominant_country(countries, dot_width, row, column);
            let land = if country == NEUTRAL_LAND {
                Style::default().bg(theme.neutral_land())
            } else {
                state
                    .country_styles
                    .get(usize::from(country))
                    .and_then(|style| *style)
                    .unwrap_or_else(|| Style::default().bg(theme.land()))
            };
            buffer[(area.x + column as u16, area.y + row as u16)]
                .set_symbol(&braille(mask))
                .set_style(Style::default().fg(foreground).patch(land));
        }
    }
}

impl Map {
    fn render_anchors(
        &self,
        countries: &[u16],
        state: &MapState,
        theme: Theme,
        viewport: Viewport,
        area: Rect,
        buffer: &mut Buffer,
    ) {
        for (country, style) in state.country_styles.iter().enumerate() {
            let Some(style) = style else { continue };
            let country = country as u16;
            if countries.contains(&country) {
                continue;
            }
            let Some((world_x, world_y)) = self.countries.anchor(country) else {
                continue;
            };
            let horizontal = (world_x - viewport.center_x + 0.5).rem_euclid(1.0) - 0.5;
            let dot_x = (viewport.width as f64 / 2.0 + horizontal * viewport.scale).round() as i32;
            let dot_y = (viewport.height as f64 / 2.0
                + (world_y - viewport.center_y) * viewport.scale)
                .round() as i32;
            if dot_x < 0 || dot_y < 0 {
                continue;
            }
            let cell_x = dot_x as usize / 2;
            let cell_y = dot_y as usize / 4;
            if cell_x >= usize::from(area.width) || cell_y >= usize::from(area.height) {
                continue;
            }
            buffer[(area.x + cell_x as u16, area.y + cell_y as u16)]
                .set_symbol("•")
                .set_style(Style::default().fg(theme.water()).patch(*style));
        }
    }
}

fn dominant_country(countries: &[u16], dot_width: usize, row: usize, column: usize) -> u16 {
    let mut selected = u16::MAX;
    let mut selected_count = 0;
    for dot_y in 0..4 {
        for dot_x in 0..2 {
            let candidate = countries[(row * 4 + dot_y) * dot_width + column * 2 + dot_x];
            let count = (0..4)
                .flat_map(|other_y| (0..2).map(move |other_x| (other_y, other_x)))
                .filter(|&(other_y, other_x)| {
                    countries[(row * 4 + other_y) * dot_width + column * 2 + other_x] == candidate
                })
                .count();
            if count > selected_count || (count == selected_count && candidate < selected) {
                selected = candidate;
                selected_count = count;
            }
        }
    }
    selected
}

fn render_status(area: Rect, buffer: &mut Buffer, zoom: f64) {
    let status = status_line(zoom);
    for (index, character) in status.chars().take(usize::from(area.width)).enumerate() {
        buffer[(area.x + index as u16, area.y + area.height - 1)]
            .set_symbol(&character.to_string())
            .set_style(
                Style::default()
                    .fg(ratatui::style::Color::Gray)
                    .bg(ratatui::style::Color::Black),
            );
    }
}

fn render_resize_message(area: Rect, buffer: &mut Buffer) {
    let message = "Resize terminal: minimum 20 columns x 8 rows";
    for (index, character) in message.chars().take(usize::from(area.width)).enumerate() {
        buffer[(area.x + index as u16, area.y + area.height / 2)]
            .set_symbol(&character.to_string())
            .set_style(Style::default());
    }
}

fn status_line(zoom: f64) -> String {
    format!("  {OSM_ATTRIBUTION}  |  Zoom {zoom:.2}  +/- zoom  Arrows/hjkl pan  Esc quit")
}

fn braille(mask: u8) -> String {
    char::from_u32(0x2800 + u32::from(mask))
        .expect("valid Braille mask")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        INITIAL_ZOOM, Map, MapState, NORTH, SOUTH, SPAIN_CENTER_X, SPAIN_CENTER_Y,
        dominant_country, status_line, visible_rows,
    };
    use crate::domain::{CountryId, Game, Proximity};
    use crate::infrastructure::tui::theme::{ColorMode, Theme};
    use ratatui::{Terminal, backend::TestBackend, style::Color};

    #[test]
    fn loads_centered_on_spain_at_zoom_one() {
        let map = Map::load().expect("map assets should load");

        assert_eq!(map.zoom, INITIAL_ZOOM);
        assert_eq!(map.center_x, SPAIN_CENTER_X);
        assert_eq!(map.center_y, SPAIN_CENTER_Y);
    }

    #[test]
    fn chooses_the_most_represented_country_in_a_braille_cell() {
        let countries = [1, 1, 1, 2, 1, 2, 3, 3];

        assert_eq!(dominant_country(&countries, 2, 0, 0), 1);
    }

    #[test]
    fn breaks_country_ties_by_stable_catalog_index() {
        let countries = [4, 4, 4, 4, 2, 2, 2, 2];

        assert_eq!(dominant_country(&countries, 2, 0, 0), 2);
    }

    #[test]
    fn navigation_clamps_latitude_wraps_longitude_and_bounds_zoom() {
        let mut map = Map::load().expect("map assets should load");

        for _ in 0..16 {
            map.zoom_in();
        }
        assert_eq!(map.zoom, 1.99);
        map.pan(0.0, -100.0);
        assert_eq!(map.center_y, NORTH);
        map.pan(100.0, 0.0);
        assert!((0.0..1.0).contains(&map.center_x));

        for _ in 0..16 {
            map.zoom_out();
        }
        assert_eq!(map.zoom, 0.0);
        map.pan(0.0, 100.0);
        assert_eq!(map.center_y, SOUTH);
    }

    #[test]
    fn status_includes_openstreetmap_attribution() {
        assert!(status_line(INITIAL_ZOOM).contains("OpenStreetMap contributors"));
    }

    #[test]
    fn skips_polygons_outside_the_viewport() {
        let rings = [vec![(20, 20), (30, 20), (30, 30), (20, 30), (20, 20)]];

        assert_eq!(visible_rows(&rings, 10, 10), None);
    }

    #[test]
    fn rasterizes_only_the_rows_covered_by_a_polygon() {
        let rings = [vec![(2, 3), (7, 3), (7, 8), (2, 8), (2, 3)]];

        assert_eq!(visible_rows(&rings, 10, 10), Some(3..9));
    }

    #[test]
    fn renders_a_deterministic_game_state_with_test_backend() {
        let map = Map::load().expect("map assets should load");
        let mut game = Game::new(CountryId::new(0));
        game.submit(CountryId::new(0), Proximity::new(0, false))
            .expect("target guess is accepted");
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).expect("test terminal starts");
        let theme = Theme::new(ColorMode::Ansi256);

        terminal
            .draw(|frame| {
                map.render_with_guesses(frame.area(), frame.buffer_mut(), game.guesses(), theme)
            })
            .expect("first frame renders");
        let first = terminal.backend().buffer().clone();
        terminal
            .draw(|frame| {
                map.render_with_guesses(frame.area(), frame.buffer_mut(), game.guesses(), theme)
            })
            .expect("second frame renders");

        assert_eq!(first, *terminal.backend().buffer());
        assert!(first.content().iter().any(|cell| cell.symbol() != " "));
        assert!(
            first
                .content()
                .iter()
                .any(|cell| cell.style().bg == Some(Color::Indexed(35)))
        );
    }

    #[test]
    fn displays_resize_message_below_minimum_size() {
        let map = Map::load().expect("map assets should load");
        let mut terminal = Terminal::new(TestBackend::new(19, 7)).expect("test terminal starts");

        terminal
            .draw(|frame| map.render(frame.area(), frame.buffer_mut()))
            .expect("small frame renders");

        let text = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(text.contains("Resize terminal"));
    }

    #[test]
    fn keeps_every_accepted_guess_in_the_presentation_state() {
        let mut game = Game::new(CountryId::new(2));
        game.submit(CountryId::new(0), Proximity::new(4_000, false))
            .expect("first guess is accepted");
        game.submit(CountryId::new(1), Proximity::new(500, false))
            .expect("second guess is accepted");

        let state = MapState::from_guesses(game.guesses(), 196, Theme::new(ColorMode::Ansi256));

        assert!(state.country_styles[0].is_some());
        assert!(state.country_styles[1].is_some());
    }
}
