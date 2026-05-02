# Changelog

All notable changes to this package will be documented in this file.

The format is based on Keep a Changelog, and versions follow Semantic Versioning.

## [Unreleased]

## [0.1.2](https://github.com/Justkant/mv7mute/compare/v0.1.1...v0.1.2) - 2026-05-02

### Added

- *(release)* switch to per-app releases and changelogs
- add mv7mute-tray and restructure as a Cargo workspace
- add release automation and version command
- initial implementation of mv7mute

### Fixed

- *(release)* publish mv7mute-core for release-plz packaging
- *(release)* enable tagging after squash-merged release PRs

## [0.1.1](https://github.com/Justkant/mv7mute/compare/v0.1.0...v0.1.1) - 2026-05-02

### Changed

- Align the tray app version with the CLI for unified app release tags.

## [0.1.0](https://github.com/Justkant/mv7mute/releases/tag/mv7mute-tray-v0.1.0) - 2026-05-01

### Added

- _(release)_ switch to per-app releases and changelogs
- add mv7mute-tray and restructure as a Cargo workspace
- add release automation and version command
- initial implementation of mv7mute

### Fixed

- _(ci)_ use single v{{ version }} tag for unified releases

### Other

- add libxdo-dev to Linux dependencies in all workflows

### Added

- Added a first tray application scaffold built on `winit` and `tray-icon`.
- Added a typed core device-state API for tray consumers alongside the existing command runner.
