# Native caption editing

Reviewed 2026-11-19. The public G25 copy has 439 PDF pages and
17,721,447 bytes. Its unversioned cover and May 22, 2025 creation
metadata accompany the version 25 filename. The downloaded and cached
PDFs both have SHA-256
`3f1ebc429df07efdd47b6218a148dc346a215043f60bf89ee9a1cb90629bd5b3`.
The 728,816-byte Poppler layout extraction has SHA-256
`6af3ecb27a0b6c19f2b64005ad95582ba7795876c54a42bf210232f75e84c43f`.
Printed page p is PDF page p + 1, counting the cover as PDF page 1.
[Public guide](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf).

## Public behavior

- G25 printed p92 describes selecting the Text tool, clicking in
  an existing caption's text field and typing. Printed p74 lists
  `t` for the Text tool. Printed pp48 and 93 describe selecting
  part of existing text for formatting. [Editing](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf#page=93),
  [Shortcut](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf#page=75),
  [Formatting](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf#page=49),
  [Character styling](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf#page=94).
- Printed p38 gives Enter or Return as the default caption newline.
  A caption preference can require Alt+Enter for that newline. Clicking
  outside or choosing another tool closes the text box. Printed p423
  also uses Escape to close a newly created caption in a tutorial.
  Commit and discard semantics for edits to existing captions remain
  open. [Newlines and closing](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf#page=39),
  [Creation tutorial](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf#page=424).
- Printed p89 documents moving an object to the front and sending
  a bond behind other objects. The CDX specification defines signed Z order from
  back to front. Caption click precedence in an overlap remains open.
  [Drawing order](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf#page=90),
  [CDX ZOrder](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/ZOrder.htm).
- The January 2026 support notes scope the Alt single-click change
  in 25.5.0.5789 to opening an atom-label field, and specify Windows
  Alt and macOS Option for repeating an atom label. The public caption
  workflow above uses a Text-tool click. [Atom label entry](https://support.revvitysignals.com/hc/en-us/articles/44882776753300-ChemDraw-Double-clicking-on-an-Atom-adds-a-new-bond-instead-of-the-text-box-to-enter-the-label),
  [Atom label repetition](https://support.revvitysignals.com/hc/en-us/articles/44882820468884-ChemDraw-Unable-to-repeat-the-atom-labels-by-Double-clicking).
- The September 28, 2026 Desktop 26.1 announcement describes selecting
  attached-data labels and the revised Paste Special entries across
  Prime, Professional and Signals. The vendor chart lists the Text tool
  for Windows, Mac and Web. Exact Desktop 26.1 caption modifiers,
  confirmation, overlap and platform bindings remain open.
  [Desktop release](https://support.revvitysignals.com/hc/en-us/articles/54009539308052-Release-Announcement-ChemDraw-Desktop-26-1),
  [Tool availability](https://assets.revvitysignals.com/asset/0b9656e8-339d-47ab-9834-6c08beb6abee/Full-Dot-Chart-ChemDraw-Suite-of-Products.pdf#page=2).

## Bondline interaction

Select the generic Text control, then use an unmodified primary press
and release on the same painted caption run. Run the shared source
and whole-page preflight, then resolve the exact painted pixel in
reverse paint order within the current clip. Identify the target by its stable TextId and
zero-based run index. Direct targeting requires painted ink, so blank
and empty runs have no pointer hit.

Open a one-run form with a bounded single-line UTF-8 draft. Apply
confirms through `text.set_run`, retaining the ordinary history and
source-preserving same-format Save path. Cancel and Escape discard the
draft. Return leaves it open and uncommitted. The existing validation
rejects line controls. Bind the gesture and form to weak host, view,
source, file and modal ownership, and validate current ownership at
release and Apply. A stale form keeps its draft copyable with Apply
disabled. These choices define the original Bondline workflow.

## Independent pointer witnesses

The existing four-caption source has SHA-256
`4a31a7bb94bbefb504f075bbfddd2042847b4511150cf5406613f66e38c971c7`.
Its encounter order maps to Text IDs 4, 1, 3, 2, and its paint order is
1, 3, 4, 2. The target with three existing runs is the second anonymous
source object. FontTools PointInsidePen confirms the following interior
points in the applicable outlines, with the center and eight neighboring
points at 0.25 pt offsets checked.
[Fixture and font provenance](../../crates/io/tests/fixtures/cdx-caption-edit/SOURCES.md).

| Page point in pt | Expected target |
| --- | --- |
| (100.5, 96.5) | Text 4, run 0. Identical H outlines overlap, with green Text 4 at depth 4 above black Text 3 at depth −5. |
| (125.75, 218.5) | Text 2, run 0. The blue A has ink at this point. |
| (151.875, 216.5) | No hit. The middle space run advances from x = 149.90625 to 153.8671875 pt. |
| (188, 216.5) | Text 2, run 2. The orange Ω has ink at this point in the original source. |
| (217, 216.5) | Text 2, run 2 after the earlier AV run grows to AVAV. |

Cancel the draft from the overlap after confirming its owner. Apply
AVAV at the blue A and compare the saved bytes with the existing
`multi_after.cdx` witness. Probe the shifted orange run on the
updated canvas.

File and shared command behavior has independent evidence. Native tool
selection, draft input, post-Apply repaint, history and Save/reopen
remain pending for the new form.
[Command and file verification](../verification-cdx-caption-edits.md).
