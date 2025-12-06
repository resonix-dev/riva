use url::Url;

use super::extractor::RivaError;

pub fn normalize_track_url(input: &str) -> Result<String, RivaError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(RivaError::InvalidUrl);
    }

    let candidate = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    };

    let parsed = Url::parse(&candidate).map_err(|_| RivaError::InvalidUrl)?;
    let host = parsed.host_str().unwrap_or_default();
    if host.contains("soundcloud.com") || host == "snd.sc" {
        return Ok(parsed.into());
    }

    Err(RivaError::InvalidUrl)
}
