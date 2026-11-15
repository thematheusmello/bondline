# Command contract

The delivered engine has one typed registry in `bondline-commands`. Names, Rust parameter
types, schemas and discovery metadata originate in the same declaration. The UI and external
adapters must call it; they must not implement their own authoring operations.
The delivered registry has 27 commands; the complete list is below.

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
history retains complete StoredContent: a source-backed StoredDocument drawing,
original/current whole-SDF snapshots, or an immutable complete SmilesFileDocument.
A backed session returns an additional source
object: {format: "cdxml"|"cdx"|"mol"|"sdf"|"smiles", byte_length, modified}.
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
| `document.open_smiles` | `path`, optional `discard_unsaved=false` | Validate all supported notation records and retain the entire source; clean source-file session without a drawing projection |
| `document.save_smiles` | `path`, optional `overwrite=false` | Atomic exact whole-file save, including ignored lines, opaque application bytes and original endings; no notation regeneration |
| `file.convert_smiles_to_cdxml` | Required `expected_revision`, `input_path`, `output_path`, `bond_length_pt`, `origin_x_pt`, `origin_y_pt`, `page_width_pt`, `page_height_pt` | Deferred create-only conversion of one bounded neutral carbon/Single path into new native CDXML; current session and all its baselines remain unchanged |
| `file.export_smiles_fragment` | Required `expected_revision`, positive `page_id`, positive `fragment_id`, `output_path` | Create-only noncanonical explicit-H SMILES file for exactly one complete-source selected fragment; geometry/document context are explicitly not encoded and the current session is unchanged |
| `sdf.record.inspect` | Required `expected_revision`, positive `record_id`; optional `field_offset=0`, `field_limit=64` (1..64) | Deferred read-only ordered field page for one captured record; explicit value omissions and bounded compact JSON |
| `smiles.record.inspect` | Required `expected_revision`, positive `record_id` | Deferred exact selected notation and global byte ranges/counts; opaque application data is not disclosed;128KiB object budget |
| `sdf.field.set_value` | Required `expected_revision`, positive `record_id`, zero-based existing `field_index`, `value_lines` | Deferred immutable existing-field edit, one whole-content history commit; byte-preserving no-op and strict stale/input guards |
| `document.close` | `expected_revision`, optional `action="request"` (`request`, `discard`, `cancel`) | Shared conditional close intent; retains authored data/history; native shutdown waits for host acknowledgement |
| `ui.controls` | `{}` | Discover delivered menu/shortcut bindings, typed invocation and contextual enabled/disabled reason |
| `ui.bond_tool` | Required `order` (`single`, `double`, `triple`) | Select a transient drawing tool; explicit operation receipt/epoch, no authored edit or history entry |
| `ui.file` | `expected_revision`, `action`, conditional `request_id` / `close_epoch` | Request Open/Save/Save As presentation or cancel the matching picker; a native host is required |
| `page.set_size` | `page_id`, `width_pt`, `height_pt` | One validated page edit |
| `bond.create` | `page_id`, `begin_x_pt`, `begin_y_pt`, `end_x_pt`, `end_y_pt`; optional `order="single"` | One independent neutral-carbon fragment with the requested Single/Double/Triple bond; each endpoint receives literal H=`4-order_weight` |
| `bond.extend` | Required `expected_revision`, `page_id`, `fragment_id`, `begin_atom_id`, `end_x_pt`, `end_y_pt`, `hydrogen_policy="rectify_terminal_carbon"`; optional `order="single"` | Add the requested bond/new carbon to a consistent supported terminal carbon, subtracting the bond weight from its literal H; native complete CDXML/source-free data only |
| `bond.set_order` | Required `expected_revision`, positive `page_id`, `fragment_id`, `bond_id`, explicit `order` (`single`, `double`, `triple`) and `hydrogen_policy="rectify_neutral_carbon"` | Edit one existing edge and both consistent neutral-carbon literal H counts; preserve geometry/IDs/direction, one history commit, validated no-op; source-free/byte-proven native CDXML only |
| `history.undo` | `{}` | Previous authored snapshot |
| `history.redo` | `{}` | Next authored snapshot |
| `render.capture` | `page_id`, `width_px`, `height_px`, `backend` | Read-only drawing page or restricted all-record SDF virtual page1 PNG; explicit `cpu` or `gpu` |

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
The selected tool's order/epoch is captured at press; changing mode invalidates old
gestures/hover even if the mode later returns to the same order. The UI passes that order
explicitly. Direct commands with an absent order always retain Single behavior.

`ui.bond_tool` returns `ui_bond_tool_operation:{order,epoch,changed}` and outer
`changed:false`. Real selection changes advance the checked transient epoch; repeated
selection is a no-op. Selection persists across Undo/Redo/Open, outside document/source,
dirty state and close intent. `ui.controls` adds the three `tool.bond.*` bindings with
`selected` booleans, closed typed invocations, no assigned shortcuts and contextual
enablement; existing non-tool bindings omit `selected`. Pending picker/close or incomplete
drawing-source guards apply. Loopback notifies after publishing a real mode change.

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
and supported positioned neutral-element Single/centered Double/Triple graphs can be captured, including safe branches,
cycles and disconnected components. Source-free/CDXML use literal H, including connected
skeletal unknown-H carbon. Actual MOL backing instead supplies closed neutral C/N/O/halogen
display counts for missing H, without changing authored None or literal counts. It retains
the actual current Page/IDs/frame, with no SDF repacking or policy inferred from filename.
Unresolved/unsupported chemistry, overlapping/off-
viewport labels and excessive scenes fail explicitly; no content is dropped. Native hover
overlays are transient UI feedback and do not enter captured scenes or authored state.
Labels use the attributed Apache font at the original 12-point style, bounded to 8..256 em
canvas units. An unreadable/enormous scale returns unsupported_scale; enlarge the capture
or change the view. Physical publishing, general labels/styles and file export remain missing.
The same rules apply to both backends and leave authored state unchanged.

```json
{"command":"render.capture","params":{"page_id":1,"width_px":512,"height_px":512,"backend":"gpu"}}
```

Preparation retains the complete immutable Drawing/SDF owner and revision under the engine
lock, including source needed for display classification. Rendering and PNG
encoding run outside that lock; later edits cannot alter the captured revision. The result
never installs state or overwrites newer edits. Queries leave revision/history unchanged.
Work completion/cancellation/drop releases the extra source ownership; a self-contained
PNG reply does not retain it. MOL display grants no new edit/Save authority. Native vector
painting uses the same IO producer through Engine::drawing_page_scene without a direct
UI-to-IO dependency. The new MOL canvas workflow is headlessly tested, not natively verified.
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
`file_changed`, `unsupported_content`, `invalid_document`, `invalid_sdf`, `invalid_smiles`, `invalid_smiles_export`, `resource_limit`, `not_found`, `already_exists`,
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
original: Save clears dirty while source remains690/modified. The shared setter below now
creates such candidates through the same registry.

Open validates the whole input before applying it, including later records; unsupported
model projection is not a reason to drop valid opaque MOL data. Save borrows all current
bytes directly and uses the same FilePlan/Completed reservation, atomic publisher,
stale-generation, trusted UI request and close barriers as drawing formats. Drawing-to-SDF
and SDF-to-CDXML/CDX/MOL saves return `unsupported_content` before temporary creation.
Drawing-only authoring commands retain the typed wrong-kind error on SDF.
The Rust API now exposes `Engine::content()` for all content, while `document()` and
`stored_document()` return Result; callers must handle a collection explicitly.

The native chooser/known Save/retry routes are wired to the same commands. SDF uses one
Open/Save family with `.sdf`/`.sd` hints. The workspace clears old drawing interactions and
now schedules an asynchronous read-only chemical preview with a collapsed metadata detail.
It uses the same render.capture route and complete source, not an authored drawing. Actual
app-frame/uploaded-image/paint/cache/close behavior passes headless tests; the new operating-
system window workflow remains unverified. The no-executor compatibility helper still shows
only its static metadata table. Original packing/control choices do not establish reference
visual parity. Shared field inspection/existing-value editing remain separate from the
then-missing native field editor; the original existing-value editor below now has bounded
Mac evidence. Earlier actual native file acceptance retains its own source scope.
See [SDF session evidence](verification-sdf-session-files.md).

