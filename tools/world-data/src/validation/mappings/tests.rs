use super::validate;
use crate::{
    catalog::{Catalog, Country, SourceRecordSelector},
    source::FeatureCollection,
};
use std::path::Path;

const SOURCE: &str = r#"{
  "features": [{
    "properties": {"iso3": "AAA", "name": "Alpha"},
    "geometry": {"type": "Polygon", "coordinates": [[[0, 0], [1, 0], [1, 1], [0, 0]]]}
  }]
}"#;

fn catalog_with(source_records: Vec<SourceRecordSelector>) -> Catalog {
    Catalog {
        countries: vec![Country {
            iso3: "AAA".to_owned(),
            name: "Alpha".to_owned(),
            aliases: Vec::new(),
            source_records,
        }],
    }
}

#[test]
fn reports_a_missing_source_mapping_with_its_catalog_location() {
    let catalog = catalog_with(vec![SourceRecordSelector {
        iso3: "AAA".to_owned(),
        name: "Missing".to_owned(),
    }]);
    let source: FeatureCollection = serde_json::from_str(r#"{"features": []}"#)
        .expect("empty source collection should deserialize");

    let error = validate(&catalog, &source, Path::new("countries.toml"))
        .expect_err("missing mapping must fail");

    assert!(error.contains("countries[0] (AAA)"));
    assert!(error.contains("no matching source record"));
}

#[test]
fn rejects_a_source_mapping_to_a_different_country_iso3() {
    let catalog = catalog_with(vec![SourceRecordSelector {
        iso3: "BBB".to_owned(),
        name: "Beta".to_owned(),
    }]);
    let source: FeatureCollection = serde_json::from_str(
        r#"{
            "features": [{
                "properties": {"iso3": "BBB", "name": "Beta"},
                "geometry": {"type": "Polygon", "coordinates": [[[0, 0], [1, 0], [1, 1], [0, 0]]]}
            }]
        }"#,
    )
    .expect("source should deserialize");

    let error = validate(&catalog, &source, Path::new("countries.toml"))
        .expect_err("cross-country source mapping must fail");

    assert!(error.contains("must match the owning country ISO3"));
}

#[test]
fn rejects_ambiguous_and_duplicate_source_mappings() {
    let catalog = catalog_with(vec![SourceRecordSelector {
        iso3: "AAA".to_owned(),
        name: "Alpha".to_owned(),
    }]);
    let ambiguous_source: FeatureCollection = serde_json::from_str(
        r#"{"features": [
            {"properties": {"iso3": "AAA", "name": "Alpha"}, "geometry": {"type": "Polygon", "coordinates": [[[0, 0], [1, 0], [1, 1], [0, 0]]]}},
            {"properties": {"iso3": "AAA", "name": "Alpha"}, "geometry": {"type": "Polygon", "coordinates": [[[0, 0], [1, 0], [1, 1], [0, 0]]]}}
        ]}"#,
    )
    .expect("source should deserialize");
    let error = validate(&catalog, &ambiguous_source, Path::new("countries.toml"))
        .expect_err("ambiguous source selector must fail");
    assert!(error.contains("selector is ambiguous"));

    let duplicate_catalog = catalog_with(vec![
        SourceRecordSelector {
            iso3: "AAA".to_owned(),
            name: "Alpha".to_owned(),
        },
        SourceRecordSelector {
            iso3: "AAA".to_owned(),
            name: "Alpha".to_owned(),
        },
    ]);
    let source: FeatureCollection =
        serde_json::from_str(SOURCE).expect("source should deserialize");
    let error = validate(&duplicate_catalog, &source, Path::new("countries.toml"))
        .expect_err("duplicate source selector must fail");
    assert!(error.contains("assigned more than once"));
}
