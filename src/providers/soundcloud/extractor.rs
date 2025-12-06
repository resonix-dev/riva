use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::{Client, Response, StatusCode};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::from_slice;
use thiserror::Error;
use tokio::sync::OnceCell;

use super::models::{StreamInfo, Track, Transcoding, TranscodingLocation};
use super::normalize::normalize_track_url;

const RESOLVE_ENDPOINT: &str = "https://api-v2.soundcloud.com/resolve";
const CLIENT_ID_ENDPOINT: &str = "https://api-v2.soundcloud.com/client_id";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36";
const HOMEPAGE_URL: &str = "https://soundcloud.com";
const MAX_ASSET_PROBES: usize = 16;

type Result<T> = std::result::Result<T, RivaError>;

static CLIENT_ID_CACHE: OnceCell<String> = OnceCell::const_new();

#[derive(Debug, Error)]
pub enum RivaError {
    #[error("invalid or unsupported SoundCloud URL")]
    InvalidUrl,
    #[error("network request failed: {0}")]
    Network(#[from] reqwest::Error),
    #[error("failed to parse SoundCloud response: {0}")]
    Json(#[from] serde_json::Error),
    #[error("SoundCloud client id discovery failed")]
    ClientId,
    #[error("the provided URL did not resolve to a track")]
    UnsupportedResource,
    #[error("track does not expose streaming formats")]
    NoStreams,
}

pub async fn extract_streams(track_url: &str) -> Result<Vec<StreamInfo>> {
    let normalized = normalize_track_url(track_url)?;

    let client = Client::builder().user_agent(USER_AGENT).build()?;

    let client_id = get_client_id(&client).await?;
    let track = resolve_track(&client, &normalized, client_id).await?;

    let media = track.media.ok_or(RivaError::NoStreams)?;
    let mut streams = Vec::new();

    for transcoding in media.transcodings.into_iter() {
        if should_skip_transcoding(&transcoding) {
            continue;
        }

        if let Some(stream) = fetch_transcoding(
            &client,
            transcoding,
            client_id,
            track.track_authorization.as_deref(),
            track.artwork_url.as_deref(),
        )
        .await?
        {
            streams.push(stream);
        }
    }

    if streams.is_empty() {
        return Err(RivaError::NoStreams);
    }

    Ok(streams)
}

async fn resolve_track(client: &Client, url: &str, client_id: &str) -> Result<Track> {
    let response = client
        .get(RESOLVE_ENDPOINT)
        .query(&[("url", url), ("client_id", client_id)])
        .send()
        .await?
        .error_for_status()?;

    let track: Track = parse_json(response).await?;
    if track.kind.as_deref() != Some("track") {
        return Err(RivaError::UnsupportedResource);
    }

    Ok(track)
}

async fn fetch_transcoding(
    client: &Client,
    transcoding: Transcoding,
    client_id: &str,
    track_authorization: Option<&str>,
    artwork_url: Option<&str>,
) -> Result<Option<StreamInfo>> {
    if transcoding.url.is_empty() {
        return Ok(None);
    }

    let mut request = client
        .get(&transcoding.url)
        .query(&[("client_id", client_id)]);
    if let Some(token) = track_authorization {
        request = request.query(&[("track_authorization", token)]);
    }

    let response = request.send().await?;
    if matches!(
        response.status(),
        StatusCode::NOT_FOUND | StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
    ) {
        return Ok(None);
    }
    let response = response.error_for_status()?;
    let redirect: TranscodingLocation = parse_json(response).await?;

    Ok(transcoding.into_stream_info(redirect.url, artwork_url))
}

fn should_skip_transcoding(transcoding: &Transcoding) -> bool {
    if transcoding.url.is_empty() || transcoding.format.is_none() {
        return true;
    }

    if transcoding
        .preset
        .as_deref()
        .is_some_and(|preset| preset.starts_with("abr"))
    {
        return true;
    }

    if transcoding
        .format
        .as_ref()
        .map(|format| format.protocol.as_str())
        .is_some_and(|protocol| protocol.starts_with("ctr-") || protocol.starts_with("cbc-"))
    {
        return true;
    }

    false
}

async fn get_client_id(client: &Client) -> Result<&'static str> {
    CLIENT_ID_CACHE
        .get_or_try_init(|| async { fetch_client_id(client.clone()).await })
        .await
        .map(|value| value.as_str())
}

async fn fetch_client_id(client: Client) -> Result<String> {
    if let Ok(id) = fetch_client_id_endpoint(&client).await {
        return Ok(id);
    }

    fetch_client_id_from_html(client).await
}

async fn fetch_client_id_endpoint(client: &Client) -> Result<String> {
    #[derive(Deserialize)]
    struct ClientIdPayload {
        client_id: String,
    }

    let response = client
        .get(CLIENT_ID_ENDPOINT)
        .send()
        .await?
        .error_for_status()?;

    let payload: ClientIdPayload = parse_json(response).await?;
    if payload.client_id.is_empty() {
        return Err(RivaError::ClientId);
    }

    Ok(payload.client_id)
}

async fn fetch_client_id_from_html(client: Client) -> Result<String> {
    let homepage = client.get(HOMEPAGE_URL).send().await?.error_for_status()?;
    let html = homepage.text().await?;

    if let Some(id) = extract_client_id(&html) {
        return Ok(id);
    }

    for script_url in discover_asset_scripts(&html)
        .into_iter()
        .take(MAX_ASSET_PROBES)
    {
        let script = client.get(&script_url).send().await?.error_for_status()?;
        let body = script.text().await?;
        if let Some(id) = extract_client_id(&body) {
            return Ok(id);
        }
    }

    Err(RivaError::ClientId)
}

async fn parse_json<T>(response: Response) -> Result<T>
where
    T: DeserializeOwned,
{
    let bytes = response.bytes().await?;
    Ok(from_slice(&bytes)?)
}

fn extract_client_id(source: &str) -> Option<String> {
    static CLIENT_ID_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"client_id[\"']?\s*[:=]\s*[\"']?([a-zA-Z0-9]{16,})"#)
            .expect("valid client_id regex")
    });

    CLIENT_ID_REGEX
        .captures(source)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
}

