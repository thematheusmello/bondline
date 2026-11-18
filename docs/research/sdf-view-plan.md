# SDF view navigation — bounded proposal

Research only, 2026-11-15. **The API below is proposed, not approved or implemented.**
No product/test/Cargo/Git/native changes or new runtime evidence accompany this plan.

## Evidence and current constraint

The pinned [vendor guide 25](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf)
(17,721,447B, SHA256 `3f1ebc429df07efdd47b6218a148dc346a215043f60bf89ee9a1cb90629bd5b3`)
documents appearance-only zoom, 1–999 percentage and General-toolbar roles on printed 26;
printed 79 lists F5–F8. It establishes neither 26.1 defaults nor SDF navigation/pan behavior.
Existing [view research](view-navigation-plan.md) separates those facts from Bondline's
original 100%=one logical unit/point, 1.25 stepping, pan and persistence policies.

Pinned [PhotoCraft view code](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/crates/ui-egui/src/app/view.rs)
and [transform](https://github.com/storytold/photocraft/blob/0fb40c534a8f2ca2e1aa6182f7ab06ae9fbed32e/crates/ui-egui/src/canvas/view_xform.rs)
support separating logical zoom from DPI and sharing transforms. Its raster 100%, zoom
ladder and UI-side navigation handlers are not adopted. No proprietary artifacts were used.

Current `ui::preview` fits a registered whole-page PNG to its logical rectangle. Its key
has source/close/rect/pixels/DPI, but no view epoch. SDF exits the drawing branch before
canvas metrics/pan/wheel handling. `ui.view`, controls and pan helpers require
`stored_document()/document()`, so they refuse complete SDF sessions.

`prepare_sdf_preview` already derives **every** record off-lock, retaining exact mappings,
authored H versus display-H, existing translations and a minimum-A4 page that grows with
content. That exact extent is unavailable from the collection summary. Never assume A4
for larger files, parse MOL during painting/locked navigation, or manufacture a Drawing.

## Smallest proposed shared path

Keep the 28-command registry. Add one optional closed object to `render.capture`:
`view: { logical_width, logical_height, expected_view_epoch }`. Dimensions are bounded
integer logical units, distinct from existing physical `width_px/height_px`. Absence
selects the **unchanged** whole-page path, schema compatibility and approved-golden bytes.
Current-view requests capture Engine's view state, whole content owner, source stamp and
close epoch atomically; callers do not supply a second independent camera definition.

For SDF, the existing registered worker derives its complete preview, then uses its real
page size and display-H with shared `PageView`. Drawing uses the existing source-aware
IO adapter. No new parser, pool, generic job framework, source model or Save lease.

Bootstrap requires an explicit design decision: a dedicated current-view completion can
carry validated extent plus `Result<PNG, Error>`. Finish may install only a transient
`PageSize` cache after checking Weak `CaptureStamp`, view epoch and close context, even when Fit
truthfully returns `unsupported_scale`. **That is an observable error-side effect and
must be specified/tested; never label the failed image successful.** It confers no source,
edit or export authority. Cancelled/stale/closed completions install nothing. Key the
installed extent by source, not camera epoch: normal zoom/pan must not invalidate it.
Invalidate on content replacement/history/field Apply, not merely matching size. Cache no Page,
source Arc or record subset. If this error contract is rejected, use an explicit deferred
preparation action through the same registry instead; do not hide preparation in UI code.

Add a shared `Engine::view_page_size()` target accessor: actual Drawing size or current
prepared SDF extent, otherwise `viewport_unavailable`. Extend existing `ui.view`,
`view_percent` and controls through it; preserve wrong-kind/close/picker guards and no-op
epoch rules. Expose the bounded prepared extent on explicit view inspection for headless
clients. SDF has no own-last-drawn focus; retain the documented page-center fallback.
Pan helpers accept `PageSize`, not a fabricated Page. Do not change initial Fit/Open
persistence: an unreadable overview remains an honest error with reachable Actual/Zoom
controls and pan, rather than automatic repacking or skipped records.

## Rendering, DPI and UI callers

Add a narrow shared `PageView` raster-output operation beside
`graph_scene_with_display_hydrogens`, sharing its producer. It accepts physical output dimensions, derives
uniform density from actual logical/output sizes, generates glyphs at that density and
projects/clips geometry. Preserve logical 100%=1pt, signed camera translation, whole-source
chemistry/overlap/paper checks, ink/resource/cancellation bounds and existing physical
glyph 8..256 limits. Keep actual destination-to-capture logical readability checks too.
Multiplying zoom percent by DPI is incorrect: it changes the 999% semantic bound and
logical camera rounding. No oversampling at 1x, raster stretching or offscreen-validation
shortcuts. Specify physical rounding against the shared logical transform, not UI math.

`commands::capture/Prepared/finish` need the optional context, captured view and qualified
extent completion; existing `UiCaptureWork` and CLI/MCP/loopback still call that registry.
`ui::preview` keys additionally include view epoch/logical metrics; changes immediately
cancel/drop old textures/errors, preserving job ownership through decode/retirement.
Register the actual SDF rectangle metrics, reuse header/View controls and late ordered
pan/wheel actions outside the drawing-only branch. Preserve focus/modal/foreground/DPI
guards, field-draft protection and pending/worker/native-close barriers. No authoring hits
or bond tools are enabled for SDF.

## Independent tests before implementation

1. Freeze baseline RED: 1x actual-App SDF has no usable Actual/Zoom recovery. Keep the
   existing Fit/refusal and whole-page capture tests unchanged. Prove opted-in failed
   capture exposes no PNG success, but permits only its current bounded extent cache.
2. Original 740-byte two-record fixture: retain source SHA
   `e5cbdc14329c59bad55148c5de37058de90f651af10966c30cf8b89b3b5efd39`, five atoms,
   three bonds, authored H=None and existing coordinates. At 848×651, 100%, center(100,144),
   independent logical anchors are `(396,290),(432,290),(432,254),(432,398),(396,362)`;
   compare physical 1x/2x projection/rounding, actual main/subscript glyphs and CPU/GPU.
   Use unchanged external font/PNG consumers; fresh images are not approved goldens.
3. Test tall multi-record geometry exceeding A4; pan to first/middle/last without
   reordering, packing changes or partial projection. Unsupported chemistry or invalid
   paper ink in an off-camera record still refuses the entire capture. Viewport cropping
   is explicit; it never means the omitted record/H was removed from validation/source.
4. Bounds, fractional zoom, capped raster density, 1x readability, physical upper glyph
   limits, small/large canvases, cancellation and stale A→B→A epochs; mixed View/wheel/
   slider/source/field Apply/close events; historical finish and poisoned-job retirement.
   Camera must not alter raw bytes, model=None, revision/history/dirty/Save/ACK authority.

Then Root runs local gates and separately owns native 1160×800 1x/2x recovery, navigation,
field edit/Save/reopen and independent whole-file consumers. Windows/Linux and native
input remain unverified here. Exact source pins and concurrent-source qualification are
in ignored `target/sdf-view-plan/`; no jobs are running.

## Receiving decision after source review

The proposal above remains an exact research record. The implementation decision is
smaller: SDF Actual/Fit/SetZoom need no page bounds, so they may be offered before an
extent is prepared. Relative zoom from an explicit percentage likewise needs no bounds.
Only pan and relative-from-Fit need the actual source-qualified extent. All are camera
operations; none grants a chemical projection edit/Save authority. Existing Drawing
fully-mapped, pending-file and close guards remain.

Adopt the optional closed `render.capture.view` context above. On success only, a private
completion can install Weak CaptureStamp + PageSize after exact source/view/close/cancel
checks. Errors install nothing and return the actual error. There is no error-side-effect
or separate preparation command. The first successful explicit100% capture supplies real
pan bounds; retained previous centers are clamped by the actual derived extent.

Default UI Fit keeps its exact existing whole-page request (`view` absent), physical
margin policy, pixels and existing independent tests. Explicit percentages opt into the
logical camera and actual physical output size. Headless opt-in logical Fit is a distinct
transform: scaling a32-logical-unit margin at2x cannot match the legacy32-physical-pixel
margin. Do not silently equate those paths or rewrite old Fit expectations. New logical
Fit needs an additional affine oracle. The renderer API is additive
`PageView::at_pixel_size(width_px,height_px)` plus `pixel_density()`; generate fresh glyphs
at the transformed viewport, preserve the logical percentage and verify exact1x identity.

Independent actual-App, command and renderer tests are being declared before product
edits in three isolated worktrees at `af7fab0`. This decision section is not a test pass,
native acceptance, approved golden or completed SDF navigation workflow.
