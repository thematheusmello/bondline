# Centered Double/Triple rendering

2026-11-14. Shared renderer implementation from
[multiple-bond-plan.md](research/multiple-bond-plan.md). This is the original
`centered_multiple_v1` display profile, not a measured Professional26.1 default.
Command/tool/native integration and source-aware adapter error mapping are Root's
separate receiving work. No native window, input, activation, hosted Actions,
reference runtime, dependency/lock, fixture, font, shader or golden change occurs here.

## Delivered geometry and scene APIs

```rust
centered_bond_strokes(&Stroke, BondOrder) -> Result<Vec<Stroke>, Error>
GraphScene::bond_strokes(FragmentId, BondId) -> impl Iterator<Item = &Stroke>
```

The constructor validates the axis and supports exactly Single, Double and Triple.
Single returns one exact unchanged axis. With existing canvas stroke width w,
adjacent separation is s=3w: Double offsets are -s/2,+s/2; Triple offsets are
-s,0,+s. Offsets follow the unit normal to authored begin→end, shifting complete
butt strokes without moving atoms or changing source order/coordinates/H. The
original short-axis guard is s<=L/3, or L>=9w, for Double/Triple only. It is not a
vendor/journal preset, universal IUPAC conformance threshold or inherited file style.
The public constructor and GraphScene use the same implementation. Preview callers
must distinguish geometry success from scene-level checks below.

One BondHit, original axis, endpoint pair, BondId and neighbour vote represent each
logical bond. Component strokes and viewport-clipped polygons provide paint/hits.
The accessor returns the actual component primitives before common masks/clipping,
one for Single and none for a missing target. A white inter-line gap has no zero-
tolerance axis hit. Atom/label priority and stable-ID tie arithmetic remain unchanged.
The existing logical4096-hit bound and actual expanded4096-primitive bound stay
separate. Every component counts before geometry allocation; masks/glyph pixels
still count. No component is replaced by an axis-only fake bond.

## Literal data, labels, visibility and congestion

No valence is selected here. The resolved entry still requires exact supplied
coverage and agreement with authored Some(count). Literal Some, including unusual
counts, remains authoritative; connected unknown carbon remains skeletal. Literal
unknown noncarbon retains its previous zero-count display behavior, not an inferred
chemistry claim. Authored None, all metadata, Page geometry and IDs stay unchanged.

Only scenes containing a multiple bundle route their literal labels through the
existing component-aware label-layout choices. This closes a reproduced Double+
Single junction case where an authored H1 prefix hid incident multiple ink. The
same exact authored counts now give the same clear attachment as the resolved
entry, including stacked H where needed. Every component is checked against H
masks and all unrelated label masks. All-Single literal and resolved scenes retain
their prior route/pixels. Existing resolved physical-paper ink checks remain;
literal scenes retain viewport clipping. A required component clipped away entirely
is refused rather than reducing the apparent order.

Each required multiple component needs an actual unmasked CPU pixel center. Each
adjacent gap needs a pixel center outside all painted component ink and label
masks. Favorable subpixel widths0.95 and0.5 pass; invisible/merged phases fail with
typed BondScale. If masking removes otherwise representable ink, LabelOverlap is
returned. No blanket width>=1 or whole-frame allocation substitutes for this check.

The bounded query scans the major axis with per-row cancellation. Integer minor
intervals come from monotone searches using exactly the frozen covers along/cross-
squared arithmetic. This avoids both square-root boundary estimates and a reproduced
near-horizontal finite-cap rounding failure. At most two final candidates are
examined per row, with logarithmic lattice-bound searches. Rectangular label/anchor
exclusions jump by integer endpoints. Painted-stroke exclusions jump past a known
covered contiguous row interval using the same frozen covers predicate and bounded
binary search. No arbitrary clear callback remains. These are bounded queries,
not a hard native-thread latency/RSS guarantee.

The original centered profile reserves a multiple bundle's full corridor from
unrelated bond ink: envelope width4w for Double,7w for Triple. This is an explicit
congestion policy, not a general chemical layout/join solver. Shared bonds may
meet at the authored anchor; only its closed half-pixel anchor square is exempt
from unmasked common-ink refusal. The square is an exact integer query mask and
is never painted. Safe larger/attached graphs pass, with both literal and unknown
H; no terminal-pair topology gate is used. Unsafe overlaps/refused styling return
an error for the complete scene. Single-only crossing behavior is unchanged.

## Independent expectations and source evidence

At a declared w2, horizontal A(96,160)→B(224,160), Double centerlines are y157/163;
Triple y154/160/166. Literal pixel-center strip/gap checks and complete CPU/GPU
RGBA comparisons verify both. The separate analytic A(20,20)→B(68,84) case has
normal(-4/5,3/5), declared separation6 and endpoint tolerance1e-5 canvas units.
Diagonal full-image tests use literal shifted endpoints and independent dot/cross
strip membership, plus whole CPU/GPU equality. Vertical, mirror and authored
endpoint reversal retain the same physical bundle. No image is called human-approved.

Visibility/reference tests compare against an exhaustive64²-grid existence query
using the unchanged covers predicate: 1344 base mask/intersection combinations,
1344 additional actual anchor-square/painted-exclusion combinations, plus explicit
wide-gap and rounded-cap controls. This is an independent search algorithm
reference, not an independent implementation of covers. A separate24576-point
check matches the actual anchor-square predicate to its integer mask. The adopted
short rule tests both sides and exact L=9w. Expanded primitive tests distinguish
4096 from4098 before deliberately degenerate geometry. Cancellation, clipping,
missing hits, masked labels, nonincident corridor occupancy and nonterminal carbon
junctions have decisive checks. Parent file/chemistry acceptance remains separate.

