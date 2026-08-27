use crate::{
    catalog::{Catalog, Country, SourceRecordSelector},
    source::{Feature, FeatureCollection, SourceRecordKey},
};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

pub(super) fn validate(
    catalog: &Catalog,
    source: &FeatureCollection,
    catalog_path: &Path,
) -> Result<(), String> {
    MappingValidator::new(source, catalog_path).validate(catalog)
}

struct MappingValidator<'a> {
    source_index: HashMap<SourceRecordKey, Vec<&'a Feature>>,
    mapped_records: HashSet<SourceRecordKey>,
    catalog_path: &'a Path,
}

impl<'a> MappingValidator<'a> {
    fn new(source: &'a FeatureCollection, catalog_path: &'a Path) -> Self {
        Self {
            source_index: source.index_by_selector(),
            mapped_records: HashSet::new(),
            catalog_path,
        }
    }

    fn validate(mut self, catalog: &Catalog) -> Result<(), String> {
        for (country_index, country) in catalog.countries.iter().enumerate() {
            for selector in &country.source_records {
                self.validate_selector(country, country_index, selector)?;
            }
        }
        Ok(())
    }

    fn validate_selector(
        &mut self,
        country: &Country,
        country_index: usize,
        selector: &SourceRecordSelector,
    ) -> Result<(), String> {
        let key = SourceRecordKey::new(&selector.iso3, &selector.name);
        let location = format!(
            "{}: countries[{}] ({}) source record {} / {}",
            self.catalog_path.display(),
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
        let records = self
            .source_index
            .get(&key)
            .ok_or_else(|| format!("{location}: no matching source record"))?;
        if records.len() != 1 {
            return Err(format!(
                "{location}: selector is ambiguous and matches {} source records",
                records.len()
            ));
        }
        if !self.mapped_records.insert(key) {
            return Err(format!(
                "{location}: source record is assigned more than once"
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
}

#[cfg(test)]
mod tests;
