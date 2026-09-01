# Iteration 008: Hardening

Status: Blocked
Started: 2026-09-01
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

- [x] Run all quality checks with warnings denied.
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

- `make check`: passed on 2026-09-01: format, Clippy with warnings denied,
  130 tests (one ignored engineering measurement), catalog validation, and
  deterministic asset regeneration.
- `make ci`: passed on 2026-09-01, including all workspace targets.
- `cargo test --release --test performance -- --ignored --nocapture`: passed on
  2026-09-01. Windows 11 Pro 64-bit (10.0.26200), Intel i7-10750H, 15.8 GiB
  RAM, commit `5213dcf1ee7e1bbc39592e4a0602153595109fdc`. Decode p95 was
  3.47 ms; render p95 was 20.83 ms (48x20), 33.01 ms (70x30), 30.03 ms
  (100x30), and 37.94 ms (200x60). `assets/world-v2.bin` is 764,068 bytes and
  `worldhunt.exe` is 2,750,976 bytes. Peak RSS and real-TTY first-frame timing
  remain pending manual measurement.
- Manual interactive smoke for iteration 007: pending in a real TTY. It blocks
  the status transition to `In Progress` and all completion claims for 008.

## Decisions

- The decoder validates binary structure, encoded identifiers, anchor bounds,
  and proximity invariants. Raster ownership of anchors remains a generator
  invariant rather than a runtime requirement because valid source-selected
  anchors can fall outside the reduced raster.
- The ignored performance harness uses only public production APIs and reports
  median and p95 from 20 post-warmup samples. It is intentionally excluded from
  `make check` until platform measurements establish regression budgets.

## Deviations

- Automated hardening work began while 007 remains open because its only
  remaining evidence is an external interactive-terminal smoke test. The
  iteration is therefore `Blocked`, not `In Progress`, until that gate is met.

## Outcome

Pending.
