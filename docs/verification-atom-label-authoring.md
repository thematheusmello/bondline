# Literal elemental atom-label authoring

2026-11-16. The [source contract](research/atom-label-authoring-contract.md) separates
published element/attached-H semantics from original authoring policies and modern
label-entry/repetition evidence. This gate implements the shared command and headless
adapters. **A native atom editor, inline Text tool and complete M2 are not delivered.**

## Delivered boundary

`atom.set_label` is the33rd command, with six required closed scalar parameters:
`expected_revision`, `page_id`, `fragment_id`, `atom_id`, `element` and
`explicit_hydrogens`. IDs are positive; element is atomic number1..118 and attached H is
literal0..65535. H0 is distinct from the old unknown/None value. A physical H atom with
positive inline H refuses. Other counts are literal format data, not a claim of valid
valence, saturation or renderability. No general inference or neighboring-H correction.

Only source-free or completely mapped native CDXML permits this edit. Opaque CDXML,
MOL/CDX, raw/retained SMILES and SDF keep their existing source/refusal boundaries,
including validated no-ops. A charged, isotopic or radical target refuses rather than
losing its metadata. Unrelated atoms, metadata, bonds, directed endpoints, geometry,
fragment/page arrays and stable IDs remain exact. No identity is allocated or new model
heap field introduced. Generic imported-model mutation is never used to bypass backing.

`commands/src/atom.rs` constructs one immutable Fragment replacement, then the existing
Engine commit handles whole-content history, source/saved baselines and close/file
invalidation. Revision is checked before source/target work; the existing global
committed-close refusal retains precedence. A fully validated no-op shares the old
source/model, preserving revision, redo, saved baseline, camera/tool and approved close.
Changed success commits once. Existing FilePlan/Save lease/receipt rules are unchanged;
this command grants no native presentation, publication or Save acknowledgment.

## Independent RED and focused receiving

An independent author freezes the initial1024-byte JSON-dispatch case before product:
SHA256 `93701f3e022f376bd4d46f2a1a113d186876a5a73a46afc29949272c2eefe326`.
Root64681 compiles and reaches intended0/1 `UnknownCommand("atom.set_label")`.
The1189-byte real-CLI declaration is SHA256
`e900e78eaceeb2b49d52122e0fd5c98d5c6595c8b2a8228b17dc4630023072a4`;
Root21623 separately reaches actual CLI `unknown_command`. No new public Rust API is
required merely to run those RED cases. Minimal implementation then passes each1/0
(56348 core,4866 CLI).

Frozen expanded core25685 bytes, SHA256
`3f96dfe29b03dc19106170625da134723a91342bf68d74ad6de4a94ba6f66244`, retains
the exact initial prefix. Root63647 passes all nine without a behavioral correction:

- Sparse/reversed IDs, absent positions and unrelated charged/isotopic/radical atoms
  and Dative bond retain every unaffected field.
- Six-field closed schema, runtime numeric bounds, missing/invalid targets, revision
  precedence and every foreign/opaque/collection backing refuse atomically.
- Unsupported target metadata refuses even a would-be no-op; source/model sharing,
  redo/close approval, changed commits and identity floors follow existing policy.
- Sixty-four fixed-seed label sequences Undo/Redo complete snapshots exactly.
- All118 elements and u16MAX H are literal model/XML coverage, not chemical-validity
  claims. Four actual Unicode CDXML Save/reopen cases independently verify methanol,
  methylamine, methyl chloride and formaldehyde.

The independent author additionally qualifies its reader against four original manual
XML positives and four literal-negative controls before inspecting product files.
Actual pinned Expat/RDKit results are CO/CH4O, CN/CH5N, CCl/CH3Cl and C=O/CH2O.
Wrong element/H/directed bond/charge controls fail raw expectations; they are not claimed
to be invalid XML or RDKit syntax. This qualification was agent-executed; Root owns the
subsequent Cargo receiving. Receipt3910 bytes, SHA256
`4320fc84e0269df94687352edc7da715daf0870e34b3284a0fae3427aa726e8d`.

