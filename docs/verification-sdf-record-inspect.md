# Read-only SDF record inspection

This isolated bondline-rxn slice starts atb842e36, the accepted whole-SDF session feature.
It adds only sdf.record.inspect through the one registry, retaining all18 existing names
for a total19. No setter, delete/navigation, preview, UI field editor, new listener/pool,
dependency, lock, codec/model/renderer/fixture/golden change or native operation belongs to it.
It is partial M1. The original query policy is separate from reference UI parity.

## Contract and ownership

The typed closed parameters are expected_revision, positive record_id, field_offset=0 and
field_limit=64 within1..64. Preparation checks kind/revision/record/offset/limit using compact
existing metadata. It clones immutable SdfDocument backing and captures current result,
revision and session generation without mutating any selection, policy or content.

Only deferred completion reparses the existing bounded source, builds ordered field views
and measures exact JSON. It reuses Prepared/Completed and the current two-worker/reply/close
barriers. No engine guard crosses parsing/cloning/budget work; a failed worker reservation
has no transient side effect to undo. Queries take no save lease. Current/saved/original,
dirty/history/close and file metadata remain unchanged. A historical completion retains its
captured record/revision/generation after newer work and never installs current UI state.
Relevant Debug output contains only identity/counts, never headers or values.

The optional sdf_record result contains the captured identity, actual per-record version,
record/MOL byte lengths, count/offset/limit, ordered occurrence fields, next offset and
page_limited flag. Each field reports exact header text without EOL, name/textual DT,
retained value bytes/line count, optional complete logical lines and explicit omission.
Duplicate names and leading-zero DT identifiers remain literal. [] means empty data;
null means an omitted value, never an empty substitute. No raw MOL/source is disclosed.
Automatic document.inspect still contains only the old collection metadata.

Both raw value bytes and serialized line-array bytes must be<=65,536 to disclose data.
Reasons are value_byte_limit or value_json_limit, with exact retained raw byte/line counts.
The complete compact UTF-8 sdf_record object is<=1,048,576 bytes, counting metadata,
headers, vector punctuation and JSON escaping. The maximal fitting prefix up to64 entries
is returned; nonterminal pagination always advances, including omitted-value occurrences.
Zero-field and exact-end-offset queries return an empty terminal page. Request-count versus
byte-budget truncation is distinguished by page_limited. Full state/CLI/loopback envelopes,
LF framing and MCP's structured/text duplication are outside this page-only budget.

Cancellation is checked around the existing bounded parser and between field operations;
one parser/serialization call is not preemptible. Query results/temporary borrowed views
remain in-flight/client data outside retained Engine accounting, consistent with the
existing exclusion. There is no new persistent cache or model allocation to account.
See control-protocol.md for exact errors, byte meanings and envelope caveats.

## Independent expectations and test-first evidence

Before implementation, target/sdf-record-red.log exits101 with two intended failures:
the exact18-name registry lacks the required query, and an actual Open→query returns
UnknownCommand(sdf.record.inspect). No production implementation preceded that runtime RED.
The first GREEN log then shows both original query/schema cases passing.

The original690-byte and777-byte fixtures stay byte/hash exact. Literal headers, values,
occurrence indices, leading zeros, byte/line counts and actual molecular versions in the
new shared test support are independent expectations. The unchanged raw scanner and
per-MOL pinned RDKit consumers check actual original/CRLF/opaque source files. They do not
define duplicate-name retention by RDKit's last-value map or discard valid DT-only data.

An independent Python stdlib compact-JSON process checks real pages and value arrays.
It reconstructs the next candidate prefix and requires it to exceed1MiB when a page is
byte-limited. An original literal source with escaped3KiB headers and160 tab/quote/backslash
lines per field forces multiple pages; every occurrence is returned once in order.
Boundary vectors include an exact65,536-byte JSON array, a65,537-byte raw value,
JSON-only and escape-only oversized values, empty/spaces/tab data and CRLF byte counts.
These are test-only original vectors, not a new corpus pin or reference asset.

Scoped command/CLI/MCP tests pass, including actual executable Open→query, stdio tools and
authenticated socket queries for both record encodings. Automatic inspection before/after
discloses no field data. Captured reverse completions/cancellation, stale/wrong-kind/missing
records/invalid bounds, dirty source/history/saved baselines, zero/end pages, worker-busy
atomicity, save-lease independence and close/reply flush lifetime are exercised. Read-only
queries request no repaint. Existing command names, lifecycle assertions and original
format/source tests remain meaningful; only exact discovery/count expectations add the query.

Ignored logs: sdf-record-red.log, sdf-record-first-green.log, sdf-record-expanded.log,
sdf-record-scoped.log and sdf-record-clippy.log. All use the absolute RXN target,
incremental0/dev-test debug0 and the existing pinned BONDLINE_PYTHON. Scoped all-target
clippy passes with -D warnings. The final frozen-source aggregate will be recorded below
separately from those scoped checks.

## Frozen candidate aggregate

`cargo xtask ci` printed the exact isolated workspace
`/Users/sarenaberg/Workspace/doozle/open-source-world/bondline-rxn` and exited0.
It passed592 workspace tests plus3 macOS bridge tests =595 total,0 failed/ignored,
with workspace rustfmt/all-target clippy, strict layers, unchanged all-three-platform
declared license closures/assets and the pinned corpus. This is14 additional tests on the
581-check b842e36 base; it does not include the separate later SMILES integration.
All prior93 UI tests remain unchanged and pass, without an actual native query claim.

Ignored `target/sdf-record-ci.log` is72,817 bytes/SHA-256
2f39ae3021a8afd480534a40b92e35e332cc5c4a61142c8db886cbd62476543a.
The12 frozen source/test files are hashed in `target/sdf-record-source-freeze.json`,
3,690 bytes/SHA-256
aa84385d60cb03eda70b86e15e1ada6eeba0f5e56c6cfc21b0c7622ae7932fe4.
All source/test hashes stayed unchanged throughout this gate; the three documentation
updates are additive. No live build/native/helper job belongs to this handoff. Root owns
the atomic commit, independent review, receiving integration and hosted evidence.

## Remaining scope

No new UI or native query flow is implemented or claimed. Explicit field setter, editor,
navigation/delete/identity across reordered records, chemical preview/layout and reference
existing-field/export behavior remain separate work. The current headless query may return
historical results; a future UI must validate the full captured identity before display.
