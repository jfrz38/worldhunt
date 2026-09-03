use super::{HEADER_LENGTH, decode};

const WIDTH_OFFSET: usize = 6;
const HEIGHT_OFFSET: usize = 8;
const COUNTRY_COUNT_OFFSET: usize = 10;
const CELLS_LENGTH_OFFSET: usize = 16;
const BORDERS_LENGTH_OFFSET: usize = 20;
const ANCHORS_LENGTH_OFFSET: usize = 24;
const DISTANCES_LENGTH_OFFSET: usize = 28;

fn section_starts(asset: &[u8]) -> (usize, usize, usize, usize) {
    let cells_length = u32::from_le_bytes(
        asset[CELLS_LENGTH_OFFSET..CELLS_LENGTH_OFFSET + 4]
            .try_into()
            .expect("cell length"),
    ) as usize;
    let borders_length = u32::from_le_bytes(
        asset[BORDERS_LENGTH_OFFSET..BORDERS_LENGTH_OFFSET + 4]
            .try_into()
            .expect("border length"),
    ) as usize;
    let anchors_length = u32::from_le_bytes(
        asset[ANCHORS_LENGTH_OFFSET..ANCHORS_LENGTH_OFFSET + 4]
            .try_into()
            .expect("anchor length"),
    ) as usize;
    let distance_start = HEADER_LENGTH + cells_length + borders_length + anchors_length;
    (
        HEADER_LENGTH,
        HEADER_LENGTH + cells_length,
        HEADER_LENGTH + cells_length + borders_length,
        distance_start,
    )
}

#[test]
fn rejects_corrupt_assets() {
    assert!(decode(b"WHMP").is_err());
}

#[test]
fn rejects_invalid_versions_lengths_and_identifiers() {
    let original = include_bytes!("../../../../assets/world-v2.bin");

    let mut version = original.to_vec();
    version[4] = 1;
    assert!(decode(&version).is_err());

    let mut future_version = original.to_vec();
    future_version[4] = 3;
    assert!(decode(&future_version).is_err());

    let mut length = original.to_vec();
    length[17] = 0;
    assert!(decode(&length).is_err());

    let mut identifier = original.to_vec();
    identifier[36] = 250;
    identifier[37] = 0;
    assert!(decode(&identifier).is_err());
}

#[test]
fn rejects_truncated_headers_and_sections() {
    let original = include_bytes!("../../../../assets/world-v2.bin");
    let (cells_start, borders_start, anchors_start, distances_start) = section_starts(original);

    for end in [
        0,
        HEADER_LENGTH - 1,
        cells_start + 1,
        borders_start + 1,
        anchors_start + 1,
        distances_start + 1,
        original.len() - 1,
    ] {
        assert!(
            decode(&original[..end]).is_err(),
            "truncation at {end} should fail"
        );
    }
}

#[test]
fn rejects_zero_dimensions_and_inconsistent_section_lengths() {
    let original = include_bytes!("../../../../assets/world-v2.bin");
    for offset in [WIDTH_OFFSET, HEIGHT_OFFSET, COUNTRY_COUNT_OFFSET] {
        let mut asset = original.to_vec();
        asset[offset..offset + 2].copy_from_slice(&0_u16.to_le_bytes());
        assert!(
            decode(&asset).is_err(),
            "zero header field at {offset} should fail"
        );
    }

    for offset in [
        CELLS_LENGTH_OFFSET,
        BORDERS_LENGTH_OFFSET,
        ANCHORS_LENGTH_OFFSET,
        DISTANCES_LENGTH_OFFSET,
    ] {
        let mut asset = original.to_vec();
        asset[offset..offset + 4].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(
            decode(&asset).is_err(),
            "oversized section at {offset} should fail"
        );
    }
}

#[test]
fn rejects_anchors_outside_the_raster_bounds() {
    let original = include_bytes!("../../../../assets/world-v2.bin");
    let (_, _, anchors_start, _) = section_starts(original);
    let mut asset = original.to_vec();

    asset[anchors_start..anchors_start + 2].copy_from_slice(&u16::MAX.to_le_bytes());
    assert!(decode(&asset).is_err());
}

#[test]
fn rejects_invalid_proximity_sections() {
    let original = include_bytes!("../../../../assets/world-v2.bin");
    let (_, _, _, distance_start) = section_starts(original);
    let distances_length = u32::from_le_bytes(
        original[DISTANCES_LENGTH_OFFSET..DISTANCES_LENGTH_OFFSET + 4]
            .try_into()
            .expect("distance length"),
    ) as usize;
    let adjacency_start = distance_start + distances_length;

    let mut length = original.to_vec();
    length[28] = 0;
    assert!(decode(&length).is_err());

    let mut adjacency_value = original.to_vec();
    adjacency_value[adjacency_start] = 2;
    assert!(decode(&adjacency_value).is_err());

    let mut asymmetric_distance = original.to_vec();
    asymmetric_distance[distance_start + 2] = 1;
    assert!(decode(&asymmetric_distance).is_err());
}

#[test]
fn decodes_the_committed_asset() {
    let (map_data, proximity) =
        decode(include_bytes!("../../../../assets/world-v2.bin")).expect("asset should decode");
    assert_eq!(map_data.dimensions(), (720, 300));
    assert_eq!(map_data.country_count(), 196);
    assert_eq!(
        proximity
            .between(0, 0)
            .expect("self is indexed")
            .distance_km,
        0
    );
}
