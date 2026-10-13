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
| Version pins, tests and release checks | [Compatibility and testing](Compatibility-and-Testing.md) |

## Two interfaces

**WASM** uses the Rust `chilpa-plugin-api` message types and
`chilpa-plugin-sdk`. A module exports operations such as `chilpa_search`,
`chilpa_browse` and `chilpa_resolver`. It has no network, disk or clock of
its own; the host makes those calls for it.

**Process** speaks JSON-RPC 2.0 over standard input and output, one object
per line. The host calls `initialize`, then methods such as `search`,
`page.get` or `track.resolve`, depending on the capabilities the plugin
advertises. Process plugins run only in the Linux sandbox for now, and the
Rust SDK does not implement this interface.

Both are validated against `manifest.json`, with `api_version: 1`. A process
plugin also answers `protocol: 1` in `initialize`. The `protocol-4` Git tag
is a source revision, neither of those numbers.

## Where the contracts are

- WASM data shapes: `chilpa-plugin-api/src/lib.rs`.
- WASM exports and host calls: `chilpa-plugin-sdk/src/`.
- Test module: `fixture/`.
- Process extensions: `docs/native-pages.md`, `docs/playback-events.md`,
  `docs/plugin-cooperation.md` and `docs/track-lyrics.md`.

When a page and a host build disagree, the host is right and the page gets
fixed. A plugin is ready when its package loads in Chilpa and every path it
declares has been exercised there.
