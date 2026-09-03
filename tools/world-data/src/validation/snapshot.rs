use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::path::Path;

#[derive(Debug, Deserialize)]
struct SourceMetadata {
    dataset_id: String,
    retrieved_on: String,
    download_url: String,
    metadata_url: String,
    publisher: String,
    license: String,
    license_url: String,
    record_count: usize,
    sha256: String,
}

pub(super) fn validate(
    metadata_path: &Path,
    metadata_text: &str,
    source_bytes: &[u8],
    source_record_count: usize,
) -> Result<(), String> {
    let metadata: SourceMetadata = toml::from_str(metadata_text).map_err(|error| {
        format!(
            "{}: invalid source metadata: {error}",
            metadata_path.display()
        )
    })?;
    validate_provenance(&metadata, metadata_path)?;

    let actual_checksum = format!("{:x}", Sha256::digest(source_bytes));
    if metadata.sha256 != actual_checksum {
        return Err(format!(
            "{}: SHA-256 mismatch: expected {}, found {}",
            metadata_path.display(),
            metadata.sha256,
            actual_checksum
        ));
    }
    if metadata.record_count != source_record_count {
        return Err(format!(
            "{}: record_count is {}, but the source snapshot contains {source_record_count} records",
            metadata_path.display(),
            metadata.record_count,
        ));
    }
    Ok(())
}

fn validate_provenance(metadata: &SourceMetadata, metadata_path: &Path) -> Result<(), String> {
    for (field, value) in [
        ("dataset_id", &metadata.dataset_id),
        ("retrieved_on", &metadata.retrieved_on),
        ("download_url", &metadata.download_url),
        ("metadata_url", &metadata.metadata_url),
        ("publisher", &metadata.publisher),
        ("license", &metadata.license),
        ("license_url", &metadata.license_url),
    ] {
        if value.trim().is_empty() {
            return Err(format!(
                "{}: provenance field {field} must not be empty",
                metadata_path.display()
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
