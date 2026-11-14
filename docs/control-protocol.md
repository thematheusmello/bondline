# Command contract

The delivered engine has one typed registry in `bondline-commands`. Names, Rust parameter
types, schemas and discovery metadata originate in the same declaration. The UI and external
adapters must call it; they must not implement their own authoring operations.
The delivered registry has nineteen commands; the complete list is below.

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
threshold including the current document and saved baseline. Shared source/model/fragment
allocations count once; baselines and reserved vector capacity count too. A changed commit
discards redo, then evicts the oldest Undo prefix until the retained suffix fits. Current
document and saved baseline are always kept, even if together they exceed the threshold.
A later save may trim farthest Redo states too. Arc/allocator headers, in-flight snapshots, temporary work,
caches and client-held snapshots are excluded; this is not a process-memory ceiling.
These are explicit Bondline policies, not claims about another application's history limits.
Revision overflow rejects the operation before history eviction or mutation.

Drawing snapshots include a fragments array, empty in the default application. Engine
history retains complete StoredContent: either a source-backed StoredDocument drawing
or original/current whole-SDF snapshots. A backed session returns an additional source
object: {format: "cdxml"|"cdx"|"mol"|"sdf", byte_length, modified}.
The field is omitted for native in-memory documents. Raw source bytes are not serialized.
The open/save commands below use that complete backing. bond.create authors a limited native
chemical fragment; source-backed creation is refused unless a complete profile is proven.
Exact current native encoding byte equality now permits headless edits/captures after
reopening. Other modified source-backed models cannot export stale XML; exact undo restores
original bytes/exportability.
Failed imports retain the prior complete state/history/path. Registry saves encode through
StoredDocument's format-specific export guards; no command result includes raw source bytes. Native file-menu delivery
is a separate UI integration; general imported structural editing remains missing.
See model.md for the data boundary; serialized graphs are not a project-save workflow.

| Name | Parameters | Effect |
|---|---|---|
| `document.inspect` | `{}` | Read-only snapshot |
| `document.open_cdxml` | `path`, optional `discard_unsaved=false` | Bounded native CDXML read; successful application starts a clean document session and clears history |
| `document.save_cdxml` | `path`, optional `overwrite=false` | Atomic complete-source save; default no-clobber; authoring revision/history retained |
| `document.open_cdx` | `path`, optional `discard_unsaved=false` | Bounded read of the supported CDX projection with complete source; new clean session and cleared history; A4 preview policy, faithful preview/editing unavailable |
| `document.save_cdx` | `path`, optional `overwrite=false` | Atomic exact unchanged CDX backing only; changed models, source-free encoding and lossy cross-format export refuse |
| `document.open_mol` | `path`, optional `discard_unsaved=false`, `points_per_unit=24`, `origin_x_pt=72`, `origin_y_pt=72` | Validate and decode the actual V2000/V3000 version; retain complete source/frame; same guarded new-session behavior as CDXML |
| `document.save_mol` | `path`, optional `overwrite=false`, `version="auto"` (`auto`, `v2000`, `v3000`) | Atomic MOL save with captured output version; refuses unsupported conversion/metadata/layout loss; imported source retains its frame |
| `document.open_sdf` | `path`, optional `discard_unsaved=false` | Validate every mixed V2000/V3000 record and retain complete ordered source; clean collection session and cleared history, without a drawing projection |
| `document.save_sdf` | `path`, optional `overwrite=false` | Atomic whole-source collection save; retains every record, MOL version, ordered field and byte; no molecular regeneration |
| `sdf.record.inspect` | Required `expected_revision`, positive `record_id`; optional `field_offset=0`, `field_limit=64` (1..64) | Deferred read-only ordered field page for one captured record; explicit value omissions and bounded compact JSON |
| `document.close` | `expected_revision`, optional `action="request"` (`request`, `discard`, `cancel`) | Shared conditional close intent; retains authored data/history; native shutdown waits for host acknowledgement |
| `ui.controls` | `{}` | Discover delivered menu/shortcut bindings, typed invocation and contextual enabled/disabled reason |
| `ui.file` | `expected_revision`, `action`, conditional `request_id` / `close_epoch` | Request Open/Save/Save As presentation or cancel the matching picker; a native host is required |
| `page.set_size` | `page_id`, `width_pt`, `height_pt` | One validated page edit |
| `bond.create` | `page_id`, `begin_x_pt`, `begin_y_pt`, `end_x_pt`, `end_y_pt` | One independent C-C fragment; two C atoms with three explicitly authored H each, one single bond |
| `bond.extend` | Required `expected_revision`, `page_id`, `fragment_id`, `begin_atom_id`, `end_x_pt`, `end_y_pt`, `hydrogen_policy="rectify_terminal_carbon"` | Add one Single bond/new C-H3 to a positioned neutral terminal C-H3, deliberately changing that target to H2; native complete CDXML/source-free data only |
| `history.undo` | `{}` | Previous authored snapshot |
| `history.redo` | `{}` | Next authored snapshot |
| `render.capture` | `page_id`, `width_px`, `height_px`, `backend` | Read-only page preview PNG; explicit `cpu` or `gpu` |

