use anyhow::{Result, anyhow};
use reqwest::header::{ACCEPT, HeaderMap, HeaderValue};

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

        let resp = client.get(&url).headers(headers.clone()).send().await?;
        let status = resp.status();
        if !status.is_success() {
            return Err(anyhow!("Failed to fetch events: {}", status));
        }
        let json: serde_json::Value = resp.json().await?;
        if let Some(events) = json.get("asset_events").and_then(|v| v.as_array()) {
            all_events.extend(events.clone());
        }
        if let Some(cursor) = json.get("next").and_then(|v| v.as_str()) {
            if !cursor.is_empty() {
                next = Some(cursor.to_string());
                continue;
            }
        }
        break;
    }
    Ok(serde_json::to_string(&all_events).unwrap())
}