`render.capture` accepts only virtual `page_id:1` for SDF; another positive ID returns
`not_found`. Existing dimensions/backend validation is unchanged. Preparation retains the
whole source and historical state, and the same registered deferred work builds every
record off-lock. No Save reservation is acquired. A separate display-H sidecar preserves
authored None/literal counts; the source never becomes drawing/edit/export authority.
Successful replies keep document:null, complete collection metadata and unchanged state.
Old captures report their historical revision/source/file with current close/UI metadata.
Early/late cancellation can prevent reply publication without applying a partial view.

The current profile uses strict neutral nonisotopic/nonradical Single/centered Double/Triple geometry and a
closed neutral C/N/O/halogen derivation for absent H; literal resolved counts remain exact.
Shared24pt/unit and translation-only72pt rows are original packing. Actual label/stroke ink
must fit the physical paper and readable font bounds. Later unsupported records, unsafe
placement, unreadable scale or resource limits refuse the entire capture; no first-record
fallback, invented geometry or H suppression is allowed. CPU/GPU use the same scene.
The same producer is now wired to read-only native preview; its actual window workflow is
unverified. No general chemical clean-up, new export format or full reference parity follows. See [command evidence](verification-sdf-capture-command.md)
and [independent image/adapter evidence](verification-sdf-capture-oracles.md) for scoped
verification and remaining limits.

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
delete operation or UI editor is delivered by this query. This is an original inspection API,
not evidence of a reference-product screen. See
[query verification](verification-sdf-record-inspect.md).

### Existing SDF field-value editing

```json
{"command":"sdf.field.set_value","params":{"expected_revision":1,"record_id":1,"field_index":5,"value_lines":["revised","multiline"]}}
```

All four parameters are required and the object is closed. The target is an existing ordered
occurrence, never a name/property-map lookup. Positive record IDs are the same local ordinals
used by inspection; field_index is zero-based. Record/field order, IDs, count, headers, names,
textual DT identifiers, molecular versions and all untouched source bytes stay exact. []
authors an empty value; spaces/tabs are nonempty data. Field/header creation, renaming,
removal, record navigation/deletion, molecular editing and conversion are outside this command.

Inputs must satisfy two65,536-byte limits: the sum of UTF-8 line bytes plus one authored LF
per line, and the compact UTF-8 JSON value_lines array including escaping/punctuation. These
are linked to inspection's raw/vector limits, so accepted changed LF data can be disclosed
without omission. Empty arrays have zero authored bytes but a two-byte JSON representation.
The existing request1MiB bound is separate. ASCII/tab/control, nonempty physical line,
200-byte line and column-zero dollar-prefix constraints remain the unchanged codec rules.
Each changed line uses LF; the original blank terminator, padding and every other byte keep
their source spelling. A logical no-op preserves original CRLF and shares the old source/index.

Preparation validates cheap kind/revision/record/index metadata and captures the complete
original/current Arc plus revision/generation, without authoring or transient mutation.
Input budgets, format validation, splice and full-source reparse run off-lock through the
same two workers. Completion checks cancellation, committed close, revision/generation and
the exact current input Arc before one existing Engine commit. Reverse, old-generation,
cancelled or invalid candidates leave all state/history/source/saved data intact. Parsing
and one bounded serialization stage are not preemptible; cancellation is checked between
stages. There is no extra pool, save lease, listener or filesystem write in the setter.

A real change advances revision once, clears redo, retains complete source in Undo and
updates dirty/source metadata; normal history accounting/close/picker invalidation applies.
A no-op leaves revision, history/redo, current outer/source/index identities, dirty state,
saved association and close/trusted receipt approval unchanged. Save still captures whole
content: an older pending Save publishes its captured bytes and keeps a newer field edit
dirty, with its reservation held through finish/drop. Editing then undoing to saved data
cannot revive an invalidated trusted close-Save acknowledgement. Source.modified always
describes original bytes, independently of file.dirty's saved baseline.

