# Contributing to Riva

First off, thank you for investing your time in improving Riva! The following guidelines help us review changes consistently and keep the project healthy.

## Ground Rules

- Be excellent to each other. Read and follow the expectations laid out in `CODE_OF_CONDUCT.md`.
- Prefer public issues for bugs and feature requests so everyone can follow along.
- Keep changes focused. Small, reviewable pull requests land faster.

## Development Environment

1. Install the latest stable Rust toolchain (`rustup default stable`).
2. Clone the repo and install dependencies:
   ```bash
   git clone https://github.com/resonix-dev/riva.git
   cd riva
   cargo fetch
   ```
3. Enable only the providers you need when iterating: `cargo test --no-default-features --features youtube`.

## Workflow

1. **Discuss** – open an issue (or comment on an existing one) to describe what you plan to change.
2. **Develop** – branch off `main` and write your code plus tests:
   ```bash
   git checkout -b feature/my-change
   cargo fmt
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test
   ```
3. **Document** – update `README.md`, `CHANGELOG.md`, or relevant docs whenever behavior changes.
4. **Pull Request** – fill out the PR template and ensure the CI workflow is green.

## Testing Bench

- Unit tests live next to the modules they cover (for example, `src/providers/soundcloud/extractor.rs`).
- Integration tests live under `tests/`. Add a new file when validating a cross-module flow.
- Prefer deterministic fixtures over live API calls. Record small JSON snippets when needed.

## Coding Style

- Run `cargo fmt` before committing.
- Treat Clippy warnings as errors (`cargo clippy -- -D warnings`).
- Keep functions small and focused. Extract helpers if a function grows beyond ~40 lines.
- When adding comments, explain _why_ rather than _what_.

## Commit Messages

- Use the imperative mood ("Fix race condition" not "Fixed").
- Reference issues when relevant: `Fix #123: normalize YouTube shorts URLs`.

## Releasing

1. Update `CHANGELOG.md` with noteworthy changes.
2. Bump `Cargo.toml`.
3. Tag the release (`git tag vX.Y.Z && git push --tags`).

If you have any questions, feel free to open a discussion or ping a maintainer in your issue/PR. Happy hacking! :rocket:
