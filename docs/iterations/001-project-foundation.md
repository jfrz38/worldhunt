# Iteration 001: Project Foundation

Status: Completed
Started: 2026-08-26
Completed: 2026-08-27

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

- [x] Create the root `Cargo.toml` and select the Rust edition and toolchain
  policy.
- [x] Add a repository `.gitignore` and any minimal formatter or lint settings.
- [x] Add `src/lib.rs`, `src/main.rs`, and the initial `domain`, `application`,
  and `infrastructure` modules described in `docs/architecture.md`.
- [x] Keep `main.rs` as the composition root and document the enforced
  `infrastructure -> application -> domain` dependency direction.
- [x] Add Ratatui and Crossterm with only required features.
- [x] Implement guarded terminal initialization and idempotent restoration.
- [x] Restore completed setup steps if a later initialization step fails.
- [x] Restore the terminal after normal exit, application error, and panic.
- [x] Implement a blocking event loop with `Esc`, `Ctrl+C`, and resize handling.
- [x] Render a minimal placeholder that reports current terminal dimensions.
- [x] Add unit tests for lifecycle behavior through an infrastructure-local
  terminal backend seam or test double.
- [x] Document colocated unit tests in separate `tests.rs` files and reserve
  `tests/smoke.rs`
  for production-wiring smoke scenarios once the first one is implementable.
- [x] Add GitHub Actions for format, Clippy, tests, and platform compilation.
- [x] Update root README build and run instructions for the skeleton.

## Acceptance Criteria

- [x] `cargo run` opens the alternate screen and renders without busy-looping.
- [x] `Esc` and `Ctrl+C` exit cleanly.
- [x] Resize events redraw without terminating the process.
- [x] Raw mode, screen, cursor, and related terminal state are restored on every
  tested exit path.
- [x] Domain and application modules can be imported through the library crate
  without exposing Ratatui, Crossterm, or other infrastructure types.
- [x] Import review confirms that domain and application do not depend on
  infrastructure modules.
- [x] Format, Clippy with warnings denied, and all tests pass.
- [x] The test layout uses only the unit and smoke categories defined in
  `docs/testing.md`.
- [x] Windows, Linux, and macOS compile checks pass in CI.
- [x] After merging into `develop`, `docs/README.md` marks this iteration
  completed and iteration 002 next.

## Verification

- `cargo fmt --check`: passed locally.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed locally.
- `cargo test --workspace`: passed locally; 10 unit tests.
- `cargo check --all-targets --all-features`: passed locally on Windows.
- `make check` and `make ci`: passed locally; the latter also builds all
  workspace targets.
- Import review: passed; domain and application contain no infrastructure or
  terminal-library imports.
- Manual terminal interaction: passed locally; the placeholder rendered and
  `Esc` restored the terminal.
- GitHub Actions platform matrix: passed before integration.
- Post-merge `make check`: passed locally.

## Decisions

- Use Rust edition 2024 with Rust 1.88.0 as the pinned repository toolchain and
  minimum supported version, matching Ratatui's minimum supported Rust version.
- Track each completed terminal setup step in an RAII session, restore in
  reverse order, attempt every cleanup step, and preserve the first cleanup
  error.
- Keep unit tests in separate sibling `tests.rs` files while declaring them from
  the production module with `#[cfg(test)] mod tests;`.

## Deviations

None yet.

## Outcome

The project foundation is complete. Iteration 002 can add world-data tooling
without adding generator models to the runtime crate.
