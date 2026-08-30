use crate::domain::{CountryId, GuessInput};

pub trait CountryCatalog {
    fn playable(&self) -> &[CountryId];
    fn name(&self, _country: CountryId) -> Option<&str> {
        None
    }
    fn resolve(&self, input: &GuessInput) -> Option<CountryId>;
}
