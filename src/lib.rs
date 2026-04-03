mod client;
mod config;
mod error;

#[cfg(feature = "youtube")]
pub use client::YoutubeClientType;
pub use client::{HealthResponse, RivaClient};
pub use config::{DEFAULT_RIVA_BASE_URL, RivaConfig};
pub use error::{ApiErrorBody, RivaError};
