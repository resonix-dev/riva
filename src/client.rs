use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use reqwest::{Client, RequestBuilder, Response};
use serde::{Deserialize, Serialize};
#[cfg(feature = "youtube")]
use serde_json::Value;
use url::Url;

use crate::config::RivaConfig;
use crate::error::{ApiErrorBody, RivaError};

#[derive(Debug, Clone)]
pub struct RivaClient {
    http: Client,
    config: RivaConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub time: String,
}

#[cfg(feature = "youtube")]
#[derive(Debug, Clone, Copy)]
pub enum YoutubeClientType {
    Web,
    Android,
    Ios,
}

#[cfg(feature = "youtube")]
impl YoutubeClientType {
    fn as_str(self) -> &'static str {
        match self {
            Self::Web => "WEB",
            Self::Android => "ANDROID",
            Self::Ios => "IOS",
        }
    }
}

impl RivaClient {
    pub fn new(config: RivaConfig) -> Result<Self, RivaError> {
        let mut headers = HeaderMap::new();

        if let Some(secret) = &config.access_secret {
            let bearer = format!("Bearer {secret}");
            let value = HeaderValue::from_str(&bearer).map_err(|_| {
                RivaError::InvalidInput("access secret contains invalid header bytes".to_owned())
            })?;
            headers.insert(AUTHORIZATION, value);
        }

        let http = Client::builder().default_headers(headers).build()?;

        Ok(Self { http, config })
    }

    pub fn from_env() -> Result<Self, RivaError> {
        let config = RivaConfig::from_env()?;
        Self::new(config)
    }

    pub fn config(&self) -> &RivaConfig {
        &self.config
    }

    pub async fn health(&self) -> Result<HealthResponse, RivaError> {
        let url = self.join_path("health")?;
        let response = self.send(self.http.get(url)).await?;
        Ok(response.json::<HealthResponse>().await?)
    }

    #[cfg(feature = "youtube")]
    pub async fn youtube_info(
        &self,
        id: &str,
        client_type: Option<YoutubeClientType>,
    ) -> Result<Value, RivaError> {
        let id = normalize_youtube_id(id)?;
        let mut url = self.join_path(&format!("providers/youtube/{id}/info"))?;
        if let Some(client_type) = client_type {
            url.query_pairs_mut()
                .append_pair("client_type", client_type.as_str());
        }

        let response = self.send(self.http.get(url)).await?;
        Ok(response.json::<Value>().await?)
    }

    #[cfg(feature = "youtube")]
    pub async fn youtube_stream(
        &self,
        id: &str,
        itag: u32,
        client_type: Option<YoutubeClientType>,
    ) -> Result<Response, RivaError> {
        let id = normalize_youtube_id(id)?;
        let mut url = self.join_path(&format!("providers/youtube/{id}/stream/{itag}"))?;
        if let Some(client_type) = client_type {
            url.query_pairs_mut()
                .append_pair("client_type", client_type.as_str());
        }

        self.send(self.http.get(url)).await
    }

    #[cfg(feature = "soundcloud")]
    pub async fn soundcloud_stream(&self, track_url: &str) -> Result<Response, RivaError> {
        let track_url = normalize_soundcloud_track_url(track_url)?;
        let mut url = self.join_path("providers/soundcloud/stream")?;
        url.query_pairs_mut().append_pair("url", &track_url);
        self.send(self.http.get(url)).await
    }

    fn join_path(&self, path: &str) -> Result<Url, RivaError> {
        self.config
            .base_url
            .join(path)
            .map_err(RivaError::InvalidBaseUrl)
    }

    async fn send(&self, request: RequestBuilder) -> Result<Response, RivaError> {
        let response = request.send().await?;
        if response.status().is_success() {
            return Ok(response);
        }

        let status = response.status();
        let body = response.text().await?;
        let message = serde_json::from_str::<ApiErrorBody>(&body)
            .map(|v| v.error)
            .unwrap_or(body);

        Err(RivaError::Api { status, message })
    }
}

#[cfg(any(feature = "youtube", feature = "soundcloud"))]
fn validate_not_empty(field_name: &str, value: &str) -> Result<(), RivaError> {
    if value.trim().is_empty() {
        return Err(RivaError::InvalidInput(format!(
            "{field_name} cannot be empty"
        )));
    }
    Ok(())
}

#[cfg(feature = "youtube")]
fn normalize_youtube_id(input: &str) -> Result<String, RivaError> {
    let raw = input.trim();
    validate_not_empty("youtube id", raw)?;

    if is_youtube_video_id(raw) {
        return Ok(raw.to_string());
    }

    if let Some(url) = parse_possible_url(raw) {
        if let Some(id) = youtube_id_from_url(&url) {
            return Ok(id);
        }
    }

    let candidate = raw
        .split(['?', '&', '/', '#'])
        .next()
        .map(str::trim)
        .unwrap_or_default();
    if is_youtube_video_id(candidate) {
        return Ok(candidate.to_string());
    }

    Err(RivaError::InvalidInput(
        "could not extract a valid YouTube video id from input".to_owned(),
    ))
}

