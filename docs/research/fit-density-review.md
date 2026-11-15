# SDF legacy Fit, density and readout

Read-only design review at `976ef42`, 2026-11-15. No product/test/SDK/dependency edits,
native input or test execution. Calculations below are source-derived arithmetic,
not measurements of a native window, display DPI or successful chemical rendering.

## Conclusions

1. **Show “Fit” for every SDF Fit state**, independently of cached bounds. Fixing only
   the unknown-bounds `unwrap_or(100.0)` case removes the initial invented percentage,
   but subsequently exposing `view_percent()` still describes another transform at
   higher density. Absolute numeric editing should continue to dispatch `set_zoom`.
2. **Keep the existing camera selection rule.** `PreviewKey::request` selects the
   current-view request only when `zoom_percent.is_some()`. Fit always uses the legacy
   whole-page request. It does not depend on cache presence now; preserve that property.
   Returning from explicit Zoom to Fit, or editing a field, must not select another
   Fit renderer just because bounds appeared/disappeared.
3. Treat relative steps as a separate policy. Existing F7/F8 resolve **logical Fit**,
   then multiply/divide by1.25. That is deterministic original Bondline behavior,
   but is not a1.25/0.8 change relative to the displayed legacy overview at every DPI.
   A label-only change should not silently replace this core command contract.

## Distinct root causes

`view_controls::zoom` currently replaces an unknown percentage with100. That invents
   a current value while Fit has neither prepared extent nor a measured numeric basis.

When an extent exists, `Engine::view_percent` and relative actions instead call
`fit_percent(logical_width, logical_height, extent)`: `PageViewport` uses its rounded
logical paper and up-to32 logical-unit margins. The native legacy PNG is fitted at
its **physical** output dimensions, keeping up-to32 physical-pixel margins, then painted
into the logical rectangle. Its scale differs at density2. This is not rounding noise.

For A4 and logical848×651, the independently calculated scalar fit bases are:

| Density | Logical Fit percentage | Legacy physical-fit scalar expressed logically | F7 target | F8 target |
|---|---:|---:|---:|---:|
|1|69.715608%|69.715608%|87.144511%|55.772487%|
|2|69.715608%|73.495370%|87.144511%|55.772487%|

The logical paper is415×587 at(216,32). At density2, physical1696×1302 instead fits
875×1238 at(410,32). Thus current relative targets have geometric factors1.185714 and
0.758857 against that legacy2x basis. Actual glyph guards can refuse a target; these
numbers do not establish readable or successful1x preview. Keep every glyph/H guard.

A field edit changes the full source stamp and invalidates the prepared extent even
when MOL geometry stays exact. This intentionally changes bounds availability, not
camera mode. A Fit readout depending on cached percentage will therefore flip after
an edit despite unchanged Fit image geometry. Do not reuse stale source bounds or
change the camera to hide that transition.

## Small next acceptance slice

First add one actual-App regression at density1 and2: open the unchanged SDF in Fit;
obtain bounds through successful Actual capture; return to Fit; edit one existing
field; retire/retry the real preview. At every Fit stage assert visible **and
accessible** Fit, unchanged legacy request/projection, and exact legacy PNG bytes
where rendering succeeds. A guard refusal must remain honest with no stale image.
Check an absolute numeric SetValue still enters explicit Zoom; source/history/Save
remain unchanged except the intentional field edit. Freeze the initial readout RED
before implementing the wider Fit presentation policy.

Formatter-only cosmetics need care: pinned egui0.29.1 `DragValue` still publishes its
underlying numeric `WidgetInfo` value. `response.rs` writes it to AccessKit. A displayed
“Fit” string can coexist with invented numeric100 and relative increment/drag behavior.
Require honest accessibility and absolute entry; do not merely relabel a fake number.

For now, document/test F7/F8's existing logical-Fit basis explicitly. If a later product
decision requires stepping from actual visible legacy scale, supply bounded physical
output and logical destination context through the same registry and compute the
basis centrally. DPI alone is insufficient under texture caps/rounding. That is a
separate command-contract change, not this label fix. Missing extent may still disable
Fit-relative actions; preserving strict source qualification takes precedence over
inventing bounds. No cache-presence camera switch or hidden first-record shortcut.

## Sources and unknowns

Exact source pins are in ignored `target/fit-density-review/inputs.json` and the final
receipt. Reviewed preview requests, header/controls, `view_percent`/relative handlers,
shared renderer and their callers. Public roles come from the already pinned
[guide25](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf), printed26/79;
[existing research](view-navigation-plan.md) records its SHA and PhotoCraft logical/DPI
sources. It does not establish Professional26.1 Fit percentages, stepping factors or
SDF defaults. Those remain unknown; all numeric policies above are original choices.
No native hardware/OS accessibility behavior is inferred from headless arithmetic.
