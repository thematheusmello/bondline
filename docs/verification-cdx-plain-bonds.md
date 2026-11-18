# Plain CDX bond verification

2026-11-18, source contract6c9fff0 onbaa3d90, independent fixtures/helperc29c1a1 and
integrated IO37d094d.
Read [the locked contract](research/cdx-plain-bonds.md) and
[fixture provenance](../crates/io/tests/fixtures/cdx-plain-bonds/SOURCES.md).
Native acceptance and final local gates are recorded separately from the headless and
bounded sanitizer evidence below. M1 and the full M0–M5 goal remain incomplete.

## Implemented data boundary

The existing IO reader maps binary Order1/2/4 to Single/Double/Triple and keeps absence
as Single. Both global Bond-owner and local property lists admit0601/0602/0603 with
strict duplicates. Only absent/explicit0 Solid passes; Display2/position are Double-only,
including explicit0. All six position values map to the existing directed side enum;
absence remains Unspecified. Automatic/manual distinctions remain raw source data.
No endpoint sorting, H inference, scalar normalization or new data type is introduced.

Old Order wrong-width InvalidProperty/unsupported-order behavior stays; new unsupported
widths/enum/non-Double values refuse UnsupportedProperty. Existing object/query/stereo/
metadata/resource/identity guards remain. Unchanged/no-op Save retains every raw byte;
model drift refuses until restored. Full/display/chemical-authoring and cross-format
permissions stay closed. Legacy detached page edits/history remain model-only. No UI,
command or renderer production changes, new registry record or planned-tool binding.

## Frozen independent evidence

Sixteen original Python-struct files cover plain Single/Double/Triple, absent/explicit
Solid, all six automatic/manual Center/Right/Left codes plus absent position, reversed
Begin/End, reordered fields, short/extended lengths, three accepted envelopes, knownH2/
H1 and unresolved H. Source IDs remain distinct from dense app IDs; the Bond precedes
the second Node. Complete record spans/owners/payloads/presence and independently built
A4 model JSON are frozen without product imports/writer/output.

Manifest SHA256 `15d2d29afb7d8e4f0faa2e9cc7007c3348387604b379dbd3876b6cb1cc393aa8`;
facts `41adc4e4d94ea734013679b0e89e96622a12a2a0e92071c79ff1c158a62462af`;
reader `40b6fac40ea9e6cc7e6932d56b0d0f9e3d636d79912efff2d68fa90a99ee5d84`.
All source/fact/observation/check files reproduce byte-for-byte. The149 independent
refusal controls cover each owner, width, duplicate, unsupported order/style/position,
non-Double metadata including0, and short/extended truncation. Old18 literal sources/
helpers and all their pins remain unchanged.

Actual pinned RDKit2026.09.2 agrees on all16 graph/order/direction observations across
the three envelopes. Ethene gives C2H4/C=C; ethyne gives C2H2/C#C. Known inline H2/H1
agrees. Its inferred unknown-H formula is an external observation only; authoring H stays
None. Raw-coordinate/y-reversal behavior is recorded explicitly. This does not prove
side, automatic/manual state, point-space layout, rendering or stereo. Those literal
source fields have independent struct/public-spec evidence. Observation SHA256
`0cc5d3a4a5f919d14c1a307267bb829dd53eaeb6cfe6234c71fd63d53f2b58b8`.

## Runtime RED before production

Existing-API IO76132 runs0/4: Double/Triple refuse UnsupportedBondOrder, explicit Solid/
position sources refuse0601, and a known Bond field wrongly passes on an implicit root.
Three partial-scaffold unused warnings are distinct from these actual runtime failures.

Root27993 command cases run0/2 at actual Order/0601 Decode errors; App runs0/1 with the
unchanged ten-second FileWorkflow wait and actual unsupported0601 Decode diagnostic.
Three actual CLI/initialized MCP/authenticated socket cases run0/3: CLI ethene refuses
Order record16, MCP ethyne and socket auto_right refuse0601 record17. Exact49 discovery
and independent input readers execute before applicable failures. These compile and run;
no missing API/compiler error substitutes for runtime RED. No assertion is weakened
after those REDs. Later test-only strengthening is qualified below.

