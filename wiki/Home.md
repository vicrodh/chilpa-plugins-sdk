# Chilpa plugin development

I use this wiki as the entry point for writing a Chilpa plugin. The pages are
versioned in this repository under `wiki/`. I check the published pages
against their source before release. I check the host code before treating a documented extension
as supported by a particular Chilpa build.

I reviewed the runtime and manifest guidance against Chilpa host
`1cce7299` and the `protocol-4` SDK on 2026-10-11. Optional process contracts
need their own host check for the build being targeted.

| I need | I read |
| --- | --- |
| A small, sandboxed source | [WASM quickstart](WASM-Quickstart.md) |
| Native libraries, a process, or an account the plugin manages | [Native plugins](Native-Plugins.md) |
| A valid package and an informed permission request | [Manifest and permissions](Manifest-and-Permissions.md) |
| Version pins, tests, and release checks | [Compatibility and testing](Compatibility-and-Testing.md) |

## Two interfaces

**WASM** uses the Rust `chilpa-plugin-api` message types and
`chilpa-plugin-sdk`. A WASM module exports operations such as
`chilpa_search`, `chilpa_browse`, and `chilpa_resolver`. It has no direct
network, disk, or clock access; the host mediates those calls.

**Process** uses JSON-RPC 2.0 over newline-delimited standard input and
output. The host calls `initialize` and methods such as `search`,
`page.get`, or `track.resolve` according to the capabilities the plugin
advertises. Process plugins currently start only in the Linux sandbox.
The Rust WASM SDK does not implement this interface.

Both use a validated `manifest.json`, including `api_version: 1`. For a
process plugin, the `initialize` result separately declares `protocol: 1`.
The SDK Git tag `protocol-4` is a source revision, not either version number.

## Contracts in the repository

I keep the WASM data shapes in `chilpa-plugin-api/src/lib.rs`, the WASM
exports and host calls in `chilpa-plugin-sdk/src/`, and the test module in
`fixture/`. The process extensions have detailed contracts in
`docs/native-pages.md`, `docs/playback-events.md`,
`docs/plugin-cooperation.md`, and `docs/track-lyrics.md`.

When a page and a host build disagree, I follow the host implementation for
that build and correct the page. A plugin is ready only after its package
loads and its declared paths are exercised in Chilpa.
