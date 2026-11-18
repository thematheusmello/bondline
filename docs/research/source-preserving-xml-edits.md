# Source-preserving batch XML edits

2026-11-17, base `ab7c312`. This is the first required format primitive for the complete
noncanonical CDXML workflow, not completion of that workflow or M1. The canonical native
profile, source authority, commands, rendering and all existing behavior stay unchanged
until the separate source-aware model writer is implemented and verified.

## Public basis and source boundary

[XML1.0 §3.1](https://www.w3.org/TR/2008/REC-xml-20081126/#sec-starttags) defines quoted
attributes, insignificant attribute order and empty versus paired element syntax.
§2.10/3.3.3 distinguish source whitespace from processor-normalized attribute values.
[CDXML introduction](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDXML.htm) explicitly
allows whitespace and equivalent numeric spellings.
[General](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/General.htm) distinguishes same-level
object/property ordering from significant containment. These facts do not permit dropping
unknown semantics or reserializing a projection. No proprietary code/assets are used.
Namespaces1.0 [§6.1–6.2](https://www.w3.org/TR/2009/REC-xml-names-20091208/#defaulting)
defines default namespace scope and explicit empty declarations. Bondline's self-contained
fragment contract refuses destination-dependent expanded names rather than silently
changing an unqualified inserted element's namespace.

## Locked syntax API

Add `bondline_cdxml::Edit` and `Document::apply_edits(&[Edit]) -> Result<Vec<u8>, Error>`.
Edit borrows an actual roxmltree Node and value text; it has three alternatives:

- SetAttribute{element,name,value}: replace an existing ordinary value or insert a missing
  unnamespaced ASCII name using the existing Writer name/escaping rules.
- RemoveAttribute{element,name}: remove only the ordinary attribute token; missing is an
  exact no-op. Adjacent whitespace stays unchanged.
- AppendChildren{element,xml}: append self-contained XML content without declaration/DTD.
  Empty string is an exact no-op. Inherited-only namespace prefixes are not self-contained.
  With a nonempty destination default namespace, every unqualified element must be under
  an explicit local default declaration/reset; otherwise return InvalidFragment. Prefixed
  elements with their own bindings and explicit `xmlns=''` remain supported.

Each element must belong to this exact parsed Document. Namespace declarations cannot be
edited. Existing qualified attributes can change/remove; insertion retains the Writer's
closed name policy. An XML-decoded string-equal Set preserves its original source lexeme,
including character references; generic syntax never infers numeric equivalence.

New attributes use ` name="escaped"` immediately before the original `/>` or `>` delimiter,
retaining all old whitespace. Multiple insertions at that offset follow caller order.
Self-closing append removes only `/`, retains the original `>`, adds children and closes
the original qualified name. Paired parents retain both tags and existing child content.
Parent/child updates and attribute insertion plus self-closing append compose in one batch.

Validate all targets and conflicts before publishing output. Duplicate same-element/name
operations or duplicate append ownership return ConflictingEdits, including no-op entries.
MAX_EDITS is MAX_ATTRIBUTES+MAX_NODES (200000); excess returns EditLimit before duplicate
checks. Invalid self-contained child XML/declaration/DTD/parent escape returns InvalidFragment.
Existing character/name/input/output/node/attribute/depth bounds remain. Final resource
errors match direct Document::parse, not a newly invented parser-limit conversion.

Apply a checked, nonoverlapping byte plan and revalidate the final document once, retaining
every unrelated source byte and immutable input on success/error. No per-edit reparsing,
native I/O, dependency, unsafe code, chemical interpretation or edit authority is added.
Source tag metadata and ordinary attribute spans are indexed once per edited element.
The long-tag control reports elapsed time without a hardware-dependent pass threshold.

## Full workflow receiving continues

The first actual existing-CLI counterexample is an original464B document with sparse IDs,
mixed quotes, numeric lexemes and a bond before its atoms. Open succeeds; bond.set_order
to Double refuses unsupported_content and leaves all input bytes unchanged. The intended
source-aware writer must change only Order/H values, then support current typed native
update/append commands, source-aware saving, exact Undo/Redo and native editable reopening.
Opaque labels/styles/unknown semantic content must remain guarded and preserved.

Original scoped XML↔baseline model bindings can be reconstructed from immutable source
and the existing import traversal. Generated IDs need an explicit contract: deterministic
per-snapshot IDs preserve original IDs but can renumber generated-only objects on later
edits across fragments. Stronger stability requires state and is not inferred. Whole-MOL
loss permission must stay separate from a new source-preserving CDXML edit capability.
Independent tests and these remaining policy choices precede that producer; a batch syntax
pass cannot be advertised as imported-document editing or full format/application parity.
