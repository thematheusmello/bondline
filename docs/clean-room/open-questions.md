# Open questions

- Public 26.1 release notes are available, but the full current menu/tool/shortcut inventory
  still needs a documented audit. Do not claim completeness from the 25 guide alone.
- The archived CDX SDK does not establish every modern extension. Preserve unknown data.
- The chosen native dependency graph must be audited for macOS, Windows, Linux and later wasm;
  crate-level MIT labeling does not establish runtime closure or asset licensing.
- CPU/GPU raster tolerances and human-approved goldens remain to be established before claims.
- Editable Office embedding and modern naming/prediction behavior need public evidence.
- REA 6.9.0 requires a newer Node runtime than the currently selected Node 22.11.0. Keep it
  outside the shipped Rust application; do not install a system-wide replacement as a shortcut.
