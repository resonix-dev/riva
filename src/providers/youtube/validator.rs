use reqwest::{Client, StatusCode, header};
use tokio::task::JoinSet;

use super::models::StreamInfo;

const MAX_VALIDATION_CONCURRENCY: usize = 8;

pub async fn filter_working_streams(client: &Client, streams: Vec<StreamInfo>) -> Vec<StreamInfo> {
    let mut pending = streams.into_iter().enumerate();
    let mut tasks = JoinSet::new();
    let mut active = 0usize;
    let mut working = Vec::new();

    loop {
        while active < MAX_VALIDATION_CONCURRENCY {
            if let Some((idx, stream)) = pending.next() {
                let client = client.clone();
                tasks.spawn(async move {
                    let accessible = stream_accessible(&client, &stream.url).await;
                    (idx, accessible.then_some(stream))
                });
                active += 1;
            } else {
                break;
            }
        }

        if active == 0 {
            break;
        }

        if let Some(result) = tasks.join_next().await {
            active -= 1;
            if let Ok((idx, Some(stream))) = result {
                working.push((idx, stream));
            }
        }
    }

    working.sort_by_key(|(idx, _)| *idx);
    working.into_iter().map(|(_, stream)| stream).collect()
}

async fn stream_accessible(client: &Client, url: &str) -> bool {
    match client.head(url).send().await {
        Ok(resp) if resp.status().is_success() => return true,
        Ok(resp) if is_fatal(resp.status()) => return false,
        _ => {}
    }

    match client
        .get(url)
        .header(header::RANGE, "bytes=0-1")
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            let _ = resp.bytes().await;
            true
        }
        _ => false,
    }
}

fn is_fatal(status: StatusCode) -> bool {
    status.is_client_error()
}
