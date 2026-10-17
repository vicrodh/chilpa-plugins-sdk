# Process contract

Set `"runtime": "process"` in `manifest.json` and package an executable
named `plugin`. The host launches it inside Bubblewrap on Linux or Seatbelt on macOS.
Build separate native packages for each advertised OS and architecture.
It speaks JSON-RPC 2.0, one object per line on standard input and output.
Diagnostics go to standard error, and every reply echoes its request's `id`.
The host makes concurrent calls, so keep reading input while another call
waits.

The first request is `initialize`. Answer with `protocol: 1` and only the
capabilities whose methods work. That number is separate from the manifest's
`api_version: 1` and from its permissions. The current host's greeting
carries `data_dir`, `cache_dir`, the locale, the host identity and
`playback_events: 1`. Use the supplied directories rather than hard-coding
Linux paths; macOS supplies absolute private directories. See
`wiki/Native-Plugins.md` for per-platform catalog packages and confinement limits.

For a source, get `search` and `track.resolve` working against the host
before adding optional contracts. In a `chilpa-plugins-sdk` checkout:

- `docs/native-pages.md`: `page.get`, `page.action`, library cards, quality
  and playlist editing.
- `docs/playback-events.md`: measured playback.
- `docs/track-lyrics.md`: lyrics of the source's own tracks.
- `docs/plugin-cooperation.md`: `host.request` and `host.reply`.

Read the contract and the target host's code before advertising a
capability.

The broker allows only the `plugins_read`, `library_read` and
`playlists_write` operations the manifest declares. Never read another
plugin's files or credentials, or call it directly. A broker write that
times out may have gone through; check the destination before retrying.
Declare `folder_write` for an export folder the listener picks. Use
`own_account` only when the process owns its login, and say plainly where it
keeps the credential. Never log credentials, stream URLs or resolution
tickets.

Test concurrent requests, malformed input, a closed pipe, timeouts,
permission denial, startup and restart. Run a host test against the exact
packaged executable in an isolated profile; a unit test of the JSON-RPC
handler does not prove the sandbox can launch it.
