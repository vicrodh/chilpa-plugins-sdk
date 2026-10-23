# WASM contract

Use `chilpa-plugin-sdk`. The `plugin!` macro exports `chilpa_api_version`,
`chilpa_alloc` and each declared operation. Each operation's function takes
one `chilpa_plugin_sdk::api` message and returns `Result<Reply, String>`. The
exports include `chilpa_search`, `chilpa_browse`, `chilpa_resolver`,
`chilpa_playlists`, `chilpa_playlist_tracks` and `chilpa_revoke`. The
manifest lists no operations: the host calls them by kind and treats a
missing export as unsupported, so export only what you implement.

The public [Radio plugin](https://github.com/vicrodh/chilpa-pg-radio) is a
working source with no account. Its `manifest.json`, `src/` and recorded
tests show directory requests through the host, stable station IDs, mirror
fallback, stream resolution and unavailable HLS entries. Its network domain
and station parsing belong to that service; derive a new plugin's
permissions and data mapping from its own code.

Every call gets a fresh instance. Use `host::store_get` and
`host::store_set` only with `storage` declared, and never for a secret: the
store is plain files. Use `host::http` or `host::get` only for hosts declared
under `network`; their subdomains are allowed too. The host drops any
`Authorization` header the plugin sets and fails replies over 8 MiB or
20 seconds. A transport failure returns `Err`; an HTTP 404 or 429 is still a
`Response`. The directory cache, `cache::get`, lives in the store and is for
bounded directory reads, never credentials or audio. A service account uses
the manifest's `auth`, and the host adds its credential to every request
(`wiki/Credentials.md`).

Keep the same stable external ID through search, browse and resolve. A
resolver returns a `Stream` the target host can deliver. The current WASM
API cannot play HLS through `Stream`, so mark those entries unavailable.
With no audio at all, set `delivers_audio: false` in the manifest and never
invent a URL. Each reply is capped at 500 entities of each kind, 24 facts per
entry and four root columns. Pagination must end. Time, fuel and memory
limits are in `wiki/WASM-Quickstart.md`.

In a standalone plugin, replace the local SDK path with a reviewed Git `rev`
or tag. The host and the existing WASM plugins pin `protocol-4`, which is a
Git label; `manifest.api_version` is 1. Test the logic natively with
`chilpa_plugin_sdk::testing`, which does not enforce the store or domain
rules, build `wasm32-unknown-unknown` in release mode, package
`manifest.json` and `plugin.wasm` under the manifest ID, and load the
package in Chilpa.
