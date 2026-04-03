use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorBody {
    pub error: String,
}

#[derive(Debug, Error)]
pub enum RivaError {
    #[error("invalid base URL: {0}")]
    InvalidBaseUrl(url::ParseError),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("http client error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("request failed with status {status}: {message}")]
    Api { status: StatusCode, message: String },
}
