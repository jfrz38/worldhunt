# WorldHunt Documentation

This directory is the source of truth for WorldHunt's product scope, technical
design, architectural decisions, and implementation progress.

## Current Status

- Project phase: implementation
- Active iteration: [007 - Playable TUI](iterations/007-playable-tui.md)
- Next iteration: [008 - Hardening](iterations/008-hardening.md)
- MVP target: a fully local, cross-platform terminal geography game

Only one iteration may be `In Progress` at a time. An iteration becomes active
when implementation starts, not when its plan is written.

## Specifications

| Document | Purpose |
| --- | --- |
| [Product](product.md) | Product behavior, MVP scope, and exclusions |
| [Architecture](architecture.md) | Hexagonal layers, dependency rules, runtime structure, and data flow |
| [World data](world-data.md) | Geographic source, preprocessing, raster, and distances |
| [TUI design](tui-design.md) | Layout, controls, map rendering, and color behavior |
| [Testing](testing.md) | Unit and smoke test structure, geographic invariants, and CI expectations |

## Accepted Decisions

| ADR | Decision | Status |
| --- | --- | --- |
| [0001](decisions/0001-flat-map.md) | Use a flat equirectangular map | Superseded |
| [0002](decisions/0002-embedded-world-data.md) | Embed preprocessed world data | Accepted |
| [0003](decisions/0003-country-distance.md) | Measure minimum geodesic territory distance | Accepted |
| [0004](decisions/0004-playable-countries.md) | Use a curated catalog of 196 playable countries | Accepted |
| [0005](decisions/0005-zoomable-web-mercator-map.md) | Use a zoomable Web Mercator map | Accepted |

## Iterations

| Iteration | Goal | Status |
| --- | --- | --- |
| [001](iterations/001-project-foundation.md) | Establish the Rust project and safe terminal lifecycle | Completed |
| [002](iterations/002-country-catalog.md) | Define countries, aliases, provenance, and licenses | Completed |
| [003](iterations/003-map-data-generator.md) | Generate validated map assets and country overlays | Completed |
| [004](iterations/004-distance-calculation.md) | Precompute territorial distances and adjacency | Completed |
| [005](iterations/005-game-domain.md) | Implement the game independently of the TUI | Completed |
| [006](iterations/006-map-rendering.md) | Render a responsive colored terminal map | Completed |
| [007](iterations/007-playable-tui.md) | Integrate input, attempts, victory, and new games | In Progress |
| [008](iterations/008-hardening.md) | Harden behavior, tests, performance, and portability | Blocked |
| [009](iterations/009-mvp-release.md) | Prepare the first distributable release | Planned |

Iteration status values are `Planned`, `In Progress`, `Blocked`, `Completed`,
and `Superseded`. See [the iteration workflow](iterations/README.md) for update
rules, branch conventions, and the standard template.
