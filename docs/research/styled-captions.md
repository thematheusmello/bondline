# Styled caption file workflow

Contract2026-11-17, baseb34e786. The full M0–M5 target is unchanged. Implement a real
foreign CDXML caption-only Open→render→edit run→history→Save/reopen workflow first;
chemical dependent labels and the full Text tool remain required subsequent work.

## Public evidence and boundaries

The SDK [Text object](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Text.htm) defines
free text separately from chemical Nodes, with at least one ordered
[Style](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Style.htm) child. Style font/face/color
are UINT16 and size is an XML point coordinate. Text
[position](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/2DPosition.htm) is the
left baseline for a left-aligned caption. Explicit
[Justification](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Justification.htm)
is subordinate to CaptionJustification if that is present. The first supported profile
requires explicit Left and has no competing caption default. No missing layout default
or chemical meaning is inferred.

[Font](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Font.htm) requires logical name/id,
not charset. First admitted captions use the exact already-bundled Apache Roboto Regular
family, explicit run font/point size/face0/colour and no charset override. No Arial alias,
synthetic face, system font, replacement glyph or new font asset is introduced. Logical
resource decoding remains separate from font availability. Unknown fonts/faces/defaults/
extra properties remain retained and unsupported. The SDK's LabelColor/CaptionColor
pages are future-only; actual run color uses ForegroundColor. Public CDX string size/
offset examples are inconsistent, so this work adds no binary text conversion.

The profile is a caption-only page with explicit Poster dimensions and a white default
palette background (absent table or an explicitly white entry2), ordinary resource tables
and explicit Left-baseline text. BackgroundColor/bgcolor is future-only in the archived
SDK and is not listed for Document: an initial root-bgcolor fixture assumption was
corrected against that primary page before source-profile production. No unsupported
background property is treated as a working reference setting.
Every visible/source object must be covered; mixed chemical/unknown objects are not
admitted by the caption capability. Producer strings grant no authority. Unknown source
still saves unchanged. This is one complete format workflow, not full foreign parity.

## Locked data and parser interfaces

Model adds checked owned `TextId`, `TextStyle`, `TextRun`, `StyledText`:

- `TextId::new(u32)->Option<Self>`, `get()->u32`.
- `TextStyle::new(String,f64,u16,[f64;3])->Result<Self,TextError>`; getters
  `font_family()->&str`, `size_pt()->f64`, `face()->u16`, `rgb()->[f64;3]`.
  Nonempty font name is bounded256 UTF-8 bytes; size is finite above zero through100000;
  RGB channels are finite0..1. Raw face bits remain data, not renderer support.
- `TextRun::new(String,TextStyle)->Result<Self,TextError>`; `text()->&str`,
  `style()->&TextStyle`, `with_text(String)->Result<Self,TextError>` preserving style.
- `StyledText::new(TextId,Point2,Vec<TextRun>)->Result<Self,TextError>`; getters
  `id`, `position`, `runs`; `with_run_text(usize,String)` and `with_position(Point2)`
  return checked copies. One through4096 runs and at most1MiB total text; strings obey
  XML1.0 character legality. Model may retain multiline/unsupported faces; renderer and
  source capability make support explicit. Validated closed JSON follows these field
  names, omitting no present text/style values.
- Page adds `texts()->&[Arc<StyledText>]`, omitted from old JSON when empty.
  Document adds `append_texts(PageId,Vec<StyledText>)` and
  `replace_text(PageId,StyledText)`, checked IDs/limits with atomic errors. History
  accounting counts unique text Arcs, Vec/String capacities and style strings.
  Document-wide text count and run count are at most100000 each; aggregate UTF-8 text
  is at most64MiB, preventing a cross-product of per-object allowances.

Standalone cdxml adds `Document::styled_text(Node)->Result<TextObject,TextError>`.
The borrowed view owns no model/OS/render dependency: ordinary owned t, one or more
ordinary direct s children, ordered decoded text and optional exact font/face/color u16
and finite positive point size. Missing values remain None, never defaults. Getter names
on `TextObject`: `element`, `runs`; on `StyleRun`: `element`, `text`, `font`, `size_pt`,
`face`, `color`. Unknown attributes remain in nodes; caller completeness checks them.
Reject foreign nodes, unexpected/mixed content and malformed scalars. Existing syntax
bounds apply; caller model/render budgets are additional. SetText edits only s leaves.

## Rendering and authoring integration

Render supports fully resolved single-line regular Roboto captions, including supported
Unicode, spaces, differing run sizes/colours and baseline kerning within each run. A run
boundary intentionally resets kerning. Missing glyphs, unsupported faces/font families/
line controls, unreadable scale, off-paper ink and work/primitive limits refuse. Empty
text/spaces advance without inventing ink. Fixed Unicode glyphs are not a substitute for
the still-missing CDXML script/formula faces. Per-run logical and physical readability
must be checked; one large run cannot hide another small run. A whole Page admits at
most4096 caption runs,4096 glyphs and1048576 aggregate glyph bounding pixels before raster work,
plus the existing4096 scene primitives. Existing label getter and
legacy chemical pixels stay exact. Shared CPU/GPU primitives render the same caption ink.

The shared render path projects Page texts; owner display/current-view/capture must use
current objects. No partial first-text/first-fragment rendering, silent clipping, glyph
replacement or changed chemical-label defaults. New font/ink expectations use independent
FontTools/Pillow and Expat/Decimal, not only a writer/reader round trip.

IO owns a dedicated checked caption capability, not global chemical `is_fully_mapped`.
It retains complete source/baseline, maps page text IDs and run indices, and permits only
same-object/same-style run text edits (and independently proven placement edits later).
Root/Page/style/resource drift or added chemical objects revokes this capability.
Source writing uses existing immutable SetText batches; unedited bytes/styles/IDs and
unrelated lexemes stay exact. Whole MOL/SMILES/CDX loss guards must reject caption data.
Source-free/Canonical model encoding must emit all owned texts and deterministic logical
font/palette resources, preserving fixed white/black defaults for existing chemical
content. Legacy documents without texts retain their exact old encoding. As in the
existing native graph profile, source object IDs and application ordinals are distinct;
this increment does not promise stable generated identities across save/reopen.

Append one command `text.set_run` with required closed `expected_revision`, `page_id`,
`text_id`, `run_index` (zero-based), `text` fields. Validate source/revision/IDs/domain,
the final renderable candidate and its source-preserving Save representation before one
normal history commit. No-op Arc/source/redo sharing, Save leases, cancellation/stale
results and all old command schemas/order remain. CLI/MCP/loopback share this entry.

The current native app must display accepted captions through existing File Open and
capture paths. This increment does not activate the full Text tool or invent an inline
rich editor. New native interaction requires actual OS evidence; App tests alone stay
distinct. Chemical dependent label edits, insertion/deletion/reordering, layout/rotation/
wrapping, font/style UI, mixed foreign diagrams and broader caption faces remain backlog.

Use independent RED→GREEN for model/parser/render/registry; actual Unicode files and raw
Expat/Decimal before font/chemical checks; CPU/GPU + per-run/Pillow comparisons, no-op/
history/source/error/adapters/App guards; bounded parser fuzz and full local gates.
Actions remains OFF; no reference binary/assets/fonts/help text are inspected or copied.
