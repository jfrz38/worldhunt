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
- [ADR 0005](../decisions/0005-zoomable-web-mercator-map.md) selects the
  active map projection and navigation.
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
- Validated offline Web Mercator basemap and country-ID overlay assets for the
  active Braille renderer.

## Out of Scope

- Pairwise distance generation, game rules, and terminal widgets.
- Game behavior, distance calculation, color themes, and playable TUI layout.

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

- [x] The raster preserves country identity and never globally unions all land.
- [x] Every raster value is a valid or reserved identifier.
- [x] Every playable country has a valid visual anchor.
- [x] Representative islands, holes, and small countries pass focused tests.
- [x] The preview is recognizable, responsive, and uses correct world orientation.
- [x] Repeated generation produces byte-identical output on the supported
  generation environment.
- [x] The runtime decoder rejects invalid magic, unsupported versions, invalid
  lengths, and unknown identifiers.
- [x] Generated and encoded asset structures do not appear in domain or
  application public APIs.
- [x] The chosen resolution and generated size are recorded in the outcome.

## Verification

- `make ci`: passed locally on 2026-08-28. This ran formatting, Clippy with
  warnings denied, 47 workspace tests, catalog validation, deterministic asset
  regeneration, and workspace build.
- `cargo run --release -p world-data -- generate --check`: passed locally on
  2026-08-28; `world-v1.bin` is byte-identical at 648,816 bytes and the country
  tiles/details total 271,670 bytes.
- `cargo run -p world-data -- preview`: passed locally on 2026-08-28.

## Decisions

- The asset uses uncompressed little-endian sections: a 32-byte header, `u16`
  raster identifiers, `u8` borders, and `(u16, u16)` anchors. `WHMP` and version
  1 identify the format. Iteration 004 will introduce a new version when it
  adds distance data.
- The selected internal raster is `720 x 300`, covering `90 degrees N` through
  `60 degrees S`; its generated size is 648,816 bytes. This removes Antarctica
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
- ADR 0005 supersedes the planned fixed equirectangular TUI with an offline Web
  Mercator Braille renderer. The raster remains the deterministic preprocessing
  asset and future container for proximity data; the country overlay supplies
  the active renderer's stable country identity.

## Deviations

- The merged implementation added a Web Mercator OpenStreetMap basemap,
  country-ID vector overlays, Braille rendering, zoom, and pan. These features
  were originally assigned to later work and conflict with ADR 0001. This
  closure records them through ADR 0005, adds offline attribution and runtime
  overlay validation, and leaves game-state rendering, themes, anchors markers,
  and layout to iterations 005 through 008.

## Outcome

All branch acceptance criteria pass. The deterministic preprocessing raster is
720 x 300 and 648,816 bytes; country overlays and map details total 271,670
bytes. Completion remains pending the required post-merge verification on
`develop`.
