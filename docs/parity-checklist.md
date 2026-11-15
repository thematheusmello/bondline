# Parity checklist

Target: publicly documented Professional-family26.1 desktop behavior.
Reviewed through2026-11-15. Historical authorized installed-profile records below retain
their original method; current research follows the public-source-only boundary in AGENTS.md.
The historical installed license tier remains unknown.
This is a source-cited implementation backlog, not a claim of feature parity.

States: **missing** = no delivered reference workflow; **wired** = code calls an operation
but the stated platform/workflow lacks execution evidence; **verified** = only the stated
scope has independent evidence. A page-only history test does not verify chemical editing.

The detailed inventory below records functional control names and our concise action
summaries. Explicit paths use >; a menu-action label or parenthesized choice records a
functional capability without asserting undocumented submenu nesting. No proprietary help paragraphs, screenshots, icons, templates, sample libraries
or preset values are shipped. Labels naming other products exist only in this reference
record; Bondline must use its own branding and original assets.

Source G25 is the public university-hosted vendor guide in clean-room/sources.md. Its
filename identifies version 25, its cover is unversioned, metadata dates it 2025-05-22,
and the footer is © 2025. Exact reviewed PDF: 17721447 bytes, SHA-256
3f1ebc429df07efdd47b6218a148dc346a215043f60bf89ee9a1cb90629bd5b3.
It has 439 PDF pages. **G25 numbers below are printed pages: printed page p is zero-based
PDF index p, or one-based PDF page p+1.** The cover is index0. The guide stays in the ignored
research cache, not in this repository.

R26.0 and R26.1 are the official May and September 2026 desktop release announcements.
Their desktop additions are recorded separately from cloud-only features. Older guide
labels are not proof that an entire modern menu or platform binding is unchanged.

Inventory limits remain explicit: no full public 26.1 menu reference was located. The
current source does not enumerate every palette glyph or every platform-specific menu.
Dynamic file/style/template lists are controls, not permission to copy their libraries.
The complete modern inventory gate remains open; see clean-room/open-questions.md.
The versioned visual evidence and first connected-drawing acceptance sequence are in
[official-ui-evidence.md](research/official-ui-evidence.md). The pinned engineering review
in [reference-projects.md](research/reference-projects.md) explains which command, gesture,
history, persistence and verification patterns are useful without importing incompatible assets.
The [modern audit](research/modern-inventory-audit.md) adds25.5 gesture/selection and Mac
workspace qualifications plus bounded desktop-video evidence. Its observed Signals menu
differs from older Search labels; exact build/menu contents remain unresolved. It does not
identify a full26.1 desktop screen specification or increase the count by duplicating rows.

The [installed26.1.0.6801 Mac map](research/installed-feature-map.md) now supplements this
backlog with14 menus/503 physical entries,28 Save choices and three inspected Settings
panels. Search and Signals coexist in that exact profile. Bounded original drawing/history/
local save/close/reopen routes have observation evidence; tooltip/shortcut/context/deeper
flow coverage is incomplete. [Implementation tickets](research/installed-implementation-tickets.md)
keep dependencies and missing/wired/verified scopes explicit. Physical entries do not add
to576 logical anchors or establish a feature-completion percentage.

The [36-cell main-palette map](research/installed-tool-palette.md) adds static button,
command and13 flyout declarations matched to a target-window image. Runtime tooltip,
active tool, enabled state and flyout flows remain unobserved; no shipped glyph is copied.

## Workflow overview

| Capability | Source | State | Evidence / next work |
|---|---|---|---|
| New/open/save documents | GUIDE-25 / MAC-FILE-KEYS | partial: Mac CDXML/CDX/MOL/SDF Open/Save verified | Complete-source registry persistence, explicit format choice, native Save As and dirty-close protection; File New/multi-doc/other formats remain missing. |
| Undo/redo | GUIDE-25 / MAC-HISTORY | verified for pages and supported C2/C3/C4 edits, Mac only | Exact engine properties; native page and bond Undo/Redo plus saved-C3/dirty-C4 recovery. Other chemical edits and Windows/Linux runtime missing |
| Selection and transforms | GUIDE-25 | missing | M2; bounded atom/bond hit testing and transient hover exist, not persistent selection or transforms |
| Bond, ring, chain tools | GUIDE-25 | partial: bounded Single/Double/Triple Mac authoring verified | Shared create/extend/set_order, explicit H policies, transient tool state, component geometry and exact complete-source/history/consumer checks. Actual Mac multiple creation, existing-bond click/1/2/3 keys/endpoint Double→Triple and Unicode Save As pass at readable420pt-square scale; native multiple extension and broader keys/styles/selection/rings/chains remain incomplete. See verification-bond-order-{command,ui,native}.md and verification-native-sdf-and-extension.md. |
| Atom labels, charges, isotopes, radicals | GUIDE-25 | partial display; authoring missing | Neutral elemental symbols/literal H and source-aware ordinary MOL OH/NH2 pass headless CPU/GPU/consumer/capture and actual-frame vector/hit checks, preserving source/model None. Native element-label workflows, authoring tools and charged/isotope/radical display remain missing; see verification-neutral-element-labels.md |
| Stereo bonds and labels | GUIDE-25 | missing | M2/M3 |
| Text, captions, chemical formatting | GUIDE-25 | missing | M2/M3 |
| Reaction and curved arrows | GUIDE-25 | missing | M3 |
| Brackets, repeating units and annotations | GUIDE-25 | missing | M3 |
| Document/object styles, journal settings | GUIDE-25 | missing | Public author-guideline sources required |
| Structure check and cleanup | GUIDE-25 | missing | M3; independent chemistry oracles |
| Formula/mass and analysis | GUIDE-25 | missing | M3; pinned conventions |
| Naming and mixtures | PROFESSIONAL-26.1 | missing | M5; public standards/oracles |
| Associated-data label selection | PROFESSIONAL-26.1 | missing | Documented 26.1 behavior |
| Separate CDXML/MDL paste entries | PROFESSIONAL-26.1 | missing | M4; platform clipboard evidence |
| View navigation | GUIDE-25 printed26/74/79 | verified (bounded Mac) | Numeric zoom, Actual/Fit/Magnify/Reduce, pan controls/wheel, unchanged A4 authoring and150% editable persistence. Original scale/step/anchor policies; guards and current-view capture/SDF/physical-DPI limits remain. See verification-view-{render,commands,ui,native}.md. |
| Original SDF field inspector | CTFILE-2026 / original Bondline workflow | verified (bounded Mac/read-only) | Ordered duplicate/empty/spaces/record replacement and exact Unicode Save As/reopen pass natively. Actual-App tests add copy, JSON-budget pagination, omission, source freshness, readonly accessibility/text storage and retirement. Read-only inspection scope; the separate editor is below. No Professional field-screen parity, native large-value/race or full control-addressability claim. See verification-sdf-inspector.md. |
| Original SDF existing-value editor | CTFILE-2026 / original Bondline workflow | verified (bounded Mac); native lifecycle races unverified | Explicit occurrence draft→multiline Apply→Unicode Save As→Undo/Redo→reopen yields exact788 bytes; both molecular records pass unchanged independent consumers. Nine actual-App cases and two real-worker gates cover ordered Save/navigation/close requests, stale recovery, no-op and invalid-input refusal. No new/delete/reorder, Professional field-screen parity, native race, Windows/Linux or full addressability claim. See verification-sdf-editor-integration.md. |
| CDXML/CDX interchange | CDXML-INTRO / CDX-INTRO / CDX-SYNTAX | partial: Mac CDXML and guarded unchanged CDX files verified | CDXML native files retain their existing evidence. CDX now uses complete source-backed open/save through engine/CLI/stdio/socket, independently checked saved bytes/struct/RDKit and lifecycle/refusal tests. Actual Mac CDX choice/Known Save/Unicode Save As/reopen/retry/dirty-close refusal and Undo recovery pass on exact450962c. Faithful CDX preview, imported structural editing, labels/queries/styles, foreign/general corpus and full format parity remain missing; see verification-cdx-application.md |
| MOL/SDF/RXN/SMILES/InChI | GUIDE-25 / CTFILE-2026 | partial: V2000/V3000 limited Mac file workflows verified | Both codecs/owned backing/atomic I/O/shared commands and external consumers; actual Mac version/default/author/Open/Save/Save As/reopen/close/refusal/retry. Ordinary MOL absent-H depiction now has independent raw/CPU/GPU/adapter/Save/history checks and headless initial/post-command canvas/hit acceptance; its native-window workflow remains unverified. V3000 application372+3 local gate passes; own hosted43531603511 GREEN. Mixed V2000/V3000 SDF library has independent/raw/consumer evidence; fetched V2000 corpus remains distinct. A legacy RXN container also has independent roles/raw/consumer evidence; it is not an application reaction workflow. Bounded aliphatic/aromatic SMILES notation/source ownership and explicit-H neutral Fragment export have literal token/graph/output/bounds and qualified external-consumer checks; generated output is noncanonical, with named opt-in fixed-H normalization only, no general inference/perception. Complete accepted SMILES-file source Open/Save now retains every record/opaque byte through commands/CLI/stdio/socket, with independent raw/per-record consumers and bounded container ASan evidence. Explicit record inspection now returns exact selected notation/ranges/counts under a128KiB object budget without disclosing application data or mutating state. Its .smi/.smiles chooser and metadata-only view are wired/headlessly tested; actual native acceptance and chemical preview/editing remain unverified/missing. The restricted create-only conversion command now passes engine/CLI/stdio/authenticated-loopback convert→explicit CDXML Open/edit/history/Save/reopen under the local686 gate. Explicit selected-fragment SMILES file export now has actual engine/CLI/stdio/authenticated-loopback, full-graph/order/metadata and semantic-negative evidence under the local715 gate, with complete-source guards and unchanged active state. Native export/conversion controls and Copy As clipboard behavior remain missing. Native mixed-version SDF Open/Save/reopen and failed-import retention now pass onbb6d5b9; static summary only. Shared bounded SDF field inspection/existing-value edits now pass command/CLI/MCP tests. Restricted all-record neutral/Single SDF capture now passes independent headless image/CLI/MCP tests; the asynchronous native preview is wired/headlessly tested through actual frame/texture instructions; native-window acceptance is unverified. Native chemical display acceptance, record operations, application RXN, broader layout/export, interactive Windows/Linux and SMILES/InChI remain incomplete. M1/M3 |
| SVG/PDF/PNG, print and Office clipboard | GUIDE-25 | missing | M4 |
| Templates and biopolymers/HELM | GUIDE-25 / release notes | missing | Original assets only; M5 |
| Prediction and advanced reference workflows | GUIDE-25 | missing | Full source audit required |

