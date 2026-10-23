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

The manifest does not list operations: the host calls them by kind (every
`source` is asked for `chilpa_search` and `chilpa_playlists`, for example)
and treats a missing export as unsupported, so export only what you
implement. A source with audio also exports `chilpa_resolver` and returns a `Stream` for its own stable
`external_id`. Keep that ID the same through search, browse, queue and
resolve. If the plugin cannot supply audio Chilpa can play, set
`delivers_audio: false` or `playable: false`. A segmented HLS response can't
be played through the WASM `Stream` API yet.

The SDK's `host` module provides `http`, `get`, `now`, `log`, `store_get`
and `store_set`, all through the host. Declare `network` for every HTTP
domain and `storage` before touching the store. Every WASM call gets a fresh
instance, so Rust statics do not survive between calls. `cache::get` keeps a
bounded directory cache and request budget in the store, so it also needs
`storage`. Don't use it for account or audio responses.

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

Native `cargo test` does not check the WASM ABI. Also build the WASM target,
load the package in Chilpa and run every operation it declares. Then read
[Compatibility and testing](Compatibility-and-Testing.md) before calling it
ready.
