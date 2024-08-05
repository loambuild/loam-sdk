# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.14.1](https://github.com/loambuild/loam/compare/loam-cli-v0.14.0...loam-cli-v0.14.1) - 2024-08-05

### Added
- *(CLI)* enable string for account def in environments.toml ([#137](https://github.com/loambuild/loam/pull/137))

### Other
- release ([#141](https://github.com/loambuild/loam/pull/141))
- add urls to cargo.toml files ([#140](https://github.com/loambuild/loam/pull/140))

## [0.14.0](https://github.com/loambuild/loam/compare/loam-cli-v0.13.2...loam-cli-v0.14.0) - 2024-07-31

### Other
- Fix/support init scripts with quotes ([#134](https://github.com/loambuild/loam/pull/134))

## [0.13.2](https://github.com/loambuild/loam/compare/loam-cli-v0.13.1...loam-cli-v0.13.2) - 2024-07-23

### Fixed
- *(CLI)* explain how to use loam init ([#126](https://github.com/loambuild/loam/pull/126))

### Other
- wip ([#131](https://github.com/loambuild/loam/pull/131))
- *(CLI)* simplify tests ([#129](https://github.com/loambuild/loam/pull/129))

## [0.13.1](https://github.com/loambuild/loam/compare/loam-cli-v0.13.0...loam-cli-v0.13.1) - 2024-07-20

### Other
- update stellar deps ([#118](https://github.com/loambuild/loam/pull/118))

## [0.13.0](https://github.com/loambuild/loam/compare/loam-cli-v0.12.0...loam-cli-v0.13.0) - 2024-07-19

### Fixed
- *(CLI)* don't overwrite accounts ([#121](https://github.com/loambuild/loam/pull/121))
- *(CLI)* use Loam SDK deps with `loam init` ([#120](https://github.com/loambuild/loam/pull/120))

## [0.12.0](https://github.com/loambuild/loam/compare/loam-cli-v0.11.0...loam-cli-v0.12.0) - 2024-07-17

### Added
- feat!(CLI): auto build clients for all contracts  ([#117](https://github.com/loambuild/loam/pull/117))

## [0.11.0](https://github.com/loambuild/loam/compare/loam-cli-v0.10.3...loam-cli-v0.11.0) - 2024-07-16

### Added
- *(CLI)* support `init` contract scripts in `environments.toml`

## [0.10.3](https://github.com/loambuild/loam-sdk/compare/loam-cli-v0.10.2...loam-cli-v0.10.3) - 2024-07-15

### Fixed
- when compiling with no profile add extra flags to ensure small wasm ([#115](https://github.com/loambuild/loam-sdk/pull/115))

## [0.10.2](https://github.com/loambuild/loam-sdk/compare/loam-cli-v0.10.1...loam-cli-v0.10.2) - 2024-07-15

### Fixed
- *(CLI)* core example: rm `loam-soroban-sdk` dep ([#112](https://github.com/loambuild/loam-sdk/pull/112))
- `loam dev` clone Arc of build command instead of cloning each time ([#108](https://github.com/loambuild/loam-sdk/pull/108))

## [0.10.1](https://github.com/loambuild/loam-sdk/compare/loam-cli-v0.10.0...loam-cli-v0.10.1) - 2024-07-13

### Fixed
- *(CLI)* never default to debug ([#100](https://github.com/loambuild/loam-sdk/pull/100))
- properly rename Cargo.toml.remove ([#106](https://github.com/loambuild/loam-sdk/pull/106))

## [0.10.0](https://github.com/loambuild/loam-sdk/compare/loam-cli-v0.9.4...loam-cli-v0.10.0) - 2024-07-13

### Added
- allow creating new projects with loam init ([#95](https://github.com/loambuild/loam-sdk/pull/95))
- add `derive_contract` to make it macro more explicit ([#93](https://github.com/loambuild/loam-sdk/pull/93))

### Other
- add dev command ([#87](https://github.com/loambuild/loam-sdk/pull/87))
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
