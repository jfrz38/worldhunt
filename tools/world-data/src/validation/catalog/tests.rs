use super::validate;
use crate::catalog::{Catalog, Country, SourceRecordSelector};
use std::path::Path;

fn valid_catalog() -> Catalog {
    Catalog {
        countries: (0..196)
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
            .collect(),
    }
}

#[test]
fn reports_normalized_name_collisions_with_both_values() {
    let mut catalog = valid_catalog();
    catalog.countries[1].aliases.push("Country-0".to_owned());

    let error = validate(&catalog, Path::new("countries.toml"))
        .expect_err("normalized collision must fail");

    assert!(error.contains("Country-0"));
    assert!(error.contains("Country 0"));
}

#[test]
fn rejects_invalid_and_duplicate_iso3_values() {
    let mut invalid_iso3 = valid_catalog();
    invalid_iso3.countries[0].iso3 = "aaA".to_owned();
    let error =
        validate(&invalid_iso3, Path::new("countries.toml")).expect_err("invalid ISO3 must fail");
    assert!(error.contains("three ASCII uppercase letters"));

    let mut duplicate_iso3 = valid_catalog();
    duplicate_iso3.countries[1].iso3 = duplicate_iso3.countries[0].iso3.clone();
    let error = validate(&duplicate_iso3, Path::new("countries.toml"))
        .expect_err("duplicate ISO3 must fail");
    assert!(error.contains("duplicate ISO3 value"));
}

#[test]
fn rejects_empty_and_duplicate_canonical_names() {
    let mut empty_name = valid_catalog();
    empty_name.countries[0].name = "   ".to_owned();
    let error = validate(&empty_name, Path::new("countries.toml"))
        .expect_err("empty canonical name must fail");
    assert!(error.contains("canonical name must not be empty"));

    let mut duplicate_name = valid_catalog();
    duplicate_name.countries[1].name = duplicate_name.countries[0].name.clone();
    let error = validate(&duplicate_name, Path::new("countries.toml"))
        .expect_err("duplicate canonical name must fail");
    assert!(error.contains("duplicate canonical name"));
}

#[test]
fn rejects_countries_without_source_records() {
    let mut catalog = valid_catalog();
    catalog.countries[0].source_records.clear();

    let error = validate(&catalog, Path::new("countries.toml"))
        .expect_err("countries require at least one source record");

    assert!(error.contains("at least one source record is required"));
}
