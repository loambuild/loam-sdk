# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.8.1](https://github.com/loambuild/loam-sdk/compare/loam-sdk-macro-v0.8.0...loam-sdk-macro-v0.8.1) - 2024-06-25

### Other
- cargo fmt and clippy -Dpedantic fixes, add to CI ([#89](https://github.com/loambuild/loam-sdk/pull/89))

## [0.8.0](https://github.com/loambuild/loam-sdk/compare/loam-sdk-macro-v0.7.0...loam-sdk-macro-v0.8.0) - 2024-05-21

### Added
- [**breaking**] rename riff to subcontract ([#48](https://github.com/loambuild/loam-sdk/pull/48))

### Other
- Add and edits readmes ([#47](https://github.com/loambuild/loam-sdk/pull/47))

## [0.7.0](https://github.com/loambuild/loam-sdk/compare/loam-sdk-macro-v0.6.4...loam-sdk-macro-v0.7.0) - 2024-03-22

### Added
- [**breaking**] rename Owner to Admin ([#37](https://github.com/loambuild/loam-sdk/pull/37))
- use crate_path to use `loam_sdk::soroban_sdk` ([#38](https://github.com/loambuild/loam-sdk/pull/38))

## [0.6.4](https://github.com/loambuild/loam-sdk/compare/loam-sdk-macro-v0.6.3...loam-sdk-macro-v0.6.4) - 2023-09-01

### Fixed
- use versions directly so they get bumped by release ([#24](https://github.com/loambuild/loam-sdk/pull/24))

## [0.6.3](https://github.com/loambuild/loam-sdk/compare/loam-sdk-macro-v0.6.2...loam-sdk-macro-v0.6.3) - 2023-09-01

### Other
- release ([#15](https://github.com/loambuild/loam-sdk/pull/15))

## [0.6.2](https://github.com/loambuild/loam-sdk/releases/tag/loam-sdk-macro-v0.6.2) - 2023-08-31

### Added
- add contract_import macro and refactor to make clippy happy ([#13](https://github.com/loambuild/loam-sdk/pull/13))

### Fixed
- add versions to workspace deps in root Cargo.toml ([#19](https://github.com/loambuild/loam-sdk/pull/19))
- use actual versions ([#18](https://github.com/loambuild/loam-sdk/pull/18))
- use correct versions ([#17](https://github.com/loambuild/loam-sdk/pull/17))
- added description and license for crates

### Other
- Prepare for publishing ([#12](https://github.com/loambuild/loam-sdk/pull/12))
