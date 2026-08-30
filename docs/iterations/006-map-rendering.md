# Iteration 006: Map Rendering

Status: In Progress
Started: 2026-08-30
Completed:

## Goal

Render a recognizable, responsive world heat map whose visual clues remain
usable across modern terminal color capabilities.

## Dependencies

- Iteration 003 provides the offline Web Mercator basemap, country overlays,
  details, and anchors.
- Iteration 005 provides game state and accepted guesses.
- [TUI design](../tui-design.md) defines visual behavior.

## Scope

- Responsive map viewport and aspect-ratio fitting.
- Braille rendering with eight samples per terminal cell.
- Country styles based on current game state.
- Truecolor, ANSI 256-color, and monochrome themes.
- Distance bands, borders, neutral territories, water, and win color.
- Downsampling and anchor markers for guessed small countries.
- Test-backend coverage for representative sizes and states.
- TUI renderer modules contained in `infrastructure/tui`.

## Out of Scope

- Text input, complete attempt panel, animation, and mouse interaction.
- Alternative projections or a globe.

## Tasks

- [x] Implement layout, map, and theme modules under `infrastructure/tui`.
- [x] Define renderer inputs that expose neither mutable domain state nor
  application internals.
- [x] Fit the current Web Mercator viewport into an arbitrary Ratatui rectangle.
- [x] Sample each map cell into a 2 by 4 Braille grid.
- [x] Preserve country identity while downsampling the source raster.
- [x] Render water, unguessed land, neutral land, and borders distinctly.
- [x] Define stable absolute distance bands.
- [x] Implement and compare truecolor and ANSI 256-color palettes.
- [x] Honor `NO_COLOR` with a usable monochrome strategy.
- [x] Render the winning target with a distinct non-red style.
- [x] Add anchor markers for guessed countries absent from the sampled raster.
- [x] Define below-minimum-size rendering behavior.
- [ ] Add colocated renderer unit tests with semantic assertions and selected
  stable snapshots.
- [ ] Measure render time at representative terminal sizes.

## Acceptance Criteria

- [ ] The map has correct orientation and recognizable continents.
- [ ] It remains centered and proportionally fitted after resize.
- [ ] Every guessed country uses the color band for its absolute distance.
- [ ] Earlier guesses remain colored after later guesses.
- [ ] The correct target is visually distinct from an adjacent incorrect guess.
- [ ] Small guessed countries remain visible through raster coverage or anchors.
- [ ] Truecolor, ANSI 256-color, and monochrome output remain understandable.
- [ ] A terminal below the minimum receives a clear resize message.
- [ ] Ratatui `TestBackend` unit-test rendering is deterministic.
- [ ] Rendering does not mutate `Game` or decide business transitions.

## Verification

- `cargo clippy --all-targets --all-features -- -D warnings`: passed on
  2026-08-30.
- `cargo test --workspace`: passed on 2026-08-30 (36 world-data and 46 runtime
  tests).
- `make check`: passed on 2026-08-30, including catalog validation and
  deterministic generated-asset verification.
- Manual visual and performance checks remain pending.

## Decisions

- `WORLDHUNT_COLOR=truecolor|ansi256|mono` is an explicit diagnostic override;
  `NO_COLOR` forces monochrome when no override is supplied.

## Deviations

None yet.

## Outcome

Renderer implementation and automated verification are complete. Manual visual
validation and representative release-build render measurements remain before
the iteration can be completed.
