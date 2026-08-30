//! Read-only access to the embedded, generated map asset.

mod catalog;
mod decoder;
mod map_data;
mod proximity;

pub use map_data::MapData;

use crate::domain::{
    CountryId, GuessInput, Proximity,
    ports::{CountryCatalog, CountryProximity},
};

use self::{catalog::RuntimeCountryCatalog, proximity::ProximityData};

/// Validated runtime data exposed only through renderer and domain-facing APIs.
pub struct WorldData {
    map_data: MapData,
    catalog: RuntimeCountryCatalog,
    proximity: ProximityData,
}

/// Backward-compatible map-only access to the embedded world data.
pub fn decode_embedded() -> Result<MapData, String> {
    WorldData::decode_embedded().map(|world_data| world_data.map_data)
}

impl WorldData {
    pub fn decode_embedded() -> Result<Self, String> {
        let (map_data, proximity) = decoder::decode_embedded()?;
        let catalog = RuntimeCountryCatalog::load(map_data.country_count())?;
        Ok(Self {
            map_data,
            catalog,
            proximity,
        })
    }

    pub fn map_data(&self) -> &MapData {
        &self.map_data
    }
}

impl CountryCatalog for WorldData {
    fn playable(&self) -> &[CountryId] {
        self.catalog.playable()
    }

    fn name(&self, country: CountryId) -> Option<&str> {
        self.catalog.name(country)
    }

    fn resolve(&self, input: &GuessInput) -> Option<CountryId> {
        self.catalog.resolve(input)
    }
}

impl CountryProximity for WorldData {
    fn between(&self, first: CountryId, second: CountryId) -> Option<Proximity> {
        self.proximity
            .between(first.value(), second.value())
            .map(|record| Proximity::new(record.distance_km, record.adjacent))
    }
}

#[cfg(test)]
mod tests {
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
    }
}
