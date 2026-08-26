# Iteration 001: Project Foundation

Status: Planned  
Started:  
Completed:

## Goal

Establish a maintainable Rust application that can enter, render, resize, and
leave a terminal UI without corrupting the user's terminal state.

## Dependencies

None. This is the first implementation iteration.

## Scope

- Root Rust package and workspace configuration.
- Initial `domain`, `application`, and `infrastructure` module structure.
- Inward dependency rules and `main.rs` composition root.
- Ratatui and Crossterm integration.
- Safe terminal setup and teardown.
- Minimal event loop and responsive placeholder screen.
- Formatting, linting, tests, and baseline CI.
- Rust unit-test conventions and the reserved root smoke-test structure.

## Out of Scope

- Countries, geographic data, game rules, and final visual design.
- Async runtime, configuration files, persistence, and release packaging.

## Tasks

- [ ] Create the root `Cargo.toml` and select the Rust edition and toolchain
  policy.
- [ ] Add a repository `.gitignore` and any minimal formatter or lint settings.
- [ ] Add `src/lib.rs`, `src/main.rs`, and the initial `domain`, `application`,
  and `infrastructure` modules described in `docs/architecture.md`.
- [ ] Keep `main.rs` as the composition root and document the enforced
  `infrastructure -> application -> domain` dependency direction.
- [ ] Add Ratatui and Crossterm with only required features.
- [ ] Implement guarded terminal initialization and idempotent restoration.
- [ ] Restore completed setup steps if a later initialization step fails.
- [ ] Restore the terminal after normal exit, application error, and panic.
- [ ] Implement a blocking event loop with `Esc`, `Ctrl+C`, and resize handling.
- [ ] Render a minimal placeholder that reports current terminal dimensions.
- [ ] Add unit tests for lifecycle behavior through an infrastructure-local
  terminal backend seam or test double.
- [ ] Document colocated `#[cfg(test)]` unit tests and reserve `tests/smoke.rs`
  for production-wiring smoke scenarios once the first one is implementable.
- [ ] Add GitHub Actions for format, Clippy, tests, and platform compilation.
- [ ] Update root README build and run instructions for the skeleton.

## Acceptance Criteria

- [ ] `cargo run` opens the alternate screen and renders without busy-looping.
- [ ] `Esc` and `Ctrl+C` exit cleanly.
- [ ] Resize events redraw without terminating the process.
- [ ] Raw mode, screen, cursor, and related terminal state are restored on every
  tested exit path.
- [ ] Domain and application modules can be imported through the library crate
  without exposing Ratatui, Crossterm, or other infrastructure types.
- [ ] Import review confirms that domain and application do not depend on
  infrastructure modules.
- [ ] Format, Clippy with warnings denied, and all tests pass.
- [ ] The test layout uses only the unit and smoke categories defined in
  `docs/testing.md`.
- [ ] Windows, Linux, and macOS compile checks pass in CI.
- [ ] `docs/README.md` marks this iteration completed and iteration 002 next.

## Verification

Not run; iteration has not started.

## Decisions

None yet.

## Deviations

None yet.

## Outcome

Pending.
