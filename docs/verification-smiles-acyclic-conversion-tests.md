# Independent acyclic conversion command and adapter acceptance

Started at `97fd2ac` in the isolated `bondline-smiles-acyclic-conversion-tests` worktree. This record distinguishes declarations, actual baseline failures, received production and subsequent checks. Native interaction and general SMILES import are not verified by these tests.

## Declared contract and independent expectations

The existing `file.convert_smiles_to_cdxml` command remains one of 30 registered commands. Its eight required scalar parameters remain unchanged. Optional `profile` is a closed choice: omitted/default `neutral_carbon_path_v1`, or explicit `neutral_acyclic_v1`. Old parameter serialization, old profile output and its refusals remain exact. Conversion creates a new file through the existing publisher/save reservation and never implicitly Opens it or creates active-document Save acknowledgement.

The new profile is a bounded connected neutral nonisotopic acyclic H/C/N/O/halogen graph with Single/Double/Triple bonds. Origin means the minimum atom-center bounding coordinates; this is deliberately different from the old path endpoint anchor. The new IO test oracle receives original notation and literal atom/H/endpoint/order/formula/length/page expectations, not expectations reconstructed by Bondline's writer.

Five command cases declare:

1. Explicit typed profile parsing through the existing dynamic command API, without new Rust APIs or file access.
2. One optional closed schema, old eight-field serialization/default output, explicit old-profile equivalence and invalid-profile atomic refusal.
3. Literal `CC(C)O` (C3H8O), `CC(=O)O` (C2H4O2) and `CC#N` (C2H3N) conversions, retaining a dirty complete Drawing and a complete multi-record SMILES source owner. Source, history/redo, close policy, association and every automatic-state field stay exact; data values remain private. Output filenames include Unicode and a misleading `.mol` suffix while their actual content is independently consumed as CDXML.
4. Explicit ordinary Open of converted `CC(C)C`, actual CPU/GPU registered PNG equality, carbon order edit, complete Undo/Redo, real Unicode CDXML Save/reopen and V3000 export consumed by the unchanged external MOL consumer. The source graph/H vectors are C4H10 then C4H8; coordinates and graph are also checked by the new independent consumer.
5. Old/new-profile refusals, malformed input, cancelled/stale requests, create-only destination protection, the same save lease before/after publication, historical receipt after New, and no generic native Save binding/ACK authority.

Two adapter cases reuse the unchanged stdio `Client`, authenticated socket framing and finite timeouts from `cli/tests/sdf_capture_adapters.rs` through a new child module. They cover actual CLI stdio discovery/conversion, whole-source Save/reopen/privacy, explicit converted-drawing Open, plus authenticated conversion while the current Drawing is dirty, no automatic repaint, stale/refused publication and the shared native-save lease.

## Test-first evidence

The initial declaration is the exact 806-byte source provided by Root, SHA-256 `020e953ef7d56291a4133300e837d53f42c77599723f2e37c80a837caee9ff0e`. Its original source remains in `target/smiles-acyclic-conversion/first-profile-declaration.rs`.

- **Intended runtime RED, handle48122, exit101:** the existing `Command::parse` rejects the unknown `profile` field; one test fails. No file was accessed and no missing Rust API caused this failure. Log `first-runtime-red.log`, 5,035 bytes, SHA-256 `042341e2e0e86f20ebd7ba8069674f3685c7434514939bb2396c79079eca7a23`.
- **Declaration compile, handle5334, exit0:** the expanded five-case command test compiles against the unchanged baseline using only actual existing APIs. This is not behavioral GREEN.
- **First adapter run, handle10135, exit101:** stdio fails at the absent profile schema as intended. The authenticated case stops earlier on an unsupported `expected_revision` key mistakenly included in the new test's `bond.create` setup. This is a test setup error, not conversion failure. Original 10,421-byte wire source and log remain archived.
- **Source-backed setup correction:** existing `BondCreate` has only order/page/endpoints. Remove only the extra setup key, preserving the expected created revision1, dirty state and all converter assertions. The inverse is recorded in `wire-setup-correction.json`; no production change is made.
- **Intended authenticated runtime RED, handle30646, exit101:** after that setup correction, the real authenticated conversion receives `invalid_params` for unknown `profile`. Log `auth-profile-runtime-red.log`, 1,423 bytes, SHA-256 `6113d12be5839a90e5a817dd7333e1f637b379ed8714f0f52c456f8fddd9930e`.

Root approved rustfmt-only formatting of the new declaration before final integration. Its formatted prefix is 828 bytes, SHA-256 `9352b7fd1c6ebbdfcfe2b829a950535448824d0480495c912ee0ab8be654816f`; substituting the archived 806-byte prefix reconstructs the pre-format test exactly. No assertion changed. The diff remains in `original806-formatting.diff`.

One additional source-reviewed setup correction moves the exact same `!finish_ui_file(id,false)` assertion after binder-refusal/state checks. The established API dismisses the presentation even when it returns false; checking a binder against that already-dismissed ID would test a stale request instead. Original source and exact move are archived in `sequence-correction.json`. This correction precedes any backend execution and is not a product failure.

## Consumers, preservation and execution limits

