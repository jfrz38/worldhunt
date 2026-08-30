# Iteration 007: Playable TUI

Status: In Progress
Started: 2026-08-30
Completed:

## Goal

Integrate the game and map into the complete keyboard-driven MVP experience.

## Dependencies

- Iteration 005 provides game behavior.
- Iteration 006 provides responsive map rendering and themes.
- Iteration 001 provides the terminal lifecycle and event loop.

## Scope

- `TuiApp` presentation state and keyboard action handling in infrastructure.
- Composition of use cases and concrete port implementations in `main.rs`.
- Editable country input.
- Guess submission and recoverable messages.
- Wide and narrow layouts.
- Attempt history with distances and border clues.
- Victory presentation, new game, and quit actions.
- Resize behavior preserving the active game.

## Out of Scope

- Persistence, statistics, daily mode, mouse input, and packaging.
- Search suggestions and autocomplete popups.

## Tasks

- [x] Compose `StartGame`, `SubmitGuess`, and concrete world-data and random
  implementations in `main.rs`.
- [x] Implement `TuiApp` in `infrastructure/tui/app.rs` with presentation state,
  event handling, and calls to the application use cases.
- [x] Keep editable key handling in `infrastructure/tui/input.rs` and country
  normalization or alias resolution behind `CountryCatalog`.
- [x] Implement printable text input and Unicode-safe backspace behavior.
- [x] Submit guesses on `Enter` and clear input after accepted guesses.
- [x] Display unknown, empty, and duplicate input as non-fatal messages.
- [x] Implement wide map-plus-history layout.
- [x] Implement narrow stacked layout.
- [x] Keep recent attempts visible when the history exceeds its viewport.
- [x] Show kilometers and `Borders target` consistently.
- [x] Display target name and attempt count after victory.
- [x] Enable `N` for a new game only in the post-win state.
- [x] Preserve game and input state across resize events.
- [x] Ensure `Esc` and `Ctrl+C` work from every state.
- [x] Add unit tests for event-to-action mapping and TUI presentation-state
  transitions, using fake domain ports where isolation is needed.
- [x] Add the first automated smoke target for startup with the embedded asset
  and initial-frame rendering when it can run reliably without a real TTY.
- [ ] Perform a complete manual game smoke test.

## Acceptance Criteria

- [ ] A user can complete multiple consecutive games without restarting.
- [ ] Unknown and repeated input never terminates or corrupts the game.
- [ ] The map and attempt history agree on every submitted country.
- [ ] Shared-border guesses display text rather than misleading `0 km`.
- [ ] Wide and narrow supported terminals remain usable.
- [ ] Resize does not reset target, guesses, input, or win state.
- [ ] Victory offers clear new-game and quit actions.
- [ ] The application requires no network or external runtime data file.
- [ ] Terminal cleanup still passes all lifecycle checks.
- [ ] Automated and manual smoke checks remain short and use production wiring.
- [ ] The TUI depends inward on application and domain; neither inner layer
  imports TUI or world-data implementation modules.

## Verification

- `make check`: passed on 2026-08-30, including format, Clippy with warnings
  denied, 91 unit and smoke tests, catalog validation, and deterministic asset
  verification.
- Manual interactive and visual smoke test: pending in a real terminal.

## Decisions

None yet.

## Deviations

None yet.

## Outcome

Pending.
