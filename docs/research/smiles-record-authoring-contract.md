# Complete-source SMILES record authoring

2026-11-16. Contract/design and independent declarations, not implementation or native
verification. This is Gate B of the [source-import plan](smiles-source-import-plan.md):
selected **and all-record** Drawing import, live source mapping, supported chemical edit,
and whole-source Save. Gate A paste and the four converters are separate operations.
Full M0–M5 and broader chemistry/file parity remain required.

## Evidence versus original policy

[OpenSMILES1.0, SMILES Files](https://github.com/opensmiles/OpenSMILES/blob/9240a2887158699503605bc838e1de9a9f7883f4/opensmiles.asciidoc#L1372-L1383)
specifies zero or more line records, optional SP/TAB-separated application data, LF/CRLF,
and ignored blank/leading-whitespace lines. Data interpretation belongs to the application;
it is not a chemical-property map. The Disconnected Structures section explains that a
dot does not necessarily disconnect the final graph (`C1.C1`). Preserve actual connectivity.
Bond/H/isotope and aromatic source semantics remain those in the
[aromatic contract](aromatic-authoring-contract.md).

The existing file codec deliberately retains opaque byte-oriented data and ignored lines,
has explicit limits, and accepts final EOF without LF as a qualified extension. It validates
every meaningful notation with the current parser. Unselected records may be chemically
unprojectable only within that accepted syntax; stereo syntax refused by the parser still
prevents complete file Open. Do not silently broaden or weaken this codec for selection.

Pinned PhotoCraft separates [retained metadata](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/crates/doc/src/lib.rs#L875-L888),
[historical conversion reports](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/crates/doc/src/import_conversion.rs#L1-L104),
and [snapshot jobs](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/crates/engine/src/jobs.rs#L1-L27).
Reports are not live editing/resource instructions; source ordinals belong to a snapshot.
Its jobs apply results through the command/history path and distinguish cancellation from
completed external effects. Adopt these distinctions, not its code, report framework or
limits. The following selection, mapping, layout and persistence policies are original
Bondline decisions. Public GUIDE25 paste evidence does not prove a26.1 selected/all-record
`.smi` Drawing-import or record-authoring flow.

## Locked IO ownership and mapping

The first useful APIs are:

```rust,ignore
pub enum RecordSelection { Record(file::RecordId), All }
StoredDocument::from_smiles_records(
    source: SmilesFileDocument, selection: RecordSelection,
    length: f64, origin: Point2, page: PageSize, cancel: &Cancellation,
) -> Result<StoredDocument, DocumentError>;
stored.smiles_file() -> Result<&SmilesFileDocument, DocumentError>;
stored.smiles_records() -> Option<&[SmilesRecordMapping]>;
stored.has_smiles_projection() -> bool;
stored.with_smiles_fragment(page_id: PageId, replacement: Fragment, cancel: &Cancellation)
    -> Result<StoredDocument, DocumentError>;
```

Keep complete immutable original/current `SmilesFileDocument` owners, a validated **current**
mapped-model baseline and shared live maps inside Drawing backing. Original bytes never
become a regenerated Fragment. `is_fully_mapped()` stays false: opaque/unselected source
and derived geometry are not globally native-encoded. `source.format="smiles"`, byte length
describes original bytes, and `source.modified` compares original/current complete files.
An additional original model is optional; do not confuse its role with current validity.

Each selected record becomes one Fragment, even if it contains disconnected components.
Record order and source atom/bond-array order choose checked new global IDs. Maps expose
record_id()/fragment_id(); atoms carry atom_id, zero-based notation_atom and absolute range;
bonds carry bond_id, zero-based notation_bond, actual source endpoint IDs, optional bond
range and complete optional RingClosure. All ring open/close and bond-token spans are
absolute current-file bytes. Map arrays follow **current IR/token order**, independently
of stable model array order and authored bond begin/end direction.

Selected unsupported chemistry/layout refuses the entire import; unselected bytes remain
exact. Missing positive record is `not_found`; empty valid file+All is `unsupported_content`.
Pre-cancel is `cancelled`, aggregate64atoms/128bonds are `resource_limit`, invalid geometry
parameters use existing `invalid_params`. Do not project a first/empty Drawing on failure.

## Geometry and command transition

Use one aggregate normalized model graph for all selected atoms, with unique local IDs,
one existing components-layout budget and one whole-page certificate. Then partition its
positions back into record Fragments, retaining every input atom/bond/H/direction. A model
view is valid here; synthesizing a partial `SmilesDocument` IR over whole source is not.
Keep exact L, upper-left all-center origin, current shelves/gaps, no rotation/rescale to
conceal failure. Final labels, physical H, stroke, cross-record collisions and paper checks
are mandatory. Do not reset search/certificate limits per record or component.

Propose one shared closed `document.import_smiles_records`: expected_revision, explicit
selection `{mode:"record",record_id}` or `{mode:"all"}`, bond_length_pt, origin_x_pt,
origin_y_pt, page_width_pt and page_height_pt. It captures the already-open complete
SmilesFile, not a path/temporary file or clipboard prefix. Initially accept raw file sessions;
changing an existing backed Drawing's selection needs its own guarded unsaved transition.

Import is a guarded clean **session transition**: revision/generation/view epoch advance,
current=saved=new backed Drawing, history/ID namespace reset as for Open, source path/SMILES
association retained, no bytes written and no Save ACK. Otherwise a projection-only variant
change would falsely dirty unchanged source. Preserve complete input until off-lock work
finishes; refuse stale owner/revision/generation/close/file context. Native selection and
all-record controls must call this same registry command, with actual desktop evidence later.

## One supported chemical transaction and exact source Save

Initial `with_smiles_fragment` permits only Bond.order and Atom.explicit_hydrogens changes.
Fragment identity, atom/bond array order, IDs, elements, isotope, charge, radical, positions
and directed endpoints stay exact. Writer finite-valence rules still apply. Compare the
current model to its validated mapped baseline before any operation: detached `model_mut`
drift cannot be legitimized by a later replacement, rendered, or saved as stale source.
A genuine no-op shares model/source/index/maps and preserves original notation spelling.

Extend the existing writer additively to return notation plus **emitted AtomId order**.
Its old byte-only API/output remains exact. After parsing generated notation, map each
atom ordinal through that emitted list; derive bond IDs by unique mapped endpoint pairs
and record actual source endpoint direction. Preserve model direction separately. Never
assume DFS emission follows sorted IDs, nor recover identities by molecular isomorphism:
symmetric structures make that ambiguous. Ring bonds appear on closing tokens, not merely
writer traversal edges. Current token ranges must be validated against actual output.

Splice only the target record's notation span using the immutable whole-file helper;
replacement is a nonempty complete ASCII notation token without LF/CRLF/names. Reparse the
whole candidate under unchanged limits. Preserve separator, opaque data, record ending,
ignored lines, duplicate records and every unselected byte. Later absolute offsets shift;
refresh selected maps against the new owner. Local token spans plus current RecordInfo can
avoid changing chemical identities when a preceding record grows. The file-splice feature
has separate independent ownership/tests; this note does not implement it.

SMILES-backed `bond.set_order` must create one off-lock immutable candidate containing
model+current whole file+maps/baseline, then commit once with existing source/revision/
generation/close guards. Keep the old native/CDXML path exact. The UI currently calls
Engine.execute under the frame mutex for hotspot edits: merely returning a deferred plan
would still run source reparsing there. Route this backing through one narrow registered
native work path using existing workers/reply/retirement barriers, not a second edit engine.

Auto Save remains SMILES and borrows the complete validated current file through existing
FilePlan/Completed and its continuously held one-save reservation. No per-Fragment export
wrapper, added LF, reconstruction of application data, or fake native ACK. A captured Save
retains that whole snapshot across later edits/Open; publication stays truthful and cannot
clean/reassociate a newer generation. Save clears file.dirty, not source.modified. Undo/Redo
restore complete model/file/maps exactly. Returning chemistry to its original graph by a
new edit need not restore original aromatic spelling; Undo does restore original bytes.

Reject geometry/page/partition changes and unsupported graph mutations before source Save;
SMILES cannot encode authored2D coordinates or paper/style. Initial generic create/extend/
paste/page edits stay refused for this backing. Add a source-qualified display capability
and this operation-specific edit capability, not a global completeness override. CDXML/MOL/
CDX Save must refuse source/context loss; any future derived export needs explicit scope.

Count current/saved/history original/current file buffers and indices, model/baselines,
mapping/range containers and reserved capacities with allocation deduplication. Include outer
inline storage once; do not double-charge nested wrappers. Preserve128entries/128MiB policy,
current oversized retention, complete-byte equality and metadata-only Debug/state.

## Decisive independent expectations

The frozen IO declarations use a53-byte original: three records `CC`, `C1.N.C1`, duplicate
`CC`, with CRLF/LF and opaque NUL/FF metadata. ALL yields IDs1–7; middle source tokens map
to3,4,5 and closure3→5. A valid Double/H edit emits `[CH2]=[CH2].[NH3]`, token IDs3,5,4 and
a63-byte exact literal splice; later CC ranges35..37 become45..47. Original model order,
positions, source data and unrelated Fragment Arcs remain exact. This is not a round trip
through only our reader/writer: unchanged raw scanner and external Fragment chemistry
consumer inspect the actual candidate. Further cases cover no-op sharing/accounting,
selected retention versus later-All refusal, detached geometry, identity/direction/isotope
refusals,64/65 atom and129bond bounds, cancellation and error codes.

Command/adapters still need selected/all import, one bond edit, actual whole-source Save,
reopen/reimport, source/saved/history distinctions, stale/reverse/cancel/busy, captured Save
and trusted-close receipt tests. Actual native record selection/order edit/Unicode Save
must follow; unit/App/consumer evidence is separate. Chemistry, stereo/charge, fused/large
layout and reference record-browser behavior remain qualified outstanding work.

## Exact primary evidence

Live primary raw fetches on2026-11-16 matched the previously pinned caches byte for byte.
No reference code/assets/dependencies were adopted and no builds/native actions were run.

| Source | Bytes / SHA256 |
|---|---|
| OpenSMILES commit9240a288…, file grammar1372–1383, disconnected and H sections | 86816 / `00f0423da612de913770133aa13e456d0cebc562dd50166534a8a7f9a834bd21` |
| PhotoCraft commit0fb40c53…, doc metadata875–888 | 65401 / `11d5769d2bf41adc0b2e410da5b04aba3eb7429e201453b4c6dc38296e4d5363` |
| PhotoCraft same pin, import_conversion1–104 | 18380 / `8b9d6b52ce836d36d16ce252384b05238dd0db16f6952fd9173075ee3d19f82e` |
| PhotoCraft same pin, jobs1–27/521–537 | 107608 / `03242c486f21e643c5cec7fff8e379ff4970c2b5cc6dd6255221330aabdd1594` |

Exact reviewed Bondline files, captured source and URLs are under
`target/smiles-record-authoring/research/source-evidence.json`. Independent declarations
and preimplementation freezes are under its `test-declarations/` sibling. The initial
constructor missing-API RED is Root execution evidence, not execution by this researcher.
