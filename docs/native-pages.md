# Native catalog pages

This contract is for JSON-RPC process plugins. It does not add exports to the
Rust WASM SDK. I check the target host's `pages` negotiation and `page.get`
handling before advertising it. The wiki's `Native-Plugins.md` is the
starting point for a new process plugin.

I advertise `capabilities.pages: true` in `initialize` to supply native pages. The host renders the documents with its album grids, artist portraits, track lists and navigation. References stay opaque to the host.

I can supply optional `capabilities.navigation` titles in `initialize`, for example
`{"discover":"Music","library":"Collection","albums":"Albums"}`. The stable
section IDs and routes do not change. Missing or empty titles keep the host's
defaults. I supply titles in the requested locale.

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

A listing returns `items` and an optional `next` reference instead of `shelves`. The host preserves each page in navigation history. I use stable section IDs so the listener's visibility, order and size preferences survive content changes.

Each item uses the same shape as `library.list`: `kind` (`album`, `artist`, `label`, `playlist` or `track`), `ref`, `title`, `image`, `artist`, `artist_ref`, `album_ref`, `year`, `duration_ms`, `genre`, `label`, `label_ref`, `release_date` and `quality` are optional except the item's identity. Track references may include playback context; I interpret and validate them in the plugin, including favorite and resolve operations. The host passes the complete reference unchanged.

I can add a `header` with item identity, image and description to an artist, label or spotlight page. A shelf can supply `extra_tracks` alongside its cards. Playlist shelves can supply `tags` (`id`, `slug`, `name`); each playlist carries its matching tags. The host filters these without replacing playlist identity.

For catalog quality I supply `quality: {"codec":"flac","tier":"hi_res"}`
when the provider advertises high-resolution lossless audio but omits the exact
resolution. I only include `bits` and `rate` when the provider supplies those
numbers. The host shows the advertised tier without inventing a sample rate;
the decoded stream's measured format takes precedence during playback.

Discover documents can include `tab`, `genres` (`id`, `name`) and `genre_filter`. The native Discover convention uses `discover:<tab>:<comma-separated-genre-ids>`. Preferences are isolated by plugin and tab. Ordinary pages need not implement that convention.

A document can set `personal: true` when its content depends on listening history. After the plugin acknowledges a measured playback start, I invalidate only its personal pages. Ordinary catalog pages remain cached.

Album metadata can include `credits`, `booklets` (`name`, `url`), `copyright`, `upc`, `description` and related `shelves`. I keep navigation references on the album and its tracks so artist and album context links stay within the source. Booklet URLs are opened only when the listener clicks them.

`page.action` performs explicit listener actions:

```json
{"action":"favorite","ref":"album:42","on":true}
{"action":"pin","item":{"kind":"album","ref":"album:42","title":"Record"},"on":true}
```

A successful action invalidates the relevant source library and personal pages. Errors remain visible in the source status. A plugin owns its pins and account data; uninstall removes them with its private data directory.

Playback history must come from the audible playback lifecycle, never from a catalog fetch or preloading. I use the [playback events contract](playback-events.md), keep each playback ID once, and associate history with the account that resolved the stream.

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
peak is an optional positive finite linear amplitude. The plugin maps its
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

I advertise `capabilities.playlist_edit: true` when a source implements the
playlist actions below through `page.action`. The host shows creation on its
playlist library tab. Playlist cards carry `editable`, `copyable`, `public`
and `description`; I only set `editable` for the current account's own lists.
The plugin checks ownership again before every write.

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
per-entry editing without hiding the row. I preserve the playlist listening
context when resolving a ref that also carries membership identity.

Successful writes return `changed_refs`, an array of affected playlist refs.
The host invalidates those cards' tracks, its native page documents and
library listings. A search does not invalidate them. The facade exposes
pending/completed/error state per source and echoes a UI `request_id`;
controls wait for completion, and an error leaves the editor open. Removing
a plugin clears this transient action state and late replies are discarded.

I do not automatically retry writes: a timed-out request may have reached the
service. Copying a large list can create the destination before a later
track batch fails; the error identifies that partial list for inspection.

## Shared playlists and folders

I advertise `playlist_edit: true` and `library: ["playlists"]` to expose source-owned
playlists in the shared picker. Each playlist carries `editable` for the current
account. Source-owned playlists accept only references belonging to that source;
local playlists can retain tracks from several sources. I keep the source identity
and metadata when a source is disabled, and unavailable tracks stay in place.

`playlist.create` returns `item.ref` or `ref` naming the created playlist. The host
can then call `playlist.add` in batches, waiting for each acknowledgement. A partial
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

The plugin handles `page.action` with `folder.create` (`title`, `parent`),
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

The host limits concurrent requests to two, backs off after failures, and
updates the shared track cache, album header and playlist rows without playback.
Catalog refreshes with incomplete quality retain known technical values. Actual
decoder measurements can still correct them. The album header uses the highest
known track resolution while the list preserves each track's individual format.
