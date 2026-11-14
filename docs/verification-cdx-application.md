# Guarded CDX application files

This slice starts from43ca90d in the isolated bondline-mol-v3000 checkout, branch
feat/cdx-file-commands, combining metadataf67883a and the previously validated00e5b02.
The baseline local gate is453 workspace plus3 macOS bridge tests, session89159 exit0,
target/cdx-application-base-ci.log. Root's newer renderer/native acceptance remains
separate: this worker changes no UI rendering/input/gesture code or root checkout.

## Delivered boundary

Fifteen commands share one registry. document.open_cdx takes closed parameters path
and optional discard_unsaved=false; document.save_cdx takes path and overwrite=false.
Both use the existing FilePlan/Completed, source snapshots and per-Engine save lease.
Open validates the supported CDX decoder/projection, retains every byte, applies its
explicit A4 preview policy and starts a clean associated cdx session with empty history.
Engine::from_document also initializes the source's CDX family without inventing a path.
Missing H stays absent; BoundingBox does not infer paper. JSON includes source size/
modified state and the elemental projection, never warning text or opaque source bytes.

Save publishes only exact unchanged backing. Edited models, source-free encoding,
CDX-to-CDXML/MOL metadata loss and unsupported input refuse. Exact Undo restores the
source baseline and its exportability. All imports stay is_fully_mapped=false; default
capture/preview and bond creation remain unavailable. No native encoder, style fidelity,
chemical inference or general CDX compatibility is claimed.

Native format hints/choices/extensions, associated Known Save, Save As, exact failed
retry and dirty-close failure handling are wired in files.rs/file_workflow.rs. They
use the same named commands via the shared executor and its reservation/request/worker/
reply/close barriers. Existing controls/shortcuts remain authoritative shared metadata.
No second dispatcher or file worker exists. CDX close-Save refusal keeps the window
open and its exact format/path retry; cancellation uses the existing shared policy.
These are synthetic/worker tests, not an actual native chooser observation.

## Test-first and independent evidence

- target/cdx-commands-red.log, session99926 exit101: four intended runtime REDs for
  missing command/schema rows and UnknownCommand(document.open_cdx), before production.
- cdx-commands-first-green.log: three successes; one negative test used width/height
  instead of the real width_px/height_px capture parameters and returned invalid_params.
  Its parameters were corrected without changing the intended unsupported-preview check.
  cdx-commands-green.log then passes five core cases; a sixth adds trusted Known Save.
- cdx-ui-routing-red.log: the new FileFormat::Cdx exposed six exhaustive missing UI
  routing/label cases at compile time. The first routed run found an incorrect test
  expectation using a noncanonical path; the expected canonical associated path was
  corrected. Production path/receipt semantics were retained.
- cdx-files-scoped-tests.log: new loopback test incorrectly reused a connection after
  invalid_params, which intentionally closes it. The correction uses a second client
  and still proves that the shared document survives. Existing protocol behavior was
  preserved. cdx-files-scoped-green.log passes all188 scoped tests in commands/MCP/CLI/UI.

Core commands save all four existing original licensed fixtures:65750-byte legacy
explicit,65746-byte header-document,323-byte metadata explicit and319-byte metadata
header. Their exact bytes and complete backing survive save/reopen. The actual saved
files face the unchanged cdx_oracle.py, cdx_header_oracle.py and cdx_metadata_oracle.py
stdlib-struct assertions and mandatory external RDKit2026.09.2 consumer. Reference
InChI/OPSIN and Bondline chemistry calculations are not part of this slice. RDKit's
external CC/C2H6 result is not authored H or an implemented chemical analysis feature.