Stale revision/current input returns stale_revision, wrong kind unsupported_content, and
missing record/occurrence not_found. Invalid positive-ID/JSON shape returns invalid_params.
The command's aggregate input budgets return resource_limit; existing codec errors retain
their codes (invalid_sdf for malformed lines/control/prefixes, unsupported_content for
non-ASCII, resource_limit for line/global quotas). No parser/writer code is relabeled.
Successful setter results are normal metadata-only state; use explicit inspection to read
values. Debug omits input/candidate field text. Changed edits notify after response flush;
no-op/refused edits do not repaint. Worker/reply activity retains the existing close drain.

This is a shared headless command, without a new native UI/editor or claimed reference
field workflow. See [setter verification](verification-sdf-field-command.md).

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
refuse. Its selected fragment must contain only positioned neutral carbon with
Single/Double/Triple bonds. Its starting atom must have exactly one incident bond with
literal H=`4-existing_bond_weight`, and sufficient H for the requested order (weights1/2/3).
That count is reduced by the new weight, and the added carbon receives literal H=`4-new_weight`.
Unknown/inconsistent/insufficient H, nonterminal starts and other metadata/elements/orders
refuse. This explicit policy is not general valence perception. The free endpoint
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

## Restricted create-only SMILES conversion

`file.convert_smiles_to_cdxml` explicitly selects the `neutral_carbon_path_v1` conversion
policy. Every parameter is required; there is no implicit geometry, overwrite switch,
discard flag, selection, native request ID or alternative H policy. Length must be finite
and positive; origin is finite within ±100000pt and page dimensions are positive and at
most100000pt. Runtime validation also covers direct typed Rust callers. A stale expected
revision fails `stale_revision` before file work. The command is **not read-only** in
registry/MCP metadata: it publishes a file, although authored state is unchanged.

The native IO operation reads the explicitly chosen complete source under its64KiB cap,
retains the notation owner while validating/layout/encoding, and creates a new CDXML file
through the existing atomic publisher. Only actual connected1–12 neutral nonisotopic
carbon/Single paths that satisfy fixed-H normalization, geometry and the1024reference
ink/page checks convert. Aromatic markers or bonds, including uppercase `C:C`, and other
unsupported content refuse. Input/filename suffix never selects a decoder or changes the
fixed CDXML output format. Output must not exist; input bytes and permissions are not edited.
This is conversion to a new drawing, not source-backed SMILES Open/Save or general cleanup.

Success adds optional `conversion` to the latest Engine state:

```json
{"conversion":{"profile":"neutral_carbon_path_v1","input_revision":0,"session_generation":0,"input_path":"/chosen/input.smi","input_byte_length":3,"atom_count":2,"bond_count":1,"output":{"path":"/chosen/new.cdxml","byte_length":408,"directory_synced":true}}}
```

Paths/byte lengths in that example are illustrative, not reference bytes. Counts and output
receipt describe actual work. `input_revision`/`session_generation` are captured at preparation;
the input file is actually bounded/read by its worker. `revision`, `document`/`collection`,
source, saved/file association, dirty/history, close epoch/approval and native file intent
remain the current session's state at finish. Conversion neither opens the generated drawing
nor marks anything saved. No `file_operation` or native completion acknowledgment is fabricated.
Automatic inspect/state omits `conversion`; it is a returned receipt, not persistent content.

The existing per-Engine save reservation covers native saves and conversion together, from
prepare through publication and finish/drop. A second save/conversion fails `busy` before IO.
Existing two-worker, cancellation, reply-publication and close-drain barriers remain shared.
A worker-budget refusal drops the unexecuted plan/lease without changing session state.
Canonical output must be representable as Unicode before writing. An unrepresentable actual
receipt path discovered after publication remains a truthful `save_published` error.

Finishing an old published conversion returns its captured receipt with the **latest** Engine
state even after edits, a new Open generation or committed close. It does not revive an old
session/close approval or claim rollback. Cancellation before publication creates no output;
after publication the real outcome remains reportable. Dropping an unexecuted plan or a
published completion releases its reservation without updating the active session.