The received new `smiles_acyclic_oracle.py` is owned/qualified independently by the IO acceptance worker. Its initial 7,498-byte identity is SHA-256 `d877bb2e7faeeccb66d253c4b514e635893cbf5ab400172b2859bf2e8f9c0475`. Root approved the exact 78-byte connectivity assertion addition: that 7,576-byte consumer is SHA-256 `c745f25d72823b2e492c3f71a43d20dc72d4fed4641ba548276de121dc7cb60a`. It rejects a cycle plus isolated vertex even when the total edge count equals atoms-minus-one. Original bytes and the exact one-line diff are retained.

The independently qualified final consumer is 10,090 bytes, SHA-256 `190628332cb63f1287edd4debcdff1ebaeaf451fa885ac965f06229080789a0a`. Its earlier blanket `STEREONONE` contract was incorrect for the source-unspecified internal `CC=CC` collinear depiction: RDKit reports unknown `STEREOANY`, not assigned E/Z. The correction allows ANY only for the independently checked source context with antiparallel raw-Decimal substituents at both ends; assigned bond or atom stereo still refuses. It keeps truthful `stereo_none` and adds `no_assigned_stereo` and actual `bond_stereo` metadata.

None of this command/wire suite's positive vectors has that internal degree-two/degree-two Double. Both new expected-result objects therefore keep `stereo_none:true` and add only `no_assigned_stereo:true` plus one literal `STEREONONE` per independently declared bond. Before/after source, exact diffs and verified inverses are in `oracle-output-correction/`; every original graph/H/formula/source/history/lease assertion remains exact. This new-consumer correction precedes the production snapshot and is not a product fix or an independent run of the owner's qualification controls. All previous consumers, fixtures, fonts and approved golden remain protected.

External consumers are invoked with `BONDLINE_PYTHON`, `-B` and actual emitted files. Missing consumers fail. Source bytes are independently literal; no whole-file property dictionary or first-record projection is used. Molecular correctness and actual pixel agreement are separate checks; matching CPU/GPU PNG bytes is not a new human-approved golden.

The ignored `protected-baseline.json` records 924 existing source/configuration/test/fixture identities. The existing CLI aggregate changes only by its explicit 77-byte child-module suffix; its entire old prefix is archived and checked. Root-approved production/oracle ports must be listed separately from author files and this old baseline.

All Cargo commands use this worktree's absolute `target`, `CARGO_INCREMENTAL=0`, dev/test debug0 and the pinned external Python environment. No Git operation, native GUI/input, Actions dispatch, dependency/lock change or old-test/fixture/oracle rewrite was performed.

## Exact received backend and independent GREEN

Root's `root-product-snapshot.json` is 1,815 bytes, SHA-256 `ea701d056ba66fd2372c42b863816fb71549031fbce47f59c303d46700262d40`. Before copying, every announced source hash and each existing worktree baseline were verified and archived. Nine listed command/IO/render files were received; the UI-only file was deliberately omitted. This is an explicit received replacement of eight baseline source files plus one new layout module, not a claim that every old source hash stayed unchanged. No producer code was edited here.

The final 20,723-byte command tests (`ee5dbba0ac4c288e093fa3cd68b288ec50613ca82bcdcaeaccba4ac69060cbd5`), 10,532-byte wire tests (`a7748fdabdb682af51a5f79a1ffc68fcb14147adb6389bc9c477c2516123234a`) and final qualified oracle stayed exact for all receiving runs:

| Actual independent command | Handle / result | Log SHA-256 |
| --- | --- | --- |
| `cargo test -p bondline-commands --locked --test smiles_acyclic_conversion` | 88997, exit0; **5 passed/0 failed/0 ignored** | `70a4c2ee1b1b32c0554dea3b1e843e563a2a2c4167c789e279ed8441c9de98dd` |
| `cargo test -p bondline-cli --locked --test sdf_capture_adapters acyclic_conversion_wire` | 81769, exit0; **2 passed/0 failed/0 ignored**, 11 unrelated cases filtered | `6f925ea3f368a9d7a59b8aefebdab4df119fec2c66eae6fd22623c6c3d367b24` |
| `cargo clippy -p bondline-commands -p bondline-cli --all-targets --locked -- -D warnings` | 25193, exit0 | `ce56ce94c0d35fafe492e13b0ca2034cfb84263c255a2b5931da709224a354f0` |
| `cargo fmt -p bondline-commands -p bondline-cli -- --check` | terminal exit0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` (empty log) |

Logs are retained respectively as `five-core-first-receiving.log`, `two-wire-first-receiving.log`, `commands-cli-clippy.log` and `commands-cli-fmt.log`. The first receiving runs pass without any further test, input, timeout, threshold, oracle or production change. Exact source/owner retention, historical receipts, publication reservations, native-binder refusal, actual external files and the one CPU/GPU comparison are assertions within those seven cases, not separate inflated test counts.

Full workspace/integration testing remains Root's responsibility; no full package, native, hosted, Windows/Linux interaction, fuzz, general aromatic/stereo import or new golden approval is claimed here. The bounded neutral-acyclic profile remains partial M1. Final author/source/log identities and all preserved baseline checks are sealed in the ignored `target/smiles-acyclic-conversion/final-handoff.json`; all owned jobs are terminal.
