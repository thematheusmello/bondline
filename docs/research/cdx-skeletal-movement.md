# Moving skeletal atoms in CDX

Bondline reuses its Marquee selection, drag, arrow-key nudge, revision and selection-epoch commands. The published guide describes selection and movement in the drawing interface. The existing selection contract records the reviewed version, page references and key conventions. [Selection and translation](selection-translation-plan.md)

## Coordinate ownership

The SDK assigns property 0x0200 to the owning object. CDX represents each two-dimensional position as two signed 32-bit integer words, vertical then horizontal, in units of 1/65536 point. The axes increase down and right. A Node and a caption Text can each own a position. This workflow starts only on the fully certified skeletal source, where every chemical Node is a direct Fragment child with exactly one explicit position. Other coordinate-bearing objects, dependent child labels and fragments with unmodelled geometry cannot enter it. [2DPosition](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/2DPosition.htm) · [Coordinate data types](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXCoordinates.htm) · [Skeletal source qualification](cdx-skeletal.md)

The scoped decoder assigns live atom IDs by Fragment and direct Node encounter order. A move matches each current binary position against its atom's current model point. Raw object IDs may repeat under different Fragments. Each changed position replaces only its two existing signed words. Property tags, original short or extended length headers, resource tables, chemical fields, implied flags and all other bytes retain their values.

## Bondline's movement policy

Round each requested displacement component once to the nearest grid unit, with ties to even. Applying that same integer vector to each selected atom preserves their relative positions. Conversion checks the finite displacement and each resulting signed coordinate before publication. A valid zero or sub-grid move retains the current snapshot and redo history. Unknown IDs, stale selection, detached sources and an unqualified final drawing fail before publication. Changes that require visible carbon glyphs or exceed the page cannot enter the hidden-carbon profile.

Preview and commit use the same stored candidate. The preview retains its source-derived hydrogen counts for rendering. The editable model keeps every source omission. One successful drag or nudge commits one history operation. Undo restores the previous source bytes and Redo restores the move. Saving writes the currently certified CDX. A new selection grant enables the Marquee operation alone. Other chemistry, caption, page controls and conversion retain their previous source checks. [Source snapshots and history](dependent-label-translation.md)

## Independent reproduction

Run the Python generator in the fixture directory to reproduce every expected CDX from an independently decoded source. The aliased input has two Fragments with identical child IDs and long-form length encoding on both central position records. The two-fragment move changes one atom in each Fragment. The other cases preserve missing H properties and dormant implied flags through a fractional center-atom movement. [Fixtures](../../crates/io/tests/fixtures/cdx-skeletal-move/SOURCES.md)
