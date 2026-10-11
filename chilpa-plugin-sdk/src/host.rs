//! Everything a plugin can ask of the world: a log, a clock, a small store
//! and HTTP to the domains its manifest declared.
//!
//! There is nothing else. A plugin never holds a credential: when its
//! source has an account, Chilpa adds the authorization to the request on
//! its side.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// How much room the first attempt gives a reply. A reply that does not fit
/// costs a second request, so the first buffer is generous: memory that is
/// never touched costs nothing.
#[cfg(target_arch = "wasm32")]
const FIRST_ROOM: usize = 4 * 1024 * 1024;

/// How important a log line is.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Level {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

/// A request the host makes on the plugin's behalf.
#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize)]
pub struct Request {
    pub url: String,
    /// `GET` when empty.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub method: String,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub headers: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub body: String,
}

impl Request {
    pub fn get(url: &str) -> Self {
        Self { url: url.to_string(), ..Self::default() }
    }

    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.insert(name.to_string(), value.to_string());
        self
    }
}

/// What came back. A status that is not 200 is still a reply: the plugin
/// decides what a 404 means.
#[derive(Clone, PartialEq, Eq, Debug, Default, Deserialize)]
pub struct Response {
    #[serde(default)]
    pub status: u16,
    /// With their names in lowercase.
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(default)]
    pub body: String,
}

impl Response {
    pub fn ok(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// The body as JSON, or why not: a status that is not 2xx, or a body
    /// that is not what was expected.
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, String> {
        if !self.ok() {
            return Err(format!("the server answered {}", self.status));
        }
        serde_json::from_str(&self.body).map_err(|e| format!("the reply was not understood: {e}"))
    }
}

/// Makes a request. `Err` is the request not happening at all: no network,
/// a domain the manifest did not declare, a reply too large.
pub fn http(request: &Request) -> Result<Response, String> {
    let message = serde_json::to_vec(request).map_err(|e| e.to_string())?;
    let raw = raw::http(&message)?;
    let value: serde_json::Value =
        serde_json::from_slice(&raw).map_err(|e| format!("the host answered something odd: {e}"))?;
    if let Some(reason) = value.get("error") {
        return Err(reason.as_str().map(str::to_string).unwrap_or_else(|| reason.to_string()));
    }
    serde_json::from_value(value).map_err(|e| format!("the host answered something odd: {e}"))
}

/// A plain GET.
pub fn get(url: &str) -> Result<Response, String> {
    http(&Request::get(url))
}

/// A line to Chilpa's log, with the plugin's id in front.
pub fn log(level: Level, message: &str) {
    raw::log(level as i32, message);
}

/// The time, in unix seconds.
pub fn now() -> i64 {
    raw::now()
}

/// Reads from the plugin's own store. It needs the `storage` capability.
pub fn store_get(key: &str) -> Option<Vec<u8>> {
    raw::store_get(key)
}

/// Writes to the plugin's own store. False if it could not be done. The
/// store is for settings and markers: a value over a megabyte is refused.
pub fn store_set(key: &str, value: &[u8]) -> bool {
    raw::store_set(key, value)
}

/// Escapes a value for the path or the query of a URL.
pub fn encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(byte as char),
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

#[cfg(target_arch = "wasm32")]
mod raw {
    use super::FIRST_ROOM;

    #[link(wasm_import_module = "chilpa")]
    extern "C" {
        #[link_name = "log"]
        fn host_log(level: i32, ptr: i32, len: i32);
        #[link_name = "now"]
        fn host_now() -> i64;
        #[link_name = "store_get"]
        fn host_store_get(kptr: i32, klen: i32, out: i32, room: i32) -> i32;
        #[link_name = "store_set"]
        fn host_store_set(kptr: i32, klen: i32, vptr: i32, vlen: i32) -> i32;
        #[link_name = "http"]
        fn host_http(rptr: i32, rlen: i32, out: i32, room: i32) -> i32;
    }

    pub fn log(level: i32, message: &str) {
        unsafe { host_log(level, message.as_ptr() as i32, message.len() as i32) }
    }

