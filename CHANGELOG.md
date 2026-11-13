# Changelog

All notable changes to WorldHunt are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- git-cliff: end of header -->
## [Unreleased]

### Added

- *(tui)* Show version in compact status border

### Fixed

- *(ci)* Use supported version placeholder

### Changed

- [Dependency Update]: bump toml in the cargo-dependencies group
- [Dependency Update]: bump toml in the cargo-dependencies group
- [CI Update]: bump jfrz38/check-version-change
- [Dependency Update]: bump rand in the cargo-dependencies group
- [CI Update]: bump jfrz38/check-version-change
- [Dependency Update]: bump the cargo-dependencies group with 2 updates
- [Dependency Update]: bump the cargo-dependencies group with 2 updates
- Bump version to {new-version}
- [Dependency Update]: bump the cargo-dependencies group with 2 updates
- [CI Update]: bump jfrz38/check-version-change
- *(ci)* Run dependabot monthly
- [Dependency Update]: bump the cargo-dependencies group with 2 updates
## [0.1.1] - 2026-09-03

### Added

- Increase map layout
- Improve countries suggested
- Add CODEOWNERS file with owner assignment

### Fixed

- Use correct version

### Changed

- *(world-data)* Remove unused asset preview code
- *(countries)* Remove Spanish Sahara name
- Bump version to {new-version}
- Add dependabot
- Create lightweight tag
- [CI Update]: bump the github-actions group with 3 updates
- Fix invalid dependency bump
- [Dependency Update]: bump the cargo-dependencies group with 3 updates
## [0.1.0] - 2026-09-03

### Added

- Add MIT License to the project
- Add Rust terminal application foundation
- Add world-data catalog validator
- Add world map asset pipeline
- Render the world map in the TUI
- Add world detail asset generation
- Add EH and IC to world map
- Add country overlay map assets
- Add zoomable TUI map navigation
- *(world-data)* Add territorial proximity generator
- *(world-data)* Add v2 proximity matrix asset
- *(game-domain)* Add core game use cases and ports
- *(tui)* Add theme-aware map rendering and docs updates
- *(tui)* Add playable input flow and smoke test
- *(tui)* Add country suggestions and recenter guesses
- *(tui)* Add hints, surrender, and inline suggestions
- Implement playable TUI with country catalog integration
- *(tui)* Add rendered_guesses_for_map method to include target guess on surrender
- Add asset link to README
- Add CLI argument handling for help and version
- *(countries)* Add Libya as playable name

### Fixed

- Handle antimeridian polygons in raster generation
- *(world-data)* Anchor raster countries from source polygons
- Update dependencies and fix TUI app integration issues
- *(world-data)* Validate polygon rings and positions
- Add infinite horizontal scroll
- Allow hyphen for countries

### Changed

- Initial commit
- Revise README with improved title and description
- Rename project from Country Guess TUI to WorldHunt
- Add project documentation
- Add project quality automation
- Document project foundation workflow
- Move unit tests to separate files
- Clarify test and iteration workflow
- Mark iteration 001 complete and 002 active
- Optimize pull request checks
- Split world-data validation modules
- Format code
- Simplify map tile projection inputs
- Update validation counts
- Update 196-country catalog references
- Update Web Mercator map docs and notices
- Update distance calculation notes
- Update world-data docs for v2 asset
- *(iterations)* Update game domain status
- *(tui)* Move map renderer tests to dedicated file
- *(tui)* Move tests to dedicated file
- Update TUI and iteration status
- Update playable TUI documentation
- Update playable TUI documentation
- Move class tests into dedicated modules
- Update hardening docs and runtime contract
- Expand hardening coverage
- Prepare documentation and metadata for MVP release
- Add GitHub workflows and validation scripts for release
- Configure workspace publishing and exclude tools
- Improve release asnd use check-version-change
- Improve readme
- Remove technical documentation
- Improve documentation
- Improve release pipeline adding quality checks
- Improve script verification
- Update Cargo.toml dependencies and include files
- Update OpenStreetMap metadata for zoom-1 basemap
