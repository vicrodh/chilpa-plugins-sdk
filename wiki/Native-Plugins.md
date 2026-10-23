# Native process plugins

Write a process plugin when the WASM host calls fall short, for example when
you need an existing native library. Chilpa runs these inside a platform
sandbox: Bubblewrap on Linux and Seatbelt on macOS. The package holds
`manifest.json` with `"runtime": "process"` and an executable named
`plugin`. Chilpa inspects and approves the package before the process
starts.

## Wire protocol

One JSON-RPC 2.0 object per line, in on standard input and out on standard
output. Standard output carries only protocol messages. Diagnostics go to
standard error, which the host copies to its log as `plugin[<id>]: <line>`.
The host's request IDs are integers; echo each one unchanged in its result
or error. The host makes concurrent calls, so keep reading input while a
request waits on a service or on a `host.reply`. Lines that are not JSON are
skipped.

The plugin may send two notifications: `host.request`, for the
[broker](https://github.com/vicrodh/chilpa-plugins-sdk/blob/main/docs/plugin-cooperation.md),
and `log`, whose message goes to the host log like a line of standard error:

```json
{"jsonrpc":"2.0","method":"log","params":{"message":"catalog refreshed"}}
```

A broker request carries a string `id` that the later `host.reply` echoes,
and needs `plugins_read`, `library_read` or `playlists_write` in the manifest
for the operation it uses. Never call another plugin directly or read its
data. A broker write that times out may already have gone through, so don't
retry it blindly.

Answer JSON-RPC error `-32601` at once for a method you do not implement.
Some methods reach plugins that never advertised them, and an unanswered
call holds the host until its time limit. The exception is `search` on a
`source`: global search sends it to every process source and shows an error
next to the results, so a source that does not search answers empty lists.

## Initialize

The first request is `initialize`. Answer within 15 seconds with
`protocol: 1` and the capabilities you actually serve. Another protocol
number, or no answer in time, and the plugin does not start.

```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocol":1,"playback_events":1,"host":{"name":"chilpa","version":"0.1.0"},"data_dir":"/data","cache_dir":"/cache","locale":"en","network":{"enabled":true}}}
{"jsonrpc":"2.0","id":1,"result":{"protocol":1,"capabilities":{"search":true,"pages":true}}}
```

`data_dir` and `cache_dir` are the plugin's private directories: `/data` and
`/cache` on Linux, absolute private paths on macOS. `locale` is the window's
language. `network.enabled` is false in offline mode, and the host restarts
every process plugin when that setting changes. `playback_events: 1` says
the host reports playback. The manifest's `api_version: 1` is a separate
package check.

Capabilities are negotiated here; they are not manifest permissions.
Advertise one only after its methods work against the host you target. The
host compares some with the number `1` and the rest with `true`, and any
other value leaves the capability off.

| Capability | Value | What it turns on |
| --- | --- | --- |
| `search` | `true` | Open music link, artist scenes (with `pages`), and the broker's `sources.list` and `catalog.search`. Global search does not need it. |
| `pages` | `true` | Native pages: the source opens on `page.get({"ref":"discover:home"})`, and the window sends `page.action`. |
| `navigation` | object | Titles for the sections `discover`, `library`, `albums`, `tracks`, `artists`, `labels` and `playlists`. |
| `home` | `true` | A Discover entry in the sidebar, built from the `home` shelves. |
| `library` | array | Library tabs, each read with `library.list`: any of `albums`, `tracks`, `artists`, `labels` and `playlists`. |
| `favorites` | `true` | Hearts: `favorites.set` for tracks (needs `"tracks"` in `library`), `page.action` `favorite` for the rest (needs `pages`). |
| `playlist_edit` | `true` | Creating and editing the account's playlists through `page.action`. |
| `playlist_remove` | `true` | Removing a saved playlist the listener does not own sends `playlist.delete` instead of `favorite` with `on: false`. |
| `folder_edit` | boolean | The account's playlist folders can be edited. Defaults to `playlist_edit`. |
| `playlist_limit` | integer | Most tracks per playlist an import writes. Default 2000, clamped to 1–10000. |
| `scene_search_limit` | integer | Searches per artist scene. Default 5, at most 1000; 0 leaves the source out. |
| `track_quality` | `true` | `track.quality` for the rows on screen. |
| `track_lyrics` | `1` | `track.lyrics`. |
| `playback_events` | `1` | `playback.event`. |
| `playlist_import` | `1` | The playlist import screen. |
| `settings` | `true` | The settings form. `"auth": "browser"` does the same. |
| `downloads` | `true` | The downloads view. |

## Methods

| Method | When the host sends it | Limit | Contract |
| --- | --- | --- | --- |
| `search` | Global search, to every enabled `source`; Open music link, the broker and artist scenes, to sources with `search`; artist lookups by name | 30 s; 15 s from a link | [source](https://github.com/vicrodh/chilpa-plugins-sdk/blob/main/docs/native-source.md) |
| `track.resolve` | Before a track plays, sometimes ahead of time | 20 s | [source](https://github.com/vicrodh/chilpa-plugins-sdk/blob/main/docs/native-source.md) |
| `track.get` | A list shows a track the host has not read; a linked track | 30 s; 15 s from a link | [source](https://github.com/vicrodh/chilpa-plugins-sdk/blob/main/docs/native-source.md) |
| `album.get`, `playlist.get` | The listener opens or plays an album or playlist | 30 s | [source](https://github.com/vicrodh/chilpa-plugins-sdk/blob/main/docs/native-source.md) |
| `artist.get` | An artist or label card without `pages`; artist lookups by name | 30 s | [source](https://github.com/vicrodh/chilpa-plugins-sdk/blob/main/docs/native-source.md) |
| `home` | The listener opens a source without `pages` | 30 s | [source](https://github.com/vicrodh/chilpa-plugins-sdk/blob/main/docs/native-source.md) |
| `library.list` | Each tab named in `library` | 30 s a page | [source](https://github.com/vicrodh/chilpa-plugins-sdk/blob/main/docs/native-source.md) |
| `favorites.set` | A track heart, with `favorites` | 30 s | [source](https://github.com/vicrodh/chilpa-plugins-sdk/blob/main/docs/native-source.md) |
| `link.resolve` | Open music link, to sources with `search`, all at once | 15 s | [source](https://github.com/vicrodh/chilpa-plugins-sdk/blob/main/docs/native-source.md) |
| `link.identify` | Open music link, to every process plugin, one at a time | 35 s | [source](https://github.com/vicrodh/chilpa-plugins-sdk/blob/main/docs/native-source.md) |
| `page.get`, `page.action` | With `pages` | 90 s, 30 s | [pages](https://github.com/vicrodh/chilpa-plugins-sdk/blob/main/docs/native-pages.md) |
| `track.quality` | With `track_quality` | 20 s | [pages](https://github.com/vicrodh/chilpa-plugins-sdk/blob/main/docs/native-pages.md) |
| `track.lyrics` | With `track_lyrics: 1` | 4 s | [lyrics](https://github.com/vicrodh/chilpa-plugins-sdk/blob/main/docs/track-lyrics.md) |
| `playback.event` | With `playback_events: 1` | 2 s | [playback events](https://github.com/vicrodh/chilpa-plugins-sdk/blob/main/docs/playback-events.md) |
| `settings.open`, `settings.submit` | With `settings`, when the listener opens them | 15 s | [settings](https://github.com/vicrodh/chilpa-plugins-sdk/blob/main/docs/native-settings.md) |
| `downloads.open`, `downloads.action` | With `downloads`, while the view is open | 30 s | [settings](https://github.com/vicrodh/chilpa-plugins-sdk/blob/main/docs/native-settings.md) |
| `import.*` | With `playlist_import: 1` | 45 s | [cooperation](https://github.com/vicrodh/chilpa-plugins-sdk/blob/main/docs/plugin-cooperation.md) |
| `chilpa_revoke` | Uninstalling from Settings | 2 s | [playback events](https://github.com/vicrodh/chilpa-plugins-sdk/blob/main/docs/playback-events.md) |

The `analysis.*` methods go only to Chilpa's own Mood plugin. Pass opaque
references back unchanged; the pages contract lists the few conventions the
host relies on.

## Accounts and files

Declare `own_account` only when the plugin really logs in and stores its own
credential; Chilpa cannot verify how it handles it. The manifest's `auth`
field is for WASM plugins: the host adds that credential to WASM `http`
requests, and a process plugin never receives it. Keep tokens, stream URLs
and resolution tickets out of logs. Persistent data goes under the supplied
`data_dir`, temporary data under `cache_dir`, and uninstalling deletes both.
Linux maps them to `/data` and `/cache` and points `HOME` and
`XDG_CACHE_HOME` at them; macOS also sets `CHILPA_PLUGIN_DATA_DIR` and
`CHILPA_PLUGIN_CACHE_DIR`. [Credentials](Credentials.md) says where an
account credential goes.

Writing to a folder the listener picked needs `folder_write`; a path inside a
request grants no access. The host restarts the plugin when the listener
picks the folder. On Linux the folder appears at `/export`. On macOS the
sandbox allows it, but current hosts do not tell the plugin its path.

Test concurrent requests, a closed pipe, timeouts, malformed input, a failed
login, and the package installed in a clean Chilpa profile. Before relying
on an optional method, check that the host you target implements it.

## Sandbox and limits

On Linux the process gets 2 GiB of address space, 256 open files and no core
dumps. The package is read-only at `/plugin`, `/tmp` is private, and there
is no `/home` or `/run`. The environment holds only `PATH`, `HOME`,
`XDG_CACHE_HOME` and `SSL_CERT_DIR`.

On macOS the host clears inherited environment variables and limits open
files to 256. It ends the process when its physical memory passes 2 GiB,
checked eight times a second. The process cannot fork or run another
program. It reads its package and the system libraries, and writes only to
its data and cache directories and an approved export folder.

On both, the process has direct network access when the manifest declares
any domain and Chilpa is online, and no outside network otherwise; macOS
still allows local-host connections for local audio readers. A line over
32 MiB from the plugin closes the channel. When the plugin stops reading and
256 messages wait for it, new calls fail at once. A plugin that ends starts
again on the next call that needs it, after 1, 2, 5 and then 30 seconds;
the count resets once it has run for five minutes, and calls fail while it
is down.

The macOS implementation uses the system's `sandbox-exec` and Seatbelt
profile interface. [Apple does not support custom Seatbelt profiles](https://developer.apple.com/forums/thread/661939)
as a public API for third parties; compatibility must be tested on supported
macOS releases. The host fails closed if confinement cannot start. Test the
exact ZIP through the actual host installer, permission review and sandbox,
in a temporary profile without a real account or audio device.

## Native source example

[chilpa-pg-spotify](https://github.com/vicrodh/chilpa-pg-spotify) is a complete
source example. It implements the existing process contract without
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

Start with its account-free fixture and protocol tests before using a real
account. Full playback requires Premium; the guided setup recommends a personal
Client ID. Shared application quotas and private service endpoints can restrict
features independently. Linux and macOS have separate release ZIPs, and the Mac
package requires a host with native process support. Check the repository's
validation report for the tested scope, including unverified live Mac playback.
Adapt the contract and resource boundaries to the new source. Do not copy
service credentials or assume its endpoints grant another plugin access.

## Platform packages

Build the executable and every bundled dependency for the target OS and
architecture. A native ZIP is not interchangeable between Linux and macOS.
The current macOS distribution targets Apple Silicon (`macos-aarch64`);
Windows process hosting is not implemented. On macOS, ship a Mach-O
executable with its runtime libraries inside the package; a script cannot
invoke a separate system interpreter under the worker policy. Older macOS
host builds without the native sandbox cannot run these packages.

An index entry keeps its top-level Linux `package` and `sha256` for older
hosts and adds an optional platform map:

```json
"packages": {
  "linux-x86_64": {"package": "https://example.org/plugin-linux.zip", "sha256": "<Linux SHA-256>"},
  "macos-aarch64": {"package": "https://example.org/plugin-macos.zip", "sha256": "<macOS SHA-256>"}
}
```

The host chooses `{OS}-{ARCH}` using Rust's platform names and verifies that
package's SHA-256. A declared map without the current platform omits the
plugin from the visible catalog and prevents staging it. An omitted map
declares a portable package, such as a WASM ZIP. Native process plugins must
declare every supported target in `packages`; never publish a
platform-specific binary as portable. Protocol and manifest API versions
remain 1.
