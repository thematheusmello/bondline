# Bond-order edit command acceptance

The frozen `bond.set_order` acceptance now passes 13 command/file tests, four
CLI tests and two stdio MCP tests, plus warnings-denied all-target clippy and
rustfmt. The actual baseline on supplied base `1322290` rejected the command
with `unknown_command`; all behavior expectations stayed byte-identical after
Root supplied the real backend. This is headless evidence, not native UI or
complete connected-editing parity.

## Frozen behavior contract

The closed JSON object requires `expected_revision`, positive `page_id`,
`fragment_id`, `bond_id`, `order` (single/double/triple), and the explicit
`hydrogen_policy: "rectify_neutral_carbon"`. It edits the existing bond, with no
identity allocation or position/direction change. Affected endpoints must be
neutral, nonisotopic, nonradical carbon with exact literal H consistent with
four minus the sum of existing incident Single/Double/Triple weights. The new
weight must fit four. Only the selected bond and its two endpoint H counts change.

Position None remains None and is supported: this command edits graph data,
not geometry. Unaffected atoms' elements, isotope/charge/radical metadata,
positions and literal/unknown H are preserved. Source-free and byte-proven native
CDXML data are eligible; actual foreign/source-incomplete or collection content
is refused atomically. A validated same-order request preserves the whole content
owner, model Arc, revision, history, redo, saved state and file association.

