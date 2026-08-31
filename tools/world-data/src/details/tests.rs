use super::encode;
use crate::validation::load_validated_repository;
use std::path::Path;

#[test]
fn encodes_canary_islands_with_spains_country_identifier() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repository root exists");
    let validated = load_validated_repository(root).expect("committed data is valid");
    let spain = validated
        .catalog
        .countries
        .iter()
        .position(|country| country.iso3 == "ESP")
        .expect("Spain is catalogued") as u16;
    let bytes = encode(&validated).expect("details encode");

    assert_eq!(&bytes[..8], b"WHDL\x02\0\x07\0");
    assert_eq!(u16::from_le_bytes([bytes[8], bytes[9]]), spain);
}
