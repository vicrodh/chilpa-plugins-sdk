# Native process plugins

Write a process plugin when the WASM host calls fall short, for example when
you need an existing native library. Chilpa runs these inside a platform sandbox: Bubblewrap on Linux and
Seatbelt on macOS. The package holds `manifest.json` with
`"runtime": "process"` and an executable named `plugin`. Chilpa inspects
and approves the package before the process starts.

## Wire protocol

One JSON-RPC 2.0 object per line, in on standard input and out on standard
output. Standard output carries only protocol messages; diagnostics go to
standard error. Every result or error echoes its request's `id`. The host
makes concurrent calls, so keep reading input while a request waits on a
service or on a `host.reply`.

The first request is `initialize`. The host sends `protocol: 1`,
`playback_events: 1`, its name and version, `data_dir`, `cache_dir` and a locale.
Use the supplied directories: Linux maps them to `/data` and `/cache`;
macOS supplies absolute private directories.
Answer with `protocol: 1` and the capabilities you actually serve:

```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocol":1,"playback_events":1,"host":{"name":"chilpa","version":"0.1"},"data_dir":"/data","cache_dir":"/cache","locale":"en"}}
{"jsonrpc":"2.0","id":1,"result":{"protocol":1,"capabilities":{"search":true}}}
```

Any other protocol number is rejected. The manifest's `api_version: 1` is a
separate package check. Advertise `search`, `pages`, `track_quality`,
`track_lyrics`, `playback_events`, `playlist_edit`, `playlist_import`,
`downloads` or any other capability only after its methods work against the
current host. Capabilities are negotiated here; they are not manifest
permissions.

## Source methods and extensions

A source usually starts with `search` and `track.resolve`. Catalog pages use
`page.get` and `page.action`, with the document shape in
`docs/native-pages.md`. Pass opaque references back unchanged; only the
plugin interprets them. Add the optional contracts you need:

| Contract | Covers |
| --- | --- |
| `docs/native-pages.md` | Pages, library cards, quality and playlist actions |
| `docs/playback-events.md` | Resolution tickets and measured playback reports |
| `docs/plugin-cooperation.md` | The host broker: reading catalogs and writing selected playlists |
| `docs/track-lyrics.md` | Lyrics for this source's own tracks |

To use the broker, send a `host.request` notification and match its `id`
against the later `host.reply`. Never call another plugin directly or read
its data. A write that times out may already have gone through, so don't
retry it blindly. Declare `plugins_read`, `library_read` or
`playlists_write` in the manifest for the operations you use.

## Accounts and files

Declare `own_account` only when the plugin really logs in and stores its own
credential. Chilpa can neither verify that nor erase the credential. If
Chilpa should own the login, use the manifest's `auth` field instead. Keep
tokens, stream URLs and resolution tickets out of logs. Persistent data goes
under the supplied `data_dir`, temporary data under `cache_dir`.
The host also sets `CHILPA_PLUGIN_DATA_DIR` and `CHILPA_PLUGIN_CACHE_DIR`. Writing to a
folder the listener picked needs `folder_write`; a path inside a request
grants no access.

Test concurrent requests, a closed pipe, timeouts, malformed input, a failed
login, and the package installed in a clean Chilpa profile. Before relying
on an optional method, check that the host you target implements it.

## Native source example

I use [chilpa-pg-spotify](https://github.com/vicrodh/chilpa-pg-spotify) as a
complete source example. It implements the existing process contract without
changing Chilpa core or embedding another player UI. Its README attributes
the MIT-licensed Spotifast work by Carmine Paolino and records immutable
upstream revisions and dependency licenses.

| Contract | Implementation to read |
| --- | --- |
| Concurrent requests, initialization and typed errors | `src/main.rs`, `src/web_api.rs` |
| Account-owned browser setup and callbacks | `src/auth.rs`, `src/callback.rs`, `src/wizard.rs` |
| Search, pages, stable references and library snapshots | `src/catalog.rs`, `src/native_catalog.rs`, `src/snapshot.rs` |
| Permission-checked playlist creation and track additions | `src/playlist_actions.rs` |
| Source lyrics and direct audio resolution | `src/lyrics.rs`, `src/stream.rs` |
| Executable protocol and exact-package host checks | `tests/test_protocol.py`, `tools/check-host-catalog`, `tools/check-macos-host` |

I start with its account-free fixture and protocol tests before using a real
account. Full playback requires Premium; the guided setup recommends a personal
Client ID. Shared application quotas and private service endpoints can restrict
features independently. Linux and macOS have separate release ZIPs, and the Mac
package requires a host with native process support. I check the repository's
validation report for the tested scope, including unverified live Mac playback.
I adapt the contract and resource boundaries to my own source; I do not copy
service credentials or assume its endpoints grant another plugin access.

## Platform packages

I build the executable and every bundled dependency for the target OS and
architecture. A native ZIP is not interchangeable between Linux and macOS.
The current macOS distribution targets Apple Silicon (`macos-aarch64`);
Windows process hosting is not implemented. On macOS I ship a Mach-O
executable with its runtime libraries inside the package; a script cannot
invoke a separate system interpreter under the worker policy. Older macOS host builds without
the native sandbox cannot run these packages.

An index entry keeps its top-level Linux `package` and `sha256` for older
hosts and adds an optional platform map:

```json
"packages": {
  "linux-x86_64": {"package": "https://example.org/plugin-linux.zip", "sha256": "<Linux SHA-256>"},
  "macos-aarch64": {"package": "https://example.org/plugin-macos.zip", "sha256": "<macOS SHA-256>"}
}
```

The host chooses `{OS}-{ARCH}` using Rust's platform names and verifies that
package's SHA-256. A declared map without the current platform omits the plugin from the visible
catalog and prevents staging it. An omitted map declares a portable package,
such as a WASM ZIP. Native process plugins must declare every supported target
in `packages`; never publish a platform-specific binary as portable. Protocol and manifest API versions remain 1.

## macOS confinement

I keep the package read-only and write only to my private data/cache or an
explicitly approved export folder. The worker cannot fork or execute another
program. The host clears inherited environment variables, limits descriptors
and monitors physical memory. Disabling network access denies external
connections; local-host networking remains available for local audio readers.
Network-enabled native workers have direct networking, unlike WASM's
host-mediated HTTP domain checks.

The macOS implementation uses the system's `sandbox-exec` and Seatbelt
profile interface. [Apple does not support custom Seatbelt profiles](https://developer.apple.com/forums/thread/661939) as a
public API for third parties; compatibility must be tested on supported
macOS releases. The host fails closed if confinement cannot start. I test
the exact ZIP through the actual host installer, permission review and
sandbox, in a temporary profile without a real account or audio device.
