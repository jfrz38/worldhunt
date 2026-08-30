use crate::domain::{CountryId, GuessInput};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CountrySuggestion {
    pub country: CountryId,
    pub completion: String,
}

pub trait CountryCatalog {
    fn playable(&self) -> &[CountryId];
    fn name(&self, _country: CountryId) -> Option<&str> {
        None
    }
    /// Returns canonical country identities matching a normalized prefix.
    fn search(&self, _input: &GuessInput, _limit: usize) -> Vec<CountryId> {
        Vec::new()
    }
    /// Returns matching country identities with the text that should be completed.
    fn suggestions(&self, input: &GuessInput, limit: usize) -> Vec<CountrySuggestion> {
        self.search(input, limit)
            .into_iter()
            .filter_map(|country| {
                self.name(country).map(|completion| CountrySuggestion {
                    country,
                    completion: completion.to_owned(),
                })
            })
            .collect()
    }
    fn resolve(&self, input: &GuessInput) -> Option<CountryId>;
}
