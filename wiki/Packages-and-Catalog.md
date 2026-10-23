# Packages and catalog

A plugin ships as a ZIP. Chilpa installs it from a catalog the listener
added or from a local file, with the same checks, and never approves it on
install: the plugin waits switched off until the listener reviews its
permissions.

## The package

The ZIP holds exactly one `manifest.json`, at its root or inside one
top-level folder, and beside it the program: `plugin.wasm` for WASM, or an
executable named `plugin` for a process plugin. Other files, such as icons
or libraries, may sit next to them.

The archive can be up to 64 MiB, expand to 128 MiB, hold files of up to
32 MiB each and at most 4096 entries. It cannot contain links or special
files, and its paths must be portable: no `\`, `:` or `<>"|?*`, no part
ending in a dot or a space, no Windows device names such as `CON`, at most
32 levels and 1024 characters, and no two paths that differ only in case.

A catalog install also needs the manifest's `id` and `version` to equal the
catalog entry's. A local ZIP over an installed plugin needs a newer
`version`. The [manifest page](Manifest-and-Permissions.md) has the rest of
the rules.

## Catalog entry

A catalog is a JSON file of at most 1 MiB with up to 5000 entries:

```json
{
  "version": 1,
  "plugins": [{
    "id": "app.example.hello",
    "name": "Hello Source",
    "version": "0.1.0",
    "description": "A small catalog that needs no account.",
    "author": "Example",
    "category": "music",
    "repository": "https://github.com/example/chilpa-pg-hello",
    "package": "https://github.com/example/chilpa-pg-hello/releases/download/v0.1.0/hello-0.1.0-wasm.zip",
    "sha256": "<SHA-256 of the ZIP, 64 hex digits>",
    "api": {"min": 1, "max": 1}
  }]
}
```

`id`, `name`, `repository`, `package`, `sha256` and `api` are required.
`description`, `author`, `category` and `packages` are optional, and so is
`version` in the format, though an install needs it to match the manifest.
Other fields are ignored. `package` is an `http` or `https` URL without
credentials. An entry whose `api` range leaves out 1, or whose `package` or
`sha256` is unusable, stays listed and cannot be installed.

A native process plugin keeps its top-level Linux `package` and `sha256` for
older hosts and adds a platform map:

```json
"packages": {
  "linux-x86_64": {"package": "https://example.org/plugin-linux.zip", "sha256": "<Linux SHA-256>"},
  "macos-aarch64": {"package": "https://example.org/plugin-macos.zip", "sha256": "<macOS SHA-256>"}
}
```

The host picks `{OS}-{ARCH}` with Rust's platform names and verifies that
package's SHA-256. A map without the current platform leaves the plugin out
of the visible catalog and prevents staging it. An omitted map declares a
portable package, such as a WASM ZIP, so a process plugin must list every
supported target in `packages` and never publish a platform-specific binary
as portable.

Plugins by other authors are listed in
[chilpa-3rd-party-plugins](https://github.com/vicrodh/chilpa-3rd-party-plugins),
under the rules in its
[CONTRIBUTING.md](https://github.com/vicrodh/chilpa-3rd-party-plugins/blob/main/CONTRIBUTING.md).
