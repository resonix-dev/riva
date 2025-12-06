use reqwest::Client;
use serde_json::from_slice;
use thiserror::Error;

use super::models::{ClientContext, PlayerRequest, PlayerResponse, StreamInfo, player_js_url};
use super::normalize::normalize_video_id;
use super::signature::{SignatureDecipher, SignatureResolver};
use super::validator::filter_working_streams;

const PLAYER_ENDPOINT: &str = "https://www.youtube.com/youtubei/v1/player";
const ANDROID_API_KEY: &str = "AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8";
const ANDROID_CLIENT_VERSION: &str = "19.33.34";
const ANDROID_USER_AGENT: &str = "com.google.android.youtube/19.33.34 (Linux; U; Android 13) gzip";

type Result<T> = std::result::Result<T, RivaError>;

#[derive(Debug, Error)]
pub enum RivaError {
    #[error("invalid or unsupported YouTube URL")]
    InvalidUrl,
    #[error("network request failed: {0}")]
    Network(#[from] reqwest::Error),
    #[error("failed to parse API response: {0}")]
    Json(#[from] serde_json::Error),
    #[error("no playable streams found for this video")]
    NoStreams,
    #[error("YouTube blocked playback: {0}")]
    Playability(String),
}

pub async fn extract_streams(video_url: &str) -> Result<Vec<StreamInfo>> {
    let video_id = normalize_video_id(video_url)?;

    let client = Client::builder().user_agent(ANDROID_USER_AGENT).build()?;

    let request = PlayerRequest::new(&video_id, android_client_context());
    let response = client
        .post(format!("{PLAYER_ENDPOINT}?key={ANDROID_API_KEY}"))
        .json(&request)
        .send()
        .await?
        .error_for_status()?;

    let payload: PlayerResponse = {
        let bytes = response.bytes().await?;
        from_slice(&bytes)?
    };

    if let Some(status) = payload.playability_status.as_ref() {
        let code = status.status.as_deref().unwrap_or("UNKNOWN");
        if code != "OK" {
            let reason = status
                .reason
                .clone()
                .unwrap_or_else(|| "Unknown restriction".to_string());
            return Err(RivaError::Playability(reason));
        }
    }

    let js_player_url = player_js_url(&payload).map(|url| url.to_owned());
    let streaming = payload.streaming_data.ok_or(RivaError::NoStreams)?;
    let signature_decipher = SignatureDecipher;
    let signature_resolver = js_player_url.as_deref().map(|url| SignatureResolver {
        js_url: url,
        decipher: &signature_decipher,
    });

    let mut streams = Vec::new();

    if let Some(formats) = streaming.formats {
        streams.extend(
            formats
                .into_iter()
                .filter_map(|f| f.into_stream_info(false, signature_resolver.as_ref())),
        );
    }

    if let Some(formats) = streaming.adaptive_formats {
        streams.extend(
            formats
                .into_iter()
                .filter_map(|f| f.into_stream_info(true, signature_resolver.as_ref())),
        );
    }

    let streams = filter_working_streams(&client, streams).await;

    if streams.is_empty() {
        return Err(RivaError::NoStreams);
    }

    Ok(streams)
}

fn android_client_context() -> ClientContext<'static> {
    ClientContext {
        client_name: "ANDROID",
        client_version: ANDROID_CLIENT_VERSION,
        android_sdk_version: 33,
        os_version: "13",
        device_make: "Google",
        device_model: "Pixel 7",
        user_agent: ANDROID_USER_AGENT,
        hl: "en",
        gl: "US",
        time_zone: "UTC",
        platform: "MOBILE",
    }
}
