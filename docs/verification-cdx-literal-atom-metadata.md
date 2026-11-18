# Literal CDX atom metadata verification

2026-11-18, contract42ccefb/clarification4af42c5, original fixtures9a56c97/helper9b77891,
IO4996ee7 (isolatedc018738) and actual adaptersd414c6a. Read the
[locked contract](research/cdx-literal-atom-metadata.md) and
[fixture provenance](../crates/io/tests/fixtures/cdx-literal-metadata/SOURCES.md).
This adds literal read-only file coverage; M1 and the full M0–M5 goal remain incomplete.

## Implemented boundary

The existing CDX projection now reads Node-owned isotope0420 (signed16), charge0421
(signed8 or the documented four-byte signedINT32>>24 recovery), radical0422 (distinct
None/Singlet/Doublet/Triplet) and explicit inline H042B (unsigned16). Missing H stays None;
present0 stays Some0. Isotope0/absence remain natural None; negative isotope and unknown
radical values refuse. Other integer widths are unsupported, not universally malformed.
Framing length representation is independent from payload width.

Both global Node-owner and per-Node property lists include the four tags. Wrong owners
and duplicate known properties refuse, including identical zeroes. Explicit NodeType1
and the old absent-type default remain; Unspecified0, child Text, queries, stereo,
unsupported bonds/geometry/identity and all other source restrictions stay exact.
Metadata populates existing Atom fields locally before returning the Fragment. No model,
registry, renderer, command or UI production change is needed.

Unchanged/no-op CDX Save retains every original byte, including low legacy-charge bytes,
field presence/order, short/extended lengths, envelope, source IDs and opaque content.
Source identity includes those bytes even for equal models. The existing detached page
operation may dirty the model/history; CDX Save refuses drift until Undo restores source.
Full/display/chemical-authoring and cross-format permissions do not broaden. All49
serialized command records/schema/order and catalogue16/124 stay exact. No grey tool is enabled.

## Independent declarations and actual results

Eighteen original Python-struct sources and complete facts precede production; manifest
SHA256 `75374f4c9c015183c1556aa6c3f81b1b0f2668e80893e1dddba8ac9bd8874c3c`,
facts `11215730fc66ee42af06c63bc80f1851bfc7814617d2e2a9a5269bacaf671d15`.
Portable source/facts/observation reproduction is exact. The independent reader has36
refusal controls. A draft generator's explicit NodeType0 mistake is corrected to Element1
BEFORE freeze; sources/facts/actual RDKit are rerun. No product guard is relaxed.

Pinned RDKit2026.09.2 binary-CDX agrees on valid original13C methylammonium isotope/
charge/H in explicit, extended and embedded envelopes, methyl doublet and2H+ node/H0.
Two FOUR-BYTE charge cases differ: payload01000000 is normative0 versus RDKit1;
AABBCC7F is normative127 versus RDKit−86. Expected values follow the property-specific
SDK algorithm and independent struct, not consumer output. Other high1/high−1/min cases
agree. External electron count2 cannot distinguish singlet/triplet; enum preservation
is independently verified without claiming spin inference or universal interoperability.

Existing public IO runtime RED0/4 precedes production. Frozen13 grouped cases then run
RED3pass/10fail→GREEN13/0. Three command cases run RED0/3 at Node0420→GREEN3/0; the App
case runs RED0/1 (ten-second workflow timeout with actual Decode Unsupported0420 error)
→GREEN1/0. No new public API/compiler failure substitutes for these runtime REDs.

Exactly six old valid-field witnesses become stronger positive data/raw/capability
assertions: cdx_graph Isotope13/ChargeFF/Doublet2/H0, cdx_document H4 and cdx_metadata H0.
The document test retains unsupported_content using an existing unsupported0430 Node
field; old syntax/resource tests and every other negative remain. Full IO529/0/0 passes;
two new-test cloned-reference lint qualifications use from_ref without changing values
or assertions, followed by final focused13/0 and all-target clippy/fmt success.

Actual CLI/initialized MCP/authenticated loopback3/0 pass whole independent model/facts,
exact49 discovery, Unicode unchanged files/reopen, source/no-op/capture/file owners and
model-only page drift→Save refusal→Undo/source restoration. CPU/GPU capture and lossy
conversion remain refused. Initial E0308 in a test assertion is corrected before the
separate runtime RED. Whole-model helper reuse strengthens assertions after RED, before
GREEN. All caller production code stays unchanged.

The App test uses real controls for retained Ring→Open inferred CDX→known Save→Unicode
CDX Save As→explicit reopen, with complete independent model/raw/facts and clean history.
Its SDK input evidence is separate from the actual native acceptance below.

## Actual native macOS files

A separate signed `target/cdx-literal/Bondline CDX Review.app`, PID86673, runs the
4996ee7 production code. Later commits change only tests/fuzz/docs. The signed executable
has SHA256 `d4ab13ba53e07e61c8965717b123a3ef5f922e19a05c2e209fc4bd40b038ab3f`;
the unbundled build has SHA256
`d3d43d062b1e39a29a289c09e790f239ee0f2ba999eb12fe2341571abcbae6b2`.

