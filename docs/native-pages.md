# Native catalog pages

This contract is for JSON-RPC process plugins; it adds no exports to the Rust
WASM SDK. Check the target host's `pages` negotiation and `page.get`
handling before advertising it. A new process plugin starts at the wiki's
[Native plugins](../wiki/Native-Plugins.md) page; the methods that are not
about pages are in [native source](native-source.md).

A plugin that supplies native pages advertises `capabilities.pages: true` in
`initialize`. The host renders the documents with its album grids, artist
portraits, track lists and navigation. It passes references back as it
received them, with these exceptions:

- It builds its own keys from page references with spaces and `|`, so keep
  both characters out of the references of pages and of the albums,
  artists, labels and playlists that open them.
- Track hearts work only on references that start with `track:`, and the
  host compares them on the part before `@playlist:`.
- Artist scenes accept only references that start with `artist:`.
- When the listener adds tracks to one of the plugin's playlists, the host
  sends each track reference up to its first `@`.

`initialize` can also carry optional `capabilities.navigation` titles, in the
requested locale, for example
`{"discover":"Music","library":"Collection","albums":"Albums"}`. The keys are
`discover`, `library`, `albums`, `tracks`, `artists`, `labels` and
`playlists`. Section IDs and routes stay the same. A missing or empty title
keeps the host's default.

`page.get({"ref":"<reference>","refresh":false})` returns a document:

```json
{
  "title": "Discover",
  "tabs": [{"title":"Home","ref":"discover:home","active":true}],
  "shelves": [{
    "id":"releases", "title":"New releases", "kind":"album",
    "items":[{"kind":"album","ref":"album:42","title":"Record"}],
    "more":"releases:0", "enabled":true
  }]
}
```

A listing returns `items` and an optional `next` reference instead of `shelves`. The host keeps each page in navigation history. Keep section IDs stable so the listener's visibility, order and size preferences survive content changes.

The source opens on `page.get({"ref":"discover:home","refresh":false})`. A
shelf's `more` reference comes back as `page.get({"ref":"<more>"})`, answered
with a listing. The host waits 90 seconds for a page and 60 for more of a
shelf, and does not read the same page again within 30 seconds. `refresh` is
true when it reads a page again after an action or a retry marked it stale.

