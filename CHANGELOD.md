# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- New build report to show output file name, elapsed time, and total output size.
### Changed
- Error handling to better highlight what caused an error to occur. An error will now display a list of causes, in order.
- `boon build` learned how to build all supported targets at the same time using the `--target all` option.
### Fixed
- `boon love download` no longer makes an extra unnecessary HTTP request when downloading LÖVE which should improve performance.
- `boon --version` now displays the correct release version.
- Unnecessary references (pointers) to small integers values have been removed, slightly improving performance.
- Library dependencies have been updated, improving performance and fixing many issues.

## [0.1.1] - 2020-02-11
### Fixed
- Incorrect macOS download locations for LÖVE 11.3.

## [0.1.0] - 2019-04-17
### Added
- The initial release for boon.
- Native LÖVE builds.
- Windows (32/64-bit) builds.
- macOS builds.
- LÖVE version manager.