fn discover_asset_scripts(source: &str) -> Vec<String> {
    static SCRIPT_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"https://a-v2\.sndcdn\.com/assets/[^"]+\.js"#).expect("valid asset regex")
    });

    SCRIPT_REGEX
        .find_iter(source)
        .map(|m| m.as_str().to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::models::{Transcoding, TranscodingFormat};
    use super::*;

    fn base_transcoding() -> Transcoding {
        Transcoding {
            url: "https://api-v2.soundcloud.com/transcoding".into(),
            preset: Some("progressive".into()),
            quality: Some("hq".into()),
            snipped: Some(false),
            format: Some(TranscodingFormat {
                protocol: "https".into(),
                mime_type: "audio/mp4".into(),
            }),
            duration: Some(1000),
        }
    }

    #[test]
    fn skip_transcoding_detects_invalid_inputs() {
        let mut transcoding = base_transcoding();
        assert!(!should_skip_transcoding(&transcoding));

        transcoding.url.clear();
        assert!(should_skip_transcoding(&transcoding));

        let mut abr = base_transcoding();
        abr.preset = Some("abr100".into());
        assert!(should_skip_transcoding(&abr));

        let mut ctr = base_transcoding();
        ctr.format = Some(TranscodingFormat {
            protocol: "ctr-hls".into(),
            mime_type: "audio/mp4".into(),
        });
        assert!(should_skip_transcoding(&ctr));
    }

    #[test]
    fn extracts_client_id_from_html() {
        let html = r#"
            <script>const client_id="1234567890abcdef";</script>
        "#;
        let extracted = extract_client_id(html);
        assert_eq!(extracted.as_deref(), Some("1234567890abcdef"));
    }

    #[test]
    fn discovers_asset_urls() {
        let html = r#"
            <script src="https://a-v2.sndcdn.com/assets/app-cb123.js"></script>
            <script src="https://a-v2.sndcdn.com/assets/app-cb124.js"></script>
        "#;
        let urls = discover_asset_scripts(html);
        assert_eq!(urls.len(), 2);
        assert!(urls[0].starts_with("https://a-v2.sndcdn.com/assets/"));
    }
}