Native file presentation cannot bind this command. `prepare_ui_file` rejects it before
reservation; the generic plan binder explicitly refuses it rather than silently treating it
as Open/Save or ignoring an ID. A conversion cannot complete, clear or renew a trusted native
Save acknowledgment. CLI, stdio MCP and authenticated loopback use the same registry/worker
operation; native conversion controls and interactive acceptance remain missing. See
[verification-smiles-conversion-command.md](verification-smiles-conversion-command.md).

## Selected-fragment SMILES export

`file.export_smiles_fragment` exports the complete supported chemical graph of the exact
application page/fragment IDs supplied. Its four parameters are required and closed. Positive
missing IDs return `not_found`; zero IDs or unknown/wrong-shaped parameters fail
`invalid_params`; mismatched revision fails `stale_revision` before IO. No first fragment,
UI selection or implicit H is substituted. The registry/MCP flag is `read_only:false`
because the command writes a new file.

The current content must be Drawing with `StoredDocument::is_fully_mapped()` true. SDF,
always-incomplete CDX and unknown/extended/noncanonical CDXML refuse. A false MOL complete
classification also refuses conservatively, including coordinate-preservation limitations;
this does not independently prove its chemical projection incomplete. The entire immutable
StoredContent owner stays captured, not only a projection or authority boolean. Selection
and source eligibility are checked before IO and from that same owner on the worker.

The unchanged writer requires supported neutral H/C/N/O/F/Cl/Br/I, RadicalNone and exact
authored H counts, with its finite valence guards. It preserves positive model isotopes,
physical H nodes, Single/Double/Triple, branches, cycles and all components of the chosen
fragment. None H and unsupported chemistry refuse; no normalizer, renderer or carbon-path
layout restriction is applied. Authored IDs choose deterministic traversal, not canonical
chemical order, and are not encoded as atom-map/class fields.

The new file is the unchanged writer notation plus exactly one LF: no name, BOM, header,
trailing spaces or additional record. The writer's65,536-byte notation cap remains, so the
file may be65,537 bytes. Existing parser/converter raw-input caps remain65,536; a maximum
export therefore exceeds their current whole-file input cap by one byte. No round-trip
SMILES project/import compatibility is claimed and no limit is silently changed.

Success adds `smiles_export` to the latest current Engine state:

```json
{"smiles_export":{"profile":"neutral_explicit_h_fragment_v1","scope":"selected_fragment_chemical_graph","not_encoded":["geometry","document_context"],"input_revision":1,"session_generation":1,"page_id":1,"fragment_id":2,"atom_count":2,"bond_count":1,"output":{"path":"/chosen/new.smi","byte_length":13,"directory_synced":true}}}
```

Those example counts/bytes describe `[13CH3][OH1]` followed by LF. `geometry` excludes
coordinates/depiction; `document_context` excludes application identities, page/layout/style,
unselected content and original container/source bytes. These are declared export categories,
not permission to discard unmodeled chemical data. The output is not an editable drawing
or a whole-document Save. The complete active current/saved/original/source, history,
identity floors, association/format/dirty, close and native intent/ack remain unchanged.
Ordinary state/inspect omits `smiles_export`; it is only a publication receipt.

Preparation uses the same save reservation as native saves and conversion. The whole owner
and lease survive worker publication in Completed until finish/drop. A second publication
fails `busy` before IO; cancelled/refused/dropped work releases its reservation. Existing
two-worker, reply-flush and close barriers apply without a new pool/dispatcher. Finish
returns the latest state and the captured revision/generation/IDs even after edits/Open or
committed close, without installing an old session or inventing rollback after publication.
The generic native-file binder explicitly refuses export; it cannot complete or renew a
native Save acknowledgment. Native export controls/clipboard workflows remain missing.
See [verification-smiles-export-command.md](verification-smiles-export-command.md).

## Complete SMILES source-file sessions

document.open_smiles requires only path and accepts
optional discard_unsaved=false; document.save_smiles requires only path and accepts
optional overwrite=false. Both closed schemas reject record/layout/conversion options.
Open validates every meaningful notation and starts a clean whole-file generation. Any
later failed record leaves the prior entire session untouched. Save uses captured complete
content and the same shared write reservation through finish/drop; an old-generation
receipt cannot associate or mark a newly opened file saved. There is no implicit conversion.

