# Native capture identity and image preflight

2026-11-14. Headless contract prerequisite for the native preview/cache step in
[the SDF plan](research/sdf-preview-plan.md). This adds no native worker, texture,
UI/cache integration, input or window acceptance. Actions remains disabled.

## Opaque content identity

`Engine::capture_stamp()` returns an opaque `CaptureStamp`. It is Clone, Debug,
PartialEq and Eq, with private Weak<StoredContent>, revision and generation.
Equality uses Weak::ptr_eq plus both counters. `capture_stamp_matches()` compares
the current complete content identity. It does not compare only a projected
model, source length, metadata summary or content value. Different Engines and
new Opens have different identities; Undo returning the old owner does not
restore its old revision. Same-size SDF field edits invalidate a stamp even
when the automatic collection summary and document:null are unchanged.

The stamp has no public constructor or Serialize implementation; its compile-fail
doctest enforces the latter. Debug exposes only revision/generation, not source
bytes, paths, pointers or owner Debug. Cloning stamps does not retain a strong
owner or raw-source allocation. Weak backing-control-block retention is separate
from retaining the dropped value's source payload; no hard RSS claim is made.

Close state is deliberately independent. An Approved/Committed close does not
change this identity. Native preparation and image installation must separately
guard close, viewport/DPI and the registered-work lifecycle. These helpers do not
mutate authored/saved state, history, file requests or trusted acknowledgments.

