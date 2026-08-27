# Testing Strategy

## Goals

Tests protect game rules, application orchestration, generated geographic
invariants, terminal lifecycle, and responsive rendering. The suite must be
deterministic, easy to navigate, and fast enough for normal development.

WorldHunt uses only two test categories for the MVP:

- Unit tests isolate a module or unit of behavior.
- Smoke tests verify that a real executable boundary starts and performs its
  essential operation with production wiring.

Integration, acceptance, architecture, and mutation test suites are not part of
the MVP. Architectural rules remain mandatory, but are enforced through Rust
module visibility, Clippy, import review, and normal compilation rather than a
separate test category.

## Rust Test Conventions

The standard Rust test harness is the default framework. Unit tests live beside
the production module they protect, in a separate `tests.rs` file declared with
`#[cfg(test)] mod tests;`. This keeps implementation and test files readable
while retaining the idiomatic Rust module relationship and access to private
behavior without making implementation details public.

Automated smoke tests live under the root `tests/` directory and exercise the
crate through public or executable boundaries. The initial structure is:

```text
src/
|-- domain/                 # production modules and sibling tests.rs files
|-- application/            # production modules and sibling tests.rs files
`-- infrastructure/         # production modules and sibling tests.rs files
tests/
|-- smoke.rs                # Cargo smoke-test target
`-- smoke/                  # smoke scenarios and shared helpers
tools/world-data/
`-- src/                    # colocated generator unit tests
```

Test modules mirror the production modules they protect. For example, a module
at `src/infrastructure/tui/terminal.rs` keeps its unit tests at
`src/infrastructure/tui/terminal/tests.rs`, while `src/infrastructure/tui/mod.rs`
keeps them at `src/infrastructure/tui/tests.rs`. Shared test helpers
should be introduced only when they remove meaningful duplication and must
remain close to the tests that use them. Test-only data files may live in a
clearly named `testdata/` directory beside the relevant package.

Test names describe the condition and observable result. Tests should have a
clear arrange, act, and assert flow, equivalent to `given / when / then`,
without requiring comments or helper abstractions when the code is already
clear.

## Unit Test Rules

- Test behavior and invariants, not trivial getters or implementation shape.
- Keep each test independent and deterministic.
- Use explicit fixed inputs by default; use a seeded generator only when varied
  data adds useful coverage.
- Do not mock domain entities or Value Objects.
- Use small hand-written fakes for traits in `domain/ports/` when testing
  application use cases.
- Verify collaborator calls only when the interaction is part of the behavior,
  such as proving that invalid input does not query proximity.
- Use builders or factory helpers only when direct construction obscures the
  scenario; no Object Mother pattern or mocking framework is mandatory.
- Cover the happy path and relevant expected failures for every non-trivial
  unit.
- Do not target an arbitrary coverage percentage. Missing important behavior
  matters more than line coverage.

Additional Rust test libraries may be adopted only for a demonstrated need.
Ratatui's `TestBackend` is the appropriate backend for renderer tests. Snapshot
assertions may be used for stable representative layouts, but they remain unit
tests and must be accompanied by semantic assertions for critical text and
styles. Property-testing and mocking libraries are not baseline dependencies.

## Domain Unit Tests

- Domain values reject invalid state at construction.
- A game starts with an explicit eligible target and empty attempt history.
- A repeated `CountryId` does not add an attempt.
- A normal guess records the supplied domain proximity correctly.
- Guessing the target wins the game.
- No further guess mutates a completed game.

These tests load no embedded asset and construct no Ratatui or Crossterm types.

## Application Unit Tests

Application use cases use deterministic fake implementations of the traits in
`domain/ports/`:

- `StartGame` requests the playable catalog and selects an eligible target.
- Starting a game produces fresh domain state without retaining prior guesses.
- `SubmitGuess` passes the submitted domain name to `CountryCatalog` and uses
  the resolved country before requesting its proximity.
- Unknown input does not query proximity or mutate the game.
- An accepted guess records the proximity returned by the port.
- A deterministic target selector produces repeatable target sequences.
- Port failures do not expose infrastructure-specific details.
- Each use case is exercised through its single public `dispatch` method.

## Infrastructure And Generator Unit Tests

- The catalog has exactly 196 playable countries.
- ISO3 values, canonical names, and normalized aliases are unique.
- Every playable catalog entry maps to valid source geometry.
- Polygon holes and `MultiPolygon` components survive parsing.
- Every raster cell uses a valid or reserved identifier.
- Every playable country has a visual anchor.
- Distance and adjacency matrices have the required dimensions and symmetry.
- Every distance diagonal entry is zero.
- Regeneration produces byte-identical output.
- Invalid input data fails with an actionable diagnostic.
- Known canonical names and aliases resolve to the expected country.
- Catalog matching applies the configured case, whitespace, punctuation, and
  diacritic normalization.
- `CountryCatalog` and `CountryProximity` map decoded records into domain
  values and reject invalid encoded identifiers.
- Generated asset structures never escape through domain port APIs.

Generator unit tests use small synthetic geometries and selected real-world
reference fixtures. Initial reference cases include:

- France and Spain share a border.
- The United States and Russia are close through Alaska.
- Countries around longitude 180 are not treated as a world apart.
- Archipelagos use the closest participating island.
- A long boundary segment is densified within the documented tolerance.
- Small countries retain an anchor even when absent from a reduced raster.

Absolute expected distances must come from an independent geodesic reference
and include a documented tolerance. Tests must not merely duplicate the
implementation's own output.

Ratatui renderer unit tests cover representative presentation states:

- Wide and narrow playing layouts.
- Minimum and below-minimum terminal sizes.
- Empty and overflowing attempt histories.
- Unknown-input, shared-border, and victory states.
- Truecolor, ANSI 256-color, and monochrome themes.

Terminal lifecycle unit tests use an infrastructure-local backend seam or test
double to verify setup rollback and teardown order without changing the
developer's real terminal.

## Smoke Tests

Smoke tests are deliberately few and exercise production wiring rather than
repeating detailed unit scenarios. The MVP smoke coverage verifies:

- The application starts with the embedded asset and renders an initial frame.
- Normal quit and Ctrl+C restore the real terminal.
- A user can submit a valid guess and complete a basic local game.
- The world-data executable validates the committed asset in `--check` mode.
- Final release artifacts start and exit on Windows, Linux, and macOS.

Automate a smoke scenario when it can run reliably without controlling a real
interactive terminal. Terminal behavior and exact release artifacts require
documented manual smoke checks on each target platform. A smoke test must remain
short and must not become a second exhaustive functional suite.

## Architecture Verification

Architecture verification is a quality check, not a separate test suite:

- `domain` imports neither `application` nor `infrastructure`.
- `application` imports `domain` but not `infrastructure`.
- Ratatui, Crossterm, serializers, generated asset models, and concrete random
  libraries appear only in infrastructure or tools.
- Traits used to cross into infrastructure live in `domain/ports/`.
- `main.rs` is the composition root for concrete implementations.

Every iteration reviews these rules when adding or moving modules.

## Quality Commands

The repository Makefile is the canonical development interface. Cargo remains
the underlying test runner and can be used directly when Make is unavailable.

The expected baseline is:

```text
make check
```

`make test` runs all unit tests and the automated smoke target. `cargo test
--test smoke` may be used during development to run only smoke scenarios. The
smoke target is added when its first automated scenario exists.
`make check` also runs `cargo run -p world-data -- validate`, which validates
the committed catalog and source snapshot. Asset generation and its `--check`
mode are added in iteration 003.

CI runs the Ubuntu quality job on every pull request to `develop` or `main`.
The Windows and macOS compile matrix runs for pull requests to `main`, on a
weekly scheduled run of `develop`, and through manual dispatch. This keeps the
development feedback loop short while requiring full portability validation for
release promotion. Changes to the toolchain, terminal dependencies, platform
conditionals, packaging, or workflows should also trigger a manual full matrix
before merging into `develop`.

Data generation may run on one pinned environment if cross-platform
floating-point differences prevent byte-identical output; any such restriction
must be documented rather than hidden.

## Performance Checks

Iteration 008 records release-build measurements for startup time, decoded data
size, peak memory during normal use, map render time at representative terminal
sizes, generated asset size, and final binary size. These are engineering
checks, not a third test category. Initial limits are set only after the first
complete implementation is measured.

## Completion Rule

An iteration is `Completed` only after its acceptance criteria, relevant unit
tests, smoke checks, and verification commands pass. Commands and concise
outcomes are recorded in the iteration document. Intent to test later is not
verification.