The delivered CLI exposes `commands` and `run --cmd '<request>'`. Repeat `--cmd` to execute
sequential operations in one in-memory engine; stdout is one final JSON snapshot. Failure
prints a structured error to stderr and exits 2 (I/O/serialization failures exit 1). It does
not make a multi-command transaction. Explicit open/save commands persist the supported
CDXML/CDX/MOL subsets and whole SDF collections; otherwise invocations remain independent. `--help` lists actual operations.

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
labels are a rendering error recoverable with Undo. Reopening/editing/rendering is supported
only for proven native-profile files; other unknown source data remains guarded and preserved.
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
run in order; connections share the native document and history. Authoring, file and close
effects request repaint after their response is written/flushed; a failed write still
notifies committed effects. There is no MCP handshake on this separate raw control channel.

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
Unmapped source-backed documents return unsupported_content before rendering. Empty pages
and supported positioned neutral-carbon/Single graphs can be captured, including branches,
cycles and disconnected components. Authored H counts render literally; missing H stays
skeletal only for connected carbon. Isolated unknown-H/other chemistry, overlapping/off-
viewport labels and excessive scenes fail explicitly; no content is dropped. Native hover
overlays are transient UI feedback and do not enter captured scenes or authored state.
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
CLI calls finish synchronously. A cloneable CommandExecutor shares two workers across the
delivered native UI and loopback adapter; stdio has its own process-local executor. Rendering
and file work share that budget; excess work returns `busy`. Each deferred plan reserves
its worker while the engine mutex is still held, before any spawning/await gap. Short
authoring commands remain available until host shutdown commits. Preparation may already
commit a short edit, so adapters must never replay it.

MCP cancellation uses the SDK request token. Deferred loopback requests have a 30-second
cancellation deadline. Aborting a connection task during server shutdown signals its worker;
the worker retains its budget until it actually exits. CPU scanlines, GPU readback polling
and stage boundaries cooperate. Driver calls and a bounded PNG encoder call cannot be
preempted. A disconnected client is detected by the next socket I/O, not continuously during
rendering. A committed authoring edit still succeeds if cancellation arrives afterward.
Cancellation/error codes include `cancelled`, `render_failed`, `backend_unavailable` and
`worker_failed`. A missing GPU is an error, not evidence of a CPU capture.

## File operations and saved state

Every result includes `file: {path: string|null, dirty: boolean, format: "cdxml"|"cdx"|"mol_v2000"|"mol_v3000"|"sdf"}`. Dirty compares the complete
current snapshot with the last successful save/open baseline, including unknown source data.
Undo back to that baseline is clean; exact no-ops do not dirty it. `source.modified` instead
describes the retained original backing's baseline. A changed native-profile file may be
saved with `source.modified=true` and `file.dirty=false`; these describe different baselines.

```json
{"command":"document.open_cdxml","params":{"path":"target/example.cdxml"}}
```

Open rejects `unsaved_changes` unless `discard_unsaved:true` is explicit. Reading/parsing runs
outside the engine lock. Apply requires the original revision/session generation; otherwise
`stale_revision` retains the current document. Success assigns canonical path, increments
revision/generation and clears history as a new single-document session. Invalid, unsupported,
missing, oversized and cancelled inputs retain complete document/source/history/path.

```json
{"command":"document.save_cdxml","params":{"path":"target/example.cdxml","overwrite":false}}
```

