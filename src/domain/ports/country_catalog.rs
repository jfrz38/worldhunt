use crate::domain::{CountryId, GuessInput};

pub trait CountryCatalog {
    fn playable(&self) -> &[CountryId];
    fn resolve(&self, input: &GuessInput) -> Option<CountryId>;
}