The bounded native SDF additions to MF-002/004/005 are exact mixed777-byte Open,
known Save, Unicode.sd Save As/reopen, explicit misleading-suffix decoder choice,
wrong-suffix/conversion refusals and malformed-second-record retention. Separate690-byte
and extra refusal checks are controller-only. See
[exact native evidence](verification-native-sdf-and-extension.md); these rows do not
establish chemical preview, data-edit UI or other platforms.

## File and application controls

| ID | Reference control/action | G25 p. | State | Notes |
|---|---|---|---|---|
| MF-001 | File > New Document | 13, 39 | missing | Style-dependent New label; project creation/persistence missing. |
| MF-002 | File > Open | 14 | partial verified, Mac CDXML/CDX/MOL/SDF | Shared presentation request/Cancel, decoder choice/hints and failed-open retention. CDX native picker/explicit format/malformed-import retention and guarded no-preview state pass on exact450962c; source is retained. Other formats/platform workflows missing. |
| MF-003 | File > Recent-document list | 14 | missing | Dynamic list; no sample files copied. |
| MF-004 | File > Save | 14 | partial verified, Mac CDXML/CDX/MOL/SDF | Shared known-path/encoding intent and guarded preparation. CDX exact source/receipt/refusal and native known-path misleading-suffix Save/retry/dirty-close refusal/Undo recovery pass. Existing Mac XML/MOL CmdS/QuitSave evidence remains; history-across-save differs G25 and general formats/styles are missing. |
| MF-005 | File > Save As | 39, 59, 356 | partial verified, Mac CDXML/CDX/MOL/SDF | Shared binding and addressable picker; actual CDX choice/Unicode Save As/reopen preserve all four original vectors and one ignored native source, with independent actual-byte consumers. Existing XML/MOL native evidence remains. Unsupported metadata/layout loss refuses; other formats and reference multi-format parity remain missing. |
| MF-006 | File > Revert | 14 | missing | Recover last saved document. |
| MF-007 | File > Close | 78, 259 | partial verified, single-window Mac | Shared document.close through UI/CLI/MCP/socket; File Close/red close/CmdW and Save/Discard/Cancel, epoch/receipt guards and shared-work drain have exact native evidence. Multi-document/platform/reference-dialog parity missing. |
| MF-008 | File > Exit | 78, 290 | partial verified, Mac Quit | Native Quit/CmdQ uses shared close policy; committed edits survive Cancel, Save/Discard exit, remote approval waits for replies/work. Windows/Linux exit gestures, Dock/session logout and immediate unobserved gesture/Q ordering remain unverified. |
| MF-009 | File > Page Setup | 24-25, 78 | missing | Paper size/orientation, margins and printer workflow missing; page-size engine alone is narrower. |
| MF-010 | File > Print | 25, 78 | missing | - |
| MF-011 | File > Document Annotations | 14 | missing | Add/delete keywords and their values. |
| MF-012 | File > Document Properties | 16 | missing | Typed custom properties. |
| MF-013 | File > Document Settings | 19, 22-23, 40, 59 | missing | Detailed settings categories below. |
| MF-014 | File > Preferences | 15, 19, 32-39 | missing | The guide explicitly places this under File. |
| MF-015 | Application menu > Settings (macOS) | 19 | missing | Platform-specific path; do not infer all Mac shortcuts from CTRL tables. |
| MF-016 | File > Apply Document Settings | 40, 49 | missing | Merge settings from a source document. |
| MF-017 | File > Apply Document Settings from > Other | 49 | missing | Alternate label in the same guide. |
| MF-018 | File > Open Style Sheets | 13, 49 | missing | Original independently sourced journal settings only. |
| MF-019 | File > Open Style Sheets > New Document | 428, 438 | missing | Referenced stationery action; no reference stationery file copied. |
| MF-020 | File > Open Templates | 7, 204 | missing | Dynamic catalog; original assets required. |
| MF-021 | File > Open Templates > New Templates | 204 | missing | Author/edit a template palette. |
| MF-022 | File > Open Samples > Hotkeys Cheat Sheet | 67 | missing | Create Bondline's own explanatory material; never copy the sample. |
| MF-023 | File > Open Samples > Shortcuts Cheat Sheet | 77 | missing | Create original material. |
| MF-024 | File > List Nicknames | 97, 207 | missing | Define/remove abbreviations with explicit chemistry; KB-NICKNAME-VISUAL warns its particular B(pin) workaround is visual-only for analysis, not every nickname. |
| MF-025 | File > Run ChemScript | 208-209 | missing | Reference scripting integration; Bondline registry/MCP exists, API parity missing. |
| MF-026 | File > Re-run Previously Selected ChemScript | 208 | missing | - |
| MF-027 | File > ChemScripts | 208-209 | missing | Dynamic script catalog; reference scripts are not copied. |
| MF-028 | File > ChemScripts > Layout Molecules in a Circle | 209 | missing | Independently implement the named layout behavior if supported. |

## Edit, clipboard and insertion controls

| ID | Reference control/action | G25 p. | State | Notes |
|---|---|---|---|---|
| ME-001 | Edit > Undo | 78 | verified (page size and isolated C-C creation, Mac only) | history.undo; exact generated properties and native workflows/error recovery. Other chemical history missing. |
| ME-002 | Edit > Redo | 78 | verified (page size and isolated C-C creation, Mac only) | history.redo; exact generated properties and native workflows. Other chemical history missing. |
| ME-003 | Edit > Undo Autonumber Reaction | 20 | missing | Dynamic history label; autonumbering missing. |
| ME-004 | Edit > Redo Autonumber Reaction | 21 | missing | Dynamic history label; autonumbering missing. |
| ME-005 | Edit > Repeat last command | 78 | missing | Separate from redo. |
| ME-006 | Edit > Repeat Rotate | 119 | missing | - |
| ME-007 | Edit > Cut | 78 | missing | - |
| ME-008 | Edit > Copy | 78 | missing | - |
| ME-009 | Edit > Paste | 78 | missing | - |
| ME-010 | Edit > Clear | 78, 121 | missing | - |
| ME-011 | Edit > Select All | 19-20, 116 | missing | - |
| ME-012 | Edit > Copy As > CDXML Text | 352 | missing | Syntax parser exists; editable clipboard mapping missing. |
| ME-013 | Edit > Copy As > MOL Text | 352 | missing | - |
| ME-014 | Edit > Copy As > MOL V2000 Text | 352 | missing | - |
| ME-015 | Edit > Copy As > SMILES | 350 | missing | - |
| ME-016 | Edit > Copy As > SLN | 351 | missing | - |
| ME-017 | Edit > Copy As > InChI | 351 | missing | - |
| ME-018 | Edit > Copy As > InChIKey | 352 | missing | - |
| ME-019 | Edit > Copy As > HELM | 154 | missing | - |
| ME-020 | Edit > Copy As > HELM (Natural Analog) | 157-158 | missing | - |
| ME-021 | Edit > Copy As > 3MF (Ball and Stick) | 352 | missing | Mesh/3D publishing depth remains visible. |
| ME-022 | Edit > Copy As > 3MF (Stick) | 352 | missing | - |
| ME-023 | Edit > Copy As > PNG | 354 | missing | PNG scene capture exists; selection clipboard workflow missing. |
| ME-024 | Edit > Paste Special > MOL/CDXML Text (legacy) | 353, 78 | missing | Combined guide entry superseded by R26.1 split entries; do not implement it as current parity. |
| ME-025 | Edit > Paste Special > SMILES | 351 | missing | - |
| ME-026 | Edit > Paste Special > InChI | 351 | missing | - |
| ME-027 | Edit > Paste Special > HELM | 150, 155 | missing | Guide also spells the path Paste/Paste Special. |
| ME-028 | Edit > Paste Special > Name as Structure | 218 | missing | - |
| ME-029 | Edit > Paste Special > Biopolymer / FASTA options | 161 | missing | Separate sequence forms below. |
| ME-030 | Edit > Paste Special > FASTA PEPTIDE | 157 | missing | - |
| ME-031 | Edit > Paste Special > FASTA DNA | 158 | missing | - |
| ME-032 | Edit > Insert File | 358 | missing | - |
| ME-033 | Edit > Insert Object | 358 | missing | OLE/file embedding requires public format evidence and independent consumers. |

