# Window view command and adapter acceptance

Final source port: all6 command,2 CLI/stdio,1 authenticated socket and2 supplementary
private guard cases pass independently. The10 actual-frame UI and2 direct SDK cases
also pass; see [verification-view-ui.md](verification-view-ui.md). Workspace formatting
and relevant all-target warnings-denied clippy pass. These23 focused cases are separate
from Root's aggregate workspace and native gates.

Independent tests started at `2ac7739` in the separate `bondline-view-acceptance`
worktree, 2026-11-15. This record concerns window navigation, not authored geometry,
general selection, chemical calculations or complete reference parity. Root owns
production and integration. No native window, user input or GitHub Actions was used.

## Evidence and original policies

The public [vendor-authored 25 guide](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf),
printed26, describes appearance-only zoom and percentage entry1–999; printed79 lists
Actual F5, Fit F6, Magnify F7 and Reduce F8. The reviewed source is17,721,447 bytes,
SHA-256 `3f1ebc429df07efdd47b6218a148dc346a215043f60bf89ee9a1cb90629bd5b3`.
These are version25 facts, not measured26.1 defaults. The research record is
[view-navigation-plan.md](research/view-navigation-plan.md).

The locked `ui.view` contract is original Bondline policy:100% means one logical
canvas unit per authored point; the step factor is1.25; Fit clears explicit zoom and
center; Actual and percent changes preserve center. Relative operations require
explicit bounded logical viewport dimensions. Absolute pan is bounded by the first
page and requires Zoom mode. Stored camera state survives authoring history and Open;
projection alone clamps it to a replacement page. Only Magnify can adopt a validated
local Create/Extend midpoint. None of these numeric/anchoring/persistence choices is
presented as a vendor default.

`ui.view` uses a closed tagged object. The optional five-field `ui_view_operation`
reports explicit nulls for derived Fit/page center, the checked epoch and transient
change status. Outer `changed` remains false. Ordinary document inspection omits
this explicit operation result. Actual source snapshots, revision, history, saved
state, file association, dirty flag and source-based capture stamps must be unchanged.

## Tests and RED evidence

`crates/commands/tests/view_state.rs` uses public dispatch and independent literal
expectations, without new production Rust types. Its six cases cover:

- Exact modes, center, epochs and no-ops while preserving complete content/model
  identity and CaptureStamp validity.
- Closed/ranged parameters, inclusive page bounds and busy/source guards before
  no-op; pure inspection has its separately specified close/picker policy.
- Fit resolution from a declared500-point/1000-unit vector, relative stepping and
  directional limits that cannot reverse an operation.
- Local drawing focus versus camera/history/Open lifetimes, including own order
  conversion and an imported smaller page.
- Complete source preservation, unchanged whole-page CPU capture and exact native
  and MOL Save bytes; wrong-kind and opaque-source refusals.
- Unused fields on every empty action, which must be rejected before epoch changes.

The initial3,173-byte test was frozen at SHA-256
`362b6838011671fd205f2f8d579f8a146ee4f8f0b8bca7f2a92500373db85e44`.
The five-test15,565-byte source retained that prefix exactly, SHA-256
`6a2f931a02436af8b3c56c25ba2080a3385f61a4207ff0bcd508c45819af2fc7`.
The sixth was additive:16,306 bytes, SHA-256
`102789287d8e2f47ede391cca0eb739978b149f404024e622b4f43d798b5afb5`.

Independent old-source executions all exited101 with the intended missing command:

| Session | Command | Result | Ignored log |
|---|---|---|---|
|44609|`cargo test -p bondline-commands --test view_state --locked`|0/1; UnknownCommand|`target/view-dispatch-initial-red.log`|
|3662|same, additive five|0/5; UnknownCommand|`target/view-state-five-red.log`|
|4785|same, additive six|0/6; UnknownCommand|`target/view-state-six-red.log`|

Root separately observed the five-test minimal backend failure where pinned Serde
unit action variants accepted extra fields. It fixed the actions to empty struct
variants with unchanged wire JSON. The sixth test explicitly protects all three
empty actions; no unknown-field expectation was relaxed.

New adapter tests use the actual CLI executable, stdio MCP protocol and an
authenticated loopback server. They require conservative mutating-tool metadata,
matching text/structured responses, exact camera results, closed-parameter errors,
private unchanged authored state across requests, shared camera state across clients,
and notification only for a real camera change. Socket tests never record a token.
The CLI/stdio old-source baseline45474 exited101,0/2: command/tool discovery absent.
Its log is `target/view-adapters-cli-red.log`.
The authenticated loopback baseline74709 exited101,0/1: the real server returned
`unknown_command`. Its log is `target/view-adapters-control-red.log`.

