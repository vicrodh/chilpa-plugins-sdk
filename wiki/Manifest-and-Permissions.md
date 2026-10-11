# Manifest and permissions

I put `manifest.json` next to the plugin binary. The host validates the
whole manifest before loading it; unknown capability types fail instead of
being ignored. This small WASM catalog example has no network, storage, or
audio permission:

```json
{
  "id": "app.example.hello",
  "name": "Hello Source",
  "version": "0.1.0",
  "api_version": 1,
  "kind": "source",
  "delivers_audio": false,
  "capabilities": [
    {"type": "register_source"},
    {"type": "register_panel"}
  ]
}
```

The required fields are `id`, `name`, `version`, `api_version`, and
`kind`. The ID is at most 128 characters, lowercase ASCII letters, digits,
dots and dashes, includes a dot, and has no leading, trailing, or doubled
dot. `kind` is `source`, `metadata`, or `scrobbler`. `runtime` defaults
to `wasm`; a native package explicitly sets `process`. I use
`description` to explain the source and its account needs before approval.
`delivers_audio` defaults to true, so I set it to false if I cannot
resolve playable audio.

| Permission in `capabilities` | I request it when |
| --- | --- |
| `network` with `domains` | A WASM host request or package artwork needs those exact hosts. |
| `storage` | A WASM plugin uses `store_get`, `store_set`, or `cache::get`. |
| `register_source`, `register_panel`, `register_command` | The plugin contributes those surfaces. |
| `library_read`, `plugins_read`, `playlists_write` | A process plugin uses the corresponding host broker operation. |
| `folder_write` | A process plugin writes into a folder explicitly selected for it. |
| `own_account` | A process plugin handles its own authentication and credential. |

The host also recognizes `guest_credentials` for its compatibility bridge;
it is not a general WASM or process plugin permission. Domain permissions
are host names, not URLs or wildcards. The bridge has its own explicit
exception for `*`; I do not apply that exception to a new plugin.

I request only what the implementation uses. The installation approval
records the runtime, kind, auth configuration, and permissions. A later
change to those requests needs fresh approval. A version bump alone does not
grant new access. `own_account` is different from enforced permissions:
the plugin declares how it handles the credential, and Chilpa cannot inspect
what a separate process does with it.

The manifest `api_version` is currently 1. The process `initialize`
`protocol` is also 1. Neither number is inferred from SDK Git tags. I
read [Compatibility and testing](Compatibility-and-Testing.md) before
changing a pin or a package.
