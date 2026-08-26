# Iteration 003: Map Data Generator

Status: Planned  
Started:  
Completed:

## Goal

Transform the detailed source boundaries into a compact, deterministic world
map that preserves country identity and can be rendered without runtime
geometry processing.

## Dependencies

- Iteration 002 supplies the validated catalog and source snapshot.
- [ADR 0001](../decisions/0001-flat-map.md) selects the map projection.
- [ADR 0002](../decisions/0002-embedded-world-data.md) selects preprocessing
  and embedding.

## Scope

- Typed parsing of source `Polygon` and `MultiPolygon` geometries.
- Geometry and catalog mapping validation.
- Equirectangular country raster and border mask.
- Coverage-aware handling of small polygons.
- Visual anchors for playable countries.
- Versioned deterministic binary asset without distances.
- Infrastructure decoder and read-only map-data representation for runtime use.
- Human-inspectable preview output.

## Out of Scope

- Pairwise distance generation, game rules, and terminal widgets.
- Zoom levels and projections other than equirectangular.

## Tasks

- [ ] Define typed source geometry and metadata structures.
- [ ] Parse polygons, multipolygons, holes, and antimeridian-split components.
- [ ] Validate coordinate ranges and required mappings.
- [ ] Define reserved raster identifiers for water and neutral land.
- [ ] Prototype `720 x 360` and measure whether it is sufficient.
- [ ] Rasterize with coverage or supersampling rather than center-only tests.
- [ ] Resolve overlapping source records through a documented deterministic
  policy.
- [ ] Derive a border mask from territory transitions.
- [ ] Select and validate an interior visual anchor for every playable country.
- [ ] Define asset magic, version, sections, lengths, and decoding checks.
- [ ] Implement the runtime decoder in `infrastructure/world_data/decoder.rs`.
- [ ] Expose raster, borders, and anchors to the renderer through
  `infrastructure/world_data/map_data.rs` without treating them as domain
  objects.
- [ ] Generate an ANSI, text, or image preview for manual inspection.
- [ ] Add `generate` and `--check` modes.
- [ ] Add colocated unit tests for deterministic generation and malformed
  input.
- [ ] Commit the generated asset only after all invariants pass.

## Acceptance Criteria

- [ ] The raster preserves country identity and never globally unions all land.
- [ ] Every raster value is a valid or reserved identifier.
- [ ] Every playable country has a valid visual anchor.
- [ ] Representative islands, holes, and small countries pass focused tests.
- [ ] The preview is recognizable and uses correct world orientation.
- [ ] Repeated generation produces byte-identical output on the supported
  generation environment.
- [ ] The runtime decoder rejects invalid magic, unsupported versions, invalid
  lengths, and unknown identifiers.
- [ ] Generated and encoded asset structures do not appear in domain or
  application public APIs.
- [ ] The chosen resolution and generated size are recorded in the outcome.

## Verification

Not run; iteration has not started.

## Decisions

None yet.

## Deviations

None yet.

## Outcome

Pending.
