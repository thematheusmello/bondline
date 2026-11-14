# Next gate: Double/Triple depiction

2026-11-14. Planned and unexecuted M1 file-display dependency / M2 drawing work.
Research only: no application, chemistry consumer, render test, native UI or feature code
ran for this plan. Full M0–M5 and context-sensitive professional multiple bonds remain targets.

## Primary evidence and its limits

| Source reviewed | Factual requirement |
|---|---|
| [IUPAC2008 GR1.6](https://iupac.qmul.ac.uk/drawing/drawing.html#GR16) | Multiple segments remain parallel and visibly associated; separation beyond33% of bond length is discouraged. This is no numeric default. |
| [IUPAC2008 GR1.10](https://iupac.qmul.ac.uk/drawing/drawing.html#GR110) | Double bonds may be centered or side-offset. Offset secondary segments use neighbour-angle-dependent shortening; centered segments have different joining rules. Ring/substitution context matters. |
| [SDK BondSpacing](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/BondSpacing.htm) | Relative spacing: CDXML percent versus CDX tenths-percent;18%/180 is an example. Missing values inherit and ultimately depend on last-used style. |
| [SDK BondSpacingAbs](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/BondSpacingAbs.htm) | Absolute segment separation; this property wins a conflict with relative spacing. Its table lists Bond ownership; prose also describes containment. |
| [SDK BondLength](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/BondLength.htm) | A default drawing length with inherited/style fallback, not permission to move existing atom coordinates. |
| [SDK DoublePosition](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Bond_DoublePosition.htm) | Center/Right/Left are directional relative to authored begin→end; CDX0/1/2 versus256/257/258 distinguishes automatic/manual. Same CDXML names appear for both code groups. Missing means unspecified, not Center. |

SDK repository pin: `d1e4e6ca7d847578d2b032143d686aa8641664e3`, files under
[`docs/cdx_sdk/properties/`](https://github.com/IUPAC/IUPAC-FAIRSpec/tree/d1e4e6ca7d847578d2b032143d686aa8641664e3/docs/cdx_sdk/properties).
All four pinned raw files match their published pages byte-for-byte. Initial raw requests
incorrectly omitting `docs/` returned404; the corrected paths and all published pages returned200.
IUPAC HTML is the2008 edition:347,564bytes, SHA-256
`32d2c2d694059e78b6b6a3ed8c274770377e73058d11c174951e7b4e1ce4f71e`.
Exact URL/status/size/hash receipts and raw research stay ignored under
`target/multiple-bond-research/`. No reference26.1 runtime, tier, default or pixel evidence
was collected. Existing register entries CTFILE-2026 / IUPAC-DIAGRAM / CDXML-BOND-ORDER apply;
Root must add the new property sources/questions when implementing.

## Actual current boundaries and necessary callers

| Location | Current behavior / next change |
|---|---|
| `model/src/graph.rs` | Already stores Double/Triple and authored endpoints. No spacing/position/style fields. No new model field is needed for an explicitly original display-only gate. |
| `io/src/mol/{mod.rs,v3000/mod.rs}` and writers | Already map/write orders2/3, retain raw source and declared/unknown H. Reuse; do not rewrite codecs or imported bytes. |
| `io/src/cdxml/{graph.rs,write.rs}`, `document.rs` | Already retain/order-encode2/3. Spacing/DoublePosition remain unmodelled source data; noncanonical imports stay guarded. Encoder/native-v0 must remain unchanged. |
| `io/src/cdx.rs` | Elemental mapping remains Single-only and backing incomplete. Renderer expansion must not unlock CDX projection/edit/export. |
| `io/src/display.rs`, `io/src/sdf/preview.rs` | Derived H already weights Double2/Triple3; literal counts/None, physical-H policies and source guards must remain exact. No renderer inference. |
| `render/src/page.rs` | Explicit non-Single rejection is the scene blocker. Preflight, neighbour voting, labels, incident clearance, full-mask tests and paper-ink validation currently assume one stroke. |
| `render/src/hits.rs` | One BondHit contains one axis stroke/polygon. Picking must group all visible component ink under one existing BondId target. |
| `render/src/{stroke,scene,gpu}.rs`, `label.rs` | Existing butt strokes/CPU/GPU painting and masks are reusable. No shader/font/primitive variant is required. Preserve exact Single output. |
| `commands/src/capture.rs`; native UI | Capture, `Engine::drawing_page_scene`, SDF preview jobs and both native scene-acquisition sites must consume the same scene. Root's integrated UI uses the Engine helper; this research checkout still has older direct render calls. `paint_hover` currently outlines only the axis with width+4 and needs shared bundle extents. |

## Proposed first complete slice, not a reference default

Implement centered Double and centered Triple geometry first; start with isolated terminal
neutral pairs. Side-offset placement, ring choice, adjacent-bond joining and broader substituted
cases remain explicit work; refuse unimplemented contexts rather than call all multiple bonds faithful.
An explicitly adopted original `centered_multiple_v1` policy can use separation `s=3*w`,
where `w` is the existing one-point stroke scaled to the canvas. This is a proposal to freeze
before code, not a discovered SDK/journal/reference default. Geometry tests take declared
numeric spacing; never derive it from the SDK's18% example or an unobserved style sheet.

For axis A→B, validate nonzero finite length before normalization. Let unit normal
`n=(-dy,dx)/hypot(dx,dy)`. Double uses offsets `[-s/2,+s/2]`; Triple uses `[-s,0,+s]`.
Offset each entire butt stroke; do not move atoms, alter bond order/H or extend endpoints.
The Single branch must preserve its exact current arithmetic, order, masks, hits and golden.

- Keep one logical neighbour vote and hit entry per authored bond, not per drawn segment.
  Store bounded component strokes/polygons with the original axis/endpoints/target; distance
  is the minimum to visible component ink, with existing atom priority and stable-ID ties.
  At zero tolerance a white gap is not fake ink; five-unit tolerance remains a view policy.
- Check every component against H masks/unrelated labels, not just the axis. Each required
  segment must retain visible ink after all label masks; a hidden segment cannot turn a
  Triple into an apparent Double. Reuse separate stacked masks and element attachment rules.
- Preflight actual expanded stroke counts before allocation: hit budget counts logical
  atoms+bonds;4096 primitive budget counts every stroke/mask/glyph. Preserve cancellation,
  numeric/degenerate checks and bounded clipping. Validate all new butt corners against
  physical paper where required; refuse tiny/merged or unplaceable depictions explicitly.
- Native hover must cover the rendered bundle through shared scene geometry, not a second
  independently chosen spacing formula. Preview data/hits grant no new edit or Save authority.

## First independent REDs to execute next

1. Author original V2000 and V3000 raw vectors: C at `(0,0,0)` and `(2,0,0)`, one order2
   or3 bond, no VAL/HCOUNT. Open through existing registry at24pt/unit/origin(72,72).
   Require exact A=`(72,72)`, B=`(120,72)`, authored H=`None,None`, source/version/backing.
   Unchanged external MOL/RDKit consumer must independently report `C=C/C2H4/H[2,2]`
   or `C#C/C2H2/H[1,1]`. Literal-H pages supply a second route, not the source oracle.
2. First actual scene/capture expectation is success with exactly2/3 separate segments;
   current `page.rs` predicts unsupported_content, but no new RED has been executed here.
   With proposed `w=1pt,s=3pt`: Double centerlines y=`70.5,73.5`; Triple=`69,72,75`.
   Check independent axis-normal geometry and pixel-center coverage, not only nonblank PNGs.
3. Geometry unit case in declared canvas units: A=`(20,20)`, B=`(68,84)`, `w=2,s=6`;
   `n=(-4/5,3/5)`, Double offsets±3 and Triple offsets0/±6. Predeclare endpoint tolerance
   `1e-5`canvas units after f32 conversion for this bounded case; exact raw/model data stay exact.
   Reverse endpoints, vertical/horizontal axes, mirrored coordinates and diagonal cases.
4. Require one stable hit from every component, gap/tolerance and atom/label priority checks;
   H-mask/unrelated-label interference, hidden-segment, page edge, tiny scale, degenerate,
   cancellation and expanded-budget failures must never yield partial/misidentified scenes.
5. Actual CPU/GPU full RGBA and analytic strip/gap probes; controls collapse a segment,
   change2↔3, change H or mirror geometry. Keep external Pillow/FreeType/oracles and approved
   Single golden exact. No new image becomes approved without a human decision.
6. Complete-owner Open→capture→exact Save/reopen, history and historical deferred work;
   mixed SDF must retain/display every supported record or refuse the whole sheet. Test actual
   shared CLI/MCP paths and native synthetic frames; later native-window validation is separate.

## Existing refusal tests and remaining questions

Upgrade only now-supported rendering cases, preserving their original input/counts and adding
the stronger geometry/chemistry controls above: `render/tests/{carbon_scene,graph_scene}.rs`,
Double cases in `commands/tests/{mol_display,sdf_capture}.rs`, and any newly affected IO cases.
Preserve unsupported atom/other-order/refusal coverage. An existing Double input with literal
H3 must remain literal; do not replace it with H2 to conceal behavior. If a nonterminal case is
outside the first closed profile, retain its refusal. `commands/tests/extend.rs` Double refusal,
Single-only SMILES path layout and CDX mapping negatives must remain: these authorize editing,
conversion or mapping, not display. No fixture/pin or golden refresh is part of the gate.

Open questions: exact26.1/style-specific numeric spacing; adoption of the original3w policy;
low-resolution separation/refusal threshold; relative spacing against default versus actual
bond length; inheritance and absolute-spacing owner/prose discrepancy; automatic/manual flag
round trips through shared XML names; context-sensitive offset/shortening/joining; persistent
style modelling/versioning for native CDXML. Resolve each explicitly in later slices; never
infer a last-used style, import a proprietary preset or label this plan verified parity.
