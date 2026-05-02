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

## [0.1.1](https://github.com/Justkant/mv7mute/compare/v0.1.0...v0.1.1) - 2026-05-01

### Added

- _(release)_ switch to per-app releases and changelogs
- add mv7mute-tray and restructure as a Cargo workspace

### Fixed

- _(ci)_ use single v{{ version }} tag for unified releases

### Changed

- Split the project into a Cargo workspace with dedicated `mv7mute-core`, `mv7mute`, and `mv7mute-tray` crates.
- Kept the CLI crate thin by moving MV7 command logic into the core crate.

## [0.1.0] - 2026-04-30

### Added

- Initial MV7 mute CLI release.
- Atomic unlock, command, and relock flow for mute operations.
- Commands for toggle, on, off, status, lock, and unlock.
- Cross-platform HID transport built on hidapi.