Save snapshots complete data/revision and publishes through native-file-io.md's contract.
Default conflict is `already_exists`; overwrite requires `true`. Unsupported modified opaque
data returns `unsupported_content`. JSON save paths must resolve to representable canonical
Unicode destinations before writing. Native io PathBuf callers retain Linux non-UTF8 support.

Successful file work adds `file_operation: {operation, format, path, input_revision, byte_length}`;
save also includes `directory_synced`. Save does not advance authoring revision or add a
history edit. It associates saved snapshot/path only with the same document generation.
Later edits stay dirty; a prior-session save cannot relabel a newly opened file. Associated
format is authoritative even when a path's extension disagrees; Save As can explicitly
change it. Source format and associated save format are separate: saving a native snapshot
as MOL does not invent retained original MOL bytes in that snapshot. Save may
trim history for storage accounting. Retaining history differs from G25 printed14's
since-save Undo description; exact26.1 behavior remains an explicit parity question.

`Prepared::complete` returns opaque Completed; Engine::finish applies it after worker I/O.
Each Engine permits one outstanding save across all formats, paths and adapters. Its
reservation starts during prepare, survives publication in Completed, and ends only when
finish returns or the work/result is dropped. Another Save returns `busy` before I/O or
state changes. This prevents reversed receipt application from replacing the latest saved
baseline/encoding. Queries, authoring edits, Open and capture remain available; different
Engines have independent slots. Error/cancellation/pre-completion drop releases the slot.
An old-generation or committed-session receipt still reports actual published bytes without
relabeling the current session. Callers must apply successful Completed to its Engine;
deliberately dropping a published receipt releases the slot without updating association,
baseline or pretending to roll back bytes. Delivered adapters always finish successful work.
Open checks cancellation before apply. Save cancellation before publication retains old bytes;
after publication the worker reports the real outcome. Explicit cancellation awaits that
outcome, even if an OS syscall exceeds the request deadline. Dropping the awaiting task signals
cancellation but retains the worker permit until it exits.

Stable file errors include `invalid_path`, `file_too_large`, `not_regular_file`, `symlink_target`,
`file_changed`, `unsupported_content`, `invalid_document`, `invalid_sdf`, `resource_limit`, `not_found`, `already_exists`,
`permission_denied`, `unsupported_filesystem`, `io_error`, `recovery_required`. Post-publication
error is `save_published`; its message states that new bytes exist and no rollback occurred.
Recovery messages identify a retained original path. Directory-sync false is a capability
limit, not durable directory metadata evidence. No autosave, recovery journal or general job
scheduler is implied by this bounded executor.

MOL drawing coordinates use `x_pt = origin_x_pt + points_per_unit*x` and
`y_pt = origin_y_pt - points_per_unit*y`. Scale must be finite and positive; origins obey
the model's finite ±100000pt bound. These defaults are an original Bondline placement policy,
not chemical units or a reference application's preset. `document.save_mol` has no frame
parameter: imported MOL keeps its captured frame; native/proven native-CDXML uses the
default frame. V2000 four-decimal coordinates can round by half a0.0001 raw unit plus
floating error; V3000 uses shortest-roundtrip f64 spelling without that quantization.
Unchanged same-version imports preserve every byte, including headers and line endings.
Save version defaults to the associated MOL encoding, otherwise the original MOL encoding,
otherwise V2000. This choice is captured during preparation, before deferred work, and
the receipt reports the actual selected encoding. Explicit choices change the association
only after successful publication. They do not invent a new original source allocation.

Original V3000 source cannot currently be converted to V2000; it refuses before publication.
V2000-source export to V3000 requires complete native regeneration and a separate strict
raw-coordinate precision proof. Native/proven native-CDXML can explicitly export either
version within the existing layout guards. Unknown/null selectors fail `invalid_params`
before I/O; an unknown on-disk version stamp is `invalid_mol`. There is no fallback decoder.
Source info remains `{format:"mol",byte_length,modified}` and never exposes raw data/headers.
MOL source-to-CDXML, unknown CDXML-to-MOL, multiple authored fragments/pages and changed
MOL page dimensions refuse `unsupported_content`. No implicit lossy export mode exists.
Malformed MOL reports `invalid_mol`; valid but unsupported content reports `unsupported_content`.
See file-formats.md for exact supported fields and remaining M1 scope.
See [V3000 application evidence](verification-v3000-application.md) for the independent
RED/GREEN tests, native acceptance status and remaining platform limits.

