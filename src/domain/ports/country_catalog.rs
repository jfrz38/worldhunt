use crate::domain::{CountryId, GuessInput};

pub trait CountryCatalog {
    fn playable(&self) -> &[CountryId];
    fn name(&self, _country: CountryId) -> Option<&str> {
        None
    }
    /// Returns canonical country identities matching a normalized prefix.
    fn search(&self, _input: &GuessInput, _limit: usize) -> Vec<CountryId> {
        Vec::new()
    }
    fn resolve(&self, input: &GuessInput) -> Option<CountryId>;
}
