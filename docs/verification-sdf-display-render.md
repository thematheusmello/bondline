# Display-only resolved-H renderer prerequisite

Verified headlessly on macOS, 2026-11-14. This is the shared renderer prerequisite
for SDF preview, not application SDF display, chemical perception, a native-window
workflow, or a new human-approved golden.

## Contract and source

`graph_page_scene_with_display_hydrogens(width, height, page, counts, cancel)`
requires a `BTreeMap<AtomId, u16>` containing exactly every page atom. Any authored
`Some(n)` must equal its supplied count, including zero and large literal counts.
Missing, extra or conflicting entries refuse with `UnsupportedChemicalContent`.
The renderer chooses no valence, mutates no page, and leaves authored `None`
unchanged. Positioned neutral, nonisotopic, nonradical elements and Single bonds
remain the supported profile. Physical H cannot itself carry attached H.

[IUPAC graphical representation recommendations, GR-2.1.5](https://iupac.qmul.ac.uk/drawing/drawing.html#GR215)
place the element at the original bond endpoint and direct bonds at that symbol.
[GR-2.1.6](https://iupac.qmul.ac.uk/drawing/drawing.html#GR216) describes reversing
labels to avoid bonds, and [GR-2.1.7](https://iupac.qmul.ac.uk/drawing/drawing.html#GR217)
permits a second line above or below for labels with bonds on both sides.
The reviewed cached HTML is 347,564 bytes, SHA-256
`32d2c2d694059e78b6b6a3ed8c274770377e73058d11c174951e7b4e1ce4f71e`,
at root `target/smiles-layout-research/iupac-drawing.html`, lines 737–820.
Reference text and images are not shipped as product assets.

The implementation first tries the existing horizontal preference, then the
other side, above and below. The H component must not intersect any incident
bond polygon. Stacked H and element components have separate white masks and
hit rectangles: intervening whitespace cannot cover or select an incident bond.
The chemical anchor and all source bond coordinates stay fixed. A completely
masked bond refuses; H is never suppressed to obtain clearance. The one-em line
spacing, deterministic candidate order, font style and finite bounds are original
Bondline policies, not claimed reference-product defaults. Existing unrelated
bond/label overlap refusals remain, including conservative whole-label overlap
checks; this is not a general congestion solver. Symmetric multi-character
element centering remains outside this slice.

The additive entry validates actual glyph rectangles, white masks and all four
stroked butt-cap corners inside the paper interior as well as the viewport.
Paper fitting alone is not treated as an ink proof. Cancellation, the 4,096
primitive limit and 4,096 hit-target preflight remain enforced. The old literal
entry retains its previous geometry, clipping and unknown-H semantics.

## Tests and independent expectations

- `display_attachment.rs`: original C—N(H1)—C and C—C(H2)—C opposed coordinates;
  finite chemical anchors, both incident corridors, visible H on another line,
  exact page retention, and CPU/GPU RGBA equality. Requiring a bond hit at the
  corridor avoids confusing black H glyph ink with a black bond pixel.
- Eight mirrored/vertical terminal OH/NH2 scenes compare every CPU RGBA byte
  against manually authored literal-H pages through the protected old entry,
  then compare GPU bytes. Source pages with `None` stay unchanged. Upper/lower
  third bonds on literal methine test the free second line without rotating the
  molecule; a short bond hidden by its labels refuses.
- `display_hydrogens.rs`: missing/extra/mismatching maps, authored 0/1/12/65535,
  unchanged unknown H, physical H, positive empty page, multiple fragment/atom
  IDs, cancellation, actual paper edges, 4,097 hit targets, and 300 separated N
  glyphs exceeding the primitive budget.
- The paper unit test distinguishes interior centerlines from stroke corners
  outside the paper; black glyph rectangles outside the interior also refuse.
- `display_label_oracle.py` independently composes pinned FreeType masks from
  fixed OpenType metric vectors. All 24 C/N/O × above/below × H1/2/12/65535
  cases pass: minimum whole-label IoU **0.9553025152957172**, threshold **0.90**.
  Its uniform 4×, BASIC, 96/64-em, BOX policy is the same as the existing
  elemental oracle; there is no fitted translation or threshold change.
  Wrong symbol, opposite line and dropped-H controls reject. These controls
  do not establish universal digit discrimination or chemical validity of
  unusually large literal H counts.

Pinned consumer: the existing isolated `BONDLINE_PYTHON`, Pillow 12.3.0,
FreeType 2.14.3. Original Apache Roboto SHA-256 remains
`56a45233d29f11b4dfb86d248e921939d115778f87325e7ae8cc108383d6664d`.
The old elemental oracle still passes all 80 cases, minimum whole-label IoU
`0.934356629653821`; its narrower digit-control qualification is unchanged.

## Actual RED/GREEN record

All commands used this worktree's absolute `CARGO_TARGET_DIR`,
`CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0`,
`CARGO_PROFILE_TEST_DEBUG=0`, and root's pinned isolated Python. Toolchain:
rustc 1.94.1 / cargo 1.94.1. Logs below are disposable `target/` artifacts.

| Check | Actual result | Log |
|---|---|---|
| Initial additive API test | Compiler RED, missing exported API, exit 101 | `sdf-display-api-red.log` |
| Old literal opposed N(H1) scene | Runtime RED, no second H line; exit 101 | `sdf-display-attachment-red.log` |
| Strengthened old literal corridor assertion | Runtime RED, Atom2 returned where Bond1 was expected; exit 101 | `sdf-display-attachment-hit-red.log` |
| Initial additive/attachment cases | 3 tests GREEN, exit 0 | `sdf-display-first-green.log` |
| Independent stacked masks | 24 comparisons and 3 controls GREEN | `sdf-display-expanded-first.log` |
| Expanded terminal comparison | Runtime RED, vertical terminal pixels changed; 2 pass/1 fail | `sdf-display-expanded-attachment.log` |
| Production clearance correction | Same expected terminal pixels and all 6 integration tests GREEN | `sdf-display-expanded-green.log` |
| `cargo test -p bondline-render --locked -- --nocapture` | **45 pass**, exit 0 | `sdf-display-render-full-tests.log` |
| `cargo clippy -p bondline-render --all-targets --locked -- -D warnings` | Exit 0 | `sdf-display-render-clippy.log` |
| `cargo fmt -p bondline-render -- --check` | Exit 0 | `sdf-display-render-fmt.log` |

The expanded terminal RED exposed an overly strict candidate guard equating
whole-label and symbol-only clipping intervals. It unnecessarily moved a valid
vertical terminal carbon label. Production now retains existing horizontal
padding when the actual H component is clear. Expectations were retained;
only oversized Frame Debug failure output was replaced with a first differing
RGBA-byte diagnostic. The failed log is retained (13,263,831 bytes; SHA-256
`5bd9dbb24c3e32c3d36b36735221285bdce093dd9b3726ae035efe5a99516f95`).

Final test log: 6,646 bytes, SHA-256
`d1076269fe0285d714c936ed05b67cdca19775bcae928ae96fa0647014c21472`.
Clippy log: 544 bytes, SHA-256
`6b59ac35f45c1582a4e4e7b9172b2e7e1b09a1eb746df1cfb1b7e57de351ead1`.
The complete ignored handoff records all source/log and protected-file hashes.

Offscreen GPU evidence is Apple M3 Max, Metal, IntegratedGpu. Ten new CPU/GPU
scene comparisons pass, alongside the existing renderer GPU checks. The approved
single-bond golden remains exactly
`1b650231527f47db5dc7c368644939907d2b038d32f25e77578318f483521bbc`.
All prior renderer test/oracle/golden files, font, Cargo manifest and lock remain
byte-identical to the pre-change snapshot (23 protected files). No dependency,
model, parser, chemistry, UI or application-source authority was changed.

Native windows were not launched or activated because the user is using the
desktop. Windows/Linux native chemical workflows and a newly human-approved
stacked-H golden remain unverified. No hosted workflow was enabled, dispatched
or rerun; Actions stay disabled. All producer jobs are terminal. Root must
integrate the actual IO sidecar and application capture and run its receiving
workspace gate before attributing those workflows to this renderer evidence.
