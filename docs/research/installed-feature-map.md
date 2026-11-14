# Installed desktop feature map

Reviewed 2026-11-14. This is the first, **partial** installed-runtime reconciliation for the
complete Professional desktop parity target. It supplements the
[qualified public inventory](../parity-checklist.md), [public UI evidence](official-ui-evidence.md),
[modern inventory audit](modern-inventory-audit.md) and [research policy](parity-research-policy.md).
It does not replace their source versions or establish complete screen, shortcut or workflow
coverage. The first blank-state menu pass is reconciled control by control in the
[installed menu catalog](installed-menu-catalog.json). Implementation work is ordered in
[installed implementation tickets](installed-implementation-tickets.md).
The investigator's [native observation report](installed-ui-observations.md) owns method,
capture and cleanup details; this map owns control reconciliation and implementation scope.

## Exact reference profile

| Field | Observed value / limit |
|---|---|
| Profile ID | `CD261-MAC-ARM64-6801` |
| Installed bundle / identifier | `/Applications/ChemDraw.app`; `com.revvity.ChemDraw` |
| Short version / bundle version | Both `26.1.0.6801` |
| Host | macOS `27.2`, `arm64` |
| Executable | `Contents/MacOS/ChemDraw`, universal Mach-O; selected runtime architecture `arm64` |
| Executable SHA-256 | `d2960e9f83de35787190b45acf9ecae5171e7662b7692ea24bea7a716ef14f83` |
| Info.plist SHA-256 | `e1f058cbb08f30d9603e0cb8e3b07a1c7a4537e8161190353303ee81f695ae4b` |
| Product tier | Unknown pending an explicit runtime identity observation; bundle identity alone does not establish Professional entitlement |
| Capture tools | Existing REA `6.9.0` launcher with existing Node `26.12.1`; CUA for native interaction |
| Initial process/window | Preexisting PID `50732`, selected document window `56131`; these are session handles, not stable IDs |
| Initial document | No edit/save of preexisting contents was performed. One initial sample window later disappeared and was restored with a changed ID/position; see final-state limits below. Its help/sample contents are not implementation assets |
| Owned observation document | New blank window `56198`, created through the installed New control; no user document was replaced |

