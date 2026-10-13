# chilpa-plugins-sdk

What you need to write a Chilpa plugin: the WASM protocol, the Rust SDK, a
test fixture for the host, the contracts for native plugins, and the wiki.
The install catalog is [chilpa-plugins](https://github.com/vicrodh/chilpa-plugins);
each plugin has its own repository: [Radio](https://github.com/vicrodh/chilpa-pg-radio),
[Podcasts](https://github.com/vicrodh/chilpa-pg-podcasts) and
[Playlist Import](https://github.com/vicrodh/chilpa-pg-playlist-import).

## Two runtimes

| Runtime | Interface | Use it when |
| --- | --- | --- |
| WASM | `chilpa-plugin-api` messages and `chilpa-plugin-sdk` exports | The host's HTTP, clock, log and storage calls are enough. |
| Process | JSON-RPC 2.0, one object per line | You need native libraries or the plugin handles its own account. Chilpa runs these only in its Linux sandbox for now. |

Both use a `manifest.json` with `api_version: 1`. A process plugin also
answers `protocol: 1` in `initialize`. The `protocol-4` Git tag is an SDK
revision, not API version 4. The Rust SDK covers WASM only.

## Documentation

Start at [the wiki](wiki/Home.md). Its pages live here so they change in the
same review as the SDK.

- [WASM quickstart](wiki/WASM-Quickstart.md), with a runnable
  [example](examples/hello-source/)
- [Native process plugins](wiki/Native-Plugins.md)
- [Manifest and permissions](wiki/Manifest-and-Permissions.md)
- [Compatibility and testing](wiki/Compatibility-and-Testing.md)

The process contracts go into detail: [catalog pages](docs/native-pages.md),
[playback events](docs/playback-events.md), [plugin cooperation](docs/plugin-cooperation.md)
and [lyrics of a source's own tracks](docs/track-lyrics.md). None of them
adds a WASM export.

After `hello-source`, read [Radio](https://github.com/vicrodh/chilpa-pg-radio):
a real source with no account. It searches and browses a public station
directory, resolves streams, declares only the network and storage it uses,
and tests against recorded replies. It marks HLS-only stations unavailable,
because the WASM stream path cannot play them yet.
[Podcasts](https://github.com/vicrodh/chilpa-pg-podcasts) needs no account
either, but its directory calls are signed with a secret that Chilpa's proxy
holds.

## Workspace

- `chilpa-plugin-api`: the WASM v1 messages. Depends only on `serde`.
- `chilpa-plugin-sdk`: the `plugin!` macro, the host calls, an optional
  directory cache, and a fake host for native tests.
- `fixture`: fixed answers for the host's integration tests.

Pin the SDK to a reviewed commit or tag, never to `main`. The host and the
existing WASM plugins pin `protocol-4`; what came after it is the directory
cache and documentation.

```sh
cargo test --workspace                                       # SDK
cargo test --manifest-path examples/hello-source/Cargo.toml  # example
rustup target add wasm32-unknown-unknown
tools/build-plugin examples/hello-source <out-dir>
tools/install-dev <out-dir>/app.example.hello <XDG_DATA_HOME>
```

`build-plugin` writes `plugin.wasm` and `manifest.json` into a directory
named after the plugin ID, with the release profile (a debug build is much
larger). `install-dev` copies that directory into a development data home.

## Authoring skill

[skills/create-chilpa-plugin](skills/create-chilpa-plugin/SKILL.md) is an
agent skill for writing plugins. It reads the right wiki page, checks the
host contract, and won't call a plugin ready until permissions,
compatibility and integration are checked. It does not install or publish
anything.

Copy the whole `skills/create-chilpa-plugin` directory, `references/`
included, into your agent's skills directory. In Codex, a prompt like this
is enough:

> Use $create-chilpa-plugin to create a Chilpa plugin for <service>.
> Research the service API and Chilpa host support, choose the runtime,
> implement only verified capabilities, and test the package in an isolated
> profile.

## Wiki

Wiki pages are written in `wiki/` and reviewed here.
`python3 tools/sync-wiki <clean-wiki-checkout>` copies them into the wiki
repository and turns `Page.md` links into wiki links (`./Page/`).
