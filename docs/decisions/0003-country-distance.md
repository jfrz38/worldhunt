# ADR 0003: Measure Minimum Geodesic Territory Distance

Status: Accepted  
Date: 2026-08-26

## Context

The game's heat clue needs one distance between a guessed country and the
target. Possible definitions include projected Euclidean distance, center-to-
center geodesic distance, and minimum distance between territorial boundaries.

Countries may be large, elongated, cross the antimeridian, or consist of many
islands. A calculated centroid may lie in the ocean and can make neighboring
countries appear far apart.

## Decision

Distance is the minimum geodesic distance between any participating territory
of the two countries. All polygon components in each mapped country record are
included. Countries that touch or overlap are stored as adjacent and have zero
territorial separation.

Distance and adjacency are precomputed from detailed geometry. The game shows
`Borders target` instead of `0 km` for an incorrect adjacent guess.

## Consequences

- Islands naturally contribute through their nearest component.
- Neighboring countries receive the strongest possible proximity clue.
- The antimeridian and Earth curvature must be handled explicitly.
- Results such as the proximity of the United States and Russia are
  geographically correct but may surprise players.
- A centroid is still useful as a visual anchor, but not for gameplay distance.
- Distance generation is more complex than point-to-point Haversine distance.

## Alternatives Considered

### Euclidean longitude and latitude

This is inexpensive but geometrically invalid, particularly near the poles and
longitude 180.

### Country center to country center

This is easy to explain and calculate, but creates arbitrary results for large
countries and archipelagos and does not make shared borders distance zero.

### Capital-to-capital distance

This measures cities rather than territories and requires another curated data
source. It does not match the visual map clue.

### Mainland-only distance

This avoids some surprising island results but requires subjective exceptions
and misrepresents island states. The MVP uses all mapped components instead.
