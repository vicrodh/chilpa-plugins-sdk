# Native plugin playback events, version 1

An optional extension for native process plugins on JSON-RPC protocol 1. The
host sends playback facts. Provider-specific reporting, authentication,
request bodies, thresholds and retries stay inside each plugin.

**Status:** the host implements version 1 for local queue playback through
native process sources. It keeps resolution tickets through preload, measures
consumed frames with the output's `held()` accounting, and delivers requests
through a persistent background queue. Casting uses the advancing positions
the receiver reports, with the limits described below. The extension adds no
WASM export and leaves the WASM ABI as it is.

## Negotiation and stream identity

A plugin advertises support in its `initialize` result:

```json
{"protocol":1,"capabilities":{"playback_events":1}}
```

A supporting host includes `playback_events: 1` in its `initialize` parameters.
Without that host capability, the plugin must not infer playback from requests
for metadata, URL resolution, a preload, a download or network activity.

A successful `track.resolve` result includes an opaque `resolution_id`:

```json
{"url":"https://audio.example/stream","resolution_id":"opaque-ticket","delivery":"direct","preview":false}
```

The host retains that ticket with the exact resolved resource, including a
resource resolved ahead of time. It does not substitute the last ticket for the
same track. The plugin associates the ticket with the actual returned format,
account and any provider reporting material. Tokens and reporting blobs stay
inside the plugin. The host must not log the stream URL or ticket.

A resolution ticket can be reused while its resolved resource is valid. A
`playback_id` cannot: the host creates a new UUID for every audible occurrence,
including repeat-one, replay, and consecutive copies of the same track.

## Delivery and acknowledgement

The host sends `playback.event` requests on a worker queue, never synchronously
on the audio or GUI thread. It preserves order per playback occurrence and
retries an unacknowledged request with the same playback ID and sequence.
A JSON-RPC request ID identifies one transport call; it is not a playback ID.

```json
{"jsonrpc":"2.0","id":41,"method":"playback.event","params":{"kind":"started","playback_id":"48058762-0bde-4c19-82c3-748ea8a1d354","resolution_id":"opaque-ticket","started_at":1790956800}}
```

```json
{"jsonrpc":"2.0","id":41,"result":{"accepted":true}}
```

`accepted` means the plugin has accepted the event into durable local state.
It does not mean an upstream service has accepted a report or counted a listen.
A plugin can cancel an obsolete occurrence explicitly:

```json
{"accepted":false,"discard":true}
```

This rejects the entire occurrence, including its queued checkpoints. Use it
for signed-out accounts, tickets removed on sign-out, or a ticket that cannot
be reconciled after restart. It is not a delivery acknowledgement. A timeout,
RPC error or any other response remains retryable; the host never changes the
ticket to make a rejected start work. Persist resolution tickets with their
account binding if delayed starts should survive a plugin restart.

Plugins may also accept ordered notifications, but notifications have no
acknowledgement and cannot support reliable delivery on their own.

The host bounds its queue and retains unacknowledged terminal events across
restart. Backpressure, an unavailable plugin or a reporting failure must not
stop audio. Unknown `resolution_id` requires reconciliation, not a fabricated
start. Never resolve a replacement URL and report its ticket as though it were
the resource already played. Report delivery must not resume under a different
account. Sign-out cancels account-specific pending work according to the
plugin's privacy policy.

## Events

| Kind | Required fields besides `kind` | Meaning |
| --- | --- | --- |
| `started` | `playback_id`, `resolution_id` | The first audio from this occurrence reached the active output. |
| `progress` | `playback_id`, `sequence`, `played_ms` | A checkpoint of cumulative audio actually reproduced. |
| `ended` | `playback_id`, `sequence`, `played_ms` | The occurrence ended, including interruption or failure after starting. |

`started_at`, when supplied on `started`, is UTC Unix seconds at the actual
start. Otherwise the plugin uses receipt time. Supply it when queuing or
retrying delayed events. `sequence` is a positive, strictly increasing integer
per occurrence; start reserves sequence zero. `played_ms` is a nonnegative
integer in milliseconds and must never decrease within an occurrence.

Optional diagnostic fields on `progress` are `state` (`playing`, `paused`,
`buffering` or `seeking`) and `position_ms`. Neither contributes to listening
time. Optional `reason` on `ended` is `finished`, `stopped`, `skipped`, `replaced`,
`error` or `shutdown`. Plugins must tolerate unknown optional fields.

```json
{"kind":"progress","playback_id":"48058762-0bde-4c19-82c3-748ea8a1d354","sequence":1,"played_ms":12000,"position_ms":180000,"state":"paused"}
```

```json
{"kind":"ended","playback_id":"48058762-0bde-4c19-82c3-748ea8a1d354","sequence":2,"played_ms":12500,"reason":"skipped"}
```

In this example only 12.5 seconds were reproduced, even though a seek moved
the cursor to three minutes. The plugin determines whether and how to round
that duration for its provider.

## Host measurement rules

