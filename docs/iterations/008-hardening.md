# Iteration 008: Hardening

Status: Planned  
Started:  
Completed:

## Goal

Remove correctness, portability, accessibility, and operational risks before
calling the local game release-ready.

## Dependencies

- Iteration 007 provides a complete playable vertical slice.
- [Testing strategy](../testing.md) defines required coverage.

## Scope

- Cross-platform compilation and manual terminal checks.
- Geographic and generated-data edge cases.
- Terminal lifecycle failure paths.
- Responsive UI regression coverage.
- Color accessibility and monochrome usability.
- Startup, memory, render, asset, and binary measurements.
- Documentation correction based on actual implementation.

## Out of Scope

- New game modes and convenience features not required by the MVP.
- Release publication and version tagging.

## Tasks

- [ ] Run all quality checks with warnings denied.
- [ ] Expand unit tests for synthetic polygon, island, hole, pole, and
  antimeridian behavior.
- [ ] Audit all asset decoder length and identifier checks.
- [ ] Test setup rollback and teardown after injected terminal failures.
- [ ] Exercise minimum, narrow, wide, and unusually large terminal dimensions.
- [ ] Verify long country names, aliases, and long attempt histories.
- [ ] Review palettes for ordered contrast and common color-vision deficiencies.
- [ ] Verify the game remains playable with `NO_COLOR`.
- [ ] Build and smoke-test on Windows, Linux, and macOS terminals.
- [ ] Measure release startup time, memory, render time, asset size, and binary
  size.
- [ ] Define justified performance limits from the measurements.
- [ ] Run a focused diff review for regressions and accidental artifacts.
- [ ] Reconcile architecture, world-data, TUI, and testing docs with reality.

## Acceptance Criteria

- [ ] All unit tests, automated smoke tests, and data checks pass consistently.
- [ ] Format and Clippy pass with no warnings.
- [ ] CI compiles on Windows, Linux, and macOS.
- [ ] Manual smoke tests pass on all three target platforms.
- [ ] Every tested exit and failure path restores the terminal.
- [ ] Monochrome mode communicates clues through text and non-color styling.
- [ ] Performance measurements and limits are recorded in the outcome.
- [ ] No unresolved high-severity correctness or operational issue remains.
- [ ] Documentation accurately describes implemented behavior and commands.

## Verification

Not run; iteration has not started.

## Decisions

None yet.

## Deviations

None yet.

## Outcome

Pending.