Items have the shape described in [native source](native-source.md#items): `kind` and `ref` identify one, and `title`, `image`, `artist`, `artist_ref`, `album_ref`, `year`, `duration_ms`, `genre`, `label`, `label_ref`, `release_date`, `quality` and the rest are optional. Track references may carry playback context after an `@`; the plugin interprets and validates them, including for favorite and resolve operations.

An artist, label or spotlight page can have a `header` with item identity, image and description. A shelf can supply `extra_tracks` alongside its cards. Playlist shelves can supply `tags` (`id`, `slug`, `name`); each playlist carries its matching tags. The host filters these without replacing playlist identity.

A shelf with `kind: "mixed"` holds cards of several kinds in the order of its
`items`, and each card keeps its own kind, so an artist opens the artist.
Tracks beside the cards go in `extra_tracks`. A shelf's `header` item becomes
a card the listener can act on, like a library card. When one shelf replaces
older ones, list their IDs in `previous_ids`: the listener's order, size and
visibility carry over, a preference saved for the new ID wins, and the shelf
stays visible if any older one was.

When the provider advertises high-resolution lossless audio without the exact
resolution, catalog quality is `quality: {"codec":"flac","tier":"hi_res"}`.
Include `bits` and `rate` only when the provider gives those numbers. The host shows the advertised tier without inventing a sample rate;
the decoded stream's measured format takes precedence during playback.

Discover documents can include `tab`, `genres` (`id`, `name`) and `genre_filter`. The native Discover convention uses `discover:<tab>:<comma-separated-genre-ids>`. Preferences are isolated by plugin and tab. Ordinary pages need not implement that convention.

A document can set `personal: true` when its content depends on listening history. After the plugin acknowledges a measured playback start, the host invalidates only that plugin's personal pages. Ordinary catalog pages stay cached.

Album metadata can include `credits`, `booklets` (`name`, `url`), `copyright`, `upc`, `description` and related `shelves`. Keep navigation references on the album and its tracks so artist and album links stay within the source. Booklet URLs are opened only when the listener clicks them.

`page.action` performs explicit listener actions:

```json
{"action":"favorite","ref":"album:42","on":true}
{"action":"pin","item":{"kind":"album","ref":"album:42","title":"Record"},"on":true}
```

Current hosts never send `pin`.

The window sends `page.action` only to a plugin with `pages: true`, except
`playlist.*` actions to one with `playlist_edit: true`. It waits for one
action per plugin at a time, up to 30 seconds. An action may also carry the
window's `request_id`, which the plugin can ignore. When the listener removes
a saved playlist, the host sends `playlist.delete` if the list is the
listener's own and editable or the plugin advertises `playlist_remove: true`,
and `{"action":"favorite","ref":"<playlist>","on":false}` otherwise.

After a successful `favorite` the host updates that item's heart and
re-reads the plugin's library tabs without re-reading pages. After any other
successful action it also marks every page of the plugin stale. Errors remain
visible in the source status. A plugin owns its pins and account data;
uninstall removes them with its private data directory.

Playback history must come from the audible playback lifecycle, never from a catalog fetch or preloading. Use the [playback events contract](playback-events.md), count each playback ID once, and tie history to the account that resolved the stream.

## Artist scenes

When the listener asks an artist scene to include plugins, the host uses the
sources that advertise `search: true` and `pages: true` and a
`scene_search_limit` above zero. For each candidate artist from MusicBrainz,
within that budget, it sends `search({"text":"<name>","limit":5})` and keeps
the artists whose `ref` starts with `artist:` and whose `title` matches the
name. When several match, it reads each with `page.get({"ref":"<artist>"})`
and compares up to two ISRCs against MusicBrainz credits. They come from a
shelf with `id` `tracks` or `top-songs` and `kind: "track"`, whose items
carry `isrc` and an `artist_ref` equal to that artist. A timeout, a closed
channel, or an error with code `-32001`, `data.terminal: true`,
`data.retry_after_ms` or `data.status` 401, 403 or 429 spends the rest of the
source's budget for that scene.

## Availability

An item may carry `available: false` only when the source explicitly knows
that the entity cannot be played. An absent value means unknown, not false.
For a terminal missing album, `album.get` returns
`{"availability":"unavailable","tracks":[]}`. The host retains its cached
identity and metadata and dims it instead of repeatedly preloading it.
The album's status does not mark every separately playable track unavailable.
Authentication, entitlement, rate limits and network failures remain errors;
they must not be converted into withdrawn catalog entries. Transient failures
use a retry cooldown; an explicit refresh may retry immediately.

## Provider normalization

`track.resolve` may return:

```json
{"normalization":{"gain_db":-4.2,"peak":0.93,"source":"provider","scope":"track"}}
```

The gain is a finite number in dB ready for the host's normalization stage;
peak is an optional positive finite linear amplitude. The host reads only
`gain_db` and `peak`; `source` and `scope` are informative. The plugin maps its
provider's units and reference into this contract. A gain of zero is present
data, not a missing value.

When normalization is enabled and processing is permitted, valid provider
data replaces local tags and local analysis. The host neither adds the gains
nor reads a second local value to supplement the provider peak. Without valid
provider gain it retains the existing local fallback. One stage applies gain
and existing boost/headroom limits. Seek, pause and preloading retain the same
resource metadata. Normalization off and the unprocessed output policy bypass
that stage. Original-byte casting does not apply host DSP; receiver processing
is separate and must not be described as host normalization.

## Playlist editing

A source that implements the playlist actions below through `page.action`
advertises `capabilities.playlist_edit: true`. The host then offers creation
on its playlist library tab. Playlist cards carry `editable`, `copyable`,
`public` and `description`; set `editable` only on the current account's own
lists. The plugin checks ownership again before every write.

Actions share an opaque playlist `ref`:

| Action | Additional input |
| --- | --- |
| `playlist.create` | `title`, `description`, `public`; no existing ref |
| `playlist.update` | `title`, `description`, `public` |
| `playlist.copy` | `title`, `description`, `public` for the new list |
| `playlist.delete` | none |
| `playlist.add` | `tracks`: array of opaque track refs |
| `playlist.remove` | `members`: array of opaque membership strings |
| `playlist.move` | `members`, `before`: insertion slot in the current order |
| `playlist.search` | `text`; returns `{read_only:true, items:[...]}` |

A membership identifies one occurrence, not a song. Duplicate songs retain
separate track refs and membership strings. A missing membership disables
per-entry editing without hiding the row. When resolving a ref that also
carries membership identity, keep the playlist's listening context.

Successful writes return `changed_refs`, an array of affected playlist refs.
The host invalidates those cards' tracks, its native page documents and
library listings. A search does not invalidate them. The facade exposes
pending/completed/error state per source and echoes a UI `request_id`;
controls wait for completion, and an error leaves the editor open. Removing
a plugin clears this transient action state and late replies are discarded.

Writes are never retried automatically: a timed-out request may have reached
the service. Copying a large list can create the destination before a later
track batch fails; the error identifies that partial list for inspection.

## Shared playlists and folders

To show source-owned playlists in the shared picker, advertise
`playlist_edit: true` and `library: ["playlists"]`. Each playlist carries
`editable` for the current account. Source-owned playlists accept only
references from that source; local playlists can hold tracks from several
sources. When a source is disabled, its identity and metadata are kept, and
unavailable tracks stay in place.

`playlist.create` returns `item.ref` or `ref` naming the created playlist. The host
can then call `playlist.add` in batches of up to 100 tracks, waiting for each
acknowledgement; the broker writes batches of up to 50. A partial
failure identifies that the playlist already exists instead of creating another one.
Local folders, sidebar visibility and ordering belong to the host; moving a source
playlist into a local folder never moves or copies it in the remote account.
Native account folders use the separate contract below.

### Native account folders

A plugin with account folders includes `folders` in `library.list` for `kind:
"playlists"`. An empty array advertises support even when the account has no
folders. Omitting it leaves the host's local folders as the only destination.

```json
{"items":[{"kind":"playlist","ref":"playlist:42","native_folder":"folder:a"}],
 "folders":[{"ref":"folder:a","name":"Road","parent":""}],"total":1}
```

References are opaque and scoped to the plugin. `parent` and `native_folder`
are empty for the account root. The host displays the hierarchy separately
from local folders, including empty folders. Its create and move dialogs label
local and native destinations explicitly. Local placement remains a sidebar
preference and does not change remote membership.

Reading account folders does not imply permission to change them. Advertise
`folder_edit: false` for read-only folders, or `true` once their writes work;
without it the host follows `playlist_edit`. Read-only folders stay visible,
and the window does not offer to change them. `folder_icon` in the manifest
gives these folders their artwork.

When folder editing is supported, the plugin handles `page.action` with `folder.create` (`title`, `parent`),
`folder.rename` (`ref`, `title`), and `playlist.folder` (`ref`, `folder`). Each
write waits for acknowledgement and propagates failures to the open dialog.
A successful move clears any local placement and reloads the source collection.
Plugins validate folder identities and account permissions before writing.

## Technical quality before playback

A source can advertise `capabilities.track_quality: true` when generic catalog
labels lack the actual delivery format. The host calls `track.quality` with
`{"ref":"track:42"}` for displayed rows, including remote tracks inside local
playlists. The result contains technical metadata only:

```json
{"ref":"track:42","quality":{"codec":"wav","bits":24,"rate":96000}}
```

This operation must not open audio, register a playback start, or create an
active player session. Plugins inspect metadata or clear stream manifests and
cache the result. Missing values stay missing; a catalog tier never invents
bit depth or sample rate. The codec names what the plugin delivers to Chilpa,
including a lossless container conversion when applicable.

The host limits concurrent requests to two and waits 20 seconds for each. It
skips a track whose rate and bit depth it already knows (rate alone for
`aac`, `mp3` and `m4a`), asks about the same track at most every five minutes, and after an
error leaves the plugin alone for a minute. It updates the shared track
cache, album header and playlist rows without playback.
Catalog refreshes with incomplete quality retain known technical values. Actual
decoder measurements can still correct them. The album header uses the highest
known track resolution while the list preserves each track's individual format.
