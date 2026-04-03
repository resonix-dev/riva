use std::env;

use url::Url;

use crate::error::RivaError;

pub const DEFAULT_RIVA_BASE_URL: &str = "https://riva.resonix.dev";

const BASE_URL_ENV_KEYS: &[&str] = &["RIVA_BASE_URL", "RIVA_SERVER_URL", "RIVA_URL"];
const SECRET_ENV_KEYS: &[&str] = &["RIVA_ACCESS_SECRET", "RIVA_API_KEY", "RIVA_TOKEN"];

#[derive(Debug, Clone)]
pub struct RivaConfig {
    pub base_url: Url,
    pub access_secret: Option<String>,
}

impl Default for RivaConfig {
    fn default() -> Self {
        Self {
            base_url: Url::parse(DEFAULT_RIVA_BASE_URL).expect("default URL is valid"),
            access_secret: None,
        }
    }
}

impl RivaConfig {
    pub fn from_env() -> Result<Self, RivaError> {
        let base_url =
            env_value(BASE_URL_ENV_KEYS).unwrap_or_else(|| DEFAULT_RIVA_BASE_URL.to_owned());

        let mut config = Self::new(base_url)?;
        config.access_secret = env_value(SECRET_ENV_KEYS);
        Ok(config)
    }

    pub fn new(base_url: impl AsRef<str>) -> Result<Self, RivaError> {
        let mut parsed = Url::parse(base_url.as_ref()).map_err(RivaError::InvalidBaseUrl)?;
        if !parsed.path().ends_with('/') {
            let path = format!("{}/", parsed.path());
            parsed.set_path(&path);
        }

        Ok(Self {
            base_url: parsed,
            access_secret: None,
        })
    }

    pub fn with_access_secret(mut self, access_secret: impl Into<String>) -> Self {
        self.access_secret = Some(access_secret.into());
        self
    }
}

fn env_value(keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| env::var(key).ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_adds_trailing_slash_to_base_url() {
        let config = RivaConfig::new("https://example.com/api").expect("config should build");
        assert_eq!(config.base_url.as_str(), "https://example.com/api/");
    }
}
