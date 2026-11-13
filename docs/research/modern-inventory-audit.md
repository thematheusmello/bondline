# Modern desktop inventory audit

Reviewed 2026-11-13. **The complete Professional 26.1 menu/tool/hotkey inventory remains
open.** This audit adds version-qualified vendor guidance and bounded public desktop-video
observations. It does not mark any Bondline capability implemented or verified.

Read alongside [official-ui-evidence.md](official-ui-evidence.md),
[reference-projects.md](reference-projects.md), the [parity checklist](../parity-checklist.md)
and [source register](../clean-room/sources.md). Their existing G25 guide, release notes,
cloud-demo observations and download-center login evidence were not repeated. No desktop
reference application, installer, installed configuration or help bundle was acquired.

## Scope and evidence rules

- Observed means the cited document states it, or the specified screenshot/video frame
  actually shows it. A document statement is not a replayed application interaction.
- Inferred platform or version information is explicitly identified. A publication or
  article-update date does not establish the pictured application's build.
- The public knowledge-base section listed 699 articles. Only its first two 100-article
  pages, sorted by update date, were title-screened, with selected relevant bodies read;
  this is not a complete support audit.
- Focused searches for desktop 26.1/26.0 user guides and manuals did not locate a public,
  version-identified complete guide. This is a discovery result, not proof none exists.
- Reference images and complete browser Evidence remain ignored research artifacts under
  `.cache/research/modern-inventory/` and `target/research/modern-inventory/`. They are not
  redistributable product assets, templates or artwork. This report paraphrases behavior
  and names functional controls; it ships no reference screenshots or help paragraphs.

## New source map

The raw public article JSON was fetched without authentication from each article's matching
`/api/v2/help_center/en-us/articles/<id>.json` URL. Exact source URLs, response sizes,
timestamps, SHA-256 values and acquisition failures are in the ignored acquisition manifest.