## View, windows and toolbar controls

| ID | Reference control/action | G25 p. | State | Notes |
|---|---|---|---|---|
| MV-001 | View > Show Main Toolbar | 12, 422 | missing | - |
| MV-002 | View > Show Object Toolbar | 122 | missing | - |
| MV-003 | View > Show Structure Toolbar | 241 | missing | - |
| MV-004 | View > Show BioDraw Toolbar | 129 | missing | KB-BIODRAW corroborates Professional template-palette role, build unknown; exact toolbar/palette naming still needs modern evidence. No library copied. |
| MV-005 | View > Show HELM Monomer Toolbar | 138 | missing | - |
| MV-006 | View > Show Add-ins Toolbar | 391 | missing | Remote/account integrations are optional, not required for offline creation. |
| MV-007 | View > Other Toolbars > Pen Tools | 113 | missing | - |
| MV-008 | View > Other Toolbars > Query Tools | 319, 330, 344 | missing | - |
| MV-009 | View > Other Toolbars > Bracket | 324 | missing | - |
| MV-010 | View > Other Toolbars > Chemical Symbols | 330-331 | missing | - |
| MV-011 | View > Show Ring Fill Colors | 55 | missing | - |
| MV-012 | View > Show Highlight Colors Toolbar | 56-57 | missing | Wrapped path; named toolbar also appears under Colors. |
| MV-013 | View > Templates | 129, 203 | missing | Original catalog assets only. |
| MV-014 | View > Show Preview | 18 | missing | Preview during document opening. |
| MV-015 | View > Magnify | 26, 79 | verified (bounded Mac) | Shared ui.view/F7, actual150→187.5% and own recent drawing focus. Original factor/units; modern defaults remain unknown. |
| MV-016 | View > Reduce | 26, 79 | verified (bounded Mac) | Shared ui.view/F8; actual187.5→150%, transient history/source invariants. |
| MV-017 | View > Actual | 26 | verified (bounded Mac) | Actual/F5 and editable percentage on unchanged A4.100% is original one-logical-unit/point, not physical monitor calibration. |
| MV-018 | View > Show Document | 26 | verified (bounded Mac) | Fit to window/F6 preserves exact legacy geometry; actual menu and76.44% scene pass. Modern label correspondence remains unconfirmed. |
| MV-019 | View > Show Rulers | 26, 79 | missing | - |
| MV-020 | View > Show Crosshair | 27, 79 | missing | - |
| MV-021 | View > Show Info Window | 111-112, 120 | missing | - |
| MV-022 | View > Show Chemical Warnings | 127 | missing | - |
| MV-023 | View > Show Reaction Interpretation | 198, 200 | missing | - |
| MV-024 | View > Show Analysis Window | 222 | missing | - |
| MV-025 | View > Show Chemical Properties Window | 235 | missing | - |
| MV-026 | View > Show Periodic Table Window | 54, 91 | missing | - |
| MV-027 | View > Show Character Map Window | 94 | missing | - |
| MV-028 | View > Show Chem3D Hotlink Window | 239 | missing | Guide says separate installation/SKU is required; no proprietary runtime is bundled. |
| MV-029 | Window > Open-document/template list | 14 | missing | Dynamic active-window selection. |
| MV-030 | Toolbar > Dock Toolbar | 13 | missing | Context control. KB-MAC-DOCK describes Mac application boundary, not drawing-window target; exact26.1/platform persistence unverified. |
| MV-031 | Toolbar > Float Toolbar | 13 | missing | Keep Mac/Windows workspace evidence separate; KB-MAC-DOCK screenshot age/build unknown. |
| MV-032 | Toolbar > Tear off a palette | 12-13 | missing | Pointer gesture; original UI implementation required. |

## Object and layout controls

| ID | Reference control/action | G25 p. | State | Notes |
|---|---|---|---|---|
| MO-001 | Object > Object Settings | 43, 45, 184 | missing | - |
| MO-002 | Object > Apply Object Settings from | 49 | missing | - |
| MO-003 | Object > Annotate | 99 | missing | - |
| MO-004 | Object > Attach Data | 358 | missing | Data-label associations added in R26.1. |
| MO-005 | Object > Fixed Lengths | 89 | missing | Geometry constraint toggle. |
| MO-006 | Object > Fixed Angles | 89 | missing | - |
| MO-007 | Object > Bring to Front | 89, 78-79 | missing | - |
| MO-008 | Object > Send to Back | 89, 79 | missing | - |
| MO-009 | Object > Rotate | 118-119 | missing | Numeric rotation dialog. |
| MO-010 | Object > Rotate 180 degrees Horizontal | 79 | missing | Preserve chemical meaning according to documented behavior; independent stereo evidence required. |
| MO-011 | Object > Rotate 180 degrees Vertical | 80, 437 | missing | - |
| MO-012 | Object > Flip Horizontal | 121, 437 | missing | Mirror operation distinct from rotation. |
| MO-013 | Object > Flip Vertical | 121 | missing | - |
| MO-014 | Object > Flatten | 192 | missing | - |
| MO-015 | Object > Scale | 124 | missing | Numeric scale dialog. |
| MO-016 | Object > Center on Page | 124 | missing | - |
| MO-017 | Object > Group | 110, 123 | missing | - |
| MO-018 | Object > Ungroup | 123 | missing | - |
| MO-019 | Object > Join | 122 | missing | Topological edit, not a display-only grouping. |
| MO-020 | Object > Add Frame (rectangle) | 113 | missing | - |
| MO-021 | Object > Add Frame (brackets) | 113 | missing | - |
| MO-022 | Object > Add Frame (parentheses) | 113 | missing | - |
| MO-023 | Object > Add Frame (braces) | 113 | missing | - |
| MO-024 | Object > Align > Left edges | 81, 125 | missing | KB-ALIGN-25.5 requires whole structures, not partial chemical selections;26.1/mixed objects unobserved. |
| MO-025 | Object > Align > Centers | 81, 125 | missing | Same version-qualified whole-structure enablement as MO-024. |
| MO-026 | Object > Align > Right edges | 81, 125 | missing | Same version-qualified whole-structure enablement as MO-024. |
| MO-027 | Object > Align > Top edges | 81, 125 | missing | Same version-qualified whole-structure enablement as MO-024. |
| MO-028 | Object > Align > Middle (top/bottom) | 81, 125 | missing | Guide describes top/bottom center; whole-structure qualification as MO-024. |
| MO-029 | Object > Align > Bottom edges | 81, 125 | missing | Same version-qualified whole-structure enablement as MO-024. |
| MO-030 | Object > Distribute > Horizontally | 81, 125 | missing | KB-DISTRIBUTE corroborates shortcut/menu/context alternatives; exact context path/build still unknown. |
| MO-031 | Object > Distribute > Vertically | 81, 125 | missing | - |
| MO-032 | Object > Show Stereochemistry | 437 | missing | Source path from tutorial; naming differs from other stereo controls. |
| MO-033 | Group context > Integral | 129 | missing | BioDraw decomposition. |
| MO-034 | Group context > Ungroup | 123, 129 | missing | - |

## Structure and chemical query controls

| ID | Reference control/action | G25 p. | State | Notes |
|---|---|---|---|---|
| MS-001 | Structure > Clean Up Structure | 126, 81 | missing | - |
| MS-002 | Structure > Check Structure | 126-127 | missing | - |
| MS-003 | Structure > Clean Up Reaction | 196 | missing | - |
| MS-004 | Structure > 3D Clean Up | 241 | missing | - |
| MS-005 | Structure > Structure Perspective | 241-243 | missing | - |
| MS-006 | Structure > Toggle Aromatic Display | 103 | missing | - |
| MS-007 | Structure > Toggle Hydrogen Display | 314 | missing | - |
| MS-008 | Structure > Expand Label | 95, 98, 161, 186-188 | missing | - |
| MS-009 | Structure > Contract Label | 98, 162, 185 | missing | - |
| MS-010 | Structure > Define Nickname | 206 | missing | - |
| MS-011 | Structure > Add Multi-Center Attachment | 189 | missing | - |
| MS-012 | Structure > Add Variable Attachment | 190 | missing | - |
| MS-013 | Structure > Autonumber Reaction | 19-20 | missing | - |
| MS-014 | Structure > Clear Reaction Numbers | 21 | missing | Wrapped menu path; guide prose names the action. |
| MS-015 | Structure > Analyze Stoichiometry | 200 | missing | - |
| MS-016 | Structure > Convert Structure to Name | 210 | missing | - |
| MS-017 | Structure > Convert Name to Structure | 216, 218-219 | missing | - |
| MS-018 | Structure > Add Structure to Dictionary | 219 | missing | - |
| MS-019 | Structure > Enhanced Stereochemistry > Mark as Absolute | 36, 226-227 | missing | - |
| MS-020 | Structure > Enhanced Stereochemistry (AND marker) | 226-227 | missing | Group identity/numbering is part of the behavior. |
| MS-021 | Structure > Enhanced Stereochemistry (OR marker) | 226-227 | missing | - |
| MS-022 | Structure > Atom Properties | 309-311 | missing | Query/atom property dialog, not just element selection. |
| MS-023 | Structure > Bond Properties | 320-321 | missing | - |
| MS-024 | Structure > Properties (Atom Properties tab) | 310 | missing | Alternate path in guide; modern label unresolved. |
| MS-025 | Structure > Bracket Properties | 324 | missing | - |
| MS-026 | Structure > Expand Generic Structure | 337 | missing | - |
| MS-027 | Structure > Expand Generic Structure to SDF | 340 | missing | - |
| MS-028 | Structure > R-Logic Query | 338 | missing | - |
| MS-029 | Structure > Map Reaction Atoms | 341-342 | missing | - |
| MS-030 | Structure > Clear Reaction Map | 343 | missing | - |
| MS-031 | Structure > Add 3D Property | 344 | missing | Constraint alternatives below. |
| MS-032 | Structure > Predict 1H-NMR Shifts | 250-251 | missing | Independent prediction/oracle research required. |
| MS-033 | Structure > Predict 13C-NMR Shifts | 250 | missing | - |
| MS-034 | Structure > Make Spectrum-Structure Assignment | 251 | missing | - |
| MS-035 | Structure > Clear Spectrum-Structure | 252 | missing | - |

