# Complete-source SMILES record authoring

2026-11-16. The [contract](research/smiles-record-authoring-contract.md) separates
notation-span replacement, retained drawing projection and command/native integration.
Public OpenSMILES file semantics and pinned PhotoCraft ownership/job distinctions are
sources; selection, geometry, identity and lifecycle policies are original Bondline work.
Local feature commits: codec `78f7d8d`, retained projection `162f79d`, shared commands
`58666e4`. All are part of the verified integrated source described below.

## Notation-only file transaction

`SmilesFileDocument::with_record_notation` replaces exactly one existing nonempty token.
It preserves every other byte, including duplicate records, opaque NUL/non-UTF8 data,
ignored lines, separators and LF/CRLF/final EOF. The complete candidate is reparsed under
the existing limits before return. An exact no-op shares source and index. Cancellation
wins at entry; missing records, invalid notation and resource limits keep typed errors
and absolute source context. Standalone notation's optional terminal EOL is deliberately
refused here so a replacement cannot insert a record or absorb application data.

The first independent 1187-byte declaration was frozen as SHA256
`2f31b5a2f153fb8aed8e70f0295a825dadea5dab013d9f70aac1d0980b2c21cc`.
The initial combined IO run exits 101 with the intended missing setter, selection and
constructor API errors only; see `target/smiles-record-authoring/io-initial-red.log`.
The six-case expansion is 14276 bytes, SHA256
`43bf783a8206398d93b4eab91829da465317da0d5ceaea14e0810c4dbcfdba4b`.
First receiving 4425 exits 0: six tests pass, including literal spans and unchanged raw
scanner/RDKit consumers over actual modified bytes. An in-progress mapped-writer field
warns as unused; this isolated run is not a warnings-denied or full-workspace pass.

One test-helper portability correction uses the existing Python override/default
convention; it changes no input, assertion, script or pinned-version check. Its 14336-byte
declaration is SHA256 `6a9bdfa78ca04f11c95b9a699b4033bc5710f081d281adf5559bbe1856061eea`.
Receiving 80221 passes the same six cases plus seven projection cases without warnings.
Final receiving rustfmt is mechanical; the original declarations remain archived under
ignored `target/smiles-record-authoring/` with the source and helper-diff receipts.

```sh
cargo test -p bondline-io --test smiles_file_edit --locked
```

Boundary cases cover 65536/65537 token bytes, a complete 64 MiB file whose replacement
would exceed the file cap, record ordinal 65536, invalid or missing targets, cancellation
before no-op and exact source/index identity. Existing parsers, limits and independent
consumers remain unchanged. This file transaction alone grants no Drawing, model-edit,
native selection or Save acknowledgment authority.

## Retained drawing projection

`StoredDocument::from_smiles_records` accepts an explicit record or all records and owns
the complete original/current file, original/current model baselines and source maps.
One aggregate chemistry pass and component layout precede splitting back into one Fragment
per record. The shared 64-atom/128-bond and geometry/ink budgets apply to the whole selection.
Unselected accepted records may be unprojectable; selected failures refuse atomically.

The first 1313-byte constructor declaration, SHA256
`d37cb1e1da5b811401917665dbe77c1534d272811938991a7fae8acad83531a7`, reaches the intended
missing selection/constructor errors before implementation. The seven-case expansion is
14626 bytes, SHA256 `d31f58b4ee9e5b8416b3c4cea37d6013549f549d790e56d5d5a1bec0f7101900`.
First receiving 80221 passes all seven unchanged plus the six codec cases without warnings.

The literal 53-byte three-record case changes only its middle `C1.N.C1` to
`[CH2]=[CH2].[NH3]`, producing 63 bytes. Model atom order remains C3/N4/C5 and the bond
remains directed 3→5, while emitted token order is 3/5/4. Absolute later ranges move by
ten bytes; original metadata, geometry, identities and unrelated Fragment Arcs survive.
The unchanged raw scanner and external Fragment consumer verify actual output. Further
cases cover exact no-op sharing, all/selected/empty/missing cases, 64/65 atoms and 129 bonds,
entry cancellation, detached geometry and non-H/order replacement refusals.

`is_fully_mapped` stays false. Display and complete-source Save have a separate validated
current-projection capability; detached `model_mut` changes cannot acquire it. Only H and
integer order changes can synchronize a record. Cross-format output and unencoded geometry
remain refused. The old byte-only writer output and normalizer profiles remain exact.
One accountant deduplicates original/current files, maps, models and raw source owners.

```sh
cargo test -p bondline-io --test smiles_record_projection --locked
```

The seven IO product pins remain exact from the implementation handoff through integrated
receiving. Five independent test declarations reproduce their committed form using only
rustfmt; no expected values or assertions changed. This library slice does not deliver
native record selection, drawing navigation or editing controls.

## Shared commands, history and actual transports

`document.import_smiles_records` is the 32nd command. Its seven required fields are
expected_revision, selection, bond_length_pt, origin_x_pt, origin_y_pt, page_width_pt and
page_height_pt. It captures the already-open raw SMILES owner without rereading a path.
Success starts a clean Drawing namespace: revision/generation/view epoch advance, history
clears, current equals saved, and the SMILES association stays. No file is published and
no Save acknowledgment is granted. The compact whole-file summary adds projected_records
only for the backed Drawing; the old raw-file summary remains exactly three fields.

