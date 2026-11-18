# Named benzene and cyclopentadiene authoring

2026-11-16, based on skeletal checkpoint5fa23d1/2271bc5. Lock this command increment
before product changes; native tool integration follows through the same builder. Full
M0–M5, automatic styles, aromaticity perception and broader ring workflows remain required.

## Public basis and explicit policy

[GUIDE25](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf), printed
101–103/423–424, documents named rings, oriented drawing/attachment and Double-first
versus Shift Single-first. Cached PDF17721447B SHA256
3f1ebc429df07efdd47b6218a148dc346a215043f60bf89ee9a1cb90629bd5b3;
layout text728816B SHA2566af3ecb27a0b6c19f2b64005ad95582ba7795876c54a42bf210232f75e84c43f.
Root reads the resonance paragraph directly. New web access fails its17.7MB size limit;
the unchanged public-source cache remains usable. Version25 is not complete26.1 evidence.

[IUPAC GR1.10.4/3.5](https://iupac.qmul.ac.uk/drawing/drawing.html#GR1104) supports inward
new Double offsets and alternating six-rings, with fused-face preferences. The
[SDK DoublePosition](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Bond_DoublePosition.htm)
defines directed sides. Root verifies the primary IUPAC text. None of these specifies
all attachment phase ties or permits silent rewriting of authored bonds. Wire shape,
phase ranking, fixed-H proof and metadata preservation below are original Bondline policy.
The reviewed proposal9306B/SHA2568d348c974ee0962559ac835bfb8fd69dd88235eb9ba8450cef17c56f3898f56f
and receipt6181B/SHA25676be77e36f803037ba2fa28717b87e50f73eabf45fa57e8e872ca64048a1748c pin sources.

## Closed shared command

Extend ring.create, keeping registry38. Accept either the exact legacy size3–8 object,
or named kind=benzene/cyclopentadiene with derived size6/5 and no size field. Common
expected_revision/page_id/hydrogen_policy/placement fields remain. Named requests alone
may supply phase=double_first/single_first; omission is contextual. Reject null, both or
neither selectors, unknown keys and phase on a legacy request. Schema must express these
same two closed objects and the optional non-null phase. Legacy JSON/geometry/H/default
metadata and public RingCreate struct stay exact. A typed request union can wrap it;
compiler-required Command variant conversions change no old test expectations.

Reuse the existing complete-source guard, polygon/precision/occupancy certificate,
ID floors, fragment validator and one normal commit/history. Do not create a new model,
converter, worker, runtime dependency or dispatcher. Preview eventually calls this same
candidate builder without publishing IDs or Save authority. No detached fallback.

## Integer patterns and phase selection

Perimeter edge i joins vi→v(i+1). Benzene base[2,1,2,1,2,1] has two distinct left
rotations; cyclopentadiene base[2,1,2,1,1] has five. Enumerate increasing rotation index.
Free omission chooses0; explicit phase filters edge0 weight2/1 before choosing the first.
For attachments, choose the lowest phase matching the preserved shared-edge order and
all proven target H budgets, honoring an explicit phase without fallback. This contextual
tie policy is original, not a claimed reference algorithm. No aromatic bond flag or
perception call is inferred from a named integer template.

Prove existing shared atoms neutral/natural/nonradical C with supported S/D/T incidence
and literal H=4−Wold once. Subtract only new incident weights. Free new atoms have checked
H=4−(previous+next weight): sixH1 for benzene, fourH1/oneH2 for cyclopentadiene. All new
atoms opt into ShowNonTerminalCarbonLabels=false; old flags remain exact. New Double
placement is Right for positive side, Left for negative; new Singles stay Unspecified.

- Atom anchor v0 consumes edge0+edge(n−1). Benzene needs3H. Cyclopentadiene phase4 can
  consume2H, enabling a chemically valid spiro union; do not consider only two patterns.
- Bond placement retains exact old B→E as v0→v1 and matches only Single/Double order.
  Consume edge(n−1) at B and edge1 at E, never recounting the reused edge. Cyclopentadiene
  Single-first rotations1/3/4 require endpoint H budgets(2,2)/(2,1)/(1,2).
- All old bonds retain ID/order/direction/DoublePosition and all old atoms retain other
  fields/coordinates/order. No neighboring resonance rephasing or inferred-H repair.
  Identity floors publish only after success; failures leave complete state/source exact.

## Shared style and native continuation

Preserve an existing shared Double's Center/Unspecified/Left/Right exactly, including a
side facing an older ring. **Do not impose a blanket style refusal** on chemically valid
headless construction, and do not rewrite the side to beautify the new ring. This differs
from the proposal's conservative style ban. Automatic IUPAC-preferred fused-face solving,
explicit restyling and resonance rephasing remain missing, not implicitly delivered.
The actual native candidate must still pass every existing physical/logical ink, gap,
mask, hit, paper, scale, resource and source guard; a real geometry refusal stays truthful.

Native integration adds two named controls to the same effective tool/epoch/context and
original procedural icons. Single-six versus benzene are distinct modes. Legacy size
controls and modifier behavior stay exact. Shift-only selects Single-first for named
gestures; lock its ordered press/move/release timing in the native plan before App tests.
Retain preview/cancel/source/view/tool/file/close/DPI fences and one release/one commit.
The user's open older bundle/draft is not touched or hot-reloaded. Native/platform checks
remain separate from Engine, actual adapters and offscreen App evidence. Actions stays off.

## Independent acceptance

Observe an existing-JSON named-kind runtime RED first, separate from any new Rust API
compile qualification. Original analytic polygons/order/H/flag/side vectors, all phases,
asymmetric shared budgets, reversal and old metadata retention precede implementation.
Unicode actual CDXML must pass unchanged Expat/Decimal and pinned RDKit consumers for
C6H6/C5H6, tolueneC7H8, cyclohexane/benzene fusedC10H12 and Cp-spiroC10H14. Preserve exact
legacy vectors, malformed/schema/source/precision/budget/history/identity guards. Verify
CLI/initialized stdio/authenticated loopback through the same registry, then actual App
and native free/atom/bond/Shift/history/Save workflows. MOL loss refusals remain strict.
