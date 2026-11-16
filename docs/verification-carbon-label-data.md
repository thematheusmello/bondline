# Persistent non-terminal carbon-label metadata

2026-11-16, based on contract4b139d8. This is a data/format prerequisite for skeletal
display and unsaturated-ring tools, not working skeletal display or milestone completion.
The [locked contract](research/carbon-label-data.md) and source register distinguish
standard node data from unknown root inheritance, child Text and reference26.1 behavior.

## Delivered boundary

Atom owns optional show_non_terminal_carbon_labels. None is omitted and retains the old
Bondline literal-label profile; explicit false/true retain presence and value without
changing literal H, geometry, topology or identities. This compatibility policy does not
claim SDK absence differs from no. Exact node yes/no maps to false/true; malformed values
refuse typed mapping while standalone CDXML keeps every source byte. The native writer
appends the standard attribute only when specified. The existing exact re-encoding proof
alone grants complete native editing; unknown/root/Text content stays source-backed.

Both MOL model writers refuse any specified flag, including dormant or true values.
Manual retained-MOL comparison and retained-SMILES H/order replacement include it;
detached model drift cannot return stale source. Explicit graph-only SMILES export keeps
its already documented depiction-loss scope. Raw SDF/RXN and CDX implied-flag syntax are
separate controls, not new model writers or binary flag projection.

The shared renderer temporarily refuses any specified value, after cancellation. Every
absent/null legacy pixel, hit, glyph, mask, H value and source guard stays unchanged.
The next increment must replace this refusal with tested skeletal semantics; persistence
alone does not establish a useful skeletal drawing workflow.

## Actual test-first evidence

All handles below are Root executions. Workers declared tests/reviewed sources only.
No old expectation, fixture, external oracle, font, shader, approved golden or dependency
was changed. Six adopted test files equal their frozen declarations after rustfmt only;
the receipt is target/skeletal-carbon/root-receiving/declaration-lineage.json.

| Stage | Actual result |
|---|---|
| Initial existing IO API18680 | Runtime RED0/1: source bytes match first; Atom JSON lacks explicit false. |
| Initial model JSON32630 | Runtime RED0/1: closed model rejects the unknown field. |
| Public field API | Separate E0609 compile RED; this is API qualification, not runtime behavior. |
| First model73733 | JSON1/API1 GREEN after adding only the field/default. |
| Expanded model88990 | JSON4/API1 GREEN. Expanded declarations follow the first field draft; the initial1,232-byte prefix stays exact. |
| Shared renderer96283 | RED2/1: legacy/cancel controls pass, specified metadata is silently ignored. |
| Expanded IO54896 | RED2/7: raw/consumer controls pass; mapping, writer and no-loss guards fail. |
| Private retained-MOL46409 | RED0/1: None versus Some(false) is incorrectly treated as the same authored fields. |
| Registry55131 | Runtime RED0/1: the independent actual-file consumer finds the missing attribute. |
| IO receiving | GREEN26 library cases, including the new private comparison, plus9 new integration cases. |
| Renderer53005 | GREEN3, including complete CPU/Metal-GPU legacy frames and entry cancellation. |
| Registry28985 | GREEN1: Unicode files, exact Expat/Decimal properties, unchanged graph oracle, pinned RDKit, order/no-op/unrelated-label/history and MOL-loss refusal. |

The new increment adds19 focused cases: model5, renderer3, IO9, private comparison1 and
registry1. Original non-dense IDs, literal counts and topology are independently specified.
The new XML consumer is qualified using3 manual positives and8 semantic negatives;
it does not widen the old graph consumer. The registry case deliberately avoids deciding
same-target atom.set_label visibility/no-op policy before the following display contract.

Frozen declaration SHA256 values:

| Declaration | SHA256 |
|---|---|
| Expanded model JSON | 8082848cc03f405fdf6e51695fcaf7be52b2828a4c0385f3efefdde3589f517b |
| Public field API | b0d596995472fca31ecbc2852df857f1110853469c571794443dcfe328b86c1f |
| Shared renderer | 6ceb7641e1f49433d3d9b67a4d47487b0ee22d503931bdeff4556609702c80f5 |
| Expanded IO | a3321456f7e24b8e9eb97413c717cb069c0470968918cdde43833617390746d9 |
| Registry | e19aa281e9bb643268a3af99036d8f41a8754116913f8d0b29b1a65bedd69ea7 |
| Private comparison | bcde5d914f94f646cb7977dc43ebeee1fc078132644f3187dc0886b7739405a4 |

## Bounded parser smoke and local gates

Fresh ignored source/corpus directories .cache/fuzz/carbon-label-data-4b139d8 contain
11 initial files/3,871B, largest442B: unchanged original native-v0, original node absence/
yes/no/malformed values, root/Text controls and native-profile yes/no variants. Sources
are separate from mutation inputs; fuzz-seeds.json records every exact byte identity.
The unchanged cdxml_parse harness exercises syntax/geometry/mapping/source/native proof.

ASan25188 exits0 after10,000 runs with seed873269411, max_len65536 and timeout5.
Final cov3013/features8406,369 retained inputs/reported86KiB, RSS171MiB,2seconds,
largest exercised442B. The64KiB ceiling is not full-size coverage or a security audit.
Tools are cargo-fuzz0.13.2/libFuzzer0.4.13/nightly-2026-04-01, with this workspace's
absolute target, incremental0/release-debug0 and final -Cdebuginfo=0. Root/isolated
manifests/locks, original fixture and MCP test source are pinned before the run.

Full local aggregate19662 exits0:1470 passed/0 failed/1 ignored across210 result lines,
including fmt, warnings-denied workspace/vendor clippy/tests, layers/licenses/corpus.
Log181965B SHA25632b6494f0ad8292d106334e8d3dc2220c02ac7ff74193a6b093a14a6056f6b99.
Native/CLI build19572 exits0. No native window or hosted workflow is started. Actions
stays off. The historical natural MCP replacement-bind failure remains unattributed;
consumed investigations are not repeated, and this pass establishes no causal repair.

The independent source review finds no material DATA-scope issue. Sealed report5778B
SHA256dcc7b1eea72438cf54c5c63dab2157116165b3109ac9370df7a2e8cc86daa879;
source receipt11799B SHA25601fa51543e47be1fb3b6ef66a65fa122de24abe4e350d7e7c19e48810681c362.
It verifies the six declaration/live identities and manual loss guards; execution remains
Root-owned. New Atom direct struct literals intentionally require the additive field;
workspace callers retain the unchanged Atom::new signature/default. Root/isolated
manifests/locks, native-v0 fixture and unchanged MCP test still match pre-fuzz pins.

Built native binary SHA256d0873d6c812187d936c915265137d8da6be658720e4dc88a7a747f680c7b5c64;
CLI SHA2568bbba4e858e2e01866ab77b80baa9b0e25f2942e453227d0e5e23e49b1e97958.
Exact final product/test/log/binary identities are in the ignored receiving receipt;
all named Cargo handles are terminal and reviewers have no live jobs.

Reproduce from the repository root using the literal pinned interpreter path:

```sh
export BONDLINE_PYTHON="$PWD/.cache/oracles/format-python/bin/python"
export CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
cargo test -p bondline-model --test carbon_label_json --test carbon_label_api --locked
cargo test -p bondline-io --test carbon_label_formats --lib --locked
cargo test -p bondline-render --test carbon_label_data --locked
cargo test -p bondline-commands --test carbon_label_data --locked
cargo xtask ci
```

Do not retry an unchanged naturally failing MCP test to obtain green. Preserve failure
evidence and run only not-started stages separately. Native/DPI/Windows/Linux skeletal
interaction, actual visibility commands, aromatic tools and full M0–M5 remain unverified
or missing. Next lock the sourced eligibility/mandatory-label/explicit-authoring policy,
then test and implement it through the existing shared renderer and registry.
