# Native source-backed SMILES records

2026-11-16. Extends the [retained-record engine](verification-smiles-record-authoring.md),
not the four create-only converters or Drawing paste. The complete M0–M5 target remains
open. This is an original Bondline import presentation; exact Professional26.1 record
selection screens/defaults are unknown.

## Contract and provenance

`file.import_smiles_records` presents **File → Import SMILES records…** through the existing
`ui.file` action `import_smiles_records`. The form invokes only the existing
`document.import_smiles_records`; there are still32 registry commands. Default control
counts become20 Drawing/14 SDF/10 raw SMILES. Import is enabled only for an already-open
raw SMILES file and an available native host. One record (initial ordinal1) or All records
and all five geometry values are explicit. Initial48pt bond length,72/72pt origin and A4
paper are original policies, as are the form, labels and absence of a shortcut.

The [source contract](research/smiles-record-authoring-contract.md) pins OpenSMILES file,
connectivity and H semantics and PhotoCraft's retained-source, historical-result and
snapshot-job distinctions. Existing pinned egui0.29.1 widget/input/AccessKit APIs guide
the adapter. No proprietary implementation or asset is reused. Generated coordinates
remain a derived view; whole-source Save retains every selected/unselected record and
opaque byte. `is_fully_mapped` stays false. Validated projection display, View and order
tools are separate capabilities; generic create/extend/paste/page/cross-format edits
remain refused.

`UiSmilesRecordsContext` captures source identity/revision, close/view/tool epochs, file
association and opaque file-context identity under the origin mutex. Import additionally
requires its exact Accepted presentation. Reserve the existing two-worker capacity before
consuming intent; parse/layout/whole-file edits run off-lock. Check context atomically with
finish and capture a private post-finish proof under that same lock. This grants no Save
lease, file publication or trusted Save/close acknowledgment. Capacity exhaustion leaves
intent queued; consumed failures require explicit retry, never implicit rebase.

Both existing-bond click and numeric1/2/3 paths queue the typed `bond.set_order` for this
backing. Native/CDXML edits keep their existing immediate path. A pending order blocks
new File/Edit/Page/tool/canvas mutations, including Num2→Save in one input packet. View
and close remain available and invalidate/cancel old work. A previously Running Save
may coexist, but cannot clean its newer edit. Returning that same Save to an Accepted
retry form invalidates both unstarted/prepared order context and even a completed no-op
order's presentation proof; request ID/epoch alone cannot detect that phase transition.

Import forms and order work retain actual JoinHandles through `is_finished` and `join`.
A sent result or released executor permit is not thread retirement. Frame-start modal
and order-busy fences prevent old input from editing a newly revealed drawing. Detached
import drafts stay copyable; success after late cancellation/context change is reported
truthfully. Engine poison cancels/drains actual workers. Generic Save continues through
the existing continuously held shared FilePlan reservation and receipt rules.

## Independent RED and GREEN evidence

Root owns all Cargo/native execution. Independent agents declared tests before receiving
product changes; original declarations and invertible setup-only differences remain under
ignored `target/native-smiles-records/`.

- App80778:0/1 intended missing-menu runtime RED. Original1423-byte declaration SHA256
  `75d38544338172f5b30ac0a6f47ed3b20206d6e2ebabfc49eb548ed6f27630da` remains the prefix.
- Worker32859: missing `UiSmilesRecordsContext`/native prepare APIs compiler RED. Later
 31533/2428 are new-test setup failures: the test referenced IO/model without direct MCP
  dependencies. Replace only those setup expressions with the actual existing commands
  and serialized enum comparison; no dependency or behavioral expectation was changed.
- Controls61695:0/3 runtime RED, exposing19/13/9 instead of20/14/10 plus absent capability.
  All three pass after the typed binding and qualified View/tool guards.
- Worker33285:9/2 runtime RED. Initial pending-form error classification and same-request
  Running→Accepted retry were wrong. Shared order validation fixes both;17802 passes11/0.
- The additive no-op result-proof regression reaches11/1 runtime RED; private post-finish
  proof also checks pending presentation phase.99507 then passes all12. An earlier filter
  run selected only two old tests; it is not the new regression's RED evidence.
