# Original CDX syntax fixtures

All fixture bytes in this directory were authored for Bondline and are licensed
MIT OR Apache-2.0. No proprietary sample file was copied. They are syntax test
data, not a model or chemical-compatibility claim.

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

Fixtures must remain unchanged when adding parser features; author new vectors
rather than replacing an existing failure case with easier data.
