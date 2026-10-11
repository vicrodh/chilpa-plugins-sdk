//! What a Chilpa plugin written in Rust needs so it does not write the ABI
//! by hand.
//!
//! A plugin is a WebAssembly module. Chilpa writes a JSON message into its
//! memory, calls one exported function per operation, and reads a JSON
//! reply back. The plugin has no network, disk or clock of its own: it asks
//! the host, through the functions in [`host`].
//!
//! A plugin is three things:
//!
//! ```ignore
//! use chilpa_plugin_sdk::{api, host, plugin};
//!
//! fn browse(ask: api::Browse) -> Result<api::Listing, String> {
//!     let reply = host::get("https://example.org/stations")?;
//!     // ... turn `reply.body` into a listing ...
//!     Ok(api::Listing::default())
//! }
//!
//! plugin! {
//!     api_version: 1,
//!     chilpa_browse => browse,
//! }
//! ```
//!
//! **A plugin remembers nothing between calls.** Every call runs in a fresh
//! instance; what has to last goes to [`host::store_set`].
//!
//! Built for anything but `wasm32`, the host functions are served by
//! [`testing`], so the logic of a plugin is tested natively against
//! recorded replies, without a sandbox.

pub use chilpa_plugin_api as api;
pub use serde;
pub use serde_json;

pub mod host;
#[cfg(not(target_arch = "wasm32"))]
pub mod testing;

use serde::{de::DeserializeOwned, Serialize};

/// Hands the host a piece of this module's memory.
///
/// The memory is never given back. A call lives in its own instance and the
/// whole instance is dropped when it ends, so freeing would be work with no
/// effect.
#[doc(hidden)]
pub fn alloc(len: i32) -> i32 {
    let mut buffer: Vec<u8> = Vec::with_capacity(len.max(0) as usize);
    let ptr = buffer.as_mut_ptr();
    std::mem::forget(buffer);
    ptr as i32
}

/// Runs one operation: reads the message, calls the plugin's function and
/// packs its reply the way the host reads it.
///
/// An `Err` becomes `{"error": "..."}`, which is how a plugin says "this
/// went wrong" without it counting as the plugin being broken.
#[doc(hidden)]
pub fn run<I, O>(ptr: i32, len: i32, operation: fn(I) -> Result<O, String>) -> i64
where
    I: DeserializeOwned,
    O: Serialize,
{
    // SAFETY: the host wrote `len` bytes at the address `alloc` gave it.
    let input = unsafe { std::slice::from_raw_parts(ptr as usize as *const u8, len.max(0) as usize) };
    let reply = match serde_json::from_slice::<I>(input) {
        Ok(message) => match operation(message) {
            Ok(output) => serde_json::to_vec(&output).unwrap_or_else(|e| error(&e.to_string())),
            Err(reason) => error(&reason),
        },
        Err(e) => error(&format!("the message could not be read: {e}")),
    };
    let packed = ((reply.as_ptr() as usize as u64) << 32) | reply.len() as u64;
    std::mem::forget(reply);
    packed as i64
}

fn error(reason: &str) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({ "error": reason })).unwrap_or_default()
}

/// Declares what the module exports: the two functions every plugin has,
/// and one per operation.
///
/// Each operation is a function from its message to `Result<reply,
/// String>`; the name on the left is the one in [`api::op`].
#[macro_export]
macro_rules! plugin {
    (api_version: $version:expr, $($operation:ident => $function:path),+ $(,)?) => {
        #[no_mangle]
        pub extern "C" fn chilpa_api_version() -> i32 {
            $version
        }

        #[no_mangle]
        pub extern "C" fn chilpa_alloc(len: i32) -> i32 {
            $crate::alloc(len)
        }

        $(
            #[no_mangle]
            pub extern "C" fn $operation(ptr: i32, len: i32) -> i64 {
                $crate::run(ptr, len, $function)
            }
        )+
    };
}
