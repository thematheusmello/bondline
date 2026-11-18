# Chain authoring: locked engine increment

2026-11-16, base f077ed2. This is a public-source contract and original Bondline
policy, not observed26.1 behavior. The native chain controls stay disabled until their
complete interaction is implemented and checked. Full M0–M5 remains required.

## Evidence and remaining workflow

[GUIDE-25](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf),
printed104–105, documents drag/count, atom attachment, separate Snaking/Control,
click-to-open Add Chain and Alt-repeat. Printed42 (Drawing Settings; extracted text
line1616) documents an interior Chain Angle from1 through179 degrees. Printed74
documents Shift+x; its exact26.1/Mac behavior is unverified. Fixed Lengths and Fixed
Angles have conditional behavior; do not flatten their modifier rules into a universal
reversal shortcut. The complete native workflows/settings remain subsequent work.

[IUPAC2008 GR3.2](https://iupac.qmul.ac.uk/drawing/drawing.html#GR32) recommends
normally horizontal overall chains; it does not mandate a30° first edge.
[GR4.1](https://iupac.qmul.ac.uk/drawing/drawing.html#GR41) recommends120° ordinary
Single-chain interiors and trans progression, with180° at Single+Triple/two-Double
centers. A literal C4 budget is not stereo perception or an automatic attachment pose.
The explicit headless first edge below never rotates existing geometry to satisfy a
preferred depiction. A later automatic native pose must account for these distinctions.

Public guide PDF17,721,447B SHA256
3f1ebc429df07efdd47b6218a148dc346a215043f60bf89ee9a1cb90629bd5b3;
extracted text728,816B SHA256
6af3ecb27a0b6c19f2b64005ad95582ba7795876c54a42bf210232f75e84c43f;
IUPAC HTML347,564B SHA256
32d2c2d694059e78b6b6a3ed8c274770377e73058d11c174951e7b4e1ce4f71e.
Source refresh report5835B SHA256
5a6abc22573b957abce5763ce4a6c89534df92142838dc9aca1d6f57603025ee
is disposable under target/chain-authoring/source-refresh. No proprietary program,
asset, template or sample is used.

## Typed request and authoring policy

Add exactly one registry command, `chain.create`, with closed typed `ChainCreate`:
required expected_revision:u64, positive page_id:u32, count:u8 in2..64,
angle_degrees:f64 in1..179, hydrogen_policy:"rectify_neutral_carbon", and placement.
No defaults, positional arrays, null/unused selectors or unknown fields are accepted.
Count includes an existing anchor. This inclusion and the2..64 resource bound are
original policies; the public guide does not resolve shared-anchor counting.

Placement is a closed mode-tagged `ChainPlacement`:

- free: begin_x_pt, begin_y_pt, end_x_pt, end_y_pt, side;
- atom: positive fragment_id/atom_id, end_x_pt, end_y_pt, side. The start is exactly
  the target's authored position. No edge-sharing, merging or implicit snapping.

Coordinates are finite physical points within the existing±100000 model bound.
`ChainSide` is positive/negative: with φ=180°−θ, positive uses
r=(cosφ e_x−sinφ e_y, sinφ e_x+cosφ e_y), so cross(e,r)>0. This is visually clockwise
on the y-down page. Negative flips φ. The explicit first edge defines length and orientation. There
is no inferred48pt default, page containment, automatic scaling or rendering threshold.
Angle remains explicit even for a two-carbon result. All new edges are directed Single.

Only source-free Drawing or completely mapped native CDXML has geometry authority.
Wrong content and incomplete/foreign/SMILES/CDX/MOL backings remain refused. A file
association alone does not establish authority. Unrelated metadata needs no new
renderability/chemistry check. Existing byte ownership and loss guards remain.

An attachment requires positioned, neutral, natural, nonradical carbon; incident
Single/Double/Triple weights must sum to≤4, literal H must equal4 minus that sum,
and at least one H must be available. This is target-local, independent of degree and
neighbor element. Aromatic/query incident orders are unsupported. Decrease only the
anchor's H by one. Preserve every other old atom field/array entry, bond direction,
order, DoublePosition, coordinate, ID and unrelated fragment Arc exactly. No chemical
normalization, resonance rephasing or stereo inference is performed.

Free creates count new C in one new fragment. Attachment appends count−1 C to the
existing fragment and spends no fragment ID. New carbon has literal H3 at a terminus,
H2 internally, both ShowTerminalCarbonLabels and ShowNonTerminalCarbonLabels false,
and ordinary neutral natural atom defaults. The existing isolated C/C2 mandatory-label
rule still applies. Flags do not erase H or authorize lossy export.

## Geometry, errors and transaction

Let A/B be the first edge, e=B−A, θ=angle_degrees and r the signed rotation of e
by180°−θ. Independent construction is P_i=A+ceil(i/2)e+floor(i/2)r for i0..count−1,
with P_0/P_1 retained exactly as A/B. This alternates two fixed vectors rather than
accumulating rotations. Reject duplicate rounded vertices. Certify every actual
rounded edge against length hypot(e), relative norm tolerance1e−10, and every interior
angle against θ within1e−10 radians, with alternating signed turns. Divide by length
before products; do not square tiny lengths or impose a blanket minimum. Collapse,
nonfinite or distorted precision refuses atomically.

Every new center must differ exactly from all existing atom centers on that page.
This is center occupancy, not a complete crossing/ink/paper collision guarantee.
The native tool will separately validate the full candidate through existing physical
and logical renderer guards. Repeating occupied geometry refuses; creation has no
successful no-op.

Existing outer Engine guards keep their precedence: committed close, then stale
expected_revision before the builder. Like ring.create, a valid headless edit may
invalidate a pending close/file context; the native foreground owner separately
blocks canvas gestures. Do not introduce a new global pending-intent mutation ban.
Within the builder:
source authority, count/angle/policy, page lookup, target lookup/H proof/position,
first-edge/rounded geometry, page occupancy, checked IDs, model validation, commit.
Deserialization precedes dispatch and need not use this builder precedence.
Invalid requests/geometry use invalid_params (new InvalidChain); unsupported target
chemistry/position uses unsupported_content (new UnsupportedChain). Existing missing
target/page, occupied endpoint, model/resource/ID/revision errors retain their codes.

Reuse bond::Ids/maxima and one immutable StoredDocument candidate. Publish identity
floors only after one normal Engine commit. Undo/Redo restores complete content;
floors remain monotone across history. Refusals preserve content/owners, source, file,
selection, history and floors. Existing selection invalidation applies on success.
Like existing immediate ring authoring, prepare commits synchronously; cancellation
after that winning commit cannot retroactively undo it. No worker/job abstraction.

## Independent acceptance and scope

First observe UnknownCommand via the existing JSON runtime API before producer code.
Then cover analytic geometry (both sides, angle range, tiny/rounded cases), literal
metadata, branch/weighted anchors, immutable unrelated state, errors/source authority,
history/floors and real Unicode files through CLI, initialized stdio and authenticated
loopback. Check raw XML flags/attributes before an unchanged independent chemistry
consumer; its aromatic sanitization must not replace the authored integer-bond proof.
Independent fixed graphs/formulae include propane C3H8, isobutane C4H10, propene C3H6
and toluene C7H8. C2 still needs mandatory labels.

Registry38→39; native/control/catalogue counts and enabled bindings stay unchanged.
TD-022 remains missing at the native workflow level. Add Chain, drag counter, repeat,
Snaking/Control, fixed settings, modifier ordering, hotkeys/defaults, complete chemistry,
styles, actual native/platform verification and broader M1–M5 remain required.
