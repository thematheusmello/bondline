# Drawing-settings data: receiving record

2026-11-16, contract538df6c on path/preview base65af201, `feat/multiple-bonds`.
The full M0–M5 goal remains active and incomplete. This increment stores supported root
creation preferences; it does not deliver a settings command/form, effective inherited
styles or native tool application. Registry40, controls31/20/14/10 and catalogue12
delivered/128 disabled remain unchanged. Both chain controls stay grey. Actions remains
OFF; no hosted run, push or publication.

## Scope and independent expectations

[The locked contract](research/drawing-settings-data.md) cites the public SDK property,
numeric-type and published scalar-comparison pages, plus GUIDE25. XML uses unscaled
points/degrees; binary values are signed little-endian fixed16.16. No absent numeric
default is inferred. Original tests author their own scalar vectors and CDX envelopes;
no linked vendor sample/binary/image is downloaded or copied.

`DrawingSettings` is private validated Copy data with two independent optional values.
Empty omission preserves old document JSON/native-v0 bytes. Root projection accepts
supported values independently; invalid/nonfinite/out-of-subset/ambiguous values remain
in authoritative source without new hard Open failures. Nested values are not flattened.
The existing CDX child containment/property sets, framing and incomplete authority remain.
Exact full native XML re-encoding alone establishes regeneration authority.

Whole-document MOL V2000/V3000 now refuses any nonempty settings before raw/fragment
reuse or publication; existing forbidden-version precedence stays first. Existing whole
model equality blocks retained XML/CDX/MOL and synchronized SMILES after style-only drift.
Explicit fragment SMILES export keeps its existing declared geometry/document-context
loss. Existing pages, positions, H, shared fragments and scene stay unchanged.

Old graph/RDKit consumers ignore root settings and cannot establish their preservation.
The new small `crates/io/tests/drawing_settings_root.py` uses Expat ordered root attributes
and Decimal, disables parameter entities and supplies no defaults. Manual source, changed
scalar and nested controls qualify it before it reads producer output. Its SHA256 is
`c8c20e1c83c0ecb69bf8dbd0f02e803f44566d291a53806860e4924057fd586f`.
Unchanged graph/chemical consumers remain separately scoped.

## Actual RED, GREEN and focused checks

Root owns all executions and integration. Workers supply ignored declarations/source
reviews only; no worker test/build/native execution is inferred.

| Execution | Actual result |
|---|---|
| XML82691, first existing-API test | RED0/1: independently authored root24.5pt/120.5° loads, but serialized model settings are Null. This precedes new types/producer. |
| CDX3733, first existing-API test | RED0/1: original explicit envelope with raw length3178496 (`00 80 30 00`) and angle9011200 (`00 80 89 00`) lacks model48.5/137.5. |
| Producer98821 | GREEN2/0 for both unchanged first runtime expectations. |
| Model85537 | 3/0: independent presence, strict map JSON/ranges/types/arrays, old JSON, equality/Arc/fixed-size accounting. |
| IO57736 | 9/0: XML4, CDX4, CDX style drift1. Canonical/partial/invalid/nested/opaque values, native classification, current/original separation, explicit/implicit roots, duplicate/width/sign/nonroot behavior and exact passthrough/restoration. |
| Loss71217 | 4/0: both MOL versions, source-free/native/MOL style drift, unchanged-fragment reuse, existing/absent atomic destinations, retained CDX/SMILES drift and unchanged model/scene. Positive unstyled preconditions prevent older guards from masking the new loss guard. |
| Semantic export70588 | 1/0: actual registered fragment export preserves state/source/Arc, exact `[CH3][CH3]\n`, old declared loss and unchanged external C2H6/CC interpretation. |

There are17 new focused cases; the first two are included in later expanded targets.
New-type cases are post-producer acceptance, not invented missing-export compile REDs.
No test syntax/setup correction or weakened behavioral predicate is required here.

Reproduction, using the pinned oracle environment described in `fuzz/README.md`:

```sh
export BONDLINE_PYTHON="$PWD/.cache/oracles/format-python/bin/python"
export CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
cargo test -p bondline-model --test drawing_settings_model --locked
cargo test -p bondline-io --test drawing_settings --test cdx_drawing_settings --test cdx_style_drift --test drawing_settings_loss --locked
cargo test -p bondline-commands --test drawing_settings_semantic_export --locked
```

