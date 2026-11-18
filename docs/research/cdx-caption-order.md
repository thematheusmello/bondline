# Ordering direct-Page CDX captions

Reviewed 2026-11-18. Extend the [single-caption profile](cdx-caption-preview.md)
to a Poster Page with multiple direct Text objects. Each Text keeps the earlier
explicit baseline, single supported style run and paper/resource requirements.
The whole-source preview remains read-only, with exact CDX Save and reopen.

The [Page](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Page.htm) and
[Text](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Text.htm) specifications
allow direct sibling Text objects. [ZOrder](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/ZOrder.htm)
is tag 0x000A, a signed INT16. Paint the lower explicit value first. Decode the
canonical two-byte little-endian form and admit it only on its own Text object.
The general [numeric description](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXNumeric.htm)
also describes historical width tolerance. Other widths stay outside this
profile until independently resolved.

The ZOrder page treats omission as zero, leaves order within the zero group
unspecified and describes other objects above that group. This agrees with
positive Z values over one omitted value. It leaves negative explicit values
beside omission ambiguous. Accept any distinct explicit values, or one
omission with all other values positive. Repeated effective values and
omission beside a negative value remain outside the profile. The public page
also permits omission when overlap is impossible. Bondline still needs a
non-overlap proof across supported cameras for that source shape.

The [binary format](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDX.htm)
defines scoped 32-bit IDs and allows ID zero for an unreferenced object. Reject
repeated nonzero IDs among caption siblings. Allocate model Text IDs in ascending
source ID order, using source encounter for anonymous zero-ID captions. The
unchanged single caption keeps ID 1. Paint in the separate Z order so reordering
records of named captions does not change model identity or pixels. The native
model IDs belong to the current snapshot. The source retains the original IDs
and all ordering bytes.

Verify independently generated reverse-order and overlapping examples on both
CPU and Metal. Retain exact legacy error codes and locations for unsupported
width, duplicate field or sibling ID, wrong owner, unresolved order and a bad
peer caption. Test actual Unicode Save and reopen and the existing source/edit
and cross-format boundaries. Leave the production registry, native
tool catalogue and previous fixtures intact.