Actual CLI, stdio MCP and authenticated loopback processes save the319-byte metadata
header vector and feed those files to the same independent consumer. Expected SHA-256
is9af210eb978d085b143fad97074d55aa6512c44e02370d4a41e0376d06176a39;
positions are literal(10.25,-3.5)/(26.25,8.5), two C/one Single, missing H remains null.
Discovery asserts all fifteen exact names and both closed schemas/default permissions.
Negative paths cover schema/dirty-open/cross-format/malformed/preview/source-free/changed-
source refusals and exact state/history/bytes. Deferred tests exercise stale open, save
cancellation, busy through published completion, slot release and current trusted receipts.
The loopback Save result has changed=false and still triggers the receipt repaint.

## Checks and continuation

All work uses this checkout's own absolute CARGO_TARGET_DIR, CARGO_INCREMENTAL=0,
CARGO_PROFILE_DEV_DEBUG=0/CARGO_PROFILE_TEST_DEBUG=0 and the existing root
.cache/oracles/format-python/bin/python through absolute BONDLINE_PYTHON. Python bytecode
generation is disabled. Scoped all-target warnings-denied clippy passes; its log is
target/cdx-files-scoped-clippy.log. The complete workspace gate and exact final hashes
are recorded below after execution; native acceptance and hosted execution remain pending.

No codec/model/renderer, dependency, manifest, lock, reference asset or approved golden
changes. All reference/native1196-byte artifacts remain ignored and unmodified. Remaining
M1 work includes broader mappings/foreign files, source-preserving editing/encoder, faithful
preview and actual native/platform CDX workflows. The existing general corpus gaps remain
qualified; this is an original-vector file workflow, not full application compatibility.

The final own-worktree cargo xtask ci passes465 workspace plus3 native bridge tests
(468 total), session80939 exit0. It prints the invoked workspace as
/Users/sarenaberg/Workspace/doozle/open-source-world/bondline-mol-v3000 and includes
workspace rustfmt, warnings-denied all-target clippy, strict layers, declared licenses
and the unchanged five-file/four-source-bundle corpus. Native closures remain154 Mac,
165 Windows and220 Linux plus two bundled assets. The exact ignored log is
target/cdx-file-commands-ci.log, SHA-256
96315fe2786f1f26b05d5b02f67160c18ede6404e1dec26dd7c3e914e9decf7f.
This is local macOS execution, not new hosted or interactive Windows/Linux evidence.
No new parser/fuzz dependency closure or native application was launched for this slice.

Final intentional source/docs/log identities, immutable guards and the exact next native
acceptance steps are in target/cdx-file-commands-handoff.json. Root owns reviewing,
committing, merging its newer renderer, publishing and the new exact native/hosted pass.
No worker Git operation, root build/edit, live job or GUI/controller remains.

## Receiving integration

Root merges e37d129 onto a3613c0 (the newer graph/input implementation, source-only
metadata and test-only font correction). The five application production files match the
frozen worker handoff exactly; Root's six graph/input/capture production files and approved
golden remain unchanged. Three documentation conflicts were resolved by preserving both
the newer records and the incoming application scope. No runtime dependency/lock changes.

The exact receiving `cargo xtask ci` passes499 workspace plus3 native bridge tests,
502 total, session95800 exit0. target/cdx-application-integrated-ci.log SHA
7c5a36e4b5dcaa5ec9db510056876f9e5e92581aaf9a611d24522aa6e264c99e records the correct Root
workspace, formatting, all-target clippy, layers, unchanged license closures/assets and
independent corpus. Native CDX acceptance remains pending on this frozen integration;
earlier renderer/file/worker tests are not a substitute for the actual chooser workflow.


## Actual native macOS acceptance

Frozen source450962c was built and run on macOS27.2 arm64 / Apple M3 Max / Metal,
using pinned Rust1.94.1, Root's absolute target, incremental0 and default dev debuginfo.
The379-file Rust/Cargo/WGSL manifest is
8137960861d59212592ae77a3f1fd605b0fa9084fc2bd08267e19fa506aee0d0;
the executed17,799,320-byte binary hashes to
e9491a720572889372264ebea3cf111d147f3beeae67e0c4367e6fe1d66e5749.
All379 source hashes and that binary were checked again after cleanup and independently
by Root before changing its checkout. This is the exact15-command native build; the later
16-command terminal-extension source has separate headless/hosted evidence.