- App4960:1/4 runtime RED. A real All→All records label correction fixes three early
  failures. A new pan helper inspected the old Fit accessibility tree immediately after
  F5; five setup-only lines wait for both actual sliders using its unchanged10s bound.
  Real SetValue and exact100%/200pt assertions stay.11112 then passes all five.
- The independent real-producer test passes: actual registered completion sends its
  actual result, then remains behind a finite cleanup-owned thread gate. Executor idle
  does not clear workflow busy or authorize close until the real producer retires.
- A sixth App case preserves both input orders: Import release before All activation
  must select only record1; reverse order must select all three.29304 first stops at a
  new-test API setup assertion: pinned egui advertises Focus plus default Click verb,
  not an explicit Default action bit. Only that assertion is corrected.1653 then reaches
  the intended runtime0/1 RED: the later radio incorrectly selects[1,2,3]. Reusing the
  converter's immutable radio/typed event-action replay fixes it;25444 passes all six
  unchanged behavior cases. No old input assertions, deadlines or retries were changed.

The App tests exercise actual frames, controls, off-lock work, whole-source persistence
and unchanged external raw/RDKit consumers. They cover selected/all import, unsupported
All→explicit selected retry, current view/painted bonds, click/key order, two saturated
worker permits, same-packet edit→Save refusal, Unicode Save/reopen/reimport, history and
pending cancel/source/close/view ABA and ordered record selection. They do not establish
native platform interaction.

```sh
cargo test -p bondline-commands --test native_smiles_records --locked
cargo test -p bondline-mcp --test ui_smiles_records --locked
cargo test -p bondline-ui --lib native_smiles --locked
cargo test -p bondline-ui --lib real_sent_order_result_remains_busy --locked
cargo xtask ci
```

Initial aggregate26982 passes1291/0/1 before the sixth App regression/fix,186 result lines.
Its157555-byte log is SHA256
`a39206ae94816cb8a3b1cd608ffcc816b57b332a119692a698c3fae1527065ea`.
Final-source aggregate65944 **fails825/1**,125 result lines, after workspace formatting
and warnings-denied clippy pass. Unchanged MCP test
`shutdown_without_clients_releases_the_listener_address` fails at
`replacement_bind 127.0.0.1:57773`, macOS AddrInUse/Os48. Log98890 bytes, SHA256
`61ca9fb3ef4a56fc1ae399e2f4722551184832030499d46e19ad3470a2d3cafb`.
No failure-time owner capture exists; do not assign a cause or repeat the exhausted20
diagnostic contexts. The old test stays byte-exact SHA256
`fd96b257cd6c00b52e97544d652ed69fdbaaaa454e1f3a9360d77f3292218fb0`.

Separate21784 runs the eight remaining MCP integration binaries, model/render/UI/compat/
xtask tests, all workspace doctests, macOS AccessKit tests/clippy and layers/licenses/
corpus. It exits0: **466/0/1**,67 result lines;60603-byte log SHA256
`06cf9fe61283f29cbc5c86530b21d3cc45d20e5042bd069040d65c815d656614`.
It neither reruns nor skips assertions in the failed control binary. These remaining
passes are not a green aggregate. Runtime closures remain155/166/221 packages plus two
assets; corpus remains eight files/five bundles with its explicit omissions.

No parser/fuzz, dependency/lock, font, SDK, old fixture/oracle, corpus pin or approved
golden changes. Four old count expectations add one control, and one old exact ID set
adds that ID; the registry count stays32. Actions remains disabled. Full M0/M1 and
M2–M5 remain incomplete.

Final protection checks reproduce all four frozen declarations through rustfmt alone;
291 other tracked test/fixture/oracle files remain byte-exact. Only the four count changes,
one ID and two-line App bridge differ in old tests. Parser/model/chem/render/vendor/compat
and dependency sources remain exact to base01e79da. All582 parity IDs/order, the actual
32-command CLI/document table and369 local links pass. Receipt1438 bytes, SHA256
`cf1b431b9fc9af88938ab3eecd9ab40ce76f207018cc965865d07191fe5b41d8`.
The first auxiliary checker pointed at the control declaration in the wrong directory;
only its lookup was corrected before this pass. No product/test change followed.

