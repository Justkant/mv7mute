# Changelog

All notable changes to this package will be documented in this file.

The format is based on Keep a Changelog, and versions follow Semantic Versioning.

## [Unreleased]

### Changed

- Split the project into a Cargo workspace with dedicated `mv7mute-core`, `mv7mute`, and `mv7mute-tray` crates.
- Kept the CLI crate thin by moving MV7 command logic into the core crate.

## [0.1.0] - 2026-04-30

### Added

- Initial MV7 mute CLI release.
- Atomic unlock, command, and relock flow for mute operations.
- Commands for toggle, on, off, status, lock, and unlock.
- Cross-platform HID transport built on hidapi.
