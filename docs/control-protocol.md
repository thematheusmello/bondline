# Command contract

The delivered engine has one typed registry in `bondline-commands`. Names, Rust parameter
types, schemas and discovery metadata originate in the same declaration. The UI and external
adapters must call it; they must not implement their own authoring operations.
The delivered registry has47 commands. `bondline-cli commands` provides the authoritative
schemas; the current contracts are listed below.

## Existing plain caption runs

`text.set_run` is record47; all previous46 records/schemas/order remain exact. All five
keys are required in a closed object:

```json
{"command":"text.set_run","params":{"expected_revision":1,"page_id":1,"text_id":1,"run_index":0,"text":"Phase β & q"}}
```

IDs name application Page/Text objects, not source XML or font IDs; `run_index` is
zero-based. The command replaces decoded UTF-8 text while preserving style, position,
identity and every other run. Source-free/complete-native captions and the separate
explicit caption-only source capability are eligible. It does not grant chemical,
page-size or style authority on foreign captions. Stale revision, closed session,
unsupported source, invalid/missing target and unrenderable candidate refuse atomically.
A real edit receives one ordinary history step after final source-write and whole-page
render certification. Equal text preserves sharing, redo, close state and Save contexts.
Captured Save still acknowledges its captured snapshot; later edits remain dirty.

Native File Open/canvas/view/capture display accepted captions, and remote edits repaint
the same app. The canvas remains read-only and the full Text tool remains disabled.
Roboto Regular single-line/glyph/scale/page/work bounds are explicit; no font fallback
or chemical inference occurs. [Contract](research/styled-captions.md) and
[evidence](verification-styled-captions.md) separate actual Mac acceptance from headless
adapters, App frames and missing editor/platform workflows.

## Atomic document appearance and creation settings

`document.set_document_settings` appends record 46, retaining the previous 45 command
records, schemas and order. Its four required keys remain required in a closed parameter
object; an optional nullable `bond_spacing_percent` now extends this same record:

```json
{"command":"document.set_document_settings","params":{"expected_revision":1,"bond_length_pt":40,"chain_angle_degrees":120,"line_width_pt":0.6,"bond_spacing_percent":18}}
```

The three original values are nullable; null clears explicit data. Width is finite,
above zero through100000 points. Spacing omission preserves its current value, null
clears it and a number sets a finite percentage from1 through100. Typed nonfinite
values refuse serialization rather than accidentally clearing a value. Old wire requests
remain valid; Rust `DocumentSettingsSet` literals add the optional update field.

Explicit separation is percentage × preferred bond length /100. Absent preferred length
uses Bondline's original48pt convention only in its supported native/source-free profile.
Missing spacing retains the exact legacy3pt path. Width and label metrics are independent.
The final settings/style candidate is certified once when appearance or an active spacing
reference length changes. A closed or invisible gap refuses with `unsupported_scale`
before the single history commit; no intermediate preference-only candidate is authored.
Geometry, no-op sharing/redo, source/close/revision guards and Save leases stay intact.
The old pair command and Rust pair/three-field Apply helpers preserve unaddressed spacing.
File > Document Settings uses four bounded fields and the same registry entry; new native
width/spacing interaction remains unverified. See [spacing contract](research/document-bond-spacing.md)
and [receiving](verification-document-bond-spacing.md).
The weak Open request includes nullable `bond_spacing_percent` alongside the other
presented scalars; a blank value grants no authoring or Save authority.

## Shared request and result rules

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
Exact current native encoding byte equality permits the existing canonical edit path.
A separate closed Bondline-v0 profile now preserves equivalent lexical/ID/order variants
through supported typed updates/appends, while checking current identity/prefix drift.
Opaque extensions still refuse edits; exact undo restores original bytes/exportability.
This adds no command/schema or native verification. Whole-document MOL conversion remains
canonical-only; explicit fragment SMILES export continues to report omitted geometry/
document context. See [source-aware receiving](verification-source-preserving-cdxml-profile.md).
Failed imports retain the prior complete state/history/path. Registry saves encode through
StoredDocument's format-specific export guards; no command result includes raw source bytes. Native file-menu delivery
is a separate UI integration; general imported structural editing remains missing.
See model.md for the data boundary; serialized graphs are not a project-save workflow.

