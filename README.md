
# WorldHunt

[![CI](https://github.com/jfrz38/worldhunt/actions/workflows/ci.yml/badge.svg)](https://github.com/jfrz38/worldhunt/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/worldhunt.svg)](https://crates.io/crates/worldhunt)
[![Downloads](https://img.shields.io/crates/d/worldhunt.svg)](https://crates.io/crates/worldhunt)
[![License](https://img.shields.io/crates/l/worldhunt.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.88-blue)](https://www.rust-lang.org/)

WorldHunt is an offline hot-and-cold geography guessing game for the terminal.
Inspired by [Globle](https://globle-game.com/), it brings the same hunt for a hidden country to an interactive world map in your terminal.

Name a country, see how far away it is, and use the map and border clues to home in on the answer.

https://github.com/user-attachments/assets/c8230141-3345-4453-91b8-eff04bb03bd5

## Install And Play

WorldHunt is distributed as a Rust crate; prebuilt binaries are not available.

```sh
cargo install worldhunt --locked
worldhunt
```

Everything needed to play is included. No network connection or external data files are required.

## Play

There is one hidden country and 196 possible answers. Guess as often as you need: every accepted guess shows its distance from the target, or tells you when it shares a border with it. The map keeps track of every attempt, so you can see whether you are getting warmer.

**About the country list:** Country lists are never entirely neutral. WorldHunt uses the 193 UN member states as its starting point and deliberately includes Palestine, Vatican City, and Western Sahara, for a total of 196 playable countries.

Just type a country and press `Enter`. Suggestions appear after two characters if you want a hand with the name.

### Controls

| Input | Action |
| --- | --- |
| Type a country + `Enter` | Make a guess |
| `Tab` | Highlight the next matching country |
| `Enter` with a suggestion highlighted | Fill in that country name; press `Enter` again to make the guess |
| `/hint` + Enter | Reveal one target-name letter without an attempt |
| `/surrender` + Enter | Reveal the target and end the current game |
| `+` / `-`, arrows | Zoom and pan the map |
| N | Start a new game after victory or surrender |
| Esc / Ctrl+C | Quit and restore the terminal |

## Terminal Notes

The minimum supported terminal size is `48x20`. At width `90` and above, the map and attempt history appear side by side; narrower terminals stack them.

Set `WORLDHUNT_COLOR=truecolor`, `WORLDHUNT_COLOR=ansi256`, or `WORLDHUNT_COLOR=mono` to choose a palette. `NO_COLOR` selects monochrome when no explicit mode is set. Distance and border clues remain textual in every mode.

## Requirements

- Rust 1.88 or newer. [Rustup](https://rustup.rs/) installs Rust and the formatter and linter components selected by `rust-toolchain.toml`.
- A terminal with ANSI and Unicode support. Mouse-wheel zoom is available when the terminal reports mouse input.

## Development

```sh
git clone https://github.com/jfrz38/worldhunt.git
cd worldhunt
make run
make check
```

Run `make help` for the available commands. `make release-check` validates the exact crate contents and performs a crates.io dry run; it is also the release workflow's package gate.

Architecture, data provenance, testing strategy, decisions, and iteration records are maintained in [`docs/`](docs/README.md). Contributions and bug reports are welcome through [GitHub issues](https://github.com/jfrz38/worldhunt/issues). Please include terminal, operating-system, and color-mode details for visual or input problems.

## Data And Licenses

WorldHunt code is licensed under [MIT](LICENSE). Geographic source data and embedded derivatives have separate attribution and licensing requirements; see [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) and the metadata under `data/source/`. The game displays OpenStreetMap attribution while the map is visible.

## Limitations

- There are no prebuilt binaries, installers, online accounts, saved games, or daily challenges.
- Country recognition follows the curated 196-country catalog and its documented aliases; it is not a general place-name search.
