# Truthful Fit readout

2026-11-15, based on `976ef42`. This is a UI presentation correction, not a new
camera or file-format policy. All SDF Fit states display `Fit`, including when bounds
are prepared. Unknown Drawing Fit also displays `Fit` until layout is known; known
Drawing percentages and explicit SDF percentages retain the existing numeric control.
The field's accessibility node has no invented numeric value in Fit mode. Its numeric
explanation tooltip is disabled there rather than exposing an internal editor seed.

The existing DragValue, input-event ordering and `ui.view` registry remain. Recording
its setter calls distinguishes an explicit100 entry from untouched Fit even when the
editor seed is100. Typing alone does not commit; Enter/blur does, and Escape cancels.
There is no model, source, history, adapter, rendering, dependency or SDK change.

## TDD and independent checks

The independent actual-App fixture creates the unchanged740-byte ordinary SDF. Its
first4,207-byte test was frozen as SHA256
`8909f2f7df22d3201ad68fa8e63e9820f4ff7a8b827025f44f8b73d0465a90b0`.
Independent57640 and receiving Root RED both fail the actual painted `Fit` assertion,
not setup or compilation. Root first tried an over-specific `--exact` filter that
selected zero tests; that attempt is not RED evidence. The corrected command is:

```sh
BONDLINE_PYTHON=/absolute/path/to/pinned/python CARGO_INCREMENTAL=0 \
  cargo test --locked -p bondline-ui --lib unknown_fit_stays_truthful -- --nocapture
```

Root57556 then passes that unchanged test. Four final cases additionally require
explicit100 on blur, numeric accessibility100, prepared Actual→Fit at2x with textual
and nonnumeric Fit, and truthful known Drawing Fit/explicit150. The first test prefix
remains exact. One additive case originally looked for `Fit` instead of the existing
`Fit to window` menu label; the only approved correction changes that locator. Original
declaration/log and correction are retained by the independent owner. No old assertions,
fixtures, pixel or chemistry oracles or approved golden were weakened.

Root42920 passes all30 existing-plus-first-new view/inspector/editor cases. Independent
four-case execution passes4/0 and UI all-target clippy; its initial formatting check
requires only rustfmt's multiline `matches!` layout. The formatter changes no behavior.
An independent source review then finds an actual disabled-state defect: gating Fit
presentation on control enablement brings back100 while the accepted picker disables
Zoom. A fifth additive test fails that visible assertion on worker66097/Root72603.
Selecting Fit presentation by actual content/camera instead fixes it once; unchanged
five-case Root50735 and worker12179 pass5/0, and worker UI clippy/fmt pass. Full final
test source13,394B/SHA256`6f646c111e9454294e4436afc837b2c1dc1083dca9648334a74c24b2f9b9ac87`
preserves both earlier4,207/9,381-byte prefixes. [Independent receipts and classifications](verification-fit-readout-tests.md)
retain every earlier attempt.

Root77768 final `CARGO_INCREMENTAL=0 cargo xtask ci` with the pinned Python passes
**1021 tests,0 failed,1 ignored**,149 result lines including3 native AccessKit cases.
Rustfmt, all-target warnings-denied clippy, layers, licenses and corpus all pass.
`target/fit-readout/root-ci-final.log`:122,094B/SHA256
`fe1bf7c0a94e167d6c57f4da55b221fdfcb24db8fd24c77b7869c49c8976b0ee`.
Earlier64576 passes1020/0/1 before the fifth case and disabled-state fix. No broader
claim follows from the unchanged154/165/220 license declarations,2 assets or8-file/
5-bundle corpus; general CDX/chemistry remain missing. No new fuzz or hosted execution.

## Actual native scope

Root32266 builds the actual macOS app. Ignored bundle
`target/fit-readout-native/Bondline.app`, identifier `org.bondline.fit-readout-review`,
PID84150, executable21,773,960B/SHA256
`acb89f940b6b86adc281987cfd0c6a2eb3671013fedcf94a8762218008db204a`.
Build receipt pins pre-rustfmt `view_controls.rs` SHA256`03d5969d…a302`; intermediate
`337ed87f…c8af` differs only by that formatter layout. This initial binary predates
the separately tested disabled-state correction.

Authenticated loopback opens an ignored exact740-byte copy, then Actual/current-view
capture prepares real bounds, then Fit. Read-only native AX and actual screenshot
observations show `Zoom, Value: Fit` both before and after preparation, with the existing
glyph-scale refusal still visible. An accepted Fit never promises readable chemistry.
Further loopback Actual/pan restores both structures at100%, center100/143; native AX
and screenshot confirm the image and controls. Revision1, clean source, Undo0/Redo0 and
SHA256`e5cbdc14329c59bad55148c5de37058de90f651af10966c30cf8b89b3b5efd39`
remain exact. Private startup tokens are never printed or included in public records.

Root23857 then builds the actual final `4b9342af…7dc72` UI source. The clean initial
review app is closed through its registry, and final PID16278 runs the same ignored
bundle, executable21,773,960B/SHA256
`21efac416ddd1d2933da1d6bdb61b8b0d33aa495e15726acaec78a9a66b1445a`.
Native AX and screenshot show the real accepted Open picker with the disabled Zoom
field still displaying `Fit`. Registry Cancel, Actual/current-view capture and pan
restore both structures; native screenshot/AX and read-only inspection confirm100%,
center100/143, epoch2, revision1, clean740 bytes and Undo0/Redo0. The first immediate
restart PID11741 vanished before startup output without diagnostics; cause is unknown.
A later launch records startup readiness and succeeds. That failure is not hidden or
treated as a product-flow pass. Build/source/state receipts remain under the ignored
bundle directory, with the initial receipts retained separately.

No OS input was injected: the desktop-availability clarification remains pending.
This verifies real native painting/AX after registry operations, not native typing,
wheel/slider accessibility, clipboard, physical monitor DPI or Windows/Linux interaction.
One receipt script initially requested nonexistent `history`; a subsequent read-only
query records the actual `undo_depth`/`redo_depth`, without replaying the mutations.

## Remaining limits

[The independent design review](research/fit-density-review.md) separates the label
defect from the existing high-DPI relative-step discrepancy. Native SDF Fit still uses
the exact legacy physical-margin image; F7/F8 use declared logical Fit math. Labeling
all SDF Fit as `Fit` is stable across cache invalidation and does not silently alter
that camera policy. Relative steps, broader accessibility and native input acceptance
remain separate work. Full M0–M5 is incomplete.
