# ADR 0001: Use a Flat World Map

Status: Accepted  
Date: 2026-08-26

## Context

WorldHunt needs to show all guessed countries and their relative heat at the
same time. A terminal has limited resolution, non-square character cells, and
widely varying dimensions. The initial alternatives were a rotatable ASCII
globe and a flat world map.

## Decision

The MVP will use a flat Plate Carree/equirectangular world map. It will be
rendered from a preprocessed raster and fitted responsively into the terminal.
Unicode half-blocks will improve vertical resolution.

## Consequences

- The entire world and all previous guesses remain visible simultaneously.
- Projection and raster sampling are simple and deterministic.
- The map adapts to small and resized terminals.
- Area and shape distortion increases toward the poles.
- Small countries still require visual anchors.
- The rendering representation must not be used for geographic distance.

## Alternatives Considered

### Rotatable globe

A globe is visually distinctive and avoids presenting the whole Earth through
one flat projection, but it hides half the countries, requires rotation, and
adds spherical projection and occlusion work before the core game is proven.

### Robinson or Natural Earth projection

These projections produce a more familiar atlas shape but require more complex
rasterization and do not remove the need for separate geodesic calculations.
They may be reconsidered after the MVP.
