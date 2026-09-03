use std::collections::HashMap;

use serde::Deserialize;
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

use crate::domain::{
    CountryId, GuessInput,
    ports::{CountryCatalog, CountrySuggestion},
};

const EXPECTED_COUNTRY_COUNT: usize = 196;

#[derive(Deserialize)]
struct CatalogFile {
    countries: Vec<CatalogCountry>,
}

#[derive(Deserialize)]
struct CatalogCountry {
    name: String,
    #[serde(default)]
    aliases: Vec<String>,
}

pub(super) struct RuntimeCountryCatalog {
    playable: Vec<CountryId>,
    canonical_names: Vec<String>,
    names: HashMap<String, CountryId>,
    search_names: Vec<(String, String, CountryId, bool)>,
}

impl RuntimeCountryCatalog {
    pub(super) fn load(country_count: u16) -> Result<Self, String> {
        let catalog: CatalogFile = toml::from_str(include_str!("../../../data/countries.toml"))
            .map_err(|error| format!("embedded country catalog is invalid: {error}"))?;
        if catalog.countries.len() != EXPECTED_COUNTRY_COUNT
            || catalog.countries.len() != usize::from(country_count)
        {
            return Err("embedded country catalog does not match world data".to_owned());
        }

        let mut canonical_names = Vec::with_capacity(catalog.countries.len());
        let mut names = HashMap::new();
        let mut search_names = Vec::new();
        for (index, country) in catalog.countries.into_iter().enumerate() {
            let id = CountryId::new(index as u16);
            canonical_names.push(country.name.clone());
            for (canonical, name) in std::iter::once((true, country.name))
                .chain(country.aliases.into_iter().map(|name| (false, name)))
            {
                let normalized = normalize_name(&name);
                if normalized.is_empty() {
                    return Err("embedded country catalog has ambiguous names".to_owned());
                }
                search_names.push((normalized.clone(), name, id, canonical));
                if let Some(previous) = names.insert(normalized, id)
                    && previous != id
                {
                    return Err("embedded country catalog has ambiguous names".to_owned());
                }
            }
        }
        Ok(Self {
            playable: (0..country_count).map(CountryId::new).collect(),
            canonical_names,
            names,
            search_names,
        })
    }
}

impl CountryCatalog for RuntimeCountryCatalog {
    fn playable(&self) -> &[CountryId] {
        &self.playable
    }

    fn name(&self, country: CountryId) -> Option<&str> {
        self.canonical_names
            .get(usize::from(country.value()))
            .map(String::as_str)
    }

    fn search(&self, input: &GuessInput, limit: usize) -> Vec<CountryId> {
        let query = normalize_name(input.as_str());
        if query.chars().count() < 2 {
            return Vec::new();
        }

        let mut matches = self
            .search_names
            .iter()
            .filter(|(name, _, _, _)| name.starts_with(&query))
            .map(|(_, _, country, canonical)| (*country, *canonical))
            .collect::<Vec<_>>();
        matches.sort_unstable_by(|(left, left_canonical), (right, right_canonical)| {
            right_canonical
                .cmp(left_canonical)
                .then_with(|| self.name(*left).cmp(&self.name(*right)))
        });
        matches.dedup_by_key(|(country, _)| *country);
        matches
            .into_iter()
            .take(limit)
            .map(|(country, _)| country)
            .collect()
    }

    fn suggestions(&self, input: &GuessInput, limit: usize) -> Vec<CountrySuggestion> {
        let query = normalize_name(input.as_str());
        self.search(input, limit)
            .into_iter()
            .filter_map(|country| {
                self.search_names
                    .iter()
                    .filter(|(normalized, _, matched_country, _)| {
                        *matched_country == country && normalized.starts_with(&query)
                    })
                    .min_by(
                        |(_, left, _, left_canonical), (_, right, _, right_canonical)| {
                            left.chars()
                                .count()
                                .cmp(&right.chars().count())
                                .then_with(|| right_canonical.cmp(left_canonical))
                                .then_with(|| left.cmp(right))
                        },
                    )
                    .map(|(_, completion, _, _)| CountrySuggestion {
                        country,
                        completion: completion.clone(),
                    })
            })
            .collect()
    }

    fn resolve(&self, input: &GuessInput) -> Option<CountryId> {
        self.names.get(&normalize_name(input.as_str())).copied()
    }
}

fn normalize_name(name: &str) -> String {
    let mut normalized = String::new();
    let mut needs_space = false;
    for character in name.nfkd().flat_map(char::to_lowercase) {
        if is_combining_mark(character) {
            continue;
        }
        if character.is_alphanumeric() {
            if needs_space && !normalized.is_empty() {
                normalized.push(' ');
            }
            normalized.push(character);
            needs_space = false;
        } else {
            needs_space = true;
        }
    }
    normalized
}

#[cfg(test)]
mod tests;
