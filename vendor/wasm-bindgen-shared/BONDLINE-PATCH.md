# Reviewed wasm-bindgen-shared browser patch

2026-11-20. Upstream [wasm-bindgen-shared0.2.130](https://github.com/wasm-bindgen/wasm-bindgen/tree/c93b8ae848d4d9e6beaeaf9516e18a7cadd818a2/crates/shared), MIT OR Apache-2.0.
Published crates.io archive13698B, SHA-256
`3b1f9c3798725016c6ee5f2f57869ec0831e8bbc0ab0310a86e787e97f41389e`. VCS`c93b8ae848d4d9e6beaeaf9516e18a7cadd818a2`,
path`crates/shared`. All11 published files are retained; only the files
in the table below differ. Original full notices remain byte-exact beside this file.

The identifier helper is a build-tool API; inspected runtime users need the shared
type schema and module-path hash, not Unicode identifier classification. Keep the
complete original helper and unicode-ident dependency on non-wasm targets; omit
both on wasm32 targets. There is no replacement identifier algorithm, copied
Unicode table, changed declared license or expanded allowlist. The original
normalized manifest alone is authoritative; Cargo.toml.orig remains the archive
receipt. The macro/CLI schema files, build script and schema approval are unchanged,
so the installed matching wasm-bindgen0.2.130 CLI remains compatible.

| Changed file | Published bytes / SHA-256 | Patched bytes / SHA-256 |
|---|---|---|
| Cargo.toml | 1867 / `bb5f8ff99ad057721e310a968ccb91a9c7c3df07f421fa638f12921b74d3b613` | 1909 / `acf73a12883d8175262f11eca63432973387138279b1554edfb3d3a66b7e53bc` |
| src/identifier.rs | 4849 / `05ce1e3dbaaa2c5a9be7ab66ccb11c924c46c75df214fa7477aed9b35152b10f` | 4887 / `6a3e64a0c6fa0d13cf6f77d864a6219c977d730999abaa2b532e6ee050f24bd0` |

Byte-exact upstream notices:

- [LICENSE-APACHE](LICENSE-APACHE), SHA-256`a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`.
- [LICENSE-MIT](LICENSE-MIT), SHA-256`378f5840b258e2779c39418f3f2d7b2ba96f1c7917dd6be0713f88305dbda397`.

Byte-exact schema files:

- `src/lib.rs`:SHA-256`348517b67a8859abdbfee4c702ec641ad96bf9346369408b52167a15728b7c00`.
- `src/tys.rs`:SHA-256`bae2dc8dbd8e44c863a62e00f7b5503b17e37dfaf03bc89c484527ea8c1bd5ec`.

[Research](../../docs/research/web-workspace.md) records the official sources,
and [receiving evidence](../../docs/verification-web-workspace.md) separates real
Chrome WebGPU execution, native checks and the strict target-specific runtime audit
from unverified browsers and complete M5. Keep this a local reviewed fork; no
upstream acceptance or broad browser compatibility is claimed. Reinspect actual
consumers and rerun the workflow/license gates when upgrading.
