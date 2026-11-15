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
