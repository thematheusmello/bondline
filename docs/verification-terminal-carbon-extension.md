# Terminal-carbon extension: engine and headless verification

2026-11-14. Isolated `bondline-rxn`, branch `feat/terminal-carbon-extension`, base983ffee.
Original Rust code is MIT OR Apache-2.0. No renderer, native UI/input, codec, encoder,
dependency, lock, asset, pinned fixture or approved golden changed for this feature.
Its native gesture remains missing; no native runtime or platform workflow is claimed.

## Delivered operation

`bond.extend` is one required typed registry operation, discovered and called by the
existing CLI, stdio MCP and authenticated loopback adapters. Parameters are the closed
seven-field object:

```json
{"expected_revision":1,"page_id":1,"fragment_id":1,"begin_atom_id":2,
 "end_x_pt":180,"end_y_pt":240,"hydrogen_policy":"rectify_terminal_carbon"}
```

The starting atom must be positioned neutral carbon with literal H3, exactly one
Single C–C bond, and a selected fragment containing only positioned neutral carbon/
Single bonds. The explicit policy authors target H2, a new terminal carbon/H3 and one
Single bond, preserving the fragment and all old IDs/order/data. Other starting H
values/None, nonterminal starts, other elements, charges/isotopes/radicals and orders
refuse. Counts on other atoms remain literal and untouched; this operation does not
certify an arbitrary pre-existing molecule's chemistry or infer unknown H.

Source-free or byte-proven complete native CDXML is allowed. MOL/CDX source backing
is refused independently of associated filename/format or drawability. Source-free
data saved to MOL retains its authority; reopening that file creates actual MOL backing
and refuses extension. No source is detached, rebuilt from an incomplete projection,
or relabeled. Native-v0 recognition and guarded export remain unchanged.

New atom/bond IDs exceed document-wide maxima and session floors. No FragmentId is
allocated; fragmentID=u32MAX alone succeeds. Revisions, finite point bounds and exact
occupied-atom endpoints are validated before commit. Failure retains full state,
saved/source baselines, redo and ID floors. `Document::replace_fragment` reuses the
same global validator as append, excludes the old fragment from counts and atomically
retains unrelated Arc payloads/order. No heap-owning model field was added.

## Actual test-first evidence

Logs are ignored in this checkout's own `target/`:

- `terminal-model-red.log`: `cargo test -p bondline-model --test replace_fragment
  --locked` exited101 because the required atomic replacement API did not exist.
  Tests use independent literal ethane/propane connectivity, failure/collision/budget
  expectations and unrelated shared-payload identity.
- `terminal-command-red.log`: before production changes, the independent literal
  propane/history test ran and failed `UnknownCommand("bond.extend")`, exit101.
  Expected geometry is(100,200)→(140,200)→(180,240), one fragment, H3/2/3 and two
  authored Single bonds; it is not a round trip through our writer/reader.
- `terminal-first-green.log`: minimal model/command implementation passed all3 model
  replacement and then6 command feature tests. Further negative, property, schema,
  close/file-baseline and actual adapter/consumer coverage was added afterwards.

A first new saved-file oracle incorrectly assumed A4 for the pinned native-v0 fixture;
that fixture explicitly says612×792 points. The failed run is preserved in
`terminal-files-green.log` (despite that early filename, its result is FAILED).
The oracle now requires the independently selected paper profile and checks exact
612×792 for native-v0 and analytic A4 for a new source-free document. No product/fixture,
geometry assertion or tolerance was changed to hide this error. A new property-test
borrow needed a local Cow binding; a legacy CLI discovery count needed14 instead of13.
Those test corrections are separate from the substantive RED evidence above.

## Independent consumers and guards

`commands/tests/terminal_chain_oracle.py` independently reads **actual published file
bytes** with Expat and Decimal: one page/fragment, explicit paper, unique IDs, node
attributes, authored H, literal geometry and bond references/order. It then requires
external RDKit2026.09.2 to consume that same CDXML as propane/butane, neutral carbons,
H3/2/3 or H3/2/2/3, Single connectivity, CCC/C3H8 or CCCC/C4H10. Missing RDKit fails;
the consumer is not linked into the product. The old ethane oracle is unchanged.

Actual executable CLI, stdio MCP and authenticated loopback tests create→extend→save,
consume saved bytes independently, reject stale requests and verify Undo/Redo. CLI
and engine tests reopen the saved propane, extend its current terminal to butane and
consume the new saved bytes. Existing FilePlan/Completed publication and save-slot
ownership are reused. A Save prepared before an extension publishes its captured
ethane and leaves newer work dirty; another Save stays busy through completion.
A prepared Open becomes stale after an extension. Old close approval cannot return
after extension/Undo; committed close refuses the new command too.

Coverage also includes required policy/schema and unknown/missing fields; missing or
zero IDs; wrong fragment; stale revisions; coincident/nonfinite/out-of-range/free-end
geometry; literal-H overrides/unknowns; unsupported metadata/order; atom/bond ID and
revision exhaustion; fragmentMAX success; global graph-item budget; fresh branch IDs;
unrelated authored-H and Arc preservation; exact native-v0/source retention and saved
dirty versus original-source modified baselines. Seeded property tests exercise128
sets of native-backed extension sequences and exact full-snapshot Undo/Redo.

