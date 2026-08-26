# Iteration 006: Map Rendering

Status: Planned  
Started:  
Completed:

## Goal

Render a recognizable, responsive world heat map whose visual clues remain
usable across modern terminal color capabilities.

## Dependencies

- Iteration 003 provides the raster, borders, and anchors.
- Iteration 005 provides game state and accepted guesses.
- [TUI design](../tui-design.md) defines visual behavior.

## Scope

- Responsive map viewport and aspect-ratio fitting.
- Half-block rendering with two vertical samples per terminal cell.
- Country styles based on current game state.
- Truecolor, ANSI 256-color, and monochrome themes.
- Distance bands, borders, neutral territories, water, and win color.
- Downsampling and anchor markers for guessed small countries.
- Test-backend coverage for representative sizes and states.
- TUI renderer modules contained in `infrastructure/tui`.

## Out of Scope

- Text input, complete attempt panel, zoom, animation, and mouse interaction.
- Alternative projections or a globe.

## Tasks

- [ ] Implement layout, map, and theme modules under `infrastructure/tui`.
- [ ] Define renderer inputs that expose neither mutable domain state nor
  application internals.
- [ ] Fit the `2:1` map into an arbitrary Ratatui rectangle.
- [ ] Sample two vertical map pixels into each `▀` cell.
- [ ] Preserve country identity while downsampling the source raster.
- [ ] Render water, unguessed land, neutral land, and borders distinctly.
- [ ] Define stable absolute distance bands.
- [ ] Implement and compare truecolor and ANSI 256-color palettes.
- [ ] Honor `NO_COLOR` with a usable monochrome strategy.
- [ ] Render the winning target with a distinct non-red style.
- [ ] Add anchor markers for guessed countries absent from the sampled raster.
- [ ] Define below-minimum-size rendering behavior.
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

Not run; iteration has not started.

## Decisions

None yet.

## Deviations

None yet.

## Outcome

Pending.
