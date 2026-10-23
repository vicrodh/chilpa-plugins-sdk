# Process contract

Set `"runtime": "process"` in `manifest.json` and package an executable
named `plugin`. The host launches it inside Bubblewrap on Linux or Seatbelt
on macOS. Build separate native packages for each advertised OS and
architecture and list them in the catalog entry's `packages` map
(`wiki/Packages-and-Catalog.md`).

It speaks JSON-RPC 2.0, one object per line on standard input and output.
Diagnostics go to standard error or a `log` notification. The host's
request IDs are integers; echo each unchanged. The host makes concurrent
calls, so keep reading input while another call waits. Answer `-32601` at
once for any method you do not implement, except `search` on a `source`,
which every global search reaches: answer it with empty lists.

The first request is `initialize`, which carries `data_dir`, `cache_dir`,
the locale, the host identity, `playback_events: 1` and `network.enabled`.
Answer within 15 seconds with `protocol: 1` and only the capabilities whose
methods work. `playback_events`, `track_lyrics` and `playlist_import` must
be the number 1; the others are `true` or the types in the capability table
of `wiki/Native-Plugins.md`. Use the supplied directories rather than
hard-coding Linux paths.

For a source, get `search` and `track.resolve` working against the host
before adding optional contracts. In a `chilpa-plugins-sdk` checkout:

- `docs/native-source.md`: search, resolve, track, album, playlist and
  artist reads, `home`, `library.list`, `favorites.set` and links.
- `docs/native-pages.md`: `page.get`, `page.action`, playlist editing,
  quality and the reference conventions.
- `docs/native-settings.md`: the settings form, the sign-in window and the
  downloads view.
- `docs/playback-events.md`: measured playback.
- `docs/track-lyrics.md`: lyrics of the source's own tracks.
- `docs/plugin-cooperation.md`: `host.request`, `host.reply` and imports.

Read the contract and the target host's code before advertising a
capability.

The broker allows only the `plugins_read`, `library_read` and
`playlists_write` operations the manifest declares. Never read another
plugin's files or credentials, or call it directly. A broker write that
times out may have gone through; check the destination before retrying.
Declare `folder_write` for an export folder the listener picks; on Linux it
appears at `/export`. The manifest's `auth` never reaches a process plugin:
use `own_account` when the process owns its login, keep the credential in
`data_dir`, and say plainly where it lives (`wiki/Credentials.md`). Never log
credentials, stream URLs or resolution tickets.

Test concurrent requests, malformed input, a closed pipe, timeouts,
permission denial, startup and restart. Run a host test against the exact
packaged executable in an isolated profile; a unit test of the JSON-RPC
handler does not prove the sandbox can launch it.
