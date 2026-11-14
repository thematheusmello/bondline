# Next M1 fidelity gate: source-backed MOL display

2026-11-14. Read-only source investigation; no new runtime, consumer, GPU or native
failure was executed. The proposed test/API below has not run. The complete goal remains
M0–M5; actual native SDF acceptance still waits for explicit user availability.

Implementation update: the planned runtime RED was subsequently observed and the closed
MOL slice is now implemented. See [shared facts](../verification-mol-display-model.md),
[complete ownership](../verification-mol-capture-owner.md),
[independent captures](../verification-mol-display-tests.md) and
[headless native-canvas wiring](../verification-mol-display-ui.md). The historical plan
below retains its exact original expectations and unexecuted-at-authoring qualifiers.
Actual native-window acceptance, broader orders/perception and M1 remain incomplete.

## Evidence and requirement

MOL V2000/V3000 strict mapping correctly retains absent valence as authored H=None.
StoredDocument classification can mark these ordinary molecules fully mapped because
encoding retains that fact. This establishes source coverage, not faithful depiction.
CaptureInput::Drawing and both native drawing graph-construction sites still use the
literal renderer; its unknown noncarbon H becomes zero for labeling. Consequently bare
O/N is reachable for ordinary neutral OH/NH2 source-backed MOL. This is a source-derived
finding, not a newly observed native error.

[BIOVIA Chemistry2026 CTfile Formats](https://www.3ds.com/assets/invest/2026-01/biovia_chemistry_2026_ctfile_formats.pdf)
defines unmarked/declaration fields, not a universal normal-valence algorithm.
[IUPAC GR2.1.1](https://iupac.qmul.ac.uk/drawing/drawing.html#GR211) requires faithful
heteroatom/H depiction. The existing original closed C/N/O/halogen display profile has
published OpenSMILES/RDKit support, explicit provenance and independent SDF evidence;
it can be reused without altering authored None or claiming general perception.

## Exact first unexecuted RED

Use the unchanged original740-byte fixture
crates/io/tests/fixtures/sdf-preview/valence-absent-mixed.sdf. Its raw MOL spans are:

| Span | Actual version | Default individual-MOL model coordinates, pt | Display H |
|---|---|---|---|
| 0..371 | V3000 ethanol | (42,60), (78,60), (78,24) | 3,2,1 |
| 410..713 | V2000 methylamine | (108,108), (72,72) | 3,2 |

Open each raw span through document.open_mol at the existing24pt/unit/origin(72,72).
Assert fully mapped complete backing, exact authored None and these actual coordinates.
Require real1024² CPU/GPU PNGs against independently authored literal-H expectations and
the unchanged external mol_oracle on the exact raw bytes. These coordinates differ from
SDF-packed coordinates; never translate, mirror or repack a MOL to make the test pass.

Suggested next command after adding the real test:

```sh
cargo test --locked -p bondline-commands --test mol_display ordinary_valence_absent_mol_displays_oh_and_nh2_without_authoring_h
```

This is a planned command, not a pass or an observed failure. Also check exact source
Save/reopen, source/state/history/association preservation, explicit H0/unusual literals,
unknown/unsupported refusals, cancellation and historical complete-owner retention.

## Smallest complete source-aware route

Factor only shared topology/default-H facts from io::sdf::preview into reusable IO code.
Keep existing SDF record context, physical-H policy, typed errors, provenance and unknown/
literal semantics exact; existing callers/tests must continue to work. Apply source-backed
MOL display to its actual current Page/IDs/frame, never SDF row packing. The renderer
consumes resolved facts and still chooses no valence.

Deferred drawing capture needs the complete immutable drawing owner for source-aware
classification, rather than only a cloned Page. Route both initial and post-edit native
graph acquisition through the same source-aware producer. Preserve source-free/native/
CDXML literal behavior and every existing edit/Save guard; a display fact grants no new
encoding authority. Source format/backing is distinct from associated filename/file format.
Generic CDXML absent-H interpretation remains a separate sourced question. Do not narrow
already supported literal elements/counts to only the derived profile.

Actual native workflow validation remains separate. No new golden approval, dependency,
proprietary asset, hosted run or whole-M1 completion follows from this next gate.