These are original Bondline authoring/authority policies. Existing provenance
entries [OpenSMILES](https://github.com/opensmiles/OpenSMILES/blob/9240a2887158699503605bc838e1de9a9f7883f4/opensmiles.asciidoc),
[BIOVIA CTfile](https://www.3ds.com/assets/invest/2026-01/biovia_chemistry_2026_ctfile_formats.pdf) and
[public CDXML bond orders](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Bond_Order.htm)
provide chemical/format context, not this command's API or a universal valence engine.
No reference binary/runtime, asset, template or help prose was used.

## Independent tests and intended evidence

`crates/commands/tests/bond_set_order.rs` contains 13 tests:

- A full literal Single→Double graph and history expectation, retained from the
  first actual RED. All nine pair conversions additionally use nonsequential
  fragment/atom/bond IDs and a reversed authored bond direction.
- Internal C4 chain and four-carbon ring edits with fixed endpoint H vectors,
  retaining all other bonds and geometry. A separate positive case has absent
  positions and an unaffected isotopic/charged/radical oxygen with unknown H;
  an unrelated fragment retains its exact Arc.
- Same-order sharing with a live redo branch; closed/required parameters, invalid
  policy/order/types/IDs, stale revision and missing targets; both endpoint sides
  with ten invalid chemistry/H cases, including validated same-order refusals.
- Every currently modelled non-Single/Double/Triple incident order; second-endpoint
  overvalence and inconsistent old valence; complete state/content/stamp equality
  on failure, preventing partial first-endpoint mutation.
- Unknown CDXML metadata, retained CDX, actual MOL backing and SDF/SMILES collections
  refuse. A source-free model merely associated with a MOL file remains eligible.
- All nine native-CDXML reopen/edit workflows save actual Unicode CDXML plus
  V2000 and V3000 files. Expat independently checks literal XML nodes, coordinates,
  H, IDs/references and order. Pinned external RDKit consumes those exact CDXML
  bytes. The existing unchanged `io/tests/mol_oracle.py` consumes each real MOL
  output, checking literal C2 formula/H/order/coordinates expectations. Reopened
  native/model data and source bytes are compared separately.
- The protected native-v0 source restores exactly after Undo. A prepared Save
  retains its old Single input while Double is authored; its save lease lasts
  through finish, and stale Open cannot replace a newer Triple edit.
- A following create checks that no IDs were allocated. Registry discovery
  requires the exact six fields and a closed, mutating command. A fixed-seed
  128-case property test checks internal order sequences and exact complete
  snapshots through immediate and full Undo/Redo.

The C2 expectations are literal `CC/C2H6/[3,3]`, `C=C/C2H4/[2,2]`, and
`C#C/C2H2/[1,1]`. Internal chain endpoint H is `[2,2]`, `[1,1]`, or `[0,0]`
with unchanged terminal H3. They are not copied from actual command output or
computed by the implementation under test. Consumers use the actual saved
bytes; a self-round-trip alone is not interoperability evidence. Geometry and
source authority remain independent checks. No renderer/native success is inferred
for ring strain, absent coordinates or other graph-only cases.

## Observed REDs and harness checks

First focused run, session 46628, compiled the new behavioral test and failed
at runtime with `UnknownCommand("bond.set_order")` (0 pass, one fail). Its exact
first test and log are archived under ignored `target/bond-set-order-first-red.*`.

The expanded draft initially failed to compile because the test used a nonexistent
`PageSize::LETTER` and two boxed-error conversions in its proptest helper. Those
harness errors were corrected to the actual constructor and explicit TestCaseError
mapping. This compiler failure (session 82297) is retained separately; it is not a
product behavior RED and no production source changed.

The complete 13-test target then compiled and failed as expected against the
unchanged missing command (session 81783, 0 pass/13 fail). Negative tests report
`unknown_command` instead of their required specific error; the property also
fails at the missing command. Final rustfmt check passed. This baseline establishes
registry absence, not that every future assertion has already executed.

```text
cargo +1.94.1 test --locked -p bondline-commands --test bond_set_order
cargo +1.94.1 fmt -p bondline-commands -- --check
```

Runs use this worktree's absolute target, Rust1.94.1, incremental0, dev/test debug0,
and Root's existing pinned `BONDLINE_PYTHON`. The frozen ignored handoff records
source/log hashes and all 316 protected pre-existing identities. No pre-existing
source, parser/writer, fixture, oracle, golden, dependency or lock is changed.

## Actual GREEN and delivered adapter coverage

Root's backend port receipt `target/bond-order-backend-port.json` has SHA-256
`f68edf8a593c9b28adbd1d234e9fcb5aafe9f491f8a14f2cc4521e5636426aa4`.
All 12 received files matched both that receipt and Root's actual bytes before
execution. Production `commands/src/bond.rs` is SHA-256
`c6d599b6dde2566b0d6d35b64f94de330f7180e856b2f3d8b5a725ccedd9c070`;
`commands/src/lib.rs` is
`13530ab57d57b2e2b318cb1c7ecf79e11393d8029c454b512b5727a666fbc0fb`.
The other received changes are Root's registry catalogue expectations and the
minimal two-line private MCP Client bridge. All 304 unported original identities
remain exact. This worker made no runtime, existing-test, fixture, oracle, font,
golden, dependency or lock changes.

`cli/tests/bond_order_adapters.rs` exercises actual processes: discovery/closed
schema, Single→Double→Triple authoring through history/native reopen, both MOL
versions consumed by the unchanged external oracle, Unicode files, validated
no-op/redo, typed refusals and absence of later output after rejected commands.
`cli/tests/support/bond_order.rs`, through the existing Client, verifies actual
stdio calls and text/structured-result agreement, mutating MCP annotation,
required fields, history/no-op/stale/invalid errors, native-source continuation,
real molecular outputs, overvalence and foreign MOL refusal with unchanged state.
No replacement protocol harness or new command adapter was added.

Before backend reception, actual CLI session 33562 failed all four tests on
`unknown_command` or missing discovery. Stdio session 78485 failed both new
cases with actual JSON-RPC -32601 / unknown Bondline tool. Those RED logs and
the exact pre-backend adapter hashes remain in the ignored adapter handoff.
A draft review before this freeze made the foreign-source refusal compare
Inspect-before with Inspect-after; Open operation results have different
operation metadata. No expectation changed after the frozen adapter RED.

After the exact backend port:

- Session 56950: 13 command/file tests pass, including all nine real CDXML/MOL
  workflows and the fixed-seed 128-case property.
- Session 36120: all four CLI adapter tests pass.
- Session 4648: both new stdio tests pass, with 15 unrelated cases filtered out.
- Session 77820: commands and CLI all-target clippy exits zero.
- Session 61405: relevant commands/CLI rustfmt check exits zero.

```text
cargo +1.94.1 test --locked -p bondline-commands --test bond_set_order
cargo +1.94.1 test --locked -p bondline-cli --test bond_order_adapters
cargo +1.94.1 test --locked -p bondline-cli --test mcp bond_order::
cargo +1.94.1 clippy --locked -p bondline-commands -p bondline-cli --all-targets -- -D warnings
cargo +1.94.1 fmt -p bondline-commands -p bondline-cli -- --check
```

No expectation or production correction was needed for these GREEN runs.
The command tests remain 30,051 bytes / SHA-256
`59337ce15ae9c4fa336a23deeea16ead63bc4f4ce5a8f3178a883705e52d5b92`;
CLI tests remain 7,952 bytes /
`636dd005ede28a1d86a28429c1f84dd69f012f0ba3c181fca0b707d56d11b7f1`;
stdio child tests remain 7,667 bytes /
`2ad4b149ad0230ea7b309f5ba4cbec3436cbf22253083016e41bbf5a807a1d51`.
All earlier RED/harness-failure receipts remain unchanged. The final ignored
handoff includes source, consumer/fixture, Root-port, log and prior-receipt hashes.

No local full workspace, Windows/Linux, hosted or native result is claimed by
this worker. Root owns the receiving aggregate and Git integration. Mapped MOL/CDX
backing remains ineligible for this edit, and wider chemistry/source editing/UI
flows remain separate. GitHub Actions stays off. No Git/GUI/input/activation action
occurred; all owned jobs have exited.


| Final ignored log | Bytes | SHA-256 |
|---|---:|---|
| `bond-set-order-command-green.log` | 1,514 | `6efbca063735f18f9d9a78fab45596f3f68d8bc8939d27f2b22a065b521a99d9` |
| `bond-set-order-cli-green.log` | 937 | `a333025bc692b1757aa909f58326339ef0de787374c88957ed31baa7953b7a17` |
| `bond-set-order-stdio-green.log` | 563 | `d990c4037aa5ef1410e2fcbc1e433d6efa1af9a6c13e4e558fbbaa1be103fceb` |
| `bond-set-order-clippy.log` | 4,680 | `d9e1c1852228047fe80e13882da9ad5c9b9bc74a986945767a60624e03e19791` |
| `bond-set-order-final-fmt.log` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
