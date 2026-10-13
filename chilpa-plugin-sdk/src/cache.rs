//! Bounded, persistent directory responses and a shared request budget per plugin.
use crate::host::{self, Response};
use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize)]
struct Budget { start: i64, count: u32, until: i64 }
#[derive(Deserialize, Serialize)]
struct Saved { url: String, at: i64, body: String }
const SLOTS: usize = 32;
#[derive(Default, Deserialize, Serialize)]
struct Index { urls: Vec<String>, next: usize }

fn read<T: serde::de::DeserializeOwned>(key: &str) -> Option<T> {
    host::store_get(key).and_then(|bytes| serde_json::from_slice(&bytes).ok())
}
fn write(key: &str, value: &impl Serialize) {
    if let Ok(bytes) = serde_json::to_vec(value) { host::store_set(key, &bytes); }
}
fn slot(url: &str) -> (String, Index) {
    let mut index: Index = read("directory-index").unwrap_or_default();
    index.urls.truncate(SLOTS);
    let position = if let Some(position) = index.urls.iter().position(|saved| saved == url) {
        position
    } else if index.urls.len() < SLOTS {
        index.urls.push(url.into());
        index.urls.len() - 1
    } else {
        let position = index.next % SLOTS;
        index.urls[position] = url.into();
        index.next = (position + 1) % SLOTS;
        position
    };
    (format!("directory-cache-{position}"), index)
}
fn reply(saved: &Saved) -> Response {
    Response { status: 200, body: saved.body.clone(), ..Response::default() }
}

/// Reuses listings across WASM invocations; a failure never triggers an immediate retry.
/// At most 20 uncached reads a minute leave room for other application services.
pub fn get(url: &str, ttl: i64) -> Result<Response, String> {
    let now = host::now();
    let (key, index) = slot(url);
    let saved: Option<Saved> = read::<Saved>(&key).filter(|v| v.url == url && v.at <= now && now - v.at < 86_400);
    if let Some(value) = saved.as_ref().filter(|v| now - v.at < ttl) { return Ok(reply(value)); }
    let mut budget: Budget = read("directory-budget").unwrap_or_default();
    if now < budget.start || now - budget.start >= 60 { budget.start = now; budget.count = 0; }
    let until = budget.until.max(if budget.count >= 20 { budget.start + 60 } else { 0 });
    if until > now {
        return saved.as_ref().map(reply).ok_or_else(|| format!("The directory is busy (429). Try again in {} seconds.", until - now));
    }
    budget.count += 1;
    write("directory-budget", &budget);
    let result = host::get(url);
    if let Ok(response) = &result {
        if response.ok() {
            // The isolated store accepts values up to one MiB, including JSON escaping.
            let value = Saved { url: url.into(), at: now, body: response.body.clone() };
            if let Ok(bytes) = serde_json::to_vec(&value) {
                if bytes.len() < 1_000_000 && host::store_set(&key, &bytes) { write("directory-index", &index); }
            }
            return result;
        }
        if response.status == 429 || response.status == 503 {
            let wait = response.headers.get("retry-after").and_then(|v| v.parse::<i64>().ok()).unwrap_or(60).max(1);
            budget.until = now.saturating_add(wait);
            write("directory-budget", &budget);
        }
    }
    if let Some(value) = saved { return Ok(reply(&value)); }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing;
    #[test]
    fn a_listing_is_reused_until_expiry_and_survives_a_network_failure() {
        testing::reset(); testing::answer("https://example.org/list", 200, "[]");
        get("https://example.org/list", 300).unwrap();
        get("https://example.org/list", 300).unwrap();
        assert_eq!(testing::asked().len(), 1);
        testing::set_clock(301); testing::answer("https://example.org/list", 503, "busy");
        assert_eq!(get("https://example.org/list", 300).unwrap().body, "[]");
        assert!(get("https://example.org/other", 300).unwrap_err().contains("429"));
        assert_eq!(testing::asked().len(), 2);
    }
    #[test]
    fn distinct_listings_keep_their_own_cache_slots() {
        testing::reset();
        for number in 0..20 {
            let url = format!("https://example.org/list/{number}");
            testing::answer(&url, 200, &format!("[{number}]"));
            get(&url, 300).unwrap();
        }
        for number in 0..20 {
            let url = format!("https://example.org/list/{number}");
            assert_eq!(get(&url, 300).unwrap().body, format!("[{number}]"));
        }
        assert_eq!(testing::asked().len(), 20);
    }
    #[test]
    fn throttling_blocks_other_queries_and_recovers_after_the_wait() {
        testing::reset(); testing::answer("https://example.org/a", 429, "busy");
        assert_eq!(get("https://example.org/a", 300).unwrap().status, 429);
        assert!(get("https://example.org/b", 300).is_err());
        assert_eq!(testing::asked().len(), 1);
        testing::set_clock(61); testing::answer("https://example.org/b", 200, "[]");
        assert!(get("https://example.org/b", 300).unwrap().ok());
    }
}
