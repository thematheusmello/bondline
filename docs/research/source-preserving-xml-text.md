# Source-preserving text edits

Contract reviewed2026-11-17 on base8e030bf. This L0 prerequisite adds no imported-label,
font/render, model, command, native tool or complete-source authority.

Public [CDXML Style](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Style.htm) defines
`s` as one styled string, stored as character data; [Text](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Text.htm)
contains styled runs. Editing a leaf must retain its run attributes and surrounding
structure. Synchronizing dependent text with chemical properties remains later IO work.

The original Bondline API is one additional `Edit::SetText { element, value }` in the
existing checked immutable batch editor. The element must belong to this parsed document.
Every direct child must be text, or the element must be empty. Elements, comments and PIs
cause `InvalidTextContent` even for equal/empty requests. No markup is silently removed.

The supplied value is decoded text. Exact decoded equality is a byte-preserving no-op,
including original entities, CDATA segmentation, CRLF and empty-element spelling. A real
change replaces the complete inner span with escaped character data. Start/end tags,
attributes, prefixes, namespace declarations and unrelated source remain exact. Clearing
text retains paired tags. Expanding a self-closing element retains its raw QName and
pre-slash whitespace. No normalization, trimming or chemical interpretation is implicit.

The shared XML escape mapping is an original serialization policy, also escaping quotes,
apostrophes, TAB and LF. Illegal XML characters return `InvalidTextValue`; only the shared
helper's existing attribute-character error is remapped, leaving old attribute behavior
and all other errors unchanged. Existing byte/node/depth/attribute/edit limits and final
whole-document reparse apply. Closing-tag overhead is counted before escaping/allocating.

Duplicate SetText and same-container AppendChildren conflict in either order, including
no-ops. Attribute operations on distinct targets may coexist. An ancestor append plus a
descendant text edit is legal; existing descendant-close-before-ancestor-append ordering
must remain. Invalid later operations return no partial result or source mutation.

[XML1.0](https://www.w3.org/TR/2008/REC-xml-20081126/) §§2.2/2.4/2.7/2.11/4.1 underpin
legal character data, escaping, CDATA, line endings and references. Pinned roxmltree0.21.2
merges adjacent text segments without expanding the first segment's source range; replacing
that text-node range can leave old text behind. Use checked element start/end metadata,
not a body search that can mistake text inside CDATA for an end tag. Compare decoded text
without allocating another full-size copy.

Independent tests must cover literal bytes and separately qualified Expat consumption;
Unicode/escaping, CDATA/entities/CRLF no-ops and changes, empty/clear forms, qualified and
long QNames, mixed-content refusal, ownership, attributes, duplicate/content conflicts,
both caller orders at shared offsets, resource bounds and atomicity. Extend parser-editor
fuzzing with strict success/error assertions; no arbitrary error counts as success.
Keep prior syntax/resource/namespace/unknown-source tests and complete IO authority intact.
