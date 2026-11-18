# SMILES authoring and complete-source import: next gates

2026-11-15. Design study only; no implementation, tests, build or native interaction.
The existing four converters and read-only file inspector are prerequisites, not complete
SMILES import/export. Full M0–M5, aromatic/charged/stereo chemistry and clipboard parity
remain required.

**Recommendation:** deliver the documented **Paste Special → SMILES → editable native
drawing → Undo/Redo → Save/reopen** next, including real benzene support. Follow it with
source-backed record editing and whole-file Save. Do not spend another release merely
installing a read-only selected-record Drawing or adding a fifth converter choice.

## What the primary evidence establishes

- [OpenSMILES v1.0](https://github.com/opensmiles/OpenSMILES/blob/9240a2887158699503605bc838e1de9a9f7883f4/opensmiles.asciidoc), SMILES Files: a file has zero or more line records; optional SP/TAB-separated application data is outside the notation. Disconnected Structures explains why dots are not component counts (`C1.C1` is connected). Its Aromaticity section gives benzene in aromatic and Kekulé forms. These facts do not define a drawing application's selection, layout or save policy.
- [GUIDE25](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf), printed350–351, visually checked: selected-text Paste Special creates a structure; Copy As produces text, with periods between multiple copied structures. The aromatic/Kekulé benzene example is present. Printed358 describes file embedding; the reviewed360–363 format tables do not establish a `.smi` selected/all-record drawing workflow. Exact insertion position, selection/focus behavior, record-name interpretation and26.1 multi-record import remain unknown. These are25 documentation facts, not a new installed-app observation.
- Pinned PhotoCraft [document metadata](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/crates/doc/src/lib.rs), [conversion reports](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/crates/doc/src/import_conversion.rs) and [jobs](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/crates/engine/src/jobs.rs) separate opaque retained payloads, historical source/target reports and live operations. Jobs apply snapshot results through one history step. A historical conversion report is not editing/resource authority. Reuse those distinctions, not their code, report framework or asset graph.

Web fetching the17,721,447-byte guide failed with a content-size limit; the hash-checked
local public copy supplied the complete relevant pages. OpenSMILES and the immutable
PhotoCraft sources were fetched successfully and matched the existing local PhotoCraft
copies. No proprietary binary, icon, font or help prose enters the product.

| Reviewed public bytes | Bytes | SHA256 |
|---|---:|---|
| OpenSMILES, commit9240a288… | 86816 | `00f0423da612de913770133aa13e456d0cebc562dd50166534a8a7f9a834bd21` |
| GUIDE25 PDF | 17721447 | `3f1ebc429df07efdd47b6218a148dc346a215043f60bf89ee9a1cb90629bd5b3` |
| PhotoCraft doc, commit0fb40c53… | 65401 | `11d5769d2bf41adc0b2e410da5b04aba3eb7429e201453b4c6dc38296e4d5363` |
| PhotoCraft import_conversion, same pin | 18380 | `8b9d6b52ce836d36d16ce252384b05238dd0db16f6952fd9173075ee3d19f82e` |
| PhotoCraft jobs, same pin | 107608 | `03242c486f21e643c5cec7fff8e379ff4970c2b5cc6dd6255221330aabdd1594` |

## Gate A: useful editable paste, with chemistry prerequisites

Proposed single authoring command: `structure.paste_smiles` with closed required
`expected_revision`, `page_id`, `notation`, `bond_length_pt`, `origin_x_pt`, `origin_y_pt`.
Paper comes from the captured target page. UI, CLI, stdio and loopback execute this same
operation. The native menu needs a narrowly typed clipboard intent in the same registry
if the central binding requires presentation parameters; it must not implement a second
chemical operation. Existing converter APIs/profiles/defaults/refusals stay exact.

Accept a complete notation, not a file or its first token. Reject multi-record input,
opaque suffixes, malformed/unsupported chemistry and over-budget text atomically; never
trim a `.smi` name or silently take its first line. Current64KiB/64-atom layout ceilings
remain explicit initial resource policies, not full SMILES support. The command's owned
input survives work and is omitted from automatic state/Debug. Native clipboard text is
not an imported file backing or a promise to preserve arbitrary original bytes.
Parser text limits and serialized command/reply budgets are separate; test actual JSON
envelopes before promising a maximum-size notation through every adapter.

**Benzene is an acceptance requirement, not a later optional demo.** Today the parser
retains aromatic markers, normalization rejects them, the simple-ring solver rejects
multiple ring bonds, and the renderer does not provide a general aromatic depiction.
Add a real, separately sourced chemistry projection/valid deterministic Kekulé assignment
or an explicit aromatic model, plus matching ring geometry and export semantics. The
existing Single/Double native format makes a validated Kekulé route a practical option;
the decision needs independent aromatic/H/valence/stereo vectors before code. Do not turn
lowercase atoms into uppercase without chemical validation, erase aromatic orders, add
decorative circles or special-case a benzene picture. Qualify benzene, pyridine and
`[nH]` contexts before claiming broader aromatic support. Charged salts, stereogenic
atoms/bonds, fused rings and larger structures remain explicit required work, with whole-
input refusals until their real representations exist. Add no user-facing fifth profile.

Reuse the pure layout/certificate machinery, but new aromatic behavior belongs behind
the new operation/chemistry boundary; do not weaken frozen neutral-converter negatives.
Remap every pasted atom/bond/fragment to checked document-global IDs, preserving direction
and physical H. Append one batch via `Document::append_fragments`; one paste is one history
step. Validate the assembled target page, including existing ink and cross-fragment pairs,
not just the pasted graph on an empty synthetic page. Never shrink or move existing work
to obtain fit. UI placement/style defaults are original policy until separately observed;
the core takes explicit geometry and a stale/occupied/out-of-paper target refuses.

Use the stronger native-source guard of `bond.set_order`/`bond.extend`: source-free or
byte-proven native CDXML only. `bond.create`'s completeness check alone is insufficient
because fully mapped MOL can still reject appended fragments at Save. Preserve the full
StoredDocument source/baseline, path, saved snapshot and unrelated Arcs. After append,
ordinary registered Save must encode the complete native document; Undo/Redo must preserve
the source and identity floors. No temporary converter file, engine replacement or fake
Save acknowledgment is involved.

Native seam: pinned egui0.29.1 exposes `ViewportCommand::RequestPaste`; eframe's native
wgpu integration reads its existing clipboard provider and sends `Event::Paste`. It
normalizes CRLF to LF and sends no event for empty/unavailable text. Thus the host needs
an explicit Empty/Unavailable completion, potentially a narrow reviewed bridge; a pending
intent cannot wait forever or infer success from time. Do not promise raw clipboard-byte
identity. Consume a Paste event only for the explicit qualified SMILES intent, before
TextEdit consumes it. Ordinary text paste, focus/modal/Escape, source/view/close changes
and frame-start input fences remain intact. Reserve the existing worker slot before
spawning; work off-lock and commit only against its exact originating snapshot/revision/
generation. Keep pending/work/result/thread retirement in the existing close barriers.

## Gate B: genuine source-file record authoring, not a lossy projection

Keep `document.open_smiles`/Save unchanged: complete immutable file, metadata-only summary,
`document:null`, exact passthrough. Add an explicit drawing-import operation over that
owned source with mandatory closed `selection:{mode:"record",record_id:N}` or
`selection:{mode:"all"}` and explicit layout geometry. No default first record. A drawing
selection of zero records refuses; a later unsupported selected record refuses the entire
selection. Unselected records may be unprojectable but must remain byte-exact. Existing
whole-file syntax validation and64MiB/65536-record/per-notation64KiB bounds still apply.
Initially keep an explicit aggregate projected-atom budget; it is not a permanent general-
import definition. All-record behavior must ship with its tests, not an invisible fallback.
Proposed `document.import_smiles_records` captures the already-open complete file, expected
revision, selection and geometry; it does not reread a path or reinterpret clipboard text.

Use one Fragment per record, including that record's disconnected components. Allocate
global IDs in record/source-array order and retain bounded maps from captured ordinal,
local atom/bond ordinal and absolute token spans to global IDs. Ordinals are snapshot-
relative, not chemical identities. Never merge records, deduplicate equal structures or
split on dots. Use shared geometry/layout and whole-page certificates with aggregate
quotas; existing local helpers' `AtomId−1` assumptions require dense local views before
global remapping. Preserve source/current mappings through every supported edit.

The file-authoring gate must include the actual writer transaction, not only a read-only
Drawing. Extend StoredDocument backing with **original and current complete
SmilesFileDocument**, model baseline and shared projection/maps. Engine current/saved/Undo/
Redo retain that whole object; account for every new map/vector and shared source/baseline
allocation in the existing128MiB/128-entry policy. Do not grant `is_fully_mapped` merely
because a picture exists. Add operation-specific eligibility only when an entire selected
record's chemistry is represented and its edited notation can be independently verified.
`io::display_facts` currently requires that global completeness flag; selected-record
display therefore needs an explicit projection capability and current mapping check,
separate from full-source export or edit authority, with the selection visible to users.

Proposed immutable `with_record_notation(id,notation,cancel)` replaces only the existing
notation span, preserving separator, opaque/nonUTF8 application bytes, ignored lines,
original endings and every other record. Reparse under the original limits before
returning a candidate. A genuine no-op must share the old source/index instead of
normalizing its spelling. Reuse the explicit-H writer only within its proved coverage;
its DFS can reorder atoms, so extend its internal result with a generated-token→global-ID
map rather than assuming reparsed local IDs equal the old IDs. Atomically commit the
edited model, current full file and refreshed maps in one history step. Original source
and the last saved snapshot remain distinct.

Begin with existing supported record edits, such as consistent carbon `bond.set_order`,
then extend coverage; do not claim aromatic/charged/stereo editing before the shared
chemistry gate supports it. Source-file Save writes the complete **current** file via
FilePlan/Completed and its existing lease, not a Fragment writer. SMILES cannot encode
authored paper/style/2D geometry: keep generated layout distinguishable from authoring,
and refuse Save/clean acknowledgment for unsupported layout edits. Explicit derived
CDXML export may state excluded source context/unselected records and leave the source
session unchanged; it must not masquerade as lossless source Save or bypass `to_cdxml`.
New-session import follows existing guarded Open/unsaved decisions, generation and clean
baseline rules. Later edits use ordinary dirty/history semantics. Stale, cancelled,
foreign-source or committed-close completions cannot install a candidate; historical
published saves still report their actual effect without reassociation.
The first file-edit gate may permit graph edits that leave generated positions/paper
unchanged; coordinate/page/style editing needs its own persistable or explicit-loss path.

## Implementation seams and decisive evidence

| Seam | Required work; existing reusable boundary |
|---|---|
| Chemistry/model/io | Aromatic semantics and geometry; existing SMILES IR, normalization, components, explicit-H writer, StoredDocument, whole-file spans and `append_fragments`. Keep all existing public profiles exact. |
| Commands | New paste/import schemas and candidates; shared ID floors/history; source-edit eligibility; `content.rs`, `files.rs`, `capture.rs`, `bond.rs`, `file_ui.rs`, controls and retained accounting. Source summary/results must not disclose opaque data. |
| Native/MCP/CLI | Central Paste Special binding, qualified clipboard completion and ordered UI workflow; existing shared executor/two-slot/reply/close path. Actual adapters call the same authoring operation. No new pool or JS/frontend. |
| Formats/config/tests | New backing must update all encoder/source/display/accounting matches; no silent `write_document(model)` escape. Update closed schemas/catalogue counts, layer/license checks only if real code requires it; old fixtures/oracles/fonts/golden/corpus remain protected. |

Exact review targets include `io/src/{document.rs,files/smiles.rs,smiles/file.rs,
smiles/normalize.rs,smiles/write.rs,smiles/components.rs}`, `commands/src/{content.rs,
lib.rs,files.rs,bond.rs,controls.rs,file_ui.rs,capture.rs}`, `mcp/src/work.rs` and
`ui/src/{lib.rs,file_workflow.rs,smiles_records.rs}`. Update public IO/command reexports;
CLI discovery stays registry-derived. Preserve/extend the real `io/tests/{smiles_file.rs,
smiles_normalize.rs,smiles_write.rs,smiles_aromatic.rs}`, `commands/tests/{smiles_files.rs,
history.rs,smiles_record_oracles.rs}`, `mcp/tests/{ui_smiles_work.rs,ui_smiles_conversion.rs}`
and CLI `sdf_capture_adapters.rs`/stdio tests, adding new operation tests rather than
weakening their existing contracts. The unchanged `smiles_file_raw_oracle.py`,
`smiles_oracle.py` and qualified geometry consumers remain independent checks. Any required
new chemistry crate/SDK clipboard bridge needs actual layer/notice/lock and native tests;
none is introduced by this study.

Freeze independent expectations and observe RED before implementation:

1. Paste `c1ccccc1`, its Kekulé equivalent and `c1ccccc1.CC` into a nonempty native drawing.
   Independently verify full graph/H/aromatic equivalence and no invented stereo; retain
   every pre-existing object/Arc. Edit the pasted C–C, Undo/Redo, Save/reopen and consume
   the actual file independently. One Undo removes the whole paste; IDs never recycle.
2. Actual native menu/clipboard plus CLI/stdio/authenticated loopback must exercise the
   same command. Test empty/unavailable clipboard, text focus, delayed payload after New,
   worker cancellation/retirement and same-frame stale canvas input. Reject
   `CC PRIVATE`, multiple record lines and unsupported later components without edits.
3. Use an original multi-record file with duplicate records, interleaved `C1.N.C1` atom
   order, CRLF/finalEOF, ignored lines and opaque NUL/nonUTF8 metadata. Selected/all maps
   must be literal and global; changing record2 must leave every other byte exact.
   Independent raw-record and per-notation consumers have separate jobs.
4. File-backed edit→whole Save→reopen must preserve all records/opaque data and the intended
   graph. Test writer-DFS remapping, no-op identity, Undo/Redo, source/current/saved sharing,
   quota boundaries, invalid later records, close/source ABA and a published old Save
   finishing after replacement. No source-only save may clean unencoded layout edits.
5. Assert full logical/physical geometry, CPU/GPU parity and real native viewport behavior
   separately. Do not fix failures with shrinking, dropped H, relaxed ink guards or new
   unapproved goldens. Root's current DPI work is outside this study.

This recommends a useful chemical authoring path first and a real source rewrite/save
path next. It does not complete M1, Copy As/selection/clipboard interoperability, general
SMILES chemistry, or any later milestone. Reviewed source hashes and guide-page renders
remain ignored under `target/smiles-source-import-research/`; no source was copied into
the product and no implementation or consumer test was executed here.
