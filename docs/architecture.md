# Architecture

## Principles

- Keep the release application in one primary Rust crate.
- Organize runtime code into domain, application, and infrastructure layers.
- Keep game rules independent of terminal, rendering, serialization, and data-generation libraries.
- Make dependencies point inward: `infrastructure -> application -> domain`.
- Perform expensive geographic work before release, not at startup.
- Treat generated data as a versioned contract with validated invariants.
- Prefer explicit data and small modules over framework-style abstractions.

## Dependency Rules

The domain contains game concepts and rules. The application layer orchestrates actions through domain ports. Infrastructure translates terminal input, renders output, decodes embedded data, and implements the ports required by the inner layers.

- `domain` does not import `application` or `infrastructure`.
- `application` may import `domain`, but not `infrastructure`.
- `infrastructure` may import both inner layers.
- Ratatui, Crossterm, binary decoding, generated assets, and random-number libraries remain in infrastructure.
- Traits required across the boundary live in `domain/ports`; concrete implementations live in infrastructure.
- `main.rs` is the composition root.

These boundaries are enforced by Rust module visibility, focused public APIs, tests, and import review. Separate crates are unnecessary while those boundaries remain easy to protect.

## Repository Shape

```text
worldhunt/
|-- assets/                 # Embedded world, country-overlay, and detail assets
|-- data/
|   |-- countries.toml      # Catalog and aliases
|   `-- source/             # Provenance metadata and embedded OSM tiles
|-- src/
|   |-- domain/             # Game rules, values, errors, and ports
|   |-- application/        # Start-game and submit-guess use cases
|   `-- infrastructure/     # TUI, embedded world data, and random selection
|-- tests/                  # Public smoke coverage
`-- tools/world-data/       # Unpublished deterministic data generator
```

The root package contains the shipped application. `tools/world-data` is an unpublished workspace package used to validate and regenerate embedded assets; it is not a runtime dependency.

## Runtime Flow

```text
Crossterm event -> TUI controller -> application use case -> domain ports
                                                        -> embedded world data
TUI renderer -> Ratatui buffer -> terminal
```

`TuiApp` owns editable input, messages, redraw decisions, and quit state. It translates terminal events into presentation actions and application use-case calls. Rendering consumes immutable map and game state; it does not mutate the game or decide business rules.

`Game` owns the target, accepted guesses, completion state, and legal transitions. `StartGame` selects an eligible target. `SubmitGuess` resolves catalog input, obtains proximity, and applies the result to `Game`.

## Embedded Data

`infrastructure/world_data` decodes the validated `world-v2.bin` asset and implements `CountryCatalog` and `CountryProximity`. The active map embeds offline OpenStreetMap vector tiles, country-ID overlay tiles, visual anchors, and small geographic details. Map identifiers use the catalog's stable indexes but do not leak into domain APIs.

The decoder validates magic values, versions, section lengths, identifier ranges, anchor bounds, matrix symmetry, and adjacency invariants before exposing domain-facing data. The generator writes deterministic output, so CI can compare regenerated assets byte for byte.

## Terminal Lifecycle

Terminal setup and teardown are infrastructure responsibilities. Raw mode, alternate screen, cursor visibility, and mouse capture are restored after normal exit, recoverable setup failures, and panics. The event loop redraws only after presentation changes or terminal resize events.
