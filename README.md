# chilpa-plugins-sdk

I keep the versioned WASM protocol, Rust SDK, host-test fixture, developer
documentation, and wiki source pages in this repository. The installable
plugin catalog lives at [chilpa-plugins](https://github.com/vicrodh/chilpa-plugins).
The plugin sources live in their own repositories: [Radio](https://github.com/vicrodh/chilpa-pg-radio),
[Podcasts](https://github.com/vicrodh/chilpa-pg-podcasts), and
[Playlist Import](https://github.com/vicrodh/chilpa-pg-playlist-import).

## Choose a runtime

| Runtime | Interface | Use it when |
| --- | --- | --- |
| WASM | `chilpa-plugin-api` messages and `chilpa-plugin-sdk` exports | The plugin can work through host-mediated HTTP, clock, log and storage calls. |
| Process | JSON-RPC 2.0, one object per line | The plugin needs native libraries or an account it handles itself. The host currently starts process plugins only in its Linux sandbox. |

Both runtimes use `manifest.json` with `api_version: 1`. The Git tag
`protocol-4` names an SDK revision; it is not API version 4. A process
plugin also negotiates `protocol: 1` in `initialize`. The Rust SDK does
not implement the process protocol.

## Documentation

I start new plugin authors at [the wiki](wiki/Home.md). Its pages live here
so their changes can be reviewed with the SDK:

- [WASM quickstart](wiki/WASM-Quickstart.md), with a runnable
  [example](examples/hello-source/)
- [Native process plugins](wiki/Native-Plugins.md)
- [Manifest and permissions](wiki/Manifest-and-Permissions.md)
- [Compatibility and testing](wiki/Compatibility-and-Testing.md)

The detailed native contracts are [catalog pages](docs/native-pages.md),
[playback events](docs/playback-events.md), [plugin cooperation](docs/plugin-cooperation.md),
and [lyrics of a source's own tracks](docs/track-lyrics.md). These documents
describe process plugins; they do not add WASM exports.

## Real WASM reference

[Radio](https://github.com/vicrodh/chilpa-pg-radio) is an account-free source
to study after the minimal `hello-source` example. It uses a public station
directory, declares the network and storage permissions it needs, and covers
search, browsing, stream resolution, and recorded-response tests. It marks
HLS-only stations unavailable because the current WASM stream path cannot
play them. [Podcasts](https://github.com/vicrodh/chilpa-pg-podcasts) asks no
credential from listeners, but its directory calls require an application
secret held by Chilpa's proxy; Radio is the simpler credential-free reference.

## Workspace

- `chilpa-plugin-api` defines the WASM v1 messages. It depends on `serde`.
- `chilpa-plugin-sdk` provides the `plugin!` macro, mediated host calls,
  an optional directory cache, and a native fake host for tests.
- `fixture` has fixed answers for host integration tests.

I pin the host and each WASM plugin to a reviewed SDK commit or immutable tag.
I change the pin only after checking compatibility and building a module
against the host. `protocol-4` is the current shared pin. Changes after
that tag add the directory cache and documentation.

I run `cargo test --workspace` for the SDK's native tests and
`cargo test --manifest-path examples/hello-source/Cargo.toml` for the
example. For a WASM package I install the `wasm32-unknown-unknown`
target, then run:

```sh
tools/build-plugin examples/hello-source <out-dir>
tools/install-dev <out-dir>/app.example.hello <XDG_DATA_HOME>
```

The first command writes `plugin.wasm` and `manifest.json` under the
plugin ID. The second copies that directory to a development data home.
The release profile strips the module; a debug build is much larger.

## Authoring skill

I keep a reusable agent skill at
[skills/create-chilpa-plugin/SKILL.md](skills/create-chilpa-plugin/SKILL.md).
It reads the relevant wiki page, checks the host contract, and requires
permission, compatibility, and integration checks before a plugin is called
ready. It does not grant permission to install or publish a plugin.

I install it by copying the whole `skills/create-chilpa-plugin` directory
into my Codex skills directory; its `references/` files travel with it.

With the skill installed, this prompt is enough to start a plugin:

> Use $create-chilpa-plugin to create a Chilpa plugin for Bandcamp. Research
> the service API and Chilpa host support, choose the runtime, implement only
> verified capabilities, and test the package in an isolated profile.

Replace the service and desired behavior with your own request. The skill
checks permissions, credentials, SDK pins, and the host contract before
calling a package ready.

I maintain the project wiki from the Markdown files in `wiki/`. I run
`python3 tools/sync-wiki <clean-wiki-checkout>` to copy reviewed pages;
it turns `Page.md` links into wiki links to `./Page/`. I review the
source diff here before updating the wiki repository.
