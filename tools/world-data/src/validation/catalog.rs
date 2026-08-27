use crate::{catalog::Catalog, normalization::normalize_name};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

const EXPECTED_COUNTRY_COUNT: usize = 195;

pub(super) fn validate(catalog: &Catalog, catalog_path: &Path) -> Result<(), String> {
    if catalog.countries.len() != EXPECTED_COUNTRY_COUNT {
        return Err(format!(
            "{}: expected {EXPECTED_COUNTRY_COUNT} playable countries, found {}",
            catalog_path.display(),
            catalog.countries.len()
        ));
    }

    let mut iso3s = HashSet::new();
    let mut canonical_names = HashSet::new();
    let mut normalized_names: HashMap<String, (&str, &str)> = HashMap::new();
    for (index, country) in catalog.countries.iter().enumerate() {
        let location = format!(
            "{}: countries[{}] ({})",
            catalog_path.display(),
            index,
            country.iso3
        );
        if country.iso3.len() != 3
            || !country
                .iso3
                .chars()
                .all(|character| character.is_ascii_uppercase())
        {
            return Err(format!(
                "{location}: iso3 must contain exactly three ASCII uppercase letters"
            ));
        }
        if !iso3s.insert(&country.iso3) {
            return Err(format!("{location}: duplicate ISO3 value {}", country.iso3));
        }
        if country.name.trim().is_empty() {
            return Err(format!("{location}: canonical name must not be empty"));
        }
        if !canonical_names.insert(&country.name) {
            return Err(format!(
                "{location}: duplicate canonical name {}",
                country.name
            ));
        }
        if country.source_records.is_empty() {
            return Err(format!(
                "{location}: at least one source record is required"
            ));
        }

        for value in std::iter::once(&country.name).chain(country.aliases.iter()) {
            let normalized = normalize_name(value);
            if normalized.is_empty() {
                return Err(format!(
                    "{location}: name or alias {value:?} normalizes to an empty value"
                ));
            }
            if let Some(previous) = normalized_names.get(&normalized) {
                if previous.0 == country.iso3.as_str() {
                    continue;
                }
                return Err(format!(
                    "{location}: {value:?} conflicts with {previous_value:?} from {previous_iso3} after normalization to {normalized:?}",
                    previous_iso3 = previous.0,
                    previous_value = previous.1,
                ));
            }
            normalized_names.insert(normalized, (country.iso3.as_str(), value));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
