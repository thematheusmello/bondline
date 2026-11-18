# Source-preserving batch XML edits — 2026-11-17

This M1 prerequisite starts on `ab7c312`; `3240a1f` separately records the user's logo/tool
recheck. The full M0–M5 goal remains active. The [contract](research/source-preserving-xml-edits.md)
uses public XML/CDXML sources and preserves the current canonical model/source guard.
It does not enable a new command/tool or make imported documents editable.

## Independent declarations and actual failures

Root owns execution and tracked edits. Workers independently declare exact expected
bytes, resource failures and Expat expanded names; source-only reviews do not count as
execution. Original declarations, frozen receiving inputs and logs remain ignored under
`target/noncanonical-cdxml/`.

| Receiving stage | Actual result |
| --- | --- |
| Existing CLI noncanonical profile | An original 464-byte document opens, but `bond.set_order` to Double exits2/unsupported_content. Sparse XML IDs, mixed quotes, numeric lexemes and bond-before-atoms remain unchanged. This runtime counterexample is still unresolved by the syntax primitive. |
| First batch declaration `7565` | API RED: missing Edit/apply_edits. No runtime test ran. Expanded `75907` and resource `60269` independently fail to compile on the same missing API. |
| Initial producer `99718` | First seven exact-byte/Expat cases and four resource cases pass. Resource expectations come from independently assembled final XML and the unchanged parser, including its existing Xml node-limit error. |
| Shared-boundary `85617` | Runtime RED0/1: appending to root and its selfclosing last child silently places the root addition inside the child. The actual output is well-formed but has the wrong nesting. |
| Ownership sort `34513` | Same literal assertion and Expat check pass in both caller orders. Descendant closures precede ancestor appends at a shared source offset. The complete cdxml crate passes. |
| Attribute index `18325` | All cdxml tests pass; one unused inner pattern binding warns, then is removed without changing behavior. Source attribute spans are indexed once per element. |
| Helper lint `10215` | Clippy refuses two new test-only expect_err calls. The helper changes to Result/ok_or/?, keeping all vectors/discriminant/display assertions. Root's first mechanical rewrite drops closing delimiters; fmt and `53570` fail at that setup syntax. Restoring the frozen helper and correcting the rewrite permits clippy to exit0 in the fresh receiving run. No product failure or relaxed assertion is inferred. |
| Long-tag control `18924` | Passing-first exact-byte performance control: one128KiB padding attribute plus255 new attributes, at the final256-attribute bound. Observed185.684833ms before tag caching; this is not an invented behavioral RED. |
| Tag cache `67407` | Identical control passes in1.728167ms on the same local development profile. Timing has no pass threshold or general performance claim. A second passing-first control confirms the documented compacted QName/whitespace offset limits do not corrupt full-source spans. |
| Namespace `44680` | Runtime RED3/1: `<n/>` appended into a prefixed destination carrying a nonempty default namespace is silently captured. Explicit namespace/reset, non-element and no-default controls already pass. |
| Namespace fix `32654` | Same four namespace cases pass, including paired/inherited-default parents, nested resets and multiple siblings with independent literal Expat expanded names. All33 cdxml tests pass, retaining every old parser/writer/attribute/numeric test. |
| Final focused clippy | Exit0 with `-p bondline-cdxml --all-targets --locked -- -D warnings`; no warnings. |

The initial seven-case declaration is9,918B/SHA256
`b8b65cd5a3740dc7171d1200148ac9795adb024c57a1d42ef2e81f4df9f46551`;
the four resource declarations are5,966B/SHA256
`d97681a64c10a2aa08cbe54a6bf04326c431a881dcda7e8d5aababf964d0f8d7`.
The namespace declaration is6,484B/SHA256
`45d341e49a781719441f69cfb356d7fc4c3b86aee262159c00710c8d04fb80fb`.
Only formatting and the explicitly qualified error-helper conversion change those
adopted expectations. No old parser, resource policy, corpus, consumer or golden is weakened.
The two new Expat helpers also adopt the existing python/python3 fallback when
BONDLINE_PYTHON is unset; this changes setup only, preserving every vector/assertion.
An actual unset-environment `cargo test -p bondline-cdxml --locked` passes all 33 cases.