## Actual macOS workflow and independent files

Native build54599 exits0. The sole old preview is inspected Saved8/7/2, quit normally,
then its existing `target/smiles-ring-native/Bondline.app` is refreshed. No obsolete
preview is reopened or additional bundle created. New executable21605552 bytes, SHA256
`bd40e72eb760cd61979d289139bdd0d474304c2a7fcdc7efd639a607c8403332`.
Bundle ID remains `org.bondline.neutral-simple-ring-review`; no control port is requested.
Inline CUA AX/pixel observations establish:

- Open the original32-byte two-record source. All refuses the isotopic second record,
  keeping Saved raw content and its geometry/selection draft. Explicit One/default1
  retry imports2 atoms/1 bond/1 fragment while preserving the unselected isotope/data.
- F5Actual100% renders. With Double selected, clicking the existing Single commits
  Double/H2. CmdZ restores Saved Single/H3; ShiftCmdZ restores the dirty Double. Native
  Unicode Save As creates the actual41-byte `selected-edited-λ.smiles` and reports Saved.
- Open the separate52-byte three-record source, choose All and import7/3/3. F5 recovers
 100%. A fresh native positional event at the middle bond (right click, no mutation)
  followed by key2 changes that record's edge. CmdZ/ShiftCmdZ restore Saved/dirty states.
  Unicode Save As creates the actual62-byte `all-edited-λ.smiles` and reports Saved.
- Explicit Open returns that saved file to raw3-record/62-byte mode. Explicit All import
  restores7/3/3; default Fit78.79% refuses the existing distinct-pixel multiple-bond guard.
  F5Actual100% restores the complete scene. Existing-Double click still reports unavailable
  alignment without editing. A fresh positional event and key1 make the reimported edge
  Single; CmdZ returns to Saved Double. The sole current preview is left in that state,
  Single tool selected, PID34839.

The unchanged independent raw scanner, per-notation pinned RDKit and actual-Fragment
consumer verify exact literal saved bytes, chemistry/H and no stereo/radical assignment.
Separately qualified CLI reimport supplies actual serialized model data to that consumer;
it is not introspection of the live native Engine. The selected output keeps the entire
unprojected `[13CH4]` record, isotope13 and opaque data exact. The all-record output changes
only middle notation to `[CH2]=[CH2].[NH3]`, preserves first/duplicate records and opaque
NUL/FF metadata, and retains final EOF without LF. Its middle formula is C2H7N; first/last
remain C2H6. Both original inputs remain exact. This52-byte native vector omits the final
LF present in the earlier53-byte IO vector; no old fixture is changed.

| Actual native output | Bytes | SHA256 |
|---|---:|---|
| selected-edited-λ.smiles |41|`b7f9cbea5e312b6514f0ce8fc8151e233a9933547c23af6ee6923471a4955117`|
| all-edited-λ.smiles |62|`3f14371f0786c5fc3d3107f6e91b7bfdeac0f1a31314c9ca4007246182fa517c`|

Ignored native receipts/files are under `target/native-smiles-records/native/`.
`independent-files.json`13115 bytes, SHA256
`74c03d85ecc98a58c95acd2fd7ab1b5110f6162f97ff53f9279e22f0b3e892b2`.
No screenshot becomes an approved golden. Two native pan attempts (AX string SetValue
and visible-handle drag) produce no visible or AX change; cause is unknown. AX focus of
the picker folder does not change focus, and typed CJK in a path is omitted on one attempt;
selecting its exact Unicode file row and ordinary OS paste of the output name work.
Do not infer a file-codec Unicode defect. CJK filename glyph coverage, geometry/ordinal
typing, native lifecycle races, measured DPI/2×, wider chemistry/layout, exact reference
screens and Windows/Linux remain unverified or limited. The SMILES picker hint still
describes raw-only preview; reconcile that wording with explicit record import separately.
