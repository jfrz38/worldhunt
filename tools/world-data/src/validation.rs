use crate::{catalog::Catalog, source::FeatureCollection};
use std::{fs, path::Path};

mod catalog;
mod mappings;
mod snapshot;

#[derive(Debug, PartialEq, Eq)]
pub struct ValidationReport {
    pub country_count: usize,
    pub source_mapping_count: usize,
    pub non_playable_record_count: usize,
}

#[derive(Debug)]
pub(crate) struct ValidatedWorldData {
    pub(crate) catalog: Catalog,
    pub(crate) source: FeatureCollection,
}

pub fn validate_repository(repository_root: &Path) -> Result<ValidationReport, String> {
    Ok(load_validated_repository(repository_root)?.report())
}

pub(crate) fn load_validated_repository(
    repository_root: &Path,
) -> Result<ValidatedWorldData, String> {
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

    load_validated_contents(
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

fn load_validated_contents(
    catalog_path: &Path,
    catalog_text: &str,
    source_path: &Path,
    source_bytes: &[u8],
    metadata_path: &Path,
    metadata_text: &str,
) -> Result<ValidatedWorldData, String> {
    let catalog: Catalog = toml::from_str(catalog_text)
        .map_err(|error| format!("{}: invalid catalog TOML: {error}", catalog_path.display()))?;
    let source: FeatureCollection = serde_json::from_slice(source_bytes)
        .map_err(|error| format!("{}: invalid source JSON: {error}", source_path.display()))?;
    snapshot::validate(
        &metadata_path,
        metadata_text,
        source_bytes,
        source.features.len(),
    )?;
    catalog::validate(&catalog, catalog_path)?;
    mappings::validate(&catalog, &source, catalog_path)?;

    Ok(ValidatedWorldData { catalog, source })
}

impl ValidatedWorldData {
    fn report(&self) -> ValidationReport {
        let source_mapping_count = self
            .catalog
            .countries
            .iter()
            .map(|country| country.source_records.len())
            .sum();
        ValidationReport {
            country_count: self.catalog.countries.len(),
            source_mapping_count,
            non_playable_record_count: self.source.features.len() - source_mapping_count,
        }
    }
}

#[cfg(test)]
mod tests;
