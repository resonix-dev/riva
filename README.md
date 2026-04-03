# riva

Server-backed Rust client for the Riva media proxy.

By default, this crate targets the riva servers.
You can override the server with environment variables.

## Features

- `youtube` (enabled by default)
- `soundcloud` (enabled by default)

Disable defaults if needed:

```toml
[dependencies]
riva = { version = "1", default-features = false, features = ["youtube"] }
```

## Environment Variables

Base URL (first non-empty value wins):

- `RIVA_BASE_URL`
- `RIVA_SERVER_URL`
- `RIVA_URL`

Access secret (optional, first non-empty value wins):

- `RIVA_ACCESS_SECRET`
- `RIVA_API_KEY`
- `RIVA_TOKEN`

If an access secret is set, the client sends `Authorization: Bearer <secret>`.

## Quick Start

```rust
use riva::RivaClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = RivaClient::from_env()?;

    let health = client.health().await?;
    println!("{} @ {}", health.status, health.time);

    Ok(())
}
```

## YouTube

```rust
use riva::{RivaClient, YoutubeClientType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = RivaClient::from_env()?;

    let info = client
        .youtube_info("rkaiKn5iGzc", Some(YoutubeClientType::Web))
        .await?;
    println!("video info: {info}");

    let response = client
        .youtube_stream("rkaiKn5iGzc", 18, Some(YoutubeClientType::Web))
        .await?;

    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let bytes = chunk?;
        println!("received {} bytes", bytes.len());
    }

    Ok(())
}
```

For `bytes_stream()` consumption, add:

```toml
futures-util = "0.3"
```

and import:

```rust
use futures_util::StreamExt;
```

## SoundCloud

```rust
use riva::RivaClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = RivaClient::from_env()?;

    let response = client
        .soundcloud_stream("https://soundcloud.com/kordhell/trageluxe")
        .await?;

    let bytes = response.bytes().await?;
    println!("downloaded {} bytes", bytes.len());

    Ok(())
}
```

## API

- `RivaClient::from_env()`
- `RivaClient::new(config)`
- `RivaClient::health()`
- `RivaClient::youtube_info(...)` (feature: `youtube`)
- `RivaClient::youtube_stream(...)` (feature: `youtube`)
- `RivaClient::soundcloud_stream(...)` (feature: `soundcloud`)
