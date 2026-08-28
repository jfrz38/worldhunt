# Product Specification

## Vision

WorldHunt is a hot-and-cold geography guessing game for the terminal. The
player enters country names, learns how far each guess is from a hidden target,
and sees guessed territories form a heat map as the guesses get closer.

The MVP is deliberately local and focused. It should start quickly, require no
network connection, and remain usable across Windows, Linux, and macOS.

## Core Game Loop

1. Select a random target from the playable country catalog.
2. Accept a country name or known alias in English.
3. Reject unknown and repeated guesses without consuming an attempt.
4. Show the guessed country on the map using a color derived from its distance.
5. Add the guess and its distance to the attempt history.
6. Repeat until the target is guessed.
7. Reveal the target as the winning country and offer a new local game.

There is no attempt limit. A country that shares a border with the target is
reported as `Borders target` instead of `0 km`.

## MVP Scope

- A curated catalog of 196 playable countries: 193 United Nations member
  states, Palestine, Vatican City, and Western Sahara.
- English canonical names and explicit English aliases.
- Random, unlimited local games.
- A flat, responsive world map rendered in the terminal.
- Persistent coloring of all guesses during the current game.
- Minimum geodesic distance between country territories.
- Exact distance in the attempt history, rounded to kilometers.
- Keyboard input, error messages, win state, and new-game action.
- Truecolor output when supported and an ANSI 256-color fallback.
- A monochrome fallback when color is disabled.
- Windows, Linux, and macOS support.

## Territorial Policy

All polygon components belonging to a playable country's own dataset record
participate in distance calculations. This includes islands and archipelagos.
A dependency represented by a separate dataset record is not automatically
merged into its sovereign state.

Non-playable countries and territories may remain visible as neutral land, but
they cannot be selected as guesses or targets. Disputed or exceptional records
must be handled explicitly in the curated catalog rather than inferred at
runtime.

## Input Behavior

Country matching is case-insensitive and normalizes repeated whitespace,
common punctuation variants, and diacritics where practical. Aliases are
explicit data, not fuzzy matches. Examples include `USA`, `US`, and
`United States of America` for `United States`.

Fuzzy suggestions and localization are not part of the MVP. Invalid input
must produce a recoverable message and preserve the current game.

## Out of Scope for the MVP

- Daily challenges, seeded calendar puzzles, streaks, and statistics.
- Persistent games or settings.
- Online accounts, leaderboards, telemetry, and network services.
- Sharing result grids.
- Mouse selection.
- Zoom and pan.
- A rotatable globe or alternative map projections.
- Languages other than English.
- User-selectable country catalogs or territorial policies.
- Audio and animation beyond simple terminal updates.

These features may be proposed later as new iterations and decision records.

## MVP Success Criteria

The MVP is complete when a user can download and run WorldHunt without an
external data file or network connection, play a full game against any of the
196 targets, understand every clue through map color and textual distance, and
exit without leaving the terminal in an altered state.
