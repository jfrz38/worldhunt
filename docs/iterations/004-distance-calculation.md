# Iteration 004: Distance Calculation

Status: In Progress
Started: 2026-08-28
Completed:

## Goal

Generate trustworthy adjacency and territorial distance data for every pair of
playable countries, including islands and antimeridian cases.

## Dependencies

- Iteration 003 supplies validated detailed geometry and the asset format.
- [ADR 0003](../decisions/0003-country-distance.md) defines the distance
  semantics.

## Scope

- Shared-border and overlap detection.
- Geodesic minimum-distance approximation between country boundaries.
- Long-segment densification and spatial candidate indexing.
- Symmetric adjacency and `196 x 196` distance matrices.
- Independent reference cases and documented tolerance.
- Distance sections in the generated asset and validated runtime lookup.
- A validated infrastructure lookup over the decoded proximity data.

The work is split into two reviewable parts. The first generates and validates
the matrices in the world-data tool without changing the version-1 asset. The
second adds asset sections, decoding, and the infrastructure lookup.

## Out of Scope

- Center-to-center direction hints, route drawing, and runtime calculations.
- Alternative territorial policies.

## Tasks

- [x] Select and document the geodesic primitive and Earth model.
- [x] Normalize longitude behavior around the antimeridian.
- [x] Detect touching and overlapping territories before distance calculation.
- [x] Densify boundary segments to enforce a measurable error bound.
- [x] Build a spatial index to avoid exhaustive point-pair comparison.
- [x] Compute every unordered playable-country pair once.
- [x] Round and encode distances safely in `u16` kilometers.
- [x] Store adjacency independently from rounded distance.
- [x] Add matrix dimension, diagonal, symmetry, and range validation.
- [x] Compare selected distances with an independent geodesic reference.
- [x] Add generator unit tests for synthetic polygons, islands, poles, and
  antimeridian cases.
- [x] Extend the asset and decoder without exposing geometry at runtime.
- [x] Implement validated constant-time matrix lookup in
  `infrastructure/world_data/proximity.rs`, ready to be adapted to the domain
  port in iteration 005.
- [x] Record generation time, tolerance, and observed maximum error.

## Acceptance Criteria

- [x] Distance and adjacency matrices have exactly 196 rows and columns.
- [x] Both matrices are symmetric and all distance diagonal values are zero.
- [x] France and Spain are adjacent.
- [x] The United States and Russia are geographically close through Alaska.
- [x] Antimeridian test fixtures take the short path across longitude 180.
- [x] An archipelago uses its closest participating component.
- [x] Incorrect adjacent guesses can be distinguished from the correct target.
- [x] Reference cases satisfy the documented accuracy tolerance.
- [x] Runtime lookup is constant-time and performs no geometric calculation.
- [x] Matrix indexes, encoded values, and binary asset structures remain inside
  the world-data infrastructure module.

## Verification

- `cargo test --workspace`: passed on 2026-08-28 (36 world-data and 27 runtime
  tests).
- `cargo run --release -p world-data -- generate`: generated `world-v2.bin`.
- `cargo run --release -p world-data -- generate --check`: passed on
  2026-08-28; the generated asset is current.
- `make ci`: formatting, Clippy, tests, catalog validation, and deterministic
  generation passed. Its final workspace build is pending because Windows
  denied removal of an already-open `target/debug/worldhunt.exe`.

## Decisions

- WGS84 is the Earth model. The generator uses a Vincenty inverse for candidate
  measurement, with GeographicLib as a non-convergence fallback and reference.
  GeographicLib direct interpolation densifies only segments longer than 5 km.
- A normalized planar topology predicate determines adjacency before distance.
  It checks touching, crossing, containment, and overlap, including shifted
  copies around the antimeridian.
- Each country has an R-tree of WGS84 ECEF boundary samples. The eight closest
  point pairs by chord distance are measured on the ellipsoid; this avoids an
  exhaustive point-pair comparison while preserving the 5 km sampling bound.
- The adjacency diagonal is false. Distance diagonal values are zero. A
  non-adjacent gap below 500 m may legitimately round to zero kilometers.
- `WHMP` version 2 has a 36-byte header. It stores full row-major `u16`
  distances and validated byte-per-entry adjacency after the v1 raster,
  border, and anchor sections. The runtime rejects v1 and future versions.
- Runtime proximity remains private to `infrastructure/world_data`. Its lookup
  returns adjacency and distance together by checked row-major index and does
  not calculate geometry.

## Deviations

None yet.

## Outcome

Both parts are implemented. The generated `world-v2.bin` is 764,068 bytes:
the 36-byte header is followed by the existing 648,784 bytes of sections,
76,832 bytes of distances, and 38,416 bytes of adjacency data. It represents
196 countries, 317 adjacent unordered pairs, and a maximum encoded distance of
19,341 km. Generator and runtime tests cover v2 layout, malformed proximity
sections, matrix invariants, France-Spain adjacency, United States-Russia
proximity, and constant-time lookup. Formal completion remains pending merge
and verification on `develop`.