#[cfg(feature = "youtube")]
fn youtube_id_from_url(url: &Url) -> Option<String> {
    let host = url.host_str()?.to_ascii_lowercase();

    if host == "youtu.be" {
        return first_path_segment(url)
            .filter(|id| is_youtube_video_id(id))
            .map(ToString::to_string);
    }

    if !host.ends_with("youtube.com") {
        return None;
    }

    if let Some(v) = url.query_pairs().find_map(|(k, v)| {
        if k == "v" || k == "vi" {
            Some(v.into_owned())
        } else {
            None
        }
    }) {
        if is_youtube_video_id(&v) {
            return Some(v);
        }
    }

    let mut segments = url.path_segments()?;
    let first = segments.next()?;
    let second = segments.next();

    match first {
        "shorts" | "embed" | "v" | "live" => second
            .filter(|id| is_youtube_video_id(id))
            .map(ToString::to_string),
        _ => None,
    }
}

#[cfg(feature = "youtube")]
fn is_youtube_video_id(value: &str) -> bool {
    value.len() == 11
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

#[cfg(feature = "soundcloud")]
fn normalize_soundcloud_track_url(input: &str) -> Result<String, RivaError> {
    let raw = input.trim();
    validate_not_empty("soundcloud track url", raw)?;

    if let Some(url) = parse_possible_url(raw)
        && is_soundcloud_host(&url)
    {
        return Ok(url.into());
    }

    let cleaned = raw.trim_matches('/');
    let mut parts = cleaned.split('/');
    let artist = parts.next().unwrap_or_default().trim();
    let track = parts.next().unwrap_or_default().trim();
    let has_only_artist_and_track = parts.next().is_none();

    if !artist.is_empty() && !track.is_empty() && has_only_artist_and_track {
        return Ok(format!("https://soundcloud.com/{artist}/{track}"));
    }

    Err(RivaError::InvalidInput(
        "soundcloud input must be a soundcloud URL or in the format artist/track".to_owned(),
    ))
}

#[cfg(any(feature = "youtube", feature = "soundcloud"))]
fn parse_possible_url(input: &str) -> Option<Url> {
    Url::parse(input).ok().or_else(|| {
        let prefixed = format!("https://{input}");
        Url::parse(&prefixed).ok()
    })
}

#[cfg(feature = "youtube")]
fn first_path_segment(url: &Url) -> Option<&str> {
    url.path_segments()?.find(|segment| !segment.is_empty())
}

#[cfg(feature = "soundcloud")]
fn is_soundcloud_host(url: &Url) -> bool {
    url.host_str()
        .map(|h| {
            let host = h.to_ascii_lowercase();
            host == "soundcloud.com" || host.ends_with(".soundcloud.com")
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DEFAULT_RIVA_BASE_URL;

    #[test]
    fn client_defaults_to_public_server() {
        let client = RivaClient::new(RivaConfig::default()).expect("client should build");
        assert_eq!(
            client.config().base_url.as_str(),
            format!("{DEFAULT_RIVA_BASE_URL}/")
        );
    }

    #[cfg(feature = "youtube")]
    #[test]
    fn youtube_client_type_values_match_api() {
        assert_eq!(YoutubeClientType::Web.as_str(), "WEB");
        assert_eq!(YoutubeClientType::Android.as_str(), "ANDROID");
        assert_eq!(YoutubeClientType::Ios.as_str(), "IOS");
    }

    #[cfg(any(feature = "youtube", feature = "soundcloud"))]
    #[test]
    fn validates_non_empty_fields() {
        let result = validate_not_empty("x", "   ");
        assert!(matches!(result, Err(RivaError::InvalidInput(_))));
    }

    #[cfg(feature = "youtube")]
    #[test]
    fn normalizes_youtube_id_from_direct_value() {
        let id = normalize_youtube_id("dQw4w9WgXcQ").expect("id should normalize");
        assert_eq!(id, "dQw4w9WgXcQ");
    }

    #[cfg(feature = "youtube")]
    #[test]
    fn normalizes_youtube_id_from_common_urls() {
        let cases = [
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
            "https://youtu.be/dQw4w9WgXcQ?t=43",
            "youtube.com/shorts/dQw4w9WgXcQ",
        ];

        for case in cases {
            let id = normalize_youtube_id(case).expect("url should normalize");
            assert_eq!(id, "dQw4w9WgXcQ");
        }
    }

    #[cfg(feature = "soundcloud")]
    #[test]
    fn normalizes_soundcloud_from_url_or_artist_track() {
        let url = normalize_soundcloud_track_url("https://soundcloud.com/kordhell/trageluxe")
            .expect("url should normalize");
        assert_eq!(url, "https://soundcloud.com/kordhell/trageluxe");

        let from_artist_track = normalize_soundcloud_track_url("kordhell/trageluxe")
            .expect("artist/track should normalize");
        assert_eq!(
            from_artist_track,
            "https://soundcloud.com/kordhell/trageluxe"
        );
    }

    #[test]
    fn can_create_new_base_url() {
        let config = RivaConfig::new("https://riva.resonix.dev").expect("config should be valid");
        let client = RivaClient::new(config).expect("client should build");
        assert_eq!(client.config.base_url.as_str(), "https://riva.resonix.dev/");
    }
}