CDX opens use the existing limited decoder and complete StoredDocument backing, not syntax
Writer regeneration. They use the filesystem library's explicit A4 preview policy, not
inferred BoundingBox/paper settings. Source H remains absent when not authored. Every CDX
import remains is_fully_mapped=false, so default capture/preview and bond creation refuse.
An exact unchanged save preserves all known and opaque source bytes; a model/page change
refuses until exact Undo restores its original baseline. CDX-to-CDXML/MOL and source-free
CDX saves refuse before temporary publication. Native Known Save uses associated `cdx`,
even with a misleading suffix; Save As has an explicit CDX choice and guarded retries.
The same FilePlan/Completed save reservation, revision/generation checks, trusted request
receipt/current acknowledgment and close barriers apply to both new commands. No new file
worker or dispatcher exists. See [CDX application evidence](verification-cdx-application.md)
for engine/wire scope and exact450962c Mac chooser/Known Save/retry/close acceptance.
Interactive Windows/Linux, broad CDX files, editing and faithful preview remain unverified.

### Complete SDF sessions

Drawing responses retain their existing `document` object and omit `collection`. An SDF
response explicitly has `document:null` and metadata for every ordered record:

```json
{"document":null,"collection":{"format":"sdf","byte_length":777,"record_count":2,"records":[{"id":1,"mol_version":"v3000","byte_length":462,"mol_byte_length":344,"field_count":6},{"id":2,"mol_version":"v2000","byte_length":315,"mol_byte_length":282,"field_count":1}]}}
```

This example shows only the content-specific fields; revision/history/file/source/close
metadata remains present. Record IDs are positive local ordinals, preserved by an immutable
field-value replacement but not promised across another source. Automatic results and Debug
contain no raw molecular bytes, headers or field values. Each record reports its actual MOL
version; `sdf` identifies a container family, never a fake whole-file V2000/V3000 encoding.

`collection.byte_length` and Save receipt length describe current bytes. `source.byte_length`
continues to describe the original source, and `source.modified` compares original/current.
`file.dirty` compares complete current/saved content, including both original identity and
opaque bytes. A private library field candidate exercises a704-byte current save from a690-byte
original: Save clears dirty while source remains690/modified. No SDF edit command is exposed yet.

Open validates the whole input before applying it, including later records; unsupported
model projection is not a reason to drop valid opaque MOL data. Save borrows all current
bytes directly and uses the same FilePlan/Completed reservation, atomic publisher,
stale-generation, trusted UI request and close barriers as drawing formats. Drawing-to-SDF
and SDF-to-CDXML/CDX/MOL saves return `unsupported_content` before temporary creation.
All drawing-only authoring/capture commands return the same typed wrong-kind error on SDF.
The Rust API now exposes `Engine::content()` for all content, while `document()` and
`stored_document()` return Result; callers must handle a collection explicitly.

The native chooser/known Save/retry routes are wired to the same commands. SDF uses one
Open/Save family with `.sdf`/`.sd` hints. The workspace clears old drawing interactions and
shows a static count/record summary with chemical preview explicitly unavailable. This is
an original partial presentation, not the observed26.1 all-record drawing layout. Explicit
field inspection is available through the query below. Editing, molecular layout, field UI
and actual native SDF acceptance remain missing or unverified in this checkpoint.
See [SDF session evidence](verification-sdf-session-files.md).

### Explicit read-only SDF record inspection

```json
{"command":"sdf.record.inspect","params":{"expected_revision":1,"record_id":1,"field_offset":4,"field_limit":2}}
```

Parameters are typed/closed. Revision must match during preparation; record_id is a positive
local ordinal in that captured collection. A missing record returns not_found. field_offset
may equal the record's field count, returning an empty terminal page, but may not exceed it.
field_limit must be1..64 at runtime as well as in discovery schema. Wrong content returns
unsupported_content; stale revision returns stale_revision; invalid IDs/limits/offsets or
unknown/preview parameters return invalid_params. Committed close rejects new queries.

