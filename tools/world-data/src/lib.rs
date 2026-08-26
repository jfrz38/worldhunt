//! Build-time validation for the curated WorldHunt country data.

mod catalog;
mod normalization;
mod source;
mod validation;

pub use validation::{ValidationReport, validate_repository};
