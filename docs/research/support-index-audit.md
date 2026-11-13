# Desktop support-index audit

Reviewed 2026-11-13. **Title screening of the selected English support section is complete;
the Professional 26.1 desktop control inventory is not.** This continues the bounded
[modern inventory audit](modern-inventory-audit.md), reusing its first 200 titles and
completed video samples. No Bondline implementation or verification state changes here.

## Coverage and reproducible source boundary

The public [ChemDraw support section](https://support.revvitysignals.com/hc/en-us/sections/360010804431)
was obtained through its unauthenticated
[`articles.json` endpoint](https://support.revvitysignals.com/api/v2/help_center/en-us/sections/360010804431/articles.json?per_page=100&sort_by=updated_at&sort_order=desc&page=1),
with `per_page=100`, `sort_by=updated_at`, `sort_order=desc`, and pages 1–7.

- Pages 1–2 reuse the prior audit's saved responses; pages 3–7 are newly acquired.
- All seven responses report 699 articles. Pages 1–6 each contain 100; page 7 contains 99
  and has no next page. Concatenation contains **699 unique article IDs, no duplicates**.
- All 499 remaining titles on pages 3–7 were screened. This completes title screening of
  these 699 records, not body reading of 699 articles or coverage of other sections/locales.
- Text/markup of 20 selected articles was examined below: 19 from the newly screened pages
  and one 26.0 Mac article from page 1. Two selected articles provide their useful steps
  only as images; those images were not inspected in this continuation.
- A mechanical search of the remaining 499 titles plus HTML-stripped bodies for
  `\b26\.\d` returned zero matches. This is a search result, not proof that a modern guide
  does not exist elsewhere, nor a manual review of those bodies.

Exact saved response bytes, named `public-section-page<N>.json`, remain under ignored
`.cache/research/modern-inventory/`. The acquisition manifest for pages 3–7 is
`remaining-section-manifest.json`. These hashes identify complete raw section responses,
not separately fetched article responses.

| Page | Articles | Bytes | SHA-256 |
|---|---:|---:|---|
| 1, reused | 100 | 330207 | `571190b5b7b8a2757ff37c1064e7cafbcfbeae8a7964b6575bdf9103c9248140` |
| 2, reused | 100 | 259549 | `687ef70f4f935fd42155fb47a40b871154b04c622ee867bc3754878dbd056abc` |
| 3 | 100 | 207856 | `4efa747372eebf4d9de2e8a168608b2477d91426d4eb2319405b3ac16ff74aeb` |
| 4 | 100 | 195448 | `81ea5449a259d0ade0bf022715197853f6db85b01779c1e8cae5eccbbc3355f3` |
| 5 | 100 | 208269 | `33fe64467c4db9aad70abc63bed101d8cc6aa23ae3ef01465405356d43ee9a2d` |
| 6 | 100 | 206833 | `4e5cf0d76dc83af2c98ff964ad0b9fd32a2507b42ab0c9932d296d80ca3c0ce0` |
| 7 | 99 | 265833 | `b2d8e8b5fc50cfd85150e0192572c7432fcea25eb9cf9b791a03685960f7e161` |

## Selected body findings and version limits

The table paraphrases vendor statements; none is a replayed reference-application action.
An article's 2025/2026 update timestamp does not identify the described running build.
IDs and page numbers below locate the exact body within the hashed responses above.

| Article / page | Documented observation | Qualification and inventory consequence |
|---|---|---|
| [9130327027220: user-guide access](https://support.revvitysignals.com/hc/en-us/articles/9130327027220) / 5 | The guide is available with an entitled download package or through installed Help entries: Contents, Getting Started, Tutorials, and ChemDraw Basics. | No product build specified and no public full-guide download URL supplied. This identifies an access route, not access rights or 26.1 guide acquisition. The checklist has Contents but does not separately enumerate the other three Help entries. |
| [4408233546900: copy while dragging](https://support.revvitysignals.com/hc/en-us/articles/4408233546900) / 3 | Describes Ctrl in older versions and Alt in newer versions. | No cutoff, platform, tool, selection type, or constrained-drag rule. Conflicts with the existing G25 Ctrl-drag mapping; it cannot settle 26.1 by itself. |
| [5309897982484: hotkey 0](https://support.revvitysignals.com/hc/en-us/articles/5309897982484) / 5 | For **21.0.0.28**, the body specifies a normal C–C bond for `1` and a **75% longer** bond for `0` over the described ring hotspot. | Exact historical build, not 26.1. The title says twice the length while the body says 75% longer: retain that discrepancy rather than translating the title into a 2× geometry assertion. |
| [8745549205908: crowded bond extension](https://support.revvitysignals.com/hc/en-us/articles/8745549205908) / 5 | Describes `0` over a secondary carbon as a longer bond and `1` as regular length. | No build or numeric multiplier. Corroborates a contextual operation, not the modern length ratio or all hover targets. |
| [4408232988948: movement versus rotation](https://support.revvitysignals.com/hc/en-us/articles/4408232988948) / 7 | Describes Alt+arrow translation before 19.1 and rotation in 19.1/20.0/20.1; says a return of translation was expected in 21.0. | A future expectation is not evidence that 21.0 or 26.1 implemented it. Do not overwrite G25 Shift+arrow translation or Alt+arrow rotation rows using this historical support note. |
| [4408233187092: selection tolerance](https://support.revvitysignals.com/hc/en-us/articles/4408233187092) / 5 | Places Preferences under File on Windows and the application menu on Mac. Recommends increasing selection tolerance to at least five pixels for a reported double-click problem. | No build. The recovery recommendation is **not a declared default**. Hit-test tolerance, drag threshold, and the 25.5 atom-label gesture change are separate questions. |
| [5714724001172: crosshair spacing](https://support.revvitysignals.com/hc/en-us/articles/5714724001172) / 5 | Associates crosshair spacing with units and points to Preferences → General to change inches to centimetres. | No build or physical spacing value. Do not infer a grid interval or change document geometry merely from changing displayed units. |
| [7583996816532: default save format](https://support.revvitysignals.com/hc/en-us/articles/7583996816532) / 5 | Gives Preferences → Open/Save → Saving Files → Use Default Format and a format chooser. | No build, format list, persistence scope, or initial default established. Adds control-level corroboration to DP-004, not complete format support. |
| [4408233208084: enlarged Windows UI](https://support.revvitysignals.com/hc/en-us/articles/4408233208084) / 5 | Describes a large theme under Preferences → General for small Windows 10 menus/toolbars. | Windows 10 is named; application build is not. Exact dimensions, DPI behavior, assets, and modern theme names remain unknown. Bondline must use original branding/assets. |
| [4408239540884: secondary-display docking](https://support.revvitysignals.com/hc/en-us/articles/4408239540884) / 5 | Claims post-15 Mac windows/toolbars can move to another display, but toolbars dock only on the primary display. | Historical version boundary with no exact modern build. Useful follow-up to KB-MAC-DOCK; multi-display 26.1 runtime behavior remains unobserved. |
| [4408239554324: separate document windows](https://support.revvitysignals.com/hc/en-us/articles/4408239554324) / 7 | Describes repeatedly launching from the Windows Start menu, then using each instance's File menu to open documents. | Windows procedure; no build. Does not establish modern tab/window policy, single-instance routing, Mac behavior, or cross-document save/close semantics. |
| [4408233761428: guide function key](https://support.revvitysignals.com/hc/en-us/articles/4408233761428) / 7 | Recommends Fn+F1 when the keyboard requires its function modifier to open the guide. | No application build or OS specified. Hardware Fn handling is not necessarily an additional application shortcut. |
| [4408233176340: Mac hotkey configuration](https://support.revvitysignals.com/hc/en-us/articles/4408233176340) / 7 | Separates installed hotkey-file and user-override locations across 16–21, post-22.2, and 23-and-later Mac versions. | Public location guidance is not a 26.1 key inventory. No installed proprietary configuration file was obtained, inspected, or copied. |
| [4408233119508: missing Add-in Manager](https://support.revvitysignals.com/hc/en-us/articles/4408233119508) / 5 | Says the Professional suite from 18.x does not include that manager and discusses another offering. | Tier/version claim needs a current tier chart or identified 26.1 screen before deciding whether the menu entry belongs in Professional. An add-in developer guide is a different control. |
| [53135405735444: save failure on Mac](https://support.revvitysignals.com/hc/en-us/articles/53135405735444) / 1 | Explicitly concerns **Signals ChemDraw 26.0 on Mac**, new-document/Save As failure, and an integration workaround. | This is qualified 26.0 evidence, but not Professional 26.1 layout/behavior. Do not reproduce a vendor defect or add a tenant/account requirement to offline Bondline. No tenant was accessed. |

Five additional selected text/markup bodies complete the count of 20:

- [4408239581204, document/object settings](https://support.revvitysignals.com/hc/en-us/articles/4408239581204),
  page 5: unversioned corroboration of existing settings roles, without new modern control evidence.
- [4408233143188, toolbar-setting persistence](https://support.revvitysignals.com/hc/en-us/articles/4408233143188),
  page 5: legacy Windows registry/admin workaround; not a modern implementation requirement.
- [9363947566484, Markush manual](https://support.revvitysignals.com/hc/en-us/articles/9363947566484),
  page 5: points to an attachment. The attachment was not acquired/read in this task; its
  existence does not prove a current Professional palette or license tier.
- [6057524576788, missing typography controls](https://support.revvitysignals.com/hc/en-us/articles/6057524576788),
  page 5, and [4569583406612, reaction shortcut](https://support.revvitysignals.com/hc/en-us/articles/4569583406612),
  page 6: relevant steps are image-based. No action sequence, modifier, or modern build is
  inferred from their titles or sparse extracted text.

## Concrete checklist candidates and unresolved comparisons

These began as research candidates, not implemented controls. Root review promoted only
the three provisional Help roles as MX-031–033, with explicit version uncertainty, and
qualified existing copy-drag notes. The former573-control baseline now has576 rows; no
implementation or complete26.1 inventory is inferred from that count. Preserve existing IDs.

| Candidate | Existing anchor | Next decisive evidence / acceptance target |
|---|---|---|
| SIA-001 | MX-021 and Help controls | Add provisional separate Getting Started, Tutorials, and Basics navigation roles, or explicitly resolve their absence in an identified 26.1 Help menu. Confirm destination/section and platform placement. |
| SIA-002 | KP-015, KP-016 | On an identified build, compare Ctrl/Alt/Option copy-drag with plain drag and constrained drag; record tool, target, selection, source retention, and resulting document/history. No automatic modifier substitution. |
| SIA-003 | Bond hotkeys, connected-drawing UI-005 | Test `0` and `1` over a secondary carbon/ring hotspot, an endpoint, a bond, and empty canvas. Record exact length ratio and attachment; source title/body disagreement is a test question, not a chosen answer. |
| SIA-004 | MF-014, MF-015, DP-003, MV-020 | Confirm separate Windows/Mac Preferences entry, units, tolerance default/range, and crosshair settings on 26.1. Measure hit-test and unit behavior independently; five pixels is only a support recommendation. |
| SIA-005 | DP-004; MF-004, MF-005 | Enumerate current format choices/default-save controls and their scope; keep unsupported import/export formats explicit. The delivered CDXML save workflow does not implement preference parity. |
| SIA-006 | MV-030, MV-031; UI-001 | Test Mac primary/secondary display docking and layout persistence, with build, monitor topology, scale, and restart evidence. Historical instructions cannot verify this. |
| SIA-007 | MF-002, MF-007, Window menu | Capture same-process versus separate-process document routing, dirty-close prompts, and cross-window activation on both reference platforms. Bondline's delivered single-document Mac workflow remains narrower. |
| SIA-008 | Add-ins tier inventory | Resolve whether Add-in Manager is absent from Professional 26.1 and distinguish it from Add-in Developer Guide and Signals integrations. Do not infer a control solely from suite-wide documentation. |

## Public video build-identification search

The prior audit's completed 10:01, 15:02, and 30:02 desktop samples were not repeated.
Focused public searches for a Professional desktop 26.1 About/build frame, an official
26.0/26.1 desktop guide, and a newer official desktop webinar did not identify a new,
version-proven recording. The [official Connect index](https://revvitysignals.com/chemdraw-connect)
still provides the September 23 session as the relevant drawing-workflow lead. Search
discovery is not an exhaustive video catalog or evidence that no newer recording exists.

The already captured public YouTube player DOM exposed storyboard metadata. Bounded
requests for thumbnail sheets returned HTTP 403; no usable thumbnails were acquired and
no access restriction was bypassed. Signed request URLs remain in ignored research data,
not in this report.

REA CLI 6.9.0, existing Node 26.12.1, and an owned temporary Chrome profile were used for
new timestamp probes of the [same public September recording](https://www.youtube.com/watch?v=iv3ciKZz9rg).
REA reports Chromium 157.0.8103.78 and leaves host platform/architecture unknown. The
browser viewport was 1920 by 1080. Optional YouTube cookies were rejected. No existing user
browser, account, sign-in, subscription, activation, agreement acceptance, or CUA session
was used. Selected DOM, accessibility and screenshot evidence was captured; network
body/header capture was not enabled.

The first probe sought 5:00, 7:00 and 55:30 and waited five seconds at each, but showed
only the video poster. The requested clock positions were **not decoded desktop-frame
evidence**. Record `ev_50350539d89e97b3679426ff0dbdaef3047719c2aafcda5f2331991b7e726f38`
is retained as this bounded failure. One corrected replay used the observed player Play
control after each seek; the completed prior-audit samples were not repeated.

| Actual visible timestamp | Visually inspected result | Screenshot SHA-256 |
|---|---|---|
| 5:00, paused | Professional title and blank desktop document; no About dialog or build string. This frame does not prove playback advance. | `68d0d48ea7825f5e5e50c63cc10fe5e6f656c18fd8e1f5452ddc1e31b83d7e22` |
| 7:00 → 7:04 | Professional window with a native French Page Setup dialog: paper size, orientation and margins controls are visible. This corroborates MF-009 control roles, not a complete print/setup flow, default values, or a particular application build. | `e600ac942e3e7ef0f43d18b1ce1a7c1159a890afe9590531d721be105d33deb9` |
| 55:30 → 55:34 | Professional window with annotated structures, two tabular regions and color swatches; no About/build identifier. The depiction does not establish the commands or chemistry that produced it. | `66c8e9891f9cfdceaf371cf7ec17d83fb1dda9844a8a0ed24717ba77ceca0e9f` |

The corrected evidence record is
`ev_274019c35d03d710ee74e08dbe2b2de1c25561c67f8b0f12e7c930c4c46a1772`.
The two advancing clock samples establish bounded playback, not review of the intervening
video or all 56:50. No About/build identification was found in these samples. As established
in the prior audit, the webinar predates the desktop 26.1 release announcement; its
publication date does not identify the recorded executable. Windows is inferred from
recorded taskbar/window chrome, not from the REA host.

Both replays report `cleanup: terminated-owned-process`, including the poster-only probe.
Their inputs, complete evidence, summaries, stderr and decoded images remain ignored under
`target/research/modern-inventory/youtube-september2026-about-*`. No live REA/browser or
execution session remains from this task.

## Next executable inventory work

1. Obtain a **version/build-identified Professional 26.1 Windows and Mac** public guide,
   demonstration, or already-authorized runtime. Capture About/build plus menu/palette
   state in the same session. Do not create an account, accept terms, or assume entitlement.
2. Resolve SIA-002 and SIA-003 before copying legacy keymaps into connected drawing and
   selection. Existing public evidence already establishes why tool/target/platform must
   be parameters of acceptance tests.
3. Record each menu/flyout/context target continuously with enablement and before/after
   behavior. Prioritize atom, bond, full/partial/mixed selection, empty canvas, text,
   arrows, brackets, and HELM. A title-only index supplies none of these matrices.
4. Promote qualified corroboration into existing source/row notes and keep the Help-entry
   candidates provisional until version-qualified. Do not inflate the existing 573 rows
   with duplicate support articles or interpret a completed title scan as complete M0.

No reference screenshots, proprietary templates, fonts, help paragraphs, installed hotkey
files, or binary assets were added to the repository. No PDF, installer, or reference
desktop runtime was acquired in this continuation. Product tests are unchanged: this is
documentation research, not new functionality.

## Audit validation

A local read-only script checked all seven response hashes and byte lengths, article
counts/uniqueness, the last-page terminator, the 499-record version search, the 20 selected
article links and their 19/1 page split, local links/checklist IDs, all three new decoded
image hashes, and both replay cleanup records. All assertions passed. A whitespace check
of this new file emitted no errors. Rust compilation/tests were not repeated for this
documentation-only change; they would not validate the unobserved reference behaviors.
