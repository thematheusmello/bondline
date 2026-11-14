# Explicit SMILES record inspection

2026-11-14. Initial source17e2a4a. This is an original automation command over complete
accepted SMILES files, not a reference-product record-browser or native editing claim.
The existing codec, parser, writer, layout, source/corpus fixtures, oracles, dependencies,
assets and approved golden remain unchanged. No GUI/input/activation or Actions execution.

## Contract and source authority

`smiles.record.inspect` is the25th registry name, read_only:true, with only two required
closed parameters: expected_revision:u64 and positive record_id:u32. Preparation refuses
stale revisions, wrong content, zero/absent ordinals and committed close. It captures the
whole Arc<StoredContent> plus generation; it neither reads the original path again nor
projects the first record. Work calls the existing selected-only read_record off-lock.
No renderer, carbon-path normalizer, writer or chemistry/perception policy is involved.

The optional smiles_record response has12 fields: input_revision, session_generation,
record_id, line_number, full_range, notation_range, separator_range, application_data_range,
line_ending_range, atom_count, bond_count and exact notation. All ranges are whole-file
byte offsets serialized as {start,end}. Null separator/data spans differ from Some(empty)
spans. Application data remains opaque in the retained file; it is never copied, decoded
or disclosed by this query. Counts are literal parser counts, not chemistry calculations.
Only an explicitly requested response includes notation. Automatic state/Inspect omits
smiles_record, and Debug omits notation. There is no persistent selection or receipt cache.

The serialized smiles_record object has an original128KiB budget, measured with the
existing bounded serde_json writer helper. This is not a bound on parent CommandResult,
MCP text/structured mirrors or JSON-RPC envelopes. The unchanged65536-byte notation cap
and actual current ASCII language fit below it; no input guard is broadened. The real
JSON measurement remains a guard against future serialization growth. Cancellation is
checked before/after the bounded parse, after JSON measurement, and again at finish.

The result is the same historical snapshot model as sdf.record.inspect: captured source,
file/history/revision and stamped record metadata, with current close/UI fields refreshed
by the existing result machinery. A newer Open is never overwritten. Clients must match
input_revision and session_generation before using an ordinal in another operation.
The complete source owner survives deferred work, then releases because the completed
reply is self-contained. No extra whole-file lease is held during reply delivery. No save
reservation, native file binding/ACK, mutation or repaint is added; existing worker/reply/
close barriers apply. There is no native UI control for the command.

The pinned [OpenSMILES v1.0 publisher source](https://github.com/opensmiles/OpenSMILES/blob/9240a2887158699503605bc838e1de9a9f7883f4/opensmiles.asciidoc)
separates notation from application data and file framing. Those codec rules and its
qualified final-EOF/ignored-prefix policies stay unchanged. Ordinals, API fields, JSON
budget, privacy and historical-result rules above are original Bondline policy. The
primary raw source is86816B/SHA256
00f0423da612de913770133aa13e456d0cebc562dd50166534a8a7f9a834bd21.
The official HTML timed out once during Root refresh; the pinned raw GitHub source opened.
Rendered web line numbers differ from raw blank-line positions; use section names/raw pin.

## Tests and observed RED/GREEN

An original49-byte3-record source contains mixed endings, ignored non-UTF-8 lines and
opaque NUL/non-UTF-8 data. The selected second record has exact [13CH3][OH] spelling,
literal global21..43 whole-record/21..32 notation/32..33 separator/33..42 data/42..43 LF
ranges, two atoms and one bond. Full state/file equality, no appdata or Debug disclosure,
closed schema, zero/missing/stale/wrong-kind refusal, empty sources, historical work after
input deletion/new Open, cancellation before work/after completion and lease/close/native
intent guards are checked. A private Weak-owner test proves complete-source retention
during work and release before finish. These are not independent chemistry calculations;
additional external raw/graph and actual wire verification are received separately.

All Root Cargo runs use Rust1.94.1, incremental0, the own absolute Root target/default
debug profile and the existing pinned interpreter through BONDLINE_PYTHON.

- First runtimeRED53339 exits101: one test fails UnknownCommand(smiles.record.inspect)
  before production code. Log target/smiles-record-command-red.log,923B/SHA256
  74cc54282103e70c1ead58d3581c524ea379cc73c07794bddee3727a5bfcec24.
- Expanded runtimeRED56458 exits101:0passed/5failed, all because the command/schema is
  absent. Before implementation the historical-path assertion was corrected to compare
  the documented canonical input path, following the existing shared file helper.
- First focusedGREEN71291 exits0:5passed/0failed/0ignored; no production/expectation
  correction after the actual first compile. target/smiles-record-command-focused.log.
- Two additional lifecycle/guard regressions follow that initial implementation; they
  are not represented as separate newly observed test-first failures.
- First full package3253 exits101 on a stale24-name discovery fixture. A bulk update had
  stopped at a final array entry without a comma, leaving part of the additive inventory
  update unapplied. This is an integration-fixture failure, not a new production defect.
  Only the new name/count was added; every prior parameter assertion remains.
- Final commands package59687 exits0:174passed/0failed/0ignored across25 result lines,
  including the six public cases and one private owner regression. Log
  target/smiles-record-commands-tests-final.log. All-target workspace warnings-denied
  clippy70224 and whole-workspace fmt exit0, logs -core-clippy.log and -core-fmt.log.

The195 lower-layer/oracle/asset/dependency identities frozen before implementation remain
protected. This core checkpoint does not yet claim aggregate receiving, independent new
oracle tests, actual CLI/MCP wire acceptance, native/Windows/Linux or hosted execution.
Their separate receiving evidence follows. M0/M1 and the full goal remain active.
