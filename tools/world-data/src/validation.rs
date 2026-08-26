use crate::{
    catalog::{Catalog, Country},
    normalization::normalize_name,
    source::{FeatureCollection, SourceRecordKey},
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

const EXPECTED_COUNTRY_COUNT: usize = 195;

#[derive(Debug, PartialEq, Eq)]
pub struct ValidationReport {
    pub country_count: usize,
    pub source_mapping_count: usize,
    pub non_playable_record_count: usize,
}

#[derive(Debug, Deserialize)]
struct SourceMetadata {
    sha256: String,
    record_count: usize,
}

pub fn validate_repository(repository_root: &Path) -> Result<ValidationReport, String> {
    let catalog_path = repository_root.join("data/countries.toml");
    let source_path = repository_root.join("data/source/world-boundaries.json");
    let metadata_path = repository_root.join("data/source/world-boundaries.metadata.toml");

    let catalog_text = read_utf8(&catalog_path)?;
    let source_bytes = fs::read(&source_path).map_err(|error| {
        format!(
            "{}: could not read source snapshot: {error}",
            source_path.display()
        )
    })?;
    let metadata_text = read_utf8(&metadata_path)?;

    validate_contents(
        &catalog_path,
        &catalog_text,
        &source_path,
        &source_bytes,
        &metadata_path,
        &metadata_text,
    )
}

fn read_utf8(path: &Path) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|error| format!("{}: could not read file: {error}", path.display()))
}

fn validate_contents(
    catalog_path: &Path,
    catalog_text: &str,
    source_path: &Path,
    source_bytes: &[u8],
    metadata_path: &Path,
    metadata_text: &str,
) -> Result<ValidationReport, String> {
    let catalog: Catalog = toml::from_str(catalog_text)
        .map_err(|error| format!("{}: invalid catalog TOML: {error}", catalog_path.display()))?;
    let source: FeatureCollection = serde_json::from_slice(source_bytes)
        .map_err(|error| format!("{}: invalid source JSON: {error}", source_path.display()))?;
    let metadata: SourceMetadata = toml::from_str(metadata_text).map_err(|error| {
        format!(
            "{}: invalid source metadata: {error}",
            metadata_path.display()
        )
    })?;

    let actual_checksum = format!("{:x}", Sha256::digest(source_bytes));
    if metadata.sha256 != actual_checksum {
        return Err(format!(
            "{}: SHA-256 mismatch: expected {}, found {}",
            metadata_path.display(),
            metadata.sha256,
            actual_checksum
        ));
    }
    if metadata.record_count != source.features.len() {
        return Err(format!(
            "{}: record_count is {}, but the source snapshot contains {} records",
            metadata_path.display(),
            metadata.record_count,
            source.features.len()
        ));
    }

    validate_catalog(&catalog, catalog_path)?;
    validate_source_mappings(&catalog, &source, catalog_path)?;

    let mapped_records = catalog
        .countries
        .iter()
        .flat_map(|country| &country.source_records)
        .count();
    Ok(ValidationReport {
        country_count: catalog.countries.len(),
        source_mapping_count: mapped_records,
        non_playable_record_count: source.features.len() - mapped_records,
    })
}

fn validate_catalog(catalog: &Catalog, catalog_path: &Path) -> Result<(), String> {
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
                if previous.0 == &country.iso3 {
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

fn validate_source_mappings(
    catalog: &Catalog,
    source: &FeatureCollection,
    catalog_path: &Path,
) -> Result<(), String> {
    let index = source.index_by_selector();
    let mut mapped = HashSet::new();
    for (country_index, country) in catalog.countries.iter().enumerate() {
        for selector in &country.source_records {
            validate_source_selector(
                country,
                country_index,
                selector,
                &index,
                &mut mapped,
                catalog_path,
            )?;
        }
    }
    Ok(())
}

fn validate_source_selector(
    country: &Country,
    country_index: usize,
    selector: &crate::catalog::SourceRecordSelector,
    index: &HashMap<SourceRecordKey, Vec<&crate::source::Feature>>,
    mapped: &mut HashSet<SourceRecordKey>,
    catalog_path: &Path,
) -> Result<(), String> {
    let key = SourceRecordKey::new(&selector.iso3, &selector.name);
    let location = format!(
        "{}: countries[{}] ({}) source record {} / {}",
        catalog_path.display(),
        country_index,
        country.iso3,
        selector.iso3,
        selector.name
    );
    if selector.iso3 != country.iso3 {
        return Err(format!(
            "{location}: source record ISO3 must match the owning country ISO3 {}",
            country.iso3
        ));
    }
    let records = index
        .get(&key)
        .ok_or_else(|| format!("{location}: no matching source record"))?;
    if records.len() != 1 {
        return Err(format!(
            "{location}: selector is ambiguous and matches {} source records",
            records.len()
        ));
    }
    if !mapped.insert(key) {
        return Err(format!(
            "{location}: source record is assigned to more than one playable country"
        ));
    }

    let geometry = records[0]
        .geometry
        .as_ref()
        .ok_or_else(|| format!("{location}: source geometry is missing"))?;
    if !geometry.has_valid_coordinates() {
        return Err(format!(
            "{location}: source geometry must be a non-empty Polygon or MultiPolygon with finite positions"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
