# Changelog

All notable changes to WorldHunt are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- git-cliff: end of header -->

## [0.1.2] - 2026-10-25

### Added

- Display the application version in the TUI status border.

### Fixed

- Use a supported version placeholder when preparing release pull requests.

## [0.1.1] - 2026-09-03

### Added

- Search for countries by any word in their name, with more relevant suggestions first.
- Hide countries that have already been guessed from autocomplete suggestions.

### Changed

- Give the map more space in wide terminal layouts.
- Remove the obsolete "Spanish Sahara" country alias.

### Fixed

- Generate random targets and validate map checksums correctly with the updated dependencies.

## [0.1.0] - 2026-09-03

Initial MVP release of WorldHunt, a terminal-based geography guessing game.

### Added

- Play a complete country-guessing game from the terminal, including hints, surrender, and starting a new game.
- Navigate a zoomable, horizontally wrapping world map with country overlays and guess markers.
- Search and autocomplete a catalog of 196 playable countries, including aliases and normalized input.
- Recenter the map after each guess and show proximity feedback based on geographic distance.
- Support compact and wide terminal layouts with theme-aware rendering.
- Provide `--help` and `--version` command-line options.
- Include the world-data generation and validation tools used to build the embedded map assets.

### Fixed

- Render polygons that cross the antimeridian correctly.
- Anchor country rasters to their source polygons and validate polygon rings and positions.
- Allow hyphens in country names.
- Keep the map scrollable indefinitely in either horizontal direction.

[0.1.2]: https://github.com/jfrz38/worldhunt/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/jfrz38/worldhunt/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/jfrz38/worldhunt/releases/tag/v0.1.0
