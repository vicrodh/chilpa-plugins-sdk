# Chilpa plugin development

Start here to write a Chilpa plugin. These pages are versioned with the SDK,
under `wiki/`. The runtime and manifest guidance was last checked against
Chilpa host `1cce7299` and SDK `protocol-4` on 2026-10-11. Optional process
contracts need their own check against the host build you target.

| You need | Read |
| --- | --- |
| A small, sandboxed source | [WASM quickstart](WASM-Quickstart.md) |
| Native libraries, a process, or an account the plugin manages | [Native plugins](Native-Plugins.md) |
| A valid package and a fair permission request | [Manifest and permissions](Manifest-and-Permissions.md) |
| An account: where its credential lives | [Credentials](Credentials.md) |
| A package and its catalog entry | [Packages and catalog](Packages-and-Catalog.md) |
| Version pins, tests and release checks | [Compatibility and testing](Compatibility-and-Testing.md) |

## Two interfaces

**WASM** uses the Rust `chilpa-plugin-api` message types and
`chilpa-plugin-sdk`. A module exports operations such as `chilpa_search`,
`chilpa_browse` and `chilpa_resolver`. It has no network, disk or clock of
its own; the host makes those calls for it.

**Process** speaks JSON-RPC 2.0 over standard input and output, one object
per line. The host calls `initialize`, then methods such as `search`,
`page.get` or `track.resolve`. Some methods go to every process plugin
whatever capabilities it advertised, so answer JSON-RPC error `-32601` at
once for any method you do not implement. Process plugins run in a platform
sandbox, Bubblewrap on Linux and Seatbelt on macOS; Windows does not host
them yet. The Rust SDK does not implement this interface.

Both are validated against `manifest.json`, with `api_version: 1`. A process
plugin also answers `protocol: 1` in `initialize`. The `protocol-4` Git tag
is a source revision, neither of those numbers.

## Where the contracts are

- WASM data shapes: [`chilpa-plugin-api/src/lib.rs`](https://github.com/vicrodh/chilpa-plugins-sdk/blob/main/chilpa-plugin-api/src/lib.rs).
- WASM exports and host calls: [`chilpa-plugin-sdk/src/`](https://github.com/vicrodh/chilpa-plugins-sdk/tree/main/chilpa-plugin-sdk/src).
- Test module: [`fixture/`](https://github.com/vicrodh/chilpa-plugins-sdk/tree/main/fixture).
- Process methods: [native source](https://github.com/vicrodh/chilpa-plugins-sdk/blob/main/docs/native-source.md),
  [native pages](https://github.com/vicrodh/chilpa-plugins-sdk/blob/main/docs/native-pages.md),
  [settings and downloads](https://github.com/vicrodh/chilpa-plugins-sdk/blob/main/docs/native-settings.md),
  [playback events](https://github.com/vicrodh/chilpa-plugins-sdk/blob/main/docs/playback-events.md),
  [plugin cooperation](https://github.com/vicrodh/chilpa-plugins-sdk/blob/main/docs/plugin-cooperation.md) and
  [track lyrics](https://github.com/vicrodh/chilpa-plugins-sdk/blob/main/docs/track-lyrics.md).

When a page and a host build disagree, the host is right and the page gets
fixed. A plugin is ready when its package loads in Chilpa and every path it
declares has been exercised there.
