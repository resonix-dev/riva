# Riva

Riva is a provider-agnostic media stream metadata extractor written in Rust. It ships with first-class support for SoundCloud and YouTube and exposes ergonomic helpers so you can build downloaders, previewers, or stream routers without babysitting ever-changing web APIs.

## Features

- **Provider abstraction** – a single crate that exposes provider-specific modules gated behind feature flags (`soundcloud`, `youtube`).
- **Async-first API** – powered by `reqwest` and `tokio` so you can integrate into any modern async application.
- **Strict normalization** – helpers that sanitize user-provided URLs/IDs before the networking layer touches them.
- **Testing bench** – deterministic unit tests and integration tests located under `tests/` cover URL normalization and extractor helpers.
- **Example client** – `example/` contains a miniature CLI showing how to list all SoundCloud streams for a track.

## Getting Started

### Requirements

- Rust 1.81+ (edition 2024)
- `cargo` (bundled with Rustup)

### Installation

Add Riva to your project with the providers you need:

```bash
cargo add riva --features youtube,soundcloud
```

Disable providers you do not plan to use:

```bash
cargo add riva --no-default-features --features youtube
```

### Quick Example

```rust
use riva::soundcloud::{extract_streams, StreamInfo};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let track_url = "https://soundcloud.com/kordhell/trageluxe";
    let streams: Vec<StreamInfo> = extract_streams(track_url).await?;

    for stream in streams {
        println!("{} {}", stream.protocol, stream.mime_type);
        println!("  URL: {}", stream.url);
    }

    Ok(())
}
```

An executable example that mirrors this snippet lives in `example/src/main.rs` and can be executed via:

```bash
cargo run --example riva-test -- https://soundcloud.com/kordhell/trageluxe
```

## Provider Matrix

| Provider    | Feature flag | Normalization helper        | Notes |
|-------------|--------------|-----------------------------|-------|
| SoundCloud  | `soundcloud` | `riva::soundcloud::normalize_track_url` | Discovers client IDs automatically and filters unsupported transcodings. |
| YouTube     | `youtube`    | `riva::youtube::normalize_video_id`     | Uses the lightweight mobile API and filters inaccessible streams. |

## Testing Bench

The repository ships with a deterministic testing bench that avoids hitting live provider APIs:

- `tests/normalization.rs` validates public normalization helpers for both providers.
- Module-level unit tests check extractor utilities such as SoundCloud transcoding filters and YouTube stream classification helpers.

Run the entire suite locally with:

```bash
cargo test
```

CI runs `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test` on every pull request to ensure the testing bench stays green.

## Project Structure

```
src/
  providers/
    soundcloud/   # SoundCloud extractor, models, and URL normalization
    youtube/      # YouTube extractor, models, and helpers
example/
  src/main.rs    # Minimal binary for manual experiments
```

## Contributing

See `CONTRIBUTING.md` for detailed guidelines, coding standards, and the review checklist. By participating you agree to abide by the `CODE_OF_CONDUCT.md`.

## Security

Security disclosures should follow the process documented in `SECURITY.md`. Please avoid filing public issues for vulnerabilities.

## License

Distributed under the terms of the MIT License. See `LICENSE` for details.