| Name | Parameters | Effect |
|---|---|---|
| `document.inspect` | `{}` | Read-only snapshot |
| `document.new` | Required `expected_revision`; optional `discard_unsaved=false` | Fresh source-free A4 Drawing session; clears file association/history and advances namespace/revision; retains saved files and outstanding work ownership |
| `document.set_drawing_settings` | All required: `expected_revision`, `bond_length_pt:number|null`, `chain_angle_degrees:number|null` | Replace both stored document-root preferences atomically in a complete native drawing; null clears, missing keys refuse; existing geometry/view unchanged |
| `document.open_cdxml` | `path`, optional `discard_unsaved=false` | Bounded native CDXML read; successful application starts a clean document session and clears history |
| `document.save_cdxml` | `path`, optional `overwrite=false` | Atomic complete-source save; default no-clobber; authoring revision/history retained |
| `document.open_cdx` | `path`, optional `discard_unsaved=false` | Bounded read of the supported CDX projection with complete source; new clean session and cleared history; A4 preview policy, faithful preview/editing unavailable |
| `document.save_cdx` | `path`, optional `overwrite=false` | Atomic exact unchanged CDX backing only; changed models, source-free encoding and lossy cross-format export refuse |
| `document.open_mol` | `path`, optional `discard_unsaved=false`, `points_per_unit=24`, `origin_x_pt=72`, `origin_y_pt=72` | Validate and decode the actual V2000/V3000 version; retain complete source/frame; same guarded new-session behavior as CDXML |
| `document.save_mol` | `path`, optional `overwrite=false`, `version="auto"` (`auto`, `v2000`, `v3000`) | Atomic MOL save with captured output version; refuses unsupported conversion/metadata/layout loss; imported source retains its frame |
| `document.open_sdf` | `path`, optional `discard_unsaved=false` | Validate every mixed V2000/V3000 record and retain complete ordered source; clean collection session and cleared history, without a drawing projection |
| `document.save_sdf` | `path`, optional `overwrite=false` | Atomic whole-source collection save; retains every record, MOL version, ordered field and byte; no molecular regeneration |
| `document.open_smiles` | `path`, optional `discard_unsaved=false` | Validate all supported notation records and retain the entire source; clean source-file session without a drawing projection |
| `document.import_smiles_records` | Required `expected_revision`, `selection:{mode:"record",record_id:N}` or `{mode:"all"}`, `bond_length_pt`, `origin_x_pt`, `origin_y_pt`, `page_width_pt`, `page_height_pt` | Deferred clean source-backed Drawing transition over the already-open complete SMILES file; explicit bounded selection, preserved opaque/unselected source, new namespace/history and retained file association |
| `document.save_smiles` | `path`, optional `overwrite=false` | Atomic exact current whole-file save, including supported synchronized record edits, ignored lines, opaque application bytes and original endings; refuses detached model/layout drift |
| `file.convert_smiles_to_cdxml` | Required `expected_revision`, `input_path`, `output_path`, `bond_length_pt`, `origin_x_pt`, `origin_y_pt`, `page_width_pt`, `page_height_pt`; optional `profile="neutral_carbon_path_v1"`, `"neutral_acyclic_v1"`, `"neutral_simple_ring_v1"` or `"neutral_components_v1"` | Deferred create-only conversion into new native CDXML; bounded carbon path, neutral tree, one3–8 Single cycle with attached trees, or all supported tree/ring components (64 total atoms). Current session and all its baselines remain unchanged |
| `file.export_smiles_fragment` | Required `expected_revision`, positive `page_id`, positive `fragment_id`, `output_path` | Create-only noncanonical explicit-H SMILES file for exactly one complete-source selected fragment; geometry/document context are explicitly not encoded and the current session is unchanged |
| `sdf.record.inspect` | Required `expected_revision`, positive `record_id`; optional `field_offset=0`, `field_limit=64` (1..64) | Deferred read-only ordered field page for one captured record; explicit value omissions and bounded compact JSON |
| `sdf.record.remove` | Required `expected_revision`, positive `record_id` | Deferred whole-record removal; exact surviving bytes, reindexed ordinals, one history edit; refuses the sole record |
| `smiles.record.inspect` | Required `expected_revision`, positive `record_id` | Deferred exact selected notation and global byte ranges/counts; opaque application data is not disclosed;128KiB object budget |
| `structure.paste_smiles` | Required `expected_revision`, positive `page_id`, `notation`, `bond_length_pt`, `origin_x_pt`, `origin_y_pt` | Deferred whole neutral graph append into a complete native Drawing; checked global IDs/page geometry, one history step, no file publication or Save acknowledgment |
| `sdf.field.set_value` | Required `expected_revision`, positive `record_id`, zero-based existing `field_index`, `value_lines` | Deferred immutable existing-field edit, one whole-content history commit; byte-preserving no-op and strict stale/input guards |
| `document.close` | `expected_revision`, optional `action="request"` (`request`, `discard`, `cancel`) | Shared conditional close intent; retains authored data/history; native shutdown waits for host acknowledgement |
| `ui.controls` | `{}` | Discover delivered menu/shortcut bindings, typed invocation and contextual enabled/disabled reason |
| `ui.atom_label` | Tagged `action`: `open` requires `expected_revision`, positive `page_id`, `fragment_id`, `atom_id`; `cancel` requires `expected_revision`, positive `request_id` | Queue or cancel one source-qualified native atom-label presentation; no authored edit/history or empty-target binding |
| `ui.drawing_settings` | Closed tagged `action`: `open` requires `expected_revision`; `cancel` requires `expected_revision`, positive `request_id` | Open/cancel one source-qualified native two-preference form; no authored edit/history, FilePlan or Save acknowledgment |
| `ui.bond_tool` | Required `order` (`single`, `double`, `triple`) | Select a transient drawing tool; explicit operation receipt/epoch, no authored edit or history entry |
| `ui.ring_tool` | Three closed alternatives: `size` (integer3..8), named `kind:"benzene"/"cyclopentadiene"`, or chair `kind:"cyclohexane_chair_1"/"cyclohexane_chair_2"` | Select a transient native ring tool; unchanged derived-size operation receipt, one shared tool epoch, no authored edit or history entry |
| `ui.chain_tool` | Closed tagged `select`, `constraints`, `open_add` or `cancel_add`; [details](#native-carbon-chain-tools) | Select a transient chain mode, set chain constraints or open/cancel source-qualified native numeric presentation; no authored edit/history |
| `ui.view` | Tagged `action`: `inspect`, `fit`, `actual`, `set_zoom`, `magnify`, `reduce`, `pan`; action-specific geometry/percentage | Inspect or change the transient camera without editing content/history; source-qualified display/readability guards remain |
| `ui.file` | `expected_revision`, `action`, conditional `request_id` / `close_epoch` | Request guarded New/SMILES forms or Open/Save/Save As presentation, or cancel the matching intent; a native host is required |
| `page.set_size` | `page_id`, `width_pt`, `height_pt` | One validated page edit |
| `bond.create` | `page_id`, `begin_x_pt`, `begin_y_pt`, `end_x_pt`, `end_y_pt`; optional `order="single"` | One independent neutral-carbon fragment with the requested Single/Double/Triple bond; each endpoint receives literal H=`4-order_weight` |
| `bond.extend` | Required `expected_revision`, `page_id`, `fragment_id`, `begin_atom_id`, `end_x_pt`, `end_y_pt`, `hydrogen_policy="rectify_terminal_carbon"`; optional `order="single"` | Add the requested bond/new carbon to a consistent supported terminal carbon, subtracting the bond weight from its literal H; native complete CDXML/source-free data only |
| `bond.set_order` | Required `expected_revision`, positive `page_id`, `fragment_id`, `bond_id`, explicit `order` (`single`, `double`, `triple`) and `hydrogen_policy="rectify_neutral_carbon"` | Edit one existing edge and both consistent neutral-carbon literal H counts; preserve geometry/IDs/direction, one history commit and validated no-op; native/CDXML or deferred synchronized SMILES record transaction |
| `atom.set_label` | Required `expected_revision`, positive `page_id`, `fragment_id`, `atom_id`, `element` atomic number1..118 and literal `explicit_hydrogens`0..65535 | Edit only one neutral/nonisotopic/nonradical atom's two literal fields in source-free/native CDXML; preserve IDs/geometry/neighbors and complete history, with no general valence inference. Physical H with positive attached H refuses; other backings keep their guards |
| `history.undo` | `{}` | Previous authored snapshot |
| `history.redo` | `{}` | Next authored snapshot |
| `render.capture` | `page_id`, `width_px`, `height_px`, `backend` | Read-only drawing page or restricted all-record SDF virtual page1 PNG; explicit `cpu` or `gpu` |
| `ui.selection` | Closed tagged `activate`, `set`, `all`, `clear` or `inspect`; required `expected_revision`; `set` also requires `page_id` and `atom_ids`, `all` requires `page_id` | Transient one-page native atom selection; no authored edit or native-host requirement |
| `selection.translate` | Required `expected_revision`, `expected_selection_epoch`, `delta_x_pt`, `delta_y_pt` | One atomic literal-position edit of the current native selection; complete source, chemistry, IDs and history retained |
| `ring.create` | Required `expected_revision`, `page_id`, one of the three size/named/chair alternatives, `hydrogen_policy:"rectify_neutral_carbon"`, closed tagged `placement`; named requests alone allow optional non-null `phase` | Construct one native carbon ring on paper or sharing a declared atom/edge; one whole-source history commit; [chair contract](#cyclohexane-chair-requests) |
| `ring.sprout` | Required `expected_revision`, `page_id`, `fragment_id`, `atom_id`, `hydrogen_policy:"rectify_neutral_carbon"`, closed object `ring`, `ring_begin_x_pt`, `ring_begin_y_pt`, `ring_end_x_pt`, `ring_end_y_pt`, string `side` | One new3–8 carbon cycle joined to the target by one Single, in the same Fragment and one history edit; explicit independent cycle-edge geometry, pure preview and unchanged source guards. Ring kinds: polygon+size3..8, benzene/cyclopentadiene+optional non-null phase, or either cyclohexane chair. [Contract](research/native-ring-sprout.md), [headless](verification-ring-sprout.md) and [native integration evidence](verification-native-ring-sprout.md). |
| `ui.preferences` | Closed tagged `action`: inspect; set+expected_epoch+sprout_rings_when_clicking; open+expected_revision; apply+expected_revision+request_id+sprout_rings_when_clicking; cancel+expected_revision+request_id | Inspect/set session-owned preferences or present/apply/cancel an exact native draft. Only presentation actions require a host. No authored edit, Save/close ACK or file-format field; [contract](#session-preferences-and-native-sprout). |

SMILES record import requires explicit selection of the already-open raw file. It starts
a clean Drawing namespace with generated geometry, retains the complete file association,
and publishes no bytes. Whole-file summary counts still describe all records;
projected_records identifies the visible subset. Model IDs belong to that namespace, not
record ordinals. Supported bond-order edits update model, current source and mappings in
one deferred history transaction; current source Save preserves unselected/opaque bytes.
Raw Open/reimport after saving creates a fresh namespace and does not persist 2D geometry.
Native import/navigation/input controls remain a separate worker gate. See
[independent evidence](verification-smiles-record-authoring.md).

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
`changed:false`. Real bond-tool changes advance the checked transient epoch; repeated
activation is a no-op. The remembered bond order persists across Undo/Redo/Open, outside document/source,
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
Source-backed documents without a supported complete display capability return
unsupported_content before rendering. Accepted caption-only files use their separate
display capability without granting chemical edits. Empty pages
and supported positioned neutral-element Single/centered Double/Triple plus explicit
Left/Right Double graphs can be captured, including safe branches,
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
The same commands additionally accept the strict canonical 28-byte implicit/explicit
Document + FileEnd envelopes after legacy TrailingData only, with all 16 reserved bytes
zero. Legacy successes win first; no new parameters, command, worker or encoding exists.
The original 258-byte external ethane opens at raw 0/45 pt with authored H absent, remains
`is_fully_mapped=false` and saves byte-exactly. Page edits can dirty it but Save refuses
until exact Undo. Six IO/engine/CLI/stdio/authenticated-loopback cases pass, with eight
syntax cases, 18 oracle controls and 10,000 ASan mutation runs. Bounded Mac Open, Unicode
Save As and explicit reopen retain all 258 bytes and Saved/CDX state. The native UI shows
Original source preserved plus faithful-preview refusal; 2 atoms/1 bond are IO/independent
consumer evidence, not displayed native counts. The final full gate passes1172/0/1 with
the original MCP assertions unchanged; its earlier intermittent replacement-bind failure
remains unexplained. See [the integration record](verification-cdx-file-end-integration.md).
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

`file.convert_smiles_to_cdxml` accepts an optional closed `profile`:
`neutral_carbon_path_v1` is the compatible default; `neutral_acyclic_v1` selects the
neutral-tree layout, `neutral_simple_ring_v1` the bounded single-ring layout, and
`neutral_components_v1` all supported tree/simple-ring components (64 atoms in total).
All eight existing scalar parameters remain required;
omitting/defaulting the profile retains the old serialized command and receipt policy.
There is no implicit geometry, overwrite switch,
discard flag, selection, native request ID or alternative H policy. Length must be finite
and positive; origin is finite within ±100000pt and page dimensions are positive and at
most100000pt. Runtime validation also covers direct typed Rust callers. A stale expected
revision fails `stale_revision` before file work. The command is **not read-only** in
registry/MCP metadata: it publishes a file, although authored state is unchanged.

The native IO operation reads the explicitly chosen complete source under its64KiB cap,
retains the notation owner while validating/layout/encoding, and creates a new CDXML file
through the existing atomic publisher. The default profile accepts only actual connected1–12 neutral nonisotopic
carbon/Single paths that satisfy fixed-H normalization, geometry and the1024reference
ink/page checks convert. Aromatic markers or bonds, including uppercase `C:C`, and other
unsupported content refuse. Input/filename suffix never selects a decoder or changes the
fixed CDXML output format. Output must not exist; input bytes and permissions are not edited.
This is conversion to a new drawing, not source-backed SMILES Open/Save or general cleanup.

The acyclic profile instead accepts an actual connected tree of at most64 atoms from the
existing neutral nonisotopic H/C/N/O/F/Cl/Br/I normalization, including physical H, branches
and Single/Double/Triple edges. Source graph/IDs/H remain intact. Origin means the minimum
atom-center x/y after rigid translation; the old profile keeps its endpoint anchor.
Bond length and paper are never silently scaled. Deterministic bounded alternatives must
pass real-edge/intersection, full stroke/glyph/paper and literal/resolved-scene checks.
Cancellation and exhausted budgets refuse.64 is a ceiling, not a fit guarantee.

Unassigned stereo stays unassigned. The supported degree-two internal-Double context has
collinear substituents at both ends; a qualified external consumer may report unknown
`STEREOANY`, never an assigned E/Z configuration. Potential degree-three internal-Double,
actual cycles/components, aromaticity, charges, isotopes and other unsupported chemistry
refuse in that acyclic profile. This adds no stereo perception or general source-file edit authority. See
[layout and integration evidence](verification-smiles-acyclic-integration.md).

The simple-ring profile requires one actual connected unicyclic graph, a3–8-atom Single
cycle and supported attached trees, at most64 total neutral nonisotopic H/C/N/O/F/Cl/Br/I
atoms under the same fixed-H policy. Physical H, source/IR/local identities and directed
edges remain exact. Supplied length is never scaled; minimum atom-center x/y across the
whole molecule anchors origin. Regular-ring orientation/contextual substituents, inherited
attached Single/Double/Triple/unknown-Double rules and the actual full literal/resolved
scene, stroke/glyph/paper and resource guards must all pass. The64-atom preflight reports
`resource_limit`; cancellation and exhausted geometry/ink bounds refuse without output.
Cycle Double/Triple, larger/polycyclic/disconnected/acyclic graphs, aromaticity, charge,
isotope and unsupported configurations refuse this profile. No partial first-ring result
or source-file editing authority follows. See [ring evidence and remaining limits](verification-smiles-ring-integration.md).

The component profile discovers actual-edge BFS components, not textual dots. It normalizes
once and keeps one complete Fragment with original global arrays/IDs/directed edges/fixed
H, changing only positions. Every component must satisfy the existing tree or 3–8 Single-
cycle/attached-tree rules; any unsupported member refuses the whole input. Solved poses
use original source-order shelves with gap L between atom-center boxes, supplied minimum
origin, height-aware width wrap and no rotation/rescale. K members divide the existing
100000/64 work/scene quotas per rotation; one final 100000-work whole geometry pass and
literal/resolved-H/stroke/paper certificate cover cross-component pairs. A single component
returns the corresponding old result byte-exactly. These are original layout/resource
policies, not general packing or reference clipboard parity.

Eight IO/three command/one schema/two actual-wire/three App focused cases pass unchanged, including complete
9-atom/7-bond/3-component C8H20O output, CPU/GPU equality and explicit Open/edit/history/
Unicode Save/reopen with independent C8H18O edited output. The full local gate passes1188/0/1;
bounded Mac fourth-profile conversion/refusal/Unicode retry, retained dirty source and
explicit Open Cancel/Discard, Double edit/history and Unicode Save/reopen also pass.
The two actual 1407-byte native outputs independently retain all 9 atoms/7 bonds/3 components
as C8H20O/C8H18O; only the intended first-edge order/H changes. Native authoring worked
at100%, while edited Fit78.79% refuses the distinct-pixel multiple-bond guard; initial top
label clipping and unsuccessful pan attempts leave full native overview/pan unverified.
[Integration](verification-smiles-components-integration.md) keeps these bounded checks
and earlier-profile/native-race/platform scopes separate. No registry count, required parameter,
receipt, default omission, source/close/lease/worker or Save-ACK authority changes.

Success adds optional `conversion` to the latest Engine state:

```json
{"conversion":{"profile":"neutral_carbon_path_v1","input_revision":0,"session_generation":0,"input_path":"/chosen/input.smi","input_byte_length":3,"atom_count":2,"bond_count":1,"output":{"path":"/chosen/new.cdxml","byte_length":408,"directory_synced":true}}}
```

Paths/byte lengths in that example are illustrative, not reference bytes. Counts and output
receipt describe actual work; `profile` reports the profile captured at preparation.
`input_revision`/`session_generation` are captured at preparation;
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

Generic native Open/Save presentation cannot bind this command. `prepare_ui_file` rejects it before
reservation; the generic plan binder explicitly refuses it rather than silently treating it
as Open/Save or ignoring an ID. A conversion cannot complete, clear or renew a trusted native
Save acknowledgment. CLI, stdio MCP and authenticated loopback use the same registry/worker
operation. The later specialized native conversion presentation is described below and in
[integrated evidence](verification-native-smiles-conversion.md). The original
[command evidence](verification-smiles-conversion-command.md) retains its headless scope.

### Native conversion presentation

The central `file.convert_smiles` File control has no shortcut and dispatches
`ui.file {expected_revision, action:"convert_smiles"}`. It adds a control, not a second
conversion command: that checkpoint retained30 commands. The later editable paste brings
the registry to31. Drawing exposes18 controls; readonly SDF12 at that checkpoint.
The native plan is `UiFilePlan::SmilesConversion`. Its form offers Carbon chains,
Acyclic neutral structures, Neutral ring structures and Neutral components in that order,
explicit geometry and separate input/new-output selection.
Profile actions replay in raw input order: a later choice cannot relabel an earlier Convert.
Pending/busy/browsing/stale/success states reject profile changes. Existing output refuses;
conversion browsing does not discard the current drawing. Default48pt bond/origin72pt/A4
are original UI choices, not claimed reference style defaults.

`Engine::prepare_ui_smiles_conversion(id,params)` requires the exact current Accepted
conversion request and prepares the existing eight-scalar operation with its captured profile. Success
consumes presentation before I/O, without binding a native Save request. Preparation failure
leaves the accepted intent available. `CommandExecutor` validates and reserves the existing
worker under its origin mutex. Generic bind/refusal and trusted Save/close guards remain.
The issued `ui_file_operation` integer is a presentation epoch, not a publication receipt.
Its disappearance does not establish that the background conversion has finished.

The host retains its actual job and thread through retirement, cancellation, close and
poison. Results use the real `conversion` receipt; current document/history/baselines stay
unchanged, including historical publication after replacement. Unstarted obsolete forms
and focus are dismissed; running/published outcomes remain reportable. Dropping the owner
requests cancellation. Explicit valid retry clears the previous error, while later unrelated
errors are not erased at finish. Native results direct ordinary File → Open; no automatic
Open/New or false Save acknowledgment. Native OS and remaining platform scope are stated
in verification-native-smiles-conversion.md, verification-smiles-acyclic-integration.md
and verification-smiles-ring-integration.md. The ring increment has bounded Mac C7
conversion/refusal/Unicode retry, guarded Open, external-bond edit/history and separate
Save/reopen evidence; geometry entry/browsing/races, large/bulky inputs and other platforms
remain unverified. Its full local gate passes1158/0/1. A measured-height re-layout presents the complete
success receipt without changing publication or ordered-input semantics.

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
and unavailable-preview status. The later original inspector and bounded native Unicode
Save/reopen are recorded in verification-smiles-record-integration.md. Chemical preview/
editing remains unavailable. CLI, stdio MCP and authenticated loopback use the same commands and real files.
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

Native Rust callers now have
`CommandExecutor::prepare_ui_smiles(&CaptureStamp, close_epoch, SmilesRecordInspect)`.
It checks current Open/source/close context under the originating mutex, reserves the
existing shared two-worker slot before spawning, and returns noncloneable `UiSmilesWork`.
Its blocking completion uses the same registered Core query/finish; no new wire tool,
Save lease or native file acknowledgement is created. Queries can be historical: a host
must retain pending/job state through actual thread retirement and reject stale display.
The library bridge has independent background/guard/budget/authority tests; App behavior
has separate evidence. See verification-smiles-ui-work.md and the original
research/smiles-native-workflow-plan.md.

The actual App now consumes that bridge in an original readonly record inspector.
Ordinal/Inspect/Previous/Next use the same existing query; cancel/retry/hide are local
presentation, with full source/close/request stale checks and actual-worker close gates.
Automatic metadata and external query success do not disclose notation in that UI.
No new command or wire-side selection is introduced. GUI lifecycle/input/file tests and
bounded native ordinal/Inspect/Previous and Unicode Save/reopen are scoped in
verification-smiles-record-integration.md. OS clipboard, native lifecycle races and
broader control addressability remain open.

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
local presentation controls add no named command or schema. Record removal brought the
registry to29 commands; document.new brought it to30 and structure.paste_smiles to31.

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

`ui.view` is one tagged, closed parameter object. The shared registry exposes its
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
explicit zoom and finite coordinates inside the first Drawing page or the prepared
all-record SDF extent below. `inspect` also works
for guarded/wrong-kind content and while a picker/close decision is pending. Mutating
actions respect those guards before even a no-op; committed close refuses this command.

Default Fit preserves legacy rounded paper geometry exactly. Explicit100% means one
authored point per logical canvas unit, independent of monitor DPI; it is not physical
screen calibration. Magnify multiplies the explicit percentage or declared logical Fit
percentage by1.25; Reduce divides by1.25. Steps clamp to1..999 without decreasing an already larger fitted view.
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
that every document renders at every percentage. Without `view`, `render.capture` remains
whole-page and independent of the camera. Optional current-view captures and SDF navigation
are below; saved preferences, hand gestures and physical screen calibration remain missing.
Source/evidence are in
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

## Opt-in current-view capture

First set/inspect the camera with `ui.view`; use its current epoch in the optional context:

```json
{"command":"render.capture","params":{"page_id":1,"width_px":1696,"height_px":1302,"backend":"cpu","view":{"logical_width":848,"logical_height":651,"expected_view_epoch":3}}}
```

`view` is closed; its logical dimensions are integral1..100000 and epoch is u64. Physical
output remains1..2048 per axis. Unknown/missing/invalid context fields refuse before work;
obsolete epoch returns `stale_view`. Absent or null `view` preserves the exact old
whole-page path, regardless of camera state. No second client-supplied camera definition
or implicit CPU fallback is used. Returned PNG schema is unchanged.

The source-qualified snapshot retains complete Drawing/SDF content and the captured
camera. Every SDF MOL record and display-H fact is validated before viewport clipping.
Physical geometry/glyphs are generated at actual density; the logical1..999 percentage
does not include DPI. Bounds, ink, physical glyph8..256 and cancellation limits remain.
Native presentation also checks logical glyph readability using actual output ratios.
An opted-in logical Fit differs from the legacy physical-margin whole-page Fit at2x.

Successful opted-in SDF capture can prepare its exact all-record pan bounds. It installs
only Weak source identity + PageSize if source/revision/generation, view epoch and Open/
close context still match at finish. Failed/cancelled work installs nothing; historical
successful PNG replies remain historical and cannot prepare a newer/foreign Engine.
Normal document snapshots stay unchanged. Explicit `ui.view` replies omit
`prepared_sdf_extent` until current bounds exist, then include `{width_pt,height_pt}`.
There is no public serialized cache setter or source/model/edit/Save/ACK authority.

SDF Actual/Fit/SetZoom work before preparation. Pan, and relative steps from Fit, return
`viewport_unavailable` until successful current-view preparation. Relative steps from
an explicit percentage need no invented bounds. Drawing mapped-source and wrong-kind
guards remain. Native `prepare_ui_capture_for_close` binds the pending close epoch
before reservation; a Rust-only notification flag wakes the host on first installation,
without entering JSON or changing authored/camera epochs. Default native Fit still uses
legacy capture; explicit zoom uses this opt-in route.

The legacy native SDF overview has physical margins, while relative-from-Fit uses the
declared logical fit formula. Their effective scales can differ above1x; exact native
relative-step consistency remains open. This does not change explicit percentage or pan
units. The native field now labels all SDF Fit states `Fit` and clears its numeric AX
value, independently of prepared bounds. Entering100 explicitly still selects Actual;
known Drawing percentages retain numeric presentation. See verification-fit-readout.md.

See [independent commands](verification-sdf-view-command.md),
[actual-App checks](verification-sdf-view-ui.md) and
[native scope and Root gates](verification-sdf-view-integration.md). Current-view output
is a page viewport, not operating-system window chrome, dialogs, print calibration or
complete publishing/clipboard support.

## Whole SDF record removal

```json
{"command":"sdf.record.remove","params":{"expected_revision":1,"record_id":1}}
```

The closed mutating command removes exactly one whole source-local ordinal from a
multi-record collection, including its delimiter and original ending. Surviving bytes
are untouched; reparsing assigns new ordinal IDs/ranges. Old ordinals must be discarded.
The format requires at least one compound: removing the sole record returns invalid_sdf;
a missing target returns not_found first. Wrong kind, stale input, cancelled/foreign
completion and committed close retain the common guards. No file write or new result
field is introduced. One successful edit advances revision/history; Undo/Redo restore
complete original/current snapshots. document stays null, collection stays SDF and normal
Save writes the full current collection. source.modified and file.dirty remain distinct.

CLI, stdio MCP and authenticated loopback expose the same typed command and31-name
catalogue. The edit shares the existing deferred field-candidate route and worker budget;
it has no save lease or native file/close acknowledgment authority.

The native Records list uses original Bondline confirmation controls, not a claimed
reference desktop screen. Changed field drafts resolve through Apply/Discard/Cancel
before a fresh confirmation. Keep record has no edit; Confirm record removal invalidates
old field/ordinal presentation. A same-packet Save waits for commit and actual worker
retirement. A committed result remains truthful after late cancellation.

Capture Rust-only UiSdfRemoveContext under the frame Engine guard, then release it before
prepare_ui_sdf_remove(&context, SdfRecordRemove). Under the origin mutex, this method
checks source/Open/close epoch, exact file path/format/file-request epoch and absence of
a request before reserving/preparing. Generic prepare_ui_sdf still accepts queries and
field edits only. Interposed requested/cancelled Save and direct SaveAs association
changes cannot reuse the confirmation. The independent regression reproduced a real
preparation gap before this guard; its RED is preserved separately from the unchanged
GREEN. This is preparation protection, not a global lock on subsequent explicit commands.

See [codec](verification-sdf-record-codec.md), [command](verification-sdf-record-command.md),
[App lifecycle](verification-sdf-record-removal-workspace.md),
[independent review](research/sdf-record-removal-review.md) and
[integration/native scope](verification-sdf-record-removal.md).

## Fresh document sessions

```json
{"command":"document.new","params":{"expected_revision":2,"discard_unsaved":false}}
```

The parameter object is closed; expected_revision is required. A stale revision refuses;
unsaved authored data refuses unless discard_unsaved=true. Pending close refuses ui_busy,
and committed close retains close_committed. New always starts a distinct session, even
when the prior document is already empty. It is not an Undo edit or a file publication.

Success installs a fresh source-free default A4 Drawing, clean current/saved baseline,
CDXML output format, no file association/source/collection/SMILES summary and empty history.
Revision, session generation and view epoch advance with checked arithmetic before any
mutation. IDs, prior SDF extent/focus and camera reset; Fit/centered presentation returns.
The same Engine, native host, selected tool, actual canvas dimensions and save reservation
remain. Old captured Save work can publish its own bytes truthfully; its older generation
cannot associate or clean this new session and its lease still lasts through finish/drop.
Existing source/revision/generation/foreign completion checks remain unchanged.

The central file.new control invokes ui.file actionnew, with primaryN. A native host is
required for presentation; the logical document.new command works headlessly. claim_ui_file
returns NewDocument. Rust-only create_ui_document validates the exact current Accepted New
intent under the origin mutex and calls the same registry command. New never produces a
trusted Save receipt, native close acknowledgment or background worker.

Native dirty replacement uses original Save/Discard Changes/Cancel decisions. Changed
SDF drafts resolve before a fresh New intent. Save retires that New intent and creates an
ordinary qualified Save. Only its exact current trusted receipt, unchanged full source/
revision/close context, clean baseline and actual worker retirement can resume New with
discard_unsaved=false. Failure/cancellation/replacement disarms New; an ordinary Save retry
cannot silently resume it. Close/poison cleanup retains the actual Save/result until drain.

The UI records the namespace of its last actually presented surface. Rust-only
Engine::session_generation is monotonic within that same Engine and unchanged by ordinary
edits/history/view changes; it is a cache key, not global identity or command authority.
A different New/Open namespace cancels old canvas gesture/hover/hotspot and fences raw
canvas authoring until the new surface is presented. Typed menu/key event ordering stays
intact. Full CaptureStamp qualification on asynchronous work is not replaced by this counter.

See [integrated evidence and native limits](verification-new-document.md),
[command](verification-new-document-command.md), [App](verification-new-document-ui.md),
[worker lifecycle](verification-new-document-workspace.md) and
[independent review](research/new-document-review.md). This single-window A4 workflow does
not establish style-sheet templates, multi-document windows or full26.1 parity.

## Editable SMILES append

`structure.paste_smiles` requires exactly six fields:

```json
{"expected_revision":0,"page_id":1,"notation":"c1ccccc1","bond_length_pt":64.0,"origin_x_pt":96.0,"origin_y_pt":96.0}
```

It parses one complete notation (at most64KiB), derives the bounded neutral fixed-H/
Kekulé graph and lays out all supported components in the supplied point units/current
paper. The source must be a complete native Drawing, rather than an opaque imported
projection or SMILES/SDF source session. Unsupported chemistry/layout, occupied or
unreadable whole-page geometry, cancellation and stale context refuse the whole edit.
No first-component or first-record fallback is permitted.

Preparation retains complete source identity without editing; work runs off-lock;
finish checks cancellation, revision/generation/owner and exact Open close epoch.
Success appends one complete Fragment with checked globally fresh IDs, creates one
history step, retains association/baseline and marks dirty. It publishes no file and
returns no file operation, conversion receipt or trusted Save/close acknowledgment.
Notation is omitted from Debug and automatic state. Save/reopen are separate commands.

See [source-qualified limits and independent evidence](verification-smiles-paste-engine.md).
Native Paste Special input/clipboard is not claimed by this engine checkpoint; its later
[native verification](verification-native-smiles-paste.md) establishes the explicit form.

## Native SMILES presentation and registered work

The existing `ui.file` action `paste_smiles` requests the native **Edit → Paste Special →
SMILES…** form; it requires a complete native Drawing and attached host. No clipboard
bytes are read by the command itself. Ordinary TextEdit OS paste fills a private bounded
draft. `ui.controls` exposes `edit.paste_smiles`, optional `submenu:"Paste Special"` and
the typed presentation binding. Older controls omit the new None submenu/false Alt fields.
Ctrl+Alt+P is wired off macOS; the Mac shortcut is intentionally unknown/unbound.

After the host accepts `UiFilePlan::SmilesPaste`, it captures opaque
`UiSmilesPasteContext` for the actual displayed page. Release the frame mutex before
`CommandExecutor::prepare_ui_smiles_paste(&context, PasteSmiles)`. Under one origin lock
the adapter revalidates context, reserves the shared two-worker permit and prepares the
same registry command before consuming the intent. Full capacity preserves the accepted
intent. Work runs off-lock and rechecks UI context under the same lock as engine finish.
Drop/failed start releases unused work; no FilePlan, Save lease or finish_ui_file ACK exists.

The opaque file-context identity invalidates even if path/format return to old values.
Native successful work also carries a Rust-only `ui_file_context_stamp`, captured atomically
with finish and omitted from every JSON reply. The UI checks it before adopting success
and at actual thread retirement. It cannot authorize Save/close. Historical success stays
truthful and retains copyable text; stale drafts cannot reapply. Pending/job/form barriers
and frame-start input fencing prevent early close, Save or replay onto the new drawing.
An explicit Cancel may discard the draft after worker retirement. Automatic clipboard
capture, native races/other-platform execution and complete reference parity remain open.

## Native source-backed SMILES records

`ui.controls` additionally exposes `file.import_smiles_records`, File → Import SMILES
records…, with no shortcut and typed `ui.file` action `import_smiles_records`. It is enabled
only for a raw open SMILES file, attached host and idle/Open presentation context. Default
counts are38 native Drawing/15 SDF/11 raw SMILES; retained-SMILES Drawing keeps21.
The current registry has45 commands. Accepting
`UiFilePlan::SmilesImport` opens explicit One record/All records plus the existing five
geometry parameters. Import calls only `document.import_smiles_records`; it reads the
already-retained file, not a new path. The backing's generic source-completeness flag
stays false. View/tool discovery accepts the validated projection, while page/create/
extend/paste/cross-format operations remain refused.

The host captures `UiSmilesRecordsContext` before releasing the frame mutex. Specialized
`prepare_ui_smiles_import` reserves shared capacity before consuming Accepted intent;
`prepare_ui_smiles_order` accepts only the exact captured page/fragment/bond and typed
`BondSetOrder`. Both run registered work off-lock and validate source/revision/close/view/
tool/file epochs and opaque association identity atomically with finish. Returned private
`UiSmilesRecordsResult::is_current` qualifies presentation, not Save acknowledgment. A
Running Save returned to its same-ID/same-epoch Accepted retry invalidates preparation,
finish and even a previously successful no-op result proof.

Native click and numeric order gestures queue only one pending/actual thread. New File,
history, page, tool and canvas mutations wait; View/Close remain available and invalidate
old context. An already-Running Save retains its existing publication/lease semantics.
Pending work and the actual JoinHandle participate in close and frame-start input fences;
an idle executor or ready result channel does not establish thread retirement. Explicit
retry rearms a consumed failed import only while its original source/context remains
current. No file operation/receipt, Save lease or trusted close ACK is added by import or
order. [Receiving evidence](verification-native-smiles-records.md) separates App/worker
tests from actual platform execution and exact reference-screen parity.

## Stored document drawing preferences

`document.set_drawing_settings` is one immediate authoring command, exported as
`DrawingSettingsSet`. Its closed parameter object requires all three keys:

```json
{"command":"document.set_drawing_settings","params":{"expected_revision":1,"bond_length_pt":24.5,"chain_angle_degrees":120.5}}
```

Both values accept number or null; explicit null clears the authored property. Omission
is invalid, never a partial update. Length must be finite >0 through100000pt and angle
finite1..179°, fractional allowed. Two nulls restore absent model/native-root fields;
they do not select a numeric or inherited default. Existing pages/chemistry/geometry,
view focus, camera, tool and ID floors remain unchanged.
Typed request serialization rejects nonfinite Some values instead of letting JSON encode
them as null/clear; explicit None retains null. Domain validation still runs in the engine.

After the ordinary committed-close guard, expected revision and Drawing/source authority
precede domain validation and equality. Only source-free or completely mapped native
CDXML may edit; foreign/opaque/noncanonical CDXML, CDX/MOL, SDF and raw/retained SMILES
refuse `unsupported_content`, including equal values. Stale revision is `stale_revision`;
missing/unknown/type/domain errors are `invalid_params`. All refusals are atomic.

A validated no-op retains model/source sharing, redo and weak/close/file contexts. A real
change uses one normal history commit, clearing redo/selection and invalidating old close/
file/content contexts. Immediate preparation already authors the change; later cancellation
does not roll it back. No worker, Save reservation, receipt or native presentation binding
is added. Existing Save snapshot semantics stay: same-generation completion may associate
its path and older captured baseline while the newer styled current document remains dirty.
Its historical input revision does not acknowledge the newer edit as saved.

Whole-MOL V2000/V3000 Save refuses nonempty preferences before publication. Native CDXML
persists present values; unchanged source and explicit graph-only export retain their
existing policies. [The contract](research/drawing-settings-command.md) distinguishes
this stored-data operation from tool application, inheritance and style sheets. The
native form below reuses it. Native controls are now38/21/15/11; the catalogue has
16 delivered/124 disabled, including the later chain and chair controls.

## Native drawing-settings presentation

`ui.drawing_settings` is one additional presentation command with two closed actions:

```json
{"command":"ui.drawing_settings","params":{"action":"open","expected_revision":1}}
{"command":"ui.drawing_settings","params":{"action":"cancel","expected_revision":1,"request_id":1}}
```

Open requires an attached host, the atomic command's exact source authority, Open close
state and no other File/Atom/settings request. It captures source allocation/revision/
generation, opaque file association, File/close/view epochs and effective tool. Its checked
monotone request ID is separate from atom/bond IDs. Result `ui_drawing_settings` contains
exactly `id`, `input_revision`, nullable `bond_length_pt`, nullable `chain_angle_degrees`,
nullable `line_width_pt` and `current`. Successful Open/Cancel returns `ui_drawing_settings_operation` with that
ID, while `changed` remains false. Both notify the native repaint callback after reply
publication. No FilePlan, file worker, Save receipt or close acknowledgment is created.

File > Document Settings… has three 384-byte bounded fields: Bond length (pt), Chain
angle (degrees), and Line width (pt). Absent values start blank and clearing means null.
The trusted host's Apply rechecks the exact request and executes
`document.set_document_settings` under the same Engine guard. Validated no-op closes the
form without changing redo/history; failed Apply retains the draft. Existing geometry
and view remain unchanged. The request's private cancellation token is not serialized:
only exact explicit Cancel marks it. Remote Cancel therefore dismisses its form; host
detach or context changes instead retain a copyable stale draft with Apply disabled.
Cancel needs the current revision and attached host even for a stale request. Local
Cancel can also dismiss its obsolete draft after detachment or committed close.

The new `file.document_settings` control is always listed, disabled for unavailable host,
unsupported content or foreground work. App modal-at-start fences prevent same-frame
history/canvas/File actions after dismissal; native close waits for Apply/Cancel.
Bounded core/App/CLI/stdio/authenticated-loopback and actual Mac persistence evidence is
in [the earlier pair receiving](verification-native-drawing-settings.md). The new width
field has [App/adapter/render/file evidence](verification-document-line-width.md), with
actual native interaction unverified because controller startup fails. This implements
two creation preferences and explicit root line width. Chains resolve preferences
separately below. Inheritance, other tools/settings categories,
complete reference-dialog parity and Windows/Linux native interaction remain missing.

## Native carbon-chain tools

`ui.chain_tool` appends one command, preserving every old42 command record/schema/order.
All actions and nested targets are closed tagged objects:

```json
{"command":"ui.chain_tool","params":{"action":"select","kind":"acyclic"}}
{"command":"ui.chain_tool","params":{"action":"select","kind":"snaking"}}
{"command":"ui.chain_tool","params":{"action":"constraints","fixed_lengths":false,"fixed_angles":true}}
{"command":"ui.chain_tool","params":{"action":"open_add","expected_revision":0,"page_id":1,"target":{"mode":"free","x_pt":200,"y_pt":200}}}
{"command":"ui.chain_tool","params":{"action":"cancel_add","expected_revision":0,"request_id":1}}
```

An atom target instead has exactly `mode:"atom"`, positive `fragment_id` and `atom_id`.
Free coordinates use the existing±100000pt finite bounds. Source-free or byte-proven
native CDXML is required; foreign/opaque sources retain refusal before equality/no-op.
Select/constraints need no native host. Constraints require active Chain mode; otherwise
the direct command returns `unsupported_content` and its disabled native control reports
`chain_tool_required`. Real changes advance the shared checked effective-tool epoch,
retaining remembered bond order. Both switches initially true and survive New/Open as
transient tool policy, never authored document data. Result `ui_chain_tool_operation`
has kind,fixed_lengths,fixed_angles,epoch,changed; outer changed remains false.

Open Add requires an attached native host, current revision, source/H/target/page
authority and idle File/Atom/settings/Add/Open-close context. Its checked monotone ID
also advances a modal-open epoch when no request remains. Result `ui_add_chain` contains
exactly id,input_revision,page_id,target,count,bond_length_pt,angle_degrees,current.
Default count6 includes an existing anchor. Each missing supported native preference
independently resolves to original48pt/120° without storing it or dirtying the document.
Unknown/inherited source settings do not gain guessed defaults or editing authority.

The384-byte count editor accepts trimmed ASCII2..64. Host preview tries bounded numeric
candidates against the actual full scene, paper, logical/physical ink and viewport.
Apply recomputes the same candidate index under the same Engine guard and invokes
existing chain.create once; drag invokes chain.create_path once. Old geometry/H/source
rules and one-edit history remain. Add alone seeds count repeat with a new POST-commit
proof captured under that guard. Alt-click resolves a fresh target/preferences; stale
repeat opens Add instead. Tool/constraints/view/source/File/close/association/modal or
actual canvas/DPI/layer changes disarm repeat. Returning equal values cannot revive it.

The weak request/context captures source allocation/revision/generation, opaque file
association, File/close/view/shared-tool and Atom/settings/Add open epochs. Exact explicit
Cancel checks current revision/host/ID and marks only its private serde-skipped token;
ordinary staleness/detach instead retains copyable drafts with Add disabled. Local Cancel
may dismiss an obsolete form. Open/Cancel return ui_add_chain_operation ID and wake the
native host after reply publication; changed tool operations also wake it. No worker,
FilePlan, Save receipt or close acknowledgment is created.

Native controls chain.acyclic/chain.snaking use these actions. Object > Chain constraints
contains chain.fixed_lengths/chain.fixed_angles. Delivered controls total38/21/15/11;
the additional four appear only for native Drawing and preserve existing source/modal/
close precedence. [The interaction lock](research/native-chain-interactions.md) defines
ordered physical modifiers, original count/pose/turn/reversal and repeat policies.
[Receiving](verification-native-chains.md) separates actual-App/adapters/files from
bounded Mac execution and unverified native Control-drag/DPI/other-platform acceptance.

## Native tool catalogue presentation

The native **All tools…** catalogue is roadmap presentation, not additional executable
registry entries. All124 documented roles/140 expanded entries are visible;124 planned
entries have no binding and cannot dispatch. Available entries use the same current
delivered control IDs. `ui.controls` lists delivered bindings only, now counting
38/21/15/11. The headless chain commands brought the registry to40, the stored-preferences
command to41, its native presentation to42 and chain presentation to43. Only the later
checked chain workflow binds Acyclic/Snaking. The later checked chair workflow adds
both chair bindings without another command:16 available/124 planned entries.
See [catalogue](design/tool-catalogue.md).

## Exact-target atom-label presentation

Explicit atom.set_label independently restores existing terminal and nonterminal carbon
display flags from false to true, including dormant flags and same element/H entry.
This is one real authored edit;
Undo restores false. None/true same-count calls keep old validated sharing/no-op behavior.
It does not require successful rendering or a C4 proof before accepting supplied literal
element/H, and it preserves all existing source/target/close guards. Parameters, command
count and native form remain unchanged. The original form's valid unchanged Apply can
therefore restore a hidden anchor; see [skeletal receiving](verification-skeletal-carbon.md)
and [terminal-label evidence](verification-terminal-carbon-labels.md).

`ui.atom_label` uses the attached native-host signal already used by `ui.file`.
The closed variants are:

```json
{"action":"open","expected_revision":1,"page_id":1,"fragment_id":1,"atom_id":2}
{"action":"cancel","expected_revision":1,"request_id":1}
```

Open validates revision, host availability, idle file presentation/Open close state and
the same source/target guards as `atom.set_label`. An existing atom request, even stale,
refuses with `ui_busy`. The optional `ui_atom_label` snapshot contains `id`,
`input_revision`, `page_id`, `fragment_id`, `atom_id`, `element`, `explicit_hydrogens` and
`current`. The initial target/fields never retarget. `current` checks weak source/revision/
generation, opaque file context, file/close/view epochs, tool and attached host. Open and
Cancel return `ui_atom_label_operation:ID` with `changed:false`; authored revision/history,
source, saved baseline, camera and tool remain unchanged. Request IDs increase on Open.

Cancel requires current revision and exact request ID, including a stale request; wrong
IDs refuse atomically. The global committed-close refusal still applies to commands.
Rust-only `finish_ui_atom_label(id)` can retire that exact host presentation after close
commits, without granting an edit, Save or close ACK. Host detach clears presentation.

The native form's Apply calls `apply_ui_atom_label(id,element,explicit_hydrogens)` under
the same Engine guard. It revalidates currentness, executes the existing registered
setter exactly once and retires only on success/validated no-op. A failure retains the
draft. The cheap operation adds no worker or Save lease. It adds no catalogue binding;
clients provide a real target rather than invoking an empty-target menu binding.

Existing loopback notification follows response flush and activity release for the
explicit operation marker; failed publication does not pretend a host repaint happened.
Headless CLI/stdio sessions return `ui_unavailable`. [Receiving evidence](verification-native-atom-labels.md)
records actual adapters and native Mac scope; it does not establish complete Text-tool
or cross-platform parity.

## Persistent native selection and translation

`ui.selection` is a closed tagged parameter object. Examples at the current revision:

```json
{"action":"activate","expected_revision":3}
{"action":"set","expected_revision":3,"page_id":1,"atom_ids":[2,1]}
{"action":"all","expected_revision":3,"page_id":1}
{"action":"clear","expected_revision":3}
{"action":"inspect","expected_revision":3}
```

Activate/set/all enter selection mode. Set validates the page, accepts unsorted IDs,
sorts them, and rejects zero, duplicates, missing atoms or more than100000 entries.
Empty set is allowed after page validation. Clear empties targets and keeps the current
tool. Inspect changes nothing. Activate/set/all require source-free or completely mapped
native CDXML Drawing; clear/inspect work across content kinds. No native host is required.
Transient mutations follow the existing close/file-busy tool policy. Inspect can read a
pending-close selection; all commands still refuse after committed close.

Replies include `ui_selection:{active,epoch,page_id,atom_ids}` and
`ui_selection_operation:{epoch,changed}`, with outer `changed:false`. Ordinary state
omits selection until its first actual change. Empty selection has null page_id and no
IDs. A real mode/target change advances the checked selection epoch. Mode changes also
advance the existing bond-tool epoch; only Selection or the remembered bond order is
effective. Selecting the same remembered bond order exits Selection and clears targets.
No-op/inspect/error requests do not repaint. Authenticated loopback publishes a real
selection change before releasing request activity and requesting host repaint.

```json
{"command":"selection.translate","params":{"expected_revision":3,"expected_selection_epoch":1,"delta_x_pt":24,"delta_y_pt":-12}}
```

Translation checks revision, finite deltas, complete native source authority, current
nonempty selection/epoch/private weak owner and every authored position. It replaces
affected same-ID fragments in one whole-document validation and one history commit.
Only selected positions change. Atom metadata, literal/absent H, directed bonds,
topology, arrays, pages and unrelated shared data remain exact. This performs no H or
stereo inference, snapping, layout, publication or Save/close acknowledgement. MOL,
CDX, opaque CDXML, SDF and raw/retained SMILES do not gain geometry-edit authority.

Zero or rounded-identical translation still validates everything, then retains source,
model sharing, selection, redo, close intent and saved baseline exactly. A real move
keeps the same IDs, requalifies their owner and advances selection epoch once. Other
real edits, Undo/Redo and namespace replacements clear nonempty targets and advance
their epoch; no-op/failure/Save/view/pan/DPI does not. Overflow is checked before any
mutation. Stable errors include `invalid_params`, `not_found`, `unsupported_content`,
`stale_revision`, `stale_selection` and `ui_selection_epoch_exhausted`.

Rust-only `selection_context()` supplies weak source/revision/generation, selection,
tool/view/close/file and Atom/settings/Preferences-open epochs, plus opaque file-
association proof. Raw active Atom/settings/Preferences ownership refuses capture;
exact Cancel permits a fresh proof without reviving a prior one. Canvas/DPI qualifiers
stay in the UI; losing a canvas retires disposable selection input before equal surface
restoration. `preview_selection_translation` builds the same validated immutable
candidate without committing, changing Engine state or granting Save authority.

Three shared controls are added only for eligible Drawing sources: `tool.selection`
(Marquee selection), `edit.select_all` (primary+A) and `edit.deselect` (no shortcut).
The native adapter supports atom/bond-endpoint click, connected-component double-click,
Shift toggle, anchor-based rectangular marquee, fresh selected-target drag and
nonrepeat1pt/Shift10pt arrows. A preview is transient; release is one command. Escape
cancels a gesture while retaining targets; otherwise it clears targets. Text/modal and
frame-start barriers win. Tool activations resolve their fresh binding in raw input
order so later mode changes cannot suppress earlier authoring. Full lasso, transform,
group, snapping and exact26.1/platform parity remain open. See
[command/App evidence](verification-selection-authoring.md) and
[native acceptance/cancellation receiving](verification-native-selection.md) for the
tested execution scope. Selection/history/source state is unchanged by cancellation.

## Explicit native ring construction

`ring.create` is an authoring command shared by CLI, stdio MCP and authenticated
loopback. The legacy sized request requires the current revision, positive page ID,
size3..8, explicit `hydrogen_policy:"rectify_neutral_carbon"` and a closed `placement`:

```json
{"command":"ring.create","params":{"expected_revision":0,"page_id":1,"size":6,"hydrogen_policy":"rectify_neutral_carbon","placement":{"mode":"free","begin_x_pt":120,"begin_y_pt":168,"end_x_pt":120,"end_y_pt":120,"side":"positive"}}}
```

Free requires both first-edge endpoints and side. Atom requires `fragment_id`,
`atom_id`, `end_x_pt`, `end_y_pt`, side; its existing atom is the first endpoint.
Bond requires `fragment_id`, `bond_id`, side and preserves that Single bond's authored
begin→end. Side is exactly positive/negative algebraic cross-product half-plane in
right/down point coordinates. Every variant is closed with all listed fields required.
No implicit placement, default length, snap, rotation or merge is substituted.

Only source-free/fully mapped native CDXML is editable. Free adds n C/n Single bonds
with H2 in a new Fragment. Atom adds n−1 C/n bonds and consumes2 literal target H.
Bond adds n−2 C/n−1 bonds and consumes1 H at each endpoint, retaining the shared edge.
Shared atoms must be positioned neutral natural nonradical carbon, have only supported
incident orders and literal H equal to4 minus incident Single/Double/Triple weights.
Unknown/inconsistent/insufficient H and other source/target kinds refuse explicitly.
No general valence/stereo inference or neighbor adjustment is performed.

Old arrays, IDs, shared positions, directed edges, unrelated metadata and shared storage
remain exact. New IDs follow perimeter order with closure last; attachment consumes no
new Fragment ID. Exact new-vertex occupancy on the same page refuses; no tolerance-based
fusion occurs. Finite±100000pt positions and a normalized post-rounding regularity
certificate precede the single complete-source history commit. The certificate has
1e-10 relative length/angle-in-radians limits and preserves accurately represented tiny
geometry rather than imposing a minimum length. Paper/readability guards are separate.

Revision precedes source/target validation after the global committed-close guard.
Failures preserve source/history/selection/saved baseline/file context and spend no
IDs. Actual creation uses the existing selection invalidation and checked ID floors;
Undo/Redo restores whole content. The command retains camera/focus, adds no worker,
file publication, Save lease or acknowledgement, and has no separate result receipt.
Codes include invalid_params, not_found, unsupported_content, stale_revision and existing
revision/identity/model/selection-exhaustion errors.

The command-only checkpoint exposes37 commands and adds no native binding: controls remain23 native
Drawing,20 retained-SMILES Drawing,14 SDF and10 raw SMILES. Ring palette/gesture/preview,
aromatic/chair/sprout tools, styles and broader chemistry/platform workflows remain
required. See [independent command/adapter/render evidence](verification-ring-authoring.md)
and [the locked public-source contract](research/ring-authoring-plan.md).

### Named integer-bond rings

The same command also accepts `kind:"benzene"` or `kind:"cyclopentadiene"` in place
of `size`. Its optional `phase` is `double_first` or `single_first`; omission is
contextual. Explicit null, both/neither selectors, phase on a sized request and unknown
fields are invalid. New kind/phase fields accept strings only. The registry still has38
commands; its schema is two disjoint closed object alternatives. The public Rust
`RingCreate` stays unchanged and converts into `RingCreateRequest::Sized`.

```json
{"command":"ring.create","params":{"expected_revision":0,"page_id":1,"kind":"benzene","hydrogen_policy":"rectify_neutral_carbon","placement":{"mode":"free","begin_x_pt":120,"begin_y_pt":168,"end_x_pt":120,"end_y_pt":120,"side":"positive"}}}
```

Benzene uses cyclic integer weights2,1,2,1,2,1; cyclopentadiene uses2,1,2,1,1. Rank
their2/5 unique left rotations by index. Explicit phase filters the first edge; free
omission chooses rotation0. Attachments choose the first rotation matching the unchanged
shared Single/Double edge and proven literal target H budgets. Atom attachment spends
both new incident weights; edge attachment spends only the new closing/next weights at
its authored B/E endpoints. Old H consistency is proved once per target. No fitting phase
means unsupported_content, never a changed shared edge or ignored explicit phase.

New carbon H is4 minus its two integer weights; new atoms explicitly suppress eligible
nonterminal carbon labels. New Doubles face inward: Right on positive side, Left on
negative. Every old atom flag and shared bond order/direction/DoublePosition remains
exact, including centered/unspecified/outward shared Doubles. The same geometry/source/
identity/history guards apply. No aromatic flag, general perception, resonance rephasing,
automatic fused-face restyling or H repair is inferred.

The [native integration](verification-native-unsaturated-rings.md) now routes two named
palette controls, preview and ordered Shift gestures through this same command builder.
See [the locked policy](research/unsaturated-ring-authoring.md) and
[receiving evidence](verification-unsaturated-rings.md). All known command names now
enforce the documented JSON-object parameter boundary before struct deserialization;
unknown names retain unknown_command priority.

### Cyclohexane-chair requests

The third closed `ring.create` alternative uses kind exactly
`cyclohexane_chair_1` or `cyclohexane_chair_2`, with the same required revision/page/H
policy/placement fields and **no size or phase**. Null, enum-map wrappers, combined
selectors and unknown fields refuse. `ChairRingCreate` converts to
`RingCreateRequest::Chair`; public sized/named Rust APIs remain unchanged. Registry
count stays43. Only ring.create/ui.ring_tool schemas gain an additive branch.

```json
{"command":"ring.create","params":{"expected_revision":0,"page_id":1,"kind":"cyclohexane_chair_1","hydrogen_policy":"rectify_neutral_carbon","placement":{"mode":"free","begin_x_pt":120,"begin_y_pt":120,"end_x_pt":168,"end_y_pt":120,"side":"positive"}}}
```

The original six-point orthographic template is transformed by that exact first edge
and side. Its unequal foreshortened edges use a separate1e-10 normalized rounded-similarity
certificate; the regular-ring certificate and point-domain/source guards remain intact.
Six Single edges use the existing free/atom/Single-edge H and identity accounting.
New atoms have literal H2, nonterminal-label=false and absent terminal flag. All old
metadata/geometry/directed edges remain, apart from consumed target H. Double/Triple
shared edges refuse. Constitution is editable; stereo is unspecified.

Native free click uses a horizontal pose and independently stored BondLength, or
original48pt only for known absence. Drag rotates/scales the whole candidate. Atom
click aligns its centroid outward using existing deterministic neighbor ties; bond
fusion tries the existing preferred/opposite complete candidates. Only unmodified
primary boundaries are accepted. Contextual nonrepeat9/0 needs a fresh actual bond
hotspot, keeps the active tool and commits through the same full-scene candidate.
No pointer means no authoring or tool selection; future movement cannot authorize
an earlier key. Text/modal/busy/active-gesture/source/view/close/physical guards stay.
Read [original policy](research/native-chair-workflow.md) and
[scoped independent/native evidence](verification-native-chairs.md); Alt-sprout,
stereo/ring flips and broader platform/reference parity remain separate work.

## Native ring tool, context and preview

`ui.ring_tool` accepts either `{"size":6}` (integer3..8) or a kind-only object such as
`{"kind":"benzene"}`, or the third closed chair-kind alternative above. Named kinds
are exactly benzene/cyclopentadiene; chair kinds are cyclohexane_chair_1/2 strings. Null,
enum-map wrappers, both/neither selectors, extra fields and phase are invalid. It requires a source-free
or completely mapped native CDXML Drawing and the existing tool close/file policy;
no native host is required. Eligible controls are tool.ring.3…8, labelled Cyclopropane
through Cyclooctane plus tool.ring.benzene/Benzene and
tool.ring.cyclopentadiene/Cyclopentadiene and tool.ring.chair_1/2, with no global chair
shortcut. Current native controls total38; retained-SMILES/SDF/raw stay21/15/11.
Every tool's selected state comes from one
EffectiveTool enum; a ring never leaves a bond button selected.

The explicit result alone includes `ui_ring_tool_operation:{size,epoch,changed}`;
ordinary snapshots omit it. Named size is derived6/5; that field alone does not prove
which tool is selected. Query ui.controls for the exact selected binding. Outer changed
is false; chair size is derived6. The same validated exact mode/size/kind is
an exact no-op. A mode/size change advances the shared checked tool epoch once;
leaving Selection preflights and clears its targets. Choosing the remembered bond
order from Ring is a real mode switch. Authored history/Open/New retain the mode.
Loopback repaint follows successful changed-only publication and activity release.

Rust-only ring_context(page_id) requires the actual Ring mode, supported page/source,
Open and no outstanding file intent, including Running. RingContext is weak and has
no serialized representation or Save authority. Source/namespace/history, tool-size/
mode/kind, view, close, file intent, association ABA and Atom/settings/Add open epochs
invalidate it permanently, including hidden Open→Cancel→None. Native
canvas/DPI/layer/origin qualifiers remain in the thin UI.

preview_ring_create(&RingCreate) and additive preview_ring_request(&RingCreateRequest)
are mode/host-independent, refuse committed close
before stale revision, and reuses the actual ring candidate builder. It spends no ID
floor, clears no selection and changes no authored/saved/history/file state. Native
release additionally requires that same complete candidate's ordinary scene and
logical8..256 label-em guards pass, then revalidates the context and executes ring.create
once. The headless command retains its explicit point-domain geometry contract.

Named mode captures phase only at the ordered primary press: no modifiers means
contextual phase, Shift alone means single_first. Move/release cannot change it; release
accepts none/Shift, while Ctrl/Alt/Command combinations refuse. Every modified sized
gesture still refuses. Side trials retain the same phase and cannot detach on error.
The same full-candidate visibility and context checks precede one release/commit.

The locked original48pt poses, unit-neighbor bisectors, local bond-side preference,
latched drag threshold and original sized modifier policy are in the
[native contract](research/ring-native-plan.md). They are not observed26.1 defaults.
[Receiving](verification-native-rings.md) separates actual App inputs and saved-file
consumers from unexecuted native/platform acceptance.
The [named contract](research/native-unsaturated-rings.md) and
[named receiving](verification-native-unsaturated-rings.md) extend that scope with
original procedural icons, exact-mode contexts and press-latched Shift. App/DPI tests
remain distinct from real OS/monitor and full26.1 behavior.

## Explicit headless carbon chains

```json
{"command":"chain.create","params":{"expected_revision":0,"page_id":1,"count":5,"angle_degrees":120,"hydrogen_policy":"rectify_neutral_carbon","placement":{"mode":"free","begin_x_pt":64,"begin_y_pt":112,"end_x_pt":112,"end_y_pt":112,"side":"positive"}}}
```

This one closed typed request serves Engine, CLI, initialized stdio MCP and authenticated
loopback. All fields are required; count2..64 includes a shared anchor. The explicit
angle1..179° is the chain's interior angle; no default length/angle/orientation is inferred.
The alternate atom placement supplies mode:"atom", fragment_id, atom_id, end_x_pt,
end_y_pt and side, omitting begin coordinates. Positive rotates the second edge by
180°−angle with cross(first,second)>0 (visually clockwise on a y-down page); negative
reflects it. Subsequent edges alternate those two fixed vectors. See
[the locked contract](research/chain-authoring-plan.md) for precision and occupancy.

Source-free or completely mapped native CDXML is required. An atom anchor needs known
consistent literal H on neutral natural nonradical C and enough budget after weighted
Single/Double/Triple incidence. The edit consumes one anchor H and appends count−1 C;
it preserves old directed bonds/styles/coordinates/flags and unrelated data. Free
placement creates count C in one fragment. New termini have H3, internal C H2; both
new carbon-label flags are false. The existing isolated C2 mandatory labels still apply.
These authored flags retain literal H and existing explicit format-loss refusals.

One immediate candidate commits one history edit. IDs are checked and their session
floors publish only after success, remaining monotone across Undo/Redo. Invalid/stale/
unsupported/occupied/rounded-precision requests leave content, history, source, file and
selection unchanged. Successful editing uses normal selection and close/file-context
invalidation. Like ring.create, prepare commits synchronously; later cancellation
cannot reverse a completed edit. An outstanding captured Save remains historical and
retains its existing reservation through finish.

That headless increment adds one registry command (38→39), with no new native binding.
At that checkpoint both chain entries stay disabled and native workflows remain required.
The later [native carbon-chain tools](#native-carbon-chain-tools) provide their separately
checked controls/forms/gestures; exact hotkeys and outstanding platform acceptance stay open.

## Literal headless chain paths

```json
{"command":"chain.create_path","params":{"expected_revision":0,"page_id":1,"hydrogen_policy":"rectify_neutral_carbon","placement":{"mode":"free","points":[{"x_pt":144,"y_pt":144},{"x_pt":192,"y_pt":144},{"x_pt":192,"y_pt":192},{"x_pt":240,"y_pt":192},{"x_pt":240,"y_pt":144}]}}}
```

The new closed request creates one literal ordered carbon path in one edit. Free
points has2..64 closed{x_pt,y_pt} objects; atom placement instead supplies fragment_id,
atom_id and1..63 new_points objects. The exact old atom position is prepended; no count,
angle or side field is accepted. All fields are required. Coordinates are finite within
the model±100000pt bound; point arrays/extra fields refuse. Typed Rust points validate too.

Unequal edges, mixed turns,180° interiors, represented tiny edges and crossings are
literal data, with no snapping/rotation/inference. Duplicate vertices, zero/nonfinite
edges and new centers occupying old same-page centers refuse. The shared target-local
H/source/ID/history builder preserves old fields/styles/geometry and appends neutral
Single-bond C with literal H and both false label flags. Existing mandatory labels and
explicit export-loss refusals remain. Full native ink/paper guards are additional.

Rust-only preview_chain_create/path returns the same fixed/literal candidates without
editing, reserving IDs, changing context or granting Save/gesture authority. Both require
current revision and reject committed close. Path vectors are borrowed and checked
before resolved allocation. Existing fixed ChainCreate types, fields, schema, defaults
and command variant remain exact. CLI/MCP/authenticated loopback expose both commands;
That registry40 prerequisite adds no native binding; chain entries stay disabled at that
checkpoint. The later [native carbon-chain tools](#native-carbon-chain-tools) implement
the [dependency/workflow contract](research/native-chain-plan.md) for their scoped evidence.
See [receiving](verification-chain-paths.md) for exact evidence and limits.


## Session preferences and native sprout

`ui.preferences` adds one typed command with five closed object actions; every listed
key is required and unexpected/null/scalar fields refuse. Inspect is exactly
`{"action":"inspect"}`. Set supplies expected_epoch and a boolean
sprout_rings_when_clicking. Open supplies expected_revision. Apply additionally supplies
an exact positive request_id and that boolean; Cancel supplies the exact request_id.
The ordinary committed-close guard includes this command.

The original initial value is false; this is a Bondline policy, not a claimed26.1
factory default. Values live only in this Engine session, survive New/Open/Save/history
and host detach, and never enter authored snapshots or chemical files. Real changes
advance both preference and shared-tool epochs once after checked preflight, retaining
selection/mode/remembered order and source/history. A stale expected_epoch fails
ui_stale_request before no-op detection. No-op Set/Inspect retain all input proofs.
Restart storage, shared multi-window preferences and other categories remain missing.

Presentation requires an attached host, Open close state and idle foreground ownership
across source families. A captured weak source/file/view/tool/preference proof prevents
stale Apply and ABA revival. Matching stale Cancel remains possible at the actual
current revision. Only explicit matching Cancel marks its request token; host/context
loss retains the local draft for dismissal. A separate monotone open epoch invalidates
old Bond/Ring/Selection/Chain gestures even after Open/Cancel disappears between frames.
Headless authoring and captured Save retain their prior rules and can stale the form.

Normal snapshots omit preferences; explicit command replies include
`preferences:{sprout_rings_when_clicking,epoch}`. Active drafts appear as ui_preferences
with id/input_revision/sprout_rings_when_clicking/current; the boolean is its captured
draft value. Inspect/no-op Set omit the operation.
Real Set emits `{action:"set",changed:true}` without request_id. Open/Cancel/Apply emit
action, exact request_id and whether the boolean changed; Open/Cancel are false, while
unchanged Apply still retires its form. Outer authored changed remains false. Only
operation-bearing replies wake the native host, after reply publication/activity release.
They grant no Save receipt, source rewrite or close acknowledgement.

The native File > Preferences… form exposes Building/Display, one real checkbox,
Use Defaults, OK and Cancel. Draft/default actions are local; OK uses the registered
Apply. Raw activations replay in order and terminal retirement fences the remaining
packet. Stale drafts retain local values with OK disabled. Atom-targeted ring presses
choose sprout when preference OR Alt is active; other targets and unsupported chords
retain explicit refusals. Named Shift phase is latched at press, and a late Alt cannot
convert an ordinary gesture. Click/drag generate the same complete ring.sprout candidate
and existing full-scene guards. Exact original policies/limits are in the
[lock](research/native-ring-sprout.md); [receiving](verification-native-ring-sprout.md)
distinguishes independent App/adapter evidence from actual native platform checks.
