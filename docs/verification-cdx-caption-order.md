# Ordered CDX caption verification

The [ordered caption contract](research/cdx-caption-order.md) extends the
[single-caption workflow](verification-cdx-caption-preview.md). The parser
keeps the established caption, white paper, resource and complete-source
guards. It gives supported sibling captions separate model IDs and paint
order. Existing CDXML edit authority and exact CDX Save rules still apply.

## Independent witnesses and runtime checks

The [18 original binary witnesses](../crates/io/tests/fixtures/cdx-caption/ordered/cases.json)
and provenance were pinned in `1ea2ad4`. The original CLI refused every
input. The new IO tests initially passed five and failed three at the original
unsupported Text error. All five valid models were refused. After adding the
ordered projection, the same target passed eight tests. The thirteen new
refusals and the preceding 63 keep their exact codes and details.

The first producer build exposed the Text ID's actual `u32` type. The later
warnings-denied clippy run exposed an identity conversion around the existing
fixed-width payload helper. Both were corrected without changing fixture
bytes, accepted ranges or expected errors.

The actual CLI matched complete models and 1024 × 1024 CPU/Metal output for
the front-first and back-first sources. The overlap also matched the source
with one omitted depth below an explicit positive depth. The image shows
only the upper caption's solid blue ink at `[184, 205, 196, 220]`. Independent
FreeType gives `[182.46875, 204.90625, 198.46875, 220.90625]`, within the
four-pixel tolerance. The disjoint example shows both text colours and also
matches CPU and Metal. Its identical source ID zero values represent two
anonymous objects. The shared RGBA SHA-256 for the overlap is
`befff10c3ee980936496102c465d6b121f093624f69ea9bb5f8dc381362ad2f0`.

The real CLI saved to a Unicode path and reopened the exact two-caption model.
An independent struct reader checked the actual file's source owners, signed
order fields and bytes. The IO Save test also verifies cross-format refusal.

## Actual macOS application

The independently signed review binary had SHA-256
`51baeb5b591d7a1275dd7415a1369030ed54bce4e53645df65a20d04654900db`.
Select Cyclooctane in the owned untitled window and open the overlapping
fixture. The inspector shows two captions and a read-only canvas, with only
the front blue H visible. Option-click on the H and drag on blank paper leave
the full inspected snapshots byte-identical.

Known Save advances the prepared mtime to `1795034885478544745` ns. Save As
writes `stack-再開-λ.cdx`. The two 193-byte files have the same SHA-256
`bacc5c24e6220971a3f6920b5c7237e6e96c29c683f98b0842754ce954189ce3`.
Both captions appear in the independent disjoint file. Opening the overlapping
equal-depth source brings up the old unsupported Text failure, and Cancel
returns to the disjoint document. Reopening the actual saved copy restores
both captions, the original model and the exact Metal frame above. All
observed documents remain clean, with empty history. The owned app was quit.

The evidence directory is `target/cdx-resources/next-text/caption-order.8KvDal`.
`native/native-evidence/verified.json` records independent checks of the
actual files, recovered model and PNG. Its SHA-256 is
`ea0c1b70d0166130a7956e5380618a46d0c5cc6e09371eb6c26211defcaf7bf3`.
The CJK name displays placeholder glyphs in the canvas chrome. The file
path and accessibility text preserve the characters. Physical display
density variation and the Windows, Linux and browser runs remain open.

## Sanitizer

The revised [parser harness](../fuzz/README.md#direct-page-cdx-caption-projection)
checks that every source Text object in an admitted caption document reaches
the model with one explicit run. Its complete-source, cross-format and
detached edit guards still run. The final ASan execution passed 10,000 runs
in one second. All 93 tracked seed files kept their hashes. The largest
recorded mutation was 202 bytes of the configured 16 KiB cap.

`fuzz-final.log` has SHA-256
`9384c3f435cda5c1eaaed108b4f25f41f7e76727c251eec5ec4be2dfb8239e88`.

## Final local gate

The complete `cargo xtask ci` passed **2,218/0/1** workspace tests across
326 reports and **3/0** AccessKit tests. Formatting, warnings-denied clippy,
layers, licenses, assets and the pinned corpus passed. The exact runtime and
test diff from `1ea2ad4` has SHA-256
`59c3a3224fccedba9b9669ad36149251028a14d05a09e72f27d426629686977c`.
The 279,122-byte `ci.log` has SHA-256
`a3a7657fe77359922aa59899d9eda288aa96aadc6d864e732285ed05d5cc4810`.

The IO library type-checks for Windows MSVC, Linux GNU and
`wasm32-unknown-unknown` on the pinned compiler. Application execution for
those targets remains open. The actual CLI still emits the same 49 records,
with the 47,135-byte SHA-256
`3cfd51a6e72891137cf0cc56fd756ff33652cfc685e5aaf877634b259cc41e4f`.
