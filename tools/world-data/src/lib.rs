//! Build-time validation for the curated WorldHunt country data.

mod asset;
mod catalog;
mod details;
mod mvt;
mod normalization;
mod raster;
mod source;
mod validation;

pub use asset::{generate_asset, preview_asset, verify_asset};
pub use validation::{ValidationReport, validate_repository};
