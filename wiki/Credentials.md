# Credentials

Where a plugin keeps an account credential depends on who runs the sign-in.
Use the first option that fits the service.

## Chilpa keeps it: `auth`

A WASM plugin whose service takes an API key or OAuth 2.0 declares `auth` in
its manifest. The plugin never holds the credential.

```json
"auth": {"type": "token"}
```

```json
"auth": {
  "type": "oauth2",
  "authorize_url": "https://accounts.example.org/authorize",
  "token_url": "https://accounts.example.org/token",
  "client_id": "public-client-id",
  "scopes": ["read"]
}
```

- `token`: the listener pastes a key or a long-lived token into Chilpa. It is
  sent as the `Authorization` header exactly as pasted.
- `oauth2`: Chilpa runs the authorization code flow with PKCE and a loopback
  redirect, and sends `Authorization: Bearer <access token>`. There is no
  `client_secret`, because a secret shipped inside a distributed binary is not
  secret.
  Both URLs must be `https` or loopback, or the plugin does not install.

Chilpa seals the credential with a key kept in the system keyring (libsecret
on Linux, Keychain on macOS), or with a key derived from the machine when
there is no keyring. The sealed file sits in the plugin's folder, outside its
store, and no host call returns it. The host adds it to the plugin's `http`
requests, renews an expired OAuth token before sending, and deletes it when
the listener disconnects the account, withdraws the plugin's approval or
uninstalls it.

## The plugin keeps it: `own_account`

A process plugin that runs its own sign-in, for example a browser flow or a
session the service only grants its own client, declares it:

```json
{"type": "own_account"}
```

Keep the credential in a file under the `data_dir` that `initialize`
supplies, readable only by the user. On Linux that is `/data`, where `HOME`
also points; on macOS it is a private absolute path, also in
`CHILPA_PLUGIN_DATA_DIR`.
That directory belongs to this plugin alone: on Linux the sandbox mounts only
this plugin's data and cache directories, and on macOS the profile lets it
read and write only those. Uninstalling the plugin deletes the directory and
everything in it. `cache_dir` is for data that can be rebuilt, never for a
credential.

`{"type": "own_account", "keyring": true}` tells the listener the credential
lives in the system keyring, under an entry of the plugin's own. Neither
sandbox opens the keyring service to a plugin today, so declare it only after
checking that it works in the host you target.

At install, Chilpa tells the listener that it cannot see, check or delete a
credential the plugin keeps itself. Say in the README where it lives, and
delete it when the listener signs out.

## The WASM store is not for credentials

`store_set` writes each value as a plain file in the plugin's folder. Use it
for settings and markers, never for a key, token or password. A WASM plugin
that needs an account uses `auth`.

## Rules

- A plugin never reads, requests or probes another plugin's credentials,
  store or data directory, nor Chilpa's keyring entry and sealed files.
  `plugins_read` returns catalog metadata, never credentials. A plugin that
  tries is not listed in Chilpa's catalogs.
- Keep credentials out of logs, standard error, error messages, page
  references, URLs handed to the host, playback reports and anything sent to
  another plugin.
- Send a credential only to the service it belongs to.
