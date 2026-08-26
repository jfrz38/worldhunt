# Iteration 002: Country Catalog

Status: In Progress
Started: 2026-08-27
Completed:

## Goal

Create the authoritative, validated catalog of playable countries and add the
source geographic snapshot with complete provenance and licensing.

## Dependencies

- Iteration 001 provides the Cargo workspace and test infrastructure.
- [ADR 0004](../decisions/0004-playable-countries.md) defines the 195-country
  policy.

## Scope

- Curated 195-country catalog.
- Stable ISO3 mapping, English canonical names, and English aliases.
- Input normalization rules needed to validate alias uniqueness.
- Versioned source boundary snapshot.
- Dataset checksum, source metadata, and third-party notices.
- Catalog validator in the world-data tool package.
- A clear separation between source and generator records, encoded asset
  records, and the domain country types that runtime adapters will expose.

## Out of Scope

- Rasterization, distances, TUI input handling, and fuzzy matching.
- Additional languages or alternative playable catalogs.

## Tasks

- [x] Add the `tools/world-data` workspace package.
- [x] Add `data/countries.toml` with the 195 selected countries.
- [x] Define stable canonical names, ISO3 values, and common English aliases.
- [x] Define normalization for case, whitespace, punctuation, and diacritics.
- [x] Reject aliases that become ambiguous after normalization.
- [x] Copy the exact world-boundary source snapshot into `data/source`.
- [x] Record the snapshot retrieval date and SHA-256 checksum.
- [x] Confirm mappings for Palestine, Vatican City, and exceptional source
  records.
- [x] Add `THIRD_PARTY_NOTICES.md` with World Food Programme attribution and
  Open Government Licence v3.0 terms or link as required.
- [x] Clarify in README that code and geographic data have different licenses.
- [x] Add colocated unit tests for catalog and source-schema validation.
- [x] Ensure generator and source-schema types remain outside the runtime
  domain and application modules.

## Acceptance Criteria

- [x] Exactly 195 playable countries validate successfully.
- [x] Every ISO3 value and canonical name is unique.
- [x] Every normalized canonical name and alias resolves unambiguously.
- [x] Every catalog country maps to source geometry.
- [x] Non-playable source records are reported but do not fail validation.
- [x] Source provenance, checksum, retrieval date, publisher, and license are
  documented.
- [x] The validator reports actionable errors for malformed catalog entries.
- [x] No source JSON or generator model is exposed as a runtime domain type.
- [x] Relevant format, Clippy, and test commands pass.

## Verification

- `make check`: passed locally on 2026-08-27. This ran formatting, Clippy with
  warnings denied, 20 workspace unit tests, and `world-data validate`.
- `cargo run -p world-data -- validate`: passed locally; 195 playable countries,
  196 source mappings, and 60 reported non-playable source records.
- Snapshot SHA-256: verified as
  `fabbb96742b91183f4964d94e9dca7be654e32d2beb0c4cf7450d7f185093eee`.

## Decisions

- Catalog order is intentional and stable because a future generator derives
  `CountryId` values from it.
- Source mappings use the explicit `(iso3, name)` pair. ISO3 alone is not a
  unique source identifier.
- Palestine maps to both `Gaza Strip` and `West Bank`; Vatican City maps to the
  source record named `Holy See`.
- Normalization uses Unicode decomposition, lowercase conversion, combining
  mark removal, punctuation-to-space conversion, and whitespace collapse.
  Repeated normalized values may belong to the same country but never to two
  distinct playable countries.

## Deviations

None yet.

## Outcome

Pending merge into `develop` and post-merge verification.
