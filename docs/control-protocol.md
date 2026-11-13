# Command contract

The delivered engine has one typed registry in `bondline-commands`. Names, Rust parameter
types, schemas and discovery metadata originate in the same declaration. The UI and external
adapters must call it; they must not implement their own authoring operations.

An engine JSON request is exactly:

```json
{"command":"page.set_size","params":{"page_id":1,"width_pt":612,"height_pt":792}}
```

Unknown fields, missing parameters, invalid IDs and non-finite/out-of-range dimensions fail
before mutation. Raw JSON is bounded to 1 MiB. Results contain an immutable authored snapshot,
monotonic revision, `changed` flag and undo/redo depths. Errors expose a stable code and message.
Inspection and exact no-ops do not advance revision or discard redo. A changed edit discards
the previous redo branch. Undo/redo restore authored state exactly but advance revision.
History retains up to 128 prior document snapshots, with a 128 MiB accounted-storage
threshold including the current document. Shared source/model/fragment allocations count
once; baselines and reserved vector capacity count too. A changed commit discards redo,
then evicts the oldest Undo prefix until the retained suffix fits. The current document is
always kept, even if it alone exceeds the threshold. Arc/allocator headers, temporary work,
caches and client-held snapshots are excluded; this is not a process-memory ceiling.
These are explicit Bondline policies, not claims about another application's history limits.
Revision overflow rejects the operation before history eviction or mutation.

Page snapshots include a fragments array, empty in the default application. Engine history
now retains an owned StoredDocument, including original CDXML data. An engine initialized
with that API returns an additional source object: {format: "cdxml", byte_length, modified}.
The field is omitted for native in-memory documents. Raw source bytes are not serialized.
Import/export commands are not registered yet. bond.create now authors a limited native
chemical fragment; source-backed creation is refused unless a complete profile is proven.
Exact current native encoding byte equality now permits headless edits/captures after
reopening. Other modified source-backed models cannot export stale XML; exact undo restores
original bytes/exportability.
The source/history budget is delivered headlessly; editable mapping, failed-import retention
and actual native file workflows still need implementation and independent evidence.
The io library now encodes fresh elemental CDXML through StoredDocument::to_cdxml; this is
not yet a registered command or native Save action. No command result gained raw XML bytes.
See model.md for the data boundary; serialized graphs are not a project-save workflow.

| Name | Parameters | Effect |
|---|---|---|
| `document.inspect` | `{}` | Read-only snapshot |
| `page.set_size` | `page_id`, `width_pt`, `height_pt` | One validated page edit |
| `bond.create` | `page_id`, `begin_x_pt`, `begin_y_pt`, `end_x_pt`, `end_y_pt` | One independent C-C fragment; two C atoms with three explicitly authored H each, one single bond |
| `history.undo` | `{}` | Previous authored snapshot |
| `history.redo` | `{}` | Next authored snapshot |
| `render.capture` | `page_id`, `width_px`, `height_px`, `backend` | Read-only page preview PNG; explicit `cpu` or `gpu` |

The delivered CLI exposes `commands` and `run --cmd '<request>'`. Repeat `--cmd` to execute
sequential operations in one in-memory engine; stdout is one final JSON snapshot. Failure
prints a structured error to stderr and exits 2 (I/O/serialization failures exit 1). It does
not save a project or make a multi-command transaction. `--help` lists actual operations.

Native Page presets/rotation, Edit Undo/Redo and history shortcuts call these same commands.
The canvas reads the authored dimensions; no native control edits the model directly.
Native drag creation calls bond.create once at release. Temporary preview does not enter
history. Complete/multiple drags can arrive between frames; ordered raw events preserve each
transaction. Escape, focus loss, stale revisions and outside release cancel a pending drag.
Three logical units of click jitter are ignored. These are independent initial policies.

Bond coordinates use finite ±100000 physical points; endpoints must differ. Page/graph
validation precedes commit. IDs are fresh document-wide per kind and never reused across
Undo branches within that engine session. Failed commits do not advance the identity floors.
Creation always makes an independent fragment, including at coincident existing positions;
it does not attach to existing atoms, snap angles/lengths or change bond order. Overlapping
labels are a rendering error recoverable with Undo. No save/reopen workflow is implemented.
`bondline-cli mcp` serves a process-local document over stdio using rmcp 3.9.0. It negotiates
MCP lifecycle, exposes the registry as tools and returns structured content plus text. Domain
errors are tool results with `isError`; unknown tools are protocol errors. Input lines are
bounded to 1 MiB and oversized lines are discarded before parsing. EOF closes the session.
This process has its own engine; it does not attach to the open desktop window. The stdio
process opens no network listener, HTTP transport, account or hosted service.