Only this explicit command adds sdf_record to the normal result. That object contains
input_revision, session_generation, record_id, actual mol_version, complete record/MOL
byte lengths, total field_count, requested field_offset/field_limit, ordered fields,
next_field_offset and page_limited. Each field has its zero-based occurrence index, exact
header text excluding its line ending, optional name/textual dt_number, value_byte_length,
value_line_count, value_lines and omitted_reason. DT leading zeros, duplicate order, spaces
and tabs remain literal. value_byte_length includes retained value line endings but excludes
the blank item terminator. Empty data is value_lines:[] with no omission; omitted values
are value_lines:null with an explicit reason and exact raw byte/line counts. No raw MOL or
whole-source bytes are returned. Debug remains metadata-only, even for explicit results.

Two independent bounds apply to an individual disclosed value: retained raw bytes and its
compact UTF-8 JSON line-vector must each be at most65,536 bytes. If raw bytes exceed that
limit, omitted_reason is value_byte_limit; otherwise an oversized serialized vector uses
value_json_limit. Quote/backslash/tab escaping and vector punctuation count toward the JSON
limit. No value is truncated; a field summary still occupies its ordered occurrence and
pagination can advance past it. Physical source lines retain the codec's separate200-byte
limit; these query bounds do not change format validation or authorize edits.

The compact serialized **sdf_record object alone** is at most1,048,576 bytes, including
headers, field/value arrays, metadata, escaping and continuation fields. A counting writer
checks actual serialization before accepting each field. It returns the largest fitting
prefix up to field_limit; page_limited:true marks byte-budget truncation and the next offset
is the first undisclosed occurrence. Otherwise a next offset can still indicate the request
limit. Nonterminal pages always advance; zero-field/end-offset pages end with next:null.
An impossible single-field page returns resource_limit rather than looping.

This is a page budget, not a complete response-envelope limit. Normal result/state metadata,
collection summaries, paths, wrapper keys and LF/protocol framing are outside it. MCP3.9.0's
structured result also includes a serialized text copy; that duplication/escaping is outside
the page budget. Clients must not treat the1MiB input-frame or page bounds as an overall MCP
reply limit. The existing codec, record-count, path, worker and output-deadline bounds still
apply; no listener or unbounded collection-disclosure API is added.

Preparation validates only cheap kind/revision/record/index metadata and captures immutable
source/result identity. Source reparse, field cloning and JSON budget work happen outside
the engine lock using the same two-worker budget and reply/close activity guards as captures
and file jobs. A busy worker refusal changes no selection, close intent, snapshot or history.
Cancellation is checked before/after the bounded existing parser call and between field
operations; that parser call and one bounded serialization step are not preemptible.

Queries leave current/saved/original bytes, dirty state, revision/history, file association
and close policy unchanged. A completion may return the captured revision/generation after
a later edit/Open; it never installs its page as current state. Lifecycle metadata refresh
follows the existing capture contract. Automatic document.inspect still omits all queried
headers/values, and read-only queries request no repaint. Worker/reply activity nevertheless
blocks trusted native close acknowledgement through output flush. No selection, preview,
setter, delete operation or UI editor is delivered here. This is an original inspection API,
not evidence of a reference-product screen. See
[query verification](verification-sdf-record-inspect.md).

## Close requests and host shutdown

The command registry owns single-document close policy. Native File Close, Cmd/Ctrl+W,
window Close and Mac Quit, plus close-dialog Discard/Cancel/Escape/Keep Window Open, execute
`document.close`. Save remains the existing file command. This is Bondline's protected
lifecycle policy, not proof of a complete26.1 dialog or multi-document implementation.

```json
{"command":"document.close","params":{"expected_revision":0,"action":"request"}}
```

Parameters are closed and `expected_revision` is required. Missing action defaults to
request; unknown fields/actions fail. A stale authored revision returns `stale_revision`
without changing intent. Each successful action advances checked `close_epoch`, including
Cancel and repeated Request; exhaustion returns `close_epoch_exhausted` before mutation.
Close never changes authored revision, stored source/model, history or saved baseline.

| `close` state | Meaning |
|---|---|
| `open` | No pending close intent. Cancel clears an uncommitted request/approval. |
| `needs_decision` | Request found dirty data, or an approval was invalidated. Native presents the decision UI. Saving alone never approves it. |
| `approved` | Clean Request or explicit Discard approved the current revision/path. This is permission to close, not completed shutdown. |
| `committed` | The trusted host acknowledged valid approval after draining work. New commands fail `close_committed`, except read-only Inspect. Late Cancel cannot claim to stop an already queued shutdown. |

