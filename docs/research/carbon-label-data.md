# Non-terminal carbon-label data prerequisite

2026-11-16, based on b1d843f. The preceding inset renderer is committed70d38a3.
This is the persistent standard-field step toward conventional skeletal rings, then
weighted benzene/cyclopentadiene tools. It is not full skeletal display or M0–M5 parity.

## Public basis and scope decision

The public [SDK property table](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/TableOfProperties.htm)
defines Atom_ShowNonTerminalCarbonLabels0x0443, XML ShowNonTerminalCarbonLabels,
CDXBooleanImplied, for carbon atoms with more than one bond. This is a display choice,
not NumHydrogens and not the query restriction ImplicitHydrogens0x0424.
The [official DTD](https://static.chemistry.revvitycloud.com/cdxml/CDXML.dtd) declares
the flag on both CDXML and n as `(yes|no) "no"`. Retrieved2026-11-16:28821B SHA256
5311978e514ffe154108540c3634c314dc66031a4f3c877a681fc6dca8e128c2.

The same sources also define terminal-carbon and implicit-H display flags; those remain
separate requirements. The older Node page omits these rows, and per-property mirror
URLs return archived error wrappers. They supply no additional semantics. LabelDisplay
is alignment, not hiding. Empty Text has no established hide-label guarantee; hidden
Text Visible=no is unnecessary when a direct standard flag exists. No custom attribute,
proprietary binary, sample/template, artwork or font is introduced.

The disposable source note11204B has SHA256
e1466d744ae8e042b9963ac9e7d82eadf2f71694c098242e75fc146589605f0d;
its exact receipt pins public inputs and access errors. Root/node inheritance precedence,
explicit child-Text interaction and actual vendor26.1 behavior remain unverified. Do not
guess them. Existing unknown/root styles and explicit Text stay source-backed/incomplete.
No source completeness override is authorized.

## Locked data/format increment

Add `Atom.show_non_terminal_carbon_labels: Option<bool>`, default None, omitted when
serializing None. Some(false)/Some(true) preserve explicit no/yes. Model JSON uses actual
booleans; invalid strings/numbers/objects remain closed errors. Atom::new initializes
None. This fixed-size datum owns no heap; actual-size accounting and old quotas stay.
Chemical element, positions, H/charge/isotope/radical, IDs and other atoms/bonds remain
unchanged. Copy/replacement/layout/history must preserve the field.

None is the old Bondline literal-label compatibility profile, not a claim that absent
and no have different SDK semantics. Legacy absent/default JSON and native-v0 CDXML must
remain byte-exact/editable. A future CDX writer cannot encode XML presence distinctions
through a CDXBooleanImplied flag alone; no new CDX projection/writer is implemented here.

IO reads only the exact node-level values yes/no; absence maps None, other spellings
reject typed mapping while standalone syntax retains all original bytes. New native
encoding emits the attribute only for Some, after the existing attributes. Known-node
presence alone never authorizes complete regeneration of unknown styles/Text/root data;
the existing exact-byte native re-encoding proof remains authoritative.

Both MOL model writers refuse every specified flag before publication, including yes
and dormant metadata on other elements or terminal atoms. The manual MOL authored-field
comparison includes the datum. Retained-SMILES replacement still permits only existing
H/order changes, so a flag change refuses atomically; detached drift remains guarded by
complete-model equality. Existing explicit graph-only SMILES export intentionally omits
depiction/style, distinct from full-document Save. SDF/RXN raw wrappers remain unchanged.

Until the next renderer increment defines skeletal eligibility/mandatory labels, any
specified flag refuses shared scene construction after cancellation. Do not silently
draw the old profile or erase explicit H. Missing flags keep every successful old scene,
font/glyph/mask/stroke/golden and scale/bounds behavior exact.

## Independent test-first acceptance

- Existing Atom JSON and ElementalGraph APIs give real runtime REDs for explicit false,
  separate from a missing Rust field/API qualification. Freeze literal non-dense IDs,
  coordinates/H and expected presence before product edits; no own-code-only round trip.
- Model None/false/true, closed invalid values and unchanged Fragment validation/data;
  node XML exact yes/no/absence, malformed values and raw unknown retention. Native
  writer/proof/default-v0 bytes plus independent Expat/Decimal actual-file properties.
- Both actual MOL writers and atomic file adapters reject specified data without partial
  output/destination/state/Arc changes. Retained-SMILES replacement/drift cannot lose it.
  Raw CDX implied flag bytes and SDF/RXN controls retain their separate source-only scope.
- Shared literal/resolved-H/PageView render entries refuse specified data, cancel first,
  and preserve all old absent-profile pixels/data. Actual registry Unicode Save/Open/
  label/order/history must preserve the datum; no new command is claimed in this step.
- Relevant local fmt/clippy/workspace/layer/license/corpus gates and bounded changed-mapper
  fuzz. Actions stays off. Preserve any natural MCP failure without retry/policy changes.

## Following complete skeletal workflow

Define the source-backed presentation policy and its command/adapter/UI seams before
implementation: retain literal H and compare C4/actual weighted topology, preserve
mandatory labels for isolated C/C2, identical collinear centers/cumulenes and unsupported
heteroatom/charge/isotope/radical cases. Unknown/resolved H and same-count atom.set_label
need explicit decisions, not accidental inference/no-ops. Remove suppressed glyphs and
masks together while retaining atom hits; all visible-ink/join/gap/paper/readability
guards still apply. Then wire persisted commands and native previews, followed by real
weighted unsaturated ring candidates/phase. Keep the full parity target visible.
