use super::validate_repository;
use std::path::Path;

#[test]
fn validates_the_committed_catalog_and_source_snapshot() {
    let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("world-data should remain below the repository root");

    let report = validate_repository(repository_root).expect("committed data should validate");

    assert_eq!(report.country_count, 196);
    assert_eq!(report.source_mapping_count, 197);
    assert_eq!(report.non_playable_record_count, 59);
}
