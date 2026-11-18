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
It proves the App workflow, not OS input. Native macOS, bounded ASan and final aggregate
are pending separately. Protect user-viewed review365 and older apps/files; any new
native acceptance must use a separate own app.

No dependency/lock/vendor/font/asset/golden/workflow change. Actions stays OFF; every
atomic integrated commit is pushed. Windows/Linux/WASM, full CDX Text/styles/queries/
stereo/editing/rendering, general interoperability and the remaining M0–M5 target stay
open. Historical MCP listener failures remain unexplained; no causal fix is claimed.
