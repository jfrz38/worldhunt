use super::validate;
use std::path::Path;

const SOURCE: &str = r#"{
  "features": [{
    "properties": {"iso3": "AAA", "name": "Alpha"},
    "geometry": {"type": "Polygon", "coordinates": [[[0, 0], [1, 0], [1, 1], [0, 0]]]}
  }]
}"#;

fn metadata(sha256: &str) -> String {
    format!(
        r#"
dataset_id = "world-administrative-boundaries"
retrieved_on = "2026-08-27"
download_url = "https://example.com/download"
metadata_url = "https://example.com/metadata"
publisher = "World Food Programme"
license = "Open Government Licence v3.0"
license_url = "https://example.com/license"
record_count = 1
sha256 = "{sha256}"
"#
    )
}

#[test]
fn reports_checksum_mismatches() {
    let error = validate(
        Path::new("source.metadata.toml"),
        &metadata("wrong"),
        SOURCE.as_bytes(),
        1,
    )
    .expect_err("incorrect checksum must fail");

    assert!(error.contains("SHA-256 mismatch"));
}

#[test]
fn rejects_empty_provenance_fields() {
    let metadata =
        metadata("wrong").replace("publisher = \"World Food Programme\"", "publisher = \"\"");

    let error = validate(
        Path::new("source.metadata.toml"),
        &metadata,
        SOURCE.as_bytes(),
        1,
    )
    .expect_err("empty provenance fields must fail");

    assert!(error.contains("provenance field publisher must not be empty"));
}
