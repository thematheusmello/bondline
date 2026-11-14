# CDX filesystem library

2026-11-14, receiving root based on owned-snapshot source5e6b644. This delivers bounded
native filesystem library calls, not CDX registry commands, chooser, file association,
editable profile, chemistry or source-faithful rendering.

## Behavior and scope

read_cdx_file uses the established ordinary-file/canonical-path/64MiB/cancellation reader,
then StoredDocument::from_cdx with explicit A4 preview policy. Invalid syntax or unsupported
model content fails at decode; complete source remains authoritative. No extension-based
fallback or partial replacement is added. save_cdx_file dispatches to to_cdx through the
existing security-preserving atomic path: unchanged backing only, explicit overwrite,
same-directory temporary, sync, publication-aware errors and cleanup. Changed/source-free
CDX and lossy cross-format saves refuse before temporary creation.

The shared low-level filesystem/security/publication algorithm is unchanged. No external
dependency, lock, original fixture, corpus pin, bundled asset or approved golden changes.
Future command adapters must use Engine's existing FilePlan/Completed save reservation;
these direct library APIs do not provide application state, history or a receipt lease.

## Actual TDD and independent checks

- RED target/cdx-native-io-red.log, session7576 exit101: both public filesystem calls absent.
- RED target/cdx-native-io-stages-red.log, session81675 exit101: Cdx dispatch variant absent.
- The first implementation attempt passes three filesystem cases but fails the external
  check for the header-embedded fixture: the test selected the legacy-only decoder.
  target/cdx-native-io-first-green.log is that failed attempt despite its filename. This
  is a test-harness mismatch, not a product framing failure. The test now chooses each
  fixture's preexisting strict oracle; neither decoder nor its assertions was changed.
- GREEN target/cdx-native-io-green.log, session97750 exit0:149 IO tests, including four new
  filesystem tests and one seven-stage injected publication test.

Both unchanged original65750/65746-byte scenes load from Unicode/misleading-extension paths
and save byte-exactly. Independent Python struct plus pinned external RDKit2026.09.2 consumes
the actual saved file descriptor, checking full original bytes, framing, points, graph,
external CC/C2H6 and the documented raw-coordinate convention. No own-reader/own-writer
round trip substitutes for those consumers. Existing cdx_oracle.py and cdx_header_oracle.py
remain unchanged. Original provenance/hashes are in crates/cdx/tests/fixtures/README.md.

Real filesystem assertions cover canonical receipt/size/durability, no-clobber, explicit
replacement, old bytes retained on stale/source-free/cross-format failures, no leftover
temporaries, malformed decode, oversized/directory input and cancellation before access.
The seven injected stages distinguish prepublication unchanged destination from truthful
published=true after cleanup/directory-sync errors. Complete snapshot bytes stay unchanged.

The receiving-root aggregate passes443 workspace plus3 macOS bridge tests, formatting,
warnings-denied all-target clippy, layer/license checks and the unchanged five-file/four-
bundle corpus. target/cdx-native-io-ci.log, session69486 exit0, SHA-256
6419a5585ce943fed90fe7c069837beb8b2f9209796057c69b53c404eaaa98f4.
Native declared closures remain154Mac/165Windows/220Linux with two bundled assets.
Independent read-only review found no confirmed blocking/should-fix defect: ignored
target/cdx-native-io-review.md, SHA-256
db48f7960000ee550f0b5e8358740a99803e2cb71d40379a2988760afedbf43d.
The reviewer inspected source/logs; no second build execution is claimed.
The integrated feature plus test-only socket correction00e5b02 passes every hosted
43550955963 job: Mac446/Linux444/Windows436, all actual native window smokes and six
bounded10000-run sanitizer targets. Exact jobs/adapters and the earlier5e6b644 Windows
closed-socket assertion failure remain in ci.md. Main advanced after matching GREEN.
No GUI launch is claimed for a library-only change. General native metadata and foreign
CDX files remain outside the strict projection whitelist; an unsupported input is refused.
