# Engineering reference source review

Reviewed 2026-11-13 from shallow, sparse local Git checkouts under ignored
`.cache/reference-repos/`. The pins below are the checked-out commits, not moving branch
names. No reference application, installer, setup script, test suite or model was run.
This records source observations and recommendations; it is not runtime verification of
these projects or evidence of Bondline's feature parity.

| Reference | Reviewed commit and commit date | Declared project license and actual stack |
| --- | --- | --- |
| PhotoCraft | [`0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e`](https://github.com/storytold/photocraft/tree/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e), 2026-11-13 | MIT OR Apache-2.0; Rust engine, egui/eframe and wgpu; version 0.23.1 in the manifest. |
| VoiceStudio | [`9a0ebd9fbf9d85bce6e58541677ab311f290eb60`](https://github.com/debpalash/voicestudio/tree/9a0ebd9fbf9d85bce6e58541677ab311f290eb60), 2026-11-13 | AGPL-3.0-only; Electron/TypeScript UI and Python backend; version 0.5.8. Conceptual observations only; no application code or assets may be copied or linked into Bondline. |
| REA | [`bbe2bcc979be747698d9ad9ce87af017b92e0f10`](https://github.com/morluto/rea/tree/bbe2bcc979be747698d9ad9ce87af017b92e0f10), 2026-11-13 | MIT; TypeScript/Node analysis tooling; package version 6.9.0. An external development tool, not a Rust runtime dependency. |

The license declarations are in the pinned [PhotoCraft manifest](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/Cargo.toml),
[VoiceStudio manifest](https://github.com/debpalash/voicestudio/blob/9a0ebd9fbf9d85bce6e58541677ab311f290eb60/package.json)
and [license notice](https://github.com/debpalash/voicestudio/blob/9a0ebd9fbf9d85bce6e58541677ab311f290eb60/LICENSE-NOTICE.md),
and [REA manifest](https://github.com/morluto/rea/blob/bbe2bcc979be747698d9ad9ce87af017b92e0f10/package.json).
These root declarations do not clear every dependency, font, model or fixture.

## PhotoCraft: closest engine and native application reference

| Problem | Observed implementation | Bondline decision |
| --- | --- | --- |
| One command implementation across interfaces | [`CommandSpec` and `cmd!`](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/crates/engine/src/cmd.rs) combine IDs, labels, menu paths, shortcuts, enabled predicates and handlers. [`Headless`](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/crates/automation/src/headless.rs) lists engine specifications and dispatches engine operations; the CLI calls that adapter. The parameter field is a descriptive JSON-like string. | Keep Bondline's typed `command_registry!` and generated closed schemas. Extend that metadata with menu placement, shortcuts and contextual disabled reasons as commands land. Do not substitute descriptive strings for validation. |
| Complete inventory without pretending every item works | The [`menu catalog`](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/crates/ui-egui/src/menu_catalog.rs) records ordered menu paths, separators, labels, default shortcuts and command IDs; its contract distinguishes implemented entries from disabled entries. | Give every documented Bondline control a stable inventory ID and source. Link it to a registered command and decisive evidence before marking it verified. A familiar disabled menu alone does not deliver its workflow. |
| Editable model plus opaque file payloads | The [`document model`](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/crates/doc/src/lib.rs) retains `psd_raw`, layer blocks and global/resource blocks in shared byte buffers. Its `duplicate_clears_psd_id_but_keeps_blocks` test checks identity changes without dropping opaque payloads. The current standalone PSD parser is an external pinned `craft-psd` dependency. | Bondline's `StoredDocument` already keeps source, model and baseline together. Keep that complete object in history/import paths. Next test edited exports against an independent parser and unknown attributes/elements; preserving original bytes alone does not establish safe edited export. |
| Undo memory and coherent gestures | [`History`](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/crates/ops/src/lib.rs) stores `Arc<Document>` snapshots, compound-operation context, a state cap and an approximate shared-payload byte budget. [`Gesture ownership tests`](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/crates/ui-egui/src/pen_gesture_owner_tests.rs) cover document/revision/tool changes and one-step undo for a valid gesture. | Bondline now has independently tested source/model/baseline/fragment accounting and oldest-first eviction at its own 128 MiB threshold. Import retention tests remain. Bind each future gesture preview to its document, revision and tool; cancel on target changes. Keep one committed drag as one command/history step. Maintain Bondline's own ordered-event tests and native evidence; upstream unit tests do not substitute for either. |
| Responsive work and honest cancellation | [`Jobs`](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/crates/engine/src/jobs.rs) run on snapshots; workers do not mutate the live document. Results apply as one command. A save crosses an explicit commit boundary after which cancellation is refused. Source includes cooperative work slices for the browser and disables conflicting edits during document jobs. | Reuse the prepare/work/apply pattern when cleanup/import jobs justify it. Track an input revision; reject stale results or explicitly prevent conflicting edits. Preserve Bondline's documented capture semantics. Do not announce cancellation after a save has replaced its destination. |
| Atomic native saves | [`AtomicFs`](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/crates/format/src/atomic.rs) abstracts same-directory temporary writes, flushes, replacement and directory synchronization. [`Tests`](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/crates/format/src/atomic/tests.rs) inject interrupted streaming writes and publication failures. | The first Bondline save command should validate serialized data before publication, leave the old file intact on preparation failure, and report post-publication durability failures accurately. Test failures at each boundary rather than only successful save/reopen. |
| Rendering and independent verification | [`GPU parity tests`](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/crates/gpu/tests/it/parity.rs) compare complete pixel arrays with a CPU compositor; the current `TOL` constant is `1/255 + 4*f32::EPSILON`. [`Bounded readback tests`](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/crates/gpu/tests/it/bounded_readback.rs) exercise constrained GPU readback. A [`CLI export recipe test`](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/apps/photocraft-cli/tests/it/psd_write_oracle.rs) explicitly reports the external application check as `not_run`. | Keep CPU/GPU comparison, independent image decoding and human-approved scene-specific goldens separate. Add bounded readback cases as capture sizes grow. Never count a skipped GPU adapter or an unrun external consumer as a pass. |
| Enforced boundaries | [`xtask layers`](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/xtask/src/layers.rs) checks workspace dependencies, standalone format crates and GUI dependency reachability. | Bondline already has a layer gate. Preserve parser independence and keep UI-only dependencies out of the engine as the crate graph grows. |

PhotoCraft pins its entire egui family to fork commit
`6d518fc799681571963842f95b55a6b68431b15a` and winit to
`a517b3846ee8457d84ee4975110bfda6797c44dc` in the reviewed manifest. This is evidence for
using narrow, reviewable, reproducible forks when upstream packaging or platform behavior
blocks a native application. It is **not** a drop-in license solution: PhotoCraft's
[attribution inventory](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/ATTRIBUTION.md)
includes SIL OFL fonts and ISC icons outside Bondline's shipped allowlist. Bondline already
uses its own reviewed eframe and compatibility patches; retain its target-specific license
gate and notices rather than importing PhotoCraft's graph or assets.

Its [architecture document](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/docs/architecture.md)
also separates a current-state summary from an older design plan. This review follows the
actual source paths above; it does not adopt advertised command counts or a blanket
file-format parity claim.

## VoiceStudio: evidence, recovery and portable-project lessons

VoiceStudio is not the requested Rust desktop/GPU stack. Its AGPL application implementation
is excluded from Bondline reuse. The following observations inform independently written
Rust behavior only.

- Its [license review workflow](https://github.com/debpalash/voicestudio/blob/9a0ebd9fbf9d85bce6e58541677ab311f290eb60/docs/licensing/model-review-workflow.md)
  separates a declared license from review readiness and binds a prepared review to exact
  revisions, file hashes, document hashes and notices. The corresponding
  [service](https://github.com/debpalash/voicestudio/blob/9a0ebd9fbf9d85bce6e58541677ab311f290eb60/backend/services/model_licenses.py)
  revalidates plans and file digests. For Bondline, apply that evidence discipline to
  dependency/asset release review; it does not require a model-license dialog or account
  flow in the product.
- Its [task manager](https://github.com/debpalash/voicestudio/blob/9a0ebd9fbf9d85bce6e58541677ab311f290eb60/backend/core/tasks.py)
  combines in-memory dispatch with persisted job events and reconnection sequence numbers.
  Persistence exceptions are explicitly nonfatal, so source inspection does not establish
  that every event survives a crash. When Bondline gains long operations, use stable job
  IDs, explicit terminal outcomes and recovery tests; avoid introducing a service/backend
  solely to imitate this design.
- Its [versioned project bundle](https://github.com/debpalash/voicestudio/blob/9a0ebd9fbf9d85bce6e58541677ab311f290eb60/backend/services/project_bundle.py)
  uses a manifest and SHA-256 asset list with total/member/count/depth limits and portable
  path validation. [Bundle tests](https://github.com/debpalash/voicestudio/blob/9a0ebd9fbf9d85bce6e58541677ab311f290eb60/tests/test_project_bundle.py)
  exist but were not run here. Those validation concepts are useful for future Bondline
  packaged projects. Its intentional removal of sensitive or machine-local fields is
  specific to that format: do not apply such filtering to unknown CDXML/CDX content.

## REA: reproducible observations and bounded claims

- Its [evidence schema](https://github.com/morluto/rea/blob/bbe2bcc979be747698d9ad9ce87af017b92e0f10/src/domain/evidence.ts)
  records subject identity, provider/version, operation, parameters, raw/normalized results,
  observed/derived/inferred confidence, authority, coverage and limitations. Authenticated
  records recompute their semantic identity. Bondline's screen/behavior inventory should
  similarly identify the exact reference version or media, observation scope and unknowns;
  keep inference visibly distinct from observed behavior.
- Its [artifact tool registration](https://github.com/morluto/rea/blob/bbe2bcc979be747698d9ad9ce87af017b92e0f10/src/server/registerArtifactTools.ts)
  passes shared contracts through a [generic registration path](https://github.com/morluto/rea/blob/bbe2bcc979be747698d9ad9ce87af017b92e0f10/src/server/registerEvidenceTools.ts).
  Bondline already shares registry/schema definitions across adapters. Continue deriving
  discovery and validation from those contracts; do not maintain a second MCP command list.
- Its [release-exposure contract](https://github.com/morluto/rea/blob/bbe2bcc979be747698d9ad9ce87af017b92e0f10/docs/release-exposure.md)
  distinguishes completed analysis from a security verdict, and `unknown` from an absent
  finding. Preserve that distinction in release records. Static inspection of a Bondline
  artifact can support packaging evidence; it does not verify drawing, chemistry, file
  fidelity, signing or cross-platform launch.

## Next implementation decisions supported by this review

1. Source/history byte accounting is now delivered headlessly. Finish import-failure
   retention tests and editable codecs before wiring imports into the desktop UI.
2. Add atomic save through the shared command registry, with failure injection, explicit
   publication status and independent edited-file checks.
3. Grow the sourced UI inventory into stable control IDs linked to registry entries,
   shortcuts, disabled reasons, complete interaction flows and native evidence.
4. Extend drawing gestures with owner/revision cancellation and one-step undo tests before
   adding attachment, selection and more tools.
5. Add cancellable revision-bound jobs only when an actual expensive operation lands.

All three clones succeeded. The sparse review omitted large assets/model weights, did not
fetch/build their external Git dependencies, and did not audit their full transitive
license graphs. No runtime/performance/feature-completeness claim for these repositories
was verified. No source or assets were imported into Bondline by this review.

## Shared file controls: selected implementation slice

Rechecked the same PhotoCraft pin on2026-11-13. Its CommandSpec has the metadata needed
for discovery; its control dispatcher distinguishes a menu request from direct engine I/O.
Bondline will co-locate delivered control bindings with its existing typed registry and add
finite transient file-presentation requests. A native host must explicitly acknowledge a
request; the headless session must refuse it rather than claim a chooser appeared. Actual
Open/Save remain the existing bounded file commands.

The request must bind its authored revision, path, relevant lifecycle state and a monotonic
identity. Native selection validation, immutable I/O preparation and worker registration must
happen under the same engine lock; validating first and submitting later would permit stale
discard or overwrite. Close-specific Save must retain the existing close epoch and receipt
rules. These are original Bondline safety decisions, not inferred reference internals.
This design record precedes implementation and execution; see the protocol and verification
records for delivered evidence. No generic UI dispatcher or job framework is required.

The finite shared-file-control slice is now implemented and verified within its documented
scope: typed core bindings drive File/Edit/Page, ui.file addresses native requests and
cancellation, and atomic preparation guards actual I/O. See control-protocol.md,
native-file-ui.md and verification.md. This does not complete reference inventory, every
dialog field or running-job control; the source observations above remain distinct from
Bondline's actual execution evidence.
