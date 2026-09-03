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
