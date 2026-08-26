# WorldHunt Documentation

This directory is the source of truth for WorldHunt's product scope, technical
design, architectural decisions, and implementation progress.

## Current Status

- Project phase: implementation
- Active iteration: [001 - Project foundation](iterations/001-project-foundation.md)
- Next iteration: [002 - Country catalog](iterations/002-country-catalog.md)
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
| [0001](decisions/0001-flat-map.md) | Use a flat equirectangular map | Accepted |
| [0002](decisions/0002-embedded-world-data.md) | Embed preprocessed world data | Accepted |
| [0003](decisions/0003-country-distance.md) | Measure minimum geodesic territory distance | Accepted |
| [0004](decisions/0004-playable-countries.md) | Use a curated catalog of 195 playable countries | Accepted |

## Iterations

| Iteration | Goal | Status |
| --- | --- | --- |
| [001](iterations/001-project-foundation.md) | Establish the Rust project and safe terminal lifecycle | In Progress |
| [002](iterations/002-country-catalog.md) | Define countries, aliases, provenance, and licenses | Planned |
| [003](iterations/003-map-data-generator.md) | Generate an identity-preserving world raster | Planned |
| [004](iterations/004-distance-calculation.md) | Precompute territorial distances and adjacency | Planned |
| [005](iterations/005-game-domain.md) | Implement the game independently of the TUI | Planned |
| [006](iterations/006-map-rendering.md) | Render a responsive colored terminal map | Planned |
| [007](iterations/007-playable-tui.md) | Integrate input, attempts, victory, and new games | Planned |
| [008](iterations/008-hardening.md) | Harden behavior, tests, performance, and portability | Planned |
| [009](iterations/009-mvp-release.md) | Prepare the first distributable release | Planned |

Iteration status values are `Planned`, `In Progress`, `Blocked`, `Completed`,
and `Superseded`. See [the iteration workflow](iterations/README.md) for update
rules, branch conventions, and the standard template.
