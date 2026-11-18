# Current hidden-carbon query: receiving evidence

2026-11-18, renderer-only prerequisite on base `f16b79f`, following the locked
[mixed-label contract](../../../../docs/research/mixed-skeletal-labels.md) committed as
`b30f0d6`. This does not implement foreign-source admission or enable another tool.

`hidden_carbon_atoms(fragment, cancel)` calls the unchanged
`skeletal::labels(fragment, None, cancel)` once, then returns only literal atom IDs whose
existing decision is Hidden and whose authored label is absent. It checks cancellation
initially and per visited atom. It neither infers H nor duplicates the degree, C4,
isolated-C/C2, inclusive170-degree or two-Double rules. The private classification enum,
page callers, rendering, hits, clearance and legacy absence paths are unchanged.

The existing original rules and their primary public basis remain in
[skeletal carbon display](../../../../docs/research/skeletal-carbon-display.md) and
[terminal carbon labels](../../../../docs/research/terminal-carbon-labels.md). This API
adds no new depiction rule. In particular, an absent ID is not proof that a glyph exists:
legacy paths vary. A returned ID is a display decision, not a complete source, chemical,
physical-page, style/default or editing certificate. Callers must re-evaluate the current
fragment rather than reuse a baseline's result after positions or neighbours change.

The five new [direct model tests](../hidden_carbon_atoms.rs) are original independent
literal-ID/geometry controls, frozen before producer changes with SHA256
`56330b76369c511cb6c6ecdb4c1fe6200800225231c3a3fab71938fe37dca3a2`.
They do not use IO, a generated glyph oracle or a producer-derived expected-ID helper.
They cover:

- Sparse/reordered IDs, reversed bonds and explicit-versus-legacy flags on a bent
  OH–CH2–CH2–CH2–NH2 chain; expected IDs are exactly12,41,601.
- A current middle-atom move into opposite rays, independent169/171-degree controls,
  and mandatory two-Double labels.
- Isolated carbon and isolated C2 Single/Double/Triple components with an unrelated
  atom; a current neighbour-element change revokes the C2 exception.
- Unknown H, invalid false-flag C4 and unsupported bond-order refusals, preserving the
  existing true-flag literal-label exception.
- Any authored-label presence taking priority over Hidden, plus pre-cancellation for
  populated, empty and no-flags fast paths.

The actual initial failure was a **missing-API compiler RED**, exit101/E0432 for the
unresolved `bondline_render::hidden_carbon_atoms` import on unchanged `f16b79f`. No
runtime failure is claimed for that attempt. After adding only the wrapper and re-export,
the exact frozen tests pass5/0/0 without test changes. Logs are retained in ignored
`target/mixed-labels/render/missing-api-red.log` and `hidden-api-green.log`.

The complete `cargo test -p bondline-render` run passes191/0/0 across38 test executable/
doc-test reports, including all186 previous tests. The existing external FontTools/Pillow
checks use the pinned repository Python environment; these five direct tests need no
external program. `cargo clippy -p bondline-render --all-targets -- -D warnings`,
`cargo fmt --all -- --check` and `git diff --check` pass. Full and lint logs are
`target/mixed-labels/render/full-render.log` and `clippy.log`. No old test, fixture,
tolerance, glyph, classification path or golden was changed.

This is renderer-scoped verification. New foreign-file workflows, native execution,
cross-platform evidence, the aggregate workspace gate and full M0–M5 remain separate
work. No dependency, asset, font, workflow or GitHub Actions change.