## Text, curves, colors, help and external commands

| ID | Reference control/action | G25 p. | State | Notes |
|---|---|---|---|---|
| MX-001 | Text > Font | 48 | missing | No font files are copied; shipped fonts retain the strict asset policy. |
| MX-002 | Text > Size | 48 | missing | - |
| MX-003 | Text > Style | 45, 48 | missing | - |
| MX-004 | Text menu action: Flush Left | 82 | missing | - |
| MX-005 | Text menu action: Center | 82 | missing | - |
| MX-006 | Text menu action: Flush Right | 82 | missing | - |
| MX-007 | Text menu action: Justified | 82 | missing | - |
| MX-008 | Text menu action: Automatic justification | 82 | missing | - |
| MX-009 | Text menu action: Plain | 82 | missing | - |
| MX-010 | Text menu action: Bold | 82 | missing | - |
| MX-011 | Text menu action: Italic | 82 | missing | - |
| MX-012 | Text menu action: Underline | 82 | missing | - |
| MX-013 | Text menu action: Formula | 82 | missing | - |
| MX-014 | Curves > Closed | 115 | missing | - |
| MX-015 | Curves > Filled | 115 | missing | - |
| MX-016 | Curves > Faded | 115 | missing | - |
| MX-017 | Curves > Shaded | 115 | missing | - |
| MX-018 | Color > By Element | 54 | missing | - |
| MX-019 | Colors > Show Highlight Colors Toolbar | 56 | missing | Guide uses both Color and Colors as root labels. |
| MX-020 | Color > Select a color | 109-110, 184 | missing | Dynamic palette/color editor; reference color assets are not copied. |
| MX-021 | Help > Contents | 7 | missing | - |
| MX-022 | Help > Contact Support | 7-8, 417-418 | missing | Bondline needs its own support/help destination. |
| MX-023 | Help > Check for Updates | 9 | missing | Offline creation must remain independent of updates. |
| MX-024 | Help > Hotkeys Cheat Sheet | 67 | missing | Original documentation, not the reference sample. |
| MX-025 | Help > Shortcuts Cheat Sheet | 77 | missing | Original documentation. |
| MX-026 | Help > Add-in Developer Guide | 380 | missing | Public API research, not proprietary implementation copying. |
| MX-027 | Search > Browse Revvity Documentation | 417-418 | missing | Reference web destination; outside required offline editing. |
| MX-028 | Search > Browse Revvity Signals Software Store | 417-418 | missing | Reference commerce destination; outside product scope. |
| MX-029 | Search > Browse Revvity.com | 417-418 | missing | Reference marketing destination; outside product scope. |
| MX-030 | Search > Search Reaxys | 418 | missing | External integration; preserve backlog without requiring an account. |
| MX-031 | Help > Getting Started | KB-HELP-ACCESS | missing, version unconfirmed | Separate installed-guide navigation role;26.1 presence/destination/platform need evidence. Original Bondline help required. |
| MX-032 | Help > Tutorials | KB-HELP-ACCESS | missing, version unconfirmed | Separate role, not permission to copy installed tutorials. Exact26.1 presence unproven. |
| MX-033 | Help > ChemDraw Basics | KB-HELP-ACCESS | missing, version unconfirmed | Reference label only; use original Bondline branding/help. Exact26.1 presence unproven. |

## Drawing and chemistry tools

| ID | Reference control/action | G25 p. | State | Notes |
|---|---|---|---|---|
| TD-001 | Lasso | 115-116 | missing | Freehand whole-object selection. |
| TD-002 | Marquee | 115-116 | missing | Rectangular selection. |
| TD-003 | Structure Perspective | 192, 242-243 | missing | Structure rotation/projection handling. |
| TD-004 | Eraser | 86, 91, 121 | missing | Deletion and bond-order reduction have distinct contexts. |
| TD-005 | Text | 91-94 | missing | KB-LABEL-25.5 changes existing-label entry to Alt-click; separate KB-REPEAT-25.5 gives Windows Alt/Mac Option-click repetition. Tool context/version/source-target semantics require tests; no26.1 inference. |
| TD-006 | Solid/single bond | 85–86 | verified (partial, Mac) | Existing native C3/C4 plus new Single creation, click Single→Double and endpoint Double→Triple, exact H/IDs/history/CDXML Save As at readable scale. Also headless ordered/frame tests; modifiers/snapping/general imported editing/alignment/full tool parity remain missing. See verification-bond-order-native.md. |
| TD-007 | Double bond | 85,88 | verified (partial, Mac) | Actual native tool selection/Double creation/Undo/Unicode CDXML Save As, shared typed order-aware create/extend/set_order and independent consumer/frame/image tests. Multiple terminal extension remains headless-only; reference sidedness/alignment/styles and complete tool flows remain missing. See verification-bond-order-ui.md and verification-bond-order-native.md. |
| TD-008 | Double either/crossed double bond | 86 | missing | - |
| TD-009 | Triple bond | 86,88 | verified (partial, Mac) | Native tool selection/Triple creation, hotspot3 and endpoint Double→Triple, exact source-free history/Save As and independent graph/file evidence at readable scale. Multiple terminal extension, context-sensitive styles and complete old-order×tool behavior remain missing/unverified. |
| TD-010 | Quadruple bond | 86 | missing | Guide's legacy 4-key example conflicts with newer bond-hotkey table. |
| TD-011 | Bold bond | 86, 73 | missing | - |
| TD-012 | Dashed bond | 86-88, 72 | missing | - |
| TD-013 | Dashed double bond | 73, 88 | missing | - |
| TD-014 | Bold double bond | 73, 88 | missing | - |
| TD-015 | Wavy bond | 86, 73 | missing | - |
| TD-016 | Wedged bond | 87, 73 | missing | - |
| TD-017 | Hashed bond | 73 | missing | - |
| TD-018 | Hashed-wedged bond | 87, 73 | missing | - |
| TD-019 | Dative bond | 86 | missing | - |
| TD-020 | Hydrogen bond (dotted) | 87 | missing | Requires explicit donor hydrogen; differs from ordinary dashed/coordination bonds. |
| TD-021 | Aromatic/tautomeric double-bond display | 88 | missing | Separate visual/chemical conventions need tests. |
| TD-022 | Acyclic chain | 104-105 | missing | - |
| TD-023 | Snaking chain | 104-105 | missing | - |
| TD-024 | Benzene ring | 74, 101-103 | missing | - |
| TD-025 | Cyclopentadiene ring | 74 | missing | - |
| TD-026 | Ring sizes 3-8 | 101-103, 73 | missing | Individual standard sizes are supported reference choices; fresh geometry required. |
| TD-027 | Cyclohexane chair, both orientations | 102, 73 | missing | - |
| TD-028 | Arrows palette | 106-109 | missing | Unnamed glyph variants still need independent semantic identification. |
| TD-029 | Hollow/block arrow | 107, 109-110 | missing | - |
| TD-030 | Equilibrium arrow | 109 | missing | - |
| TD-031 | Curved arrow | 109 | missing | - |
| TD-032 | Synthesis | 193 | missing | - |
| TD-033 | Retrosynthesis | 193 | missing | - |
| TD-034 | Electron pushing | 199-200, 238 | missing | - |
| TD-035 | Radical pushing | 199-200, 238 | missing | - |
| TD-036 | Mass fragmentation | 192-193 | missing | Actual bond splitting, fragment formula and mass, not decorative marks. |
| TD-037 | Rectangle | 110 | missing | - |
| TD-038 | Circle | 110 | missing | - |
| TD-039 | Oval | 110 | missing | - |
| TD-040 | Drawing line | 110-111 | missing | Excluded from chemical graph interpretation. |
| TD-041 | Solid arc: 90, 120, 180, 270 degrees | 111 | missing | - |
| TD-042 | Dashed arc: 90, 120, 180, 270 degrees | 111 | missing | - |
| TD-043 | Single bracket/brace/parenthesis | 112 | missing | - |
| TD-044 | Paired bracket/brace/parenthesis | 112-113 | missing | Repeat count/range and molecular-weight override. |
| TD-045 | Dagger | 113 | missing | - |
| TD-046 | Draw Curve | 113-115 | missing | - |
| TD-047 | Edit Curve | 113-115 | missing | - |
| TD-048 | Table | 28-31 | missing | Cells, captions, sizing and structure placement. |
| TD-049 | Templates palette | 203-205 | missing | Original independently authored assets. |
| TD-050 | Periodic table atom insertion | 91 | missing | - |
| TD-051 | Character map | 94 | missing | - |
| TD-052 | Expand Label toolbar action | 186-188 | missing | - |
| TD-053 | Name to Structure toolbar action | 216 | missing | - |
| TD-054 | Structure to Name toolbar action | 210 | missing | - |
| TD-055 | Reaction Atom-Atom Map | 341-343 | missing | Manual/automatic mapping and removal. |
| TD-056 | TLC plate | 243-245 | missing | - |
| TD-057 | TLC spot | 244-245 | missing | Spot shape, lane and Rf behavior. |
| TD-058 | Gel electrophoresis | 245 | missing | - |