Source-only producer review covers eight Rust files plus the locked contract at a sealed
pre-format read point and finds no material issue. A separate qualification finds only
formatting and reordering unchanged imports/declarations. The eight producer Rust files
still match; the later contract edit adds its receiving link without changing boundaries.
Review4039B SHA256
`d30e8b4e9854a4c655843a8d8427746003c4a24ac0592c1c333812dfaf48181a`;
qualification1663B SHA256
`d96e8b3a115ca40d78844feeb1eeb3e7b48da83be5440e149256fcc53d73e97d`.
These are source evidence, not independent execution or native acceptance.

## Full gate failure and separately completed stages

Root's single `cargo xtask ci` job94003 exits1:998 passed/1 failed/0 ignored across156
test-result lines. Formatting and warnings-denied workspace clippy pass before the
unchanged MCP test `shutdown_without_clients_releases_the_listener_address` fails its
one replacement bind at127.0.0.1:58634, AddrInUse/OS48. No unchanged retry is run. Neither
assertions, deadlines, concurrency, reuse policy nor the protected test file changes.

Only stages not started by that aggregate run are then executed:

```sh
cargo test -p bondline-mcp --test ui_capture_work --test ui_sdf_remove --test ui_sdf_work --test ui_smiles_conversion --test ui_smiles_paste --test ui_smiles_records --test ui_smiles_work --test view_control --locked
cargo test -p bondline-model -p bondline-render -p bondline-ui -p xtask --lib --bins --tests --locked
cargo test --workspace --doc --locked
cargo test -p accesskit_macos --lib --locked
cargo clippy -p accesskit_macos --all-targets --locked -- -D warnings
cargo xtask layers
cargo xtask licenses
cargo xtask corpus
```

Job61803 exits0:583/0/1 across71 result lines; vendor lint, layers, seven-asset strict
licenses, unchanged155/166/221 runtime closures and pinned corpus pass. A final target
coverage audit identifies three workspace compatibility packages whose runtime tests had
not started. Only those are then run in22798, exiting0 with9/0/0:

```sh
cargo test -p clipboard-win -p hexf-parse -p libloading --lib --bins --tests --locked
```

Combined remaining592/0/1 is **not a green aggregate**. All covered results stay separate
from94003. One post-terminal `lsof -nP -iTCP:58634` exits1 with empty output; this is not
failure-time socket/process identity or causal evidence. The cause remains unattributed.
No consumed probe/exhausted diagnostic context is rerun. The collector in
[the listener investigation](research/macos-listener-lifetime.md) remains designed only.

Ignored logs under `target/drawing-style/root-receiving`:

| Artifact | Bytes | SHA256 |
|---|---:|---|
| ci.log | 124425 | `79d4749e2941a361845c481e3a5a8ff7a64b88a5bff5dadf3974ed53a3f25962` |
| remaining-stages.log | 75514 | `d3fece4ce4b53ca283fed9e1171d9cb9fa266d31929d733e34c460cbb04f3895` |
| remaining-compat.log | 1599 | `dedc0ae72044e4e86f3d9d1bfaceaeab567a46e5854b036abdf2eda28e029148` |

## Bounded projection fuzzing

Existing unchanged `cdxml_parse`/`cdx_parse` harnesses exercise StoredDocument projection
and their source/native-edit checks. Fresh ignored caches contain10 original XML seeds
(4415B) and16 original binary seeds (860B): absent/one/both, alternate/invalid/nonfinite
XML, nested/opaque XML, explicit/implicit CDX, duplicate/wrong-width/signed/out-of-domain
binary values. They use Bondline's original fixture or manually authored standard
envelopes; committed fixtures are not mutated. Seed manifest4188B SHA256
`8565b1c77af93eb18c8fc4520e1c274f1cc66b85644d8d4bc109e77503d616fc`.

Pinned nightly-2026-04-01, cargo-fuzz0.13.2/libfuzzer-sys0.4.13, ASan,2 jobs,
`-Cdebuginfo=0`, `-runs=10000 -max_len=65536 -timeout=5`:

