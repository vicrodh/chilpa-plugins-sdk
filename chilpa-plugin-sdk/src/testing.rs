//! The world a plugin sees when it is not built for the sandbox.
//!
//! It exists so that the logic of a plugin is tested natively, against
//! replies written or recorded by the test. It is per thread, so tests that
//! run side by side do not see each other's world.

use std::cell::RefCell;
use std::collections::HashMap;

/// What the fake host holds and what it was asked for.
#[derive(Default)]
pub struct World {
    pub logs: Vec<(i32, String)>,
    pub clock: i64,
    pub store: HashMap<String, Vec<u8>>,
    /// What each URL answers: status and body.
    pub replies: HashMap<String, (u16, String)>,
    /// Every URL that was asked for, in order.
    pub asked: Vec<String>,
}

thread_local! {
    static WORLD: RefCell<World> = RefCell::new(World::default());
}

pub fn with<T>(f: impl FnOnce(&mut World) -> T) -> T {
    WORLD.with(|world| f(&mut world.borrow_mut()))
}

/// Starts from nothing: no replies, an empty store, the clock at zero.
pub fn reset() {
    with(|world| *world = World::default());
}

/// Says what a URL answers from now on.
pub fn answer(url: &str, status: u16, body: &str) {
    with(|world| world.replies.insert(url.to_string(), (status, body.to_string())));
}

/// Sets the clock.
pub fn set_clock(unix: i64) {
    with(|world| world.clock = unix);
}

/// The URLs that were asked for, in order.
pub fn asked() -> Vec<String> {
    with(|world| world.asked.clone())
}

pub(crate) fn serve(request: &[u8]) -> Result<Vec<u8>, String> {
    let value: serde_json::Value = serde_json::from_slice(request).map_err(|e| e.to_string())?;
    let url = value.get("url").and_then(|u| u.as_str()).unwrap_or_default().to_string();
    let reply = with(|world| {
        world.asked.push(url.clone());
        world.replies.get(&url).cloned()
    });
    let Some((status, body)) = reply else {
        return Ok(serde_json::to_vec(&serde_json::json!({
            "error": format!("no reply was recorded for {url}")
        }))
        .unwrap_or_default());
    };
    Ok(serde_json::to_vec(&serde_json::json!({ "status": status, "headers": {}, "body": body }))
        .unwrap_or_default())
}
