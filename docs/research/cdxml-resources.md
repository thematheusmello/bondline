# CDXML logical font and colour resources

Reviewed 2026-11-17 on base `70e6f27`. This is a standalone format prerequisite for
foreign-file appearance, not a complete imported drawing or font-rendering profile.

The public [FontTable](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/FontTable.htm),
[Font](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Font.htm) and
[font-table datatype](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXFontTable.htm)
define document-level logical fonts. Entries require name and unique UINT16 id; IDs
need not be sequential. No nonzero rule or global collision with chemical object IDs
is established. Name presence does not establish a nonempty name or an installed font.
Optional charset remains decoded source text; binary codes and XML charset names must
not be conflated. This change installs, bundles and substitutes no fonts.

The public [ColorTable](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/ColorTable.htm)
and [Color](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Color.htm) define an ordered
palette. Entries require finite r/g/b in 0..=1. Indices0/1 always mean black/white and
are not stored. Entry0 defines index2, entry1 defines3, then indices follow source
order. Both tables require at least one entry. Only when the colour table is absent
do2/3 default to white/black; a present one-entry table leaves3 unresolved. RGB stays
f64 without channel quantization, alpha or colour-space conversion.

`Document::resources()` returns a checked borrowed `Resources` view, with optional table
nodes, font lookup by ID, ascending-ID font iteration, source-order colours and checked
colour lookup. Entries expose original nodes for the existing owned batch editor.
Unknown attributes/table IDs remain accessible through those nodes and exact source;
this projection does not validate every attribute or every reference in the document.
Font lookup is O(log n); palette lookup is O(1). Existing input/node/attribute limits
bound allocations. No additional dependency or workspace layer is introduced.

The [official DTD](https://static.chemistry.revvitycloud.com/cdxml/CDXML.dtd) permits at
most one table of each kind directly under the document. Its fixed sibling order differs
from [SDK General](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/General.htm), which says
same-level order does not matter. This reader accepts either root-table order without
claiming DTD validation. Duplicate, nested and nonempty-namespace table lookalikes fail
conservatively instead of selecting a winner or activating absence defaults. Unsupported
entry objects or meaningful text fail. XML whitespace, comments, processing instructions
and unknown attributes remain source-backed, not a complete semantic certificate.

[Namespaces1.0 §6.2](https://www.w3.org/TR/2009/REC-xml-names-20091208/#defaulting)
distinguishes qualified attributes from unqualified attributes and permits empty default
namespace resets. The codec explicitly matches ordinary attributes, because pinned
roxmltree's string-name lookup also accepts namespaced attributes by local name. Empty
namespace resets on tables/entries stay unnamespaced. Existing syntax admission, including
its separate root empty-namespace restriction, is unchanged. XML whitespace around
numbers is accepted; font IDs reuse the exact integer decoder following
[IntroCDXML](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDXML.htm)'s equivalent
integral-decimal spellings. Invalid resources are typed errors, leaving syntax-only
passthrough intact.

Independent expectations use original documents, literal bytes, published domains/defaults
and a separately qualified Expat/Decimal consumer. Existing owned edits demonstrate font
name/RGB changes without disturbing unrelated bytes. Parser fuzzing adds arbitrary resource
extraction and fixed valid tables with arbitrary escaped names, IDs and RGB values. See
[receiving](../verification-cdxml-resources.md).

No IO profile, model style, command, native tool or grey catalogue binding is expanded.
Full foreign authoring still needs effective styles/defaults, usable font metrics, colours,
generated/dependent Text, source-preserving text edits and actual edit/history/Save/reopen/
render acceptance. Successful table lookup alone grants none of those capabilities.

Multiple-bond spacing stays open. Public [relative spacing](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/BondSpacing.htm)
and [absolute spacing](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/BondSpacingAbs.htm)
establish units and inheritance, but reviewed material does not decisively settle centreline
distance versus visible ink gap. The original3pt renderer policy is not a foreign default;
no guessed mapping is enabled.

Later [public corroboration](bond-spacing-evidence.md) supports a centreline convention
for plain bonds; its execution, length, Abs and inheritance limitations remain explicit.
