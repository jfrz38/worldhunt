use crate::{
    domain::{
        GuessInput,
        ports::{CountryCatalog, CountryProximity},
    },
    infrastructure::world_data::WorldData,
};

#[test]
fn maps_embedded_catalog_and_proximity_to_domain_values() {
    let world_data = WorldData::decode_embedded().expect("embedded data is valid");
    assert_eq!(world_data.map_data().country_count(), 196);
    assert_eq!(world_data.playable().len(), 196);
    let france = world_data
        .resolve(&GuessInput::new("France"))
        .expect("France is playable");
    let spain = world_data
        .resolve(&GuessInput::new("Spain"))
        .expect("Spain is playable");
    assert!(
        world_data
            .between(france, spain)
            .expect("known IDs")
            .is_adjacent()
    );
    assert_eq!(
        world_data.between(crate::domain::CountryId::new(196), france),
        None
    );
    assert!(
        world_data
            .search(&GuessInput::new("Sp"), 5)
            .iter()
            .any(|country| world_data.name(*country) == Some("Spain"))
    );
    assert!(
        world_data
            .suggestions(&GuessInput::new("Mo"), 5)
            .iter()
            .any(|suggestion| suggestion.completion == "Moldova")
    );
}
