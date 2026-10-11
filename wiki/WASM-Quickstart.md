# WASM quickstart

I use WASM when a plugin can hand catalog data and stream locations to Chilpa
through the host's limited ABI. Chilpa performs audio decoding and output.
The minimal `examples/hello-source` crate in this repository is a
catalog-only source: it intentionally declares `delivers_audio: false`.
The `fixture` crate shows browsing, pagination, search, and resolution with
recorded responses.

## Start from the example

I copy `examples/hello-source` into its own plugin repository and change
the manifest ID, name, description, and crate name. The ID is a stable,
lowercase reverse domain with at least one dot. The package contains
`manifest.json` and `plugin.wasm` under a directory named exactly after
that ID.

The example's SDK dependency is a workspace-relative path for local use.
In a separate repository I replace it with a reviewed immutable SDK pin,
using the public `chilpa-plugins` URL:

```toml
chilpa-plugin-sdk = { git = "https://github.com/vicrodh/chilpa-plugins", rev = "<reviewed commit>" }
```

I can use an immutable tag instead of `rev`. I do not track the moving
`main` branch in a released plugin. The host's `chilpa-plugin-api`
dependency and the plugin must use compatible revisions.

## Export operations

Each operation takes one message from `chilpa_plugin_sdk::api` and returns
`Result<Reply, String>`. I declare its export with `plugin!`:

```rust
plugin! {
    api_version: 1,
    chilpa_search => search,
    chilpa_browse => browse,
}
```

The manifest advertises what the host may ask. A source with audio also
exports `chilpa_resolver` and returns a `Stream` for its own stable
`external_id`. I keep the same identity through search, browse, queue, and
resolve. I set `delivers_audio: false` or `playable: false` when I cannot
actually supply supported audio. A segmented HLS response is not currently
playable through the WASM `Stream` API.

The SDK's `host` module supplies mediated `http`, `get`, `now`, `log`,
`store_get`, and `store_set`. I declare `network` for every HTTP domain
and `storage` before using the store. Each WASM call gets a fresh instance:
values in Rust statics do not persist between calls. `cache::get` also needs
`storage`; it saves a bounded directory cache and request budget in that
store. I do not use it for account or audio responses.

## Test and package

I test operation logic natively. `chilpa_plugin_sdk::testing` records HTTP
answers, clock values, store values, and requested URLs without real
network access. I test at least the ID path from listing or search to
resolution, empty or malformed service responses, pagination termination,
and unavailable streams when those paths exist.

```sh
cargo test --offline --manifest-path examples/hello-source/Cargo.toml
rustup target add wasm32-unknown-unknown
tools/build-plugin examples/hello-source <out-dir>
tools/install-dev <out-dir>/app.example.hello <test-XDG_DATA_HOME>
```

I run these from the SDK repository for the unmodified example. In a
separate plugin repository I use that repository's own `tools/build-plugin`
or call the SDK repository's script with the plugin crate path. The build
script uses the release profile and writes `plugin.wasm` and a copied
`manifest.json`. The install script copies into a development data home;
I use an isolated home for host smoke tests.

Native `cargo test` does not validate the WASM ABI. I also compile the
WASM target, load the package in Chilpa, and exercise its declared operations.
I read [Compatibility and testing](Compatibility-and-Testing.md) before
calling the package ready.
