# Live installed UI review — 2026-11-15

The user explicitly requested opening installed ChemDraw to inspect and reconstruct its
screens and functionality. This later instruction authorizes the visible runtime pass
despite the earlier public-only/no-execution agent boundary. Public specifications remain
the basis of file semantics; no proprietary code, resource or redistributable asset is
acquired by this pass. Original Bondline code/assets and the license policy remain.

## Method and selected target

The reverse-engineer-anything skill's runtime/evidence workflow was applied. Its REA MCP
native UI tools are not advertised in this session, so CUA supplies the actual interaction
and observation. No REA engine, debugger, static binary/resource inspection, subprocess
capture, server Evidence ID or artifact hash is claimed. There is no native-analysis
session to close/reset. CUA completion was recorded after the final observed window.

CUA `getApp("ChemDraw")` opens/selects `com.revvity.ChemDraw`; Raise brings its document
forward. About visibly reports version **26.1.0.6801**, product **Signals ChemDraw**.
This does not establish Professional entitlement. Licensed-person information is omitted.
The current app/build aligns with the earlier installed profile, but no executable hash
was rechecked and the entire historical503-entry catalog was not replayed.

Observations below refer to inline CUA AX/pixel results in this session. These are local
observation labels, not REA Evidence IDs or pinned exported screenshot artifacts.

## Bounded findings

| ID | Actual observation | Bondline mapping and remaining gap |
|---|---|---|
| LIVE-001 | Initial blank document is titled `Untitled ACS Document 1996-1`; selected document capture shows white paper, window chrome and scrollbars. | Original paper/scrollable workspace exists. This selected-window capture does not establish the complete application/palette layout. |
| LIVE-002 | File contains `New ACS Document 1996`, separate New from Stationery, Open, Open Templates, Open Samples, Close/Close All, Save/Save As/Save As Stationery, Duplicate, Revert To, Export, Page Setup/Print and document-settings roles. | MF-001–008 and style/template/publishing rows remain separate. No template/sample was opened, inspected or copied. Labels establish available controls, not completed flows. |
| LIVE-003 | Click File's New action: a blank `Untitled ACS Document 1996-2` appears. Window menu then lists both `…-1` and `…-2`. | MF-001: separate styled-window New is flow-observed. Bondline3b0387c has verified source-free same-window New with Save/Discard/Cancel; multiple windows/style defaults remain missing. Do not advertise these policies as matching. |
| LIVE-004 | View lists main/general/style/object/structure and other toolbar roles; Analysis, Chemical Properties, Info, Periodic Table and Character Map windows; additional tool groups; chemical-warning/reaction-interpretation roles and zoom actions. | Existing MV/control/tool anchors cover these roles. Analysis/chemistry/palette workflows remain missing. No new logical row or completion percentage is invented from this menu pass. |
| LIVE-005 | Show Main Toolbar was selected once. The subsequent Edit menu says `Repeat Show Main Toolbar` disabled. The selected document capture still exposes no floating palette. | Action invocation is observed; actual palette visibility, location and glyph/tool roles are unresolved. The command name alone is insufficient to claim a visible toolbar. No invented restoration/visibility result. |
| LIVE-006 | In the empty document, Edit's Copy As children include disabled SMILES/SLN/InChI/InChI Key/CDXML/MOL/HELM/FASTA/image roles. Paste Special includes enabled SMILES, Name as Structure, InChI, CDXML Text, MDL Text and sequence roles. | ME-015 and ME-025 are distinct operations. Bondline's explicit-file, restricted carbon-path converter and selected-fragment exporter are not these clipboard workflows. No paste/copy chemistry was executed or inferred. |
| LIVE-007 | About was closed; final menus dismissed. ChemDraw remains frontmost at the new blank `…-2` window. Window menu had confirmed the original remains. | No preexisting document edit, save, close or replacement was performed. The new blank document remains open for subsequent reference work; no analysis/background job owns it. |

## Implementation consequences

The new Bondline document feature remains one tested increment, with unchanged
[independent/native evidence](../verification-new-document.md). This pass adds an actual
reference difference; it does not retroactively change that command's source or tests.
Keep style-dependent New and independent multi-window document/history/close/file routing
in the complete parity backlog. Do not load a proprietary stationery asset as an
implementation shortcut.

For the next native SMILES converter increment, present its actual create-only file
contract and explicit profile limitations. A file-conversion dialog does not complete
Paste Special → SMILES or Copy As → SMILES. Those need their own source-qualified command,
clipboard ownership/error handling, independent graph/file checks and native acceptance.

Next reference evidence should resolve the floating palette's actual capture/visibility
and individual control/flyout states, then exercise agent-authored chemistry through
bounded creation, selection, SMILES conversion, history and persistence. Do not reuse a
sample/help document as a product fixture or infer enabled-state causes without changing
and observing the relevant selection/context.

This pass answers launch/build identity, selected menu inventory and styled second-window
New. Complete tool/icon geometry, shortcuts, dirty reference decisions, chemistry
clipboard flows, Save/reopen semantics, other platforms and one-to-one screen/functional
coverage remain unresolved. App launch and visible menus are not complete parity.

## Renewed visible-menu pass

On the user's renewed request to open ChemDraw, CUA selected the same running
`com.revvity.ChemDraw` and raised `Untitled ACS Document 1996-2`. No build identity or
entitlement was rechecked. The blank document and both numbered Window entries remain.