Expanded wire19067 bytes, SHA256
`84b9655c2f93b211244d59f1555d3176d6f668fc054d4c5cf31184203a34256f`, retains
its first prefix. Root51141 passes all four: actual CLI33-name/closed-schema discovery,
initialized stdio errors/no-op/history/saved baselines, authenticated socket changes/
no-false-repaint and complete SDF/selected-SMILES refusal. Actual native-CDXML source
edits, Undo/Redo, Unicode Save/reopen and unchanged Expat/Decimal plus pinned actual-file
RDKit consumers pass. No adapter-only label implementation or old oracle change.

## Rendering composition control

The separate6247-byte declaration SHA256
`be539591e61d6c3f04010d2681de74cdeb80ce31c9a5922cceaf8797e9b841dc` compares
native C2→O/H1 authoring with the exact original methanol V3000 reference from the
unchanged `element_capture.rs`. Published VAL4/2 arithmetic and that reference's existing
independent RDKit/coordinate test establish C/H3, O/H1 at72/72 and168/72. The native
route never imports; the reference route never calls the setter.

Root48486 passes first: all four actual1024 CPU/GPU PNGs and decoded Pillow RGBA are
equal and nonblank, with actual GPU adapter metadata. Complete state/content/model/
source ownership, camera/generation/file context/history/dirty baselines stay unchanged.
This is a passing reference control, not a new renderer defect or approved golden.
One unused `io::Write` import produces a warning; only that import is removed and
archived invertibly. Corrected6227-byte declaration SHA256
`e9461cca9bfe98a689dad246cffa90bf2af982a85db7e992531d1c0fc1a76c3c`.
All inputs/consumers/assertions remain exact; subsequent differences are rustfmt only.

```sh
cargo test -p bondline-commands --test atom_set_label --locked
cargo test -p bondline-commands --test atom_label_render --locked
cargo test -p bondline-cli --test sdf_capture_adapters atom_label_wire --locked
cargo xtask ci
```

## Aggregate and remaining work

First gate86863 passes1305/0/1 before adding the render composition case,187 result
lines. Log159124 bytes, SHA256
`a91a4d96ce216001ed03399069575a01f02ab52eb15ce4be9b03931009532ebc`.
Final-source gate80684 includes that case and reaches `local CI checks passed`:
1306/0/1 across188 result lines. Log158775 bytes, SHA256
`a6aae9b1b0298d8029e96291b336e1b4661aa92bbe247ede108c0ab752ae13e6`.
Workspace rustfmt, warnings-denied clippy/tests, three macOS vendor tests/vendor clippy,
layers, license closure and corpus pass. The process handle has retired. The earlier825/1
MCP replacement-bind failure remains unexplained; a later pass does not establish a
causal repair. Exhausted20-context diagnostics are not restarted. A distinct pinned-source
descriptor-inheritance hypothesis has a frozen two-arm external OS probe, still uncompiled
and unrun; no production/test socket policy is changed from that hypothesis.

Only eight exact old name sets and22 old catalogue counts add the single command, plus
the two-line wire-test bridge. Native controls remain20 Drawing/14 SDF/10 raw SMILES;
there is no empty-target GUI binding. No model/parser/chem/renderer/vendor/font/golden,
fixture/old-oracle, dependency/lock, corpus pin or fuzz change. Runtime closures remain
155/166/221 plus two assets; Actions stays disabled.

Final protection checks preserve222 pinned inputs except the declared catalogue metadata
and Root's registry integration. All three frozen tests reproduce live sources through
rustfmt alone after the archived unused-import cleanup. The old MCP control remains
byte-exact;582 parity IDs/order,33 live command/protocol rows and375 local Markdown links
pass. The earlier native bundle's size/hash remains exact. Receipt
`target/atom-labels/final-receiving/protection.json`:2756 bytes, SHA256
`b01955f9f15f4ebfbb390face0634d68311cdc515ab328ca02c1203f24dd4291`.

Next is a real native editor with fresh atom/label hits, typed target presentation,
explicit literal fields, Apply/Cancel, copyable stale drafts and ordered Save/close/
modal/namespace guards. It must dispatch this command; do not put an atom target into
`ui.file` or another converter. Exact26.1 Mac activation/tool precedence and broader
inline labels/aliases/automatic H, element creation, charges/isotopes/radicals, selection,
rings/chains/styles and general imported-source edits remain required. The current
desktop preview retains the earlier SMILES feature/hint build; no native label acceptance
is claimed. No milestone or the persistent M0–M5 goal is complete.
