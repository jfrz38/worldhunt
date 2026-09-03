use super::{FeatureCollection, GeometryKind, SourceRecordKey};

#[test]
fn indexes_source_records_by_iso3_and_name() {
    let source: FeatureCollection = serde_json::from_str(
        r#"{
            "features": [{
                "properties": {"iso3": "VAT", "name": "Holy See"},
                "geometry": {"type": "Polygon", "coordinates": [[[0, 0], [1, 0], [1, 1], [0, 0]]]}
            }]
        }"#,
    )
    .expect("source should deserialize");

    let index = source.index_by_selector();
    let records = index
        .get(&SourceRecordKey::new("VAT", "Holy See"))
        .expect("source record should be indexed");

    assert_eq!(records.len(), 1);
    assert!(matches!(
        records[0].geometry,
        Some(GeometryKind::Polygon(_))
    ));
}

#[test]
fn rejects_a_source_record_without_a_name() {
    let error = serde_json::from_str::<FeatureCollection>(
        r#"{
            "features": [{
                "properties": {"iso3": "VAT"},
                "geometry": {"type": "Polygon", "coordinates": [[[0, 0], [1, 0], [1, 1], [0, 0]]]}
            }]
        }"#,
    )
    .expect_err("name is required for a source record");

    assert!(error.to_string().contains("name"));
}

#[test]
fn rejects_coordinates_that_are_not_a_polygon_array() {
    let error = serde_json::from_str::<FeatureCollection>(
        r#"{
            "features": [{
                "properties": {"iso3": "VAT", "name": "Holy See"},
                "geometry": {"type": "Polygon", "coordinates": "not coordinates"}
            }]
        }"#,
    )
    .expect_err("polygon coordinates must be nested numeric arrays");

    assert!(error.to_string().contains("coordinates"));
}
