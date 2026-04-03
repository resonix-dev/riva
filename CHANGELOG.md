# Changelog

All notable changes to this project will be documented in this file. The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Dates are formatted as `DD-MM-YYYY`.

## [v1.0.0] - 03.04.2026
### Changed
- Migrated the crate to a server-backed architecture. Provider extraction logic now lives in the Riva server.
- Updated crate metadata and version to `1.0.0`.

## [v0.1.0] - 06.12.2025
### Added
- Project-wide README describing features, provider matrix, and testing bench.
- Contributor, security, and code of conduct guidelines.
- Deterministic testing bench (unit + integration tests) for normalization helpers.
- GitHub workflow, PR template, and issue templates (see `.github/`).

### Fixed
- Exposed normalization helpers for SoundCloud and YouTube via the public API.

### Security
- Documented coordinated disclosure process in `SECURITY.md`.
