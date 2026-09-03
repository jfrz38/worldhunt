use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct Catalog {
    pub(crate) countries: Vec<Country>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Country {
    pub(crate) iso3: String,
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) aliases: Vec<String>,
    pub(crate) source_records: Vec<SourceRecordSelector>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SourceRecordSelector {
    pub(crate) iso3: String,
    pub(crate) name: String,
}

#[cfg(test)]
mod tests;
