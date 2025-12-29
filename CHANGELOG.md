# Change Log

### Unreleased

- BREAKING: Moved all named colors under a feature called "colors", which is
  enabled by default. If you have `default-features=false` and want the color names,
  just add the feature `colors`.

## [0.4.0] - 2025-05-26

- BREAKING: Renamed `Linear::to_encoded_space` to just `Linear::encoded_space`.
- BREAKING: Removed all tuple conversions, since they're not commonly used.
- Simplified the `From` and `Into` conversations for arrays to use transmutation operations.

## [0.3.1] - 2024-08-30

- Fixed `YELLOW` and `YELLOW_CLEAR` to actually be Yellow instead of fuchsia colored.
- Added builders to `EncodedColor`.

## [0.3.0] - 2021-04-01

- BREAKING: changed the serialized name of `Encoded Rgb` to `EncodedColor`. This will break serialization in formats like `JSON` or `YAML`.
- Added more color names like `YELLOW`, `RED`, and `TEAL`

## [0.2.0] - 2021-04-01

- Renamed `EncodedRgb` and `LinearRgb` to `EncodedColor` and `LinearRgb`. This is simpler to understand for users and avoids ignoring the `a` component.
- Added the `rand` optional dependency

## [0.1.2] - 2021-04-01

Added default implementations and hash implementation on `EncodedRgb`

## [0.1.1] - 2021-03-15

No serious changes, but some performance improvements and additions for formatting
and working with packed u32s.

### Added

- Conversions to and from packed u32s for `rgba` and `bgra`. I supposed `bgra` simply
  because I needed them in a project as well.
- Hex conversions for `EncodedRgb`, which is just the hex for a packed `rgba` struct.
- Both of the above might not work well on a big-endian system. If someone knows more,
  I'd appreciate the advice.

### Changed

- Conversion from `encoded_to_linear` is now done with a lookup table, rather than the actual
  math. This should be a nice speedup.
- Made more functions `const` and `#[inline]` for those gains.

### Fixed

## [0.1.0] - 2021-02-28

Initial commit. See the README for project details.
