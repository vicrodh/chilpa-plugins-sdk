# WASM quickstart

Use WASM when the plugin only has to hand Chilpa catalog data and stream
locations through the host's limited ABI. Chilpa does the decoding and the
output. `examples/hello-source` is a minimal catalog-only source, with
`delivers_audio: false` on purpose. The `fixture` crate shows browsing,
pagination, search and resolution against recorded replies.

For a complete source with no account, read the
[Radio plugin](https://github.com/vicrodh/chilpa-pg-radio) after the example.
Its manifest declares the network host and storage the code uses, and its
recorded tests cover browsing, search, resolution and falling back to another
mirror. HLS-only stations are marked unavailable under the current WASM
playback contract. Its directory URLs and station fields belong to that
service, not to the SDK API.

## Start from the example

Copy `examples/hello-source` into its own repository and change the manifest
ID, name and description, and the crate name. The ID is a stable lowercase
reverse domain with at least one dot. The package is `manifest.json` and
`plugin.wasm` inside a directory named exactly after that ID.

The example points at the SDK by a path inside this workspace. In your own
repository, pin a reviewed commit of the public SDK instead:

```toml
chilpa-plugin-sdk = { git = "https://github.com/vicrodh/chilpa-plugins-sdk", rev = "<reviewed commit>" }
```

A tag works too. Never track `main` in a released plugin. The host's
`chilpa-plugin-api` revision and the plugin's must be compatible.

## Export operations

Each operation takes one message from `chilpa_plugin_sdk::api` and returns
`Result<Reply, String>`. Declare the exports with `plugin!`:

```rust
plugin! {
    api_version: 1,
    chilpa_search => search,
    chilpa_browse => browse,
}
```

The manifest does not list operations. The host calls them by kind and
treats a missing export as unsupported, so export only what you implement.
For a `source` it calls `chilpa_search` on every global search,
`chilpa_browse` when the listener opens the source, `chilpa_playlists` at
most every five minutes (a plugin without that export is not asked again
until it reloads), `chilpa_playlist_tracks` when a list opens, and
`chilpa_resolver` to play. Uninstalling from Settings first sends
`chilpa_revoke`.

A source with audio exports `chilpa_resolver` and returns a `Stream` for its
own stable `external_id`. Keep that ID the same through search, browse,
queue and resolve. If the plugin cannot supply audio Chilpa can play, set
`delivers_audio: false` or `playable: false`. A segmented HLS response can't
be played through the WASM `Stream` API yet.

## Host calls

The SDK's `host` module provides `http`, `get`, `now`, `log`, `store_get`
and `store_set`, all through the host. Every WASM call gets a fresh
instance, so Rust statics do not survive between calls. Declare `storage`
before touching the store; without it reads find nothing and writes fail.
`cache::get` keeps a bounded directory cache and request budget in the
store, so it also needs `storage`. Don't use it for account or audio
responses.

Declare `network` for every HTTP domain: a request may go to a declared host
or one of its subdomains, over `http` or `https`. The host drops any
`Authorization` header the plugin sets and adds `User-Agent: Chilpa`. A
reply is text of at most 8 MiB that arrives within 20 seconds; anything else,
an undeclared domain or offline mode comes back as `Err`. A status that is
not 2xx is still a `Response`.

A plugin whose service needs an account declares `auth` in its manifest, as
an API token the listener pastes or OAuth 2.0 that Chilpa runs. The plugin
never sees the credential: the host adds it to every request the plugin
makes, so declare only hosts that should receive it.
[Credentials](Credentials.md) has the details.

## Limits

| Calls | Wall clock | Fuel | Memory |
| --- | --- | --- | --- |
| `chilpa_browse`, `chilpa_search`, `chilpa_resolver` before playback | 45 s | 10⁹ | 32 MiB |
| `chilpa_playlists`, `chilpa_playlist_tracks`, `chilpa_revoke`, `chilpa_dsp_presets`, resolving a search result | 2 s | 5×10⁷ | 8 MiB |
| Any other call | 5 s | 10⁹ | 32 MiB |

The wall clock includes the time the host spends on the plugin's HTTP
requests. An `Err` from an operation, or a missing export, is an answer.
Breaking a limit, trapping or not answering is a failure. Three failures in
a row within a minute put the plugin in quarantine until the listener
restarts it; a successful call clears the count.

## Test and package

Test the operation logic natively. `chilpa_plugin_sdk::testing` records HTTP
answers, clock values, store values and requested URLs with no real network.
Cover at least the ID's path from a listing or search to resolution, empty or
malformed replies, pagination that ends, and unavailable streams when the
plugin has them.

```sh
cargo test --offline --manifest-path examples/hello-source/Cargo.toml
rustup target add wasm32-unknown-unknown
tools/build-plugin examples/hello-source <out-dir>
tools/install-dev <out-dir>/app.example.hello <test-XDG_DATA_HOME>
```

Those commands run the unmodified example from this repository. In your own
repository, use its copy of `tools/build-plugin`, or call this repository's
script with your crate's path. The build uses the release profile and writes
`plugin.wasm` plus a copy of `manifest.json`. `install-dev` copies the result
into a development data home; use an isolated one for host tests.

Native `cargo test` does not check the WASM ABI, and the fake host does not
apply the real one's rules: `store_set` always succeeds, with no size or key
limits, and a recorded URL is served whatever the manifest declares. Also
build the WASM target, load the package in Chilpa and run every operation it
declares. Then read [Compatibility and testing](Compatibility-and-Testing.md)
before calling it ready.
