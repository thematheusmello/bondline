# Elemental atom-label authoring contract

2026-11-16; research/design only, Root-reported clean base `0f72d16`. No implementation,
new test execution, reference application/assets or parity verification. M0–M5 remain open;
latest full aggregate's unchanged MCP replacement-bind failure is not a passing gate.

## Recommendation

Implement one shared, closed `atom.set_label` operation for an **existing** atom. Retain the
existing model's elemental identity and literal attached-H semantics, existing renderer,
whole snapshots/history and native file workflow. This is real authoring, not conversion.
Start with source-free or byte-proven editable native CDXML. Keep every other backing's
existing refusal until its own synchronized edit contract is deliberately extended.

Root-accepted required parameters for the first named command:
`expected_revision:u64, page_id:u32, fragment_id:u32, atom_id:u32, element:u8,
explicit_hydrogens:u16`. IDs positive, element1..118, H0..65535; runtime validation as
well as closed schema. Register once, `read_only:false`; all adapters discover/call it.
One added name makes33; preserve all32 existing names/schemas/defaults.
No optional inference policy, text alias map, neighbor adjustment or new chemistry crate.

The operation means “author these two literal fields”, **not** “certify a chemically
saturated atom”. Reuse all118 existing element facts; do not invent general periodic-table
valence. H with a positive inline H count refuses, matching the existing physical-H guard.
Other explicit counts remain literal, without synthesized charge/radical or silent fixing.
If the product instead requires saturated-only edits, explicitly choose the existing closed
H/C/N/O/F/Cl/Br/I finite-valence profile as a different policy before tests: it is not a
license to infer118-element chemistry. Existing SMILES export refusals must remain.

## Public behavior and semantics, with boundaries

- GUIDE25 printed90 describes Text-tool editing on an existing atom; printed95 describes
  atom hotkeys; printed38 discusses closing/newlines; printed99 separates automatic label
  alignment, hydrogen correction and document label style. Printed91 also describes label
  repetition. These establish roles, not modern defaults or a generic text implementation.
- Live primary KB44882776753300 says label-entry activation changed at25.5.0.5789 to
  Alt+single-click instead of the older double-click. KB44882820468884 separately describes
  repeating a label with the Text tool (WindowsAlt, MacOption). Do not merge entry/repeat
  semantics or infer their exact tool precedence/Mac entry binding. No26.1 runtime here.
- IUPAC2008 GR2.1/2.1.1 distinguishes proper-case element symbols, inline attached H with
  count subscripts, and separately bonded H atoms. GR2.1.5–2.1.7 concern bond/character
  attachment, reversed labels and clearance. Reuse current depiction; these are not input
  validation, automatic-H preference or fixed pixel defaults.
- OpenSMILES v1 lines190–215 distinguish bracketH0/H1 and physical H;275–309 define separate
  organic-subset inference;671–699 distinguish all three H forms. Never interpret a native
  element field as unbracketed SMILES. The historical text's114-symbol statement does not
  replace the existing IUPAC2022-based118-symbol model catalog.
- BIOVIA2026 printed9–10/65–66 distinguishes query HCOUNT from declared valence and absent
  marking. Neither query HCOUNT nor absent VAL is an alias for native literal H0. No MOL
  source edit or broader exact interchange is authorized merely by adding this command.
- Public archived SDK NumHydrogens is UINT16 and distinguishes inline H from distinct
  nodes; its absent-property inference is a format default, not Bondline model mutation.
  Original complete-source/None handling remains. The mirror is not modern26.1 proof.

## Existing seams and authority

- `model/src/graph.rs:74–99`: Atom already owns ID, element, position, isotope, charge,
  radical and optional literalH. `model/src/element.rs:17–31` resolves exact-case symbols.
  No new model heap field, source representation, identity or element table is needed.
