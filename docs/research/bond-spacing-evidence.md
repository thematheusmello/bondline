# Public evidence for plain-bond spacing

This is the historical source-review checkpoint. The later
[contract](document-bond-spacing.md) and [receiving](../verification-document-bond-spacing.md)
record actual external vectors and the integrated explicit-root workflow.

Reviewed2026-11-17. Research only: no spacing implementation, source-profile widening,
new dependency, upstream test execution or reference-program access follows.

[Octet-Draw](https://github.com/SamChemMan/Octet-Draw/tree/6395bd582d2c5ba9404e9ddaf175e52e2a9f387c)
at6395bd582d2c5ba9404e9ddaf175e52e2a9f387c has an Apache-2.0 license.
[CDXML input](https://github.com/SamChemMan/Octet-Draw/blob/6395bd582d2c5ba9404e9ddaf175e52e2a9f387c/src/cdxml.ts#L1395)
reads BondSpacing into document/local styles; [style conversion](https://github.com/SamChemMan/Octet-Draw/blob/6395bd582d2c5ba9404e9ddaf175e52e2a9f387c/src/docstyle.ts#L318)
turns its percentage into a spacing distance separately from thickness.
[Plain Double geometry](https://github.com/SamChemMan/Octet-Draw/blob/6395bd582d2c5ba9404e9ddaf175e52e2a9f387c/src/render.ts#L1326)
places paths at opposite half-spacings or one signed spacing; ink extends half a line
width around those paths. This establishes a centreline convention for that implementation.
[REL-15](https://github.com/SamChemMan/Octet-Draw/blob/6395bd582d2c5ba9404e9ddaf175e52e2a9f387c/src/render.check.ts#L726)
checks percentage-dependent geometry, but was read, not executed. Its equal nominal/
actual lengths do not settle distorted-bond behavior. Its bold-bond adjustments differ;
do not generalize the plain-bond result to every display style or copy its defaults.

The MIT-licensed [chem-svg-cdxml authored measurement report](https://github.com/scotthartley/chem-svg-cdxml/blob/71622e02b4b2b0a976c0e141ddb80db9a78f4c09/references/style-spec.md#L43)
at71622e02b4b2b0a976c0e141ddb80db9a78f4c09 explicitly reports centreline separation2.592pt
for length14.4pt, percentage18 and width0.6pt. Its author attributes the data to26.1.0.6801
with ACS Document1996 settings. This is the author's public claim, not Bondline-observed
execution. No linked reference export, font, preset or measurement asset is retrieved or
shipped. Shared provenance between public artifacts has not been excluded; do not count
them as two independently reproduced reference-product oracles.

These sources corroborate centreline spacing more strongly than the archived SDK/Guide
prose alone. For plain equal-width strokes, separation s and ink width w imply clear gap
s-w, centered-Double full extent s+w, and symmetric-Triple extent2s+w. Thus s6/w1 gives
gap5, and w2 preserves separation6 while reducing the gap to4. The reported2.592/.6 case
implies gap1.992 and centered-Double extent3.192. These are analytic consequences, not
executed pixels or adopted foreign defaults. Existing original3pt fallback stays unchanged.

The public [relative](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/BondSpacing.htm)
and [absolute](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/BondSpacingAbs.htm)
properties still need separate Abs mapping/owner/ancestry qualification. Octet-Draw's
frame-based conversion is not proof for every unequal actual/preferred length; reviewed
GUIDE25 and third-party measurements retain their own version/scope. Missing foreign
defaults, root-Abs ownership disagreement, precedence and nonplain styles stay open.

Next declare independent original vectors varying preferred versus actual lengths,
percentage, thickness, owner overrides and explicit Abs. Use an external renderer that
demonstrably honors the tested property; graph-only RDKit consumption is insufficient.
Then lock typed data/source preservation, per-bond physical metrics, ink/gap/paper/hit
guards, command/history/adapters and actual native acceptance as one coherent workflow.
Keep dormant fields and unaddressed style values; do not relax opaque-source guards.

Source-only report8743B/SHA256
`d44671471de856be50e760d69974f2679dba5f99ab31ab7cc89020526a27ffb4`
and exact commit/license/excerpt pins are disposable under target/bond-spacing/independent-research.
This document retains the actionable evidence without relying on that cache or copying
third-party implementation, proprietary paths/assets or help text into Bondline.