| Job/target | Seed | Actual final result |
|---|---|---|
| 18309 cdxml_parse | 481720635 | 10000 runs/2s, cov3021/ft7902,333 retained entries/97KiB, adaptive limit467B, RSS170MiB. |
| 96727 cdx_parse | 309174826 | 10000 runs/1s, cov683/ft1518,85 retained entries/4545B, adaptive limit80B, RSS67MiB. |

Actual commands, with the development environment above plus
`CARGO_PROFILE_RELEASE_DEBUG=0`, `RUSTFLAGS="-Cdebuginfo=0"`,
`CARGO_TARGET_DIR="$PWD/target"` and `PATH="$PWD/target/tools/bin:$PATH"`:

```sh
cargo +nightly-2026-04-01 fuzz run --target-dir "$PWD/target" --sanitizer address cdxml_parse .cache/fuzz/drawing-style-xml-538df6c -- -runs=10000 -seed=481720635 -max_len=65536 -timeout=5
cargo +nightly-2026-04-01 fuzz run --target-dir "$PWD/target" --sanitizer address cdx_parse .cache/fuzz/drawing-style-cdx-538df6c -- -runs=10000 -seed=309174826 -max_len=65536 -timeout=5
```

No crash is found in these bounded runs. This does not exercise every allocation limit,
establish a parser/security audit or fuzz the new typed model JSON deserializer. No
harness, fuzz dependency/lock or old oracle changes. Exact invocations/logs remain in the
ignored receiving directory. XML log57749B SHA256
`e9d7b6b82bc45be6848fdf0625bf35d624ab032888b3d80efabba686b68254a0`;
CDX log13879B SHA256
`057d9571501aae744d8f3ef2e585f86720e21a67886af9757b4d913ea8c83c6a`.

## Actual build and Unicode CLI persistence

Job88017 `cargo build -p bondline-ui -p bondline-cli --locked` exits0. Native SHA256
`c0594a625dd17a65753b5fe9da71a78e8e5bee383477992799ba7f7894cab785`;
CLI `8273b9ffa418931cb0ad80deb4f0a5251158bf7b0d53bd4d116368ec85623dbf`.
No native launch, copied bundle refresh, golden, DPI or new platform claim.

Actual CLI uses independently authored `styled-原.cdxml` (the original native-v0 fixture
plus root24.5/120.5), then shared registered commands: Open, `bond.create` from(300,400)
to(348,400), Undo, Redo, Save to `edited-保存-λ.cdxml`, Inspect. The resulting revision4,
one undo/no redo, two fragments, dirtyfalse/source.modifiedtrue distinguish saved current
state from the original backing. Independent root Expat/Decimal confirms both exact
settings on the actual output. Reopen through the CLI gives the same current document,
clean source/baseline and empty history; the input stays byte-identical.

Unchanged `xtask/oracles/cdxml_graph.py` independently confirms both literal point pairs,
directed Single edges and H3/H3. External pinned RDKit2026.09.2 independently reads two
neutral nonradical C2H6/CC molecules. This chemical check is separate from root metadata.
Input445B SHA256`b9c291a197ae4bf842a959e5d441efe12936f1f25da3eeeb627a5f87fd2b8d63`;
output700B SHA256`2abe5f79e8e608aaec5810c30efef341c317296977af876d012b9e123ee0ea27`.
Ignored CLI identity1416B SHA256
`36489c36666a53a4b724dc53b08cbb070c429abad3694001553599a6242ea93a`;
independent graph1831B SHA256
`2066d7c910ed05010aa9563623a6e12ed5190a1bbdaacf1cf8ecdf2d195ca691`.

## Continuation

All Root/worker execution jobs are terminal. Protected user apps17806/96449 and their
copied bundles/drawings remain untouched; both processes are observed live and both
copied binary hashes match their protected pins. Current drawings are not re-inspected. Stored
preferences are verified data only. Next lock the source-qualified atomic settings
command, history and adapter behavior, then ordered native modifier transitions and
complete settings/chain workflows with actual native/platform acceptance. Existing
M0 addressability/inventory/platform, M1 interoperability and M2–M5 gaps remain required.
