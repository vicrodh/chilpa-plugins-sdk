# Compatibility and testing

Three numbers that are easy to mix up:

| Identifier | Meaning |
| --- | --- |
| `manifest.api_version: 1` | The package API the host accepts. |
| Process `initialize.protocol: 1` | The native JSON-RPC contract, negotiated at startup. |
| SDK Git tag `protocol-4` | One source revision of the WASM API and SDK. |

The host and the existing WASM plugins pin `protocol-4`. What came after
that tag is the directory cache and these pages. A moving branch is not
assumed compatible with a released host. Before shipping, compare the host's
`chilpa-plugin-api` pin with the plugin's SDK pin. A public message shape
never changes under the same API version without testing old modules and the
host.

## Release checks

1. Check the manifest against the current host's `Manifest::validate`: exact
   ID, runtime, API version, domain hosts, auth and permissions.
2. Run native unit tests for parsing, empty and hostile replies, stable IDs,
   pagination, unavailable content, and whatever retry behavior the plugin
   has.
3. For WASM, build `wasm32-unknown-unknown` in release mode and check that
   `plugin.wasm` and `manifest.json` sit in one directory named after the ID.
4. Install into an isolated Chilpa data home, approve the permissions shown,
   and run every declared operation in the host. Native Rust tests alone
   don't prove the WASM ABI or the process protocol loads.
5. Repeat the host test with the plugin disabled, after replacing it, and
   after a restart if it keeps state or an account.

The SDK's `fixture` exercises WASM search, browse, cursors, columns,
unavailable entries and resolution. The native fake host records network,
clock, store and log calls. `hello-source` has no stream on purpose: it
tests packaging and catalogs, not audio playback.

## Runtime limits

**WASM.** Each operation gets a fresh instance, so nothing survives in
static memory; persist through the declared store only. Cap and sanitize
listings, treat a repeated cursor as the end, and remember that a non-2xx
HTTP status is a response, not a transport failure. A reply carries at most
500 entities of each kind, 24 facts per entry and four root columns. The
host rejects undeclared HTTP domains and store writes without `storage`.

**Process.** Answer requests by ID, keep reading while work is in flight,
put limits on your own service calls, and don't make startup depend on
broker calls. Check the negotiated capability before relying on optional
host behavior. A broker write that times out has an unknown result: look at
the destination before retrying. Don't advertise process support on a
platform where the host has no process sandbox.

Keep the plugin's own tests in its repository and use an isolated profile
for host tests. A package is ready when the exact built artifact loads, its
permissions match the manifest, and every advertised path works.