Actual native chooser/keyboard routes pass:

- Open the319B input under a misleading .mol suffix by explicitly choosing CDX after
  the filename hint; CmdS retains its associated cdx encoding and every original byte.
- Display the explicit unavailable-preview state; three settled frames remove prior MOL
  paper/graph/hover. A drag cannot edit the incomplete backing. No faithful preview is claimed.
- Page>Letter makes it dirty; Known Save, one Replace File retry and dirty-close Save
  refuse regeneration without changing the old bytes/inode/mtime. Cancel and CmdZ restore
  the exact clean source baseline; a subsequent Known Save succeeds.
- Malformed and unsupported double-bond imports retain complete current source/model/path,
  authored revision and history/redo. UI request/epoch changes are tracked separately.
- Unicode Save As and output Open preserve all bytes for each original319/323/65750/65746B
  fixture. Those actual outputs pass their unchanged independent struct/RDKit consumers.
- The previously authorized ignored1196B native own-drawing file also goes through actual
  native Open/Unicode Save As/reopen, byte-exact. A separate strict header decoder checks
  all99 records, the distinct FileEnd and exact EOF; external RDKit reads CC/C2H6. Its
  SHA remains7b2b71338595ad12bc34f46459f8389ec6a63cb453b1cf29b51543717139a73c.
  Those native bytes/prose/assets remain ignored and unredistributed.
- Native Save As CDXML and MOLV3000 refuse metadata loss before destination creation;
  source-free CDX Save As also refuses. MOLV2000 is independently refused through the
  live controller, rather than claimed as a chooser observation. Live open_cdx/save_cdx
  also exercise the native engine directly with a truthful unchanged-save receipt.

Matching CLI commands discovers15 closed registry tools; live ui.controls proves only its
nine File/Edit/Page bindings. No loopback registry-list operation was invented. CUA,
public CG/AX, authenticated control and external consumers are separately labeled in the
ledger. No native same-frame timing, broad renderer retest or new golden is implied.

Exact ignored report: target/native-cdx-application-handoff.md, SHA
2a38d2af6b1dd43aa04ccc5b5ad0f3dc853c2245f967d79238c289833eca21ab.
Its15-group ledger30-native-flow-ledger.json hashes to
cabb297e3209571241e571c9491fa4a1ae541d15e39f0c8f03cdf59b7c86c5f0;
31-evidence-index.json to
f8a85505001816c3fe5e43d3aba0f24ab552a1bbbe238072df4e8b9d93bff2d6 (83 finalized files,
all rehashed by Root). Cleanup28-final-cleanup.json SHA
06d786457e161ca0dc50d76744207e38757ac111ace445417488a840241b0c23 records native24284 /
controller24261/session54751 exited0 and no held input/owned live handle. The four sampled
reference windows retain IDs/frames/layers; no reference operation occurred in this pass.
Cursor was restored to the nearest integer with subpixel delta(+0.305,-0.213); exact
subpixel restoration is not claimed. CUA completion occurred once after cleanup.

Three excluded evidence probes stay recorded: an initial own offscreen-window selector,
a mistakenly resolved venv executable corrected without dependencies, and fractional
cursor-warp truncation. They are not product failures or passing workflow evidence.
The earlier historical reference-window restoration incident is not rewritten.

Own source450962c hosted43559974203 passes Mac502/Linux500/Windows492, all actual windows,
and six10000-run ASAN targets; ci.md records the job identities/adapters. Native chooser
acceptance above is Mac-only. CDX imported editing/new encoding/faithful styles and preview,
foreign/general corpus, interactive Windows/Linux and full M0/M1 remain incomplete.
