# Native process plugins

Write a process plugin when the WASM host calls fall short, for example when
you need an existing native library. Chilpa runs these only in its Linux
sandbox for now. The package holds `manifest.json` with
`"runtime": "process"` and an executable named `plugin`. Chilpa inspects
and approves the package before the process starts.

## Wire protocol

One JSON-RPC 2.0 object per line, in on standard input and out on standard
output. Standard output carries only protocol messages; diagnostics go to
standard error. Every result or error echoes its request's `id`. The host
makes concurrent calls, so keep reading input while a request waits on a
service or on a `host.reply`.

The first request is `initialize`. The host sends `protocol: 1`,
`playback_events: 1`, its name and version, `/data`, `/cache` and a locale.
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
under the sandbox's `/data`, temporary data under `/cache`. Writing to a
folder the listener picked needs `folder_write`; a path inside a request
grants no access.

Test concurrent requests, a closed pipe, timeouts, malformed input, a failed
login, and the package installed in a clean Chilpa profile. Before relying
on an optional method, check that the host you target implements it.
