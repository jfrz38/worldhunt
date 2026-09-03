use super::{decode, encode, render_tile_dots};
use crate::raster::{HEIGHT, WIDTH};
use std::path::Path;

fn empty_proximity(country_count: usize) -> (Vec<u16>, Vec<bool>) {
    (
        vec![0; country_count * country_count],
        vec![false; country_count * country_count],
    )
}

#[test]
fn rejects_an_invalid_magic() {
    assert!(decode(b"bad", Path::new(".")).is_err());
}

#[test]
fn encodes_the_expected_header_size() {
    let (distances, adjacency) = empty_proximity(1);
    assert_eq!(
        encode(
            &vec![0; usize::from(WIDTH) * usize::from(HEIGHT)],
            &vec![0; usize::from(WIDTH) * usize::from(HEIGHT)],
            &[(0, 0)],
            1,
            &distances,
            &adjacency,
        )
        .expect("valid proximity data")
        .len(),
        36 + usize::from(WIDTH) * usize::from(HEIGHT) * 3 + 4 + 3
    );
}

#[test]
fn renders_a_colored_unicode_preview() {
    let dots = vec![1, 1, 2, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0];
    let color = render_tile_dots(&dots, 4, 2, 1, true);
    assert!(
        color
            .chars()
            .any(|glyph| (0x2800..=0x28ff).contains(&u32::from(glyph)))
    );
    assert!(color.contains("\x1b[38;2;"));
    assert_eq!(color.lines().count(), 1);
    let monochrome = render_tile_dots(&dots, 4, 2, 1, false);
    assert!(
        monochrome
            .chars()
            .any(|glyph| (0x2800..=0x28ff).contains(&u32::from(glyph)))
    );
    assert!(!monochrome.contains("\x1b["));
}

#[test]
fn renders_water_and_boundaries_as_braille_dots() {
    let dots = vec![0, 2, 0, 0, 0, 0, 0, 2];

    assert_eq!(render_tile_dots(&dots, 2, 1, 1, false), "⢈\n");
}

#[test]
fn committed_asset_uses_the_cropped_dimensions() {
    let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("world-data should remain below the repository root");
    let decoded = decode(
        include_bytes!("../../../../assets/world-v2.bin"),
        repository_root,
    )
    .expect("committed asset should decode");
    assert_eq!((decoded.width, decoded.height), (WIDTH, HEIGHT));
    assert_eq!(decoded.country_count, 196);
    assert_eq!(decoded.distances_km.len(), 196 * 196);
    assert_eq!(decoded.adjacency.len(), 196 * 196);
}

#[test]
fn committed_asset_preserves_real_country_proximity() {
    let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("world-data should remain below the repository root");
    let catalog = crate::validation::load_validated_repository(repository_root)
        .expect("committed repository data should validate")
        .catalog;
    let index = |iso3| {
        catalog
            .countries
            .iter()
            .position(|country| country.iso3 == iso3)
            .expect("country should be catalogued")
    };
    let decoded = decode(
        include_bytes!("../../../../assets/world-v2.bin"),
        repository_root,
    )
    .expect("committed asset should decode");
    let entry = |first: usize, second: usize| first * 196 + second;

    let france_spain = entry(index("FRA"), index("ESP"));
    assert_eq!(decoded.distances_km[france_spain], 0);
    assert!(decoded.adjacency[france_spain]);

    let united_states_russia = entry(index("USA"), index("RUS"));
    assert!(!decoded.adjacency[united_states_russia]);
    assert!(decoded.distances_km[united_states_russia] < 100);
}
