use riva::{RivaClient, YoutubeClientType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = RivaClient::from_env()?;
    let now = std::time::Instant::now();

    let response_sc = client
        .soundcloud_stream("https://soundcloud.com/kordhell/trageluxe")
        .await?;

    let bytes_sc = response_sc.bytes().await?;

    println!(
        "sc: downloaded {} bytes in {:?}",
        bytes_sc.len(),
        now.elapsed()
    );

    // Uses the Android client to get a higher quality stream
    // Uses `itag` 18 which is 360p HLS, but you can choose any available itag
    // Our tests showed the most videos are available in 360p, but you can
    // experiment with different itags to find the best quality/availability
    // for your use case

    let response_yt = client
        .youtube_stream(
            "https://www.youtube.com/watch?v=rkaiKn5iGzc",
            18,
            Some(YoutubeClientType::Android),
        )
        .await?;

    let bytes_yt = response_yt.bytes().await?;
    println!(
        "yt: downloaded {} bytes in {:?}",
        bytes_yt.len(),
        now.elapsed()
    );

    Ok(())
}