## GREEN and receiving qualifications

The same frozen IO12 cases pass after the minimal one-file decoder, covering all16
complete independent models/records and149 refusal controls. The exact old cdx_graph
Order2 negative then fails as expected. Only that witness becomes a stronger positive
for Double, IDs/direction/Unspecified/default atoms/HNone/raw/fullfalse/displayfalse;
the old0/3/FFFF array and every other negative remain. FullIO541/0/0 across78 reports
passes;70160B log SHA256
`6150c115ae8b8e449d5ca592d60cfe9a5669bc429e65e05fe938d25bbb03c465`.

One new-test needless borrow is removed after all-target clippy. Independent review also
strengthens the named duplicate-record checks to exact independently counted indexes15/
16, without changing values, source bytes or production. Final focused12/0, warnings-
denied all-target IO clippy/scoped fmt/diff checks pass. Full541 precedes those test-only
qualifications; no product behavior changes after that full run. Two peer reviews of
the exact5575B production patch find no concrete defect. Production SHA256
`defaefa3bed27eb154fe2fff51514304c2b69d2dbb787477b7a728a39d9534ed` stays exact.

Root command2/0 and App1/0 pass against that exact producer. Peer review finds the new
App known-Save assertion could pass if Cmd-S were ignored. The test is strengthened to
require a public file-request epoch advance and successful completion with no App error;
qualified App1/0 passes with the old wait/predicates unchanged. Save As/reopen still
require independent actual-byte/model equality. This is SDK App evidence, not OS input.

Actual CLI/initialized MCP/authenticated socket3/0 pass unchanged after their runtime
REDs: seven complete directed CLI models, byte-exact49 discovery/schema/order, actual
Unicode files and independent readers, equal-model auto/manual source replacement,
captured Save publication without acknowledging the replacement, weak-owner/no-op/page
history fences and same-order/capture/loss refusals. Test14172B SHA256
`9c9f817bb3160f3a714658c68e96902587a74b8628a7d978fae885f453397c9f` is unchanged between
RED and GREEN. CLI all-target warnings-denied clippy also passes. Registry49 and
catalogue16/124 stay exact; no caller production source changes.

## Bounded sanitizer

The existing cdx_parse harness adds raw-record proof for every accepted bond's binary
order, Solid fields and exact directed side/presence. It shares the bounded record table
with existing atom metadata, checking owners/duplicates without another whole-source
scan per bond. Old framing/source/replacement/new-tree/snapshot and missing-H assertions
remain. No filesystem, Engine, native, GPU or chemical-validity behavior is fuzzed here.

Fresh [seed generation](../fuzz/README.md) keeps184 immutable sources/manifest outside
the writable corpus:16 original bond sources,149 independent refusal controls,18 unchanged
literal-atom sources and one exact131072B original opaque-padding input. Independent
reader checks accept17 new positive inputs/refuse149 controls; all seeds and fresh
regeneration remain byte-exact after execution. Manifest SHA256
`8438fb643a457f5ddd2c534914517980bbbf8d9597b51973e9f6dac2e52f6cda`.

ASan13740 passes10000/1s, cov1321/ft2901/RSS232MiB. Initial184 files total195897B, max
131072B; final mutation limit131072B and actual NEW/REDUCE maximum104971B are distinct.
The maximum is read from the log's L: record, not inferred from its limit/initial corpus.
The25214B ignored log has SHA256
`447008f3d4939b8e718513be70a348fd28324e66892371d691b387cff184974c`.
Fuzz formatting and seed self-checks pass; no dependency or manifest/lock changes.

Native user reviews/apps/files stay protected, Actions stays OFF, and every created
atomic commit is pushed. No full CDX display/edit authority, command or tool is added by
the fixture/source work. M1 and all remaining M0–M5 requirements remain incomplete.
