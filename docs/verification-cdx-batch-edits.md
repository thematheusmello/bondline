# Atomic CDX property replacement

Reviewed 2026-11-18 from `37d85f0`. Replace existing opaque payloads by
record handle. The short length covers 0–65,534 bytes and the 65,535
sentinel precedes a 32-bit little-endian length. Unchanged payloads keep
their exact length words. The published format makes each property a
record in its containing object's sequence. [CDX binary framing](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDX.htm).

The operation checks ownership against the exact parsed record index.
Distinct handles determine the source order of the splices. Even an
unchanged entry conflicts with another entry for the same record. The
combined byte count is checked against 64 MiB before allocating the
single output. Header, object IDs, owner links, terminators, unrelated
bytes and original length words for no-ops stay intact. The standalone
operation grants no app edit permission.

The compiler RED precedes the new API. The first runtime attempt rejected
the test's nonzero reserved byte for canonical 28-byte headers with a
separate file end. The two canonical fixtures now have zero reserved bytes.
The three other cases still test exact retention of the nonzero byte.
The next run passed the four batch cases. Five existing external cases
required the pinned Python setting. The qualified run passed **55/0/0**,
and warnings-denied all-targets Clippy and both formatting checks passed.

The four independent byte constructions cover all five framings, scoped
repeated object IDs, editing in both input orders, unchanged extended
lengths, empty and 65,535-byte changed payloads, duplicate/foreign/non-property
handles, and a growing payload that fits only alongside a second shrinking
record. The fuzz harness exercises ordered and reversed batches with raw
record and ownership comparisons.

Receipts are in `target/cdx-resources/next-text/label-edits`.
`batch-green-pinned.log` SHA-256 is
`96e76f2c3e8a3b7704762f39cd156f624ffc335203ea72d6b7a27ea839dc796f`.
