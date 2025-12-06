use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use url::{Url, form_urlencoded};

use super::signature::SignatureResolver;

#[derive(Debug, Clone)]
pub struct StreamInfo {
    pub itag: u32,
    pub url: String,
    pub mime_type: Option<String>,
    pub bitrate: Option<u64>,
    pub quality_label: Option<String>,
    pub audio_quality: Option<String>,
    pub fps: Option<u32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub content_length: Option<u64>,
    pub approx_duration_ms: Option<u64>,
    pub audio_sample_rate: Option<u32>,
    pub audio_channels: Option<u32>,
    pub has_audio: bool,
    pub has_video: bool,
    pub is_adaptive: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerRequest<'a> {
    pub context: RequestContext<'a>,
    pub video_id: &'a str,
    pub content_check_ok: bool,
    pub racy_check_ok: bool,
    pub playback_context: PlaybackContext,
}

impl<'a> PlayerRequest<'a> {
    pub fn new(video_id: &'a str, client_context: ClientContext<'a>) -> Self {
        Self {
            context: RequestContext {
                client: client_context,
            },
            video_id,
            content_check_ok: true,
            racy_check_ok: true,
            playback_context: PlaybackContext {
                content_playback_context: ContentPlaybackContext {
                    html5_preference: "HTML5_PREF_WANTS",
                },
            },
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestContext<'a> {
    pub client: ClientContext<'a>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientContext<'a> {
    pub client_name: &'a str,
    pub client_version: &'a str,
    pub android_sdk_version: u32,
    pub os_version: &'a str,
    pub device_make: &'a str,
    pub device_model: &'a str,
    pub user_agent: &'a str,
    pub hl: &'a str,
    pub gl: &'a str,
    pub time_zone: &'a str,
    pub platform: &'a str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackContext {
    pub content_playback_context: ContentPlaybackContext,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentPlaybackContext {
    pub html5_preference: &'static str,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerResponse {
    pub streaming_data: Option<StreamingData>,
    pub playability_status: Option<PlayabilityStatus>,
    pub player_config: Option<PlayerConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerConfig {
    pub assets: Option<PlayerAssets>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerAssets {
    pub js: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamingData {
    pub formats: Option<Vec<Format>>,
    pub adaptive_formats: Option<Vec<Format>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayabilityStatus {
    pub status: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Format {
    pub itag: Option<u32>,
    pub url: Option<String>,
    pub signature_cipher: Option<String>,
    pub cipher: Option<String>,
    pub mime_type: Option<String>,
    pub bitrate: Option<u64>,
    pub average_bitrate: Option<u64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fps: Option<u32>,
    pub quality_label: Option<String>,
    pub quality: Option<String>,
    pub audio_quality: Option<String>,
    pub audio_sample_rate: Option<String>,
    pub audio_channels: Option<u32>,
    pub content_length: Option<String>,
    pub approx_duration_ms: Option<String>,
}

impl Format {
    pub fn into_stream_info(
        self,
        is_adaptive: bool,
        resolver: Option<&SignatureResolver>,
    ) -> Option<StreamInfo> {
        let itag = self.itag?;
        let url = self.resolved_url(resolver)?;

        let mime_type = self.mime_type.clone();
        let (has_audio, has_video) = classify_mime(mime_type.as_deref());

        Some(StreamInfo {
            itag,
            url,
            mime_type,
            bitrate: self.average_bitrate.or(self.bitrate),
            quality_label: self.quality_label.or(self.quality),
            audio_quality: self.audio_quality,
            fps: self.fps,
            width: self.width,
            height: self.height,
            content_length: self
                .content_length
                .and_then(|value| value.parse::<u64>().ok()),
            approx_duration_ms: self
                .approx_duration_ms
                .and_then(|value| value.parse::<u64>().ok()),
            audio_sample_rate: self
                .audio_sample_rate
                .and_then(|value| value.parse::<u32>().ok()),
            audio_channels: self.audio_channels,
            has_audio,
            has_video,
            is_adaptive,
        })
    }

    fn resolved_url(&self, resolver: Option<&SignatureResolver>) -> Option<String> {
        if let Some(url) = &self.url {
            return Some(decode_js_url(url));
        }

        let cipher = self.signature_cipher.as_ref().or(self.cipher.as_ref())?;
        let params: HashMap<_, _> = form_urlencoded::parse(cipher.as_bytes())
            .into_owned()
            .collect();
        let base_url = params.get("url").map(|value| decode_js_url(value))?;
        let target_param = params.get("sp").map(String::as_str).unwrap_or("signature");

        if let Some(sig) = params.get("sig").or_else(|| params.get("signature")) {
            return append_signature(&base_url, target_param, sig);
        }

        let scrambled = params.get("s")?;
        let resolver = resolver?;
        let deciphered = resolver.apply(scrambled).ok()?;
        append_signature(&base_url, target_param, &deciphered)
    }
}

pub fn player_js_url(payload: &PlayerResponse) -> Option<&str> {
    payload
        .player_config
        .as_ref()
        .and_then(|config| config.assets.as_ref())
        .and_then(|assets| assets.js.as_deref())
}

fn classify_mime(mime: Option<&str>) -> (bool, bool) {
    if let Some(mime) = mime {
        if mime.starts_with("audio/") {
            return (true, false);
        }

        if mime.starts_with("video/") {
            let lower = mime.to_ascii_lowercase();
            let has_audio =
                lower.contains("mp4a") || lower.contains("opus") || lower.contains("vorbis");
            return (has_audio, true);
        }
    }

    (false, false)
}

fn decode_js_url(value: &str) -> String {
    value.replace("\\u0026", "&")
}

fn append_signature(base: &str, param: &str, signature: &str) -> Option<String> {
    let mut url = Url::parse(base).ok()?;
    {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair(param, signature);
    }
    Some(url.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_mime_detects_stream_types() {
        assert_eq!(classify_mime(Some("audio/mp4")), (true, false));
        assert_eq!(
            classify_mime(Some("video/mp4; codecs=\"avc1.4d401f, mp4a.40.2\"")),
            (true, true)
        );
        assert_eq!(classify_mime(Some("video/webm")), (false, true));
        assert_eq!(classify_mime(None), (false, false));
    }

    #[test]
    fn decode_js_url_unescapes_ampersands() {
        let source = "https://example.com/watch?foo=bar\\u0026baz=qux";
        assert_eq!(
            decode_js_url(source),
            "https://example.com/watch?foo=bar&baz=qux"
        );
    }

    #[test]
    fn append_signature_adds_param() {
        let base = "https://example.com/video?foo=bar";
        let result = append_signature(base, "sig", "12345").unwrap();
        assert!(result.contains("sig=12345"));
        assert!(result.starts_with("https://example.com/video"));
    }
}