Actual OS input checks retained eight-membered Ring→Open isotope-bearing CDX→known
Save→Unicode CDX Save As→explicit reopen, then Open four-byte-charge CDX→Unicode Save As
→explicit reopen. Complete queried models agree with independently assembled frozen
facts and caller A4, with clean history after reopening. Known Save changes file mtime
without changing its bytes. Both actual Unicode files equal their original sources:

| Case | Bytes | SHA256 |
|---|---:|---|
|13C methylammonium|353|`25966d5b508ab7ef1d064d831f85bf79bbeec65f331b0cb62e27a6a91942f554`|
|Four-byte charge AABBCC7F|230|`b9a0a9f4b58226666d4aba20fa35fb669834ead30b202ad5c99531ae03cd57eb`|

Independent struct readers and actual pinned RDKit observations are rerun on those saved
files. The first agrees on isotope/charges/H; the second retains the explicit normative
charge127 versus RDKit−86 difference. Raw low bytes are not normalized. Attached CPU and
GPU capture both refuse unsupported_content and preserve the complete state. No CDX
image, faithful preview, rendering parity or golden approval follows.

The native canvas honestly reports unmapped content and disables authoring controls.
The retained Ring inspector still shows its old drawing instructions; that preexisting
presentation limitation is not repaired by this data slice. AX row truncation is handled
using the fresh own window; filename focus/format selection are setup qualifications.
CJK filename glyph coverage remains limited, while exact UTF-8 paths/files pass.

The disposable review is quit; its exact bundle inventory and original PID are observed
absent. No post-Quit app probe launches another instance. The user-viewed creation
review365 and older apps/drawings stay protected. Local ignored receipts live under
`target/cdx-literal/research/`; startup credentials are private and are never committed.

## Bounded sanitizer

ASan97553 passes10000/1s, cov1261/ft2818/RSS173MiB. Forty-three immutable seeds and a
fresh regeneration match exactly; manifest SHA256
`bb61c910883ed54656d174fba53e9fd72b85c4b194eeae49096f88b2a19cb026`.
Independent checks accept19 new positive seeds/refuse21 controls; three original legacy
seeds retain prior qualification. Initial corpus max131072B differs from actual retained
mutation limit65746B and NEW/REDUCE max777B; no128KiB mutation claim follows.
The29351B ignored log has SHA256
`4b178ddf322325c03628a64cac41dbe6de2a38ba46dfb51df68471729750a17b`.

The old HNone invariant becomes an independent raw-record scalar/owner/duplicate proof
for all four fields, retaining strict absentHNone. The bounded linear record table avoids
repeated whole-source scans. Existing framing/source/replacement/new-tree/snapshot
assertions stay unchanged; display remains unavailable and model-page drift still refuses
CDX export. [Reproduction](../fuzz/README.md) adds no dependency and does not test Engine,
files, native input or chemistry validity through this parser harness.

## Final local gates

Full `cargo xtask ci`1390 on codef11a40c fails at the unchanged MCP
shutdown_without_clients_releases_the_listener_address test: replacement_bind51670,
AddrInUse. Before stopping,222 reports record1420pass/1fail/0ignored. Workspace fmt and
warnings-denied all-target clippy pass first. The176572B log has SHA256
`af9fda65b0b4dbb69272ec51a431f1ed9da140cafcabf683a6dc11f33adb754e`.

Run11036 completes only previously unexecuted stages: the other eight MCP targets;
model/render/UI/compat/xtask lib/bin/integration tests; workspace doctests; vendor3/0
and vendor warnings-denied clippy; layers; strict runtime licenses/seven assets; and the
pinned eight-file/five-bundle corpus. Its98 reports record751pass/0fail/1ignored,
including vendor3. The98069B log has SHA256
`f1316e84970dc580d04d0d4264a9dfad98ca9f812f6a35a6dc5c92151783f8db`.
Combined workspace coverage is2168pass/1fail/1ignored across319 reports, plus vendor3/0.
This is **not a clean aggregate or a causal listener fix**. The failing case is neither
retried nor weakened; protected test SHA256 stays
`fd96b257cd6c00b52e97544d652ed69fdbaaaa454e1f3a9360d77f3292218fb0`.
Both gate jobs and all implementation workers are terminal. Separate fuzz formatting
passes; the already completed sanitizer/native checks are not rerun to manufacture a
different aggregate result.

No dependency/lock/vendor/font/asset/golden/workflow change. Actions stays OFF; every
atomic integrated commit is pushed. Windows/Linux/WASM, full CDX Text/styles/queries/
stereo/editing/rendering, general interoperability and the remaining M0–M5 target stay
open. Historical MCP listener failures remain unexplained; no causal fix is claimed.