Start the desktop with `--control-port PORT` to opt into raw newline-delimited JSON on
127.0.0.1. Port 0 requests an available port. Startup stdout contains `control_address` and
`control_token`; treat that output as a credential, not a public log. The UI shows only the
address. Each new connection must first send exactly this object followed by LF:

```json
{"token":"<control_token from startup>"}
```

The server replies `{"authenticated":true}` followed by LF. Only then send the same
`{command,params}` objects used by the engine, each followed by LF. Each command response is exactly
one `{result:...}` or `{error:{code,message}}` object followed by LF. Requests on a connection
run in order; connections share the native document and history. Changed state requests a
native repaint. There is no MCP handshake on this separate raw control channel.

Each server creates 32 bytes from the operating system random source and represents them
as 64 lowercase hex characters. Random-source failure aborts startup; no predictable fallback
exists. Tokens are fresh for each server lifetime and cannot authenticate a different server.
Comparison uses pinned subtle 2.6.1's constant-time slice operation (its published guarantee
is best effort, not a hardware side-channel certification). Duplicate/unknown fields,
wrong types, missing/incorrect tokens or a command before authentication return
`unauthorized` and close. The response never echoes supplied or expected credentials.
Authentication permits all current document commands for that connection; there are no
per-command roles, accounts or persistent credentials. Restarting invalidates the token.
Possession is authorization; this does not isolate an already compromised local account.
The startup recipient must keep the token private. Stdio MCP and CLI requests are unchanged.

Limits are explicit Bondline policies: 1 MiB per request, 16 simultaneous connections,
30 seconds for a complete input frame or output write. Excess connections close. Missing
LF, oversize or malformed JSON/parameter shapes return an error where possible and close
the connection. An HTTP header cannot expose its body as commands. Valid domain errors
leave authored state unchanged. Dropping the server closes the listener and cancels clients;
explicit `shutdown()` also reports worker errors. No non-loopback bind option is offered.

Capture dimensions are 1..2048 pixels per axis. A successful result adds `capture` with
`mime_type: "image/png"`, pixel dimensions, the requested backend and padded RFC 4648
`png_base64`. GPU output includes the actual adapter name/backend/device type; CPU output
has `adapter: null`. There is no automatic CPU fallback. This captures the shared page
scene, not window chrome, operating-system dialogs or print-resolution chemical output.
Source-backed documents return unsupported_content before rendering. Native empty pages and
the supported two-carbon/three-H single-bond fragments can be captured. Other chemistry,
overlapping/off-viewport labels and excessive scenes fail explicitly; no content is dropped.
Labels use the attributed Apache font at the original 12-point style, bounded to 8..256 em
canvas units. An unreadable/enormous scale returns unsupported_scale; enlarge the capture
or change the view. Physical publishing, general labels/styles and file export remain missing.
The same rules apply to both backends and leave authored state unchanged.

```json
{"command":"render.capture","params":{"page_id":1,"width_px":512,"height_px":512,"backend":"gpu"}}
```

Preparation snapshots the document/revision under the engine lock. Rendering and PNG
encoding run outside that lock; later edits cannot alter the captured revision. The result
never installs state or overwrites newer edits. Queries leave revision/history unchanged.
CLI calls finish synchronously; each MCP/loopback server permits at most two capture workers
and rejects excess work with `busy`. Short authoring commands remain available at that limit.

MCP cancellation uses the SDK request token. Loopback queries have a 30-second execution
deadline. Aborting a connection task during server shutdown signals its capture worker;
the worker retains its budget until it actually exits. CPU scanlines, GPU readback polling
and stage boundaries cooperate. Driver calls and a bounded PNG encoder call cannot be
preempted. A disconnected client is detected by the next socket I/O, not continuously during
rendering. A committed authoring edit still succeeds if cancellation arrives afterward.
Cancellation/error codes include `cancelled`, `render_failed`, `backend_unavailable` and
`worker_failed`. A missing GPU is an error, not evidence of a CPU capture.
