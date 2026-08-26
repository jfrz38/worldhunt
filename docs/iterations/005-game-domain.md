# Iteration 005: Game Domain

Status: Planned  
Started:  
Completed:

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

- [ ] Define `CountryId`, country-name values, `Guess`, `Proximity`,
  `GameStatus`, and typed business outcomes in `domain/`.
- [ ] Define `CountryCatalog`, `CountryProximity`, and `TargetSelector` traits in
  `domain/ports/` without infrastructure types.
- [ ] Implement the catalog and proximity ports in
  `infrastructure/world_data/`, mapping decoded records to domain values.
- [ ] Implement `TargetSelector` in `infrastructure/random/` without exposing
  the concrete random library to the inner layers.
- [ ] Implement canonical name and alias normalization behind the
  domain-facing catalog behavior.
- [ ] Implement `StartGame::dispatch` over the playable catalog and an injected
  `TargetSelector`.
- [ ] Implement `SubmitGuess::dispatch` to resolve a country, obtain proximity,
  and invoke domain behavior.
- [ ] Implement guess transitions in `Game` without terminal, UI, asset, or
  concrete randomness dependencies.
- [ ] Reject empty, unknown, and repeated guesses with typed outcomes.
- [ ] Record distance and adjacency on accepted guesses.
- [ ] Transition to a win state when the target is guessed.
- [ ] Prevent completed games from accepting normal guesses.
- [ ] Implement new-game reset behavior.
- [ ] Add focused colocated domain unit tests and public API documentation where
  useful.
- [ ] Add colocated application unit tests using deterministic fake port
  implementations.

## Acceptance Criteria

- [ ] A full game can be played entirely through library calls.
- [ ] Domain and application modules have no Ratatui, Crossterm, serializer,
  generated asset, or concrete random-library dependency.
- [ ] Each application use case represents one action and exposes only
  `dispatch` as its public operation.
- [ ] Every accepted guess resolves to one stable country identifier.
- [ ] Duplicate and invalid guesses do not change attempt history.
- [ ] Adjacent countries expose a distinct border clue.
- [ ] Guessing the target produces one unambiguous win transition.
- [ ] Seeded tests produce deterministic target sequences.
- [ ] Starting a new game clears all previous transient game state.
- [ ] Format, Clippy, and tests pass.

## Verification

Not run; iteration has not started.

## Decisions

None yet.

## Deviations

None yet.

## Outcome

Pending.