Automatic state has document:null, no SDF collection, and optional
`smiles_file:{format:"smiles",byte_length,record_count}`. It contains neither notation nor
opaque application data and has constant-size metadata regardless of record count. Source
is {format:"smiles",byte_length,modified:false}: there is no source-file edit command yet.
Current/saved file association is independent of its suffix. Zero-record sources are valid.
Drawing/SDF operations and cross-format Save refuse unsupported_content, without temporary
output. Existing explicit SMILES conversion/export retain their separate scope and limits.

Native requests bind only actual Open/Save selections; existing .smi/.smiles picker,
known Save, Save As retry and clean-close worker draining are wired/headlessly tested.
A metadata-only view clears stale drawing gestures/hover and exposes accessible count/size
and unavailable-preview status. No chemical preview/editing or actual native execution is
claimed. CLI, stdio MCP and authenticated loopback use the same commands and real files.
See verification-smiles-file-app.md and verification-smiles-file-ui.md for exact evidence.

## Explicit SMILES record inspection

`smiles.record.inspect` has only required expected_revision and positive record_id,
read_only:true and a closed schema. It validates the current source/revision/ordinal,
captures the whole immutable owner, then reads only the selected notation off-lock.
The optional smiles_record object returns input_revision, session_generation, record_id,
line_number, full_range, notation_range, separator_range, application_data_range,
line_ending_range, atom_count, bond_count and exact notation. Ranges are file-relative
{start,end}; null differs from an empty existing data span. Application bytes are not
returned or decoded. No automatic state/Debug notation disclosure or result cache occurs.

The object's real serialized JSON is capped128KiB independently of parent/wire envelopes;
notation remains capped65536. Cancellation survives complete→finish. The response uses
captured historical source/file/revision like sdf.record.inspect, with current close/UI
fields refreshed; input/gen stamps identify the source. A newer Open is not overwritten.
No save lease, mutation, native file binding/ACK, selection, preview or repaint follows.
See verification-smiles-record.md for core scope and separate receiving evidence.

## Native preview host contract (Rust adapter)

CaptureStamp is opaque Weak-content/revision/generation identity, not serialized input or a
source lease. CommandExecutor::prepare_ui_capture validates it and close context under the
same Engine mutex before reserving its worker; the host releases its frame lock before that
call and keeps pending intent in close busy state. UiCaptureWork still calls the existing
registry. No new wire command or native file binding/ACK/Save lease is introduced.

PngCapture::png_bytes preflights the pinned own PNG/base64 envelope under dimension-specific
bounds; the native decoder still validates payload/CRC. Actual main-label em is Rust-only
presentation data and is omitted from JSON, which retains its six PNG fields. The UI tracks
its job through decode/publication/retirement after command completion, rejects full source/
view/DPI/close-epoch stale results and checks logical readability before painting. Current
Cancel/Retry/Records controls are local transient presentation, not newly addressable MCP
tools; complete control inventory/addressability remains incomplete M0 work.

## Native SDF field work (Rust adapter)

`CommandExecutor::prepare_ui_sdf(&CaptureStamp, close_epoch, Command)` accepts only
the existing `sdf.record.inspect` and `sdf.field.set_value` operations. Under the originating
Engine mutex it checks the complete source stamp, Open close state and exact captured
close epoch, then reserves the same two-worker budget before preparing the command.
Release the UI frame's Engine guard first. The opaque `UiSdfWork` completes through the
existing cancellation/finish path; drop releases unused work. It adds no wire command,
Save lease, native file binding or acknowledgement.

A query may truthfully return historical captured fields. The host must guard installation
by current source/selection/close context and retain pending/job/publication/thread-retirement
barriers. A previously accepted edit can still commit before host close acknowledgement
and invalidate approval, as the core already permits. Refresh inspection after a successful
edit rather than replaying it. [Independent adapter evidence](verification-ui-sdf-work.md)
retains the exact core/adapter scope.