- `render/src/page.rs:359–478`, `render/src/label.rs:83–102`: neutral metadata-only labels
  already cover118 symbols and literal counts; charged/isotopic/radical content and
  physicalH+inlineH refuse. Existing font/attachment/visibility/collision/DPI guards stay.
- `commands/src/bond.rs:131–236` and `commands/src/lib.rs:933–999`: reuse stable target
  lookup, immutable Fragment replacement and one Engine commit; no IDs/floors change.
  Do not use bond.set_order's carbon-H correction to implement an element replacement.
- `io/src/cdxml/write.rs:92–94` already writes literal NumHydrogens; StoredDocument retains
  original backing and model baseline. Save through StoredDocument/FilePlan, never a raw
  regenerated projection or stale source. A changed source-free drawing still has no
  original SourceInfo; changed backed CDXML keeps original source distinct from saved state.
- `io/src/document/smiles.rs:82–140` currently permits only H/order changes: element edits
  must refuse there. Retained SMILES has complete sources/token maps and remains NOT fully
  mapped. Raw SMILES/SDF, opaque CDXML, MOL and CDX cannot gain authority via a label UI.
- `ui/src/lib.rs:797–805/1339–1840` has ordered hit/key replay, namespace invalidation and
  source-specific edit routing. Reuse the actual atom hit (including label ink), not a bond
  midpoint/first-fragment fallback. Source proof/renderability remain separate.

## Required transaction and native increment

1. Validate expected revision before source/target lookup, then source authority and exact
   page/fragment/atom. Target is neutral, natural-isotope, Radical::None; reject unsupported
   target metadata rather than silently removing it. Preserve unrelated metadata verbatim.
2. Candidate changes only target.element and target.explicit_hydrogens. Retain all arrays,
   atom/bond/fragment IDs, directed endpoints, physical H nodes, positions, pages and other
   fragments. Changing H0 is not deleting an atom or creating an H node. No geometry repair.
3. Validate a logical no-op fully, then share current content/source; preserve revision,
   saved baseline, history/redo, file/close/view/tool state and identity floors exactly.
   Changed success is one whole-content history commit; no-op/failure is none. Preserve
   existing accepted-close invalidation policy and hard committed-close refusal.
4. Native first gate: a fresh source-qualified atom-target action opens a bounded editor;
   exact-case Element and Attached hydrogens fields make literal semantics visible. Original
   Apply/Cancel presentation is acceptable only if called an original incremental UI, not
   the reference inline Text tool. Symbol<=2ASCII bytes, H<=5decimal digits; typed core
   scalars remain authoritative. D/T, wildcard, aliases/Me, compound labels, charge/isotope
   syntax and multiline captions are explicit future capabilities, not guessed spellings.
5. Draft edits/paste have no authored effects. Capture owner/revision/generation, IDs,
   close/view/file-context proof; explicit Apply revalidates under the same Engine lock and
   dispatches the registered command once. Stale drafts remain copyable/Apply-disabled,
   Cancel works, and no file/Save/close ACK is fabricated. Do not put atom editing into
   ui.file or create an executable empty-target control. Reuse the existing work path only
   if real deferred work is needed; no extra scheduler/job abstraction for a cheap edit.
6. Keep ordered entry/Apply/Cancel, history and Save interactions: modal/retirement frame
   input cannot author on the newly returned canvas. Source switch/view/close/file ABA
   must not retarget a draft. Save requests cannot publish before an earlier Apply outcome.
   If later backed-SMILES element rewriting is added, it needs its own existing deferred
   source-splice/worker/retirement path, mapping/opaque-byte tests and operation capability.

## Independent acceptance before any implementation

- First JSON-dispatch UnknownCommand RED; closed schema, invalid0/119/negative/overflow,
  missing IDs/fields, wrong kind/backing/stale/committed-close leave whole state exact.
