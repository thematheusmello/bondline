# Actual scene label metrics

2026-11-14. Native preview needs to distinguish readable device-pixel labels from
readable logical labels after a high-DPI image is displayed. GraphScene now exposes
the smallest actual main atom-label em, or None when no atom label was produced.
The value comes from the same Label objects that produce ink; no source is reparsed
and no UI font-size/A4 assumption is needed. Subscripts retain their existing scale.
This fact changes neither layout, font, hits nor CPU/GPU pixels.

TDD: session76724 exits101 for the missing label_em_px accessor. The independent
analytic case has256pt paper fitted to512px, so the original12pt label is24px;
doubling the fitted paper yields48px. Both literal/resolved paths agree, while an
empty page reports None. Full renderer gate15005 passes46 tests, all-target clippy
with warnings denied and rustfmt. Existing font oracles and approved golden pass
unchanged. Logs and exact source identities are in target/preview-metrics-receipt.json.

This exposes an original rendering fact, not reference behavior or verified native
readability. The next UI must multiply actual em by the actual destination/capture
ratios and retain the existing minimum8 logical-unit main-label policy. No native
window, hosted workflow or other-platform execution occurred; Actions remains off.