    pub fn now() -> i64 {
        unsafe { host_now() }
    }

    /// Calls a host function that fills a buffer. The host answers how many
    /// bytes it wrote, or minus how many it needed, having written nothing.
    fn filled(room: usize, call: impl Fn(i32, i32) -> i32) -> Option<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::with_capacity(room);
        let code = call(buffer.as_mut_ptr() as i32, room as i32);
        if code >= 0 {
            // SAFETY: the host wrote `code` bytes, and `code` fits in `room`.
            unsafe { buffer.set_len(code as usize) };
            return Some(buffer);
        }
        let needed = code.unsigned_abs() as usize;
        // Minus one is also "there is nothing"; asking again with one byte
        // of room tells the two apart at the price of one short call.
        if needed <= room {
            return None;
        }
        let mut buffer: Vec<u8> = Vec::with_capacity(needed);
        let code = call(buffer.as_mut_ptr() as i32, needed as i32);
        if code < 0 {
            return None;
        }
        unsafe { buffer.set_len(code as usize) };
        Some(buffer)
    }

    pub fn store_get(key: &str) -> Option<Vec<u8>> {
        filled(64 * 1024, |out, room| unsafe {
            host_store_get(key.as_ptr() as i32, key.len() as i32, out, room)
        })
    }

    pub fn store_set(key: &str, value: &[u8]) -> bool {
        unsafe {
            host_store_set(
                key.as_ptr() as i32,
                key.len() as i32,
                value.as_ptr() as i32,
                value.len() as i32,
            ) == 1
        }
    }

    pub fn http(request: &[u8]) -> Result<Vec<u8>, String> {
        filled(FIRST_ROOM, |out, room| unsafe {
            host_http(request.as_ptr() as i32, request.len() as i32, out, room)
        })
        .ok_or_else(|| "the host did not answer the request".to_string())
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod raw {
    use crate::testing;

    pub fn log(level: i32, message: &str) {
        testing::with(|world| world.logs.push((level, message.to_string())));
    }

    pub fn now() -> i64 {
        testing::with(|world| world.clock)
    }

    pub fn store_get(key: &str) -> Option<Vec<u8>> {
        testing::with(|world| world.store.get(key).cloned())
    }

    pub fn store_set(key: &str, value: &[u8]) -> bool {
        testing::with(|world| world.store.insert(key.to_string(), value.to_vec()));
        true
    }

    pub fn http(request: &[u8]) -> Result<Vec<u8>, String> {
        testing::serve(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing;

    #[test]
    fn a_value_is_escaped_for_a_url() {
        assert_eq!(encode("drum & bass"), "drum%20%26%20bass");
        assert_eq!(encode("año/1"), "a%C3%B1o%2F1");
        assert_eq!(encode("plain-text_1.0~"), "plain-text_1.0~");
    }

    #[test]
    fn a_request_is_answered_by_what_the_test_recorded() {
        testing::reset();
        testing::answer("https://example.org/a", 200, r#"{"n":1}"#);
        let reply = get("https://example.org/a").unwrap();
        assert!(reply.ok());
        assert_eq!(reply.json::<serde_json::Value>().unwrap()["n"], 1);
        assert_eq!(testing::asked(), ["https://example.org/a"]);
    }

    #[test]
    fn a_url_nobody_recorded_fails_the_way_no_network_does() {
        testing::reset();
        assert!(get("https://example.org/missing").is_err());
    }

    #[test]
    fn a_status_that_is_not_2xx_is_a_reply_and_its_json_says_so() {
        testing::reset();
        testing::answer("https://example.org/gone", 404, "nope");
        let reply = get("https://example.org/gone").unwrap();
        assert!(!reply.ok());
        assert!(reply.json::<serde_json::Value>().unwrap_err().contains("404"));
    }

    #[test]
    fn the_store_round_trips() {
        testing::reset();
        assert_eq!(store_get("k"), None);
        assert!(store_set("k", b"v"));
        assert_eq!(store_get("k").as_deref(), Some(&b"v"[..]));
    }
}
