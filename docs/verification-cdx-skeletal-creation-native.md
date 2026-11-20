# Creating a printed label on a skeletal CDX atom

At 100% zoom, Option-click (Alt-click) an exposed carbon vertex on a qualified CDX drawing. The Atom label editor shows the original element, the literal attached hydrogen count if present, and the creation style. Enter a supported non-carbon element and an explicit hydrogen count. Apply creates one history entry. Reopening the saved document allows the printed label to be edited using the same gesture.

## Native controls and reference files

| Original drawing | Atom and paper point in pt | Initial attached H field | Input | Saved bytes | Reopened formula |
| --- | --- | --- | --- | ---: | --- |
| Ethanol | 1 at (56, 144) | Empty | N, 2 | 439 | CH₅NO |
| Methanol with literal H | 1 at (128, 192) | 3 | N, 2 | 379 | H₃NO |
| All-carbon propane | 2 at (224, 192) | 2 | O, 0 | 338 | C₂H₆O |

The three native saves use 12 pt black Roboto Regular. The first and third match the [independent format fixtures](../crates/io/tests/fixtures/cdx-skeletal-label-creation/SOURCES.md). The [native methanol fixture](../crates/ui/tests/fixtures/literal_h_native.cdx) uses the same original input as the 14 pt blue command fixture with the native form's style. Both use the project's MIT OR Apache-2.0 terms. To reproduce the native fixture with FontTools 4.61.1, run from the repository root:

```sh
PYTHONPATH=xtask/oracles python3 -B - <<'PY'
from pathlib import Path
from cdx_skeletal_creation import expected
result = expected("literal_h", 73, "N", 7, 2, 12, 0)
assert result == Path("crates/ui/tests/fixtures/literal_h_native.cdx").read_bytes()
PY
```

RDKit 2026.09.2 independently decoded the exact three outputs as NCO, NO and COC and returned the formulas above. The reference calculation used `Chem.MolsFromCDXML` with explicit CDX format and with sanitization and H removal disabled, followed by `UpdatePropertyCache(strict=True)` and `rdMolDescriptors.CalcMolFormula`.

## Receiving checks

The [native App tests](../crates/ui/src/cdx_skeletal_creation_app_tests.rs) feed real egui paper coordinates and event-local modifiers. All three failed at the initial Alt-press before the GUI change. A second test failure exposed a file transition: after all hidden carbons in the methanol fixture acquired labels, the retained Text tool intercepted an Alt-press on its reopened printed N. All three complete cases now pass through Cancel, Apply, Undo, Redo, byte-exact Unicode Save, reopen and the existing-label form. The tests retain Ring or Text and check Alt+Shift, secondary button and a blank paper point. The [skeletal display tests](../crates/ui/src/skeletal_display_tests.rs) cover dormant flags, unchanged ordinary gestures, selection movement and exact byte retention. Frontmost painted captions keep priority in the [mixed-page native tests](../crates/ui/src/native_caption_tests.rs).

```sh
cargo test --locked -p bondline-ui --lib cdx_skeletal_creation_
cargo test --locked -p bondline-ui --lib
```

On 2026-11-20, the UI library had 372 passes, zero failures and one previously ignored test. A separate macOS 27.2 arm64 background run opened the original files in an isolated Bondline process. The authenticated host opened the targeted form, macOS accessibility reported its controls and preset, and AXSetValue and AXPress supplied both values, Cancel and Apply. Undo restored the exact original bytes. Redo and Save to a Unicode filename produced each reference above. Reopen restored the edited structure and an existing-label editor. The accessible Analysis window reported each formula. Opening a form on the flagged hidden methanol carbon returned `unsupported_content`. The process closed normally.

| Native case | Saved CDX SHA-256 | 512 × 512 PNG SHA-256, CPU and Metal |
| --- | --- | --- |
| Ethanol | `72ced7b21b5f9d67f674b8cd1792452819147866e417835b91536e292731a5ab` | `594c887ed0b09e5e8eb17dfa59690e35112011b9fa5ce5c74fa04b605e6e2f66` |
| Methanol | `e0851af59c16c1c6c085ba6f8cba3056d15e01699a40579d9c8163874d9dc6b1` | `439797fe9f923376c39366ffaaa7d8354c60905fb6b98b27cae38d9a4df26fd7` |
| Propane | `17d27871947c650a47234623e1f541724200049cee467f3c4d745ab4a4ae25d9` | `c6c3b8b871db0cdce3c5a9d68af251f0e7929d5f3bc3bb8ce3affbbf9c49f599` |

The retained local receipt is `target/native-skeletal-creation/proof-z23ckymt/receipt.json`, SHA-256 `38a154679bc03c0471e874ac03e310f16af68b78b62d38919d32114904acfe07`. The OS run used the registry to target the form. Pointer routing was verified with App input frames. Interactive Windows and Linux validation remains open.
