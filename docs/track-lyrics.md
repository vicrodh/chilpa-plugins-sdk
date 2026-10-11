# Lyrics of a source's own tracks

This contract is for JSON-RPC process sources. It does not add a WASM
`chilpa_track_lyrics` export. I check the target host's
`track_lyrics` negotiation before advertising it.

Lyrics belong to Chilpa, not to a plugin. Chilpa finds a track's lyrics in
this order: a file next to the track or its tags, what the track's source
has for it, and Chilpa's own lyrics service for everything else. There is no
kind of plugin whose job is lyrics, and no method to look lyrics up by name.

A source plugin takes part in one way: it may have the lyrics of **its own
tracks**. When it does, they win over anything the lyrics service finds,
however finely that is timed. They are for exactly that track.

## Capability

```json
{"protocol": 1, "capabilities": {"track_lyrics": 1}}
```

Without it the host never calls the method.

## `track.lyrics`

```json
{"ref": "track:42", "language": "es"}
```

`ref` is the same opaque reference `track.resolve` takes. `language` is
optional and asks for a translation next to the original; a plugin without
translations ignores it.

### Found

```json
{
  "status": "found",
  "lyrics": {"format": "lrc", "text": "[00:12.00]First line\n[00:15.50]Second line"},
  "fingerprint": {"duration_ms": 215000, "isrc": "QZES72500001"}
}
```

`lyrics.format` is one of:

| Format | Carries | For |
| --- | --- | --- |
| `lrc` | `text` | LRC, by line or enhanced with `<mm:ss.xx>` word stamps |
| `ttml` | `text` | TTML, by line, word or syllable |
| `plain` | `text` | text with no timing; stamps inside it are not believed |
| `document` | `document` | lyrics already in Chilpa's shape, for a format only the service uses |

A `document` is:

```json
{
  "schema": 1,
  "timing": "word",
  "language": "en",
  "lines": [
    {"start_ms": 1750, "end_ms": 4770, "text": "Oh, mm-mm",
     "segments": [
       {"text": "Oh,", "start_ms": 1750, "end_ms": 2480, "space_after": true},
       {"text": "mm-mm", "start_ms": 2480, "end_ms": 4770, "space_after": false}
     ]}
  ],
  "layers": [{"kind": "translation", "language": "es", "lines": [{"line": 0, "text": "Oh, mm-mm"}]}],
  "credits": [{"name": "A Writer", "role": "songwriter"}]
}
```

Times are milliseconds from the start of the track. What the service does
not state is left out, never sent as zero: an unknown `end_ms` is absent. A
segment's `text` has no spaces around it and `space_after` says whether one
follows, so words, syllables and scripts written without spaces have one
representation. Empty lines that only mark a silence are not sent; the line
before them keeps its own end. The host reads `timing` from the lines
themselves, whatever the field says.

`fingerprint` is what the plugin knows about the track, so the host can
keep the lyrics against the right recording: its length and its ISRC. Leave
out what is unknown.

Never answer with an address for the host to fetch. The plugin reads the
lyrics itself, with its own session and inside its own allowed domains, and
hands over the result. Answers larger than 512 KiB of text are not read.

### None

```json
{"status": "none"}
```

Only when the service said so: the track has no lyrics there. The host
remembers it for days and asks its lyrics service instead.

### Failure

A JSON-RPC error, never `none`: no session, a network failure, a 5xx, an
answer that could not be read. The host falls back on its lyrics service for
this play and asks the plugin again next time. When the service asked for a
pause (a 429 with `Retry-After`), pass it on:

```json
{"code": -32000, "message": "The service is busy", "data": {"retry_after_ms": 30000}}
```

## What the host does

- It calls `track.lyrics` and its lyrics service at the same time, so a
  plugin that answers `none` adds no wait. The plugin has 4 seconds.
- Found lyrics are kept by plugin and reference, with the track they were
  found for, and dropped when the plugin is removed.
- Playback never waits for lyrics.
- No credential of the plugin's account leaves the plugin: the lyrics
  service is asked with the track's names, length and identifiers only.
