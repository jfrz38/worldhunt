# World Data

## Source

The initial source is the `World Administrative Boundaries - Countries and
Territories` dataset published by the World Food Programme and distributed by
OpenDataSoft.

- Dataset identifier: `world-administrative-boundaries`
- Publisher: World Food Programme, a United Nations agency
- License: Open Government Licence v3.0
- Dataset records: 256
- Geometry types: `Polygon` and `MultiPolygon`
- Source metadata: <https://public.opendatasoft.com/explore/dataset/world-administrative-boundaries/information/>
- Original publisher reference: <https://geonode.wfp.org/layers/geonode%3Awld_bnd_adm0_wfp>

The repository must record the exact retrieval date and checksum when the
source snapshot is added. Code remains MIT-licensed; source and transformed
geographic data retain their applicable data license and attribution.

## Observed Source Shape

The available snapshot is approximately 8.61 MiB and contains 256 records,
2,007 polygon components, and 217,141 coordinate positions. It has 141
`Polygon` records and 115 `MultiPolygon` records. Not every record has a usable
or unique ISO3 code, so the source cannot itself define the playable catalog.

Each record contains geometry, a representative `geo_point_2d`, names, ISO
codes, status, region, and ownership-related metadata. The generator must use a
typed schema and reject unexpected required fields instead of parsing into an
unstructured runtime value.

## Catalog

`data/countries.toml` defines the stable set of 196 playable countries and maps
each one to a source geometry. It also defines canonical English names and
aliases. Dataset records outside that catalog remain non-playable unless an
explicit mapping says otherwise.

Country identifiers are generated from catalog order and stored in a type large
enough for reserved values. Water, neutral land, and invalid or missing data
must not collide with playable identifiers.

## Generation Pipeline

```text
source JSON
  -> parse typed records
  -> validate polygons and catalog mappings
  -> normalize antimeridian behavior
  -> retain identity per country or territory
  -> rasterize country coverage and borders
  -> select visual anchors
  -> compute adjacency and distance matrix
  -> validate all invariants
  -> write versioned binary asset
```

## Architectural Ownership

Source schemas, generator records, encoded asset sections, matrix indexes, and
the runtime decoder are external data concerns. They live in the world-data
tool or `infrastructure/world_data` and never become domain or application
models.

At runtime, `infrastructure/world_data/proximity.rs` owns the validated decoded
matrices and their private row-major lookup. Iteration 005 will adapt that data
to the `CountryCatalog` and `CountryProximity` domain ports. Raster cells,
borders, and visual anchors remain read-only renderer data exposed by
`infrastructure/world_data/map_data.rs`; they are not game-domain concepts.

This boundary allows the asset encoding and serialization technology to change
without changing game rules or application use cases.

Countries must never be globally unioned into a single land geometry. That
would discard political identity and shared borders.

## Map Projection and Raster

The MVP uses a Plate Carree/equirectangular map cropped below `60 degrees S`;
longitude maps linearly from -180 to 180 and latitude from 90 to -60. The
internal raster is `720 x 300`.

Each raster sample stores a country or reserved land/water identifier. A
one-cell border mask records coastlines and transitions between territories.
Rasterization uses coverage or supersampling rather than testing only the center
point, because a center-only low-resolution raster removes many small countries
and islands.

The raster is a rendering representation, not the source used for geographic
distance. Reducing terminal resolution may still hide small countries, so each
playable country also has a validated visual anchor that can be marked after it
is guessed.

## Map Assets

`world-v2.bin` is the deterministic equirectangular preprocessing and proximity
asset. Its `WHMP` version-2 layout has a 36-byte header followed by raster
identifiers, borders, anchors, a full row-major `u16` distance matrix, and a
full row-major `u8` adjacency matrix (`0` or `1`). Matrix order is the stable
catalog order and the runtime validates all lengths, diagonals, symmetry, and
adjacent/zero-distance consistency before it exposes the data. The active TUI
instead renders embedded Web Mercator OpenStreetMap basemap tiles, country-ID
overlay tiles, and a small details asset. Those representations share stable
catalog indexes but are not used for territorial distance calculations.

## Visual Anchors

An anchor is used for terminal markers and possible future labels. It is not a
distance centroid. The preferred anchor is an interior point on the principal
land component. The source `geo_point_2d` may be used only after validating its
semantics and location. Exceptional archipelagos may require curated anchors.

## Territorial Distance

For playable countries `A` and `B`:

```text
distance(A, B) = minimum geodesic distance between any territory in A and B
```

All polygon components in the mapped country record participate. Separate
dependency records are not automatically merged. Touching or overlapping
territories are adjacent and have zero territorial separation.

Euclidean distance in longitude and latitude is not valid for gameplay because
it distorts high latitudes and fails at the antimeridian. The generator uses
WGS84 ellipsoidal distances. It densifies source boundary segments at no more
than 5 km apart and indexes the resulting ECEF points in an R-tree before
evaluating the closest candidates. The initial acceptable approximation target
is within 10 km, confirmed against GeographicLib reference paths. Distances
round to the nearest whole kilometer; adjacency remains a separate exact
topology result, so a non-adjacent sub-kilometer gap can still encode as `0 km`.

Distances are rounded to whole kilometers and stored in a symmetric
`196 x 196` matrix of `u16` values. Adjacency is stored separately so the UI can
show `Borders target` instead of `0 km` for an incorrect neighboring guess.

## Required Invariants

- Exactly 196 playable countries exist.
- Every playable country resolves to valid geometry.
- Canonical names, ISO3 values, and normalized aliases are unambiguous.
- Every raster identifier is known or reserved.
- Every playable country has a valid visual anchor.
- Distance matrix dimensions match the catalog.
- Distance is symmetric and every diagonal entry is zero.
- Adjacency is symmetric.
- Adjacency diagonal values are false, and an adjacent pair has zero distance.
- Generation is deterministic.
- Antimeridian cases do not receive artificial world-spanning distances.

## Regeneration

The generator provides a normal mode that writes the assets and a `--check`
mode that regenerates in memory and compares against the committed result:

```sh
cargo run --release -p world-data -- generate --check
```
