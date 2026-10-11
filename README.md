# chilpa-plugins, the `sdk` branch

What a plugin needs to be built without Chilpa's own source: the types of the
protocol and the helpers that hide the ABI. The list of plugins is on `main`.

## Crates

- `chilpa-plugin-api`: the messages Chilpa and a plugin exchange. It depends
  on `serde` and nothing else. Chilpa itself builds against a pinned revision
  of this branch, so the host and a plugin share one definition.
- `chilpa-plugin-sdk`: what a plugin written in Rust needs so it does not
  write the ABI by hand: the `plugin!` macro that declares the exports, the
  host calls (`http`, `store_get`, `store_set`, `now`, `log`), and a fake
  host for testing a plugin's logic natively.
- `fixture`: a plugin with fixed answers, used to test a host against a
  module built from Rust.

## Building a plugin

```
rustup target add wasm32-unknown-unknown      # once per machine
tools/build-plugin <crate dir> <out dir>      # leaves <out>/<id>/plugin.wasm and manifest.json
tools/install-dev <out>/<id> <XDG_DATA_HOME>  # copies it where a Chilpa run with that data home looks
```

The release profile matters: without `strip` a module of 170 KB weighs over
a megabyte.

## Using it

```toml
chilpa-plugin-sdk = { git = "<this repository>", rev = "<a commit of sdk>" }
```

Pin a revision, not the branch: a plugin and the Chilpa it runs in agree on
the protocol through the API version in the manifest, and a moving branch
breaks that quietly.