| ID | Primary public source | Version/platform actually established | New evidence and limit |
|---|---|---|---|
| S1 | [Atom label editing gesture](https://support.revvitysignals.com/hc/en-us/articles/44882776753300-ChemDraw-Double-clicking-on-an-Atom-adds-a-new-bond-instead-of-the-text-box-to-enter-the-label) | Body specifies 25.5.0.5789; platform not stated | Modifier-click opens atom-label editing; double-click no longer has that meaning. This is a documented change, not an observed 26.1 interaction. |
| S2 | [Repeating an atom label](https://support.revvitysignals.com/hc/en-us/articles/44882820468884-ChemDraw-Unable-to-repeat-the-atom-labels-by-Double-clicking) | Body specifies 25.5; Windows and Mac bindings separately stated | Text-tool repetition uses Windows Alt+single-click or Mac Option+single-click. Do not generalize the Mac binding to other tool contexts without evidence. |
| S3 | [Whole-structure alignment requirement](https://support.revvitysignals.com/hc/en-us/articles/44882777539732-ChemDraw-Object-Align-Tools-are-not-enabled-while-selecting-part-of-the-structures) | Body specifies 25.5; screenshot title identifies Professional; exact build absent | Alignment controls are unavailable for partial chemical selections and available for whole structures. The two-state screenshot was visually inspected; no interaction replay was performed. |
| S4 | [Mac toolbar docking](https://support.revvitysignals.com/hc/en-us/articles/19712365503764-ChemDraw-How-do-I-Dock-Toolbars-to-the-Document-Window-on-a-Mac) | Body dated 2026-09-22, explicitly Mac; pictured running build unidentified | Describes application-boundary docking rather than docking to an individual drawing window. The screenshot contains desktop filenames mentioning 23.0.1, which do not identify the running build. It is not a 26.1 Mac screenshot. |
| S5 | [Horizontal-distribution shortcut conflict](https://support.revvitysignals.com/hc/en-us/articles/9693972428692-ChemDraw-Unable-to-use-ChemDraw-Distribute-Horizontally-shortcut-during-Zoom-meetings) | No product build specified; Ctrl/Alt/Shift binding stated | Corroborates the horizontal-distribution shortcut and availability through an object menu or context menu. It does not enumerate the context-menu nesting. No external application's settings were changed. |
| S6 | [Professional BioDraw templates](https://support.revvitysignals.com/hc/en-us/articles/4408239586836-ChemDraw-Where-are-the-BioDraw-Templates-in-ChemDraw-Professional) | Professional explicitly named; no build | Corroborates a View-menu control for the BioDraw template palette. This is not permission to ship its template library. |
| S7 | [Legacy biopolymer toolbar removal](https://support.revvitysignals.com/hc/en-us/articles/4408239538068-ChemDraw-The-Biopolymer-toolbar-is-missing-on-ChemDraw-18-1) | Explicitly 18.1, despite a 2025 update timestamp | Historical toolbar/shortcut replacement by HELM. Useful as a warning against resurrecting a legacy toolbar; not current 26.1 layout evidence. |
| S8 | [B(pin) label-orientation workaround](https://support.revvitysignals.com/hc/en-us/articles/52166074669716-ChemDraw-Preventing-B-pin-Label-Flipping-in-ChemDraw) | Published 2026-08-05; no build/platform | Documents a nickname-based visual workaround and warns that this particular representation is unsuitable for formula/mass calculations. Do not infer chemical meaning from a displayed abbreviation alone or generalize this limitation to all nickname definitions. |
| V1 | [Official September 2026 desktop webinar](https://www.youtube.com/watch?v=iv3ciKZz9rg) | Revvity Signals uploader; description dates webinar 2026-09-23; public player metadata gives upload 2026-09-24; title bar says Professional | Bounded playback and desktop screens observed below. Exact version/build/About information was not shown in the reviewed frames. |

Pinned article JSON SHA-256 values:

| ID / article | SHA-256 |
|---|---|
| S1 / 44882776753300 | `94803034630827e2a113a8b9afb68a5fd36d94a6df6315aea0851e78db742290` |
| S2 / 44882820468884 | `a59ab5033df8267d0485e688e41a3c56b30113547c6f3abeaead12c3f567d370` |
| S3 / 44882777539732 | `ca6b406e9131200dcae48c257060c8f3d5eb1280b5c20a1a5dc85839fdc09399` |
| S4 / 19712365503764 | `1be82f23c815a6c098ded3b79dead9f39606d3b24fc63e77eb8d530abc585e1d` |
| S5 / 9693972428692 | `b461a45ea3d8bab67e8d994e6255f6988576a2c36d32f048d4159f373ae0fd29` |
| S6 / 4408239586836 | `4fb78e132693df5960ad872d952ad81e8a88b21d70234f38110a74cad66999b6` |
| S7 / 4408239538068 | `1c393a703f342863191df1bce6cc086b517c37492cbba1c171f451a6786dced1` |
| S8 / 52166074669716 | `4afbbb9fb83035292e8568eeb0a6dce5afc7baf952c5c81e718415ec1bd44c6b` |

The S3 article's visible date is January 2026, while its fetched API `updated_at` is
2026-06-30. S4's body says September 22 while its API update is October 6. Neither field
establishes screenshot age. The two public section-response hashes are
`571190b5b7b8a2757ff37c1064e7cafbcfbeae8a7964b6575bdf9103c9248140` and
`687ef70f4f935fd42155fb47a40b871154b04c622ee867bc3754878dbd056abc`.

## Concrete inventory additions or corrections to review

These proposed IDs are local audit candidates, not additions to the root checklist yet.
Preserve existing row IDs when promoting them. All corresponding Bondline behaviors remain
missing; source research is distinct from implementation status.

| Candidate | Existing anchor | Requirement or correction | Source / unresolved qualification |
|---|---|---|---|
| MIA-001 | UI-005, atom-label/tool gestures | Record Alt+single-click for entering an existing atom's label from 25.5.0.5789; do not use an older double-click tutorial as the sole current interaction oracle. | S1. The article does not specify OS; Mac label-entry binding still needs direct evidence. |
| MIA-002 | UI-005, text-tool actions | Record label repetition as a separate tool-context operation, with Windows Alt+click and Mac Option+click. | S2. Capture source/target and repeat-state semantics before implementing; they are not specified by this short article. |
| MIA-003 | MO-024 through MO-029 | Alignment enablement must distinguish partial molecular selections from complete structures. | S3 states 25.5. The same policy in 26.1 and nonchemical-object mixtures remains unobserved. |
| MIA-004 | MV-030, MV-031, UI-001 | Maintain a separate Mac docking/workspace description instead of copying the Windows document-window arrangement. | S4. Exact modern Mac docking targets, floating behavior and persistence remain unobserved. |
| MIA-005 | UI-001, search/integration menus | Resolve the observed Professional desktop `Signals` menu versus the older `Search` menu before fixing the modern top-level menu model. | V1 at 10:01, 15:02 and 30:02. Presence/order is observed; exact build, complete contents and tier-dependent behavior are unknown. This does not authorize adding an account requirement to Bondline. |
| MIA-006 | MO-030, KM-046, context controls | Corroborate Alt+Shift+Ctrl+H and retain an accessible menu/context alternative for horizontal distribution. | S5. Actual context submenu path, platform precedence and behavior under shortcut interception need execution evidence. |
| MIA-007 | BioDraw palette / templates | Attach a direct Professional source to the View-menu BioDraw-template palette control, without importing the library. | S6. Corroboration of a role; no version-identified modern palette contents. |
| MIA-008 | Nicknames, labels, analysis | Keep visual abbreviation workarounds distinct from structures with a defined chemical graph; do not compute chemistry from display text alone. | S8's particular B(pin) workaround. A full, correctly defined nickname workflow still requires independent graph/chemistry tests. |

The legacy-biopolymer removal in S7 is a version boundary, not a new missing modern
toolbar. The existing R26-001 through R26-004 release rows already cover the 26.1 mixture,
attached-data selection and paste changes; this audit does not inflate the inventory by
duplicating those rows.

## Visually inspected support screenshots

| Source | Exact public image and SHA-256 | Observed / not established |
|---|---|---|
| S3 | [1844 by 597 alignment comparison](https://support.revvitysignals.com/hc/article_attachments/44882924087828), `3b5493d0f940fc2cd9bae251804d3dc4045c95ab1d598eb549a1d991f5fd7470` | Two Professional windows show partial-atom versus whole-structure selection and different alignment-control availability. A floating Object palette and two-column left palette are visible. No tooltip names, complete flyouts or build identifier are shown. |
| S4 | [Mac workspace image](https://support.revvitysignals.com/hc/article_attachments/30246952760852), `bb3654326148d2ea5c2eb88926a9035dd4574e8e62ff029795a0ef7f36affeb4` | Mac menu bar and palettes lie outside the central document window. Desktop filenames include a 23.0.1 installer/application, so neither the current article timestamp nor those filenames prove the running build. No docking action was replayed. |

The images were opened and visually inspected, not treated as OCR-only menu lists. Their
unlabelled glyphs remain unassigned until tooltips, captions or another explicit source
connects them to roles. Fresh vector assets must be authored independently.

## Controlled public desktop-video observations

REA CLI 6.9.0 ran with existing Node 26.12.1 and a selected Chrome executable, using an
owned temporary profile. REA reports Chromium 157.0.8103.78; its environment fields leave
host platform/architecture unknown. No existing user browser, credentials or account were
used. Optional YouTube cookies were rejected; no sign-in, subscription, trial, agreement
acceptance or account action occurred. Network body/header-value capture was disabled;
selected DOM, accessibility and screenshot capture remained enabled. Every completed or
failed replay reported `terminated-owned-process`.

The successful sample used a 1920 by 1080 viewport, en-US, UTC and light mode. It loaded
V1 at 10:00, rejected optional cookies, waited five seconds, then navigated to 15:00 and
30:00 and waited five seconds at each. After-step snapshots establish these bounded
advances; they do not establish that five full seconds decoded without buffering.

| Video timestamp actually visible | Observed screen | Screenshot SHA-256 |
|---|---|---|
| 10:00 to 10:01 | Professional title; white document, two-column left tools, upper typography/general controls; File, Edit, View, Object, Structure, Text, Curves, Colors, Signals, Add-ins, Window and Help menu labels. Windows is inferred from the recorded taskbar/window chrome; exact Windows version is not established. | `e2b2c66c8f3d9991e86db2c1ff602976cdc5c9f78d0cc7973e34db36f3fd5a3f` |
| 15:00 to 15:02 | Professional document containing molecular drawings with an endpoint highlighted. No complete creation or edit sequence was observed in this short slice. | `7a33342c47fb112fb99e888489d7ea34c4b6cb57b29e4fd8030598e8a4b0df54` |
| 30:00 to 30:02 | Professional document with annotations, highlighted molecular regions and two swatch grids at the right. Custom/Remove controls are visible; their exact palette roles and interaction sequence were not established. Numeric annotations are not chemistry oracles. | `fa87861f11de8f1b6e354b1ed0285ca21d254df8b0f81c668ff42097a4d530e5` |

Complete successful replay Evidence:
`ev_361cc07940b933574c120124af4f34e74e83fde9a5fa01b1ea6e91a2c6e284ab`.
The video predates the September 28 desktop 26.1 release announcement; that date alone
does not identify the recording's build. The document filename containing `Sept26` is
also not an application version. Do not mark these frames as a 26.1 screen specification.

Earlier probes remain explicitly bounded:

- The June 2026 [official recording](https://www.youtube.com/watch?v=dWVMOKbD3A4) exposed
  title/uploader, a 58:47 player, and a cookie overlay at 0:00. Evidence
  `ev_c74776bfacf00da243df896207dedbef4426ebc4704ce6e71f3c72ff84ffc240` does not prove playback.
- A September play attempt timed out behind that overlay. Evidence
  `ev_47ffad0c3d74efd6409f46a7eb131b5d65e6bedf15273eea312f48efc7c7c134`
  is incomplete, not an application-behavior observation.
- After rejecting cookies, a 20:00 September frame showed a Wikipedia page, with the
  player paused. Evidence
  `ev_0a3ba611d51918f7ab3c9064e3585e4c38ac5873001dc9cdc5e21a01363625dc`
  proves neither a ChemDraw screen nor motion. The later multi-sample replay above
  establishes actual bounded desktop playback without relying on that failed inference.

## Remaining evidence gaps and next executable work

1. **Promote the version-qualified gesture corrections first.** Add the S1/S2/S3 source
   records and candidate rows while preserving existing IDs. Future connected-drawing and
   label tests must distinguish tool, target, modifiers and selection extent. Do not wait
   for a complete visual inventory to improve headless graph/editing correctness.
2. **Identify the desktop build before extending the video-derived menu model.** Seek an
   About/version frame in V1 or a public 26.1 desktop demonstration. Then review continuous
   preference/tool-palette/menu segments with timestamps and before/after evidence. The
   sampled frames do not enumerate every menu, flyout, dialog or tooltip.
3. **Obtain separate version-identified 26.1 Mac and Windows sources.** The Mac support
   screenshot is not such a source. Resolve the Search/Signals difference, application-menu
   differences, default/detached palette arrangements and native shortcuts. The existing
   authorized-download-center login blocker remains; no entitlement was invented.
4. **Enumerate context menus by target and selection.** At minimum: atom, bond, whole
   structure, mixed selection, arrow, bracket, label, caption, group, table, HELM residue/
   sequence and empty canvas. Record enabled, disabled and absent items separately. The
   current 573 rows and the two support screenshots cannot close this matrix.
5. **Finish palette role mapping through labelled evidence.** Review every flyout and
   tooltip plus hover hotkeys, active tool state, modifier variants and detached/docked
   behavior. A recognizable glyph or a tool count is insufficient to assign a command.
6. **Resolve the existing contradictory key/label cases.** The checklist already lists
   potassium/sulfonyl `k`, legacy bond `4`, example-only `v`, Color/Colors and page-view
   label differences. A recent generic support article still uses older page-view labels;
   an updated timestamp does not settle the modern binding or menu spelling.

The [ACS 2025 webinar page](https://www.acs.org/acs-webinars/library/chemdraw.html) identifies
a vendor presenter and links a public Tips & Tricks playlist, but states its recording
is member-only and its slide deck unavailable. Indexed text of its linked LabNotes identifies
AI-generated summaries; they were not adopted as behavioral oracles. Direct acquisition
returned a 212-byte HTML challenge stub rather than a PDF; no local PDF rendering or
visual inspection is claimed. No membership, registration or gate bypass was attempted.
Public university listings advertising 2026
training materials are discovery leads, not verified modern guides.

Cloud and desktop versions must remain distinct: the official release archive contains
a **cloud 26.1 in January 2026**, whereas desktop 26.1 is the September release. A guide,
video or screenshot labelled only `26.1` needs product/tier/platform/build attribution
before becoming a desktop parity source. Nothing in this audit establishes complete
inventory coverage or full one-to-one screen/flow parity.
