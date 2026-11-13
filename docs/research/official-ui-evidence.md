# Official desktop UI evidence

Reviewed **2026-11-13**. Target: independently implemented Professional **26.1 desktop** behavior. This is a bounded evidence inventory, not a complete screen/button inventory or a claim that Bondline implements these workflows. No reference desktop executable was run. Guide screenshots were rendered and visually inspected. The initial media discovery below did not play videos or demos; subsequent controlled browser observations are recorded separately.

## Source ledger and reproducible acquisition

| ID | Public source | Version/platform actually established | Evidence obtained |
|---|---|---|---|
| P | [Official product page](https://revvitysignals.com/products/research/chemdraw) | Current hybrid product family; no single desktop build | Public page and outbound demo, chart and download links. Marketing combines desktop and cloud capabilities. |
| G25 | [Vendor-authored guide, publicly hosted by Beloit](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf) | Filename says 25; PDF creation metadata 2025-05-22; Windows-style screenshots do not identify their build | Existing pinned 439-page PDF; visually reviewed one-based PDF pages 12–15, 40, 42, 44, 218, 223, 423. Printed page = one-based PDF page minus one, or zero-based PDF index. It is not a 26.1 visual specification. |
| T25 | [Official tier chart](https://assets.revvitysignals.com/m/6e4aae70c40d28af/original/Which-ChemDraw-is-right-for-me.pdf) | Footer June 2025; three product tiers | Downloaded exact bytes; rendered and visually checked the three columns. Flattened PDF text loses tier attribution. |
| R261 | [Desktop 26.1 release announcement](https://support.revvitysignals.com/hc/en-us/articles/54009539308052-Release-Announcement-ChemDraw-Desktop-26-1) and [public article JSON](https://support.revvitysignals.com/api/v2/help_center/en-us/articles/54009539308052.json) | Desktop 26.1; article updated 2026-09-28 | Read article and cached JSON. Confirms three chemistry/editing changes below; cloud changes have a separate heading. |
| A261 | [Official release archive](https://revvitysignals.com/signals-chemdraw-whats-new-archive1) | Desktop 26.1, September 2026 section; distinct cloud release sections | Three desktop image links discovered, but image downloads returned HTTP 403. Their content was **not** visually reviewed. |
| H260 | [Official hardware/software guide](https://assets.revvitysignals.com/asset/455df079-8d90-4706-96b9-d9ea28b1a7ce/ChemDraw-Desktop-Requirements-Hardware-Software-Guide.pdf) | Desktop 26.0; updated 2026-04-23; Windows/macOS | Downloaded five-page PDF. Useful platform boundary, not 26.1 layout evidence. |

The inherited G25 cache remains `.cache/research/professional-25-guide.pdf`. This investigation's downloads, derived page images, exact URLs, sizes, failures and hashes are recorded in `.cache/research/official-ui/evidence-manifest.json`. That disposable cache is **research only**: vendor copyrights remain; G25 also carries a confidentiality footer despite its public availability. Use reviewed media to establish layout, control roles and icon appearance, then draw original redistributable vector artwork. Do not transplant or distribute vendor files without applicable rights and the shipped-asset policy.

Pinned source SHA-256 values:

| Cache file | SHA-256 |
|---|---|
| `../professional-25-guide.pdf` | `3f1ebc429df07efdd47b6218a148dc346a215043f60bf89ee9a1cb90629bd5b3` |
| `feature-chart-2025.pdf` | `de3c9f90b18c00a6f3c4f545190576e3188c142aa3e9cbdca5fea6ae1f46dba9` |
| `hardware-26.0.pdf` | `0a1f45eafcd904bf012d1e659787958102d0f6400941f02ab2675f8c52a3bb17` |
| `release-26.1.json` | `1359d29edb39b7f23c2e9c3341a38f4bc8ca07273d735e0837db8d2918a5b739` |

Reproduce visual inspection with `pdftoppm -f 13 -l 13 -singlefile -scale-to 1800 -png <G25.pdf> <output-prefix>`; chart rendering used scale 2400. Hashes were checked locally, not inferred from URLs.

## Desktop screens and workflows supported by evidence

| Surface/flow | Concrete documented behavior | Locator and limits |
|---|---|---|
| Main workspace | Menu row; left two-column drawing palette; grouped actions, zoom, add-ins and typography controls above a white document page; gray surround, scrollbars and bottom status bar. | G25 printed 12 / PDF 13, visually reviewed. Full menu/control counts, icon variants and pixels are not established for 26.1 or macOS. |
| Tool palettes | Some tools expose nested palettes; toolbars can detach, float, dock and be shown/hidden. Opening documents and choosing an active document are separate actions. | G25 printed 12–14 / PDF 13–15. Documented behavior, not replayed interaction timing. |
| Document and object styles | Document-wide drawing settings are separate from per-selection overrides. Drawing dialog groups bond length, spacing mode, stroke widths, margins, hash size, chain angle, and atom/bond indicators. Categories also cover page layout, captions, atom/property labels, biopolymer/reaction display and colors. | G25 printed 40–43 / PDF 41–44. Drawing and label dialog screenshots visually reviewed. Values pictured are examples, not independently validated default values. |
| Label appearance | Atom-label typography has font/style/size, formula baseline and line-spacing controls, plus terminal-carbon and implicit-hydrogen display choices. | G25 printed 43 / PDF 44, visually reviewed. Original fonts are not licensed assets for Bondline. |
| First connected drawing | Activate a solid-bond tool; empty-canvas click creates a bond; endpoint click extends it. Fixed-length/angle options constrain geometry; the tutorial's chain angle is 120 degrees. Hovering and entering a bond-order number changes order. Text entry at an endpoint changes the atom label; Escape finishes. Text entry away from atoms creates a caption. | G25 printed 422–424 / PDF 423–425; PDF 423 visually reviewed, following pages read. This is stronger acceptance evidence than isolated, disconnected bonds. Author an original test scene; do not copy tutorial artwork. |
| Ring construction | Place a ring, fuse another at a bond, and adjust local bond orders by hover hotkeys. A modifier changes the ring's resonance arrangement. | G25 printed 423–424 / PDF 424–425. Exact pointer tolerance and modifier timing remain unobserved. |
| Analysis | Live calculations use the current selection or the whole drawing when none is selected. Output includes formula, exact/average mass, isotope distribution and elemental composition. Results can become an editable caption that updates with structure edits; individual fields can be hidden. | G25 printed 221–222 / PDF 222–223; PDF 223 visually reviewed. Requires independent chemistry oracles, not screenshot-derived numeric correctness. |
| Naming preferences | Naming can create a caption and optionally display IUPAC atom numbers; the preference is shown among building/display settings. | G25 printed 217 / PDF 218, visually reviewed. A UI checkbox alone does not establish naming correctness. |
| Current 26.1 deltas | Mixture names account for fractional component ratios; selecting an attached data label highlights associated atoms; the paste submenu separates CDXML text from MDL text, retaining the existing shortcut for CDXML. | R261 and A261. No shortcut key value is asserted from the announcement. These are current-version requirements, not proof of the entire old layout persisting. |

The A261 image links are [mixture naming](https://revvitysignals.com/sites/default/files/styles/coh_x_large/public/media-api/task-1218957145203727-1790628028124-0-png.png?itok=O6tWBD9o), [data-label selection](https://revvitysignals.com/sites/default/files/styles/coh_x_large/public/media-api/task-1218957145203727-1790628028760-1-png.png?itok=26V-pS4W), and [paste submenu](https://revvitysignals.com/sites/default/files/styles/coh_x_large/public/media-api/task-1218957145203727-1790628029273-2-png.png?itok=4YWfqo5H). HTTP 403 prevented pixel inspection; no image hashes are claimed.

## Product-tier boundary

T25's Professional column documents naming in both directions, stereo handling, NMR prediction, advanced biopolymer/HELM and sequence interchange, style/highlight features, and 3D/perspective operations. It labels some property calculations Windows-only. Its Signals column adds the Properties/Query/Guided Stereo panels, 3MF-related capabilities, and cloud collections/search/admin workflows. **Do not infer Professional's screen arrangement from Signals demos or put those cloud panels into the desktop inventory solely because they appear on the product page.** The June 2025 chart cannot settle subsequent tier changes.

## Public demos, video and download entry points

| Resource | Actual observation; missing evidence |
|---|---|
| [Official demo landing](https://revvitysignals.com/chemdraw-interactive-demo-playlist), [playlist player](https://demos.revvitysignals.com/player/playlist?playlistId=c073fc30-dec5-4cf2-acbf-a3c5723b1997) | Public HTML embeds a Walnut player; cached entry/player HTML, not executed demo steps. No desktop version or timestamps established. |
| [Signals capabilities overview](https://revvitysignals.com/demo/signals-chemdraw-capabilities-overview), [stereochemistry demo](https://revvitysignals.com/demo/signals-chemdraw-fast-and-accurate-stereochemistry-assignment) | Embed [Consensus overview](https://play.goconsensus.com/ub31e2a07) and [Consensus stereo](https://play.goconsensus.com/a39bb2426) players. Playback yielded no extractable content. No flows claimed as observed. |
| [How to Use Signals ChemDraw](https://www.youtube.com/watch?v=GLrFIgxqmCs) | Search metadata identifies official Revvity Signals uploader and 2024-02-26 publication. Playback unavailable; no watched timestamps, app version or platform established. |
| [Official training announcement](https://support.revvitysignals.com/hc/en-us/articles/37353746715284-New-video-on-Chemical-Drawings-for-Signals-End-Users-now-available) and [linked video](https://www.youtube.com/watch?v=vmgvfD8qNd8) | Announcement dated 2025-05-09 explicitly concerns the JavaScript drawing interface in Signals Notebook. YouTube fetch throttled. It is not desktop 26.1 evidence. |
| [Official download center](https://revvitysignals.flexnetoperations.com/) | Reached from P's download link. Read-only response contains a JavaScript-disabled continuation form; no installer URL or file was exposed. No form submitted, account created, agreement accepted or installer downloaded/executed. R261 separately directs eligible customers to this center. |

Direct HTTP downloads of the public 2024/2025 tips PDFs and demo hero screenshot also returned 403; their URLs/failures remain in the cache manifest. No failure was treated as evidence of absent functionality.

## Subsequent controlled browser observations

REA CLI **6.9.0**, existing Node **26.12.1**, selected local Chrome executable; REA reports
Chromium **157.0.8103.78**. Each scenario launched its own empty temporary profile at
1280×720, en-US, UTC, light theme. No user browser profile or credentials were used. No
system runtime/MCP registration was changed. Every result below reported
`cleanup: terminated-owned-process`. Host platform/architecture are unknown in REA's
Evidence despite the operator running on macOS; do not silently fill those fields.
Complete Evidence bundles, input JSON, summaries and screenshots are ignored research
outputs under `target/research/`. Only identifiers and observations are recorded here.

| Observation | Exact Evidence / screenshot SHA-256 | Established boundary |
|---|---|---|
| Navigate the official download center, wait for the body and settle two seconds | `ev_e44244ff870147f612a4d94ae97c6ccec69b40b9feb7f3a817a8367d28340b87`; image `024c0129d934b51c22714f75b68fb1c591b7a69b99c13077f580cc867480fdbe` | Redirected to `login.revvitysignals.com/u/login`. Visually inspected login fields and a Download Center login instruction. No account action, credentials, agreement or installer access. This supersedes the initial plain-HTTP continuation-form finding, without claiming authenticated availability. |
| Open the public playlist, wait for its body to attach, settle four seconds | `ev_6887ba4d1422a9ddc2ae3b2c956d23b4a6a63a57573d8da31f1414fa1f495919`; image `b2be92cff45cd4f4eb810aac9ff6acc2f890b9fbadfdccf75328865fcba71d25` | Visually inspected two selected web-demo cards and the start action. The wrapper AX tree exposes only an iframe; its DOM identifies the public `app.teamwalnut.com/player/playlist/` iframe URL. No desktop application or chemical editing operation observed. |
| Navigate that observed public iframe URL directly, activate the start button, wait ten seconds and for the loading message to disappear | `ev_2dc2d37e396c282a5ed5c77de242e0b9e5225667c2c9899a72cb1589fc7d69de`; final image `8bb52435015c86bda815e883a2cf95ddaa9d579cacdebfd0e7eeb539ccb6cbb1` | All declared actions completed. Visually inspected the first guided promotional slide, its Next action and the playlist sidebar. The guide describes a web application, not desktop 26.1. Inner-frame controls are not in the outer AX tree. No next slide, drawing, save, reopen or stereochemistry operation was exercised. |

The first playlist probe waited for a visible body and timed out because the wrapper's
absolutely positioned iframe leaves its body without a visible box; Evidence
`ev_8f23d9781eb29ee95fa70d7c8ec7017b20690d1ba3b00fbf807285e0ff2227b3` is incomplete.
Changing the wait to **attached** produced the entry capture above. A shorter three-second
post-start probe, `ev_0515496d679d93efdb877dc4101f9badc0f3e5ed29397e322709d7b516a6f8b2`,
captured a loading overlay. The final probe waited for that observed overlay to disappear.
The start captures still report incomplete **network events**, because some requests were
unfinished when observation ended; complete screenshots/DOM do not imply complete traffic.
No response bodies, request bodies or header values were requested. Browser playback here
establishes only those specific entry states, never reference chemistry correctness.

## Implementation order and unresolved evidence

1. Preserve the shared-registry boundary while implementing connected endpoint extension, bond-order changes, atom labels and selection/history. Use G25's first drawing workflow as a behavioral source; validate chemistry and exact undo independently.
2. Build the documented desktop shell and independently drawn tool glyphs around implemented commands. Validate actual native focus, hover hotkeys, palette activation and accessibility; keep undispatched controls visibly unavailable.
3. Add source-backed editable open/save before expanding the palette. Exercise failed loads, unknown-data retention, history branches and edited export, then compare with an independent parser/consumer.
4. Add document/object style dialogs and live selection-aware analysis with scientific reference vectors. Naming, stereo and biopolymer workflows remain explicit later work; their presence in marketing is not an implementation oracle.
5. Obtain a version-identified 26.1 Windows **and** macOS workspace capture and public desktop tutorial playback. Verify menu nesting, full tool variants, shortcut precedence, drag tolerances, multi-document behavior and current dialog changes before claiming screen-for-screen parity.

Unresolved: complete 26.1 menu/tool/hotkey coverage; exact platform layouts; desktop video playback/timestamps; installer availability beyond the observed login; newer tier changes; and all chemical editing, timing, clipboard and native integration behavior. This report does not close the existing parity inventory gap or establish any Bondline feature as verified.
