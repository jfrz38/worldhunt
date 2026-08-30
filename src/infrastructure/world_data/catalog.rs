use std::collections::HashMap;

use serde::Deserialize;
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

use crate::domain::{CountryId, GuessInput, ports::CountryCatalog};

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
        for (index, country) in catalog.countries.into_iter().enumerate() {
            let id = CountryId::new(index as u16);
            canonical_names.push(country.name.clone());
            for name in std::iter::once(country.name).chain(country.aliases) {
                let normalized = normalize_name(&name);
                if normalized.is_empty() {
                    return Err("embedded country catalog has ambiguous names".to_owned());
                }
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
mod tests {
    use crate::{
        domain::{GuessInput, ports::CountryCatalog},
        infrastructure::world_data::catalog::{RuntimeCountryCatalog, normalize_name},
    };

    #[test]
    fn normalizes_diacritics_whitespace_and_punctuation() {
        assert_eq!(normalize_name("  Cote d'Ivoire "), "cote d ivoire");
    }

    #[test]
    fn resolves_canonical_names_and_aliases() {
        let catalog = RuntimeCountryCatalog::load(196).expect("embedded catalog is valid");
        assert_eq!(catalog.playable().len(), 196);
        assert_eq!(
            catalog.resolve(&GuessInput::new("  uSa  ")),
            catalog.resolve(&GuessInput::new("United States"))
        );
        assert_eq!(
            catalog.resolve(&GuessInput::new("Côte d’Ivoire")),
            catalog.resolve(&GuessInput::new("Cote d'Ivoire"))
        );
    }
}