## Chemical symbols, orbitals and query tools

| ID | Reference control/action | G25 p. | State | Notes |
|---|---|---|---|---|
| TQ-001 | Circle plus | 237 | missing | - |
| TQ-002 | Plus | 237-238 | missing | - |
| TQ-003 | Circle minus | 237 | missing | - |
| TQ-004 | Minus | 237-238 | missing | - |
| TQ-005 | Radical cation | 237-238 | missing | - |
| TQ-006 | Radical anion | 237-238 | missing | - |
| TQ-007 | Radical | 237-238 | missing | - |
| TQ-008 | Lone pair/diradical | 237-238 | missing | Chemical interpretation depends on placement. |
| TQ-009 | Lone-pair bar | 238 | missing | Alt-click variant. |
| TQ-010 | H-dot | 237 | missing | - |
| TQ-011 | H-dash | 237 | missing | - |
| TQ-012 | Attachment point (diamond) | 330, 237-239 | missing | Ranked variable attachment. |
| TQ-013 | Polymer bead attachment | 237-239 | missing | Independent geometry; no source icons copied. |
| TQ-014 | Additional attachment-point glyphs | 237 | missing | Guide gives several glyphs the same label; exact variants remain an open question. |
| TQ-015 | s orbital | 236 | missing | - |
| TQ-016 | sigma orbital | 236 | missing | - |
| TQ-017 | Single-lobe orbital | 236 | missing | - |
| TQ-018 | p orbital | 236 | missing | - |
| TQ-019 | Hybrid/sp3 orbital | 236 | missing | - |
| TQ-020 | dxy orbital | 237 | missing | - |
| TQ-021 | dz2 orbital | 237 | missing | - |
| TQ-022 | Free Site | 319 | missing | Alt-click decreases query free-site count. |
| TQ-023 | Alternative Group | 330 | missing | - |
| TQ-024 | Add 3D Property > Angle | 344 | missing | Angle constraint; ordering and interval semantics require tests. |
| TQ-025 | Add 3D Property > Distance | 344 | missing | Constraint interval and supported geometry combinations require tests. |
| TQ-026 | Add 3D Property > Dihedral | 344 | missing | Constraint interval and supported geometry combinations require tests. |
| TQ-027 | Add 3D Property > Line | 344 | missing | Best-fit query geometry. |
| TQ-028 | Add 3D Property > Plane | 344 | missing | Best-fit query geometry. |
| TQ-029 | Add 3D Property > Exclusion Sphere | 344 | missing | Ordered selection and radius interval. |
| TQ-030 | Add 3D Property > Normal | 344 | missing | Plane and direction point. |
| TQ-031 | Add 3D Property > Point | 344 | missing | Distance/direction construction. |
| TQ-032 | Racemic stereo flag | 344 | missing | Query palette; distinct from enhanced-stereo groups. |
| TQ-033 | Absolute stereo flag | 344 | missing | Query palette. |
| TQ-034 | Relative stereo flag | 344 | missing | Query palette. |

## BioDraw and HELM authoring controls

| ID | Reference control/action | G25 p. | State | Notes |
|---|---|---|---|---|
| TB-001 | Receptor | 132 | missing | - |
| TB-002 | G-protein gamma subunit | 132 | missing | - |
| TB-003 | Helix Protein | 132-133 | missing | - |
| TB-004 | DNA | 132, 134 | missing | - |
| TB-005 | Membrane line | 132-133 | missing | - |
| TB-006 | Membrane arc | 132-133 | missing | - |
| TB-007 | Membrane ellipse | 132 | missing | - |
| TB-008 | Micelle | 132 | missing | - |
| TB-009 | Plasmid map | 132, 136-137 | missing | - |
| TB-010 | Ribosome | 132, 135 | missing | - |
| TB-011 | tRNA | 132, 135 | missing | - |
| TB-012 | Ion channel | 132 | missing | - |
| TB-013 | G-protein alpha subunit | 132 | missing | - |
| TB-014 | G-protein beta subunit | 132 | missing | - |
| TB-015 | Immunoglobulin | 132 | missing | - |
| TB-016 | Golgi body | 132 | missing | - |
| TB-017 | Endoplasmic reticulum | 132-133 | missing | - |
| TB-018 | Mitochondrion | 132 | missing | - |
| TB-019 | Cloud shape | 133 | missing | - |
| TB-020 | BioDraw template categories | 129-131 | missing | Anatomy, animals, instruments, microorganisms and organelles need original replacement artwork. |
| TB-021 | HELM > Polymer Type > AA | 138-139 | missing | Alpha/beta and L/D amino-acid groups. |
| TB-022 | HELM > Polymer Type > RNA | 139-140 | missing | Base, phosphate and ribose/deoxyribose filters. |
| TB-023 | HELM > Polymer Type > CHEM | 140 | missing | - |
| TB-024 | HELM > Polymer Type > BLOB | 140 | missing | - |
| TB-025 | HELM > Replace Current Monomer | 138, 140 | missing | - |
| TB-026 | HELM > Data Source Type | 138, 140 | missing | Library identity and document/user ownership; source monomer libraries are not copied. |
| TB-027 | HELM > Manage Monomers | 140 | missing | - |
| TB-028 | HELM > Open monomer editor | 138 | missing | - |
| TB-029 | HELM > Spotlight search | 138, 140 | missing | - |
| TB-030 | HELM > Monomer filter buttons | 138-140 | missing | - |
| TB-031 | HELM > Sequence drawing/editing | 143-152, 161-167 | missing | Expand/contract, graph/label rendering, selection and chain edits. |
| TB-032 | HELM > Shared monomers/library management | 177-182 | missing | Offline document/user libraries required; hosted sharing optional. |

## Document dialogs and settings

| ID | Reference control/action | G25 p. | State | Notes |
|---|---|---|---|---|
| DP-001 | Document Annotations dialog | 14 | missing | Add/delete keyword, content, delete all. |
| DP-002 | Document Properties dialog | 16-18 | missing | Document-level property definitions and typed values. |
| DP-003 | Preferences > General | 32-33, 38 | missing | Theme, units/tolerance and general behavior. |
| DP-004 | Preferences > Open/Save | 33, 38-39 | missing | File format, default tool, return-key and default style behavior. |
| DP-005 | Preferences > Directories | 33, 38-39 | missing | Search paths and default document location; no app install inspection. |
| DP-006 | Preferences > Building/Display | 99, 102, 104 | missing | Geometry, label positioning and chain/ring behavior. |
| DP-007 | Preferences > Warnings | 87, 127-128 | missing | Explicit validation/warning policy. |
| DP-008 | Preferences > Monomer Libraries | 178-182 | missing | User/document/library state and versions. |
| DP-009 | Document Settings > Layout | 22-23 | missing | Pages, poster dimensions/overlap and page registration marks. |
| DP-010 | Document Settings > Header/Footer | 23 | missing | Header/footer variables and alignment. |
| DP-011 | Document Settings > Drawing | 40-43 | missing | Bond geometry/style settings; values need independent sources. |
| DP-012 | Document Settings > Atom Labels | 44, 99 | missing | - |
| DP-013 | Document Settings > Captions | 44-48 | missing | - |
| DP-014 | Document Settings > Analysis | 44 | missing | Analysis field labels and formatting. |
| DP-015 | Document Settings > Colors | 54 | missing | - |
| DP-016 | Document Settings > Reaction Display | 19 | missing | Numbering and placement. |
| DP-017 | Document Settings > Biopolymer Display | 49, 164 | missing | Wrapping and authored sequence display. |
| DP-018 | Object Settings dialog | 43, 45 | missing | Supported overrides and style inheritance. |
| DP-019 | Rotate Objects dialog | 118-119, 80 | missing | - |
| DP-020 | Scale Objects dialog | 124, 81 | missing | - |
| DP-021 | Contract Label dialog | 185 | missing | - |
| DP-022 | Choose Nickname dialog | 72 | missing | - |
| DP-023 | Atom Properties dialog | 309-319 | missing | Query constraints and element/isotope/radical fields. |
| DP-024 | Bond Properties dialog | 320-321 | missing | Query/stereo/topology properties. |
| DP-025 | Bracket Properties dialog | 324-328 | missing | Polymer types, connectivity and labels. |
| DP-026 | Define Average Molecular Weight (bracket context) | 113 | missing | Explicit nonnegative override and derived calculations. |
| DP-027 | R-Logic Query dialog | 338-339 | missing | - |
| DP-028 | Analysis window | 221-224 | missing | Formula, mass, composition and supported derived fields. |
| DP-029 | Chemical Properties window/report | 228-235 | missing | Predictions must have documented method/limits and independent evidence. |
| DP-030 | Stoichiometry grid | 200-202 | missing | Reactant/product role, equivalents and typed values. |
| DP-031 | Add-in Manager | 380-382 | missing | Developer/optional integration interface; offline core independent. |

