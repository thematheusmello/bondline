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

## Actual native macOS files

A separate signed `target/cdx-bonds/Bondline CDX Bond Review.app`, PID95769, runs the
37d094d production code. Later commits change only tests/fuzz/docs. Signed executable
SHA256 `cf0fe0d0e01e3f50fb4e2280589524972e10053f6dc8eed9a0873c5384bfe9c6`;
unbundled build SHA256
`b85688aedc715402c75e039f8c2e2cfe40eec56ea42430f47c35fd970424882e`.

After explicit AX-confirmed eight-membered Ring selection, actual OS controls perform
Open manual-Right Double→known Save→Unicode CDX Save As→explicit reopen, then Open Triple
→Unicode Save As→explicit reopen. Seven complete queried models equal independently
frozen A4 expectations, with revisions1/2/3/4, exact H2/H1, clean history/source and correct
file associations. Known Save advances public request epoch3→4 and changes the owned
input's controlled2000-01-01 mtime to its publication time, without changing model/bytes.
This is an actual write witness in addition to the stronger SDK App request check.

| Actual Unicode output | Bytes | SHA256 |
|---|---:|---|
|Manual-Right Double|379|`33727f121e74d276fbc93f22ed274e193118bc166f97a36ba79849efe42df7bc`|
|Triple|367|`79856421a31d22c75737236738a4a26e0af342cc77a127746b0afe551cf3c579`|

Both actual saved files equal their frozen sources. The independent struct reader and
pinned RDKit are rerun on those files: complete observations match, including graph/
order/direction, explicit H and C2H4/C=C versus C2H2/C#C. Raw-coordinate convention is
still qualified; this does not establish layout/side/rendering fidelity. Exact raw
manual257/Right and source identity survive unchanged. Attached CPU/GPU captures,
same-order Triple edit and whole CDXML/MOL conversion all refuse unsupported_content;
the complete state and unrelated owned sentinel bytes remain unchanged.

The canvas displays its honest unmapped-source warning and unavailable drawing controls.
The retained Ring inspector still shows drawing instructions, a preexisting presentation
limit. An initial batched Ring-click/Open attempt did not establish Ring selection; it
is not counted as a passed selection. Exact Cancel followed by a fresh coordinate click
and AX-confirmed Ring-on establishes the precondition before the actual workflow above.
That batched chronology has no causal qualification or product fix. AX100-row truncation
requires fresh own-window screenshots for file fields; CJK filename glyph coverage stays
limited while exact UTF-8 files/paths pass. No other platform, human hardware/DPI,
successful CDX image or human golden approval is claimed.

The disposable review is explicitly quit; exact bundle inventory and original PID are
observed absent. No post-Quit app probe relaunches it. User-viewed creation review365 and
older apps/drawings remain protected. Ignored receipts live in
`target/cdx-bonds/research/`; startup credentials stay private0600 and are never committed.

## Final local gates

Fresh full `cargo xtask ci`2882 on code1606d62 PASSES: workspace2187/0/1 across321
result reports, plus vendor3/0. Workspace formatting, warnings-denied all-target clippy,
workspace tests/doctests, vendor tests/clippy, enforced layers, strict runtime licenses/
seven assets and the pinned eight-file/five-bundle corpus all pass. Separate fuzz fmt
and the bounded sanitizer above also pass. The275214B ignored aggregate log has SHA256
`f3fdf9aaa2c25c4ab76d9ce03486ec6f34d9a0f372bfa0dd315c706855ebbb0c`.

This is a clean local aggregate. Earlier literal-metadata1390 failed the unchanged MCP
replacement_bind51670 test, and other historical occurrences remain unexplained; this
passing run is not a causal listener fix. Protected test SHA256 remains
`fd96b257cd6c00b52e97544d652ed69fdbaaaa454e1f3a9360d77f3292218fb0`.
No failing test is retried/waived to obtain this result. No dependency/lock/vendor/font/
asset/golden/workflow change or new Windows/Linux/WASM execution is claimed.

All implementation, native, sanitizer, gate and worker jobs are terminal. Worker branch/
worktree and exact scoped patches remain for provenance; integrated source/tests are
committed and pushed. Full M0–M5 and human NH2 golden approval remain incomplete.

Native user reviews/apps/files stay protected, Actions stays OFF, and every created
atomic commit is pushed. No full CDX display/edit authority, command or tool is added by
the fixture/source work. M1 and all remaining M0–M5 requirements remain incomplete.