Every result includes `close` and `close_epoch`; close commands also include
`close_operation`, so adapters notify even when authored `changed=false`. Lifecycle metadata
is refreshed at finish. Other result fields retain their original operation's authored
snapshot/revision and capture semantics; an old capture is not a fresh inspection of the
document. Inspect current state before issuing a conditional close action.

Normal edits are allowed before host acknowledgement and invalidate approval. Any actual
authored, saved-baseline, path or associated-format change permanently invalidates it until a fresh close
action; returning to old values cannot resurrect permission. The UI's close-specific Save
continuation captures revision/epoch. Only its matching successful receipt, current path,
same revision/epoch and clean state renew Request. An ordinary save, old receipt, Cancel
followed by another Request, or a publication error cannot silently close the window.

The native host holds the shared engine mutex, checks that registered workers/replies,
UI file jobs and emitted-but-not-started file actions are absent, then calls the trusted
`Engine::complete_close` acknowledgement before issuing native Close. Registration uses the
same mutex. A loopback reply guard starts before preparation and survives through output
write/flush, preventing an independent UI frame from closing before that response is sent.
It is released before repaint. Waiting UI frames repaint periodically; dropped awaiters
still retain actual worker activity until completion. Obsolete ordinary choosers are
cancelled before rendering under approved shutdown, and UI file start/result failures retain
the window rather than allowing a later-frame exit.

The host acknowledgement is infrastructure, like file completion, not a second user-command
registry. Headless CLI/stdio have their own engines and report policy state; Request does
not kill the MCP process or claim a native window closed. Loopback controls the actual native
session. An old Open cannot resurrect a committed engine. An already-published Save still
returns its real receipt while leaving that committed session's baseline/path/history alone.
The existing publication/error contract remains in force. Broader dialog addressability,
control metadata and job-ID/cancel automation remain separate M0 gaps.

## Delivered controls and native file presentation

`ui.controls` is a read-only registry command. Its `controls` array describes the delivered
File Open/Save/Save As/Close, Edit Undo/Redo and Page A4/Letter/Rotate bindings. While a file
request exists it also describes its Cancel action. Every record has a stable `id`, label,
menu, optional typed shortcut, actual serialized `command`, `enabled` and `disabled_reason`.
The native menus and shortcut bridge consume the same typed core binding records; there is
no second JSON dispatcher or alternate implementation of an authoring operation. `primary`
maps to Command on Mac and Ctrl elsewhere; shifted bindings are matched first. This is a
point-in-time description, not permission to bypass command validation or a lease on state.
It inventories delivered controls only, not every missing reference menu/tool/button.

Direct `document.open_cdxml` / `document.save_cdxml` and their MOL counterparts perform explicit parameterized I/O.
`ui.file` instead requests native presentation, like using a File menu item:

```json
{"command":"ui.file","params":{"expected_revision":0,"action":"open"}}
```

Actions are `open`, `save`, `save_as`, `cancel`. Every action requires the current authored
revision. `request_id` is required only for Cancel; `close_epoch` is allowed only for Save
and then must match the current unsaved-close decision. Unknown fields, wrong combinations,
stale revisions and stale identities fail without consuming a request or changing data.
Headless CLI/stdio sessions have no attached native host: `ui.file` returns `ui_unavailable`.
They can still discover disabled file controls and use explicit parameterized file commands.
An authenticated native loopback client addresses the actual attached desktop session.

Every result includes `ui_file: {available, epoch, request}`. `available` reports host attachment,
not permission to replace an occupied slot. A request contains its monotonic
`id`, action, input revision/path, optional close epoch and phase. `requested` means the
intent is stored; `accepted` means a native host acknowledged it; `running` means actual
file work was prepared. None of these claims pixels were displayed or bytes were saved.
Actual I/O is established only by the existing file receipt or error. A successful `ui.file`
command adds `ui_file_operation` identifying its request/cancel operation, so even a
`changed:false` response notifies the native UI after reply publication. The current transient
state is refreshed at finish; an older operation marker may identify an earlier request.
Authored snapshots/revision retain their existing immutable input scope.

