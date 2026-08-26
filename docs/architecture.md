# Architecture

## Principles

- Keep the initial application in one primary Rust crate.
- Organize runtime code into domain, application, and infrastructure layers.
- Keep game rules independent of terminal, rendering, serialization, and data
  generation libraries.
- Make dependencies point inward: `infrastructure -> application -> domain`.
- Perform expensive geographic work before release, not at startup.
- Treat generated data as an external, versioned contract with validated
  invariants.
- Redraw only when presentation state or terminal dimensions change.
- Prefer explicit data and small modules over framework-style abstractions.

## Dependency Rules

The domain contains business concepts and rules. The application layer
orchestrates actions using domain objects and ports. Infrastructure translates
terminal input, renders output, decodes embedded data, and implements the ports
required by the inner layers.

The following rules are mandatory:

- `domain` does not import `application` or `infrastructure`.
- `application` may import `domain`, but not `infrastructure`.
- `infrastructure` may import both inner layers.
- Ratatui, Crossterm, binary serialization, generated asset structures, and
  concrete random-number libraries remain in infrastructure.
- Interfaces required by domain or application live as traits in
  `domain/ports/`; their concrete implementations live in infrastructure.
- Generated types and binary asset models never appear in domain or
  application public APIs.
- `main.rs` is the composition root and selects concrete implementations.
- A direct dependency from a TUI controller to an application use case is
  valid; no interface is required between them.

These boundaries do not require separate crates for the MVP. Rust module
visibility, focused public APIs, tests, and import review enforce them within
the root crate. Additional crates should be introduced only if the boundaries
become difficult to protect.

## Repository Shape

The planned structure is:

```text
worldhunt/
|-- Cargo.toml
|-- assets/
|   `-- world-v1.bin
|-- data/
|   |-- countries.toml
|   `-- source/world-boundaries.json
|-- src/
|   |-- main.rs
|   |-- lib.rs
|   |-- domain/
|   |   |-- mod.rs
|   |   |-- game.rs
|   |   |-- country.rs
|   |   |-- guess.rs
|   |   |-- proximity.rs
|   |   |-- errors/
|   |   `-- ports/
|   |       |-- country_catalog.rs
|   |       |-- country_proximity.rs
|   |       `-- target_selector.rs
|   |-- application/
|   |   |-- mod.rs
|   |   |-- start_game.rs
|   |   `-- submit_guess.rs
|   `-- infrastructure/
|       |-- mod.rs
|       |-- tui/
|       |   |-- app.rs
|       |   |-- input.rs
|       |   |-- layout.rs
|       |   |-- map.rs
|       |   `-- theme.rs
|       |-- world_data/
|       |   |-- decoder.rs
|       |   |-- catalog.rs
|       |   |-- proximity.rs
|       |   `-- map_data.rs
|       `-- random/
|-- tests/
|   |-- smoke.rs
|   `-- smoke/
`-- tools/
    `-- world-data/
```

The root package contains the shipped application. `tools/world-data` is a
separate workspace package used to regenerate and validate the embedded asset.
It is not part of the runtime dependency graph and its source or generated
models are not domain models.

## Domain Layer

The domain owns the stable business vocabulary and invariants:

- `country` defines `CountryId` and country-name concepts used by the game.
- `guess` defines accepted guesses and their clue information.
- `proximity` represents distance and shared-border semantics without exposing
  matrix indexes or encoded asset values.
- `game` owns the target, accepted guesses, win state, and legal transitions.
- `errors` contains typed business failures.
- `ports` contains only traits required across the domain boundary.

`Game` exposes intent-revealing behavior rather than writable fields. It does
not decode assets, resolve terminal text, select a concrete random number, or
query a binary matrix directly. It receives domain values prepared by the
application layer and applies rules such as duplicate rejection, completed-game
protection, attempt recording, and victory.

The initial ports are:

- `CountryCatalog`, which resolves validated country-name values and exposes
  the playable catalog through domain types.
- `CountryProximity`, which returns domain proximity values for two countries.
- `TargetSelector`, which chooses a target from eligible domain identifiers.

Ports may be refined during implementation, but they must not expose Ratatui,
Crossterm, serializer, generator, or raw asset types.

## Application Layer

Every request entering the core is represented by a concrete application use
case. A use case represents one action and exposes one public operation,
`dispatch`.

The MVP starts with:

- `StartGame`, which obtains the eligible catalog, selects a target through the
  `TargetSelector` port, and creates fresh game state.
