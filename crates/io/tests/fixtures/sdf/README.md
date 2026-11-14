# Original SDF V2000 vectors

`ordered-fields-v2000.sdf` is original MIT OR Apache-2.0 test data. Its two independent
ethane records use the project's original `../mol/ethane-v2000.mol` geometry; the second
has an intentionally empty title. The ordered fields contain an empty value, multiline
text, a combined named/DT header with original registry text, a DT-only header, two
different values for one duplicate name and a spaces-only value. No proprietary template,
reference application output, or vendor example is copied.

Exact fixture: 690 bytes; SHA-256
`f95838cbb55c107c649ce2eb99ac62e191f747ce6994a171e64019f8e6c53d1a`.

The first delimiter has an original suffix to exercise the published prefix rule. Extra
empty padding follows the EMPTY item as an explicit read-only consumer accommodation;
the canonical writer emits one blank terminator. Source and EOL bytes remain exact.
The SPACES value is three literal spaces. Its narrow Git whitespace attribute preserves
that meaningful data without disabling whitespace checks for code or other fixtures.
The parser requires complete records and refuses global trailing blank padding, which
could instead be a truncated next record with an empty title. It accepts only the
explicit ASCII profile; balanced optional registry parentheses and unambiguous field
identities are conservative subset policies, not claims that all other free-text
metadata is invalid under CTfile. The writer retains every supplied MOL byte, including
unknown properties and CRLF, while emitting new wrapper lines as LF. A MOL header or
property beginning `$$$$` cannot safely be embedded and is explicitly refused.

Format facts: BIOVIA Chemistry 2026 CTfile Formats, chapter 10, printed pages 97–99:
https://www.3ds.com/assets/invest/2026-01/biovia_chemistry_2026_ctfile_formats.pdf
Unknown optional header text and duplicate/DT fields must not be reconstructed from a
consumer's last-value dictionary. Fixtures are byte-preserved through `.gitattributes`.

The external test oracle separately checks every raw range and ordered value, and reads
each contained molecule with RDKit 2026.09.2. Its full SDfile supplier drops the DT-only
item and later data/record in this vector; this is recorded as a consumer limitation,
not whole-file compatibility. Named-only output is checked with the full supplier.


## Original mixed V2000/V3000 vector

`mixed-v2000-v3000.sdf` is original MIT OR Apache-2.0 data, 777 bytes, SHA-256
`165e6f5d5cff37dae6315a07843d84733aa42dbbc497b2920da1702bad9526a0`.
It embeds the unchanged original ../v3000/noncontiguous-cc.mol (344 bytes) followed
by ../mol/ethane-v2000.mol (282 bytes), retaining their separate provenance. Its
ordered data contains EMPTY, two distinct DUP items, a combined MASS/DT0009 header
with original registry text, a DT0012-only item, three spaces as a value, and an
ordinary item on the second record. No vendor example or consumer dictionary was
copied. A separate Python byte construction established all literal record/MOL/item
ranges before the Rust mixed writer existed. Each record's actual MOL version is
asserted independently of its order or filename.

BIOVIA Chemistry 2026 printed 97 explicitly permits V3000, V2000 or a mixture in
one SDfile. All raw data/header/DT/duplicate expectations remain literal; actual
molecular consumers have a separate role. Unchanged sdf_oracle.py continues checking
V2000 vectors, including its observed DT-only supplier loss. Unchanged v3000_oracle.py
checks full-precision/large molecular payloads. The supplemental external
sdf_mixed_oracle.py tests SDWriter version switching and full-supplier ordinary
properties without claiming raw-span or duplicate fidelity. Its generated 667-byte
source SHA-256 is 88516961b6a197950ac83e09c441387bd2e9ed1c2da2d94accdffd1b9c359ca6;
that generated output stays ignored, not in this fixture directory.

The existing *.sdf -text rule preserves every byte on checkout; the new narrow
whitespace rule protects the meaningful three-space value. Fuzz only ignored copies.
