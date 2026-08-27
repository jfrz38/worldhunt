# Iteration 003: Map Data Generator

Status: In Progress
Started: 2026-08-27
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

- [x] Define typed source geometry and metadata structures.
- [x] Parse polygons, multipolygons, holes, and antimeridian-split components.
- [x] Validate coordinate ranges and required mappings.
- [x] Define reserved raster identifiers for water and neutral land.
- [x] Prototype `720 x 300` and measure whether it is sufficient.
- [x] Rasterize with coverage or supersampling rather than center-only tests.
- [x] Resolve overlapping source records through a documented deterministic
  policy.
- [x] Derive a border mask from territory transitions.
- [x] Select and validate an interior visual anchor for every playable country.
- [x] Define asset magic, version, sections, lengths, and decoding checks.
- [x] Implement the runtime decoder in `infrastructure/world_data/decoder.rs`.
- [x] Expose raster, borders, and anchors to the renderer through
  `infrastructure/world_data/map_data.rs` without treating them as domain
  objects.
- [x] Generate a responsive ANSI/Unicode preview with a monochrome fallback.
- [x] Add `generate` and `--check` modes.
- [x] Add colocated unit tests for deterministic generation and malformed
  input.
- [x] Generate the asset only after all invariants pass.

## Acceptance Criteria

- [ ] The raster preserves country identity and never globally unions all land.
- [ ] Every raster value is a valid or reserved identifier.
- [ ] Every playable country has a valid visual anchor.
- [ ] Representative islands, holes, and small countries pass focused tests.
- [x] The preview is recognizable, responsive, and uses correct world orientation.
- [ ] Repeated generation produces byte-identical output on the supported
  generation environment.
- [ ] The runtime decoder rejects invalid magic, unsupported versions, invalid
  lengths, and unknown identifiers.
- [ ] Generated and encoded asset structures do not appear in domain or
  application public APIs.
- [ ] The chosen resolution and generated size are recorded in the outcome.

## Verification

- `cargo test --workspace`: passed locally on 2026-08-27 (35 tests).
- `cargo clippy --all-targets --all-features -- -D warnings`: passed locally on
  2026-08-27.
- `cargo run --release -p world-data -- generate --check`: passed locally on
  2026-08-27; the committed asset is byte-identical at 648,812 bytes.
- `cargo run -p world-data -- preview`: passed locally; manually inspected the
  responsive equirectangular Unicode preview.

## Decisions

- The asset uses uncompressed little-endian sections: a 32-byte header, `u16`
  raster identifiers, `u8` borders, and `(u16, u16)` anchors. `WHMP` and version
  1 identify the format. Iteration 004 will introduce a new version when it
  adds distance data.
- The selected internal raster is `720 x 300`, covering `90 degrees N` through
  `60 degrees S`; its generated size is 648,812 bytes. This removes Antarctica
  and unused southern ocean from the game map. Four fixed sub-pixel samples
  select the highest coverage identifier; equal coverage resolves to the lowest
  stable catalog identifier.
- Unmapped WFP records are rasterized as neutral land. Playable country IDs win
  any overlap deterministically, so neutral land is never globally unioned with
  countries.
- The preview samples each output sub-pixel by area to preserve one-cell
  coastlines and country borders at reduced terminal sizes. ANSI terminals use
  colored Unicode half-blocks; `NO_COLOR` and redirected output use a
  monochrome Unicode fallback.

## Deviations

None yet.

## Outcome

Pending.
