use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct StreamInfo {
    pub url: String,
    pub protocol: String,
    pub mime_type: String,
    pub quality: Option<String>,
    pub preset: Option<String>,
    pub duration_ms: Option<u64>,
    pub snipped: bool,
    pub artwork_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Track {
    pub kind: Option<String>,
    pub media: Option<MediaCollection>,
    pub track_authorization: Option<String>,
    #[serde(default)]
    pub artwork_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MediaCollection {
    #[serde(default)]
    pub transcodings: Vec<Transcoding>,
}

#[derive(Debug, Deserialize)]
pub struct Transcoding {
    pub url: String,
    pub preset: Option<String>,
    pub quality: Option<String>,
    pub snipped: Option<bool>,
    pub format: Option<TranscodingFormat>,
    pub duration: Option<u64>,
}

impl Transcoding {
    pub fn into_stream_info(
        self,
        resolved_url: String,
        artwork_url: Option<&str>,
    ) -> Option<StreamInfo> {
        let format = self.format?;
        Some(StreamInfo {
            url: resolved_url,
            protocol: format.protocol,
            mime_type: format.mime_type,
            quality: self.quality,
            preset: self.preset,
            duration_ms: self.duration,
            snipped: self.snipped.unwrap_or(false),
            artwork_url: artwork_url.map(|value| value.to_string()),
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct TranscodingFormat {
    pub protocol: String,
    pub mime_type: String,
}

#[derive(Debug, Deserialize)]
pub struct TranscodingLocation {
    pub url: String,
}
