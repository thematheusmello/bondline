# Explicit root bond-spacing data

2026-11-17; isolated `bondline-spacing-data`, branch `feat/bond-spacing-data`,
base `b377456`. This is the model/CDXML data increment, not final imported
appearance, renderer, command, native-form or platform verification. Root owns
those integrations and the broader gate. No dependencies or registry names change.

## Source and scope

The [published SDK BondSpacing property](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/BondSpacing.htm)
identifies tag0x0804 as **INT16**, binary10×percentage and XML percentage itself.
The initial assignment assumed UINT16; primary verification corrected that
assumption before any test bound or production was frozen. No incorrect unsigned
range is claimed as a runtime defect. The model accepts absent or finite
`0..=3276.7`, an original nonnegative signed-domain interpretation policy, not a
universal XML validity rule or a promise of lossless binary encoding. Values are
not quantized to tenths. UI1..100 is a separate Root-owned policy.

The SDK also documents [absolute spacing](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/BondSpacingAbs.htm),
overrides, inheritance and style-sheet fallback. None is inferred or modeled here.
Their presence still prevents the closed own-v0 source proof. Negative,
nonfinite, malformed/out-of-domain root values stay retained/opaque. XML source
certification additionally rejects negative nonzero mantissas that underflow to
f64−0; true signed-zero lexemes remain valid. This extra lexical rule is distinct
from the typed-f64 model domain and changes no old numeric parser.

Cached primary page:16,953 B, SHA256
`29821b45d3ad8155ae004be55084e3c9a1f1633ec8a0a564f6e937856bd4b79c`,
in ignored `target/bond-spacing-data/BondSpacing.htm`; `source.json` records its
URL/type/policy. No reference prose/assets or binary fixture is shipped.

## Behavior and compatibility

- `DrawingStyle::new(width)` retains its exact width-only contract, with spacing
  absent. The new getter and `with_bond_spacing_percent` retain independent
  optional spacing; `with_line_width` preserves it. Both remain checked Copy data.
  Closed map-only JSON omits absent properties; explicit zero stays present.
- Native writing appends present `BondSpacing` after `LineWidth`; old absent
  JSON/native-v0 bytes stay exact. Root-only projection uses ordinary attributes,
  not qualified lookalikes. Complete Preserved proof reuses the existing closed
  keys/typed certification. Per-property deltas preserve the other property's
  original numeric spelling, quotes and slot; setting only spacing no longer
  marks all style data as a changed LineWidth.
- Unknown root/child data, namespaces, foreign producer, owner overrides and Abs
  remain guarded. Unchanged source is exact; unsupported edited source refuses.
  Existing per-bond BondSpacing opacity in `commands/tests/multiple_bonds.rs`
  remains valid. No obsolete root-opacity expectation was found or removed.
- Any present spacing, including zero on Single bonds, refuses whole MOL export
  before raw/fragment reuse. Original MOL/CDX and synchronized SMILES source
  drift guards hold; restoring the original model restores exact source access.
  SDF/RXN raw-container APIs are unchanged; no model-taking writer is invented.
  The two scalar fields add no heap ownership; existing model size-based
  accounting includes them. No geometry, H, IDs or drawing settings change.

`InvalidDrawingStyle` keeps its variant/error mapping; only its authorized message
now describes both domains. No exact-message old expectation needed alteration.
No old test, fixture, consumer, dependency/lock, vendor, workflow, asset or golden
is changed.

## Independent RED/GREEN

All receipts/logs below are in this worktree's ignored
`target/bond-spacing-data/`.

| Declaration/receiving | Actual result |
|---|---|
| first model640B / `6994e471c2506bf60243f5441520453227b9618aabdbc85a8261144b41814b05` | runtime0/1, unknown JSON field;22191 exit101 |
| first original CDXML1,244B / `f4b270e31485c66bbef33f958a24eae8d8568754659c59017002a766cbf9ed42` | runtime0/1, root source remains opaque;62607 exit101 |
| expanded model/public API declaration before implementation | four intended E0599 missing getter/setter uses, exit101 |
| first coherent implementation | new model3/0 + legacy width2/0; IO5/0, expectations unchanged |
| supplemental negative-underflow declaration after that first draft | real runtime0/1: `-1e-9999` incorrectly certified complete;68819 exit101 |
| spacing-only decoded-mantissa correction | IO6/0, including negative/entity/whitespace forms and valid signed zeros |
| final formatted receiving | model41/0/0; new IO6/0/0; library clippy/fmt exit0 |

The original first declarations, expanded pins and first candidate remain archived
in `before/`, `expanded-before-production.json` and
`first-candidate-io-tests.rs`. `underflow-before-fix.json` pins the independently
declared supplemental source before correction. Formatting changes no assertions
or literal input. The source reader was qualified with original XML and a wrong
spacing expectation before actual writer output; Expat/Decimal verifies finite
numbers, literal coordinates, atom IDs/Z/H and directed bond references. Exact
expected-byte assertions separately protect all unmodified lexemes and source.
No own-reader/writer round trip is the sole acceptance oracle.

Critical logs (bytes / SHA256):

- `model-first-red.log`:1,139 /
  `4cc85ceb1b50ac30016733acffe2cf36c260914fe827a6466bf07b6e5ab79828`.
- `io-first-red.log`:4,027 /
  `209f69454bb4b10503faaaa04ba71045f26e2755be94493b2789afb27544641a`.
- `underflow-red.log`:886 /
  `13a27b932ba4f09d93ce96ed042e7cb9c499740a0af4e385f192581b76d4e644`.
- `model-final.log`:6,935 /
  `76a4b852e285f27da68c6861995e606585f3b1997f3dff1653e6cff4a90b7137`.
- `io-final.log`:1,268 /
  `f47adf22f9bc1751f7cbff546426c5f6f79862938617ac442f5bee5ea493a99e`.
- `clippy.log`:3,006 /
  `a779b841a7765b873d83b8e9c1ba058cf38fc977917662c501e5e2a019505bba`;
  `fmt.log` empty, exit0. Full identities are in `test-receipts.json` and
  `final-receipt.json`.

## Scoped reproduction and receiving boundary

Use the isolated worktree's absolute `CARGO_TARGET_DIR`, incremental0, jobs1,
DEV/TEST_DEBUG0, fixed `PROPTEST_RNG_SEED=671308429` and literal
`BONDLINE_PYTHON=/Users/sarenaberg/Workspace/doozle/open-source-world/bondline/.cache/oracles/format-python/bin/python`.
Do not resolve it into a different interpreter. Python3.13.0 supplies stdlib
Expat/Decimal; Rust/Cargo1.94.1 on Darwin27.2.0 arm64 was used.

```text
cargo test -p bondline-model --locked
cargo test -p bondline-io --test bond_spacing_data --locked
cargo clippy -p bondline-model -p bondline-io --lib --locked -- -D warnings
cargo fmt -p bondline-model -p bondline-io -- --check
```

All owned jobs are terminal. No full workspace gate, GUI/GPU/native execution,
fuzzing, Actions, push or publication was performed. Renderer/command/UI/main
docs remain Root-owned. These data APIs must be received with the separately
verified actual-spacing renderer and source-aware command/preview/capture routes
before any complete imported-appearance claim. Abs, owner inheritance/defaults,
binary spacing projection/writing and broader foreign styles remain missing.