| ID | Actual observation | Implementation consequence |
|---|---|---|
| LIVE-008 | View again exposes the main/general/style/object/structure palette roles. Invoking Show Main Toolbar and Window → Bring All to Front does not expose a palette in the selected document capture. | Palette visibility and geometry remain unresolved. Do not repeat the action or infer a successful show/hide transition from its label. |
| LIVE-009 | On the blank document, Structure exposes separate structure/reaction/biopolymer/3D cleanup, aromatic/hydrogen display, enhanced stereo groups, reaction mapping/stoichiometry and naming roles. All observed leaf actions are disabled except Convert Name to Structure. | These are control availability observations, not chemistry results. The existing structure/stereo/reaction/naming anchors remain missing; enablement with actual selections still needs a separate flow. |
| LIVE-010 | Object exposes fixed-length/fixed-angle/stereochemistry roles; its object settings, annotations, data attachment, alignment/distribution, frames, grouping/joining, stacking, flip/rotate and scale leaf actions are disabled in the blank context. | Keep geometric transforms, chemical transforms, grouping and topological Join distinct. No default toggle value, object-setting preset or selection rule is inferred. |
| LIVE-011 | Document-title controls were opened and dismissed without typing. Menus were then dismissed through their advertised Cancel action; final AX/pixels again show the unchanged blank document. | No document was renamed, edited, saved, replaced or closed. ChemDraw remains open. |

REA native UI tools are still absent from this tool session. CUA is the actual evidence
producer; no REA capture, exported screenshot, binary hash, debugger or static resource
inspection is claimed. The optional CUA launch helper is unavailable on this host; app
selection/Raise and menu actions provide the observed results. Keyboard Escape did not
produce an observed menu dismissal in this pass; advertised AX Cancel did. Its cause is
unknown, so no keyboard shortcut or ordinary native input flow is claimed from that attempt.

## Document-settings observation on renewed instruction

The user again explicitly requested opening the installed application. CUA selected
`com.revvity.ChemDraw`, raised the existing blank `…-2` document and used ordinary native
menu actions. About again visibly reports **26.1.0.6801 / Signals ChemDraw**; no entitlement,
executable/resource hash or Professional-tier equivalence is inferred. Licensed-person
information and Recent-document names are omitted from this record. REA MCP runtime tools
remain absent; these are inline CUA AX/pixel observations, not REA Evidence or exported
screenshot artifacts. No static binary/resource inspection or proprietary asset acquisition.

| ID | Observed screen/control or action | Mapping and evidence limit |
|---|---|---|
| LIVE-012 | File and View again expose the previously recorded file, settings, panel and toolbar families. In blank Structure, the cleanup, stereo, hydrogen/aromatic display, reaction mapping/analysis and structure-to-name leaves are disabled; name-to-structure is enabled. | Reconfirms only those visible menu roles and that context. No chemistry action, selection-dependent cause or successful naming flow was executed. |
| LIVE-013 | File → Document Settings opens a sheet with a left category list and persistent bottom display-unit selector, Cancel and OK. Nine categories are Layout, Header/Footer, Drawing, Text Captions, Atom Labels, Property Labels, Biopolymer Display, Reaction Display and Colors. | DP-009–017 / UI-003. Four categories below were actually opened; the other five have only visible category evidence. Bondline does not yet implement this settings workflow. |
| LIVE-014 | Layout shows Pages selected, Poster available, row/column steppers, document size, four margins, overlap/registration controls and Page Setup. Display units show Inches. The selected document shows1×1 pages,7.5×10in drawing size and0.5in margins; overlap/size/registration controls are disabled in this context. | DP-009. These are this document's visible values, not factory or journal defaults. Poster switching, numeric validation, page setup, printing and application of changes were not tested. |
| LIVE-015 | Drawing shows chain angle120°, fixed length0.2in, percentage bond spacing18, absolute spacing disabled at0.05in, bold width0.028in, line width0.0083in, margin0.022in and hash spacing0.0346in. Separate atom and bond indicator groups expose query/stereo choices; enhanced-stereo and atom-number choices belong to atoms, reaction indicators to bonds. | DP-011. Distinct controls and their selected-document values are observed. No setting was edited or adopted as a Bondline preset. Bondline's explicit conversion geometry and original centered multiple-bond profile do not complete this style system. |
| LIVE-016 | Atom Labels has a font-family table, typed size and size-list selection, line-spacing modes, font styles and Normal/Subscript/Superscript/Formula baseline roles. The current state shows Arial10, Variable spacing and Formula, with terminal-carbon labels and hide-implicit-H unchecked. | DP-012. Typography, visibility and authored-H semantics remain distinct. No font file was obtained, no font/default policy adopted and no changed label was rendered or saved. |
| LIVE-017 | Reaction Display shows a numbering start field/stepper at1, numeric-style popup showing Roman, format field `(#)` and an unchecked option for including structures above/below arrows. | DP-016. This is new field-group evidence, not a reaction editor, numbering result, range-validation rule or default-policy proof. Bondline's reaction/numbering workflow remains missing. |
| LIVE-018 | Cancel closes the settings sheet. Window still lists both numbered documents; Bring All to Front returns the same blank `…-2` selected-document capture. Menus and About are dismissed; ChemDraw remains running. | No document edit, rename, Save, Close, replacement or settings acceptance. Show Main Toolbar was invoked once in this pass, but a floating palette was not exposed by the selected-document capture; its visibility/change remains unresolved. |

The selected sheet captures establish a category sidebar, grouped controls and bottom
commit/cancel roles. They do not establish the entire app/palette geometry, icon roles,
screen scaling, keyboard bindings, settings validation, persistence, undo behavior or
Professional parity. The observed font and stylesheet values are not redistribution rights
for vendor fonts, stationery or assets. Implement original controls/assets only after a
source-qualified command/model contract and independent behavior tests; keep these DP rows
missing until their Bondline workflows actually work.