The native read-only occurrence inspector now uses this bridge for explicit record pages,
with one source/close/record/offset/request-qualified pending job. It follows the actual
`next_field_offset`, retains no other page and distinguishes omitted null values from
existing empty values. Selectable immutable text advertises ReadOnly; text undo storage
is cleared without losing the current selection. Pending/delivery/actual thread retirement
participate in the host close barrier. [Five independent actual-App tests and bounded
Mac observations](verification-sdf-inspector.md) do not establish a mutating editor,
draft/Apply workflow, native pagination race or complete control addressability. These
local presentation controls add no named command or schema; the registry stays at28.

## Existing-bond absolute order setter

`bond.set_order` requires all six target/revision/order/policy fields; it never inherits
the transient tool. The two affected endpoints must be neutral, nonisotopic/nonradical C
with literal H=`4-sum(incident weights)` and only Single/Double/Triple incident orders.
Replace the target weight and adjust both H counts by old−new after checking the new
sums do not exceed4. Unknown/inconsistent endpoint H, unsupported metadata/orders and
overvalence refuse atomically. No positions are inferred or changed; absent positions
and every unrelated atom's metadata/None remain exact. This is closed C4 arithmetic, not
general valence perception.

Zero target IDs/unknown fields/missing policy fail invalid_params; missing page/fragment/
bond fails not_found; stale revision fails stale_revision; unsupported source/profile/kind
fails unsupported_content. Source-free or byte-proven native CDXML authority is required
even for a same-order request. A valid no-op retains current Arc, revision/history/redo,
saved/source/file/close state; a real edit replaces exactly one same-ID fragment and
commits once. No identity floor advances. Direct invocation does not change tool mode/epoch.

The usual complete deferred save/capture source and stale-publication barriers remain.
CLI, stdio MCP and authenticated loopback use this one registry entry. [Source policy and
independent evidence](verification-bond-order-command.md) distinguish actual saved-file
compatibility from future native click/alignment/keyboard workflows.


For the supported native carbon drawing, plain nonrepeat hotspot1/2/3 calls this setter
with absolute Single/Double/Triple order; it does not select a global tool. The initial
hit must come from a fresh visible foreground-valid canvas position. Its logical bond
identity may survive our own order edit while the pointer stays stationary, since ink
can move. New positional input re-hits the scene; history, tool epochs, foreign revisions,
focus/Escape/pointer loss, DPI/origin changes, text editing and blocked dialogs retire the
old target. A held gesture cannot redirect a numeric edit.

Single/Double tools click an existing Single to set Double; Triple sets Triple. Single-tool
endpoint drawing across an existing Single/Double sets Double/Triple. Unsupported click
combinations, including Double alignment requests, produce visible errors and preserve
content. These are the bounded [public-guide routes](research/bond-order-edit-plan.md),
not a universal26.1 tool matrix or an invented order/alignment cycle.

Edit shortcuts and Edit/Page menu activations execute at their raw event positions;
availability and typed invocations are resolved from current central controls at each
step. History invalidates gestures/hotspots and refreshes the whole stored document,
page dimensions, viewport and graph before later pointer input. The no-drawing fallback
retains ordered whole-SDF history. File presentation still has its separate existing
host lifecycle; this does not claim arbitrary cross-dialog input ordering.


## Transient drawing view

`ui.view` is one tagged, closed parameter object. The28-command registry exposes its
seven object branches through the same CLI, stdio MCP and authenticated loopback routes.
Examples:

```json
{"action":"inspect"}
{"action":"actual"}
{"action":"fit"}
{"action":"set_zoom","percent":175.5}
{"action":"magnify","width_px":820,"height_px":680}
{"action":"reduce","width_px":820,"height_px":680}
{"action":"pan","center_x_pt":240,"center_y_pt":400}
```

Unknown/unused fields refuse. Percentage is finite1..999; relative dimensions are integral
logical canvas units1..100000, despite the historical `_px` field suffix. Pan requires
explicit zoom and finite coordinates inside the current first page. `inspect` also works
for guarded/wrong-kind content and while a picker/close decision is pending. Mutating
actions respect those guards before even a no-op; committed close refuses this command.

