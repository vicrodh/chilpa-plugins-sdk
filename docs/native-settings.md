# Native settings and downloads

A process plugin can give the listener a settings form, for an account or
anything else, and a downloads view. The plugin returns documents and the
host draws them; neither adds a WASM export. Check the target host's code
before advertising either. [Credentials](../wiki/Credentials.md) says where
an account credential goes.

## Settings form

With `capabilities.settings: true`, or the older `"auth": "browser"`, the
host calls `settings.open` with `{}` when the listener opens the settings of
the enabled plugin. The answer is the document to draw:

```json
{
  "revision": "7",
  "notices": ["Sign in to play full tracks."],
  "status": {"phase": "ready", "label": "Signed in"},
  "content": [{"text": "Connect the account you listen with."}],
  "form": {
    "fields": [
      {"key": "region", "label": "Region", "control": {"type": "select", "choices": [{"label": "Europe", "value": "eu"}, {"label": "Americas", "value": "am"}]}},
      {"key": "token", "label": "Access token", "control": {"type": "secret"}}
    ],
    "actions": [{"key": "save", "label": "Save", "fields": ["region", "token"]}]
  },
  "values": {"region": "eu"},
  "links": [{"label": "Create an account", "url": "https://accounts.example.org/join"}]
}
```

Every field is optional. `notices` are warning lines and `content` items are
paragraphs. `status.phase` is `pending`, `running`, `passed`, `ready` or
`failed`; `status.label` and `status.description` replace the host's words
for it. `progress` with `done` and `total` draws a bar. A control's `type` is
`text` (the default), `secret`, `number`, `toggle` or `select` with
`choices`, and `values` fills the form. A link with an `http` or `https`
`url` opens in the browser; `primary: true` gives it the main button style.

When the listener presses an action, the host sends `settings.submit` with
the document's `revision`, the action's `key`, and the values of the fields
that action lists. A select sends its value as a string.

```json
{"jsonrpc":"2.0","id":21,"method":"settings.submit","params":{"revision":"7","action":"save","values":{"region":"am","token":"..."}}}
```

The answer is the next document. Both methods have 15 seconds. After a
successful submit the host reads the plugin's library tabs again, and its
first page when it has `pages`. Never send a stored secret back in `values`: the form would show it.

## Sign-in window

A link can ask the host to open a sign-in page in a window of its own and
hand back the address the page ends on:

```json
{"label": "Sign in", "url": "https://accounts.example.org/login", "primary": true,
 "capture": {"until": "https://accounts.example.org/done", "action": "finish", "field": "redirect_url"}}
```

`url` and `until` are `https` URLs of at most 8192 characters. `action` and
`field` are up to 64 letters, digits, `_` or `.`. An optional `user_agent`,
on one line of up to 512 characters, replaces the window's own. When the page
reaches an address that starts with `until`, the window closes and the host
sends `settings.submit` with that action and `{"<field>": "<address>"}` as
`values`. If the listener closes the window, or 30 minutes pass, the host
sends `settings.open` instead. The window keeps its cookies and site data
between sign-ins.

Not every installation has the window. Without it, or after a link with a
malformed `capture`, which also turns the window off until Chilpa restarts,
links open in the browser. Offer another way to finish, such as a field
where the listener pastes the address.

## Downloads view

With `capabilities.downloads: true` the host offers a downloads view for the
enabled plugin. It sends `downloads.open` with `{}` when the listener opens
the view and about once a second while it stays open, and `downloads.action`
with the request the view builds. Both have 30 seconds, and each answer
replaces the document.

The current view is a browser of purchases built around one existing plugin.
It reads `signed_in`, `loading`, `loaded`, `error`, `report`, `working`,
`progress` (`completed`, `total`, `current`), `destination_ready`, `albums`
and `tracks`, whose items carry `id`, `title`, `artist`, `album`, `year`,
`image`, `purchased_at`, `downloadable` and `formats` (`id`, `label`), and
`selected`, the open purchase with its `kind` and `tracks`. It sends
`refresh`, `cancel` and `back`, `select_album` or `select_track` with an
`id`, and `download` with a `format_id` and an optional `track_id`, each as
`{"action": ...}`. Its quality filter knows that plugin's format numbers, so
treat the view as specific to it until the host publishes a general one. A
plugin that writes the files declares `folder_write`, and the listener picks
the folder from the view.
