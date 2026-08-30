# Iteration 005: Game Domain

Status: Completed
Started: 2026-08-30
Completed: 2026-08-30

## Goal

Implement a complete, deterministic, terminal-independent domain and the
application use cases that orchestrate it through domain ports.

## Dependencies

- Iteration 004 provides stable country identifiers, adjacency, and distance
  lookup.
- [Product specification](../product.md) defines the MVP rules.

## Scope

- Target selection.
- English name and alias resolution.
- Guess submission and attempt history.
- Duplicate and unknown-input behavior.
- Distance and shared-border clues.
- Win state and new-game behavior.
- Deterministic testing through injected randomness.
- `StartGame` and `SubmitGuess` application use cases.
- Domain ports for country catalog, proximity, and target selection.

## Out of Scope

- Rendering, terminal events, persistence, statistics, and daily games.
- Fuzzy matching and localized country names.

## Tasks

- [x] Define `CountryId`, country-name values, `Guess`, `Proximity`,
  `GameStatus`, and typed business outcomes in `domain/`.
- [x] Define `CountryCatalog`, `CountryProximity`, and `TargetSelector` traits in
  `domain/ports/` without infrastructure types.
- [x] Implement the catalog and proximity ports in
  `infrastructure/world_data/`, mapping decoded records to domain values.
- [x] Implement `TargetSelector` in `infrastructure/random/` without exposing
  the concrete random library to the inner layers.
- [x] Implement canonical name and alias normalization behind the
  domain-facing catalog behavior.
- [x] Implement `StartGame::dispatch` over the playable catalog and an injected
  `TargetSelector`.
- [x] Implement `SubmitGuess::dispatch` to resolve a country, obtain proximity,
  and invoke domain behavior.
- [x] Implement guess transitions in `Game` without terminal, UI, asset, or
  concrete randomness dependencies.
- [x] Reject empty, unknown, and repeated guesses with typed outcomes.
- [x] Record distance and adjacency on accepted guesses.
- [x] Transition to a win state when the target is guessed.
- [x] Prevent completed games from accepting normal guesses.
- [x] Implement new-game reset behavior.
- [x] Add focused colocated domain unit tests and public API documentation where
  useful.
- [x] Add colocated application unit tests using deterministic fake port
  implementations.

## Acceptance Criteria

- [x] A full game can be played entirely through library calls.
- [x] Domain and application modules have no Ratatui, Crossterm, serializer,
  generated asset, or concrete random-library dependency.
- [x] Each application use case represents one action and exposes only
  `dispatch` as its public operation.
- [x] Every accepted guess resolves to one stable country identifier.
- [x] Duplicate and invalid guesses do not change attempt history.
- [x] Adjacent countries expose a distinct border clue.
- [x] Guessing the target produces one unambiguous win transition.
- [x] Seeded tests produce deterministic target sequences.
- [x] Starting a new game clears all previous transient game state.
- [x] Format, Clippy, and tests pass.

## Verification

- `cargo test --workspace`: passed on 2026-08-30 (36 world-data and 39 runtime
  tests).
- `make ci`: formatting, Clippy, tests, catalog validation, deterministic asset
  generation, and workspace build passed on 2026-08-30.

## Decisions

- `WorldData` decodes the version-2 asset once and retains its renderer data,
  private proximity matrix, and a runtime catalog adapter. The asset remains
  unchanged; the adapter embeds and parses the authoritative TOML catalog. Its
  stable order is the binary matrix contract, and `make data-check` detects
  drift by deterministically regenerating the asset.
- A winning guess is recorded as an accepted history item with the distinct
  `Target` clue. Starting a new game creates a fresh `Game`.

## Deviations

None yet.

## Outcome

Implemented, merged into `develop`, and verified before iteration 006 began.
