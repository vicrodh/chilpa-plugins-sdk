# Native process plugins

I use a process plugin when the WASM host calls are insufficient, for
example when an existing native library needs its own runtime. Chilpa
currently starts these plugins only in its Linux sandbox. A package has
`manifest.json` with `"runtime": "process"` and an executable named
`plugin`. The package is inspected and approved before the process starts.

## Wire protocol

I read one JSON-RPC 2.0 object per line from standard input and write one
object per line to standard output. I reserve standard output for protocol
messages and send diagnostics to standard error. I echo each request's
`id` in its result or error. I keep reading input while another request
waits on a service or on a `host.reply`; the host can make concurrent
calls.

The first request is `initialize`. The host sends `protocol: 1`,
`playback_events: 1`, a host name and version, `/data`, `/cache`, and a
locale. I answer with `protocol: 1` and the capabilities I actually serve:

```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocol":1,"playback_events":1,"host":{"name":"chilpa","version":"0.1"},"data_dir":"/data","cache_dir":"/cache","locale":"en"}}
{"jsonrpc":"2.0","id":1,"result":{"protocol":1,"capabilities":{"search":true}}}
```

The host rejects another protocol number. The manifest's `api_version: 1`
is a separate package check. I advertise `search`, `pages`,
`track_quality`, `track_lyrics`, `playback_events`, `playlist_edit`,
`playlist_import`, `downloads`, or other capability fields only after
implementing their methods and testing them against the current host.
The capability map is a negotiation result, not a manifest permission.

## Source methods and extensions

A source generally starts with `search` and `track.resolve`. Catalog
pages use `page.get` and `page.action`; the detailed document shape is in
`docs/native-pages.md` in this repository. I retain opaque references
unchanged until the plugin interprets them. I add optional contracts only
when needed:

| Contract file | Use |
| --- | --- |
| `docs/native-pages.md` | Pages, library cards, quality, and playlist actions |
| `docs/playback-events.md` | Resolution tickets and measured playback reports |
| `docs/plugin-cooperation.md` | Host broker for reading catalogs and selected playlist writes |
| `docs/track-lyrics.md` | Lyrics for this source's own tracks |

For broker work, I send a `host.request` notification and correlate its
`id` with a later `host.reply`. I do not call another plugin directly or
read its data. A timed-out write can already have succeeded, so I do not
blindly retry it. The manifest separately declares `plugins_read`,
`library_read`, or `playlists_write` when those operations are used.

## Account and file boundaries

I declare `own_account` only when the plugin really authenticates and
stores its own credential. Chilpa cannot verify that declaration or erase
that credential. If Chilpa owns the auth flow, I use the manifest's `auth`
field instead. I never put tokens, stream URLs, or resolution tickets in
logs. I write persistent plugin data under the sandbox's `/data` and
temporary data under `/cache`. Selected export folders require
`folder_write`; a plugin cannot infer access from a path in a request.

I test a process plugin with concurrent requests, a closed pipe, timeouts,
malformed input, failed login, and a package installed in a clean Chilpa
profile. I check the current host implementation before relying on a
particular optional method.
