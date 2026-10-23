# Native source methods

The methods a process source answers, with the shapes the current host sends
and reads. It ignores fields it does not know. The transport, `initialize`,
capabilities and time limits are in the wiki's
[Native plugins](../wiki/Native-Plugins.md) page; pages, playlist actions and
quality are in [native pages](native-pages.md).

A failed call answers a JSON-RPC error with a short `message`: the host may
show it to the listener. A method you do not implement answers `-32601`.

## Items

Every list below carries items. An item has a `kind` (`album`, `artist`,
`label`, `playlist` or `track`) and a non-empty `ref`. The host skips items
without `ref` and relies on `kind` to tell a track from a card, so send both.
The optional fields it reads:

| Field | On | Meaning |
| --- | --- | --- |
| `title`, `artist`, `album` | any | Display text. |
| `image` | any | Absolute `http` or `https` URL. The host downloads it only from a host named in the manifest's `network` permission, or one of its subdomains, including every redirect. |
| `artist_ref`, `album_ref`, `label_ref` | any | References for navigation inside the source. |
| `year`, `genre`, `label`, `release_date` | albums | Card and header details. |
| `explicit` | albums, tracks | Used by the explicit filter. |
| `duration_ms`, `disc_number`, `track_number`, `isrc` | tracks | Track details. |
| `available` | any | `false` only when the source knows the item cannot play. Absent means unknown. |
| `quality` | albums, tracks | `codec`, `bits`, `rate` in Hz, and `tier: "hi_res"`. |
| `editable`, `copyable`, `public`, `description`, `tracks`, `updated_at` | playlists | Ownership, editor state, track count and last change. A changed `tracks`, `updated_at` or `editable` makes the host read the playlist again. |
| `member` | tracks of a playlist | The occurrence, for editing. |

A track listed under an album takes the album's artist, title, image and
`artist_ref` when it has none of its own.

## `search`

```json
{"jsonrpc":"2.0","id":7,"method":"search","params":{"text":"night drive","limit":20,"filters":{"quality":""}}}
{"jsonrpc":"2.0","id":7,"result":{
  "top":{"kind":"album","ref":"album:12","title":"Night Drive","artist":"Example Band"},
  "artists":[{"kind":"artist","ref":"artist:3","title":"Example Band"}],
  "albums":[{"kind":"album","ref":"album:12","title":"Night Drive","artist":"Example Band","year":2019}],
  "tracks":[{"kind":"track","ref":"track:90","title":"Night Drive","artist":"Example Band","album":"Night Drive","album_ref":"album:12","duration_ms":241000}],
  "playlists":[]}}
```

`limit` is how many results the host wants at most. `filters.quality` is
empty, `lossless` or `hi_res`; the host applies it to the answer anyway, so a
plugin may ignore it. `top` is optional and marks the source's best match.

Global search sends `search` to every enabled process plugin of kind
`source`, with or without `capabilities.search`. A source that does not
search answers empty lists: an error shows next to the listener's results.
Open music link, the broker, artist scenes and artist lookups by name also
search, with `text` and `limit` only.

## `track.resolve`

```json
{"jsonrpc":"2.0","id":8,"method":"track.resolve","params":{"ref":"track:90","purpose":"play","output":{"max_rate":192000,"max_bits":24}}}
{"jsonrpc":"2.0","id":8,"result":{"url":"https://audio.example.org/90.flac","format":{"codec":"flac"},"resolution_id":"ticket-1","normalization":{"gain_db":-4.2,"peak":0.93}}}
```

The host reads `url`, which must be `http` or `https`; `format.codec`, which
tells the decoder what the bytes are; `resolution_id` for
[playback events](playback-events.md); and `normalization`, described in
[native pages](native-pages.md). It ignores everything else.

The host fetches `url` with plain HTTP range requests: it sends no
credentials and follows no redirects. The address must answer 206 with
`Content-Range`, or 200 with `Content-Length`, and a content type that is not
text, JSON, XML or an image. The host may resolve a track before it plays and
use the address up to two minutes later, then reads it window by window while
the track plays and after every seek, so it has to stay valid all that
time.

## `track.get`

```json
{"jsonrpc":"2.0","id":9,"method":"track.get","params":{"ref":"track:90"}}
{"jsonrpc":"2.0","id":9,"result":{"track":{"kind":"track","ref":"track:90","title":"Night Drive","album":"Night Drive","album_ref":"album:12"}}}
```

The answer is the track item, alone or under `track`, with a non-empty `ref`;
the host keeps the reference it asked with. It asks when a list shows a track
of the source that it has not read yet, such as one saved in a local
playlist, and when a linked track has no `album_ref`. After a failure it
waits a minute before asking for the same track again.

## `album.get` and `playlist.get`

