# Editing a caption run on a mixed CDX page

Reviewed 2026-11-20. A Text object owns its styled string. A direct Page Text
can have the same raw ID as a Text inside a Node in another container.
Resolve the selected caption against the current direct Page children, in the
same identity order as display. Its position in paint order and an integer
ID alone are insufficient to choose a property. [Text object](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Text.htm),
[Text property](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Text.htm),
[binary IDs](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDX.htm),
[identity contract](cdx-caption-creation.md).

Use the existing source-preserving Text property writer. Preserve the run
count, declared font, face, size and color words, and every unaffected source
byte. Recompute offsets from the resulting run bodies under the established
encoding rule. Reopening the candidate must reproduce the complete expected
mixed drawing, including its chemical fragments and physical paper. A no-op
keeps the original source. Undo restores the exact prior property encoding.
[CDXString](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXString.htm),
[run encoding](cdx-caption-edits.md), [mixed rendering](cdx-mixed-caption.md).

The original witnesses place the blue caption above the red atom label, and
put the target after a different caption in paint order. The selected direct
caption has raw ID 73 and persistent live ID 19. Two Node labels use raw ID 73
in their own containers. The original UTF-8 bodies and two distinct styles
test longer and shorter prefixes, shifts of the second run's start, editing
the final Unicode body, and byte-exact history. [Fixtures and expected byte ranges](../../crates/io/tests/fixtures/cdx-mixed-caption-edit/SOURCES.md).
