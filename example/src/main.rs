use riva::soundcloud::{extract_streams, StreamInfo};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let current_time = std::time::Instant::now();
    let track_url = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "https://soundcloud.com/kordhell/trageluxe".to_string());
    let streams: Vec<StreamInfo> = extract_streams(&track_url).await?;

    for (idx, stream) in streams.iter().enumerate() {
        println!("Stream {}", idx + 1);
        print_field("url", &stream.url);
        print_field("protocol", &stream.protocol);
        print_field("mime_type", &stream.mime_type);
        print_field("quality", &fmt_opt(stream.quality.as_deref()));
        print_field("preset", &fmt_opt(stream.preset.as_deref()));
        print_field("duration_ms", &fmt_opt(stream.duration_ms));
        print_field("snipped", &stream.snipped.to_string());
        println!();
    }

    let elapsed = current_time.elapsed();
    println!("Extracted {} streams in {:.2?}", streams.len(), elapsed);

    Ok(())
}

fn print_field(label: &str, value: &str) {
    println!("  {:<18}: {}", label, value);
}

fn fmt_opt<T: ToString>(value: Option<T>) -> String {
    value.map(|v| v.to_string()).unwrap_or_else(|| "-".into())
}