```json
{"jsonrpc":"2.0","id":10,"method":"album.get","params":{"ref":"album:12","offset":0,"limit":10000}}
{"jsonrpc":"2.0","id":10,"result":{
  "album":{"kind":"album","ref":"album:12","title":"Night Drive","artist":"Example Band","artist_ref":"artist:3"},
  "tracks":[{"kind":"track","ref":"track:90","title":"Night Drive","track_number":1,"duration_ms":241000}]}}
```

`playlist.get` is the same with a `playlist` object. Instead of `tracks` the
answer may carry `shelves`, and the host takes the tracks from the shelf with
`kind: "track"`. If the `album` object itself has `shelves`, the host also
keeps it as the album's page. A terminal missing album answers
`{"availability":"unavailable","tracks":[]}`; see
[Availability](native-pages.md#availability). For an artist or label card the
host sends the same parameters to `page.get` and reads the object under
`header`.

## `artist.get`

```json
{"jsonrpc":"2.0","id":11,"method":"artist.get","params":{"ref":"artist:3"}}
{"jsonrpc":"2.0","id":11,"result":{
  "artist":{"kind":"artist","ref":"artist:3","title":"Example Band"},
  "albums":[{"kind":"album","ref":"album:12","title":"Night Drive","artist_ref":"artist:3"}],
  "shelves":[{"title":"Appears on","items":[{"kind":"album","ref":"album:40","title":"Summer Mix"}]}]}}
```

A source without `pages` gets `artist.get` when the listener opens one of its
artist or label cards; the host shows `artist` and the items of `shelves`.
To list an artist's records by name elsewhere in the window, the host
searches the name with `limit` 20, takes the artist whose `title` matches,
and keeps the records from `albums` and from album items in `shelves` whose
`artist_ref` is the artist's `ref`. With `pages`, it reads the artist's page
instead.

## `home`

```json
{"jsonrpc":"2.0","id":12,"method":"home","params":{}}
{"jsonrpc":"2.0","id":12,"result":{"shelves":[{"title":"New releases","items":[{"kind":"album","ref":"album:12","title":"Night Drive"}]}]}}
```

A source without `pages` gets `home` when the listener opens it; each shelf
is a section of its page, in order. With `capabilities.home: true` the same
shelves become the Discover entries in the sidebar. The host reads `home` at
most once every 30 seconds.

## `library.list`

```json
{"jsonrpc":"2.0","id":13,"method":"library.list","params":{"kind":"albums","offset":0,"limit":500,"refresh":false}}
{"jsonrpc":"2.0","id":13,"result":{"items":[{"kind":"album","ref":"album:12","title":"Night Drive","artist":"Example Band"}],"total":1}}
```

The host reads each tab named in `capabilities.library` with its `kind`, 500
items at a time and at most 20 pages. It stops at a page shorter than 500 or
once it has `total` items, so a library larger than one page must give
`total`. `refresh` is true on the first page when the host reads again after
a change or an explicit refresh; skip your own cache for it. The `tracks`
tab is the listener's favorite tracks, and the track hearts follow it. A
`playlists` answer may add `folders`; see
[Native account folders](native-pages.md#native-account-folders).

## `favorites.set`

```json
{"jsonrpc":"2.0","id":14,"method":"favorites.set","params":{"ref":"track:90","on":true}}
{"jsonrpc":"2.0","id":14,"result":{}}
```

The host sends it for track hearts when the plugin advertises
`favorites: true` and lists `"tracks"` in `library`, and only for references
that start with `track:`. Any result counts as done, and the host reads the
`tracks` tab again. Hearts on albums, artists and playlists go through
`page.action` instead.

## `link.resolve` and `link.identify`

Open music link asks `link.resolve` of every source that advertises
`search: true`, all at once:

```json
{"jsonrpc":"2.0","id":15,"method":"link.resolve","params":{"url":"https://music.example.org/album/12"}}
{"jsonrpc":"2.0","id":15,"result":{"kind":"album","ref":"album:12","title":"Night Drive","artist":"Example Band"}}
```

Answer an item with a non-empty `ref` when the link is the plugin's own, and
`{}` when it is not. A track opens its album, from `album_ref` or from
`track.get`.

When no source claims the link, the host asks `link.identify` of every
process plugin, one at a time, importers first, until one answers with a
`kind`:

```json
{"jsonrpc":"2.0","id":16,"method":"link.identify","params":{"url":"https://other.example.net/track/7"}}
{"jsonrpc":"2.0","id":16,"result":{"kind":"track","title":"Night Drive","artist":"Example Band","album":"Night Drive"}}
```

A `playlist` goes to the importer. Anything else the host looks up by title
and artist in the library and in the sources. Because the calls are one at a
time, a plugin that does not identify links answers `-32601` at once.
