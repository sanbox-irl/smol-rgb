# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

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
