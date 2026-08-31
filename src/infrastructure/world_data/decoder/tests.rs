use super::{HEADER_LENGTH, decode};

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
fn rejects_invalid_proximity_sections() {
    let original = include_bytes!("../../../../assets/world-v2.bin");
    let cells_length =
        u32::from_le_bytes(original[16..20].try_into().expect("cell length")) as usize;
    let borders_length =
        u32::from_le_bytes(original[20..24].try_into().expect("border length")) as usize;
    let anchors_length =
        u32::from_le_bytes(original[24..28].try_into().expect("anchor length")) as usize;
    let distances_length =
        u32::from_le_bytes(original[28..32].try_into().expect("distance length")) as usize;
    let distance_start = HEADER_LENGTH + cells_length + borders_length + anchors_length;
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
