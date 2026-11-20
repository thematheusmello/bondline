# Browser runtime notice receipts

The AccessKit0.16.3 and profiling1.0.18 crate archives omit their full notices.
`manifest.json` records the exact public VCS roots from their published
`.cargo_vcs_info.json`, raw source URLs, lengths and SHA-256 values. The four
adjacent files were fetched unchanged from those revisions on2026-11-20. The
browser packager verifies the current package/VCS/hash before retaining them.
No fabricated notice, license-label change or allowlist exception is involved.

ecolor/emath/epaint/egui-wgpu0.29.1 have the same archive omission. Their published
VCS is exactly `fe368bacc4a15697e347607e73a56c0299b3d42d`; the complete upstream
MIT/Apache notices already retained in `vendor/egui` apply to that same root.
The packager checks that identity and retains those actual files. Other selected
runtime packages supply their own published notices; Bondline crates retain the
root notices and the bundled Roboto font retains its Apache notice. Missing
unreviewed notices fail packaging. This is bounded package/source notice
retention, not a completed whole-release source audit or legal certification.