The official [Weak documentation](https://doc.rust-lang.org/std/sync/struct.Weak.html#method.ptr_eq)
describes non-owning references and allocation identity. Online docs identify
Rust 1.100.0; this uses stable APIs actually compiled on pinned 1.94.1.

## Bounded own-output PNG envelope

`PngCapture::png_bytes()` returns unchanged decoded PNG bytes or a typed
`CaptureImageError`. `code()` is resource_limit for the size bound, otherwise
invalid_capture_image. Errors contain no raw PNG/base64 source.

This is a preflight for the existing pinned `bondline_io::encode_png` producer,
not a general PNG reader. It checks exact image/png metadata, dimensions 1–2048,
checked per-dimension encoded/decoded budgets before allocating a decode buffer,
canonical standard base64, PNG signature and complete IHDR, matching header
dimensions, depth 8/RGBA 6/compression 0/filter 0/no interlace, then the exact
IHDR→one IDAT→IEND framing with no auxiliary or trailing chunks. Declared chunk
arithmetic/slices are checked. Ancillary compressed metadata is never forwarded
to the native image decoder. Valid general PNGs outside this producer profile
can be refused intentionally.

CRC, DEFLATE and actual scanline/pixel validity remain the image decoder's job.
A negative test corrupts an IDAT payload byte: this envelope preflight preserves
the bytes, while independent Pillow rejects their CRC/payload. Callers must not
treat preflight success as complete image validation.

The dimension-specific bound is derived from the actual pinned producer:

```text
R = height * (4 * width + 1)                 # RGBA8 plus one filter byte/row
B = ceil(R / 65535)                         # maximum-size stored DEFLATE blocks
PNG_limit = R + 5*B + 6 + 57
base64_limit = 4 * ceil(PNG_limit / 3)
```

The current Fast path chooses fdeflate and falls back to stored data rather
than keeping a larger result. Stored blocks need 5 bytes each, zlib header/checksum
need 6, and signature/IHDR/one IDAT/IEND add 57. At the maximum 16,779,264 filtered
bytes, PNG_limit is 16,780,612 and base64_limit is 22,374,152. This fits below the
encoder's single-IDAT split threshold. The helper also checks padding-derived
decoded length before allocating its exact decode slice; it does not allocate
from an unchecked header or merely from an arbitrary base64 input length.

The original 64×255 random-RGBA case has exactly 65,535 filtered bytes and an
actual 65,603-byte stored output, testing the block boundary rather than only
compressible page pictures. Actual 1×1,1×2048,2048×1,65×255 and2048² writer
outputs also pass their bounds. No encoder/compressor code was copied or changed.

Sources inspected for this policy:

- [W3C PNG Recommendation, 24 June 2025](https://www.w3.org/TR/2025/REC-png-3-20250624/),
  signature, chunk framing, IHDR/IDAT/IEND, RGBA8 and filtering;
  [DEFLATE RFC1951§3.2.4](https://www.rfc-editor.org/rfc/rfc1951#section-3.2.4),
  stored-block structure; [RFC4648§4](https://www.rfc-editor.org/rfc/rfc4648#section-4),
  standard base64 grouping/padding. The closed own-output subset and budgets are
  original policy, not full PNG-conformance claims.
- [png 0.18.1 encoder at its published crate revision](https://github.com/image-rs/image-png/blob/2a3f980245e3ae38b82ade96533e7b450e8477bb/src/encoder.rs):
  actual Fast fallback and one-IDAT path; local file SHA-256
  `787f6f8922588a74436c76a3b2c2b5689206f7ed13a19a36227b8809710812ce`.
  The associated common.rs SHA is
  `1da94ac8aae8bf7fb63efcf4b110416bee0aaf294d042476b70591b670c33519`.
- [fdeflate 0.3.7 stored writer](https://github.com/image-rs/fdeflate/blob/c365c7e6ffa81feb2e1fb762eed7299f05c9b0ca/src/compress.rs),
  SHA `36c2bf996976d86c6a398686aaa5b178defe526340a0717dfb7c956b4be61b6e`;
  [base64 0.22.1 checked output-slice decoder](https://github.com/marshallpierce/rust-base64/blob/e14400697453bcc85997119b874bc03d9601d0af/src/engine/mod.rs),
  SHA `5e4a6c0e86417f3b62350264ef383f91e9864390f7c315d786ecd8e9c920ee9f`.

These dependencies/locks remain unchanged. The exact crate revision facts came
from their existing .cargo_vcs_info.json files, not an invented future pin.

## Authoritative native presentation fact

Root separately received renderer commit
`f91aa949c777e724342859822fd4d8ffc0deb94f`: actual produced Labels retain main em,
and GraphScene::label_em_px() reduces those facts (None for no labels). This
slice's private CaptureInput now keeps that GraphScene until it samples the
metric once, then consumes exactly its Scene for the existing CPU/GPU/PNG path.
There is no second projection/parsing pass or duplicated UI font constant.

`PngCapture` gains `#[serde(skip)] pub label_em_px: Option<f32>`. It is a Rust-only
presentation fact; capture JSON still has exactly the same six fields, and no
command name, parameter, request schema or wire envelope changes. The parent
explicitly authorized this 0.1 Rust-struct addition after checking that no old
external struct literals existed. Forged test packets set it to None.

An empty page reports None; a literal labeled 200pt-square page reports 24px and
48px at the independently chosen 464² and 864² viewports. Both Some/None are
absent from serialized output. Native integration must compare this real em
with the actual displayed-image scale to enforce logical readability; those
DPI/cache/texture/native behaviors remain pending, not verified here.

## Actual RED/GREEN and preservation

Cargo used 1.94.1, absolute
`/Users/sarenaberg/Workspace/doozle/open-source-world/bondline-preview-core/target`,
incremental 0, dev/test debug 0 and the existing Root pinned BONDLINE_PYTHON.

```sh
cargo +1.94.1 test --locked -p bondline-commands --test capture_identity --test capture_image
cargo +1.94.1 test --locked -p bondline-commands
cargo +1.94.1 clippy --locked -p bondline-commands --all-targets -- -D warnings
cargo +1.94.1 fmt --package bondline-commands -- --check
```

- RED 86267 exits 101 before implementation: missing capture_stamp/matches,
  png_bytes and label_em_px in the actual old API. Full output is retained in
  target/preview-capture-contract-red.log.
- First GREEN 38787 exits 0: 4 identity and 8 image tests, no production correction.
  The real renderer accessor was received before this compile; no stub was used.
- Full commands 91031 exits 0: 199 passed, 0 failed/ignored, including those 12 tests
  and the compile-fail no-Serialize doctest. Historical/read-only/source/Save/UI
  guards and old capture/format consumers remain exercised by the full package.
- All-target commands clippy 88234 exits 0 with warnings denied; scoped fmt exits 0.
  The ignored handoff pins all logs, exact source diff and terminal handles.

New tests inspect actual registry-produced PNG bytes with literal header facts,
independent Pillow 12.3.0 verification and every RGBA byte of an original 8×8 page
grid. Unchanged earlier capture tests separately use Python standard base64 and
Pillow on the actual serialized registry reply. New malformed cases cover MIME,
dimensions, pre-decode limits, canonical padding/alphabet, truncation, forged
IHDR fields, unsupported encoding, extra/duplicate chunks and trailing bytes.
Stamp tests cover weak raw-source lifetime, metadata-only Debug, independent
Engines, history A→B→A, same-size source-only edits, identical reopen, close
independence and unchanged file intent/state from reading stamps.

The 151-file protected baseline has 146 exact identities, two intentional owned
core replacements (capture.rs and its lib.rs exports), and three explicitly
received renderer replacements. The new renderer presentation_metrics.rs test
is also a parent-owned received file. Those renderer facts are not relabeled
as unchanged baseline or as this worker's implementation. Within the protected set, old format/source codecs, normalizers/layout,
tests/oracles/fixtures, font/golden and dependency/lock files remain byte-exact. Existing scene pixels and PNG/wire behavior pass
their old tests; only the authorized producer metric carry-through changed.

No Git, install/dependency change, shared-artifact cleanup, Actions call,
GUI/input/activation or native window was performed. No native preview/cache or
MCP worker ownership is implemented by this slice. Root owns review, Git,
source-register/global-status updates and the next integration.
