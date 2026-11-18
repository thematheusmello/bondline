# Independent canonical CDX file-end declarations

Base29be796; original external fixture, new raw/RDKit oracle and test declarations
only. No parser/writer/runtime/dependency/oldfixture/oldoracle/pin change is made.
The exact externally generated positioned ethane is258bytes/SHA256
`856482269dbd85640b87c11f608104dba72ebd308a686f1333a893965ed560ba`.
Its original graph/generators/licenses/pins are preserved in the new fixture
provenance folder. No producer installation or generation is performed here.

The first1,667-byte test11cf281b precedes implementation. Runtime48668 compiles,
then fails0/1 with the actual existing `TrailingData`; it is not a missing-API
compiler failure. Both final words are intact. After successful parsing, this
unchanged first test requires exact source passthrough and the NEW full raw-table
and pinned RDKit consumer on the actual returned bytes. The original test prefix
remains byte-exact when the expanded cases are appended.

The expanded eight cases declare: implicit full24-event property/owner/payload/
offset conservation; explicit root at28..34 with its own end and FileEnd; new
zero-header unknown-body acceptance and unchanged noncanonical tails/legacy
successes; exactEOF and malformed/truncated file-end/header negatives; opaque
payload edits crossing the short/extended boundary with exact literal splices,
source retention/foreign-handle rejection and shifted terminal offsets; FileEnd
record-budget accounting; explicit-versus-implied root depth and64MiB bounds.
They do not bind the proposed internal semantic helper or change Writer defaults.

Contract: successful legacy parses win FIRST. Only legacy `TrailingData` can
select a28-byte prefix12+16zero-reserved profile with one DocumentEnd, exactly
one separate FileEnd and exactEOF. First root tag8000 at28 selects encoded-root
semantics; otherwise the document is implied. The existing22-byte HeaderDocument
branch retains priority. The new FileEnd consumes a record; only encoded objects
consume depth. This is a declared canonical-envelope policy, not proof of producer
intent for arbitrary zero words. The complete256-byte legacy prefix of the
external fixture remains valid; truncation tests explicitly retain that ambiguity.

Expanded compilation before product code exits101 with six missing-framing E0599
and one E0282 at the affected new-framing tuple loop. Whether that type error is
a cascade awaits actual production variants; no silent correction is counted.
No eight-case behavior GREEN is claimed. Original log/source archives are kept.

The new oracle independently uses literal original graph and complete record/
payload data, not the old original-scene verifier or Bondline's decoder. Its
default CLI reads actualCDX on stdin and emits exact compactJSON/LF; optional
`describe` emits bounded raw record/payload TSV. Eighteen external controls pass:
original raw+RDKit positive, framing/header/length negatives, changed metadata
and order that still pass chemistry but fail raw preservation, altered coordinate
detected independently by both consumers, untouched external ethene rejected by
the literal ethane chemistry checks, source-derived explicit and unknown positives,
and actualCLI/LF checks. Synthetic source-derived controls are distinguished from
actual producer outputs. The strict file-end oracle may refuse a valid legacy
prefix; it is not a general CDX reader or chemistry engine.

No source is normalized or partially returned. Record/depth/input limits and
unknown data stay explicit; old UNKNOWN70B has nonzero reserved bytes and its
old appended-zero negatives stay exact. Root owns implementation, receiving/full
gates, fuzz/application/native evidence and Git. No native/Actions/corpus admission
or broad compatibility claim follows from these declarations. Exact source/log/
consumer/control/protected hashes are in ignored `target/cdx-file-end/`.