The implementation adds immutable SetAttribute/RemoveAttribute/AppendChildren operations,
one checked byte plan and final source validation. It retains quotes, unrelated bytes,
unknown content, exact string no-op lexemes and original qualified tag names. Conflicting,
foreign, malformed, destination-dependent namespace and overbound edits fail without
mutating source. Existing edit.rs is unchanged; writer.rs only exposes its existing
closed name predicate within the crate. There is no new dependency or unsafe boundary.

## Fuzzing and aggregate receiving

The separate `cdxml_edits` harness uses existing isolated test dependencies. The v2 source
proposal is10,046B/SHA256
`beef556b1c919a65db10779b331b3ec93e18526a08b57bed8caa13dd0687d305` before formatting.
Its original25-seed recipe includes a16KiB boundary seed, namespace cases, nested
ownership, hostile fragments and current attribute/depth boundaries. Root generates a
fresh ignored corpus, preserves its original source copies and manifest, and runs the
pinned nightly/cargo-fuzz/ASan command from [fuzz README](../fuzz/README.md).

First ASan receiving60876 builds, then aborts during its constant namespace control before
mutation fuzzing. The new harness expects None for explicit `xmlns=''`, but the pinned
roxmltree API returns Some(""). Root changes only that exact expected representation;
the independent Expat test already verifies namespace absence. The failed log is5,294B/
SHA256 `49b14f8b55081a3f950e7388eb0c01a6b3a5f6afc13a0d09c040fc13c7dac31d`.
Its zero-byte artifact is preserved, not reported as a producer memory bug. Fixed-input
replay61159 exits0 and explicitly says fuzzing was not performed.

Fresh declared seeds then drive corrected ASan4982 to exit0: **10,000 runs in2s**, reached
16,384B in an actual NEW/REDUCE line, coverage2642/features7297, final reported RSS443MiB.
There are25 declared seeds, including an empty case; libFuzzer reports24 nonempty files
and executes its initial empty input separately. The final corpus has493 entries/355KiB.
This is bounded syntax fuzzing, not the full64MiB or200000-edit budget. Log72,884B/SHA256
`9befc6d1c550b8befe75795c347939d374c8f9284d26fe1dcae7a16472c4d55a`;
the original seed manifest is3,866B/SHA256
`46a63a98529b60974d84ef0f0911fabc8d60072e54e780075b25d1ea0d1adb64`.

The bounded frozen production review finds no further material issue after the Tag cache
and namespace fix. Its5,499B report SHA256 is
`f5499c6070246264435393dd28a8f1b57de86787a6eba6344f6173b6cfbc93c6`.
It separately qualifies the test-only interpreter change and reported fuzz expectation
correction; no executed result follows from source review. Earlier frozen review/input
records remain intact. No hosted/platform/native workflow follows from headless syntax
tests. Actions remains OFF.

Fresh full local `cargo xtask ci`70957 exits0: **1,735 passed/0 failed/1 ignored across253
result lines**. Fmt, warnings-denied workspace/vendor checks, strict layering, seven
bundled assets/notices, unchanged runtime license closures and pinned corpus pass. The
218,681B log SHA256 is
`d1ab9b942d053d3ad1793ccdc2aa5f3641baeb38d8bd1a3074089e5c744d55ae`.
The old offscreen artifact case stays ignored. No protected MCP listener policy/test,
workflow, lock, vendor, font, asset, corpus or golden changes. A natural passing gate
does not explain the historical AddrInUse failure. Both protected user app binaries keep
their recorded hashes; no user drawing/window is inspected or changed. All Root jobs
are terminal at the primitive checkpoint.

## Continue the full M1 workflow

The original CLI runtime RED above remains the next integration target. Reconstruct
immutable source↔baseline bindings, classify a complete supported elemental profile,
then apply typed changes and new-object appends without canonicalizing original bytes
or IDs. Preserve unknown/unsupported semantics with existing refusal and exact source
passthrough. Keep whole-MOL metadata-loss permission separate from editable CDXML source
authority. Independently test current commands/history, actual saved files, malformed/
unknown/drift cases and native editable Save/reopen before widening the guard.

No new tool, registry schema, source-profile authority, CDX/MOL/chemistry coverage,
approved golden, runtime asset/dependency, platform result or release is claimed here.
