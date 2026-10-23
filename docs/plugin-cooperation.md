# Cooperating native plugins

This contract is for JSON-RPC process plugins; it gives a WASM module no
access to other plugins. Check the target host's broker and permissions
before using an operation. The process transport is in the wiki's
[Native plugins](../wiki/Native-Plugins.md) page.

Plugins reach each other's catalog metadata and playlist creation only
through the host. A plugin never opens another plugin's storage, receives
its account credentials, or calls arbitrary methods on it.

A process manifest declares `plugins_read` for other enabled catalogs, `library_read` for local metadata, and `playlists_write` for import outputs. Installation shows these permissions next to the plugin description. Existing plugins need no new permission to answer their existing `search` and `page.action` methods.

## Transport

The existing JSON-RPC connection carries both directions. The plugin sends a **notification**:

```json
{"jsonrpc":"2.0","method":"host.request","params":{"id":"host-1","method":"sources.list","params":{}}}
```

The host sends a notification back:

```json
{"jsonrpc":"2.0","method":"host.reply","params":{"id":"host-1","result":{"sources":[]}}}
```

An unsuccessful reply has `error` (a string) instead of `result`. Request
IDs belong to the calling plugin and must be strings: the host does not
answer a request with any other `id`, nor one sent before the plugin has
finished `initialize`. Its input reader must keep receiving replies while a
command waits on a host call. Initialization must not depend on host calls:
registration is not complete yet.

The host runs at most 16 broker requests at once, outside the UI and plugin
registry locks, and answers a seventeenth with a busy error. Catalog calls
time out after 30 seconds. A timed-out write is **uncertain**, not
permission to retry: the remote service may already have changed.

| Operation | Permission | Input and result |
| --- | --- | --- |
| `sources.list` | `plugins_read` | Returns the process sources that advertise `search: true`, with `id`, `name`, `writable`, `playlist_limit`. |
| `library.search` | `library_read` | `text` up to 1000 bytes, optional `isrc`, `limit` (1–100, default 20). Returns `tracks` from present, reachable local files only. |
| `catalog.search` | `plugins_read` | `source`, `text` up to 1000 bytes, `limit` (1–50, default 20). The source must be another process source with `search: true`; the host sends it `search` with `text` and `limit`. Returns at most 100 sanitized `tracks`. |
| `playlists.create` | `playlists_write` + UI grant | `grant`, `target`, `title` (1–500 bytes), `description`, `public`, optional local `folder`. For `local`/`mixed`, also ordered `tracks` (1–10000). A remote target needs `playlist_edit`. Returns `ref`, and for `local`/`mixed` `local_id` and `added`; optionally `warning`. |
| `playlists.add` | `playlists_write` + UI grant | `grant`, `target`, `ref`, ordered source track references in `tracks` (1–50). Remote destinations only; returns `added`. |

Track metadata: `kind: "track"`, `source`, stable `ref`, `title`, `artist`, `album`, `album_ref`, `image`, `duration_ms`, `isrc`, `available`, and `quality` (`codec`, `tier`, `bits`, `rate` in Hz). Missing availability does not mean unavailable. Missing quality stays unknown. Audio resolution is not a catalog quality probe.

A mixed local playlist stores stable source references and display metadata, never expiring playback URLs. The existing source availability controls protect playback when a source is disabled. A local-only playlist rejects remote entries. Creating a local playlist and its entries is one transaction.

## Write grants

The host mints a random four-hour grant only when the listener presses **Create playlists** after reviewing a matched import. It belongs to the importer, the selected destinations, and the catalog sources available for mixed results. Plugins cannot mint grants. Additions may only address playlists created under that grant, so an import cannot modify an existing playlist. Cancel, reset, a new grant, a restart of the importer's process, or stopping or uninstalling the plugin revokes the old grant. A request a remote service has already accepted cannot be recalled; the playlists it created stay visible in the summary.

## Importer UI contract

The native greeting advertises `capabilities.playlist_import: 1`, the number,
not `true`. This enables the playlist import screen without adding a music
source to the sidebar. Only one importer is active: the first by plugin ID.
Every `import.*` call has 45 seconds, and the host asks for `import.status`
about twice a second while the screen is open.

- `import.options`: `{"inputs":[...]}`, input descriptors with `id`, `name`, `hint`, `kind` (text/file/choices/station), `private_token` to offer a token field, and `choices` (`id`, `name`) for a station list. The screen selects the input with ID `url` first.
- `import.choices`: `text` and `token`; resolves a public username into `{"choices":[{"id","name"}]}`.
- `import.preview`: starts reading a link or selected file. The screen sends the input's ID as `input`, with `text`, `choice`, `token` and `station`. The host reads only the file the listener selected, up to 16 MiB, and sends `filename` plus base64 `bytes`. Paths referenced inside playlists are not opened.
- `import.match`: `targets` (`local`, `mixed`, enabled source IDs), output `title`, `description`, `public`, `folder`. Starts matching the retained preview.
- `import.status`: optional `filter` (all/unmatched/error) and `offset`. Returns at most 200 ordered rows, the filtered `row_count`, phase, progress, a bounded log, warnings and outputs. Safe to poll while a worker runs.
- `import.commit`: host-issued `grant` and optional output settings. Requires the ready/review phase.
- `import.cancel`: cooperative cancellation; outputs report the writes completed before it.
- `import.reset`: starts another import after the current operation stops.

Phases: input, fetching, preview, matching, ready, creating, adding, done, cancelled. Leaving the screen does not cancel a job. Catalog failures are shown apart from an empty search result. Each output satisfies `matched + duplicates + unmatched = total`; `written` can be lower than matched after a write failure or cancellation. Outputs split at the destination's advertised limit (default 2000) and are added in batches of 50. Writes are never retried automatically.
