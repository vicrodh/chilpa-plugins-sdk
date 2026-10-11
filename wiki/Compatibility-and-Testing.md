# Compatibility and testing

I treat three identifiers separately:

| Identifier | Meaning |
| --- | --- |
| `manifest.api_version: 1` | The package API accepted by the host. |
| Process `initialize.protocol: 1` | The native JSON-RPC contract negotiated at startup. |
| SDK Git tag `protocol-4` | A specific source revision of the WASM API and SDK. |

The host and existing WASM plugins currently pin `protocol-4`. Changes
after that tag adds the directory cache and these documents; I do
not assume a moving branch is compatible with a released host. I compare
the host's `chilpa-plugin-api` pin with the plugin's SDK pin before
shipping. I do not change a public message shape under the same API
version without checking old modules and the host.

## Checks I run

1. I review the manifest against the current host's `Manifest::validate`:
   exact ID, runtime, API version, domain hosts, auth, and permissions.
2. I run native unit tests for parsing, empty and hostile replies, stable
   IDs, pagination, unavailable content, and retry behavior that the plugin
   actually has.
3. For WASM, I build `wasm32-unknown-unknown` in release mode and check
   that `plugin.wasm` and `manifest.json` are in one ID-named package.
4. I install into an isolated Chilpa data home, approve the displayed
   permissions, and exercise every declared operation in the host. Native
   Rust tests alone do not prove that the WASM ABI or process protocol loads.
5. I repeat the host smoke with the plugin disabled, after replacement,
   and after restart when the plugin has persistent state or an account.

The SDK's `fixture` exercises WASM search, browse, cursors, columns,
unavailable entries, and resolution. The SDK's native fake host records
network, clock, store, and log behavior. The example `hello-source`
intentionally has no stream, so it is a packaging and catalog example,
not an audio playback test.

## Runtime boundaries

For WASM I expect a fresh instance per operation. I persist only through
the declared store; I do not rely on static memory. I cap and sanitize
listings, handle a repeated cursor as an end, and avoid treating a
non-2xx HTTP status as a transport failure. The API accepts at most 500
entities of each kind per reply, 24 facts per entry, and four root columns.
The host rejects undeclared HTTP domains and store writes without
`storage`.

For process plugins I answer requests by ID, keep reading while work is in
flight, bound my own service calls, and make startup independent of broker
calls. I verify the negotiated capability before relying on optional host
behavior. A broker write that times out has an uncertain result; I inspect
the destination before retrying. I do not advertise process support on a
platform where the host has no process sandbox.

I keep a plugin's own tests in its repository and use an isolated profile
for host integration. A package is ready when the exact built artifact
loads, permissions match the manifest, and the advertised paths work.
