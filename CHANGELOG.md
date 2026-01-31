# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial open source documentation
- CONTRIBUTING.md with contribution guidelines
- SECURITY.md with security policy
- CODE_OF_CONDUCT.md for community standards
- Issue templates for bugs, features, and questions
- Pull request template

## [0.1.0] - TBD

### Added
- UBB-level operations for isolating and restoring entire UBB trays
- Config-based operations using TOML files for custom ASIC grouping
- Graph-based modeling of Galaxy system topology
- Connectivity validation using Union-Find algorithm
- Port disable calculation for isolation operations
- GUI interface (work in progress)
- Comprehensive documentation in README

### Features
- `--isolate-all-trays`: Isolate all UBB trays from each other
- `--isolate-tray <N>`: Isolate a specific UBB tray
- `--restore-all`: Restore all UBB trays to default settings
- `--restore-tray <N>`: Restore a specific UBB tray
- `--gen-config [FILE]`: Generate template TOML config file
- `--use-config <MODE> <FILE>`: Use config file for isolation/restoration
- `--gui`: Launch GUI interface

### Notes
- Requires physical hardware access (cannot run in VM/container)
- Requires exactly 32 chips to be detected
- Must run `tt-smi -glx_reset` after isolation/restore operations

[Unreleased]: https://github.com/tenstorrent/tt-glx-partition/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/tenstorrent/tt-glx-partition/releases/tag/v0.1.0