Default Fit preserves legacy rounded paper geometry exactly. Explicit100% means one
authored point per logical canvas unit, independent of monitor DPI; it is not physical
screen calibration. Magnify multiplies the current effective percentage by1.25; Reduce
divides by1.25. Steps clamp to1..999 without decreasing an already larger fitted view.
Magnify alone can center the latest successful own Create/Extend segment midpoint, bound
to its page/generation. History and unrelated geometry invalidate this focus; Set Order
retains it. No imported atom ordering is interpreted as recent drawing.

`actual` and `set_zoom` retain center; `fit` clears it. Camera state persists across
Open/Undo/Redo as an original Bondline window policy. A center outside a newly opened
page is clamped during projection, without silently editing its stored state. Later
user navigation starts from that bounded displayed center.

Normal document snapshots omit camera state. Explicit replies add:

```json
{"ui_view_operation":{"zoom_percent":175.5,"center_x_pt":240.0,"center_y_pt":400.0,"epoch":2,"changed":true}}
```

Fit reports null zoom/center. Every real view change advances the checked transient epoch;
equal actions preserve it. Outermost authored `changed` stays false, and document/source
bytes, revision, dirty flag, history, identity floors, close epoch and CaptureStamp remain
unchanged. Loopback publication requests repaint only for actual view changes.

Central `ui.controls` adds View Actual/Fit/Magnify/Reduce with F5/F6/F7/F8. Relative bindings
are disabled until the host supplies current canvas dimensions with `set_view_canvas_size`;
this layout context is not an authored command. Direct relative calls carry their own
dimensions. Host text entry suppresses canvas shortcuts. Header Zoom and position controls
submit the same registry operations at their input-event boundaries.

The view builds geometry at the requested uniform scale, then clips it using signed
integer translation. It does not enlarge an old raster or resize paper. Labels, components,
physical-paper ink and complete-source guards remain whole-source checks. Some valid zoom
parameters still refuse rendering bounds/readability; accepted state is not a promise
that every document renders at every percentage. `render.capture` remains a whole-page
capture, independent of the window camera. SDF preview navigation, saved view preferences,
hand gestures and physical screen calibration remain unsupported. Source/evidence are in
[the view research](research/view-navigation-plan.md) and
[renderer verification](verification-view-render.md); native execution is separate.


Native File menu/shortcut intents now use the same ordered event dispatch as editing
and View controls. Each resolves its current central binding at the event boundary, so
Undo followed by Save captures the undone revision. An accepted pending intent prevents
later authored input from passing behind it; the established Running/workers/Save lease
and completion/close receipt contracts remain. This original input-order policy is
independently checked against actual saved bytes and natively repeated; see
[ordered-file verification](verification-ordered-file-actions.md).

## Native SDF existing-value drafts

The original occurrence inspector now calls the already registered `sdf.field.set_value`
through `UiSdfWork`. No command, parameter schema or automation protocol was added.
The UI uses the query's explicit source-local record/occurrence; duplicate names never
identify the edit. An explicitly omitted query value cannot become an empty draft.

Draft text is temporary, capped at65536 UTF-8 bytes with eight local text-undo snapshots.
Rejected widget mutations roll back atomically. Apply splits LF-delimited text literally;
empty text gives zero value lines. Existing setter ASCII/200-byte-line/value/JSON limits
remain authoritative. Invalid input stays available for correction and starts no Save.
One changed Apply is one existing complete-source history edit; a validated no-op retains
the complete state/source/index sharing. Save remains explicit.

Navigation, File and close resolve Apply/Discard/Cancel before a changed draft can be
abandoned. A same-input-packet Apply→Save waits for actual commit and worker retirement;
fresh source/file/close context is checked before any waiting action. A still-current
external `ui.file` request survives Discard/no-op; changed Apply uses a fresh binding.
Cancel uses `file.cancel` rather than acknowledging an unperformed request.

An external Open, even of identical bytes, detaches the draft and disables Apply while
keeping its text recoverable. A successful command before late cancellation stays applied;
the UI reports that outcome. Historical success cannot replace a newer document or run
an old waiting Save/close. Actual executor/reply/native barriers still govern shutdown.

Draft widgets, decisions and worker controls are not yet a general addressable dialog API.
Bounded Mac edit/save/history/reopen is verified; native race/accessibility and Windows/Linux
execution remain separate gaps. See [exact evidence](verification-sdf-editor-integration.md).