Metadata establishes identity, not successful menu or drawing behavior. The desktop
[26.1 announcement](https://support.revvitysignals.com/hc/en-us/articles/54009539308052-Release-Announcement-ChemDraw-Desktop-26-1)
provides the public release-family context; it is distinct from the cloud 26.1 release.
The [product page](https://revvitysignals.com/products/research/chemdraw) mixes desktop and
cloud capabilities and is not an oracle for an installed screen or entitlement.

## Inventory and evidence schema

Keep physical controls, logical operations and completed flows separate. Several menus,
toolbar buttons and shortcuts can call one operation; one dynamic control can represent
different operations in different contexts. An icon's visible appearance alone does not
establish its role. Record original icon construction separately when implementing it.

| Record / field | Required contents |
|---|---|
| `profile` | Exact product/build/platform/tier, artifact hashes, observation date and tool version; unknowns remain explicit |
| `surface_id`, `parent_path` | Stable local surface ID and full physical hierarchy, including window, menu, submenu, palette, flyout, dialog group or context target |
| `control_id`, `legacy_id`, `mapping` | Stable physical control ID; existing qualified logical ID where applicable; relation `candidate`, `confirmed`, `different` or `new` |
| `label`, `role`, `icon_role` | Observed label, accessible/visual control type and evidence-backed glyph role. Preserve dynamic-label conditions; do not extract vendor artwork |
| `shortcut`, `context` | Displayed and executed binding separately; OS, active tool, selection/hotspot, keyboard focus and modifiers. A displayed shortcut has not been executed |
| `enabled`, `preconditions` | State and tested cause: empty/dirty/source-backed document, active selection, whole/partial structure, dialog state, entitlement or network prerequisite |
| `parameters`, `validation` | Field names, units, bounds, choices, defaults only when established, invalid cases and visible/structured errors |
| `flow_id`, `sequence`, `result` | Initial state; ordered actions; intermediate state; authored result; cancellation; undo/redo; save/reopen outcome where relevant |
| `evidence` | Observation versus inference; source URL/page/video timestamp or local Evidence ID/path/hash; exact profile and raw artifact locator; no credentials or user-document contents |
| `bondline` | Shared command, model/format/renderer/UI paths, tests, native evidence and exact supported variant |
| `status`, `unknowns` | Bondline `missing`, `wired`, `verified` or `unknown`, with evidence scope; unresolved semantics, source conflicts and next observation |

Reference evidence has an independent strength: `metadata`, `visible`, `state-observed`,
`flow-observed`, `documented` or `unknown`. A `visible` reference control does not verify its
flow. A `flow-observed` reference action does not verify Bondline's implementation or provide
an independent chemistry oracle. `wired` means an adapter/control dispatch exists; only
decisive behavioral evidence permits `verified`, and then only for the tested variant.

Use `IR-<surface>-<number>` for installed evidence observations, `IM-<menu>-<number>` for
physical menu records, `ID-<dialog>-<number>` for dialog records and `IF-<number>` for flows.
These IDs do not create additional logical features. Preserve legacy IDs when an observed
operation genuinely matches. If behavior differs, retain both records and explain the
version/context difference instead of silently rewriting the old source.

## Existing inventory boundary

The checklist currently contains **576 qualified control records** plus **six `UI-*`
workflow-summary records**: 582 table records, not 582 distinct controls. Its qualified
prefixes are `MF`, `ME`, `MV`, `MO`, `MS`, `MX`, `TD`, `TQ`, `TB`, `DP`, `KA`, `KB`, `KG`,
`KM`, `KP`, `R26` and `TC`. These describe documented scopes, not a proven denominator for
all installed 26.1 controls.

Most guide-derived rows retain the guide-25 qualification recorded in the source register.
The reviewed guide hash is `3f1ebc429df07efdd47b6218a148dc346a215043f60bf89ee9a1cb90629bd5b3`;
the filename identifies 25 and its cover does not identify 26.1. Do not relabel those rows
as installed-build observations. A macOS Command binding also does not verify a Windows
Control binding. Public desktop-video, cloud, tier-chart and historical Mac observations
retain their separate qualifications.

## First installed evidence

All raw captures remain ignored under `target/reference-ui/`. Committed records describe
functional facts and contain no vendor screenshots, templates, fonts, help prose or icon
files. The integration owner should register the final installed evidence in
[sources.md](../clean-room/sources.md); this report does not change historical records.

| Evidence | Exact record | What it establishes / limit |
|---|---|---|
| `IR-ID-001` | `installed-identity.json`; Info.plist hash above | Metadata-only version/platform/bundle identity |
| `IR-WORKSPACE-001` | `01-initial-rea.json`; Evidence `ev_f0e45df00f8a4f7c6fb091b4380a506e8972a3f793610d5b1c8bce99103d4156`; JSON SHA-256 `60b6d419664acb4fcec09e5f208fb9c01f3ff5bee1ce82ede875b7e605d03f8f` | REA observed the selected existing window with screenshot and ten AX nodes; no menu execution or drawing flow |
| `IR-WORKSPACE-001` image | SHA-256 `cf1cfca90fc989b9942aaa50bf03a88b5a54063249ff006a0bff73b0483d0cce`, `1228 × 1005` | Capture identity only. The canvas is an `AXLayoutArea` with no exposed internal children; the capture is not a complete tool/control inventory |
| `IR-WORKSPACE-002` | `02-blank-rea.json`; Evidence `ev_0994f12812504b9efdc47a5f86fef77d209b05e36bf8d8a98cc5210cc8e01049`; JSON SHA-256 `764e717592f22d56193e55b9ed1f3a9e3ca000f1fe1e9f9d4ae250483950dde6` | Selected owned blank window `56198`; eight AX nodes, canvas still has no exposed internal children. This is an after-state observation, not a REA replay of New |
| `IR-WORKSPACE-002` image | `02-blank-window.png`; SHA-256 `7c5c5f4f03a07b9558d97d477075d0d6395ac91fb94d7a611792c57a0e739f15`, `577 × 789` | Exact bounded owned-document capture; no document-setting values or tool-role completeness established |
| `IR-PALETTE-001` failed probe | `03-main-palette-rea.json`; SHA-256 `bad66ac2e39663e1facd80cff192e81bab3d9e0a16f582099154ac177b3d3d6f` | REA rejected the selected window with `window-mismatch`; no palette observation resulted. It is a capture-selection limit, not proof that a palette/control is absent |
| `IR-MENUS-001` | `cua-menu-ledger.json`, manual structured extraction of native CUA AX observations, **not raw REA**; SHA-256 `3e87de45b7d24d9087709bef2831a5ad069bbf7daa4a1b887a338d6fb2959c09`; hash-named frozen copy under `target/reference-map/` | All 14 top-level menus, direct ordering, exposed submenu labels and blank-state enabled flags. Approximate interval `2026-11-14T00:33:42Z`–`00:39:21Z`; shortcut glyphs/checkmarks not exposed. **Search and Signals coexist in this profile** |
| `IR-PALETTE-002` | `05-main-palette-os.png`, SHA-256 `ec9c1ee74aff8fea22e3ed917a0f4830efeaee7aa82be3e1d1243db6ab414565`; receipt SHA-256 `cc76f02269075b5fe4d58934671aa61785961abdd67a4cb50c08e68c2f350af1` | Exact target-window macOS screenshot fallback, not REA, at `2026-11-14T00:43:06.659701Z`; separate two-column Tools utility palette, window `56125`, logical frame `63 × 583` at `(20,80)`. Tooltips/individual icon roles remain unverified |
| `IF-001` | `IR-MENUS-001` event `CUA-NEW-01`, File → `New ACS Document 1996` → owned blank window `56198`; `IR-WORKSPACE-002` independently captures the result | Separate blank document created while the preexisting document remained open/unchanged. Exact stationery parameters, dirty-existing-document handling and complete lifecycle remain unverified |
| `IR-SAVE-FORMAT-001` | `12-save-format-scenario-rea.json`, Evidence `ev_45c48aa6cb4d82a646ed5c43038c3d38adfb085f80da970a6097f29a5fe3c236`; SHA-256 `9fbbaab4ee9c9623a9c216d7c1ae6784c3d1d495c86e8b1628f44cf2d4ec45c0` | One completed REA AX click on the native local Save format field exposed 28 named choices plus a separator. CUA separately confirms enabled flags; no export success/default-version policy follows |
| `IR-FLOW-001` | `cua-flow-ledger.json`, manually structured CUA/REA-linked flow extract; SHA-256 `ec67bdfe5862d6cf8ea7709f9280e81d3ac74ed9eeb93a54322577df0c019538`; hash-named frozen copy under `target/reference-map/` | Ordered original New/draw/history/conditional-local-save actions and their outputs, with exact REA IDs. Not itself a raw REA report; its checkpoint precedes close/reopen and settings completion |
| `IR-FLOW-FINAL-001` | `cua-flow-final.json`, SHA-256 `fbbec60fe0243c56d57d5e665738eac6ab5fd2e76a84d6d3795a5d0370b93d9e`; final cleanup receipt SHA-256 `91d368a6d095b56deb51cc4e19c59da378dc699fb7a87cc573146b4f550b3c43` | Extends the first extract with settings Cancel, clean Close, XML Open/local bridge, binary Recent reopen, owned-document cleanup and the original-window restoration exception. Supersedes the earlier preservation assumption |

The reported File-menu families include New from Stationery; Open; Templates; Samples;
disabled Open in the cloud companion; Recent; Close/Close All; Save; Save as Stationery;
Save As; Duplicate; Rename; Move; Revert; Share; Export; Page Setup; Print; Document
Settings, Annotations and Properties; Apply Document Settings from; and List Nicknames.
The machine-readable catalog preserves the individual functional labels and hierarchy;
neither it nor this family list copies the corresponding stationery/template/sample assets.
The main palette's REA failure is a tool-coverage limit: the reviewed cached REA 6.9.0 native
helper filters layer 0 windows while the palette is layer 3. The OS fallback has separate
provenance. Do not interpret that rejected selector as absence or silently call it REA evidence.

## First menu pass, reconciled

The catalog has **503 physical records**: 333 menu items, 26 submenu containers, 137 catalog
choices, four generalized host/private-list families and three headings. It also records the
separate Save popup's 28 named format choices. These are physical entries, not 503 unique
functions or completed features. In the blank state, 301 menu records were enabled and 202
disabled; those counts include headings and do not identify the cause of disablement.

| Top-level surface | Physical records | Same-control-role mappings | Candidate mappings |
|---|---:|---:|---:|
| Application | 10 | 2 | 1 |
| File | 140 | 19 | 4 |
| Edit | 62 | 28 | 1 |
| View | 69 | 26 | 0 |
| Object | 61 | 32 | 0 |
| Structure | 45 | 41 | 0 |
| Text | 31 | 13 | 0 |
| Curves | 14 | 4 | 0 |
| Colors | 11 | 3 | 0 |
| Search | 2 | 1 | 1 |
| Signals | 2 | 0 | 0 |
| Add-ins | 6 | 1 | 0 |
| Window | 42 | 1 | 0 |
| Help | 8 | 7 | 0 |

There are 178 same-role records and seven candidates, referring to 184 distinct existing
logical IDs. The unpaired records remain `not_matched`; they are not automatically new
logical features. Containers, parameter choices, platform services and duplicate physical
access paths need different treatment. Alias matching never promotes guide-25 flow or
shortcut evidence to this installed build. In particular, the application-menu update
control, stationery path and web destination differ from legacy guide paths and remain
candidates. Edit Delete is only a candidate for guide Clear until selection behavior is
observed. The obsolete combined MDL/CDXML paste entry is not reused: separate observed
entries map to `R26-003` and `R26-004`.

The menu pass includes 21 named stationery entries, 30 File template choices, 36 sample
choices and 29 View template choices; these name reference content roles only. Recent and
open-document filenames are omitted, retaining their control families. Services leaves and
the Font submenu remain unenumerated; the latter exposed no AX children. Separator positions,
shortcut glyphs/checkmarks, selection-dependent state changes and unexposed context/flyout
controls remain unknown. Existing `KA`/`KB`/`KG`/`KM`/`KP` shortcut rows are not verified by
menu-label matching.

The Save popup distinguishes generic and explicit V2000 RXN/MOL/SDF/RD choices and lists
XML/binary/legacy/stationery, PDF/raster, SVG, CML/TGF/SKC/MSI/SMD and two 3MF roles. The
catalog preserves all 28 names. It does not infer encoded versions, extensions, fidelity or
default-selection rules. The installed configured profile first showed a cloud-save prompt
with a local-computer bridge; this does not add an account dependency to Bondline.

The same catalog contains 12 local Save field roles: location, disclosure, filename, tags,
optional integration, format, stationery location, CR/LF/CRLF choices, Cancel and Save.
Adjacent labels establish the unnamed filename/tags/format AX fields; the disclosure role
remains unnamed. The XML preparation capture observes LF selected. Settled CDX selection
removes the EOL radios and reports no format options. Fields retain null enabled metadata
where REA does not expose it; only Save was explicitly confirmed enabled by the CUA receipt.
An unsupported radio-group AX value is not silently substituted for its exposed child radios.
Some sheet captures have `truncated=true` and retained AX gaps. The complete-AX-coverage
check failed and remains in `validation-initial-coverage-failure.json`; successful integrity
checks and completed declared actions do not establish a complete accessibility tree.
The catalog retains truncation/gap counts for its dialog sources. Visible CUA/OS field
groups and successfully completed scenarios retain their narrower evidence scope.

## Bounded original drawing/history/save flow

`IR-FLOW-001` records the following actions on the owned document. These are reference
observations; no Bondline status is promoted by them. CUA input tuples, window coordinates,
HiDPI pixels and document points are distinct. No coordinate transform, fixed-length toggle
state, angle-snapping rule or universal default is inferred from the single drag.

| Flow | Preconditions and ordered action | Actual result / scope |
|---|---|---|
| `IF-001` New | Preexisting document current → File New | Separate owned blank window; prior document preserved; dirty-document handling untested |
| `IF-002` Draw | Owned blank, no hotspot → `x` → drag input tuple `(280,300)` to `(340,300)` | Edited marker, enabled Undo Drawing/Select All and stable screenshot of an isolated line. `KG-002` is a corroborated candidate tool binding, not a complete tool/hotspot contract |
| `IF-003` Undo/Redo | Edit Undo Drawing → inspect state → Edit Redo Drawing | Undo returns blank/clean, disables Undo/Select All and enables Redo Drawing. Redo restores Edited; exact PNG SHA equals post-draw SHA |
| `IF-004` Unassociated Save route | File Save → close configured cloud prompt; File Save As → local-computer bridge | Configured profile showed an embedded cloud-save prompt. Only its close/local bridge was used; no explicit cloud submission/sign-in/account action |
| `IF-005` XML Save As | Local sheet → XML format, new owned path/name, LF → REA Save click and 250ms wait | File created, associated title, clean marker. Independent XML inspection sees one page/two atom nodes/one bond. Undo Drawing remains enabled after this exact Save As route |
| `IF-006` Associated Save As to binary | Same document associated with local XML → Save As → native sheet directly → binary format/new owned path → REA Save click and 250ms wait | No cloud bridge appeared on this route. Binary file created, associated title; settled binary options differ from XML. Decoder/chemistry parity is not established by file presence/header |
| `IF-007` XML close/reopen | Clean owned binary Close → File Open → configured cloud Open prompt → local-computer bridge → native Open exact owned XML path | New clean XML window `56477`, exact file association; CUA actions with passive REA after-state. No post-reopen edit/selection tested |
| `IF-008` Binary reopen | Close reopened XML → File Recent → exact owned binary entry | New clean binary window `56515`; passive REA after-state. Reference reader success is distinct from Bondline's parser blocker |
| `IF-009` Settings/cleanup | Inspect three settings categories → REA Cancel → clean owned documents Close → restore preexisting source/focus | No settings applied; all owned documents closed; saved hashes unchanged. One original sample window had disappeared and was restored at a different position |

Exact draw/history Evidence IDs are
`ev_4db8425971b37f244182f9122fb9acbd8bbfa03f3c505a40accf1010a37818dd`,
`ev_b2117eacc6fc625acd70268f3b193020acea27dc8ebb11835ed2ed1b30eb0bf3` and
`ev_0a6120a8f4bc3d09a2ea3c7b56b1b883383330b2a98bb5c7437b3e50c231a580`.
Their raw files are `07-after-drag-rea.json`, `08-after-undo-rea.json` and
`09-after-redo-rea.json`. Post-draw/post-redo PNG SHA-256 is
`4ff2e6fdf7fdb61c9f07e415e257bcfce7aa07cb6a60db99338dd5abf224b1ea`.
The immediate CUA draw image was stale; only the later stable capture supports the result.

`IR-FLOW-FINAL-001` records actual reopen actions, not merely a file-title inference.
XML used File Open, the configured cloud-to-local bridge and the native local Open panel;
binary used the exact owned Recent entry after its original window was closed. The
corroborating passive after-state Evidence is
`ev_8a6a052772697ce4d139fcf8d129f6ebf53b4452be6991b978edc1e58fc820bd`
(`21-reopened-cdxml-rea.json`, SHA-256
`3cd47a81ea2ff4adff369ddf389efe8ac1f01aca97a9cdf2442abc202945245b`) and
`ev_bdb8c4cbe43dd5b44feb39442e55dec95c169cec5b4ccbbc46bd9df81cdc6d17`
(`22-reopened-cdx-rea.json`, SHA-256
`8ba56e969890adf2abc3883ab38a102773dd2c131eb7c524130957b95e3a2101`).
Reopened pictures include a small red marker at the original bond location; its meaning
and complete pixel fidelity were not established. No post-reopen atom/selection edit was
tested, so the complete continued-editing workflow remains open.

Successful local save Evidence is
`ev_7806f73a2e98d94f87aca959b2faa437924f989a59260f2176f26620061b0471`
(`14-save-cdxml-scenario-rea.json`, SHA-256
`70a2b3dde483eeecfafbae3fbb5de6c9cb4583759ff9b44c7c0e7ed27b9ba77e`) and
`ev_a85f9c381b08fdf4354348187da9a422c78cd771164e4d0271334b67a2237dc6`
(`16-save-cdx-scenario-rea.json`, SHA-256
`1f6bd28797ee78cf42f5c2043194ab4f4e7f5f9bf5ce166349a27853a5ec1956`).
The prepared XML/binary sheet Evidence IDs and exact fields are linked in the catalog.

The original XML output is 2,866 bytes, SHA-256
`73de4b424e27a01e98be6830d726025fc5f36f8786d02bab594993bac12ed4b5`,
with 111 LF and no CR. Python ElementTree independently finds atom-node IDs 7/9 at
`(330,258)`/`(342.47,250.80)` and bond ID 10 joining 7→9. Element/order/H attributes are
omitted. The SDK's [node-type default](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Node_Type.htm),
[element default](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Node_Element.htm)
and [bond-order default](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Bond_Order.htm)
identify carbon/single-bond interpretation; they do not establish authored H, formula or
analysis results. The writer adds
style/font references/print metadata and a warning attribute; no warning prose, font file
or stylesheet is copied into Bondline. The binary output is 1,196 bytes, SHA-256
`7b2b71338595ad12bc34f46459f8389ec6a63cb453b1cf29b51543717139a73c`, with `VjCD0100`
signature. Both original outputs remain ignored research artifacts pending corpus/asset/
redistribution review; neither is an admitted fixture.

The independent consumer record `local-original-external-consumer.json`, SHA-256
`f8cf2e255c6cec2c668b2709dbc7490f9de725207ec2c3d27a38364d114def57`, records pinned
RDKit `2026.09.2` consuming each unmodified output as one two-atom/one-bond molecule,
returning external SMILES `CC` and formula `C2H6`. These are external-oracle results,
not Bondline chemistry, authored-H metadata, license approval or general format parity.

The receiving-root standalone parser probe at `85bf5ebfd53580b66cde9726a374658fc9befcee`
accepts the XML with exact input/output and two-node/one-bond structure, but rejects this
binary with `Error::TrailingData`. Manifest `parser-probe-evidence.json`, SHA-256
`9abccee13251991010af4c73a86c8bc8edf78218378675025516e14f4003e864`, records the command/log
and independent byte decoder. That decoder identifies document framing at offset 22, its
end at 1194 and a separate file end through 1196. Public writer documentation explains the
legacy header overlap; [M1-04a](installed-implementation-tickets.md#m1-04a--accept-the-actual-native-framing-before-model-mapping)
tracks the original framed regression and narrow repair. At this checkpoint it is in progress,
not GREEN. No bytes were trimmed and no general application CDX compatibility is claimed.

The embedded cloud dialog may have initialized network activity automatically. No network
trace was collected, so zero transmission is **not** asserted. These two configured-profile
routes do not prove a universal initial Save destination. Chemistry analysis, connected
endpoints, rings, labels, selection transforms, modifiers and full editor parity remain open.

## Settings surface reconciliation

The catalog records nine category IDs in native order: Layout (`DP-009`), Header/Footer
(`DP-010`), Drawing (`DP-011`), Text Captions (`DP-013`), Atom Labels (`DP-012`), Property
Labels (candidate `DP-014`), Biopolymer Display (`DP-017`), Reaction Display (`DP-016`) and
Colors (`DP-015`). The old Analysis label is not silently renamed to Property Labels.

Only Layout, Drawing and Atom Labels field groups were inspected. Layout distinguishes
Pages/Poster, rows/columns, dimensions, margins, overlap, registration, printer setup and
display units. Drawing separates chain angle, spacing mode, fixed length, widths, hash/
margins and atom/bond indicators. Atom Labels separates typography, spacing, formula
baseline and terminal-carbon/implicit-H flags. None was edited or applied. Values belong
to this selected stationery/profile; current journal presets need public author guidelines.

The exact REA17/18/19 source IDs/hashes are in the catalog. They select the parent document
and expose attached-sheet AX; their screenshots show parent-document pixels. The separate
Drawing/Atom Labels OS sheet images have their own provenance in the observation report.
Do not claim those REA screenshots show modal pixels. The detailed functional settings
extract is `cua-settings-ledger.json`, SHA-256
`270eb87822188455192e4f50524a0eaa40b384b258cbd3f39a90d08382d31c83`.
REA Cancel completed in `ev_3e7e42f306b36a2179b7f40b253bd9f5e82a0495c3322b26a95d06e7cf861c25`
and returned to the clean document. Six uninspected categories, parameter validation,
complete units/ranges, defaults and Apply/OK/persistence behavior remain unknown.

## Final state and observation limits

The pass ended at 2026-11-14 01:11 UTC. Every owned original/reopened document window was
closed; both saved outputs retained their recorded hashes. No owned REA/native jobs or
persistent debugger sessions remain. The preexisting application remains running and the
UI investigator issued CUA completion once, then released GUI ownership.

Do not claim uninterrupted preservation of all initial windows. Initial sample window
`56131` was present in metadata04 and absent by metadata10; its disappearance cause is
unestablished and no intentional Close targeted it. The investigator restored the exact
installed Locked source through its existing Recent entry as window `56535`, current/focused.
Its size is unchanged but its position differs. Original windows `56129`/`56130` and the
palette IDs/frames remain unchanged. Toolbar toggle restoration is inferred from twice
invoking the same action; checkmarks and initial tool state were not exposed. The final
receipt, rather than the earlier presumed-preservation checkpoint, is authoritative.

## Reconciliation register

| Installed observation / source | Existing anchor | Mapping and reference strength | Bondline location / exact status |
|---|---|---|---|
| Top-level menu bar | `UI-001`, modern-audit `MIA-005` | Confirmed visible Search **and** Signals on this installed profile; all 14 blank-state menus reconciled in the catalog, other builds unobserved | `crates/ui/src/lib.rs`, `crates/commands/src/controls.rs`; complete reference menu model missing |
| `New ACS Document 1996` | `MF-001`, `MF-019`, `KM-001` | Creation role matches `MF-001`; actual New produced a separate blank. Stationery-path mapping remains candidate, content and binding untested | No `document.new` registry command; missing. Existing default A4 document is an original Bondline policy |
| Open / Save / Save As / Close roles | `MF-002`, `MF-004`, `MF-005`, `MF-007` | Ordered CUA actions plus REA scenarios/after-states establish bounded original local XML/binary saves, clean Close, XML Open and binary Recent reopen; broader variants remain unknown | `commands/{files,file_ui,close,controls}.rs`, `ui/file_workflow.rs`; bounded Bondline Mac CDXML/MOL variants have prior verification; binary application commands and broad reference parity missing |
| Document Settings / Apply settings | `MF-013`, `MF-016`, `DP-009`–`DP-017`, `UI-003` | Nine categories observed, field groups of Layout/Drawing/Atom Labels inspected; other fields/default/application semantics unknown | `page.set_size` is narrower; style-sheet and complete settings workflows missing |
| Templates / stationery / samples | `MF-018`–`MF-023`, `TD-049` | Family visible; exact installed library/asset contents are not a redistributable product input | Original template authoring/library workflow missing; do not copy installed assets |
| Nickname control | `MF-024`, modern-audit `MIA-008` | Menu family visible; chemical expansion versus visual-only labels untested | Model-backed abbreviations and chemistry-aware nickname editing missing |

## Coverage board and next observations

| Surface/state family | This installed sweep | Next decisive evidence |
|---|---|---|
| Identity / owned blank workspace | Metadata and first captures; tier unknown | About/identity observation without personal entitlement data; owned blank capture |
| Application/File menus | First blank-state pass and New action; named dynamic stationery/template/sample choices recorded | Changed/dirty/source-backed variants, shortcuts, enablement transitions, cancellation and save/reopen on owned data |
| Edit/View/Object/Structure | All exposed blank-state direct/submenu controls reconciled | Whole-selection, partial-selection, changed-history and focused-object variants; successful operations |
| Text/Curves/Colors | All exposed blank-state menu controls reconciled; Font leaves unknown | Tool/focus-dependent controls, dialogs and one representative authored result per family |
| Search/Signals/Add-ins | Exposed blank-state controls recorded, none invoked | Separate offline desktop operations from account/network integrations; do not introduce an account requirement |
| Window/Help | Exposed menu roles recorded; private filenames generalized | Multi-document focus/close behavior; Services/help destinations; original help assets |
| Main palette / flyouts / icon roles | Exact OS fallback capture of separate two-column palette; no role-complete observation | Tooltip-labelled controls, press/hover/flyout behavior, selected/disabled state, docking/floating |
| Context menus / hotspots | Unobserved | Blank/page/atom/bond/text/arrow/whole-fragment/partial-selection contexts and modifier precedence |
| Document/object settings | Nine document categories and three panels' field groups; object settings not opened | Remaining fields/units, Apply/OK/Cancel effect, defaults versus document, independent persistence |
| Analysis/naming/reaction/biopolymer panels | Unobserved | Entry controls, live updates/errors, exact authored graph/data, reference tier and offline boundary |
| Native dialogs / printing / clipboard | Save popup's 28 choices, 12 field roles and bounded XML/binary local-save route captured | Other encodings, extension mismatch, cancellation/errors, clipboard and physical-scale consumers |
| Windows/Linux and browser | No installed evidence | Separate platform captures and binding/lifecycle verification; Mac results cannot establish these |

An unopened flyout, offscreen row, truncated AX tree or disabled control is **unknown**, not
absent. Track tested contexts explicitly rather than claiming a percentage of all features.
The priority reference flows are: new → connected drawing → label/order change → selection
edit → undo/redo → save → close → reopen; source-file open → supported edit or explicit
refusal → lossless save; and document settings → authored drawing → publishing. Each flow
needs its own parameter/error/cancellation and persistence record. New/isolated drawing/
history/local save/clean close/reopen now has bounded reference evidence. Continued editing,
general molecule workflows and complete parity remain unestablished.

## Separately scoped palette follow-up

The [36-cell palette report](installed-tool-palette.md) and
[static binding catalog](installed-tool-catalog.json) now supply candidate roles and13
secondary-toolbar declarations from the installed GUI XML. These are additional static/
visual evidence, not runtime tooltips, enabled states, hit bounds or executed workflows.
They do not change this first-pass catalog, its original source hashes or qualified
logical-feature count. Its separate controlled blank pass preserved all original IDs/frames.