There is one file-presentation slot. New requests while it is occupied return `ui_busy`.
Save uses the existing associated path and format with overwrite permission, or opens a chooser if
unnamed; Save As always requires a path choice. Host acceptance does not consume the
request: a remote Cancel can dismiss the same accepted picker. Cancel targets its exact ID:

```json
{"command":"ui.file","params":{"expected_revision":0,"action":"cancel","request_id":1}}
```

Accepted request/cancel commands advance a checked epoch without changing authored data,
history, saved baseline or path. IDs are never reused at the same revision. Actual edits,
Open, baseline/path/format changes, host detachment and close-intent changes invalidate a pending
picker permanently. Returning to old values cannot revive it. No-op edits and an unchanged
save preserve it. The native host validates the exact selection and context, reserves a
worker and prepares the real command under the same engine mutex before spawning. A known
Save cannot silently redirect to another path/format or lose its overwrite decision. The opaque
prepared work retains its originating engine and worker lifetime.

Presentation Cancel refuses a running operation; it cannot claim published bytes were
rolled back. A failed worker may restore the same picker only while its context is current;
a stale failure cannot reopen it or affect a replacement. Already-published work still reports
truthfully. Native close-Save retains the separate captured close epoch and matching-receipt
rules above. Ordinary Save never grants shutdown permission. Host claim/finish/attachment
are trusted Rust acknowledgements, not additional user-facing JSON dispatchers.

Trusted host preparation also binds the originating request ID to the opaque file plan.
Its receipt adds `ui_request_id` and `ui_request_current`; direct file commands omit both.
The latter is captured under the engine lock before the operation's own baseline/path/format
update. Invalidation before that finish makes it false even if values return. It is a
historical snapshot after return, not a live permission to close. The Engine separately
records whether the matching trusted completion remains current after its own update;
every later real context change permanently clears that private acknowledgment. The native
host consumes it through finish_ui_file(id,false), which returns false for stale or never-
completed work while still retiring the matching request. Native close-Save requires this
current acknowledgment, both receipt markers, the actual format and all
revision/path/epoch/clean-state checks. A stale publication still reports its real bytes;
it cannot renew close permission.

Folder navigation, filename/location values, directory-list state, confirmation dismissal,
full close-dialog discovery and addressable running-job cancellation remain separate gaps.
The original picker and this protocol do not establish full platform/reference-dialog parity.
See native-file-ui.md and verification.md for actual execution scope.


## Explicit terminal-carbon extension

The closed bond.extend operation requires the named H policy; missing/unknown policies
refuse. Its selected fragment must contain only positioned neutral carbon/Single data,
and its starting atom must have exactly one Single bond and literal H3. Other H counts,
unknown H, nonterminal starts and other metadata/elements/orders refuse. The free endpoint
cannot exactly occupy an existing atom. The command checks expected_revision before
lookup/allocation and commits one complete snapshot, preserving fragment/old IDs and
allocating only a new AtomId/BondId above all pages/session floors. Undo branches never
reuse these identities. Failure preserves the source, saved baseline, history and floors.

Byte-proven native CDXML or source-free authority is mandatory; actual imported MOL/CDX
backing refuses even when drawable. A source-free document associated with a MOL file
remains source-free; reopening creates MOL backing. No encoder or native-v0 profile is
changed. Published saved propane/butane files face independent Expat/Decimal/RDKit
consumers through engine/CLI/stdio/authenticated loopback. Native attachment is wired,
with actual native verification pending; this does not implement general valence,
branches/rings/merging or other tools.
See verification-terminal-carbon-extension.md for exact test-first evidence and limits.


The canvas now picks a fresh GraphScene at each eligible press: empty paper routes to
bond.create, a supported terminal atom/label routes to required-policy bond.extend.
Raw press distance controls the3-logical-unit threshold independently of the snapped
model anchor. Ordinary label/unknown-H clicks do not edit; intentional unsupported
starts or occupied releases diagnose without independent-bond fallback. Each qualified
release commits immediately, then later same-frame presses use the resulting graph
and revision. Captured held gestures are never relabeled. Complete-source, stale and
history guards remain in Engine. Native runtime remains pending; exact deterministic
RED/GREEN and source/limit evidence is in verification-bond-attachment-ui.md.
