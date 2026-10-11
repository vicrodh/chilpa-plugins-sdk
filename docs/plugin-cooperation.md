# Cooperating native plugins

This contract is for JSON-RPC process plugins. It does not give a WASM module
access to other plugins. I check the target host's broker and permissions
before using an operation. The wiki's `Native-Plugins.md` explains the
process transport.

I expose catalog metadata and playlist creation through the host. A plugin never opens another plugin's storage, receives its account credentials, or invokes arbitrary peer methods.

A process manifest declares `plugins_read` for other enabled catalogs, `library_read` for local metadata, and `playlists_write` for import outputs. Installation shows these permissions alongside the plugin description. Existing plugins need no new permission to answer their existing `search` and `page.action` methods.

## Transport

I use the existing JSON-RPC connection in both directions. The plugin sends a **notification**:

```json
{"jsonrpc":"2.0","method":"host.request","params":{"id":"host-1","method":"sources.list","params":{}}}
```

The host sends a notification back:

```json
{"jsonrpc":"2.0","method":"host.reply","params":{"id":"host-1","result":{"sources":[]}}}
```

An unsuccessful reply has `error` (a string) instead of `result`. Request IDs belong to the calling plugin. Its input reader must keep receiving replies while a command awaits a host call. Initialization must not depend on host calls: registration is not complete yet.

The host runs at most 16 broker requests concurrently, outside the UI and plugin registry locks. Catalog calls time out after 30 seconds. A timed-out write is **uncertain**, not permission to retry: the remote service may already have changed.

| Operation | Permission | Input and result |
| --- | --- | --- |
| `sources.list` | `plugins_read` | Returns enabled searchable sources with `id`, `name`, `writable`, `playlist_limit`. |
| `library.search` | `library_read` | `text`, optional `isrc`, `limit` (1–100). Returns `tracks` from present, reachable local files only. |
| `catalog.search` | `plugins_read` | `source`, `text`, `limit` (1–50). Returns at most 100 sanitized `tracks`; the source must be enabled and searchable. |
| `playlists.create` | `playlists_write` + UI grant | `grant`, `target`, `title`, `description`, `public`, optional local `folder`. For `local`/`mixed`, also ordered `tracks` (1–10000). Returns `ref`, optionally `local_id`, `warning`. |
| `playlists.add` | `playlists_write` + UI grant | `grant`, `target`, `ref`, ordered source track references in `tracks` (1–50). Remote destinations only; returns `added`. |

Track metadata: `kind: "track"`, `source`, stable `ref`, `title`, `artist`, `album`, `album_ref`, `image`, `duration_ms`, `isrc`, `available`, and `quality` (`codec`, `tier`, `bits`, `rate` in Hz). Missing availability does not mean unavailable. Missing quality remains unknown. Audio resolution is not a catalog quality probe.

For a mixed local playlist, I persist stable source references and display metadata, never expiring playback URLs. Existing source availability controls protect playback when a source is disabled. A local-only playlist rejects remote entries. Local creation and its entries share one transaction.

## Write grants

I mint a random, four-hour grant only when the person presses **Create playlists** after reviewing a matched import. It belongs to the importer, the selected destinations, and the catalog sources available for mixed results. Plugins cannot mint grants. Additions may only address playlists created by that grant, so an import cannot modify an existing playlist. Cancel, a new grant, or stopping/uninstalling the plugin revokes the old grant. A request already accepted by a remote service cannot be recalled; created playlists remain visible in the summary.

## Importer UI contract

The native greeting advertises `capabilities.playlist_import: 1`. This enables the playlist import screen without adding a music source to the sidebar.

- `import.options`: input descriptors (`id`, `name`, `hint`, `kind`: text/file/choices/station).
- `import.choices`: resolves a public username into selectable lists.
- `import.preview`: starts reading a link or selected file. The host reads only the explicitly selected local file, bounded to 16 MiB, and sends `filename` plus base64 `bytes`. Paths referenced inside playlists are not opened.
- `import.match`: `targets` (`local`, `mixed`, enabled source IDs), output `title`, `description`, `public`, `folder`. Starts matching the retained preview.
- `import.status`: optional `filter` (all/unmatched/error) and `offset`. Returns at most 200 ordered rows, total filtered `row_count`, phase, progress, bounded log, warnings, and outputs. It is safe to poll while a worker is running.
- `import.commit`: host-issued `grant` and optional output settings. Requires the ready/review phase.
- `import.cancel`: cooperative cancellation; retained outputs report writes completed before cancellation.
- `import.reset`: starts another import after the current operation stops.

Phases: input, fetching, preview, matching, ready, creating, adding, done, cancelled. Leaving the screen does not cancel a job. Catalog failures are displayed separately from an empty search result. Each output satisfies `matched + duplicates + unmatched = total`; `written` can be lower than matched after a write failure or cancellation. Parts split at the destination's advertised limit (default 2000), with batches of 50 additions. Writes are never automatically retried.
