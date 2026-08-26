use super::{validate_catalog, validate_contents, validate_repository, validate_source_mappings};
use crate::catalog::{Catalog, Country, SourceRecordSelector};
use crate::source::FeatureCollection;
use std::path::Path;

const SOURCE: &str = r#"{
  "features": [{
    "properties": {"iso3": "AAA", "name": "Alpha"},
    "geometry": {"type": "Polygon", "coordinates": [[[0, 0], [1, 0], [1, 1], [0, 0]]]}
  }]
}"#;

#[test]
fn reports_normalized_name_collisions_with_both_values() {
    let mut countries = (0..195)
        .map(|index| Country {
            iso3: format!(
                "A{}{}",
                char::from(b'A' + (index / 26) as u8),
                char::from(b'A' + (index % 26) as u8)
            ),
            name: format!("Country {index}"),
            aliases: Vec::new(),
            source_records: vec![SourceRecordSelector {
                iso3: "AAA".to_owned(),
                name: "Alpha".to_owned(),
            }],
        })
        .collect::<Vec<_>>();
    countries[1].aliases.push("Country-0".to_owned());

    let error = validate_catalog(&Catalog { countries }, Path::new("countries.toml"))
        .expect_err("normalized collision must fail");

    assert!(error.contains("Country-0"));
    assert!(error.contains("Country 0"));
}

#[test]
fn reports_checksum_mismatches() {
    let catalog = "countries = []";
    let error = validate_contents(
        Path::new("countries.toml"),
        catalog,
        Path::new("source.json"),
        SOURCE.as_bytes(),
        Path::new("source.metadata.toml"),
        "sha256 = \"wrong\"\nrecord_count = 1",
    )
    .expect_err("incorrect checksum must fail");

    assert!(error.contains("SHA-256 mismatch"));
}

#[test]
fn reports_a_missing_source_mapping_with_its_catalog_location() {
    let catalog = Catalog {
        countries: vec![Country {
            iso3: "AAA".to_owned(),
            name: "Alpha".to_owned(),
            aliases: Vec::new(),
            source_records: vec![SourceRecordSelector {
                iso3: "AAA".to_owned(),
                name: "Missing".to_owned(),
            }],
        }],
    };
    let source: FeatureCollection = serde_json::from_str(r#"{"features": []}"#)
        .expect("empty source collection should deserialize");

    let error = validate_source_mappings(&catalog, &source, Path::new("countries.toml"))
        .expect_err("missing mapping must fail");

    assert!(error.contains("countries[0] (AAA)"));
    assert!(error.contains("no matching source record"));
}

#[test]
fn rejects_a_source_mapping_to_a_different_country_iso3() {
    let catalog = Catalog {
        countries: vec![Country {
            iso3: "AAA".to_owned(),
            name: "Alpha".to_owned(),
            aliases: Vec::new(),
            source_records: vec![SourceRecordSelector {
                iso3: "BBB".to_owned(),
                name: "Beta".to_owned(),
            }],
        }],
    };
    let source: FeatureCollection = serde_json::from_str(
        r#"{
            "features": [{
                "properties": {"iso3": "BBB", "name": "Beta"},
                "geometry": {"type": "Polygon", "coordinates": [[[0, 0], [1, 0], [1, 1], [0, 0]]]}
            }]
        }"#,
    )
    .expect("source should deserialize");

    let error = validate_source_mappings(&catalog, &source, Path::new("countries.toml"))
        .expect_err("cross-country source mapping must fail");

    assert!(error.contains("must match the owning country ISO3"));
}

#[test]
fn validates_the_committed_catalog_and_source_snapshot() {
    let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("world-data should remain below the repository root");

    let report = validate_repository(repository_root).expect("committed data should validate");

    assert_eq!(report.country_count, 195);
    assert_eq!(report.source_mapping_count, 196);
    assert_eq!(report.non_playable_record_count, 60);
}
