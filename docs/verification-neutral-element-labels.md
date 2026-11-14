# Neutral elemental label preview

2026-11-14, isolated bondline-render-labels/feat/neutral-element-labels, basec2b739a.
This is a shared headless-rendering prerequisite, not atom-authoring tools, a new native
workflow, chemical perception, full source/style fidelity or completed M1/M2.

## Contract and boundaries

GraphScene now labels positioned neutral/nonisotopic/Radical::None elements, with Single
bonds only. All118 catalog elements have label-ink coverage. Authored H0 omits H, H1 adds
H, and larger u16 counts use literal decimal subscripts. For noncarbon atoms unknown H
omits H while the model keeps None; this is not a chemical-completeness certificate.
Physical H with positive inline attached-H count refuses. No valence, charge, isotope,
radical, stereo, bond-order or implicit-H inference was added.

The old carbon path retains its authored H labels, skeletal connected unknown-H behavior
and isolated unknown-H refusal. Existing white masks, glyph/stroke/hit/viewport bounds,
overlap errors, cancellation and primitive budgets stay active. Masks may extend beyond
paper into the canvas, as before; physical-page clipping/export remains missing. The
shared renderer has no IO dependency and cannot decide whether an imported projection
is authoritative. StoredDocument and the command/UI completeness guards are unchanged.

The original118-symbol catalog moved verbatim from IO to pure model lookup methods
Element::from_symbol/symbol; IO's existing Option-returning functions delegate. No model
heap field, serialization meaning, public format error, dependency, lock, asset or notice
changed. The unchanged external MOL test still checks all118 symbols with RDKit.

## Depiction and original style

