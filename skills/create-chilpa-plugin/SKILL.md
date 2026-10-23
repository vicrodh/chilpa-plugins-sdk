---
name: create-chilpa-plugin
description: Create or update a Chilpa plugin with a valid manifest, the matching WASM or process contract, permission boundaries, and host integration checks. Use for plugin authoring, not for unrelated Chilpa app changes.
---

# Create a Chilpa plugin

Build the requested plugin in its own `chilpa-pg-<name>` repository or the
location the user chose. Ask what the plugin should do only when the
available code and request do not answer it. Follow that repository's agent
instructions. Keep public code and documentation in English and avoid
provider names in shared host code when the project rules require it.

## Start from a short request

A request such as "make a Chilpa plugin for <service>" is enough to begin.
Find the service's current official API and terms, inspect the target host's
implemented contracts, and write a small feature plan. Choose the smallest
useful source: discovery and browsing first; add playable audio only when a
supported stream can actually be resolved. Prefer public endpoints and
account-free paths when they satisfy the request. If an account is required,
decide where its credential lives, following the SDK's `wiki/Credentials.md`,
before coding. When a necessary service capability or target host contract
is unknown, research it and state the remaining uncertainty; ask only for a
product choice that cannot be inferred. Do not claim that merely having a
catalog API grants streaming rights.

## Choose the interface

- Read [references/wasm.md](references/wasm.md) for a plugin that can use
  host-mediated HTTP, storage, clock and logging. Start from the SDK's
  `examples/hello-source` for a catalog-only source or `fixture` for
  search, browsing and resolution. For a real account-free source, inspect
  the public Radio plugin linked in that reference; adapt its contract and
  tests, not its service-specific IDs or endpoints.
- Read [references/process.md](references/process.md) when native code, a
  separate process, or self-managed account credentials are necessary.
  Current hosts start process plugins inside Bubblewrap on Linux or Seatbelt
  on macOS, and not on Windows. Build and test a separate package for each
  advertised OS and architecture.

With a `chilpa-plugins-sdk` checkout, read its `wiki/` page for the selected
runtime, `wiki/Manifest-and-Permissions.md`, `wiki/Credentials.md` and
`wiki/Packages-and-Catalog.md`, and the `docs/` contract for each process
feature. The pages record the host revision they were checked against.

**Check the target host's code before advertising a capability or relying
on a method.** Read where the host reads that capability and where it calls
the method, at the host revision you target, and match its types and time
limits. Do not turn a design document or an SDK page into an advertised
capability without that evidence.

## Guardrails

1. Pin the plugin to a reviewed immutable SDK commit or tag and check the
   target host's pin. Keep manifest `api_version`, process `protocol`, and
   Git tag names distinct.
2. Request only manifest permissions the implementation uses, and check
   them against the target host's manifest validator. WASM domains also
   allow their subdomains; a process plugin with any domain gets direct
   network access. Do not add wildcard network access.
3. Preserve stable opaque IDs from discovery through later operations and
   follow the host's reference conventions in `docs/native-pages.md`. Mark
   audio unavailable when a stream cannot actually be resolved through the
   supported delivery path.
4. Keep credentials, stream URLs, resolution tickets and private data out of
   logs and test output. Never keep a secret in the WASM store. A process
   plugin with `own_account` says where it keeps the credential; neither
   sandbox opens the system keyring today.
5. Test the implemented behavior with malformed and empty replies,
   permission denial, timeouts, and restart or replacement where relevant.
   Compile the target artifact and load that exact package in an isolated
   Chilpa profile. Report any integration check that could not run.

If the plugin will be listed in the third-party catalog, follow the rules in
https://github.com/vicrodh/chilpa-3rd-party-plugins/blob/main/CONTRIBUTING.md
and check its entry against `wiki/Packages-and-Catalog.md`.

Produce the plugin, its manifest, meaningful tests, and build or package
instructions. Report the selected runtime, requested permissions, SDK and
host pins, test results, and remaining limitations. Publishing, installing
into a real user profile, or changing live accounts requires authorization
from the task or session; this skill grants none.
