use anyhow::{Result, anyhow};
use reqwest::header::{ACCEPT, HeaderMap, HeaderValue};
use tokio::time::{Duration, sleep};

async fn do_request(
    client: &reqwest::Client,
    url: &str,
    headers: &HeaderMap,
) -> Result<Option<(Vec<serde_json::Value>, Option<String>)>> {
    let mut retries = 0;
    let resp = loop {
        let resp = client.get(url).headers(headers.clone()).send().await;
        match resp {
            Ok(r) if r.status() == reqwest::StatusCode::TOO_MANY_REQUESTS => {
                let sleep_secs = r
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(5);
                if retries >= 5 {
                    return Err(anyhow!(
                        "Failed to fetch events: 429 Too Many Requests after 5 retries"
                    ));
                }
                retries += 1;
                eprintln!(
                    "Received 429 Too Many Requests. Sleeping for {} seconds before retrying (attempt {}/5)...",
                    sleep_secs, retries
                );
                sleep(Duration::from_secs(sleep_secs)).await;
                continue;
            }
            Ok(r) if r.status().is_server_error() => {
                if retries >= 5 {
                    return Err(anyhow!(
                        "Failed to fetch events: {} after 5 retries",
                        r.status()
                    ));
                }
                retries += 1;
                eprintln!(
                    "Received {}. Sleeping for 5 seconds before retrying (attempt {}/5)...",
                    r.status(),
                    retries
                );
                sleep(Duration::from_secs(5)).await;
                continue;
            }
            Ok(r) => break r,
            Err(e) => {
                if retries >= 5 {
                    return Err(anyhow!("Failed to fetch events after 5 retries: {}", e));
                }
                retries += 1;
                eprintln!(
                    "Request error: {}. Sleeping for 5 seconds before retrying (attempt {}/5)...",
                    e, retries
                );
                sleep(Duration::from_secs(5)).await;
                continue;
            }
        }
    };
    let status = resp.status();
    if !status.is_success() {
        return Err(anyhow!("Failed to fetch events: {}", status));
    }
    let json: serde_json::Value = resp.json().await?;
    let events = json
        .get("asset_events")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let next = json
        .get("next")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    Ok(Some((events, next)))
}

/// Fetches all events for a given collection slug from OpenSea, supporting pagination and event_type filtering, and returns raw JSON array as String.
pub async fn get_events(
    collection_slug: &str,
    event_type: Option<&str>,
    api_key: Option<&str>,
) -> Result<String> {
    let mut all_events = Vec::new();
    let mut next: Option<String> = None;
    let client = reqwest::Client::new();

    loop {
        let mut url = format!(
            "https://api.opensea.io/api/v2/events/collection/{}",
            collection_slug
        );
        let mut params = vec![];
        if let Some(event_type) = event_type {
            params.push(("event_type", event_type));
        }
        if let Some(ref cursor) = next {
            params.push(("next", cursor));
        }
        if !params.is_empty() {
            url.push('?');
            url.push_str(&serde_urlencoded::to_string(&params).unwrap());
        }

        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        if let Some(key) = api_key {
            headers.insert("x-api-key", HeaderValue::from_str(key).unwrap());
        }
        let result = do_request(&client, &url, &headers).await?;
        if let Some((events, next_cursor)) = result {
            all_events.extend(events);
            if let Some(cursor) = next_cursor {
                if !cursor.is_empty() {
                    next = Some(cursor);
                    continue;
                }
            }
        }
        break;
    }
    Ok(serde_json::to_string(&all_events).unwrap())
}
