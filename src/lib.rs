pub mod providers;

#[cfg(feature = "youtube")]
pub mod youtube {
    pub use crate::providers::youtube::{
        RivaError, StreamInfo, extract_streams, normalize_video_id,
    };
}

#[cfg(feature = "soundcloud")]
pub mod soundcloud {
    pub use crate::providers::soundcloud::{
        RivaError, StreamInfo, extract_streams, normalize_track_url,
    };
}