Before receiving GREEN, independent source review found one schema-oracle error:
the new adapter expected top-level `additionalProperties:false`, copying the shape
of a closed struct. The [primary JSON Schema object reference](https://json-schema.org/understanding-json-schema/reference/object#extending-closed-schemas)
explains that keyword sees only properties in its own subschema. Tagged action
branches instead require top-level `type:object`/`unevaluatedProperties:false` and
each action's own closed object. Requiring a top-level additional-properties ban
without redeclaring branch properties would reject valid requests.

Root approved the narrow oracle correction: check all seven distinct closed branches
and their exact required/property sets. No execution, error or state assertion changed.
Removing the new helper and reversing its two call sites reconstructs the original
8,517-byte source SHA-256 `a8dc8a39e91280ed7c02b1ead99ca0fce741f037fb715af4d901156c15c8753a`.
The corrected source is10,243 bytes, SHA-256
`b7a764855080af64b4fa16fa54ed4ea09ab4f714c7646dc81b003c3d0686d7fc`.
Both initial RED logs and source remain ignored for audit; the correction receipt is
`target/view-adapters-schema-correction.json`. No dependency, schema implementation
or production field was changed for this test correction.

## Reproduction and remaining gate

Use the isolated interpreter and worktree-local target described in
[development.md](development.md). Focused commands are:

```sh
cargo test -p bondline-commands --test view_state --locked
cargo test -p bondline-cli --test view_adapters --locked
cargo test -p bondline-mcp --test view_control --locked
```

All baseline jobs are terminal. `target/view-acceptance-pre-port.json` records their
hashes and proves that733 of734 initial files remained exact before the port; the sole
difference was Root's test-module bridge, whose removal restored the original UI source.

Root's real11-file partial port is bound by
`target/view-acceptance-partial-port.json`, SHA-256
`4b6a9fc73aa1bbcacde88690f8f82ab906d3ff5b2ef30b08111b12ffd4bc059d`.
Every received source hash was verified, as were all726 other original files.
Independent focused results on those exact inputs were:

| Session | Target | Actual result | Ignored log |
|---|---|---|---|
|71299|command six|6/6 GREEN; original assertions exact|`target/view-state-six-partial-port.log`|
|88378|CLI/stdio|2/2 GREEN; qualified schema-only correction above|`target/view-adapters-cli-partial-port.log`|
|94953|authenticated loopback|1/1 GREEN; original source exact|`target/view-adapters-control-partial-port.log`|

The independent partial UI gate is separately3/4 with the then-missing pan control.
Later additive receiving gates are recorded in
[verification-view-ui.md](verification-view-ui.md). At that partial checkpoint, final
package checks awaited the completed source port and SDK focus correction.
The full workspace gate and native interaction remain Root's separate obligations.

## Independent final port

`target/view-final-acceptance-port.json` is19,066 bytes, SHA-256
`d1d8b7916e41209c634e57827fe9ce4423f41aedf84e6b4644eb338c6ac81260`.
All100 received Root-owned inputs and all four owned test hashes were checked;
all724 other initial files remain byte-exact. The receipt includes the unchanged
egui0.29.1 version/features routed to a reviewed local source with one focus guard
correction and preserved MIT/Apache notices. This owner edited no production input.

| Session | Command | Result | Ignored log |
|---|---|---|---|
|73656|`cargo test -p bondline-commands --test view_state --locked`|6/6; exit0|`target/view-final-core.log`|
|81650|`cargo test -p bondline-commands --lib view::tests --locked`|2/2; exit0|`target/view-final-private-guards.log`|
|87706|`cargo test -p bondline-cli --test view_adapters --locked`|2/2; exit0|`target/view-final-cli.log`|
|5105|`cargo test -p bondline-mcp --test view_control --locked`|1/1; exit0|`target/view-final-control.log`|
|—|`cargo fmt --all -- --check`|exit0|`target/view-final-fmt.log`|
|44819|`cargo clippy -p bondline-commands -p bondline-cli -p bondline-mcp -p bondline-ui --all-targets --locked -- -D warnings`|exit0|`target/view-final-clippy.log`|

Private epoch exhaustion/typed NaN and infinity tests are post-implementation
supplementary coverage; no RED is claimed for them. The six original public contract
tests retained every assertion and byte throughout integration. The only adapter
test change is the qualified schema oracle correction above. No runtime/native
workflow, full workspace pass or Windows/Linux execution is claimed by these gates.
Final input/protected/owned/log hashes are in `target/view-acceptance-final-handoff.json`.
All jobs are terminal; no passing-source rerun or production edit was needed.
