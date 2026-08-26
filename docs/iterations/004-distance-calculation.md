# Iteration 004: Distance Calculation

Status: Planned  
Started:  
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
- Symmetric adjacency and `195 x 195` distance matrices.
- Independent reference cases and documented tolerance.
- Distance sections in the generated asset and validated runtime lookup.
- A validated infrastructure lookup over the decoded proximity data.

## Out of Scope

- Center-to-center direction hints, route drawing, and runtime calculations.
- Alternative territorial policies.

## Tasks

- [ ] Select and document the geodesic primitive and Earth model.
- [ ] Normalize longitude behavior around the antimeridian.
- [ ] Detect touching and overlapping territories before distance calculation.
- [ ] Densify boundary segments to enforce a measurable error bound.
- [ ] Build a spatial index to avoid exhaustive point-pair comparison.
- [ ] Compute every unordered playable-country pair once.
- [ ] Round and encode distances safely in `u16` kilometers.
- [ ] Store adjacency independently from rounded distance.
- [ ] Add matrix dimension, diagonal, symmetry, and range validation.
- [ ] Compare selected distances with an independent geodesic reference.
- [ ] Add generator unit tests for synthetic polygons, islands, poles, and
  antimeridian cases.
- [ ] Extend the asset and decoder without exposing geometry at runtime.
- [ ] Implement validated constant-time matrix lookup in
  `infrastructure/world_data/proximity.rs`, ready to be adapted to the domain
  port in iteration 005.
- [ ] Record generation time, tolerance, and observed maximum error.

## Acceptance Criteria

- [ ] Distance and adjacency matrices have exactly 195 rows and columns.
- [ ] Both matrices are symmetric and all distance diagonal values are zero.
- [ ] France and Spain are adjacent.
- [ ] The United States and Russia are geographically close through Alaska.
- [ ] Antimeridian test fixtures take the short path across longitude 180.
- [ ] An archipelago uses its closest participating component.
- [ ] Incorrect adjacent guesses can be distinguished from the correct target.
- [ ] Reference cases satisfy the documented accuracy tolerance.
- [ ] Runtime lookup is constant-time and performs no geometric calculation.
- [ ] Matrix indexes, encoded values, and binary asset structures remain inside
  the world-data infrastructure module.

## Verification

Not run; iteration has not started.

## Decisions

None yet.

## Deviations

None yet.

## Outcome

Pending.