## Atom-hotspot keys (G25 default table)

| ID | Contextual action / key(s) | G25 p. | State | Scope / evidence |
|---|---|---|---|---|
| KA-001 | Label F: f | 67 | missing | - |
| KA-002 | Label CF3: Shift+f | 68 | missing | - |
| KA-003 | Label P: p | 68 | missing | - |
| KA-004 | Label Ph: Shift+p | 68 | missing | - |
| KA-005 | Label Ac: Shift+a | 68 | missing | - |
| KA-006 | Label H: h | 68 | missing | - |
| KA-007 | Label Br: b | 68 | missing | - |
| KA-008 | Label B: Shift+b | 68 | missing | - |
| KA-009 | Label I: i | 68 | missing | - |
| KA-010 | Label R: r | 68 | missing | - |
| KA-011 | Label K: k | 68 | missing | Conflicts with sulfonyl k at p.69; unresolved. |
| KA-012 | Label S: s | 68 | missing | - |
| KA-013 | Label Si: Shift+s | 68 | missing | - |
| KA-014 | Label Me: m | 68 | missing | - |
| KA-015 | Label N: n or w | 68 | missing | - |
| KA-016 | Label NO2: Shift+n | 68 | missing | - |
| KA-017 | Label C: c | 68 | missing | - |
| KA-018 | Label Cl: l or Shift+c | 68 | missing | - |
| KA-019 | Label X: x | 68 | missing | - |
| KA-020 | Label O: o or q | 68 | missing | - |
| KA-021 | Label D: d | 68 | missing | - |
| KA-022 | Label Et: e | 68 | missing | - |
| KA-023 | Label CO2Me: Shift+e | 68 | missing | - |
| KA-024 | Label N3: Shift+z | 68 | missing | - |
| KA-025 | Label MgBr: Shift+m | 69 | missing | - |
| KA-026 | Label Li: Shift+l | 69 | missing | - |
| KA-027 | Label OMe: Shift+o | 69 | missing | - |
| KA-028 | Label Fmoc: Shift+q | 69 | missing | - |
| KA-029 | Label Cbz: Shift+h | 69 | missing | - |
| KA-030 | Label Boc: Shift+y | 69 | missing | - |
| KA-031 | Sprout cyclopropane: v | 69 | missing | - |
| KA-032 | Sprout cyclobutane: u | 69 | missing | - |
| KA-033 | Sprout sulfonyl: k | 69 | missing | Conflicts with label K at p.68; unresolved. |
| KA-034 | Sprout t-Bu with right-angle geometry: Shift+k | 69 | missing | - |
| KA-035 | Add attachment point: period | 69 | missing | - |
| KA-036 | Add atom number: single quote | 69 | missing | - |
| KA-037 | Add negative charge: - | 69 | missing | - |
| KA-038 | Add positive charge: + | 69 | missing | - |
| KA-039 | Sprout cyclic-mode single bond: 0 | 69 | missing | Primary versus secondary/tertiary cases differ. |
| KA-040 | Sprout linear-mode single bond: 1 | 69 | missing | - |
| KA-041 | Sprout carbonyl/acetyl: 2 | 69-70 | missing | Degree/aromatic context matters. |
| KA-042 | Sprout phenyl: 3 or a | 70 | missing | Degree/aromatic context matters. |
| KA-043 | Sprout wedge: 4 | 70 | missing | Degree/aromatic context matters. |
| KA-044 | Sprout hashed wedge: 5 | 70 | missing | Degree/aromatic context matters. |
| KA-045 | Sprout cyclohexane: 6 | 70-71 | missing | Includes spiro/extended cases. |
| KA-046 | Sprout cyclopentane: 7 | 71 | missing | Includes spiro/extended cases. |
| KA-047 | Sprout methylidene: 8 | 71 | missing | Guide documents an intentionally incomplete-valence intermediate. |
| KA-048 | Sprout dimethyl/isopropyl: 9 | 71 | missing | Atom-degree dependent. |
| KA-049 | Sprout alkyne: z | 71 | missing | - |
| KA-050 | Cyclopentadienyl ligand: j | 71-72 | missing | Isolated-metal versus terminal-atom cases. |
| KA-051 | Phenyl pi ligand: Shift+j | 72 | missing | Isolated-metal versus terminal-atom cases. |
| KA-052 | Atom Properties: / or ? | 72 | missing | - |
| KA-053 | Select atom hotspot: g | 72 | missing | - |
| KA-054 | Choose Nickname: = | 72 | missing | - |
| KA-055 | Edit atom label: Enter | 72 | missing | - |
| KA-056 | Remove atom label: Backspace or Delete | 72 | missing | Label removal differs from deleting an atom. |

## Bond-hotspot keys (G25 default table)

| ID | Contextual action / key(s) | G25 p. | State | Scope / evidence |
|---|---|---|---|---|
| KB-001 | Single bond: 1 | 72 | verified (partial carbon hotspot, Mac) | Absolute existing-edge setter, required bounded H policy and fresh per-event target. Actual native stationary3→1 and Undo/Redo/stale-target checks; broader atom/source/tool contexts remain missing. See verification-bond-order-{ui,native}.md. |
| KB-002 | Dashed bond: d | 72 | missing | - |
| KB-003 | Double bond: 2 | 73 | verified (partial carbon hotspot, Mac) | Absolute existing-edge setter passes full graph/history/guards and ordered app frames. Positive native2 changes a reopened native-CDXML Single toDouble with exact H2, IDs/positions, known Save and menu Undo/Redo; stale-key refusal also observed. See verification-bond-order-native.md. |
| KB-004 | Dashed double bond: Shift+d | 73 | missing | - |
| KB-005 | Bold double bond: Shift+b | 73 | missing | - |
| KB-006 | Wavy bond: y | 73 | missing | - |
| KB-007 | Triple bond: 3 | 73 | verified (partial carbon hotspot, Mac) | Absolute existing-edge setter; native3 edit and1 restore at readable scale, existing edge/H preserved. Pixel-readability guard remains, not universal tool/style behavior. See verification-bond-order-native.md. |
| KB-008 | Double bond left: l | 73 | missing | - |
| KB-009 | Double bond centered: c | 73 | missing | - |
| KB-010 | Bold bond: b | 73 | missing | - |
| KB-011 | Double bond right: r | 73 | missing | - |
| KB-012 | Wedged bond: w | 73 | missing | - |
| KB-013 | Bring bond to front: f | 73 | missing | - |
| KB-014 | Hashed bond: Shift+h | 73 | missing | - |
| KB-015 | Bond Properties: / or ? | 73 | missing | - |
| KB-016 | Hashed-wedged bond: h | 73 | missing | - |
| KB-017 | Select bond hotspot: g | 73 | missing | - |
| KB-018 | Fuse benzene: a | 73 | missing | - |
| KB-019 | Fuse cyclopentadiene: z | 73 | missing | - |
| KB-020 | Fuse 4-membered ring: 4 | 73 | missing | Conflicts with legacy quadruple-bond example at p.86. |
| KB-021 | Fuse 5-membered ring: 5 | 73 | missing | - |
| KB-022 | Fuse 6-membered ring: 6 | 73 | missing | - |
| KB-023 | Fuse 7-membered ring: 7 | 73 | missing | - |
| KB-024 | Fuse 8-membered ring: 8 | 73 | missing | - |
| KB-025 | Cyclohexane chair orientations: 9 or 0 | 73 | missing | - |
| KB-026 | Example-only bond key: v | 74 | missing | Appears in a figure example but has no row in the bond-key table; behavior unresolved. |

## Generic and HELM keys

| ID | Contextual action / key(s) | G25 p. | State | Scope / evidence |
|---|---|---|---|---|
| KG-001 | Marquee: Spacebar | 74 | missing | No atom/bond hotspot; selection-toggle behavior also described at p.116. |
| KG-002 | Solid bond tool: x | 74 | missing | - |
| KG-003 | Acyclic chain tool: Shift+x | 74 | missing | - |
| KG-004 | Text tool: t | 74 | missing | - |
| KG-005 | Bracket tools: Shift+t | 74 | missing | - |
| KG-006 | Arrow tools: e | 74 | missing | - |
| KG-007 | Chemical Symbols tools: Shift+e | 74 | missing | - |
| KG-008 | Benzene tool: j | 74 | missing | - |
| KG-009 | Cyclopentadiene tool: Shift+j | 74 | missing | - |
| KG-010 | Chromatography tools: g | 74 | missing | - |
| KG-011 | Orbital tools: Shift+g | 74 | missing | - |
| KG-012 | Expand/contract monomer: / | 75 | missing | HELM edit mode, hover or replace context. |