- Emit start after first output consumption, not on pressing Play, opening a
  decoder, filling a buffer, downloading bytes or resolving a URL.
- Measure `played_ms` from rendered media frames and their sample rate. Do not
  derive it from wall time or subtract two seek positions. Apply the equivalent
  confirmed measurement for another output; do not invent it when unavailable.
- Pause, buffering and seek time add zero. A pause/resume keeps the same
  `playback_id`; replay after an ended occurrence gets a new one.
- A seek changes position, not accumulated listening time. Resuming at an old
  saved position starts accumulated listening time at zero.
- Send a checkpoint at least every five seconds while playing and on pause,
  seek and buffering transitions. Always send a final measured checkpoint in
  `ended`. Faster callbacks may be coalesced; terminal events may not.
- On gapless transitions, end the old occurrence and start the new one at their
  actual output boundaries. Identify each occurrence independently.
- A stream that fails before any audio reaches the output has no start or end
  to report. A failure after audible playback ends with its confirmed time.
- On orderly shutdown, enqueue final events before stopping plugins. After a
  crash, recover only confirmed checkpoints, without counting offline wall time.
- Preserve the resource's `preview` flag. The plugin decides whether previews
  qualify; it must not report a preview as a full catalog stream.

## Plugin processing rules

The plugin persists accepted events before acknowledging them and performs
upstream HTTP on a separate worker. It deduplicates start/end by `playback_id`
and checkpoints by sequence. Out-of-order or duplicate checkpoints must never
increase counted duration. Events from another account must never be mixed.

Local duplicate suppression is not an upstream exactly-once guarantee. If an
HTTP response is lost after the service accepts a report, retrying can submit
it again unless that service provides an idempotency mechanism. Preserve the
original reporting material and timestamp; do not create a new event identity
on every retry.

Plugins should expose redacted reporting diagnostics: pending count, failures
and whether the host negotiated event support. Never expose auth tokens,
provider blobs or signed audio URLs in diagnostics. A reporting failure remains
separate from playback success.

## Compatibility checks

Before a host advertises version 1, verify preload without play, first output,
pause/resume, buffering, forward/backward seek, skip, repeat-one, gapless,
pre-start failure, post-start failure, output changes, logout, shutdown,
process restart and duplicate delivery. Verify that provider HTTP does not run
on an audio or UI thread. Test the event contract with a recording plugin;
provider-specific API tests belong to that plugin.

## Current host implementation

The host keeps up to 1,024 pending occurrences in a private, atomically
replaced `native-playback-events.json` under its data directory. The file contains
plugin IDs, opaque tickets and playback facts, never stream URLs or provider
credentials. The audio thread only updates bounded memory. A worker persists
requests before delivery, requires `accepted: true`, and retries the exact
unacknowledged request after five seconds. Later progress may be coalesced;
starts and final events remain ordered per occurrence. When admission is full,
reporting yields to audio; existing pending terminal events are retained.

A live occurrence checkpoints each additional second of measured audio and on
observed pause/seek transitions. Updates depend on the render loop sampling
the output; a blocked decoder or a backend drain can delay delivery until the
next measurement. No consumption is inferred while that measurement is missing.
Backend frame accounting is the measurement boundary, not a claim that a
physical speaker was audible. An output backend must report its held frames
accurately; a sink that consumes writes immediately may leave `held()` at zero.

After restart, an unfinished saved occurrence ends with its last confirmed
checkpoint and `reason: shutdown`. An orderly shutdown stops and joins the
player before waiting up to three seconds for local persistence. Upstream
provider delivery remains the plugin's responsibility. A crash can lose samples
not yet checkpointed; it never adds offline wall time. If saved state is corrupt,
reporting stays disabled and preserves that file for recovery.

The current producer uses `finished`, `skipped`, `stopped`, `replaced` and
`error`; a normal host shutdown may end at the output as `stopped`. Buffering
and cursor position are optional diagnostics, not required counters. Providers
must not depend on those optional fields to compute listening duration.

## Removing a source

When the listener uninstalls a source, the host stops its process, cancels
its pending playback reports, and removes its account data, private cache,
integration preferences and queued tracks locally. The listener's account and
playlists at the remote service are not touched.

## Receiver playback measurements

For casting, the host keeps the resolved resource's observer for the whole
receiver session. Loading a URL and serving byte ranges emit no playback
event. Reporting starts only when the receiver's status shows the position
moving forward after a playing state. Each delta is capped by the elapsed
polling interval and large jumps are discarded; a successful seek resets the
position baseline. Paused or buffering
intervals do not advance the counter. A missing position clears the baseline.

These are receiver-reported measurements, not local device frame accounting
or proof that a physical speaker was audible. Missing or coarse status updates
can undercount. A receiver that supplies no advancing positions produces no
reports; downloaded bytes and elapsed wall time never stand in for it. Natural
completion, stop, replacement, disconnect and failure retain only confirmed
checkpoints and terminate each occurrence once.
