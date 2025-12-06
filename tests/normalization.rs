#[cfg(feature = "soundcloud")]
use riva::soundcloud::normalize_track_url;
#[cfg(feature = "youtube")]
use riva::youtube::normalize_video_id;

#[cfg(feature = "soundcloud")]
#[test]
fn soundcloud_normalization_accepts_common_inputs() {
    let canonical = "https://soundcloud.com/kordhell/trageluxe";
    let bare = "soundcloud.com/kordhell/trageluxe";
    let short = "https://snd.sc/example";

    assert_eq!(normalize_track_url(canonical).unwrap(), canonical);
    assert_eq!(normalize_track_url(bare).unwrap(), canonical);
    assert_eq!(
        normalize_track_url(short).unwrap(),
        "https://snd.sc/example"
    );
}

#[cfg(feature = "soundcloud")]
#[test]
fn soundcloud_normalization_rejects_unrelated_hosts() {
    for candidate in ["", "https://example.com/track", "soundcloud.dev/track"] {
        assert!(
            normalize_track_url(candidate).is_err(),
            "{candidate} should fail"
        );
    }
}

#[cfg(feature = "youtube")]
#[test]
fn youtube_normalization_accepts_multiple_routes() {
    let raw_id = "dQw4w9WgXcQ";
    assert_eq!(normalize_video_id(raw_id).unwrap(), raw_id);

    let watch = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
    assert_eq!(normalize_video_id(watch).unwrap(), raw_id);

    let short = "https://youtu.be/dQw4w9WgXcQ";
    assert_eq!(normalize_video_id(short).unwrap(), raw_id);

    let shorts = "https://www.youtube.com/shorts/dQw4w9WgXcQ";
    assert_eq!(normalize_video_id(shorts).unwrap(), raw_id);

    let embed = "https://www.youtube.com/embed/dQw4w9WgXcQ";
    assert_eq!(normalize_video_id(embed).unwrap(), raw_id);
}

#[cfg(feature = "youtube")]
#[test]
fn youtube_normalization_rejects_invalid_inputs() {
    for candidate in [
        "",
        "https://www.youtube.com/watch?v=short",
        "https://vimeo.com/123",
        "youtube.com/live/invalid",
    ] {
        assert!(
            normalize_video_id(candidate).is_err(),
            "{candidate} should fail"
        );
    }
}