- Original positioned C–C -> C–O/H1, C–N/H2 and C–Cl/H0; separate C=O/H0 and H/H0 cases.
  Assert exact IDs/arrays/endpoints/geometry/untouched H, whole-source baseline and one
  revision/history step. Independently consume actual CDXML with Expat+RDKit graph/H;
  source-free Save/reopen and Undo/Redo must preserve exact authored data. Include another
  neutral symbol such as Si/H0 as literal-format coverage, not saturated-chemistry proof.
- No-op after Undo preserves redo/Arc/close approval; changed edit invalidates it by the
  existing policy. Refused charged/isotopic/radical target preserves every byte/field.
  Unselected fragments and retained opaque imports must not be silently regenerated.
- Actual CLI, initialized stdio and authenticated loopback cover the same typed command,
  Save/Undo/Redo/reopen and unchanged old names; no adapter-only implementation.
- Actual-App/native: fresh atom/label hit activation; exact text/paste values; Apply and
  Cancel; stale/focus/modal/namespace guards; independent OH/NH2/Cl glyph/H controls at
  both source directions with CPU/GPU identity where appropriate. Old golden remains exact.
  Native execution must be recorded separately before claiming delivered desktop behavior.

Unknowns/backlog: exact26.1 activation/tool/Mac precedence, full inline/blur/Escape semantics,
repeat-label behavior, real atom selection/text-tool/caption roles, automatic-H preferences,
chemical warning/valence policy, aliases/contracted labels, element creation, charges,
isotopes/radicals, general source mappings and other platforms. This gate does not finishM2.

## Reviewed public pins

Exact raw responses/cache and current Root files are listed in `public-source-pins.json`
and `root-source-pins.json` beside this note; artifacts are ignored research only.

| Source | URL / inspected portion | Bytes / SHA256 |
|---|---|---|
| GUIDE25 | https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf ; printed38,90–92,95–99,423 | 17721447 / `3f1ebc429df07efdd47b6218a148dc346a215043f60bf89ee9a1cb90629bd5b3` |
| KB label entry | https://support.revvitysignals.com/hc/en-us/articles/44882776753300 ; updated2026-01-03, live public API | 1267 / `94803034630827e2a113a8b9afb68a5fd36d94a6df6315aea0851e78db742290` |
| KB repeat label | https://support.revvitysignals.com/hc/en-us/articles/44882820468884 ; updated2026-01-03, live public API | 1319 / `a59ab5033df8267d0485e688e41a3c56b30113547c6f3abeaead12c3f567d370` |
| OpenSMILES v1 | https://github.com/opensmiles/OpenSMILES/blob/9240a2887158699503605bc838e1de9a9f7883f4/opensmiles.asciidoc ; above exact lines | 86816 / `00f0423da612de913770133aa13e456d0cebc562dd50166534a8a7f9a834bd21` |
| IUPAC2008 | https://iupac.qmul.ac.uk/drawing/drawing.html ; GR2.1/2.1.1/2.1.5–7 | 347564 / `32d2c2d694059e78b6b6a3ed8c274770377e73058d11c174951e7b4e1ce4f71e` |
| BIOVIA2026 | https://www.3ds.com/assets/invest/2026-01/biovia_chemistry_2026_ctfile_formats.pdf ; printed9–10/65–66 | 1580820 / `959f2bbf5487e00fa7535ea7ee36f7ce04f39ea72920844641952276de146c94` |
| SDK NumHydrogens | https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Atom_NumHydrogens.htm ; original Wayback2019 property body148–166 | 15143 / `a89a2f54d3c3daa2de3d02a084c6db4586f85adbd5319306b5cc9835c78d516a` |

Access2026-11-16: both official KB public JSON APIs and pinned OpenSMILES raw commit return200;
OpenSMILES live HTML timed out; SDK Atom_Element.htm returns404 and its web parser cannot
extract the property pages reliably. NumHydrogens raw archived body is readable; no missing
Element page semantics are guessed. No application binary/assets, runtime or build used.
