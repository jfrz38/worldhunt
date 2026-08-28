# ADR 0005: Use a Zoomable Web Mercator Map

Status: Accepted
Date: 2026-08-28

## Context

Iteration 003 proved a Web Mercator vector basemap and country overlay more
recognizable than the planned fixed equirectangular raster at terminal sizes.
It also introduced Braille rendering, zoom, and pan. The implementation and
the previous MVP specification now disagree with ADR 0001.

## Decision

The MVP uses an offline, zoomable Web Mercator map rendered with Unicode
Braille cells. The renderer embeds OpenStreetMap vector tiles for the basemap,
versioned country-ID overlay tiles for game coloring, and a small detail asset
for geographic corrections. Zoom and pan are MVP controls.

The raster `world-v1.bin` remains an embedded, validated preprocessing asset.
It supplies stable country identities and anchors to generation tooling and is
reserved for future proximity sections; it is not the active TUI basemap.

The map remains entirely local at runtime. OpenStreetMap attribution is shown
in the TUI and recorded in third-party notices.

## Consequences

- The renderer can show a detailed, navigable map while country overlays retain
  stable catalog indexes.
- Braille cells provide a 2 by 4 sample grid per terminal cell.
- The MVP must test zoom, pan, resize, and longitude wrapping.
- Web Mercator still must not be used to calculate geographic distance.
- The ODbL source requires ongoing attribution and licence review for release.

## Supersedes

This decision supersedes ADR 0001's projection and fixed-map rendering choice.
ADR 0001 remains as the historical record of the earlier approach.
