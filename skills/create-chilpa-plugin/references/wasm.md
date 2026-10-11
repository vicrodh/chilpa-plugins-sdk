# WASM contract

I use `chilpa-plugin-sdk` for a WASM plugin. The `plugin!` macro exports
`chilpa_api_version`, `chilpa_alloc`, and each declared operation. The
function named by an export takes one `chilpa_plugin_sdk::api` message and
returns `Result<Reply, String>`. Relevant exports include
`chilpa_search`, `chilpa_browse`, `chilpa_resolver`,
`chilpa_playlists`, `chilpa_playlist_tracks`, and `chilpa_revoke`.
The host asks only for operations that the manifest declares.

The public [Radio plugin](https://github.com/vicrodh/chilpa-pg-radio) is a
working account-free source reference. I inspect its `manifest.json`,
`src/`, and recorded tests for mediated directory requests, stable station
IDs, mirror fallback, stream resolution, and unavailable HLS entries. Its
network domain and station parsing belong to that service; I derive a new
plugin's permissions and data mapping from its own implementation.

Each call receives a fresh instance. I use `host::store_get` and
`host::store_set` only with `storage` declared. I use `host::http` or
`host::get` only for exact domain hosts declared by `network`. A failed
transport returns `Err`; an HTTP 404 or 429 is still a `Response`.
The directory `cache::get` uses the store and is for bounded directory
reads, not credentials or audio.

For a source I keep the same stable external ID in search, browse, and
resolve. A resolver returns a `Stream` whose delivery is supported by the
target host. The current WASM API does not play HLS through `Stream`;
I mark such an entry unavailable. When no audio is available, I set
`delivers_audio: false` in the manifest and do not invent a URL.
The API's cleanup caps each entity kind at 500 per reply, entry facts at
24, and root columns at four. Pagination must terminate.

For a standalone Rust plugin I replace the local SDK path with a reviewed
immutable Git `rev` or tag. `protocol-4` is the current pin used by the
host and existing WASM plugins; it is a Git label, while
`manifest.api_version` is 1. I test logic natively with
`chilpa_plugin_sdk::testing`, build `wasm32-unknown-unknown` in release
mode, package `manifest.json` and `plugin.wasm` under the manifest ID,
and load the package in Chilpa.
