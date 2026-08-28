use super::{TilePosition, Viewport, fill_country_polygon};
use crate::infrastructure::tui::mvt::{self, Tile};
const COUNTRY_COUNT: usize = 196;
const NEUTRAL_LAND: usize = (u16::MAX - 1) as usize;
const WATER: usize = u16::MAX as usize;

pub(super) struct CountryOverlay {
    zoom_zero: Tile,
    zoom_one: [Tile; 4],
    anchors: Vec<(f64, f64)>,
}

impl CountryOverlay {
    pub(super) fn load() -> Result<Self, String> {
        let zoom_zero = mvt::decode(include_bytes!(
            "../../../../assets/country-map-v1/0_0_0.pbf.gz"
        ))?;
        let zoom_one = [
            mvt::decode(include_bytes!(
                "../../../../assets/country-map-v1/1_0_0.pbf.gz"
            ))?,
            mvt::decode(include_bytes!(
                "../../../../assets/country-map-v1/1_1_0.pbf.gz"
            ))?,
            mvt::decode(include_bytes!(
                "../../../../assets/country-map-v1/1_0_1.pbf.gz"
            ))?,
            mvt::decode(include_bytes!(
                "../../../../assets/country-map-v1/1_1_1.pbf.gz"
            ))?,
        ];
        let anchors = decode_anchors(include_bytes!(
            "../../../../assets/country-map-v1/anchors-v1.bin"
        ))?;
        validate_overlay(&zoom_zero, &zoom_one, &anchors)?;
        Ok(Self {
            zoom_zero,
            zoom_one,
            anchors,
        })
    }

    pub(super) fn draw(&self, countries: &mut [u16], viewport: Viewport, zoom: f64) {
        let tiles = if zoom < 1.0 {
            vec![(&self.zoom_zero, 0, 0, 0)]
        } else {
            vec![
                (&self.zoom_one[0], 0, 0, 1),
                (&self.zoom_one[1], 1, 0, 1),
                (&self.zoom_one[2], 0, 1, 1),
                (&self.zoom_one[3], 1, 1, 1),
            ]
        };
        for (tile, x, y, tile_zoom) in tiles {
            let position = TilePosition {
                x,
                y,
                zoom: tile_zoom,
            };
            for layer in &tile.layers {
                if layer.name != "country" {
                    continue;
                }
                let extent = layer.extent.unwrap_or(4096);
                for feature in &layer.features {
                    let Some(country_id) = mvt::unsigned_property(layer, feature, "country_id")
                    else {
                        continue;
                    };
                    let Ok(country_id) = u16::try_from(country_id) else {
                        continue;
                    };
                    fill_country_polygon(
                        countries,
                        extent,
                        &mvt::decode_geometry(&feature.geometry),
                        country_id,
                        position,
                        viewport,
                    );
                }
            }
        }
    }

    #[allow(dead_code)]
    pub(super) fn anchor(&self, country_id: u16) -> Option<(f64, f64)> {
        self.anchors.get(usize::from(country_id)).copied()
    }
}

fn decode_anchors(bytes: &[u8]) -> Result<Vec<(f64, f64)>, String> {
    if bytes.get(..4) != Some(b"WHCA") || read_u16(bytes, 4)? != 1 {
        return Err("country anchor asset has an invalid header".to_owned());
    }
    let count = usize::from(read_u16(bytes, 6)?);
    let expected_length = 8 + count * 8;
    if bytes.len() != expected_length || count != COUNTRY_COUNT {
        return Err("country anchor asset has an invalid length".to_owned());
    }
    (0..count)
        .map(|index| {
            let offset = 8 + index * 8;
            let x = f64::from(read_u32(bytes, offset)?) / 1_000_000_000.0;
            let y = f64::from(read_u32(bytes, offset + 4)?) / 1_000_000_000.0;
            ((0.0..=1.0).contains(&x) && (0.0..=1.0).contains(&y))
                .then_some((x, y))
                .ok_or_else(|| "country anchor asset has an out-of-range point".to_owned())
        })
        .collect()
}

fn validate_overlay(
    zoom_zero: &Tile,
    zoom_one: &[Tile; 4],
    anchors: &[(f64, f64)],
) -> Result<(), String> {
    for tile in std::iter::once(zoom_zero).chain(zoom_one) {
        let layer = tile
            .layers
            .iter()
            .find(|layer| layer.name == "country")
            .ok_or_else(|| "country overlay tile has no country layer".to_owned())?;
        if layer.version != 2 || layer.extent.unwrap_or_default() == 0 {
            return Err("country overlay has an invalid layer header".to_owned());
        }
        for feature in &layer.features {
            if feature.tags.len() % 2 != 0 {
                return Err("country overlay has malformed feature tags".to_owned());
            }
            let country_id = mvt::unsigned_property(layer, feature, "country_id")
                .ok_or_else(|| "country overlay feature has no country identifier".to_owned())?;
            let country_id = usize::try_from(country_id)
                .map_err(|_| "country overlay has an invalid country identifier".to_owned())?;
            if country_id == NEUTRAL_LAND || country_id == WATER {
                continue;
            }
            if country_id >= COUNTRY_COUNT {
                return Err(format!(
                    "country overlay has an unknown country identifier: {country_id}"
                ));
            }
        }
    }
    if anchors.len() != COUNTRY_COUNT {
        return Err("country overlay has an invalid anchor count".to_owned());
    }
    Ok(())
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, String> {
    bytes
        .get(offset..offset + 2)
        .and_then(|value| value.try_into().ok())
        .map(u16::from_le_bytes)
        .ok_or_else(|| "country anchor asset is truncated".to_owned())
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    bytes
        .get(offset..offset + 4)
        .and_then(|value| value.try_into().ok())
        .map(u32::from_le_bytes)
        .ok_or_else(|| "country anchor asset is truncated".to_owned())
}

#[cfg(test)]
mod tests {
    use super::{COUNTRY_COUNT, decode_anchors, validate_overlay};
    use crate::infrastructure::tui::mvt::Tile;

    #[test]
    fn decodes_the_committed_anchor_asset() {
        assert_eq!(
            decode_anchors(include_bytes!(
                "../../../../assets/country-map-v1/anchors-v1.bin"
            ))
            .expect("anchors should decode")
            .len(),
            COUNTRY_COUNT
        );
    }

    #[test]
    fn rejects_invalid_anchor_assets() {
        assert!(decode_anchors(b"WHCA").is_err());
    }

    #[test]
    fn rejects_a_tile_without_a_country_layer() {
        let tile = Tile { layers: Vec::new() };
        let zoom_one = [tile.clone(), tile.clone(), tile.clone(), tile.clone()];

        assert!(validate_overlay(&tile, &zoom_one, &vec![(0.0, 0.0); COUNTRY_COUNT]).is_err());
    }
}
