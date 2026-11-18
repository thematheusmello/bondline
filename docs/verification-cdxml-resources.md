# CDXML resource-table receiving

2026-11-17; base `70e6f27`. Original Rust logic/test vectors; public specifications only.
[Contract and sources](research/cdxml-resources.md).

Scope: standalone logical font/palette decoding and lookup, owned syntax edits and exact
source retention. This is a format prerequisite, not working imported-file appearance,
foreign authoring or a new UI capability. Registry46, controls39/22/16/12 and catalogue
16/124 retain their scope.

## Independent tests and actual failures

`cargo test -p bondline-cdxml --locked --test resources` first exits101: missing ResourceError
and Document::resources. This is API compile qualification, not runtime behavioral RED.
After implementation, session79326 passes the eight initial cases. Expat/Decimal is
separately qualified on both manually declared inputs before checking product output.

The ninth declared namespace/whitespace case fails session40718 with InvalidFontId (8/1).
Pinned roxmltree string-name lookup picks x:id before ordinary id. Explicit unnamespaced
attribute matching fixes it; the same test then fails session7985 with
InvalidColorComponent(r), exposing numeric XML whitespace. Trimming only XML whitespace
fixes it. Standalone session12622 passes all prior CDXML tests and9 resource cases.

A tenth case declares empty default-namespace resets on table/entry elements and preserves
old root admission. Session50397 fails9/1 with UnsupportedTable(colortable). The resource
view now recognizes None and empty namespace URIs as unnamespaced, leaving Document::parse
and complete IO source-profile authority unchanged.

Cases cover absent versus present-short palettes, intrinsic colours, nonsequential UINT16
font IDs including0/65535, integral spellings, name presence/charset opacity, invalid RGB,
namespace lookalikes, duplicate/nested tables, unsupported children, exact source and
foreign-node edit refusal. Literal expected bytes check font-name escaping and one RGB
edit; unchanged Expat/Decimal independently checks logical values, comments and unrelated
content before typed reopening.

## Integration and fuzz evidence

First full local `cargo xtask ci` session84816 exits0:1810 passed/0 failed/1 ignored
across269 result lines. Its229140B log SHA256 is
`b7b160f1e04d73782bc5f726ac9526c7ca4386313bf2e30ee0abd3aefb740410`.

Initial ASan session48136 fails during seed loading at an older attribute-edit harness
assertion. The original qualified-attribute seed is preserved as
`fuzz/artifacts/cdxml_parse/crash-473cf96e43d2ededfe85e0ac7cf6e518cc30bb5c`.
That assertion used roxmltree's local-name lookup to read back an ordinary attribute.
New independent literal/Expat legacy-writer case83428 passes1/0 with the existing
production writer unchanged: ordinary id becomes9 and x:id remains invalid. The harness
now explicitly reads the ordinary attribute and additionally checks every qualified
attribute is retained. This is a harness qualification, not a writer repair or weaker
assertion. Failure log7741B/SHA256
`b9a2d9df6854b6bbe6bfc80a2ac4fec1e42a2af4be4ef14d1d8eb155a5aa8837` remains.

Fixed-input replay46163 passes; it is not a fuzz run. Fresh qualified ASan1865 then
passes10000 runs in3s:19 original seeds, initial16KiB and actual NEW/REDUCE maximum16384B,
cov2834/ft6839/RSS381MiB. Its62689B log SHA256 is
`f5279b2067865b241570a2d996bb5f99848fb341a0edcd12289bf99a47aa1152`.
All19 separate original seed source hashes still match their manifest. The mutable
corpus is separate, never a committed fixture directory. This bounds a smoke run;
it does not establish full64MiB input, style/render, chemistry or GUI coverage.

Final aggregate5632 includes the legacy qualification: **1811 passed/0 failed/1 ignored
across269 result lines**, exit0. Fmt, warnings-denied workspace/vendor clippy/tests,
layers, seven assets/notices, strict155/166/221 runtime license closures and eight pinned
files/five corpus bundles pass. Log227941B/SHA256
`54c007d42f8433f18d40eb03e60b99bcf8b8e9012c3bc12c813013f4c5123aa4`.
Rust uses pinned tools, CARGO_INCREMENTAL=0/CARGO_BUILD_JOBS=2, zero dev/test debug,
the existing BONDLINE_PYTHON venv and PROPTEST_RNG_SEED=671308429. ASan uses the pinned
nightly/tool versions in fuzz/README.md. Logs live under ignored target/cdxml-resources/receiving.

Actual CLI46 output remains43338B/SHA256
`02bb76b9f909a3691fddbb5a1f3c2cc794a72f759f43194791b880213558a12c`.
Both locks, workflow and protected MCP control test retain their prior hashes; no
dependency/vendor/font/asset/golden/listener policy change occurs. Natural passing
socket checks do not explain historical AddrInUse. Source review finds no outstanding
codec issue at its pinned readpoint:5870B/SHA256
`2da7b82f0dd8ad6a654dd07574d422f6f4d9059dd6909b823b3ad8d936fa6c74`.
It predates the separately qualified legacy harness regression and makes no runtime claim.
Public-source review9684B/SHA256
`f1e5a0cfe94fbd122537bf206d7f3ecbfaa6fa59887bbccf2a178ab18800e347`
distinguishes historical SDK evidence from current26.1 behavior.

Native interaction is unchanged by this library-only slice and is not newly tested.
Previously unavailable native access, new Windows/Linux execution, full M0–M5 and foreign
style/Text/render workflows remain open. GitHub Actions stays OFF. User apps/drawings
remain untouched; this slice launches no native app.
