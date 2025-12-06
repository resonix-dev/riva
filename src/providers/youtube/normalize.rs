use url::Url;

use super::extractor::RivaError;

pub fn normalize_video_id(input: &str) -> Result<String, RivaError> {
    if looks_like_video_id(input) {
        return Ok(input.to_string());
    }

    let normalized = if input.starts_with("http://") || input.starts_with("https://") {
        input.to_string()
    } else {
        format!("https://{input}")
    };

    let url = Url::parse(&normalized).map_err(|_| RivaError::InvalidUrl)?;
    let host = url.host_str().unwrap_or_default();

    if host == "youtu.be" {
        let path = url.path().trim_matches('/');
        if looks_like_video_id(path) {
            return Ok(path.to_string());
        }
        return Err(RivaError::InvalidUrl);
    }

    if host.contains("youtube.") {
        if let Some(id) = url
            .query_pairs()
            .find(|(k, _)| k == "v")
            .map(|(_, v)| v.into_owned())
            .filter(|id| looks_like_video_id(id))
        {
            return Ok(id);
        }

        if let Some(segments) = url.path_segments() {
            let segments: Vec<_> = segments.collect();
            if segments.len() >= 2 {
                match segments[0] {
                    "shorts" | "embed" | "live" | "v" => {
                        if looks_like_video_id(segments[1]) {
                            return Ok(segments[1].to_string());
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Err(RivaError::InvalidUrl)
}

fn looks_like_video_id(value: &str) -> bool {
    (value.len() == 11 || value.len() == 12)
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}
