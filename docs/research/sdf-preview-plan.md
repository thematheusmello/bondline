# Next M1 slice: faithful all-record SDF display

2026-11-14. Original implementation plan. Data, resolved labels and restricted headless
capture are now delivered; see the linked verification records. Native UI/cache and actual
workflow acceptance remain pending. The current source-file/record-query foundations do
not complete chemical display. Full target remains every supported record with retained
source, useful geometry, correct labels and honest unsupported-data diagnostics.

The read-only source report under target/sdf-preview-next-slice.md is23085B/SHA256
fb57b3ac8d60e30f39ba07d9db8706bb6de55226bc808bc7d197d095e1ee0909. Its33 source/spec/fixture
identities were stable during review. It contains detailed file paths, source hashes and
unexecuted acceptance suggestions; this committed decision record preserves the essential
continuation context if disposable target files disappear.

## Required chemistry and label dependency

[BIOVIA Chemistry2026 CTfile Formats](https://www.3ds.com/assets/invest/2026-01/biovia_chemistry_2026_ctfile_formats.pdf)
defines unmarked versus declared valence and query HCOUNT; it supplies no complete normal-
valence perception algorithm. The strict current MOL mapping leaves unspecified attached
H as Model None. [IUPAC GR2.1.1](https://iupac.qmul.ac.uk/drawing/drawing.html#GR211)
means that merely omitting H from an O label cannot establish a faithful OH depiction.
Do not mistake known-H ethane fixtures for ordinary VAL-absent SDF support.

First implement a separate immutable display-H sidecar, retaining provenance and the
original Model None/source bytes. A documented initial derived profile can use neutral
C4/N3/O2/monovalent F/Cl/Br/I and checked bond-order sums, with a separate physical-H rule.
The [pinned OpenSMILES organic-subset rules](https://github.com/opensmiles/OpenSMILES/blob/9240a2887158699503605bc838e1de9a9f7883f4/opensmiles.asciidoc)
and [RDKit primary valence documentation](https://www.rdkit.org/docs/RDKit_Book.html#valence-calculation-and-allowed-valences)
are evidence for that restricted original policy, not a normative universal MOL algorithm.
Broader P/S, alternative valence, charge, radical, isotope, aromaticity and stereo behavior
needs explicit sources/tests; refuse unresolved derivation rather than clamp or infer it.
Keep existing authored Some(count), including Some(0), exact. Do not silently rectify it
or narrow already-supported literal depiction to only the derived profile. Decide and test
unusual declared-valence handling explicitly before using the sidecar for rendering.

Add a renderer entry consuming resolved display facts with matching atom IDs/provenance;
the renderer must not choose chemical valence. Preserve the literal-only entry, old tests,
font/oracles and human-approved single-bond golden. No derived Page/Fragment may authorize
an encoded Save, be assigned as authored content or advance Engine identity floors.

Source review also found a label-direction gap: horizontal neighbour voting can choose
H-left on a tie, whole-label masking then lets an opposed bond approach H instead of N,
and incident bonds are exempted from the label-overlap test. This is source-derived risk,
not a newly observed native failure. [IUPAC GR2.1.5–2.1.7](https://iupac.qmul.ac.uk/drawing/drawing.html#GR215)
requires faithful attachment/orientation. Test opposed C—N—C, mirrored/vertical OH/NH2 and
source directions. Do not rotate a molecule, drop H/ink or pick convenient coordinates to
hide the gap. Full display requires proper placement; a typed refusal is an interim limit.

## One connected source-to-scene workflow

Use existing SdfDocument.as_bytes/indexed mol_range and actual mol_version; there is no
SdfDocument.read_record API. Borrow each MOL and reuse its strict parser/mapper, retaining
all SDF records/data. A proposed io::sdf::preview producer may return a bounded ephemeral
Page, record/local-ID correspondence and display-H sidecar. No empty crate or broad job
framework is needed. Complete Sdf{original,current} remains the authored/saved content,
with document:null and unchanged wrong-kind/edit/export authority.

Remap local IDs with checked cumulative offsets; retain record/local correspondence.
Use only an explicit shared MOL frame plus one display translation per record, preserving
relative geometry/directions. Packing/sheet policy is original and must be declared, not
inferred from the reference. Preflight aggregate counts, page/viewport extents, real glyph
and stroke ink, font readability and existing renderer limits. Later-record failures,
empty CTABs, unsupported mapping, 3D/stereo/query data and degenerate drawings must produce
record-specific or honest global errors; never skip a record or generate fake geometry.

Capture full owner/revision/generation before existing registered off-lock work. Poll
cancellation between records and in new loops; acknowledge bounded parser calls that are
not internally cancellable. Reject stale UI cache results using source/revision/generation,
viewport/DPI/profile. Clear old pixels/hits/gestures immediately on key changes. Participate
in existing shared worker/close draining, without a Save reservation or new thread pool.
Extend the existing render.capture path for a declared SDF virtual page and wire the same
producer to read-only UI; preview IDs cannot be consumed as authored bond/edit targets.

## Exact first test and completion evidence

Add original mixed V2000/V3000, VAL-absent ethanol/methylamine source with independent
literal geometry/connectivity and display H [3,2,1]/[3,2], authored H still None, OH/NH2
labels and exact unchanged source. Observe the real missing-API RED for:

```sh
cargo test --locked -p bondline-io --test sdf_preview valence_absent_records_resolve_display_h_without_authoring_it
```

That planned first gate now has actual RED20654 and scoped GREEN evidence in
[the data record](../verification-sdf-preview-model.md). Continue verifying actual source bytes with the
unchanged raw/per-MOL consumers and separate external H expectations. Keep physical H,
explicit H0, unusual declarations, all-record remapping, unsupported later-record and
orientation failures. CPU/GPU must agree on actual glyph/stroke regions and both records;
nonblank output alone is insufficient. Preserve approved golden; new golden approval is
separate. Test exact Save/reopen after preview, field edit/Undo/Redo invalidation, cancel,
source/viewport changes and close draining. Actual native Open/display/Save/reopen remains
unverified until the user is available; headless code/input tests are not native evidence.

Existing reference evidence distinguishes the ordinary760-byte CC/CCC input from the
committed690-byte regression; only the mixed777-byte fixture is the observed committed
mixed input. Neither proves exact grouping, packing, field retention or reference Save
fidelity. See installed-sdf-import.md. Actions stays disabled throughout development.

## Delivered headless boundary

[Data](../verification-sdf-preview-model.md), [renderer](../verification-sdf-display-render.md)
and [shared capture](../verification-sdf-capture-command.md) close the restricted source-to-
image prerequisites. Independent actual PNG acceptance covers ordinary VAL-absent CCO/CN
in two source orientations, both records, OH/NH2, full CPU/GPU/literal-reference RGBA and
semantic negative controls; CLI/stdio/authenticated loopback preserve exact whole-source Save.
The renderer still supports only neutral nonisotopic/nonradical Single depiction, with
honest complete-capture failure for unsupported records/ink/scale. No native view, new golden
approval, general cleanup or reference arrangement parity follows. Next is the UI/job/cache
boundary above, followed by actual native acceptance when the user indicates availability.

## Concrete next native boundary (read-only source review)

The current BondlineApp::update and FileWorkflow::complete_close_if_idle already share an
Engine lock and registered executor. Add one narrow UiCaptureWork mirroring prepare_ui_file:
register under that lock before spawning, instead of invoking execute_blocking for the first
time inside a new thread. Capture an opaque weak-content/revision/generation stamp atomically;
historical PNG results and refreshed close/UI metadata alone cannot certify current pixels.

Use one desired source/view/DPI key, one outstanding job and one ready texture or keyed
error. Cancel and clear stale pixels/errors/gestures immediately. Install only while the
complete key matches and close is Open. Keep the job/pending intent in both close busy gates
through command completion, decode and publication/retirement, including cancelled work.
A bounded PngCapture base64-byte helper can reuse commands' dependency; native eframe already
exposes icon_data::from_png_bytes. Decode and ColorImage construction off-thread; no new
loader/decoder dependency is needed. Validate dimensions/resource/readability bounds, including
logical readability at high DPI; do not assume device-pixel font bounds prove that.

First tests must expose pre-spawn close registration and post-completion decode/publication
gaps, dropped/failed work release, source A→B→A, field edit/Undo/Redo and resize/DPI stale replies,
busy retry/cancel/error behavior, actual both-record PNG pixels and unchanged authored/saved/
trusted-file state. These are proposed tests, not executed UI evidence. Actual native workflow
checks still wait for explicit user availability; Actions remains disabled.
