# Manifest and permissions

`manifest.json` goes next to the plugin binary. The host validates all of it
before loading. An unknown capability type is an error, not something to
skip; unknown top-level fields are ignored. This small WASM catalog asks for
no network, storage or audio:

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

## Fields

| Field | Required | Rule |
| --- | --- | --- |
| `id` | yes | Up to 128 characters of lowercase ASCII letters, digits, dots and dashes, with at least one dot and no dot at the start, at the end or doubled. It names the plugin's folder. |
| `name` | yes | Not empty. |
| `version` | yes | Not empty. A catalog install needs the index entry's `version`; a local ZIP over an installed plugin needs a newer one. |
| `api_version` | yes | `1`. |
| `kind` | yes | `source`, `metadata`, `scrobbler`, `analyzer` or `dsp`. |
| `runtime` | no | `wasm` by default; a native package sets `process`. |
| `author` | no | Shown in the plugin's details. |
| `description` | no | Say what the source is and whether it needs an account. The listener reads it before approving. |
| `delivers_audio` | no | `true` by default. Set it to `false` if the plugin cannot resolve playable audio; its results then show as not playable. |
| `icon` | no | A path inside the package to an SVG, PNG or JPEG, or the name of a window icon. |
| `folder_icon` | no | A path inside the package to the image for the account's playlist folders. |
| `auth` | no | How a WASM plugin's account connects; see [Credentials](Credentials.md). |
| `capabilities` | no | The permissions below. |

`source` is the kind global search asks and the WASM source operations go
to. A process plugin of any kind still gets the methods its negotiated
capabilities call for, such as a playlist importer. `metadata` and
`scrobbler` are accepted, and current hosts call nothing because of them.
Current hosts run an `analyzer` only under Chilpa's own Mood plugin ID. A
`dsp` plugin supplies processing presets that the host evaluates, and it is
offered only with `process_audio`; its preset format is not documented here
yet.

The host checks and stores `icon` and `folder_icon` images at install. An
SVG must pass the host's SVG check; a PNG or JPEG can be up to 4 MiB and
4096×4096 pixels and is shown at 256×256. An `icon` that is not an image in
the package is read as an icon name; a name the window does not know gets a
generic icon, and no icon gets the plugin's initial.

`auth` is `{"type": "token"}` or `{"type": "oauth2", "authorize_url": ...,
"token_url": ..., "client_id": ..., "scopes": [...]}`. Both URLs must be
`https` or loopback and `client_id` cannot be empty, or the plugin does not
install. Only WASM `http` requests get the credential.

## Permissions

| Permission in `capabilities` | Runtime | Request it when |
| --- | --- | --- |
| `network` with `domains` | both | WASM: a host request goes to a declared host or one of its subdomains. Process: any declared domain gives the process direct network access, and the list limits which hosts Chilpa downloads its artwork from. Offline mode cuts both. |
| `storage` | WASM | The plugin uses `store_get`, `store_set` or `cache::get`. |
| `register_panel` | both | The plugin gets an entry in the sidebar. |
| `register_source`, `register_command` | both | Shown at approval; they gate nothing for WASM or process plugins in current hosts. |
| `library_read`, `plugins_read`, `playlists_write` | process | The plugin uses the matching host broker operation. |
| `folder_write` | process | The plugin writes into a folder the listener picked for it. |
| `own_account` | process | The plugin handles its own login and credential. |
| `process_audio` | `dsp` | The plugin supplies processing presets. |

A domain is a host name made of letters, digits, dots and dashes, with at
least one dot and at most 253 characters: no scheme, port, path or wildcard,
so `localhost` is refused. Current hosts do not check the stream address a
WASM resolver returns against the list. The host also knows
`guest_credentials`, the `*` domain and the `bridge` runtime, all for its
compatibility bridge; none is available to a new plugin.

The WASM store is plain files in the plugin's folder, so never keep a secret
there. Values go up to 1 MiB and keys up to 64 characters of `a-z`, `0-9`,
`.`, `_` and `-`. A process plugin gets its data and cache directories
without `storage`.

`own_account` takes an optional `keyring`. `{"type": "own_account",
"keyring": true}` tells the listener the credential lives in the system
keyring under the plugin's own entry; neither sandbox opens the keyring to a
plugin today. [Credentials](Credentials.md) covers both forms.

Ask only for what the code uses. The install approval records the runtime,
kind, auth configuration and permissions. Adding or changing a permission
(a domain list counts as one permission), or changing kind, runtime or
`auth`, needs a new approval; removing a permission does not, and a version
bump grants nothing. `own_account` is a declaration, not an enforced
permission: the plugin says how it handles the credential, and Chilpa cannot
see what a separate process does with it.

`api_version` is 1 and the process `protocol` is 1. Neither comes from the
SDK's Git tags. Before changing a pin or a package, read
[Compatibility and testing](Compatibility-and-Testing.md).
