# Iteration 007: Playable TUI

Status: Blocked
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
- Prefix country suggestions and `Tab` completion.
- Guess submission and recoverable messages.
- Center the map on each accepted guess without changing zoom.
- Wide and narrow layouts.
- Attempt history with distances and border clues.
- Victory or surrender presentation, hints, new game, and quit actions.
- Resize behavior preserving the active game.

## Out of Scope

- Persistence, statistics, daily mode, mouse input, and packaging.

## Tasks

- [x] Compose `StartGame`, `SubmitGuess`, and concrete world-data and random
  implementations in `main.rs`.
- [x] Implement `TuiApp` in `infrastructure/tui/app.rs` with presentation state,
  event handling, and calls to the application use cases.
- [x] Keep editable key handling in `infrastructure/tui/input.rs` and country
  normalization or alias resolution behind `CountryCatalog`.
- [x] Implement printable text input and Unicode-safe backspace behavior.
- [x] Show up to five canonical prefix suggestions after two normalized input characters.
- [x] Render suggestions inline with the editable input rather than in a separate status area.
- [x] Complete the first suggestion with `Tab` without changing `Enter` submission.
- [x] Submit guesses on `Enter` and clear input after accepted guesses.
- [x] Display unknown, empty, and duplicate input as non-fatal messages.
- [x] Implement wide map-plus-history layout.
- [x] Implement narrow stacked layout.
- [x] Keep recent attempts visible when the history exceeds its viewport.
- [x] Show kilometers and `Borders target` consistently.
- [x] Display target name and attempt count after victory.
- [x] Support `/hint` without recording an attempt and `/surrender` without recording a win.
- [x] Enable `N` for a new game after victory or surrender.
- [x] Preserve game and input state across resize events.
- [x] Center accepted guesses on their visual anchors while preserving zoom.
- [x] Render anchor fallbacks from final Braille-cell visibility.
- [x] Preserve Spain's country identity for Canary Island detail geometry.
- [x] Keep a valid country sample when the other Braille dots are water.
- [x] Generate map anchors from the primary source polygon and validate them against raster ownership.
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
- [ ] Hints and surrender explain their outcome without affecting accepted guesses.
- [ ] The application requires no network or external runtime data file.
- [ ] Terminal cleanup still passes all lifecycle checks.
- [ ] Automated and manual smoke checks remain short and use production wiring.
- [ ] The TUI depends inward on application and domain; neither inner layer
  imports TUI or world-data implementation modules.

## Verification

- `make check`: passed on 2026-08-30, including format, Clippy with warnings
  denied, 112 unit and smoke tests, catalog validation, and deterministic asset
  verification.
- Manual interactive and visual smoke test: pending in a real terminal.

## Decisions

- Search suggestions were brought into scope after manual gameplay validation
  showed that they are necessary for a practical country-entry flow. Suggestions
  remain a bounded prefix search; they do not introduce fuzzy matching or
  arrow-key selection.
- Geographic detail polygons carry their owning country identifier so detached
  components, including the Canary Islands, receive guess colors.
- Suggestions are shown beside the current input, while the status panel keeps
  controls and recoverable messages visible.
- Hints reveal successive alphabetic characters of the canonical target name;
  surrender reveals the target and ends only the TUI session, not the domain game.

## Deviations

- The initial map fallback considered a country visible when it occupied any
  Braille subpixel. It now evaluates final cell ownership, avoiding camera-grid
  dependent disappearance for small territories such as Palestine.
- Raster-derived anchors could select remote components of large or oceanic
  countries. Primary source-polygon anchors now guide the choice and must map
  back to an owned raster cell.

## Outcome

Pending.
