# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.10.0](https://github.com/loambuild/loam-sdk/compare/loam-cli-v0.9.4...loam-cli-v0.10.0) - 2024-06-25

### Other
- cargo fmt and clippy -Dpedantic fixes, add to CI ([#89](https://github.com/loambuild/loam-sdk/pull/89))

## [0.9.4](https://github.com/loambuild/loam-sdk/compare/loam-cli-v0.9.3...loam-cli-v0.9.4) - 2024-06-21

### Fixed
- *(CLI)* cargo-binstall in Cargo.toml ([#84](https://github.com/loambuild/loam-sdk/pull/84))

## [0.9.3](https://github.com/loambuild/loam-sdk/compare/loam-cli-v0.9.2...loam-cli-v0.9.3) - 2024-06-21

### Fixed
- name of release binary file ([#82](https://github.com/loambuild/loam-sdk/pull/82))

## [0.9.2](https://github.com/loambuild/loam-sdk/compare/loam-cli-v0.9.1...loam-cli-v0.9.2) - 2024-06-21

### Other
- Fix/cd workflow again ([#80](https://github.com/loambuild/loam-sdk/pull/80))

## [0.9.1](https://github.com/loambuild/loam-sdk/compare/loam-cli-v0.9.0...loam-cli-v0.9.1) - 2024-06-21

### Fixed
- add a little to long help to trigger release

## [0.9.0](https://github.com/loambuild/loam-sdk/compare/loam-cli-v0.8.0...loam-cli-v0.9.0) - 2024-06-21

### Added
- add new build_clients logic ([#42](https://github.com/loambuild/loam-sdk/pull/42))

### Other
- update readme.md ([#66](https://github.com/loambuild/loam-sdk/pull/66))

## [0.8.0](https://github.com/loambuild/loam-sdk/compare/loam-cli-v0.7.0...loam-cli-v0.8.0) - 2024-05-21

### Added
- Update dependencies versions to latest ([#51](https://github.com/loambuild/loam-sdk/pull/51))
- [**breaking**] rename subcontract crates ([#49](https://github.com/loambuild/loam-sdk/pull/49))
- [**breaking**] rename riff to subcontract ([#48](https://github.com/loambuild/loam-sdk/pull/48))
- make `target/loam` default directory and use symbolic linking ([#43](https://github.com/loambuild/loam-sdk/pull/43))

### Other
- Add and edits readmes ([#47](https://github.com/loambuild/loam-sdk/pull/47))

## [0.7.0](https://github.com/loambuild/loam-sdk/compare/loam-cli-v0.6.6...loam-cli-v0.7.0) - 2024-03-22

### Added
- Add new update-env command ([#39](https://github.com/loambuild/loam-sdk/pull/39))
- [**breaking**] rename Owner to Admin ([#37](https://github.com/loambuild/loam-sdk/pull/37))

### Fixed
- create .env if it doesn't exist ([#41](https://github.com/loambuild/loam-sdk/pull/41))

## [0.6.6](https://github.com/loambuild/loam-sdk/compare/loam-cli-v0.6.5...loam-cli-v0.6.6) - 2024-01-09

### Other
- update Cargo.lock dependencies

## [0.6.5](https://github.com/loambuild/loam-sdk/compare/loam-cli-v0.6.4...loam-cli-v0.6.5) - 2023-09-24

### Fixed
- ensure that deps of --package are built before specified package ([#31](https://github.com/loambuild/loam-sdk/pull/31))

## [0.6.4](https://github.com/loambuild/loam-sdk/compare/loam-cli-v0.6.3...loam-cli-v0.6.4) - 2023-09-01

### Fixed
- use versions directly so they get bumped by release ([#24](https://github.com/loambuild/loam-sdk/pull/24))

## [0.6.3](https://github.com/loambuild/loam-sdk/compare/loam-cli-v0.6.2...loam-cli-v0.6.3) - 2023-09-01

### Other
- release ([#15](https://github.com/loambuild/loam-sdk/pull/15))

## [0.6.2](https://github.com/loambuild/loam-sdk/releases/tag/loam-cli-v0.6.2) - 2023-08-31

### Added
- add CLI with build command and update soroban-sdk ([#11](https://github.com/loambuild/loam-sdk/pull/11))

### Fixed
- add versions to workspace deps in root Cargo.toml ([#19](https://github.com/loambuild/loam-sdk/pull/19))
- use actual versions ([#18](https://github.com/loambuild/loam-sdk/pull/18))
- use correct versions ([#17](https://github.com/loambuild/loam-sdk/pull/17))
- added description and license for crates

### Other
- Prepare for publishing ([#12](https://github.com/loambuild/loam-sdk/pull/12))