The reread [IUPAC2008 recommendations](https://iupac.qmul.ac.uk/drawing/drawing.html),
GR2.1/2.1.1/2.1.5/2.1.6, supply proper-case symbols, adjacent/subscripted H, character
anchoring and separation of labels from visible bonds. Cached HTML347564 bytes/SHA256
32d2c2d694059e78b6b6a3ed8c274770377e73058d11c174951e7b4e1ce4f71e.
The existing [IUPAC4May2022 catalog](https://iupac.org/wp-content/uploads/2022/05/IUPAC_Periodic_Table_A3-04May22.pdf)
supplies the same facts already present in IO; no table artwork or prose is bundled.

The current horizontal-neighbor policy chooses the less occupied side for H, preserving
carbon ties/storage-order behavior. Two-character symbols use their first/last character
as the respective facing anchor. General congestion solving, highly symmetric whole-symbol
centering, multiline labels and styled text remain deferred. This is a bounded original
placement policy, not a claim of complete recommendation/reference-product parity.

The existing Apache Roboto asset,1pt stroke,12pt em,2/3 subscript scale,1/4-em baseline
offset and0.5 coverage threshold remain. Its515100 bytes/SHA256
56a45233d29f11b4dfb86d248e921939d115778f87325e7ae8cc108383d6664d and notices are untouched.
New metric expectations come from a stdlib OpenType cmap4/hmtx/loca/glyf read of that
asset, not the Rust renderer. The retained probe covers N/O/F/l/B/r/S/i/e/g plus the
counterfactual uppercase L; existing C/H/digit vectors and oracle remain unchanged.

## Test-first and independent evidence

Before production edits, session49914 exited101 on the new positioned N–O graph with
UnsupportedChemicalContent. Session69634 then exited101/E0599 for missing model catalog
methods. The minimal implementation passed both tests in session19504. These initial
logs retain the then-unused test import warning; final warnings-denied clippy is clean.

Additional coverage checks exact authored positions/hit IDs, endpoint reversal, all118
neutral symbols, None versus Some(0) without model mutation, H0/1/2/3/12/65535, physical-H
refusal, multiple fragments, unrelated-bond/label overlap and unreadable/offscreen errors.
Two old unsupported-element examples used N/O; only those obsolete vectors change to
physical H with positive attached H. Every other old metadata/order/error assertion stays.
All previous carbon test functions, font oracle, fixture/profile data and approved golden
are retained; new N/O/halogen/catalog coverage is affirmative.

Actual native-profile CDXML and V2000 MOL files for original NH2–OH are saved to Unicode
paths, independently consumed with unchanged Expat/Decimal and RDKit scripts, reopened
with complete source and compared to the same model/CPU frame. MOL positions independently
check3,-9 and6,-9 under24pt/unit/origin72,72; external formula H3NO and SMILES NO remain
consumer evidence, not Bondline calculations. Cross-format MOL→CDXML refusal and an
opaque CDXML's false completeness/stale-export guard remain. No authority flag was relaxed.

## Font measurement correction, with failures retained

The first new96-em FreeType comparison retained the original0.90 whole-label IoU threshold.
Three of80 cases failed: N/right/H65535=.899778691116029;
Si/left/H0=.8903225806451613; Si/left/H65535=.8814984709480123.
All80 were exercised in the diagnostic pass; no case or literal count was removed.
The initial combined lib/golden command stopped at its font failure, so that log does not
establish a golden pass. Separate session69339 and the final scoped run do establish it.

Actual masks/positions and independent TTF metrics agree. The pinned
[Pillow12.3.0 driver](https://github.com/python-pillow/Pillow/blob/12.3.0/src/_imagingft.c)
uses FT_LOAD_DEFAULT; it exposes no no-hinting switch in the used path. Cached source
52402 bytes/SHA256ac833fac38fb87d53b82e49327395ee0952134e09869df2ae206b74dd90d0a20.
[FreeType2.14.3](https://freetype.org/freetype2/docs/reference/ft2-glyph_retrieval.html#ft_load_xxx)
documents grid fitting under that default. Rust rasterizes unhinted design outlines.

Only the new test oracle changed: every one of80 positives and all controls now uses one
4x FreeType/BASIC/BOX integration policy at the same final96-em canvas. Whole-label IoU
still must be>=0.90. No per-case resize, translation, production/font adjustment, threshold
relaxation or unapproved golden was used. The old carbon oracle/golden are byte-exact.

The final whole-label minimum is.934356629653821. Wrong element, case, H side and chemical
anchor controls reject. Global overlap alone can miss a small changed digit: a scoped
N/H2 positive ROI scores.969639468690702, versus H3's.5078909612625538. This control uses
the same reference and its own positive, not a general wrong-digit-detection claim.
An exploratory all-glyph threshold failed four additional thin-glyph comparisons; its
minimum.8666666666666667 remains diagnostic, not an adopted general glyph contract.
That exploratory RED and the intermediate whole-label/negative-control RED are retained.

Root-requested bounded convergence on the three original masks:

| Case | Actual/4x reference | Actual/8x reference | 4x/8x reference |
|---|---:|---:|---:|
| N/right/H65535 |.9660906423214868|.9894127377998345|.9735467015022861|
| Si/left/H0 |.9825304393859184|.9830598200105876|.9804749340369393|
| Si/left/H65535 |.9731511254019293|.9757117580826765|.9681427206116597|

The1928-byte ignored convergence receipt SHA256 is
7874b5bf9196a34b4e1a42e1a94421b8ad91309ed06138d6b14607b7f4d69780.
It identifies the inspected diagnostic script revision; final changes affect diagnostic
output/negative controls only, not that4x/8x reference geometry. Native96 failure masks,
JSON and logs remain ignored and retained. The L counterfactual metric was corrected to
the actual probe1102/[169,0,1052,1456] before the final scoped execution and is asserted
against the pinned font. An earlier focused pass is not substituted for the final source.

## Final scoped checks and remaining evidence

macOS27.2 arm64, actual Apple M3 Max/Metal/IntegratedGpu. All Cargo commands use this
checkout's absolute target, incremental0/dev-test debug0 and the existing pinned external
Python interpreter (Python3.13.0/Pillow12.3.0/FreeType2.14.3/RDKit2026.09.2). No install,
GUI/input/activation, Git mutation, hosted workflow or external account action was used.

Session62946, cargo test -p bondline-model -p bondline-render -p bondline-io --locked:
276 passed,0 failed/ignored, including actual CPU/GPU equality and every RGBA byte of the
approved single-bond golden. Log35059 bytes/SHA256
68a7d327bafc79fb7c7cff60b34f8be6bbc88ec2950b1f7d84b355def3a4ce93.
Session54831 warnings-denied all-target clippy for the same3 packages exits0;
log3003 bytes/SHA256c02e5d5ae05f2ffa5c95792cc46d94f926bf0aca242a5ee7d92272b9d51fb484.
The standalone final workspace rustfmt check exits0 with an empty log. Full receiving
workspace/layer/license/corpus checks belong to Root. No worker job remains live.

The user-approved golden remains SHA256
1b650231527f47db5dc7c368644939907d2b038d32f25e77578318f483521bbc.
Neutral labels have headless local evidence only: new native UI/accessibility/DPI and
Windows/Linux execution are unverified. Actions remains disabled by user policy. General
text/charges/isotopes/stereo/orders/layout, faithful unmapped CDX/SDF preview and new label
authoring controls remain outside this slice. No milestone or native parity is complete.
