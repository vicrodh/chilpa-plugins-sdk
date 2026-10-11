# Process contract

I set `"runtime": "process"` in `manifest.json` and package an executable
named `plugin`. The target host currently launches it only inside the
Linux sandbox. It speaks JSON-RPC 2.0, one object per line on standard
input and output. I use standard error for diagnostics and return each
request's `id`. The host may make concurrent calls, so I keep reading
input while another call waits.

The first host request is `initialize`; I answer with `protocol: 1` and
only capabilities whose methods work. This is separate from manifest
`api_version: 1` and from manifest permission requests. The current host
passes `/data`, `/cache`, locale, host identity, and
`playback_events: 1` in its greeting.

For a source I verify `search` and `track.resolve` against the host
before adding optional contracts. In the `chilpa-plugins` checkout,
`docs/native-pages.md` covers `page.get`, `page.action`, library
cards, quality and playlist editing; `docs/playback-events.md` covers
measured playback; `docs/track-lyrics.md` covers this source's own
lyrics; and `docs/plugin-cooperation.md` covers `host.request` and
`host.reply`. I read the applicable contract and the target host code
before advertising its capability.

The broker grants only declared `plugins_read`, `library_read`, or
`playlists_write` operations. I do not read another plugin's files or
credentials or call it directly. A broker write that times out may have
committed; I check its destination before retrying. I declare
`folder_write` for a selected export folder. I use `own_account` only
when the process owns its auth and disclose its credential storage honestly.
I never log credentials, stream URLs, or resolution tickets.

I test concurrent requests, malformed input, a closed pipe, timeout,
permission denial, startup, and restart. I run a host smoke against the
exact packaged executable in an isolated profile. A unit test of the
JSON-RPC handler alone does not prove the sandbox can launch it.
