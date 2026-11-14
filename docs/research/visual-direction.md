# Bondline visual direction

Reviewed 2026-11-14. This is screenshot-led design research and an original proposal,
not delivered Bondline UI or proof of reference workflows. No application was launched,
no artifact downloaded and no reference code, font, logo or icon was imported.

## Exact evidence

Existing local caches retain PhotoCraft commit
[`0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e`](https://github.com/storytold/photocraft/commit/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e)
(2026-11-13 09:33:13 UTC) and VoiceStudio commit
[`9a0ebd9fbf9d85bce6e58541677ab311f290eb60`](https://github.com/debpalash/VoiceStudio/commit/9a0ebd9fbf9d85bce6e58541677ab311f290eb60)
(2026-11-13 11:07:20 UTC). Read-only GitHub connector responses reconfirmed these commit
identities/dates. These are pinned cached revisions, not a claim to have refreshed main.
All five images below were actually viewed with `view_image`; their full SHA-256 values,
local paths and dimensions are recorded in ignored `target/visual-direction/evidence.json`.

| ID / pinned image | Visual observation |
|---|---|
| [PC-DARK: working canvas](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/docs/images/photocraft-demo.jpg) | Dark neutral chrome; compact menu and context rows; narrow grouped two-column tool rail; dominant central document; properties above layers on the right; bottom status. Blue selected-tool outline and subdued unselected glyphs. |
| [PC-VECTOR: selected geometry](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/docs/images/photocraft-vector.jpg) | Same stable shell; selected path anchors use a small accent; numeric geometry and appearance fields share an aligned inspector. Context hint explains the current tool without covering the artwork. |
| [PC-LIGHT: export](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/docs/images/photocraft-export-light.jpg) | Cool near-white panels differ from the gray pasteboard; restrained purple selected state; separated inspector groups; compact labeled inputs and a clearly stronger primary dialog action. Rounded containers remain secondary to the document. |
| [VS-CLONE: task workspace](https://github.com/debpalash/VoiceStudio/blob/9a0ebd9fbf9d85bce6e58541677ab311f290eb60/docs/media/electron/voice-cloning.png) | Within the app frame: narrow navigation, a secondary list, a large work area and a distinct right inspector. Headings, muted guidance and grouped controls establish hierarchy; one blue primary action carries most emphasis. |
| [VS-HOME: overview](https://github.com/debpalash/VoiceStudio/blob/9a0ebd9fbf9d85bce6e58541677ab311f290eb60/docs/media/electron/overview.png) | Within the frame: three prominent task cards above smaller secondary cards, restrained category accents and ample section spacing. Suitable evidence for onboarding hierarchy, not a chemical editing canvas. |

The PhotoCraft images are 2400×1500. Its [README](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/README.md)
says its gallery is offscreen app output; [ATTRIBUTION](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/ATTRIBUTION.md)
identifies the paintings as public domain. Those are upstream provenance claims, not
our execution of PhotoCraft. The screenshots alone prove no editing or export workflow.

VoiceStudio's [media notes](https://github.com/debpalash/VoiceStudio/blob/9a0ebd9fbf9d85bce6e58541677ab311f290eb60/docs/media/electron/README.md)
and [capture manifest](https://github.com/debpalash/VoiceStudio/blob/9a0ebd9fbf9d85bce6e58541677ab311f290eb60/docs/media/electron/capture-manifest.json)
identify a browser-only demo at source `471c95a27de817b3c329655ee81407125f830409`, version0.5.7,
captured 2026-10-29 05:30:59.158 UTC. The 2880×2000 images frame a 1360×850 app viewport at
scale2. The waveform backdrop, outer frame, branding and scene captions are presentation
artwork. They establish neither current native-window pixels nor live model operation.
The notes identify Inter; its exact font family is not inferred from visual similarity.

## Proposed Bondline workspace

Use PhotoCraft's document-first hierarchy as the primary reference, with VoiceStudio's
clear sectioning. Start with a cool light shell around white chemical paper. Keep the
paper and molecular ink independent from shell colors and preserve every approved
chemical-rendering reference. A later dark shell may surround the same white paper.

| Area | Original starting policy, in egui logical units |
|---|---|
| Top | Compact menu row, document title/unsaved indicator, then a tool-context row. Keep save/open actions discoverable and group current Single/Double/Triple controls together. |
| Left | About64 units for a two-column procedural tool rail; 28–32-unit targets, small separators, visible selected fill/outline plus accessible labels. Show functional tools without inventing flyouts or future behavior. |
| Center | The largest region: white page on a neutral pasteboard, with a subtle page edge. Stable page placement and hit rules take precedence over decorative cards or animation. |
| Right | An optional approximately248-unit inspector, with short section headings and aligned label/value rows for actual document/source/selection information. A preview-only collection must remain visibly read-only. |
| Bottom | A compact status row for actual page/zoom/tool guidance and current file/job/error state. Avoid duplicating a large diagnostic panel around the page. |

Proposed original tokens, chosen here rather than sampled from a reference: shell
`#F3F5F7`, raised panel/paper `#FFFFFF`, pasteboard `#E6EBEF`, divider `#D8DEE5`, text
`#1F2933`, secondary text `#52616F`, selected accent `#0B766E`, selected fill `#DFF4F0`.
Use the accent for interaction state; chemical meaning must not depend on that color.
Use one restrained border/4–6-unit radius for UI groups and no gradients over the paper.
These are design candidates, not measured reference defaults or an accessibility pass.

Use Bondline's existing permitted typography: approximately14-unit body text, 12–13-unit
secondary labels and stronger14–16-unit group headings; align numbers and retain explicit
units. Do not shrink information just to fit another panel. Chemical text, subscripts,
font metrics and readable-preview guards retain their independent rendering contract.
No font file or numeric font is taken from either reference.

## Applying the direction without overstating capability

The next actual app view should show a real original supported molecule, the command-backed
Single/Double/Triple controls, exact file state and the current paper. Use existing
`Engine::controls()` labels/enabled state and preserve tool epoch, history, source-loss,
foreground, DPI and cancellation guards. Styling does not add editor or format authority.
Avoid presenting analysis, template, ring or field-edit controls as working merely to fill
space. Keep unsupported/read-only states useful and visible; preserve their errors.

Borrow the README presentation pattern separately: original banner, a large actual-app
capture, brief feature/status captions, then build, license and contribution links. The
existing Bondline banners and real captures remain the basis. A gallery picture must be
identified as native or offscreen and show the actual revision; a design proposal is not
an app screenshot. No waveform backdrop, contributor portrait, artwork or reference
product branding belongs in Bondline's editing workspace.

PhotoCraft's code is MIT OR Apache-2.0, but its [asset attribution](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/ATTRIBUTION.md)
separately lists OFL fonts and ISC icons, outside Bondline's allowlist. VoiceStudio's
[license](https://github.com/debpalash/VoiceStudio/blob/9a0ebd9fbf9d85bce6e58541677ab311f290eb60/LICENSE)
is AGPL. This work adopts general visual patterns only; all product glyphs, branding,
layout implementation and shipped assets remain original or already approved.

This pass changes no product code. It does not verify responsive layout, contrast,
keyboard focus, tooltips, native menus or a dark theme. The next implementation must
produce actual Bondline pixels and retain layout/input tests; native inspection remains
deferred while the user is using the computer. Reference screenshots remain ignored.

## Independent actual-frame gate

Two new tests in [workspace_presentation_tests.rs](../../crates/ui/src/workspace_presentation_tests.rs)
declare the shell before implementation. Both use the real registry to set420x280pt paper
and create Single/Double/Triple pairs:6 atoms,3 logical bonds,3 fragments. Actual egui
paint and AccessKit output must place the three registered accessible tools left of the
canvas and a Document inspector with truthful counts/dimensions to its right, at1280x900
and760x520. Tests preserve the exact gray196 paper border and one-unit white inset; they
require visible nonoverlapping controls and no invented extra rail buttons. An actual
Double button click must change the shared selection while preserving the complete
authored state/content/history and exact chemical paint geometry. This is headless frame
acceptance, not a native window, contrast audit or new approved image reference.

The first actual run, handle60381, compiled and exited101 with both intended failures:
current canvas bounds were`[[0,33],[1280,878]]`, consuming the whole width without the
left rail. Literal paper checks and the first case's graph counts passed before this
assertion. Right-inspector, minimum-window and click assertions had not yet executed.
Command:`cargo test --locked -p bondline-ui --lib workspace_presentation_tests:: -- --nocapture`
with the separate absolute target, incremental/debug0 and existing pinned Python.
`target/workspace-presentation/baseline-red.log` is4180bytes, SHA-256
`bb1948c91f79dbb863ea2b4aa33fe3fe0e3d8a8c062d4052920b9003924c8684`.
The pre-implementation test is10846bytes, SHA-256
`3114a82470cf70842c1c0a7081c58ee033df8695f7f801fc476cd69818c00c00`;
its exact source and the received baseline identity remain in the ignored RED receipt.
No presentation GREEN or native execution is claimed at this checkpoint.

### Actual styled frame result

Root supplied the real styled UI. Its first attempt (21748, exit101) reserved76 units
because of grid minimum width; Root corrected that to the declared64 without changing
tests. Attempts97892/34745 passed the first layout test, then exposed a test locator
mistake: matching an accessible name alone could select the hover tooltip's Label rather
than its Button. The locator now requires both the existing name and Button role; every
input, role/bounds assertion, layout tolerance and state/paint equality remains intact.
Replacing only that predicate reconstructs the original3114a824 source exactly. Original
source and all three failed Root logs remain retained; they are not GREEN claims.

Independent handle96999 first passed both tests; after formatting the predicate,52721
again passed. All-target UI clippy16697 passed with warnings denied. Workspace fmt11495
found one formatting-only issue in Root's selected-tool text; the supplied correction
reconstructs from the preceding UI source by that single multiline formatting change.
Final UI SHA-256 is`540d4e0893ae438a0140480c098418b6511da19d8bb3deb3db170058659693da`.
On this exact source, final handle22936 passed both tests (zero failures/ignored), and
workspace rustfmt passed. Corrected test SHA-256 is
`0994a92f350e1c4de5a77ae24456da2d166998b4133a87d15fc9bc303b03a795`.
Both declared window sizes, visible tools/inspector/counts, actual Double activation,
unchanged authored state/history/content and exact chemical paint geometry now execute.
This is headless egui evidence only. No native input, new approved golden, reference-pixel
parity, hosted run or full-workspace pass is claimed by this independent test owner.

## Integrated final presentation

Root subsequently implements the original light workspace in the actual app. Final UI
source98d0dd17…5bca2 uses explicit2button rows, a48header/64rail/248inspector and24status.
Guidance/errors live in the inspector so paper geometry stays stable. The unchanged two
presentation cases pass again independently; broader old attachment/drawing suites retain
their expectations. [Root verification](../verification-workspace-presentation.md) records
actual native pixels, finite Double selection/File-picker/Cancel checks, failed native
drags and exact remaining scope. [Offscreen capture](../verification-visual-capture.md)
and native README imagery are separate artifacts, never new approved chemical goldens.
The proposed tokens above remain historical candidates; final code is the actual policy.