Mapped SMILES `bond.set_order` work is deferred through the same worker budget and registry.
One immutable model/file/map candidate commits one history step. Original, current and
last-saved state remain distinct. A synchronized whole-file Save uses the existing lease
and publication path. Raw model drift, unencoded page/layout changes and cross-format
loss refuse; native/CDXML editing behavior stays separate. Source/owner/revision/generation,
close epoch, cancellation and opaque file-context identity prevent stale application.

The first 1513-byte command declaration is SHA256
`3bf6f8a46ac2e37e22ac1dd0e3f26b935b76b394ff1b8562f8336e5e38b2c8e6`.
Run 32044 reaches runtime UnknownCommand, 0/1; initial 87177 passes 1/0. The frozen eight
cases are 28260 bytes, SHA256
`353f13413e81e3555becac1512c6411314b2987eac1d126671e1b0ae812a5601`.
They pass on 23679, including 16 fixed-seed generated whole-snapshot Undo/Redo cases,
repeat edits across DFS remapping, raw/RDKit saved-file checks, cancellation/foreign/ABA
refusals and truthful old Save publication without reassociation after New.

The first actual CLI declaration is 5133 bytes, SHA256
`faf0473c5e2f4e651358fc618d5fec61a0f90625a6421c9c4b15ccdfa16ac1db`.
Run 71774 reaches actual executable unknown_command, 0/1. The three frozen transport cases
are 15639 bytes, SHA256
`f08f2a219adfd7f86054fb0a747b4b81a4335fc0356394f716709a544f7e9025`.
First 7866 passes CLI/socket but fails stdio, 2/1: an All selector with record_id is accepted
by serde's tagged unit variant and reaches wrong-kind unsupported_content instead of the
required invalid_params. Changing only All to an empty struct variant enforces the closed
selector with the same wire spelling. Unchanged receiving 81285 passes 3/0.

The single independent render declaration is 5419 bytes, SHA256
`00b136812eff742da8e4865fd7e2ebcdac050a5e759263856679d9ef6db70773`.
Run 23679 passes it alongside the eight command cases. Actual 1024-square CPU/GPU PNGs
match before and after the middle-record edit; a real adapter is reported. Pinned Pillow
checks valid RGBA PNGs, ink and changed decoded pixels. Captures preserve complete state,
source, pointers, path, history and original file/mtime. This is headless receiving,
not a new RED, native-window observation or human-approved golden.

```sh
cargo test -p bondline-commands --test smiles_record_edit --test smiles_record_render --locked
cargo test -p bondline-cli --test sdf_capture_adapters smiles_record_authoring_wire:: --locked
cargo xtask ci
```

Old catalogue receiving 46079 passes 52/1; aggregate 63504 stops at 64/1 and 7908 at
1164/1/1. These failures are omitted new-name or 31-versus-32 expectations, including an
assertion before the old conversion workflow begins. Eight exact name lists add only the
new command; eighteen old count assertions advance by one. Every old name, behavioral
assertion, deadline, retry policy and socket check remains. Aggregate 37337 then passes
1270/0/1, 184 result lines; its log is 154263 bytes, SHA256
`b730601524007adc074396249310a9618c1367409be3757df69c9e57ee40f1d1`.
Registry descriptions are subsequently clarified to include the synchronized record path;
the final source gate is recorded below.

Native record selection, viewport/navigation and source-bound order inputs remain blocked.
The existing native hotspot path executes while holding the frame mutex, so enabling it
without a registered worker would reparse potentially 64 MiB on the UI thread. Implement
that worker and actual desktop acceptance next; do not flip the global completeness flag.
Reopening a saved SMILES file is raw-file Open followed by explicit reimport into a fresh
namespace. No saved 2D geometry or cross-session atom-ID preservation is claimed.

## Final integrated checks

After the registry-description clarification, final source gate 84163 exits 0:
**1270 passed / 0 failed / 1 ignored**, 184 result lines. Rustfmt, warnings-denied
workspace/vendor clippy, workspace/vendor tests, layers, licenses and corpus pass.
`target/smiles-record-authoring/full-ci-final-source.log` is 154932 bytes, SHA256
`6f5970731317d353cf13b4bbb479feffbeddc8161c6f561b4f6db8f2861ab71e`.
Rust 1.94.1, incremental 0/jobs 2/dev+test debug 0 and the literal pinned Python were used.
Runtime closures remain 155/166/221 packages plus two assets; corpus remains eight files
from five bundles with broader chemistry/CDX omissions. No dependency/lock, SDK, font,
old fixture/oracle, corpus or golden change and no new fuzz run. The old MCP AddrInUse
failure remains unexplained; these unchanged passing checks establish no causal fix.

The integrated receipt is 5537 bytes, SHA256
`aa123d24d0ee1ec58a6d5a0147b80ca0c79db290c50d8c5598db1ed1e5ee7100`, under ignored
`target/smiles-record-authoring/integrated-receiving.json`. All jobs are terminal.
Final protection checks preserve all 582 unique parity IDs/order, match the 32-row contract
to the actual CLI catalogue and resolve 409 local documentation links. Twenty-three old
test files and the registry unit tests are byte-exact after reversing only the allowed
new-name/count/bridge edits. The 6984-byte source/protection receipt is SHA256
`bd2f3b32c9bbf9f14f592b21f0eca4d7faf39eafb99bff42db1e182a1520e9bb`, under ignored
`target/smiles-record-authoring/final-protection.json`.
No native bundle was refreshed or launched; the existing preview remains a separate
earlier checkpoint. New Windows/Linux execution, native record workflows, wider chemistry,
reference behavior and human-approved scene coverage remain unverified/incomplete.
Actions stays disabled. No push or publication occurred; M0–M5 remains active.
