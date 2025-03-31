# Changelog

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Possible sections are:

- `Added` for new features.
- `Changed` for changes in existing functionality.
- `Deprecated` for soon-to-be removed features.
- `Removed` for now removed features.
- `Fixed` for any bug fixes.
- `Security` in case of vulnerabilities.

<!-- next-header -->
## [Unreleased] - ReleaseDate

### Changed

- bump deps

## [0.3.4] - 2024-11-15

### Changed

- bump `h3o` to 0.7

## [0.3.3] - 2024-10-10

### Fixed

- fix performance issue at higher zoom level for `tiles_for_cell`.

## [0.3.2] - 2024-10-03

### Changed

- bump deps

## [0.3.1] - 2024-07-04

### Fixed

- fix rendering glitches at high zoom levels (e.g. 19+)

## [0.3.0] - 2024-05-21

### Changed

- `TileCoord` is now private.
- `parent`, ``neighbors`, `extent` and `is_eastern` for `TileID` are now private.
- `TileID::new` is now faillible.
- zoom level is now exposed as `u8` rather than `u32`.

## [0.2.2] - 2024-05-20

### Added

- add `xy` accessor to expose the TileID coordinate.

### Changed

- bump geozero

## [0.2.1] - 2024-05-17

### Added

- add tiles_for_cell` to compute the tiles that contain or intersect a cell

## [0.2.0] - 2024-05-15

### Changed

- rename `TileID::bbox` to `TileID::cells`
- add a "scratch" mode boolean parameter to `render`

## [0.1.0] - 2024-05-14

- initial release, still need some polish