- `SubmitGuess`, which resolves a country through `CountryCatalog`, obtains its
  clue through `CountryProximity`, and asks `Game` to apply the guess.

Use cases depend on domain traits, not infrastructure implementations. Their
inputs and outputs use domain objects or Value Objects. A simple TUI boundary
may construct such a value inline; complex transport mapping should remain in
infrastructure.

No command bus, CQRS framework, or interface in front of each use case is
required. If later application behavior genuinely coordinates several related
actions, it may become an application service instead of an oversized use case.

## Infrastructure Layer

### TUI

`infrastructure/tui` is the inbound and presentation adapter. `app` owns the
event loop and TUI-level state such as editable text, messages, focus, redraw
decisions, and quit state. It translates Crossterm events into calls to
application use cases and passes immutable state to the Ratatui renderer.

`input` handles key editing and terminal action mapping. Country-name
normalization and alias resolution are not keyboard concerns; they belong to
the domain-facing catalog behavior used by `SubmitGuess`.

`layout`, `map`, and `theme` compute responsive presentation and write to a
Ratatui buffer. Rendering may consume read-only map data and domain-facing game
state, but it never mutates the game or decides business rules.

### World Data

`infrastructure/world_data` embeds and decodes the versioned asset once:

- `decoder` validates magic, version, lengths, identifier ranges, and section
  consistency before exposing data.
- `catalog` implements the `CountryCatalog` port.
- `proximity` implements the `CountryProximity` port over the precomputed
  matrices.
- `map_data` exposes raster cells, anchors, and border flags to the TUI
  renderer without making them part of the domain model.

The adapter maps raw encoded identifiers and records into domain values at its
boundary. Domain code never parses source JSON, understands binary sections, or
receives generated structures.

### Random Target Selection

`infrastructure/random` implements `TargetSelector` with the selected random
number library. Tests can provide a deterministic implementation without
changing domain behavior.

## Data Flow

```text
Crossterm event
      |
      v
TUI controller
      |
      v
Application use case -----> domain ports <----- infrastructure implementations
      |                         |                            |
      v                         v                            v
    Game                  domain values              embedded world asset
      |
      v
TUI renderer ----------> Ratatui buffer ----------> terminal
```

`SubmitGuess` is the representative flow:

1. The TUI converts submitted text into the domain input expected by the use
   case.
2. `SubmitGuess` resolves it through `CountryCatalog`.
3. The use case rejects unresolved input without mutating the game.
4. It obtains distance and adjacency through `CountryProximity`.
5. `Game` validates the transition and records the accepted guess or victory.
6. The use case returns a typed result that the TUI maps to presentation state.

## Runtime State

Presentation and domain state remain distinct. A likely minimal model is:

```rust
struct TuiApp {
    game: Game,
    input: String,
    message: Option<Message>,
    should_quit: bool,
}

struct Game {
    target: CountryId,
    guesses: Vec<Guess>,
    status: GameStatus,
}
```

`TuiApp` is an infrastructure type. `Game`, `CountryId`, `Guess`, and
`GameStatus` are domain types. Exact public APIs will be selected during
iteration 005, while preserving these ownership and dependency rules.

## Event Loop

The MVP does not require Tokio or a background tick. The TUI polls or reads
Crossterm events, translates them into presentation actions or use-case calls,
and redraws only when needed. Resize events recalculate layout without changing
the game.

## Terminal Lifecycle

Terminal setup and teardown are infrastructure responsibilities and must be
guarded so raw mode, the alternate screen, cursor visibility, and mouse-related
modes are restored after normal exit, recoverable errors, and panics.
Initialization must clean up completed steps if a later setup step fails.

## Error Handling

Expected failures such as an unknown country, duplicate guess, or submission
after victory are typed domain or application results. Infrastructure maps them
to concise UI messages. Domain and application code do not terminate the
process or produce terminal-specific errors.

Asset corruption, terminal initialization failure, and I/O failure are
infrastructure errors with technical context. They may stop the application at
the outer boundary but must not leak binary layout, library, or terminal details
into domain errors.

## Generated Asset Contract

The asset has a magic value and format version. Its infrastructure decoder
validates lengths, identifier ranges, and version compatibility before mapping
data to domain-facing values or renderer map data. The generator writes
deterministically so CI can compare regenerated output byte for byte.

Serialization technology will be selected during iteration 003 after measuring
size and decode complexity. A compact uncompressed binary is preferred if the
full asset remains small enough; compression is not a requirement by itself.
