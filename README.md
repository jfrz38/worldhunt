# WorldHunt

`worldhunt` is a hot-and-cold geography guessing game for your terminal.

Guess countries, use their distance to narrow down the hidden target, and watch your attempts get highlighted on an interactive world map as you get closer.

## Documentation

The product specification, architecture, decisions, and implementation roadmap
are maintained in [`docs/`](docs/README.md).

## Requirements

- [Rustup](https://rustup.rs/). The repository toolchain file selects Rust
  1.88.0 and installs the required formatter and linter components.

## Run

```sh
make run
```

The application enters the terminal's alternate screen. Press `Esc` or
`Ctrl+C` to exit. Resizing the terminal redraws the placeholder with its current
dimensions.

## Quality Checks

```sh
make check
```

Run `make help` to list available development commands. Cargo remains available
as the underlying tool when Make is unavailable.

The crate follows the dependency direction `infrastructure -> application ->
domain`; `src/main.rs` is only the composition root. Unit tests are colocated
with their Rust modules, in separate `tests.rs` files declared with
`#[cfg(test)] mod tests;`. The root `tests/smoke.rs` target is reserved until a
production-wiring smoke scenario can be implemented.
