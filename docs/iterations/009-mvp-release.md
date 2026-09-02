# Iteration 009: MVP Release

Status: In Progress
Started: 2026-09-02
Completed:

## Goal

Prepare the first crates.io release so users can install and play a
self-contained game without additional data or configuration.

## Dependencies

- Iteration 008 confirms correctness, portability, and performance.
- All MVP acceptance criteria in [product.md](../product.md) must be satisfied.

## Scope

- Final user-facing README, installation, and usage documentation.
- License and attribution review for the distributed crate.
- Reproducible crates.io package validation and release automation.
- Version-bump, GitHub Release, and crates.io publication procedures.
- Release publication only when explicitly requested.

## Out of Scope

- Features deferred by the product specification.
- Prebuilt binaries, archives, installers, checksums, signing, package-manager
  integrations, and auto-update.

## Tasks

- [x] Document prerequisites, installation, controls, color support, and `NO_COLOR`.
- [x] Document the 196-country policy, territorial distance behavior, and known limitations.
- [x] Define an explicit crate-content contract including assets and notices.
- [x] Exclude the generator workspace crate and generator-only GeoJSON from publication.
- [x] Add version-bump, release-tag, and OIDC publication workflows.
- [x] Document initial publication with a temporary local token and later Trusted Publishing.
- [ ] Validate the exact packaged crate from a clean checkout.
- [ ] Perform a complete manual game smoke test in a real terminal.
- [ ] Create a version tag and publish the crate only after explicit approval.

## Acceptance Criteria

- [ ] A new user can install and run the crate from documented instructions.
- [ ] The packaged crate runs without source JSON or a network connection.
- [ ] All 196 countries can be selected as targets and entered through their
  canonical names.
- [ ] Controls, color behavior, distances, and known limitations are documented.
- [ ] The package contains only required source, runtime data, notices, and metadata.
- [ ] Code and data licenses and attributions are included correctly.
- [ ] The packaged crate passes a full-game smoke test.
- [ ] No release action occurs without explicit user authorization.

## Verification

- `make check`: passed on 2026-09-02: format, Clippy with warnings denied, 130
  tests (one ignored engineering measurement), catalog validation, and
  deterministic asset regeneration.
- `make release-check`: passed on 2026-09-02. The crate contract accepted 73
  files (1.7 MiB, 892.0 KiB compressed) and `cargo publish --dry-run` compiled
  the packaged crate successfully. A clean-checkout validation remains pending.

## Decisions

- Distribution is crates.io only. GitHub Releases provide generated notes and
  immutable version tags, not binary artifacts.
- The initial `0.1.0` publish uses a temporary local publish-only token from the
  exact release tag. Future publications use crates.io Trusted Publishing with
  GitHub Actions OIDC and the protected `crates-publish` Environment.

## Deviations

None yet.

## Outcome

Pending.
