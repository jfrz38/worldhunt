use crate::domain::{CountryId, Proximity};

pub trait CountryProximity {
    fn between(&self, first: CountryId, second: CountryId) -> Option<Proximity>;
}
