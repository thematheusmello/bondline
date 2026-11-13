# Parser fuzzing

This isolated test package is never a shipped dependency. cargo-fuzz 0.13.2 drives
libFuzzer 0.4.13 with AddressSanitizer on nightly-2026-04-01. The C++ libFuzzer runtime
has NCSA terms in addition to its Rust wrapper's MIT/Apache terms; this is test tooling,
not an expansion of the native runtime dependency allowlist.

```sh
rustup toolchain install nightly-2026-04-01 --profile minimal
cargo install --locked cargo-fuzz --version 0.13.2 --root target/tools
mkdir -p .cache/fuzz/cdxml
cp crates/cdxml/tests/fixtures/unknown.cdxml .cache/fuzz/cdxml/
PATH="$PWD/target/tools/bin:$PATH" cargo +nightly-2026-04-01 fuzz run cdxml_parse .cache/fuzz/cdxml -- -runs=10000 -max_len=65536 -timeout=5
```

Do not use the committed fixture directory as the writable mutation corpus. Keep generated
inputs/artifacts ignored; minimize and add any failure as an intentional regression test.
The harness checks bounded parser execution and exact source preservation on accepted input.
When an ordinary unnamespaced attribute exists, it also replaces its value with Unicode,
markup delimiters and whitespace, reparses, verifies the decoded value and checks the
original stayed unchanged. Inputs too close to the output size bound may reject the edit.
The same actual io crate also maps accepted direct-fragment geometry and patches one
explicit position to the published 72/144 point vector, reparses and checks the original.
This pulls the io/render dependencies into the isolated build, but does not initialize a
GPU or fuzz renderer behavior. The isolated workspace uses the same original loader/hex
compatibility patches and pins gpu-descriptor 0.3.0 in its own lockfile. Existing parser/
libFuzzer package pins remain unchanged. This extra build graph is test-only.
Raw UTF-8 inputs and parsed attribute values also exercise exact integral-property and
UINT32 source-ID decoding. Successful conversions must agree with integer range checks
and retain their exact value after canonical decimal formatting. Independent Decimal
expectations live in the numeric integration tests.
The read-only elemental mapper also checks unchanged source plus unique application IDs
and fragment-local, distinct bond endpoints on accepted inputs. Its local reference maps
are exercised with the original fixture seed. Chemistry perception and a complete editable
file workflow remain outside this harness's claims. Accepted owned StoredDocument values
must export byte-exact source. The harness edits preview dimensions, requires stale-source
export to fail, restores the model and requires exact passthrough again. Unsupported page
layouts may reject the owned projection without rejecting the standalone syntax.
Independent XML semantic assertions live in the external Expat tests. A short smoke run is
not exhaustive parser conformance or a security audit.
