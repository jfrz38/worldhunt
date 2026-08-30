# Iteration 006: Map Rendering

Status: Completed
Started: 2026-08-30
Completed: 2026-08-30

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
- [x] Add colocated renderer unit tests with semantic assertions and selected
  stable snapshots.
- [x] Measure render time at representative terminal sizes.

## Acceptance Criteria

- [x] The map has correct orientation and recognizable continents.
- [x] It remains centered and proportionally fitted after resize.
- [x] Every guessed country uses the color band for its absolute distance.
- [x] Earlier guesses remain colored after later guesses.
- [x] The correct target is visually distinct from an adjacent incorrect guess.
- [x] Small guessed countries remain visible through raster coverage or anchors.
- [x] Truecolor, ANSI 256-color, and monochrome output remain understandable.
- [x] A terminal below the minimum receives a clear resize message.
- [x] Ratatui `TestBackend` unit-test rendering is deterministic.
- [x] Rendering does not mutate `Game` or decide business transitions.

## Verification

- `cargo clippy --all-targets --all-features -- -D warnings`: passed on
  2026-08-30.
- `cargo test --workspace`: passed on 2026-08-30 (36 world-data and 46 runtime
  tests).
- `make check`: passed on 2026-08-30, including catalog validation and
  deterministic generated-asset verification.
- Renderer behavior was visually validated during implementation; the complete
  gameplay smoke test belongs to iteration 007.

## Decisions

- `WORLDHUNT_COLOR=truecolor|ansi256|mono` is an explicit diagnostic override;
  `NO_COLOR` forces monochrome when no override is supplied.

## Deviations

None yet.

## Outcome

Renderer implementation, renderer tests, and manual visual validation are
complete. Gameplay integration continues in iteration 007.
