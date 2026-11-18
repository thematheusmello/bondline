# Atomic document drawing-settings command

2026-11-16, base b6497dd. Previous goal turn is PROGRESS: independently verified stored
settings, bounded projection fuzz and CLI persistence. Its full gate failed the unchanged
MCP replacement bind; separately completed stages do not make that aggregate green.
The full M0–M5 goal remains active, with Actions OFF and user windows untouched.

The command is now implemented against this lock; [receiving](../verification-drawing-settings-command.md)
records its actual runtime RED/GREEN, independent cases and failed/qualified gate boundaries.
The full native settings/chain requirements below remain unchanged.

## Evidence and scope

The [data contract](drawing-settings-data.md) and
[receiving](../verification-drawing-settings-data.md) establish optional validated root
BondLength/ChainAngle, exact-source authority and loss guards. The public SDK
[BondLength](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/BondLength.htm) and
[ChainAngle](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/ChainAngle.htm),
re-read this turn, describe creation defaults with containing-object/last-style fallback.
GUIDE25 printed40–42/104–105 supplies active-document settings and angle1..179°. These
sources do not specify our protocol, null semantics, source subset, transaction behavior
or one universal absent value. The following command policies are original Bondline choices.

This is the next engine/adapter prerequisite for the full native settings/chain workflow
in [the dependency plan](native-chain-plan.md). It does not enable a native setting form,
stylesheet resolution, fixed toggles or chain control, or apply values to existing native
creation tools. Existing atom positions/H, page/stroke/label geometry remain exact.

## Closed, full-replacement contract

Append one non-read-only registry command, `document.set_drawing_settings`, after the
existing40 records, preserving all their names/order/descriptions/schemas. Export typed
`DrawingSettingsSet` with exactly three required fields:

```json
{"expected_revision":0,"bond_length_pt":24.5,"chain_angle_degrees":120.5}
```

Both scalar fields are `Option<f64>` in Rust and number-or-null in JSON. All three keys
must be present; omission, wrong types, positional arrays or unknown keys fail. Explicit
null clears that property; two nulls restore the empty value, omitted in model JSON and
native CDXML. This replaces the pair atomically, never patches a missing field or resolves
an inherited default. Present length is finite >0 through100000pt; angle is finite
1..179 degrees, fractional allowed, reusing `DrawingSettings::new`. Typed callers receive
the same domain validation, including NaN/infinities. No duplicate model/schema dependency.

Use the existing serde/schemars versions and patterns. Serde normally decodes a missing
Option as None; field-level `deserialize_with` forwarding `Option<f64>::deserialize`,
without a default, requires presence while accepting null. In installed schemars1.2.3,
`schemars(required)` removes Option nullability. Preserve ordinary nullable field schemas
and explicitly extend the root required list instead. Verify this in the actual schema,
including ranges, closed object and all three required keys. Actual API source pins are
in the ignored source review; do not infer missing-versus-null from schema alone.

Typed request serialization must reject nonfinite Some values before JSON could turn
them into explicit-null clear requests; None still serializes as null. This closes a
separate caller boundary without changing incoming validation, schema or source authority.

## Authority, errors and transaction

Existing JSON parse/closed-parameter validation precedes Engine preparation. Preserve the
central committed-close refusal. Then check expected revision, Drawing content, and
source-free or completely mapped native CDXML authority, before domain/equality/no-op.
MOL/CDX, foreign/noncanonical/opaque CDXML, retained SMILES projections and SDF/raw SMILES
refuse, including equal-value requests; a no-op never grants unsupported-source authority.
Do not promote a projection or change any source/completeness proof.

Use a specific `UnsupportedDrawingSettingsSource` error mapped to existing
`unsupported_content`. Stale revision remains `stale_revision`, wrong/missing parameters
and domain values `invalid_params`, committed close `close_committed`. Existing overflow
errors and precedence remain. Errors preserve source/model/Arc, history/redo, file/close/
selection/view/tool contexts, generation and ID floors exactly.

After validation, equal settings return unchanged without a model clone/mutation or new
snapshot. Preserve pending/approved close, redo, selection and all weak context identities.
A real edit clones the StoredDocument, sets only its validated model value and calls the
ordinary immediate commit once. Normal revision/history/dirty/source/selection/close and
history-budget rules apply. Preserve `view_focus` explicitly, as chain/atom handlers do;
the prepare match's default None must not reset an unrelated camera focus. No geometry,
tool state, generation, identity allocation, worker, Save reservation or receipt is added.

Prepared immediate work retains existing semantics: authoring occurs during prepare;
later cancellation cannot roll it back. Never claim deferred execution/cancellation for
this command. Old in-flight weak content contexts become stale after a real edit; no-op
or refused requests preserve them. Existing Save leases, snapshot/currentness and close
rules remain unchanged; this command cannot acknowledge a file or complete a close.

## Independent acceptance and integration

First add an existing-JSON-API test using literal24.5/120.5, one previous bond and exact
unchanged pages. Observe UnknownCommand at runtime before adding producer/types. Then
add independent presence/null/domain/schema/source/no-op/history/close/context/ID and
typed API cases. Cover unchanged scene, normal selection invalidation and preservation
of the existing view focus; real counter exhaustion must fail atomically where accessible.

Exercise actual CLI, stdio MCP and authenticated loopback discovery, success, no-op,
strict-input/stale/source refusals, history and Unicode native CDXML Save/reopen. Check
raw root attributes using the existing independent Expat/Decimal reader before unchanged
graph/chemical consumers; own round trips alone are insufficient. Check the whole-MOL
loss guard via registered Save, including existing/absent destinations and no false save
acknowledgment. Keep the old semantic fragment-export policy and consumers untouched.

Audit every exact registry count/name/schema assertion and positively append this one
command; retain all old records and assertions. No native bindings/control counts or
catalogue entries change:31/20/14/10 and12 delivered/128 disabled. Execute focused checks,
fmt, warnings-denied lint, the local aggregate and existing xtask gates. Preserve a new
natural MCP failure without unchanged retry, exhausted probes or speculative policy fixes;
only then-unstarted stages may run separately. Report actual native/platform scope.

Follow with ordered native modifier events and complete settings/chain forms/gestures,
then actual native acceptance. This command alone does not complete a style workflow,
native chain tool, milestone or the full product objective.
