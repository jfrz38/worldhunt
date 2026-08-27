use super::Catalog;

#[test]
fn deserializes_a_country_with_multiple_source_records() {
    let catalog: Catalog = toml::from_str(
        r#"
            [[countries]]
            iso3 = "PSE"
            name = "Palestine"
            aliases = ["State of Palestine"]

            [[countries.source_records]]
            iso3 = "PSE"
            name = "Gaza Strip"

            [[countries.source_records]]
            iso3 = "PSE"
            name = "West Bank"
        "#,
    )
    .expect("catalog should deserialize");

    assert_eq!(catalog.countries.len(), 1);
    assert_eq!(catalog.countries[0].source_records.len(), 2);
}
