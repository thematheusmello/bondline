# Ordered bond authoring and transient tools

2026-11-14. Work in progress on `feat/multiple-bonds`, base
`a279bced1fee31eebc163b5d1e02e3ade0fd05ff`. This record covers command/notification
checks, not native-window or complete professional tool parity. Actions remains disabled.

The existing `bond.create` and `bond.extend` receive optional closed `order` values
`single` (default), `double`, `triple`. Direct commands do not inherit UI selection.
Creation authors literal neutral-carbon H counts `4 - order`. Extension requires exactly
one incident supported bond and the consistent literal terminal count `4 - old_order`,
then subtracts the requested bond weight. Insufficient/unknown/inconsistent H refuses;
the added carbon receives `4 - requested_order`. Positions, IDs and source authority
remain guarded. This is a deliberately closed operation, not general valence perception.
Sources: BOND-EXTEND-VALENCE / BOND-EXTEND-AUTHORED-H and the multiple-bond source entries
in [the source register](clean-room/sources.md).

`ui.bond_tool` has a required closed `order`, three central `ui.controls` bindings and
a separate checked epoch. Real mode changes advance that epoch; repeated selection is a
no-op. Neither enters authored state/history, dirty state, source bytes or close intent.
Selection persists across history/Open. The explicit reply's
`ui_bond_tool_operation.changed` describes transient change; outer `changed` stays false.
Loopback publication notifies the UI for real mode changes only. Active picker/close and
incomplete drawing-source guards apply. These epoch and wire policies are original.

## Observed RED and GREEN

Commands use Rust1.94.1, locked workspace dependencies, `CARGO_INCREMENTAL=0`, separate
Root/sibling targets and the unchanged external oracle environment documented in
[development.md](development.md). No dependency/fixture/font/golden or workflow changes.

- The independent worker first observed baseline explicit-Double parameter rejection and
  multiple-bond capture refusal, before Root applied the order-authoring draft. See
  [its independent command record](verification-multiple-commands.md).
- Root16296: four tool-command tests fail101 at the missing registry/bindings. After the
  single registry implementation,74975 passes4. Combined88191 passes those4 plus both
  terminal-extension tests below. Log `target/multiple-authoring-green.log`,1128B,
  SHA256 `04fff07c07df84a57d9fc7accd9dca4bba3c4d285f21054e1ca40387c41ebbce`.
- Root48631: both `ordered_extend` tests fail101 with UnsupportedBondExtension before
  generalizing the old all-Single/H3 guard. Log `target/ordered-extend-red.log`,1110B,
  SHA256 `31b3177b72dbca0c15318be15b55d34228e0c41bbb8d9c7b9b0b5e11e0c9795d`.
  Minimal consistent-H/weight checks then pass Double→Single, Triple→Single,
  Double→Double, exact Undo/Redo and over-valent atomic refusal with unused ID floors.
- Root90058: actual authenticated-loopback notification test fails101 (callback count0
  rather than1) while the tool result correctly reports an authored no-op. Adding the
  transient operation to the existing publication predicate passes19480 (1test).
  RED1294B/SHA256 `e1472dfdba1fc6b1ea8cdb0c20c30812a7fbd2a6db08f32e745590e6b87fa479`;
  GREEN582B/SHA256 `fb686d6e5312de481cfb1d0cf2eda4bfc31c02d82f5564655101795720a56e8c`.
  A subsequent Inspect request is the ordering barrier; no timing sleep or logged token.
- Existing regression82296 fails only the old control-count assertion (12versus9).
  Updating that count and adding the new command to exact discovery sets preserves
  old fields/schema assertions. The same focused gate40767 exits0. The old inconsistent
  Double/H3 extension refusal remains unchanged. Broader rendering/full-workspace checks
  are pending integration, not implied by this command gate.

The epoch-exhaustion unit check is an added boundary regression after implementation;
no separate RED is claimed. Final package/lint/frame/adapter evidence will be recorded
after receiving the frozen independent workers. Native interaction remains paused by user
instruction. Context-sensitive sidedness, ring placement, conversion hotkeys, persistent
style/inheritance and broader connected editing remain visible in the parity backlog.

## Receiving results

The frozen renderer and command/adapter/UI tests are now received. Root25282 passes all6
unchanged actual-frame tests. Root49317 passes19 supported-MOL/SDF display regressions:
only the now-supported old Double/Triple refusals become stronger positives, using the
same inputs, unchanged external consumers and complete source/history checks. Unsupported
metadata, degeneracy, empty records and inconsistent Double/H3 extension still refuse.
The independent worker's6 command/image and3 CLI/stdio tests pass; bounded ASan10000+10000
and12 fixed replay checks are separately qualified in its linked record.

Root aggregate attempts82133/61351/45487 expose stale exact command-name/control-count
inventories in three CLI test files. Their old assertions remain, with the one new
command/three new bindings added. Attempt10297 then reproduces the historical unchanged
listener assertion; [direct shutdown evidence](verification-listener-shutdown.md) and its
[portability follow-up](verification-listener-shutdown-portability.md) replace provisional
connect success as the ownership oracle without changing production shutdown behavior.

The renderer-only staged tree separately passes866 tests/zero failed/ignored under
CI76060 before commit57bedb5. Final frontend receiving is recorded with its UI integration.

The exact backend-only staged tree passes local CI58584: 883 tests, zero failures/ignored,
with its own absolute target and debug0. Log `target/multiple-commands-index-ci-isolated.log`:
129198bytes, SHA-256`1220a3882eea025ed731bb390d073f1b133ae3d7aeef9e8c84124934a86bab94`. The earlier shared-target
index check20355 emitted E0432/E0560 against old API metadata despite coherent exported
definitions/paths. Prior-render-export dep-info is present in that target; the exact
freshness mechanism is unproven. A fresh isolated build passes unchanged. Keep the failed
log and use separate targets; this is not a production API fix or a hidden retry.
