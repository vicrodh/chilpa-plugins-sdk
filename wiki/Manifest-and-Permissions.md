# Manifest and permissions

`manifest.json` goes next to the plugin binary. The host validates all of it
before loading; an unknown capability type is an error, not something to
skip. This small WASM catalog asks for no network, storage or audio:

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

Required: `id`, `name`, `version`, `api_version` and `kind`.

- `id`: up to 128 characters of lowercase ASCII letters, digits, dots and
  dashes. It needs at least one dot, and no dot at the start, at the end or
  doubled.
- `kind`: `source`, `metadata`, `scrobbler`, `analyzer` or `dsp`. Current
  hosts run an analyzer only under Chilpa's own Mood plugin ID, and a `dsp`
  plugin also needs `process_audio`.
- `runtime`: `wasm` by default; a native package sets `process`.
- `description`: say what the source is and whether it needs an account.
  The listener reads it before approving.
- `delivers_audio`: true by default. Set it to false if the plugin cannot
  resolve playable audio.

| Permission in `capabilities` | Request it when |
| --- | --- |
| `network` with `domains` | WASM: a host request goes to a declared host or one of its subdomains. Process: any declared domain gives the process direct network access, and the list limits which hosts Chilpa downloads its artwork from. |
| `storage` | A WASM plugin uses `store_get`, `store_set` or `cache::get`. |
| `register_panel` | The plugin gets an entry in the sidebar. |
| `register_source`, `register_command` | Shown at approval; they gate nothing for WASM or process plugins in current hosts. |
| `library_read`, `plugins_read`, `playlists_write` | A process plugin uses the matching host broker operation. |
| `folder_write` | A process plugin writes into a folder the listener picked for it. |
| `own_account` | A process plugin handles its own login and credential. |

`domains` are host names, not URLs or wildcards. The host also knows
`guest_credentials` and a `*` domain, both for its compatibility bridge;
neither is available to a new plugin.

Ask only for what the code uses. The install approval records the runtime,
kind, auth configuration and permissions. Adding or changing a permission
(a domain list counts as one permission), or changing kind, runtime or
`auth`, needs a new approval; removing a permission does not, and a version
bump grants nothing. `own_account` is a declaration,
not an enforced permission: the plugin says how it handles the credential,
and Chilpa cannot see what a separate process does with it.

`api_version` is 1 and the process `protocol` is 1. Neither comes from the
SDK's Git tags. Before changing a pin or a package, read
[Compatibility and testing](Compatibility-and-Testing.md).