## Menu shortcuts: guide uses CTRL, no general Mac substitution is assumed

| ID | Contextual action / key(s) | G25 p. | State | Scope / evidence |
|---|---|---|---|---|
| KM-001 | New document: Ctrl+n | 77 | missing | - |
| KM-002 | Open document: Ctrl+o | 78 | wired | Mac CmdO verified for CDXML; this Ctrl binding needs native Windows/Linux evidence. |
| KM-003 | Save: Ctrl+s | 78 | wired | Mac CmdS verified; Windows/Linux Ctrl workflow remains unverified. |
| KM-004 | Save As: Shift+Ctrl+s | 78 | wired | Typed UI shortcut and native menu flow; exact shortcut needs native evidence. |
| KM-005 | Print: Ctrl+p | 78 | missing | - |
| KM-006 | Page Setup: Shift+Ctrl+p | 78 | missing | - |
| KM-007 | Close document: Ctrl+w | 78 | wired | Mac CmdW/Cancel/Escape verified for single-window workflow; native Ctrl case unverified. |
| KM-008 | Exit application: Alt+F4 | 78 | missing | - |
| KM-009 | Cut: Ctrl+x | 78 | missing | - |
| KM-010 | Copy: Ctrl+c | 78 | missing | - |
| KM-011 | Copy CDXML: Ctrl+d | 78 | missing | - |
| KM-012 | Paste: Ctrl+v | 78 | missing | - |
| KM-013 | Select All: Ctrl+a | 78 | missing | - |
| KM-014 | Undo: Ctrl+z | 78 | wired | Mac Cmd+Z verified for page edits; this CTRL binding still needs native Windows/Linux evidence. |
| KM-015 | Redo: Shift+Ctrl+z | 78 | wired | Mac Shift+Cmd+Z verified for page edits; native CTRL workflow unverified. |
| KM-016 | Repeat last command: Ctrl+y | 78 | missing | Not an alias for redo in the guide. |
| KM-017 | Clear: Clear key | 78 | missing | - |
| KM-018 | Copy SMILES: Alt+Ctrl+c | 78 | missing | - |
| KM-019 | Copy MOL V2000 text: Alt+Shift+Ctrl+o | 78 | missing | - |
| KM-020 | Copy MOL text: Alt+Ctrl+o | 78 | missing | - |
| KM-021 | Paste SMILES: Alt+Ctrl+p | 78 | missing | - |
| KM-022 | Paste MOL/CDXML (legacy): Alt+Shift+Ctrl+p | 78 | missing | R26.1 retains the existing shortcut for CDXML; modern platform-specific binding needs confirmation. |
| KM-023 | Actual size: F5 | 79 | verified (bounded Mac) | Real native event and A4 authoring; text-entry suppression and ordered view/gesture checks. |
| KM-024 | Reduce: F8 | 79 | verified (bounded Mac) | Native187.5→150%; original factor. |
| KM-025 | Magnify: F7 | 79 | verified (bounded Mac) | Native187.5% and last drawn midpoint; generation/page-qualified focus. |
| KM-026 | Fit to window: F6 | 79 | verified (bounded Mac) | Central binding plus native menu/keys; legacy exact Fit and recovery tests. |
| KM-027 | Toggle ruler: F11 | 79 | missing | - |
| KM-028 | Crosshair: Ctrl | 79 | missing | Temporary/toggle details need the cited workflow. |
| KM-029 | Fixed lengths: Ctrl+l | 79 | missing | - |
| KM-030 | Fixed angles: Ctrl+e | 79 | missing | - |
| KM-031 | Group: Ctrl+g | 79 | missing | - |
| KM-032 | Ungroup: Shift+Ctrl+g | 79 | missing | - |
| KM-033 | Join: Ctrl+j | 79 | missing | - |
| KM-034 | Bring to front: F2 | 79 | missing | - |
| KM-035 | Send to back: F3 | 79 | missing | - |
| KM-036 | Rotate 180 degrees horizontal: Shift+Ctrl+h | 79 | missing | - |
| KM-037 | Rotate 180 degrees vertical: Shift+Ctrl+v | 80 | missing | - |
| KM-038 | Rotate dialog: Ctrl+r | 80 | missing | - |
| KM-039 | Scale dialog: Ctrl+k | 81 | missing | - |
| KM-040 | Align left: Alt+Shift+Ctrl+l | 81 | missing | - |
| KM-041 | Align centers: Alt+Shift+Ctrl+c | 81 | missing | - |
| KM-042 | Align right: Alt+Shift+Ctrl+r | 81 | missing | - |
| KM-043 | Align top: Alt+Shift+Ctrl+t | 81 | missing | - |
| KM-044 | Align middle: Alt+Shift+Ctrl+m | 81 | missing | - |
| KM-045 | Align bottom: Alt+Shift+Ctrl+b | 81 | missing | - |
| KM-046 | Distribute horizontally: Alt+Shift+Ctrl+h | 81 | missing | - |
| KM-047 | Distribute vertically: Alt+Shift+Ctrl+v | 81 | missing | - |
| KM-048 | Clean Up Structure: Shift+Ctrl+k | 81 | missing | - |
| KM-049 | 3D Clean Up: Shift+Ctrl+d | 81 | missing | - |
| KM-050 | Name to Structure: Shift+Ctrl+n | 81 | missing | - |
| KM-051 | Structure to Name: Alt+Ctrl+n | 81 | missing | - |
| KM-052 | Toggle Aromatic Display: Alt+k | 81 | missing | - |
| KM-053 | Structure Perspective (Windows): Alt+d | 81, 242 | missing | Explicit Windows binding in perspective chapter. |
| KM-054 | Structure Perspective (macOS): Option+Command+s | 56-57, 242 | missing | Explicit platform-specific binding, not an inferred substitution. |
| KM-055 | Text flush left: Shift+Ctrl+l | 82 | missing | - |
| KM-056 | Text centered: Shift+Ctrl+c | 82 | missing | - |
| KM-057 | Text flush right: Shift+Ctrl+r | 82 | missing | - |
| KM-058 | Text justified: Shift+Ctrl+j | 82 | missing | - |
| KM-059 | Text automatic justification: Shift+Ctrl+m | 82 | missing | - |
| KM-060 | Text plain: Ctrl+t | 82 | missing | - |
| KM-061 | Text bold: Ctrl+b | 82 | missing | - |
| KM-062 | Text italic: Ctrl+i | 82 | missing | - |
| KM-063 | Text underline: Ctrl+u | 82 | missing | - |
| KM-064 | Text formula: Ctrl+f | 82 | missing | Chemical text style, not a generic find action. |
| KM-065 | Text subscript: F9 | 82 | missing | In a label. |
| KM-066 | Text superscript: F10 | 82 | missing | In a label. |
| KM-067 | Degree sign: Alt+248 | 82 | missing | Guide-specific platform/input-method behavior; portability unresolved. |

## Pointer, rotation and hotspot shortcuts

