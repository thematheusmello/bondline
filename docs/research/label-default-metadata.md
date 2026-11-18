# Explicit labels beside preserved default metadata

Locked2026-11-18 on base0aa97cd, after db7607c's status-policy correction. This is a
complete bounded M1 foreign-file workflow. It does not implement omitted-run inheritance
or replace any remaining M0–M5 requirement.

## Public evidence and original authority policy

Fresh primary [Style](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Style.htm),
[Text](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Text.htm),
[Node](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Node.htm),
[LabelFont](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/LabelStyleFont.htm),
[LabelSize](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/LabelStyleSize.htm),
[LabelFace](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/LabelStyleFace.htm)
and the official DTD distinguish explicit runs from scalar defaults. Document/Node/Bond/
Text are legal Label* owners. Each default property has independent containing-owner
lookup and potentially an external stylesheet. Signed scalar fields differ from UINT16
run/font-resource fields. Only the legacy CDX composite explicitly defines−1 sentinels.

The sources do not establish arbitrary omitted-s→Label*/Caption* substitution. CDXString
does contain omitted-field examples; they do not choose the default family or resolve
all omission combinations. Preserve that uncertainty: every actual s font/size/face/color
remains required. No model, renderer, standalone StyleRun or shared caption::project_runs
fallback is added. Child Text's general as-is rule supports using its fully explicit
typography while retaining separate unused defaults; exact26.1 pixels remain unverified.

Thirty fresh HTTP receipts (25 OK/five404) are ignored research. Manifest SHA256
`8566911f1bc2ec07dca4e278f4daa6b8d831337854893072e4a7c5fa724fcb49`;
report11619B/SHA256 `fa157efa35cc2381908e1f28936c7b52c8d66817d583c0c6c6d4cedc07f3bbea`.
No reference executable, sample, artwork or font asset is inspected or copied.

## One checked source mode

Add private `CdxmlProfile::LabelDefaults` and public `has_label_defaults_projection`.
This is complete source coverage for EXPLICIT labels beside preserved scalar metadata;
the name never means run inheritance. Require at least one ordinary LabelFont/LabelSize/
LabelFace attribute on a modeled root/n/b/t owner. Include supported Page caption t as a
Text owner, with its existing fully explicit runs and all caption restrictions unchanged.
Page/group/fragment/s/resource owners and all unmodeled objects remain refused.

Reuse `xml.label_defaults(owner)` at every admitted owner, retaining per-property nearest
lookup and validation of all shadowed values. Explicit scalar font must be nonnegative
i16 and resolve to an existing resource; its family may differ from actual run typography
because it is unused. Explicit face is retained nonnegative i16 bit-field DATA, without
decoding/rendering any new face. Positive finite XML size retains the existing standalone
reader policy; no glyph-work limit is inferred from an unused scalar. Negative/default-
sentinel/unresolved-resource, malformed, qualified or unsupported-owner values refuse
complete authority but remain in unchanged raw source. Absence supplies no value.

Compose this metadata mode with the existing explicit-label graph proofs: all printed
labels have no-p/Left/Left/Roboto face0/body/whole-token-box agreement. No-bond sources
must have all atoms printed; printed flags remain data under child-Text precedence.
Bonded sources need the existing complete root ink/margin/relevant spacing/black-bond/
noncrossing geometry proof, and every no-Text atom must be CURRENT Hidden neutral,
natural, nonradical known-H/C4 carbon. Inspect raw children; a projection failure is not
absence of source Text. All old syntax/key/resource/ID/page/text/work/paper guards stay.

Keep old foreign Boxed/Connected/Mixed/Flagged predicates literal and globalfullfalse.
The new mode is orthogonal, not a family of new fallback/render implementations. Preserve
metadata owner/attribute presence, exact raw values/quotes/entities/order and resources
by the existing source spans; the immutable original source remains authoritative.

Existing48,49 and Selection consume the new precise grant. BaselineSome label deltas keep
old style/flags/presence constraints; baselineNone may gain an explicit49 label, with
existing stable all-baseline ID reservations and original resource lookup. Remaining
no-Text atoms must stay currently hidden. No-op sharing/redo/weak owners remain exact.
One final source re-import/model equality/capability and one physical-page certificate
precede commit. Default metadata is never changed, materialized or used by native49's
visibly stated original12pt black preset. Other authors/loss guards stay closed.

## Acceptance before production

Freeze bounded original sources/facts before code, independently adding only declared
metadata to existing original diagrams/expected edits. Include root/node/Text/Bond/caption
owners, independent property owners, conflicting defaults10/14/16 versus actual12pt,
entity/quote/numeric spelling, unused different font/face96, UINT16 actual font65535
versus signed scalar limits, partial/all-printed49 continuation, and exact48/move/history/
Unicode Save/reopen. A wrong16pt box for a12pt run remains a negative.

Missing each actual run field must remain refused despite complete Label*/Caption*
defaults. Cover negative/shadowed/missing-resource/owner/namespace/syntax/body/geometry/
current-hidden/no-op/other-mutator/whole-format refusals without weakening old assertions.
Two files differing only in unused metadata have equal models but DIFFERENT source/file
owners: Open/captured Save/stale gestures/forms cannot be authorized by model equality.

Observe existing-API display/write/command runtime REDs separately from missing-new-API
compiler failures. Reuse frozen independent font/pixel controls; unchanged explicit
typography must produce identical source-aware CPU/GPU frames. Verify CLI/initialized
MCP/authenticated socket/App and a fresh own native workflow. Protect PID365 and older
user apps/files. Run bounded ASan and complete local gates; record actual qualifications.
Registry49 actual bytes/schema/order and catalogue16/124 stay exact. No dependency/lock/
vendor/font/asset/old-golden/workflow changes. Commit atomically and push each commit;
Actions OFF. Omitted-run inheritance and the full M0–M5 target stay required/open.