Application IDs remain stable inside the session/history. The existing codec assigns
file IDs and dense IDs on reading; this feature does not promise unchanged internal
IDs across reopening a branched file with gaps. Native gesture target rediscovery is
still part of the separate UI slice.

## Reproducible checks and remaining scope

All Cargo calls use this checkout's absolute `CARGO_TARGET_DIR` ending in
`bondline-rxn/target`, `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0`,
`CARGO_PROFILE_TEST_DEBUG=0`, and the documented isolated root `BONDLINE_PYTHON`.

```sh
cargo test -p bondline-model -p bondline-commands -p bondline-cli -p bondline-mcp --locked
cargo clippy -p bondline-model -p bondline-commands -p bondline-cli -p bondline-mcp --all-targets --locked -- -D warnings
cargo fmt --all -- --check
```

The scoped headless run passed161 tests/0 failures/0 skipped across29 reported suites
before the final close/schema additions (`terminal-headless-tests-green.log`). The
final close test separately passed alongside the11 existing close tests in
`terminal-close-final-tests.log`; all10 focused extension and3 replacement tests pass
in `terminal-final-scoped-tests.log`. Scoped all-target clippy passed in
`terminal-headless-clippy.log`. The final aggregate below includes every final case.

A manual own-checkout executable invocation also saved a566-byte propane file,
SHA256 `d57932ca4005e1ccd39857095426ca438198cdfc0ad49e834c9e3c6131e879f7`.
`target/terminal-cli-evidence.json` records its actual14-command discovery/schema,
seven required fields, typed closed policy, exact binary hash, result state and
successful raw/Decimal/RDKit consumer. This is headless execution, not native UI.

The parent983ffee hosted run exposed a separate test-only FreeType layout-oracle
portability failure. Root applied its independently verified correctiona3613c0 here
after checking both prior test files against983ffee. Its two render test files and
verification-label-oracle-layout.md are inherited root-owned evidence, not this
feature's implementation; the production renderer/font/golden and0.90 threshold are
unchanged. The additive font source record is kept separate from this operation.

With that separate correction applied, this checkout's actual `cargo xtask ci` passes:
498 workspace tests plus3 native bridge tests,0 failures/0 ignored; rustfmt; warnings-
denied workspace all-target clippy; layers; unchanged154/165/220 declared runtime
closures and2 bundled assets;5 pinned files/4 source bundles with the existing raw/
external corpus checks. `target/terminal-extension-ci.log` prints the correct absolute
`bondline-rxn` root and ends successfully, session82702 terminal0, SHA256
`79d47a5cc3a5f5b17a6ae0a2b73a1fdbc462c868d197ee58e82a7324fac31eef`.
The21 added model/command/headless tests are included; a separate deliberately mutated
H2→H3 output is rejected by the new independent oracle. The manual566-byte saved file
and native-v0/approved golden pins remain exact.

This source still needs receiving integration, independent review and its own hosted
checks. No M0/M1/M2 milestone, general chemistry, native connected drawing, branch/
ring/merge, other bond tools or full reference parity is claimed. Use the approved
next-step plan for fresh per-press selection, raw press versus snapped anchor,
ordered hit-scene reacquisition and native acceptance after this engine slice is
integrated. No owned jobs remain running at this handoff.


## Receiving CDX/connected-command integration

In the isolated bondline-rxn checkout, feat/connected-bond-integration merges9162a1e
with450962c, retaining both CDX commands and bond.extend (16 total). Six merge conflicts
are limited to both test-helper imports, two adjacent whole new tests, discovery's
14/15→16 count, and additive source/open-question records. Both complete tests and all
exact command names are retained. Formatting orders the two helper modules; production
model/extension source matches the frozen feature handoff. No UI/render/capture/codec,
asset, dependency/lock, native-v0 or approved golden is changed by this engine feature.

The receiving cargo xtask ci passes520 workspace plus3 native bridge tests,523 total,
session54630 exit0. target/connected-bond-integrated-ci.log SHA
 dfb35244cdbdf58d69d40a9fa98d89d278054930292a2f2e561290d00b274e8d
prints the correct absolute bondline-rxn workspace and includes fmt/all-target clippy,
strict layers, unchanged license closures/assets and independent corpus consumers.
The concurrent actual CDX native acceptance runs the separate frozen Root450962c binary;
it does not test this engine extension or native attachment. Own hosted evidence remains
pending, and the next native gesture must use the same required-policy registry command.


Independent read-only integration review found no confirmed actionable defect. Its report
in bondline-rxn/target/terminal-extension-integration-review.md hashes to
cc41dee7859ed1ae5c52e40b52f360da1cec1a46115fb0119068517f413c355a (7375B); the reviewer
traced production/callers/tests without rerunning Cargo or native UI. Exact source501a315
now also passes hosted43560603081: Mac523/Linux521/Windows513, all actual window smokes,
and six10000-run parser ASAN targets. ci.md records job IDs/adapters. This is engine/headless
extension evidence; native attachment is separate work, not established by a window smoke.