| ID | Contextual action / key(s) | G25 p. | State | Scope / evidence |
|---|---|---|---|---|
| KP-001 | Select additional objects: Shift+click | 79 | missing | Lasso, marquee or structure-perspective context. |
| KP-002 | Rotate selected objects counterclockwise 15 degrees: Alt+Up | 80 | missing | - |
| KP-003 | Rotate selected objects clockwise 15 degrees: Alt+Down | 80 | missing | - |
| KP-004 | Rotate selected objects counterclockwise 1 degree: Alt+Left | 80 | missing | - |
| KP-005 | Rotate selected objects clockwise 1 degree: Alt+Right | 80 | missing | - |
| KP-006 | Rotate -12 degrees on x axis: Shift+Alt+Up | 80 | missing | - |
| KP-007 | Rotate +12 degrees on x axis: Shift+Alt+Down | 80 | missing | - |
| KP-008 | Rotate +12 degrees on y axis: Shift+Alt+Left | 80 | missing | - |
| KP-009 | Rotate -12 degrees on y axis: Shift+Alt+Right | 80 | missing | - |
| KP-010 | Move selection up 10 points: Shift+Up | 80 | missing | Selection context; hotspot context has a different action. |
| KP-011 | Move selection down 10 points: Shift+Down | 80 | missing | - |
| KP-012 | Move selection right 10 points: Shift+Right | 80 | missing | - |
| KP-013 | Move selection left 10 points: Shift+Left | 80 | missing | - |
| KP-014 | Scale around unchanged center: Ctrl+drag | 81 | missing | Scaling context; copy context uses the same modifier. |
| KP-015 | Copy selection: Ctrl+drag | 82 | missing | KB-COPY-DRAG says newer Alt without cutoff/platform;26.1 needs tool/selection/modifier/source-retention evidence. |
| KP-016 | Copy constrained to x/y: Shift+Ctrl+drag | 82 | missing | Constrained behavior cannot be inferred from unversioned copy-drag note; same gap as KP-015. |
| KP-017 | Distort resize along one axis: Shift+drag | 82 | missing | Resize handle context. |
| KP-018 | Temporary lasso/marquee operation swap: hold Alt | 82 | missing | - |
| KP-019 | Reverse chain direction: Ctrl+drag | 82 | missing | Chain tool context. |
| KP-020 | Change saturated-ring double-bond orientation: Shift+click | 82-83 | missing | - |
| KP-021 | Resonance-delocalized ring: Ctrl+click | 83 | missing | Ring tools except chairs. |
| KP-022 | Remove curve segment: Alt+Shift+click | 83 | missing | Pen tool context. |
| KP-023 | Move hotspot approximately three bonds up: Shift+Up | 83 | missing | Hotspot context; geometrically closest available path. |
| KP-024 | Move hotspot approximately three bonds down: Shift+Down | 83 | missing | - |
| KP-025 | Move hotspot approximately three bonds right: Shift+Right | 83 | missing | - |
| KP-026 | Move hotspot approximately three bonds left: Shift+Left | 83 | missing | - |
| KP-027 | Move hotspot to fragment above: Shift+Alt+Up | 83 | missing | - |
| KP-028 | Move hotspot to fragment below: Shift+Alt+Down | 83 | missing | - |
| KP-029 | Move hotspot to fragment right: Shift+Alt+Right | 83 | missing | - |
| KP-030 | Move hotspot to fragment left: Shift+Alt+Left | 83 | missing | - |
| KP-031 | Move hotspot atom/bond: arrow keys | 67 | missing | - |
| KP-032 | Focus cloned molecule: Tab | 67 | missing | - |
| KP-033 | Activate cloned/reaction hotspot: Enter | 67, 75-77 | missing | Prior hotspot and heteroatom/direction rules are part of the documented behavior. |
| KP-034 | Clone across reaction arrow: Ctrl+arrow keys | 75-76 | missing | Selection context; not just a decorative arrow. |
| KP-035 | Selection tool / last drawing tool: Spacebar | 116 | missing | Default selection-tool preference affects behavior. |
| KP-036 | Add lone-pair bar: Alt+click | 238 | missing | Lone-pair tool context. |
| KP-037 | Decrease free-site count: Alt+click | 319 | missing | Query tool context. |
| KP-038 | Override membrane fixed-angle constraint: Shift+drag | 133-134 | missing | - |
| KP-039 | Move an individual atom: Shift+drag | 88 | missing | Selection tool/atom context. |
| KP-040 | Adjust one equilibrium-arrow length: Alt+drag | 109 | missing | Arrow adjustment context. |
| KP-041 | Connect arrow at midpoint/end: Shift+drag | 108 | missing | - |
| KP-042 | Keep/curve fragmentation cut mark: Alt+drag | 193 | missing | Mass-fragmentation tool context. |

## Official 26.x desktop deltas

| ID | Reference capability | Source | State | Scope / next evidence |
|---|---|---|---|---|
| R26-001 | Non-integral mixture-ratio naming | R26.1 | missing | Naming semantics, not a cloud-only feature. |
| R26-002 | Attached-data-label selection highlights associated atoms/bonds | R26.1 | missing | Document associations and coherent selection required. |
| R26-003 | Edit > Paste Special > CDXML Text | R26.1 | missing | Separate entry; existing shortcut remains associated with CDXML. |
| R26-004 | Edit > Paste Special > MDL Text | R26.1 | missing | Separate entry; no new shortcut stated. |
| R26-005 | Invalid-stereo-bond warning | R26.0 | missing | Independent stereo validity tests required. |
| R26-006 | Updated selected atomic weights | R26.0 | missing | Use versioned public IUPAC/CIAAW data, never infer a table from product memory. |
| R26-007 | Property calculation performance and bounded LogP work | R26.0 | missing | Exact calculation limits/methods remain undocumented here. |
| R26-008 | Clearer 3D viewing during movement | R26.0 | missing | Workflow/quality evidence required; source does not define a renderer. |
| R26-009 | HELM hairpin sequences | R26.0 | missing | - |
| R26-010 | Clearer sequence crosslinks | R26.0 | missing | - |
| R26-011 | Multiple complementary strands on one RNA/DNA sequence | R26.0 | missing | - |
| R26-012 | Whole routed-bond selection highlighting | R26.0 | missing | - |
| R26-013 | ChemScript Python-version support | R26.0 | missing | Reference integration only; Bondline core remains Rust and has registry/CLI/MCP instead. |

## Additional documented context actions

These are functional action descriptions; the parenthesized contexts do not assert
undocumented menu nesting or exact capitalization.

| ID | Reference control/action | G25 p. | State | Notes |
|---|---|---|---|---|
| TC-001 | Bond context > Double > Plain | 85 | missing | Explicit context-menu path. |
| TC-002 | Bond context > Bond Position (above) | 88 | missing | Source describes three alignments. |
| TC-003 | Bond context > Bond Position (below) | 88 | missing | - |
| TC-004 | Bond context > Bond Position (centered) | 88 | missing | - |
| TC-005 | Arrow context action: bold-dashed style | 107 | missing | Guide says this style has no direct drawing tool. |
| TC-006 | Arrow context action: dipole style | 107 | missing | Guide says this style has no direct drawing tool. |
| TC-007 | Block-arrow context > Filled | 110 | missing | - |
| TC-008 | Block-arrow context > Faded | 110 | missing | - |
| TC-009 | Block-arrow context > Shaded | 110 | missing | - |
| TC-010 | Bracket context > Define Average Molecular Weight | 113 | missing | Nonnegative explicit override, not a computed mass. |

## Versioned screen and flow map

These screen IDs group the existing control inventory; they do not add to the 573 control
rows or establish complete glyph coverage. All are **missing** as reference UI workflows.
Bondline's current workspace and one approved chemical image are narrower implementation
evidence. G25 screenshots have Windows-style decoration but no independently established
application build; the filename/metadata are evidence for the guide, not every pictured UI.

| ID | Screen/state and required flow | Source and existing controls | Evidence still required |
|---|---|---|---|
| UI-001 | Main document: menus, grouped upper controls, two-column left palette, white paper, scrollbars and status; open/activate/close documents | G25 printed 12–14; MF-001–008, MV-029 | 26.1 Windows/macOS screens, menu nesting, focus, document switching and native execution |
| UI-002 | Drawing palette: selected/hover/unavailable tools, nested palettes, tear-off, dock/float and visibility | G25 printed 12–14; MV-001–010, MV-030–032, TD-* | Every glyph's role/variant and geometry, activation/hover timing, original vector assets, real tool workflows |
| UI-003 | Document settings: drawing geometry and separate atom-label typography/terminal-H controls; selection overrides remain distinct | G25 printed 40–43; MF-013, MF-016–017 | Typed settings, persistence, cancel/apply/history, all dialog controls and current platform layouts |
| UI-004 | Analysis: selection or whole-document calculations; insert a live caption, edit structure and observe update | G25 printed 221–222; MV-024 | Independent formula/mass/isotope/composition oracles, selection, captions, save/reopen and native flow |
| UI-005 | First connected drawing: empty click, endpoint extension, bond-order input, endpoint atom label, caption and ring fusion | G25 printed 422–424; TD-005–009, TD-024–026, related hotkey rows | Original acceptance scene; complete connected command model, exact history, independent graph/file checks and native workflow |
| UI-006 | Paste submenu with distinct CDXML-text and MDL-text choices | R26.1; R26-003–004, legacy ME-024/KM-022 | Actual 26.1 platform menu and shortcut, clipboard decoding, retained unknown data and editable reopening |

Implement each flow through registered commands before marking its screen delivered. For
original tool artwork, use the cited screen to identify appearance and role, then author
redistributable vector geometry. A disabled familiar control or copied screenshot is no
substitute for the flow, independent tests, accessibility and actual native execution.

## Source and completion audit

The public guide's shortcut tables were checked against rendered pages, not only extracted
column order. Menu candidates were collected across all 439 PDF pages and manually screened
for application context: operating-system package-inspection instructions, Excel ribbon
commands and Word host menus are not Bondline desktop commands. Their integration workflows
remain covered by the overview/roadmap instead of being mistaken for core app menus.

This inventory still needs a complete modern menu/palette reference and the remaining
context-menu/control enumeration. In particular, source ambiguity is not an implemented
behavior: potassium/sulfonyl `k`, the legacy quadruple-bond `4` paragraph versus the explicitly
modified ring-fusion table on p.75, example-only bond `v`,
Color/Colors and Actual/Show Document labels, and a general macOS binding table remain open.
Cloud-only 26.x notes are not silently treated as desktop requirements. No source program,
installed XML toolbar/hotkey configuration, sample file, help bundle or binary was inspected.

Pick the next unblocked roadmap item, cite its row ID in verification evidence, and implement
through the shared registry with tests before changing a state. Do not mark a row verified
because its backend, menu button, schema or placeholder exists. Keep exact platform and
workflow limitations beside the row. Preserve IDs when adding or reorganizing entries.

## Qualified capability refinements

The May2026 official chart adds nine explicit requirement groups in
[capability-chart-audit.md](research/capability-chart-audit.md), CC-01 through CC-09. They
refine the existing clipboard, document-style, stereo, query-flag, hydrogen-bond, descriptor,
format-direction, monomer and optional-integration anchors. All remain missing. These are
capability cases, not nine new menu controls; the576-control count is unchanged. The chart
is not a26.1 build or an exact workflow/default specification. Preserve its platform/tier
qualifications and the independent offline target when choosing implementation tests.