The only intentional old-test upgrades preserve exact original inputs:

- carbon_scene's former Double case keeps both authored H3, original points and
  order2. It now asserts literal two-stroke geometry, exact strip/gap pixels,
  unchanged H3 label pixels against the separately verified Single route, GPU
  equality and complete model retention. Quadruple adds negative coverage.
- graph_scene's former Double/Triple negatives keep authored None and the original
  points. New assertions verify exact components, independent ink rows, hits/gaps,
  CPU/GPU equality and None/model retention. All other unsupported kinds/metadata
  remain in the negative suite.

Primary public sources remain those in the committed plan/source register:

- [IUPAC2008 GR1.6](https://iupac.qmul.ac.uk/drawing/drawing.html#GR16) and
  [GR1.10](https://iupac.qmul.ac.uk/drawing/drawing.html#GR110): parallel associated
  segments, spacing guidance and context-sensitive centered/offset joining.
  Registered HTML SHA-256
  `32d2c2d694059e78b6b6a3ed8c274770377e73058d11c174951e7b4e1ce4f71e`.
- SDK [BondSpacing](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/BondSpacing.htm),
  [BondSpacingAbs](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/BondSpacingAbs.htm),
  [BondLength](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/BondLength.htm) and
  [DoublePosition](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Bond_DoublePosition.htm),
  repository pin d1e4e6ca7d847578d2b032143d686aa8641664e3 under docs/cdx_sdk/properties.
  These establish distinct source style/inheritance/position facts; the renderer
  does not decode them or infer Center from an absent property.

IUPAC and DoublePosition were reopened for this slice. The listed raw hashes/pin
are existing registered research evidence, not new hashes of web-rendered output.
Original3w, exact one-third short rule, pixel/gap/corridor/junction policies are
Bondline choices. No source, diagram, example, artwork, font or reference asset
was transplanted. Ring/offset choice, shortening/joining, style inheritance,
persistent style migration and reference26.1 numeric/pixel parity remain open.

## Actual TDD, failures and final gates

All Cargo commands use pinned1.94.1, the absolute worktree target,
CARGO_INCREMENTAL=0, dev/test debug0 and existing pinned BONDLINE_PYTHON.

1. Runtime RED 72838 exits 101 UnsupportedChemicalContent before production edits.
   The literal Double/Triple strip/gap test and original production305-input
   manifest are retained. First actual CPU/GPU GREEN 88965 exits 0.
2. Raster RED 55208 reproduces invisible Double w=0.95/phase=100.425. First expanded
   GREEN 90961 passes 4 tests, including favorable subpixel controls.
3. Congestion RED 41888→GREEN 5572 passes 5 tests. Analytic short RED 77502→GREEN 6390
   passes 6. Literal junction attachment RED exits 101 in a direct invocation;
   GREEN 51037 passes 7. Public-constructor E0425 RED precedes GREEN 76879.
4. Root review identified the arbitrary-clear two-candidate gap. RED 88216 proves
   a later white pixel was missed behind an actual painted blocker. Source/log
   are retained. Typed exclusions give firstGREEN12120 and expanded lib 22062=14.
5. Cap probe 84381 fails its initial control at dy=1e-10; that input was not rounded
   enough. Corrected literal dy=1e-14 has covers=true but search=false: actual
   RED 26829. Exact monotone lattice boundaries give GREEN 54976, all 4 search cases.
   Two temporary Range::is_empty compiler warnings are fixed with explicit bounds.
6. Package 82982 (--no-fail-fast) identifies only the two now-supported old refusal
   tests; unrelated Single/font/golden tests pass. Intentional upgrades 53586 pass
   all 3 carbon/all 11 graph tests with stronger expectations. No old input/count,
   oracle, fixture, glyph policy or threshold is changed to force agreement.
7. Full package 57981 exits 0:65 tests. Clippy 17251 finds one test-only unwrap;
   equivalent typed Result collection fixes it. Final all-target warnings-denied
   clippy 11325 and rustfmt exit0. The final exact-source
   package 3629 also exits 0:65 passed,0 failed/ignored. Protected source/log
   identities are recorded in the ignored frozen handoff.

```sh
cargo +1.94.1 test --locked -p bondline-render
cargo +1.94.1 clippy --locked -p bondline-render --all-targets -- -D warnings
cargo +1.94.1 fmt --package bondline-render -- --check
```

The approved Single PNG/JSON, font, label oracles, shader/CPU renderer, dependencies,
locks, old fixtures and other crate sources remain byte-exact. Fundamental existing
Stroke validate/bounds/covers/corners bodies are preserved. Single-only golden,
full RGBA, hits, literal counts, external FreeType/Pillow and resource tests pass;
this verifies those scopes rather than all possible inputs/hardware. No approved
asset is regenerated and no native OS/compositor or Windows/Linux evidence is added.
No Git/push, Actions/workflow, GUI/input/activation, dependency installation, cleanup,
broad workspace gate or application/chemistry completion is claimed here.

The protected-before manifest tracks 305 existing inputs. Only the four owned
renderer source files and two intentional feature-upgrade test files differ;
the other 299 remain byte-exact. Eight intentional files, full diff, 27 gate
logs, first-RED expectations, reviewed counterexample sources, final test-binary
identities and exact preserved Single regions are frozen under target. No owned
job remains live at handoff.
