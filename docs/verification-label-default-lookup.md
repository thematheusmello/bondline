# Label-default lookup receiving

2026-11-18, base7747111. Read the [contract](research/label-default-lookup.md),
[sources](clean-room/sources.md) and [remaining semantic questions](clean-room/open-questions.md#chemical-label-defaults-and-placement-2026-11-18).
This standalone format prerequisite advances M1, not a complete foreign-label workflow
or M0–M5 completion. No registry, model, renderer, native tool or IO authority is added.

## Source findings and implementation

The primary SDK describes independent containing-object LabelFont/LabelSize/LabelFace
fallback, then an external style sheet. Absence cannot become a universal font/size/face.
Node child Text also takes precedence when read; redundant Element/H alone does not prove
a conflicting label correct. Current source opacity remains necessary. DTD Left and the
archived eventual-Auto justification fallback differ; omitted Style-run precedence is
not established by optionality. The linked Geometry page serves ObjectTag content.

Three independent public sources qualify general Node-child placement rather than prove
a universal baseline: Apache Octet export uses a baseline convention but import does not
preserve arbitrary child positions; Apache Indigo exports differ; the MIT author's26.1
report distinguishes stored coordinates and baselines in certain formula/font cases.
Only public source/license text is read, with exact commit pins and limits in the source
register. No reference program, font, export, screenshot, sample or implementation ships.

`Document::label_defaults` returns separate borrowed values with their actual owned nodes
and decoded attribute values. Each property independently takes its nearest supported
containing owner. None stays unresolved. Font/face remain signed INT16 data, including
negative values; they are not cast to unsigned run/resource IDs or interpreted as bits.
Positive finite f64 XML sizes are a stated original reader policy, separate from binary
units and later runtime/model limits. All encountered ordinary values, including shadowed
ones, validate; unexpected-owner fields refuse. Namespace-qualified lookalikes remain
unrelated raw data. Pointer ownership and ordinary supported roles gate the query.

The implementation walks only the parser-bounded ancestor path without a new allocation
or recursion. It does not validate full object containment, resolve resources/fonts,
inject DTD values, fill omitted `s` fields, derive chemical meaning, mutate raw bytes or
make unknown source editable. Source nodes can drive the existing checked SetAttribute
API; the independent test verifies exact unchanged lexemes and unrelated-byte retention.

## Tests and qualifications

Root's initial literal source declares root font/size/face, a nearer node size14.5 and
Text face0; it checks independent owners, decoded values, untouched run absence and
exact source. Before production,98310 exits101 with the sole intended E0599 missing API.
Original1136B/SHA256
`0f3da31ab196cd9b6c39ab4cde7b99dfc78584204102da750f0a49072dc7c110`;
RED568B/SHA256
`1f6c979a7467cfa16514fb377dd90eacf3591eec9954e1eb37fa8d29e186779a`.
After implementation43497 passes1/0, log452B/SHA256
`61060b6afbc77749db1521d8f2242149a51e9beb4bab53ceb3ed01130ab228e6`.
This is an API compile RED, not a fabricated runtime regression.

The independent worker reads the locked contract and formatted initial test, preserves
its1182-byte prefix exactly and declares eight further cases. Its combined17648-byte
declaration SHA256
`09f539c3ff342a336075920cdc0008950ccf20dc9d5ef6071cccdfdf17eb1bcf`;
independent Expat/Decimal oracle3291B/SHA256
`588501a5065e4cd7110fac2dc5001e543070a3405c4777b88c671d2120129b6c`.
These expanded cases are post-draft independent controls, not additional claimed
pre-production REDs. They cover every admitted target, partial/absent defaults, signed
limits vs UINT16 resources, size policy/underflow, invalid shadowed values, error order,
unsupported owners, namespace resets/lookalikes, foreign-node identity and owned edits.

First49428 runs8/1: all lookup/guard behavior passes, but a new setup assertion expects
an explicit empty namespace to be represented as None. Pinned roxmltree returns Some
empty. Qualification accepts either representation of the same ordinary namespace and
adds an independent Expat owner/value check for that exact source. Qualified83505
passes9/0 and standalone warnings-denied clippy. No production change addresses that
test representation assumption. Qualified-attribute retention and wrong-owner negatives
remain strict; no source/format guard is relaxed.

The Expat/Decimal reader first accepts two independent manually declared source/edited
vectors. Wrong owner, overridden size/face and unsupported-owner negatives must fail
before consuming product output. Actual SetAttribute output then matches the complete
expected bytes and reopens with exact owner/value data. A decoded no-op preserves
numeric spelling, entities, quotes, CRLF and unknown content. A node from an identical
other document cannot authorize edits. Missing resources/run fields stay missing.

Independent source review finds no material discrepancy with the contract. Its report
2767B/SHA256
`f4f407e2e2b8bea9aa76865ee531002a926f41812475501bb0ca5fa58d68b5b2`
pins production4563B/SHA256
`38c2bf79acfc44492202c2ed7e084cd94639dbc47e54825a805abe9c7a53b261`.
Review is source evidence, separate from execution. Logs/declarations/receipts remain
under ignored `target/chemical-labels/`; no credentials/assets are committed.

## Bounded fuzzing

The existing `cdxml_parse` target retains every earlier invariant and16KiB guard. New
checks cover accepted-source ownership/default provenance, signed/size fields, strict
shadowed validation, per-property precedence/error ordering, absence, unsupported owners,
namespace meaning/qualified retention and unchanged source/run fields. The new stdlib
recipe creates54 original licensed seeds with separate source/corpus/manifest and refuses
an existing output directory. Old source checks remain exact outside added helpers/calls.

Root74732 exits0 with pinned nightly-2026-04-01/cargo-fuzz0.13.2/libFuzzer0.4.13,
AddressSanitizer, seed671308429,10000 runs, max_len16384/timeout5/RSS2048. It completes
in5s, cov3845/ft8352/RSS462MiB. The initial seed reaches16384 bytes; actual NEW/REDUCE
maximum is8885 bytes, not falsely claimed as16KiB mutation coverage. All54 immutable
source copies retain their size/hash. Log63629B/SHA256
`02ebd40bb3cf28f3ed0919df70c95cad213da4bdc6a45e2d4a3949c08c37e5f6`;
manifest SHA256
`10fff8a39278e10fc711bc59be08e55c4e4ed1db266d8b03e3f6b62699a29232`.
See [reproduction commands](../fuzz/README.md). No general64MiB, renderer or native
interaction coverage follows from these bounded parser runs.

## Local gates and continuation

Fresh Root40637 `cargo xtask ci` exits0: **1919 passed/0 failed/1 ignored**,288 result
lines, including79 standalone CDXML cases. It passes workspace rustfmt, warnings-denied
workspace/vendor clippy, workspace/vendor tests, layers, strict three-platform declared
runtime licenses, seven assets/notices and all eight pinned corpus files. Log242691B/
SHA256 `8da88225b3393901040181e8cc69a19af1cf32049f3deb8929ef871cdb91abf7`.
Execution is the actual Root workspace, Rust1.94.1, incremental0/jobs2/dev+test debug0,
the isolated format-python interpreter and seed671308429. Fuzz formatting also passes.

This natural aggregate pass does not explain or fix the preceding caption-stage MCP
replacement-bind failure. Listener policy/tests/timing and exhausted diagnostics remain
unchanged; no retry is added. Registry47/canonical44407-byte discovery stays exact at
SHA256 `8d5a7031cfc0ce8e444878ed058ee6080dcff601f48cdfd8149648eee0b575df`.
Catalogue16/124, model/commands/IO/UI/render source, dependencies/locks, vendor, fonts/
assets, approved goldens and workflow definitions remain unchanged. No native window,
user drawing or proprietary reference program is accessed. All Root jobs/workers are
terminal. Actions stays OFF; no push/publication.

Next integrate owned child-Text/effective style data with semantic agreement and complete
source/retention/render/edit/history/Save proofs. Do not infer a universal baseline or
missing-run fallback. H-changing tools and position edits must synchronize supported
dependent labels or explicitly refuse. Native/full Text editing, wider foreign/CDX
files and all remaining M0–M5/platform/release requirements remain active and incomplete.
