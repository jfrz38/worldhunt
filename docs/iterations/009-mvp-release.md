# Iteration 009: MVP Release

Status: Planned  
Started:  
Completed:

## Goal

Prepare a self-contained, documented first release that users can install and
play without additional data or configuration.

## Dependencies

- Iteration 008 confirms correctness, portability, and performance.
- All MVP acceptance criteria in [product.md](../product.md) must be satisfied.

## Scope

- Final user-facing README and usage documentation.
- License and attribution review.
- Reproducible release builds for supported platforms.
- Release notes and final MVP checklist.
- Release publication only when explicitly requested.

## Out of Scope

- Features deferred by the product specification.
- Package managers, auto-update, signing, and installers unless separately
  approved.

## Tasks

- [ ] Document prerequisites, build, run, controls, color support, and `NO_COLOR`.
- [ ] Document the 196-country policy and territorial distance behavior.
- [ ] Review MIT code licensing and OGL data attribution.
- [ ] Confirm the release binary embeds the expected asset version.
- [ ] Build optimized binaries for Windows, Linux, and macOS.
- [ ] Smoke-test the exact release artifacts on their target platforms.
- [ ] Record checksums and generated asset version.
- [ ] Write concise release notes with known limitations.
- [ ] Complete the final MVP acceptance checklist.
- [ ] Create a version tag and publish artifacts only after explicit approval.

## Acceptance Criteria

- [ ] A new user can build and run the game from documented instructions.
- [ ] Release artifacts run without source JSON or a network connection.
- [ ] All 196 countries can be selected as targets and entered through their
  canonical names.
- [ ] Controls, color behavior, distances, and known limitations are documented.
- [ ] Every distributed artifact has a checksum.
- [ ] Code and data licenses and attributions are included correctly.
- [ ] The final release artifacts pass a full-game smoke test.
- [ ] No release action occurs without explicit user authorization.

## Verification

Not run; iteration has not started.

## Decisions

None yet.

## Deviations

None yet.

## Outcome

Pending.
