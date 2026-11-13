# Original CDX field-vector fixtures

All fixture bytes in this directory were authored for Bondline and are licensed
MIT OR Apache-2.0. No proprietary sample file was copied. They are original format
test data; the chemical assertions below apply only to the stated authored scene.

`unknown-implicit.cdx` has the published 28-byte header, a root property,
page42, a page property with deliberately noncanonical small extended length,
custom objectC123/id77, a custom property, and balanced terminators. Its last
16 header bytes are deliberately nonzero opaque data. This verifies retained
unknown header data rather than claiming those bytes follow canonical zeros.
The magic/marker and record field widths follow public CDX documentation:
https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDX.htm .
Header-implied root framing is independently demonstrated in its public
IntroExampleSimple.htm table, whose conflicting first-listing bond references
are not used as chemistry expectations.

`unknown-implicit.cdx` SHA-256:
`34fa96aae53489f8987173af07646ef2391772ac11516bc5c62942187c9e9a96`.

## Original explicit-root chemical scene

`original-ethane-explicit.cdx` is the unchanged explicit-root output of the existing
project-original `xtask/oracles/cdx_oracle.py fixture explicit` generator. Its generator
already existed in public source commit `fb86832a0c9b7744fb3ef8f8115c7ed296164505`;
this checkpoint materializes that output as a binary fixture. It is not a downloaded
foreign file or output from a reference application.

Exact size: 65,750 bytes. SHA-256:
`63ef1a74133e45de915d5fd25b2ca3321a95dbe5114873a5a59f2f45217000d2`.

The scene has explicit document root 0, page 42, fragment 100, carbon nodes 101/102
and bond 103 referring to those nodes. Positions are (10.25, -3.5) and (26.25, 8.5)
document points, encoded as signed 16.16 with y before x. The single-bond bit is
0x0001. Original opaque properties occur on document, page, atom and bond; nested
custom objects and a 65,535-byte extended property exercise source ownership and bounds.

The regression reads this actual checked-in file, requires equality to the unchanged
generator and its pinned self-test identity, compares all 29 record events to the
independent Python `struct` decoder and checks exact Rust source passthrough. The
existing external RDKit 2026.09.2 verifier then confirms CC/C2H6, two carbons and one
single bond. Replacing only the bond-order property with 0x0002 must match the independent
full-byte double-bond vector and read as C=C/C2H4. The original file stays unchanged.

Numeric sources, generator identities and external-reader provenance are recorded in
[the independent oracle notes](../../../../xtask/oracles/cdx-README.md). RDKit's observed
coordinate output retains raw 16.16 magnitudes and reverses y; it is not a point-unit
conversion oracle. The raw point values are checked independently with `struct`.

The narrow Git `-text` attribute preserves CDX bytes across checkout settings. This
checkpoint does not add a fetched corpus entry, change parser acceptance, implement
application CDX Open/Save, or establish foreign-file/general CDX compatibility.

Fixtures must remain unchanged when adding parser features; author new vectors
rather than replacing an existing failure case with easier data.
