# World Data

## Source And Attribution

WorldHunt starts from the `World Administrative Boundaries - Countries and Territories` dataset published by the World Food Programme and distributed by OpenDataSoft under the Open Government Licence v3.0.

- Dataset identifier: `world-administrative-boundaries`
- Source metadata: <https://public.opendatasoft.com/explore/dataset/world-administrative-boundaries/information/>
- Publisher: World Food Programme, a United Nations agency
- License: Open Government Licence v3.0

The exact source URL, retrieval date, publisher, license, and SHA-256 checksum are recorded in `data/source/world-boundaries.metadata.toml`. OpenStreetMap tile provenance and hashes are recorded beside the embedded tiles. See [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) for the distributed attribution notices.

## Catalog

`data/countries.toml` defines the stable set of 196 playable countries, canonical English names, aliases, and source-geometry mappings. Dataset records outside that catalog are non-playable unless an explicit mapping includes them. Catalog order defines the stable country identifier used by generated assets.

## Generation Pipeline

```text
source JSON
  -> validate catalog mappings and polygons
  -> normalize antimeridian behavior
  -> retain identity per country or territory
  -> calculate proximity and adjacency
  -> generate map overlays, anchors, and details
  -> validate invariants
  -> write versioned binary assets
```

Source schemas, generator records, encoded asset sections, and matrix indexes are data concerns. They live in the generator or infrastructure and never become domain or application models.

## Map Assets

The active TUI renders offline Web Mercator OpenStreetMap vector tiles with country-ID overlay tiles and a small geographic-details asset. Longitude wraps at the antimeridian while latitude remains clamped to the supported map extent. Visual anchors preserve small countries and islands after terminal downsampling.

`world-v2.bin` is the deterministic preprocessing and proximity asset. Its `WHMP` version-2 layout contains catalog-indexed geography, anchors, a full row-major `u16` distance matrix, and a full row-major `u8` adjacency matrix. The runtime validates lengths, identifiers, anchor bounds, diagonals, symmetry, and adjacent/zero-distance consistency before exposing the data.

## Territorial Distance

For playable countries `A` and `B`, WorldHunt uses the minimum WGS84 geodesic distance between any participating territories in `A` and `B`. All mapped polygon components, including islands, participate. Touching or overlapping territories are adjacent and have zero territorial separation.

The generator densifies source boundary segments and searches candidates in ECEF space before calculating WGS84 ellipsoidal distances. Distances round to whole kilometers and are stored in a symmetric `196 x 196` matrix. Adjacency is stored separately so the UI can report a shared border instead of `0 km` for an incorrect neighboring guess.

## Invariants

- Exactly 196 playable countries exist.
- Every playable country resolves to valid geometry and has a valid visual anchor.
- Canonical names, ISO3 values, and normalized aliases are unambiguous.
- Distance and adjacency matrices match catalog order and are symmetric.
- Adjacent countries have zero distance; adjacency diagonals are false.
- Generation is deterministic.
- Antimeridian cases do not receive artificial world-spanning distances.

## Regeneration

Validate source data and verify committed generated assets with:

```sh
cargo run -p world-data -- validate
cargo run --release -p world-data -- generate --check
```
