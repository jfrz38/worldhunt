# Iteration 002: Country Catalog

Status: Planned  
Started:  
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

- [ ] Add the `tools/world-data` workspace package.
- [ ] Add `data/countries.toml` with the 195 selected countries.
- [ ] Define stable canonical names, ISO3 values, and common English aliases.
- [ ] Define normalization for case, whitespace, punctuation, and diacritics.
- [ ] Reject aliases that become ambiguous after normalization.
- [ ] Copy the exact world-boundary source snapshot into `data/source`.
- [ ] Record the snapshot retrieval date and SHA-256 checksum.
- [ ] Confirm mappings for Palestine, Vatican City, and exceptional source
  records.
- [ ] Add `THIRD_PARTY_NOTICES.md` with World Food Programme attribution and
  Open Government Licence v3.0 terms or link as required.
- [ ] Clarify in README that code and geographic data have different licenses.
- [ ] Add colocated unit tests for catalog and source-schema validation.
- [ ] Ensure generator and source-schema types remain outside the runtime
  domain and application modules.

## Acceptance Criteria

- [ ] Exactly 195 playable countries validate successfully.
- [ ] Every ISO3 value and canonical name is unique.
- [ ] Every normalized canonical name and alias resolves unambiguously.
- [ ] Every catalog country maps to source geometry.
- [ ] Non-playable source records are reported but do not fail validation.
- [ ] Source provenance, checksum, retrieval date, publisher, and license are
  documented.
- [ ] The validator reports actionable errors for malformed catalog entries.
- [ ] No source JSON or generator model is exposed as a runtime domain type.
- [ ] Relevant format, Clippy, and test commands pass.

## Verification

Not run; iteration has not started.

## Decisions

None yet.

## Deviations

None yet.

## Outcome

Pending.
