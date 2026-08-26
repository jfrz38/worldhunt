use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub(crate) struct FeatureCollection {
    pub(crate) features: Vec<Feature>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Feature {
    pub(crate) properties: SourceProperties,
    pub(crate) geometry: Option<Geometry>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SourceProperties {
    pub(crate) iso3: Option<String>,
    pub(crate) name: String,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(tag = "type", content = "coordinates")]
pub(crate) enum GeometryKind {
    Polygon(Vec<Vec<Vec<f64>>>),
    MultiPolygon(Vec<Vec<Vec<Vec<f64>>>>),
    #[serde(other)]
    Unsupported,
}

pub(crate) type Geometry = GeometryKind;

impl GeometryKind {
    pub(crate) fn has_valid_coordinates(&self) -> bool {
        match self {
            Self::Polygon(rings) => valid_polygon(rings),
            Self::MultiPolygon(polygons) => {
                !polygons.is_empty() && polygons.iter().all(|rings| valid_polygon(rings))
            }
            Self::Unsupported => false,
        }
    }
}

fn valid_polygon(rings: &[Vec<Vec<f64>>]) -> bool {
    !rings.is_empty()
        && rings.iter().all(|ring| {
            ring.len() >= 4
                && ring.iter().all(|position| {
                    position.len() >= 2 && position.iter().all(|coordinate| coordinate.is_finite())
                })
        })
}

impl FeatureCollection {
    pub(crate) fn index_by_selector(&self) -> HashMap<SourceRecordKey, Vec<&Feature>> {
        let mut index = HashMap::new();
        for feature in &self.features {
            let Some(iso3) = &feature.properties.iso3 else {
                continue;
            };

            index
                .entry(SourceRecordKey::new(iso3, &feature.properties.name))
                .or_insert_with(Vec::new)
                .push(feature);
        }
        index
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct SourceRecordKey {
    iso3: String,
    name: String,
}

impl SourceRecordKey {
    pub(crate) fn new(iso3: &str, name: &str) -> Self {
        Self {
            iso3: iso3.to_owned(),
            name: name.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests;
