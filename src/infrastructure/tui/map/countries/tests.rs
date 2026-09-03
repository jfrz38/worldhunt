use super::{COUNTRY_COUNT, decode_anchors, validate_overlay};
use crate::infrastructure::tui::mvt::Tile;

#[test]
fn decodes_the_committed_anchor_asset() {
    assert_eq!(
        decode_anchors(include_bytes!(
            "../../../../../assets/country-map-v1/anchors-v1.bin"
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
