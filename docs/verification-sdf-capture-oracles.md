# Whole-SDF registered capture acceptance

2026-11-14. Test-only work in supplied `bondline-mol-v3000`, initial base `3438b04`.
Root owns implementation, source integration, Git and global records. This owner writes
only new command/adapter acceptance tests and this record. No native window/input/activation,
Actions, dependency, existing consumer, fixture, font or approved golden is changed.
Offscreen CPU/GPU execution is distinct from native UI acceptance.

## Independent authored inputs

The mixed source is original V2000 ethanol followed by V3000 methylamine. Neither
molecule declares VAL/HCOUNT. Duplicate SDF field occurrences, a DT-only field and literal
space/multiline data remain in the complete source. No future writer generates the fixture.

The first source positions are C(-2,5), C(0,5), O(0,3). At the explicit 24 pt/unit,
y-flipped, minimum-x/minimum-y72 policy, their literal drawing positions must be
(72,72), (120,72), (120,120). Neutral implicit H is independently expected3/2/1.
The source atom-bounds height is48 pt, so the next row's top is72+48+72=192 pt.

The N-right methylamine source has N17(4,-1), C4(2,-2); expected positions are
N(120,192), C(72,216), with H2/H3. A separately authored mirrored-N source has
N17(0,-1), C4(2,-2), with expected N(72,192), C(120,216). The producer must not mirror
or regenerate either input. Both use a source N-to-C Single bond and nonsequential IDs.
The sheet remains at minimum A4 dimensions for these bounds.

Unchanged `mol_oracle.py` checks the actual original/saved per-MOL bytes, molecular
formula, positions and total H against pinned RDKit2026.09.2: C2H6O and CH5N. The
unchanged V3000 consumer separately confirms no literal attached H declaration. The
unchanged raw V2000 SDF scanner checks the first record's duplicate ordered values;
the second DT-only/data suffix is also asserted byte-for-byte. These jobs have different
purposes: a supplier/property dictionary never establishes ordered metadata retention.

## Pixel evidence and its limits

The reference page is manually authored at the literal coordinates with explicit expected
H3/H2/H1 and H2/H3, using only the existing literal `graph_page_scene` entry. It calls
neither the new SDF producer nor its H sidecar/resolved renderer. Existing literal renderer,
font tests and the approved golden retain their separate independent evidence; this
reference is an integration oracle, not a new independent rasterizer or approved golden.

Pinned Pillow12.3.0 decodes actual CPU/GPU/reference PNGs, validates RGBA1024×1024 and
compares every pixel byte. Two separated row regions and three analytic source-direction
chord probes are decisive. Negative reference controls omit the second molecule, remove
only O/N attached H, and reverse second-row Y direction; all must differ. The H-removal
control must affect both row regions. This checks OH/NH2 functional-group depiction rather
than accepting merely nonblank pixels. Existing horizontal preferences are retained:
the first O's vertical neighbour is a tie and may put H left; N-right and N-left inputs
must keep their own source direction. No geometry is chosen or transformed by the producer
to hide attachment/clearance problems.

## Initial actual RED and held draft

```bash
cargo test --locked -p bondline-commands --test sdf_capture
```

Session `99869` is terminal, exit101:0passed/1failed. Complete mixed SDF Open succeeds;
actual `render.capture` returns `unsupported_content` because the current session is SDF,
not Drawing. Log `target/sdf-capture-acceptance-red.log`. This is an actual runtime feature
RED, not a new API/syntax failure or simulated producer.

Independent vector-only jobs subsequently confirm the three original per-MOL H/geometry
expectations and first raw record using unchanged consumers. Evidence is
`target/sdf-capture-acceptance-vectors.log`; it does not establish capture GREEN.
The first scanner inspection also establishes its existing `value_lines` key before the
acceptance draft is held; no old consumer changes.

The prepared test exercises both inputs through complete source Open, CPU/GPU capture,
exact state/owner/close-approval retention, Save and reopen. Authored parent document stays
null; source/current/saved/history/file association must not become a derived Drawing.
Additional cases refuse absent virtual page, cancellation and later unsupported, degenerate
or empty CTABs without partial output or source loss. Root separately owns worker/history/
save-lease implementation tests. Derived display grants no Save/edit authority.

All preparation uses the isolated absolute target, incremental0/dev-test debug0 and existing
Root BONDLINE_PYTHON. At preparation the draft was held for the real IO/render/capture ports. No new stub or
future API was referenced and no capture GREEN was claimed. Root subsequently supplied
the actual backend `043b410`, composing IO `5cd4eec` and renderer `e3975c4`; these are
Root-supplied commit identities, without worker Git operations. All preparation jobs were
terminal and Actions remained disabled.

## Additive adapter draft

A separate new `crates/cli/tests/sdf_capture_adapters.rs` contains three additional tests:
actual CLI, stdio MCP and authenticated loopback Open → explicit CPU capture → exact Save.
They reuse the unchanged pinned 777-byte mixed V3000/V2000 SDF and its existing whole-source,
raw/per-MOL consumers, separately from the original VAL-absent geometry vectors above.
Every result keeps document:null and the exact two-record summary/source metadata.
Pillow validates the actual base64 PNG, CRC/decode, RGBA1024×1024 and dimensions; the detailed
pixel/label oracle remains only in the command tests. No whole image oracle is duplicated.

The persistent adapters compare the capture result (with only capture removed) and later
Inspect to the exact prior state, ensure no cached PNG in Save/automatic state, and require
all source bytes unchanged. The loopback callback must remain at the single Open repaint
after capture and a subsequent Inspect. Clients reuse the established bounded process/
network timeout and secret-token practices; no real credential enters logs or records.
At its separate preparation checkpoint this additive draft had no adapter GREEN. No old
adapter test or dependency was changed and no new command/API was invented. Real backend
execution below is separate from that historical draft.


## Actual unchanged acceptance GREEN

The received backend passed all three command/image tests and all three adapter tests
on the first actual run, session `18287`, exit 0. Both test files are byte-identical to
pre-backend draft v2: no analytic probe, expected page, coordinate/H count, source byte,
label/row condition, tolerance or error assertion was corrected after backend execution.
The original unsupported-content RED remains preserved.

The original N-right and N-left variants each compare all 4,194,304 CPU/GPU/reference
RGBA bytes through Pillow12.3.0. Both two-row regions, the blank inter-row gap, three
source-direction chord probes and three missing-row/H/Y negative controls pass. Existing
raw/per-MOL consumers, expected C2H6O/CH5N and implicit H totals, explicit ordered SDF
queries, null parent model, exact content/state/close approval, cancellation and absent/
unsupported/degenerate/empty-record refusals pass. Save and reopen retain the full source.

Actual CLI, stdio and authenticated loopback on the separate pinned777-byte mixed source
return structurally valid CPU PNGs without authored/history/source/association mutation.
Captures stay explicit and uncached. Complete saved bytes face the unchanged raw/per-MOL
consumers. The loopback capture causes no repaint; only the Open notification is present
before Save. No real credential is logged and all clients/listeners/subprocesses close.

Executed owner commands:

```bash
cargo test --locked -p bondline-commands --test sdf_capture
cargo test --locked -p bondline-cli --test sdf_capture_adapters
cargo clippy --locked -p bondline-commands -p bondline-cli --all-targets -- -D warnings
cargo fmt --all -- --check
```

The two focused suites each pass3/0/0. Scoped all-target warnings-denied clippy and
workspace formatting pass in terminal session `63180`, exit 0. Logs are
`target/sdf-capture-acceptance-images.log`, `-adapters.log`, `-clippy.log` and `-fmt.log`.
The real RED and independent vector-only logs remain distinct. No further checks were
repeated after these passed.

All runs use this worktree's absolute target, incremental0, dev/test debug0 and the
existing pinned Root `BONDLINE_PYTHON`. The handoff lists all three intentional files,
exact inputs/logs and baseline identities. The pre-backend224-file ledger is not falsely
called unchanged after the explicit Root ports:216 remain exact; eight old paths now
match the received backend/Root snapshot exactly. Old fixtures, consumers, font and
approved golden remain exact. Separate producer receipts and Root's183-command-test/
lifecycle gates are not pooled into this owner's six acceptance cases.

Root's full receiving aggregate, new Windows/Linux, native window/Open/display/Save UI,
clipboard, professional layout/chemistry and remaining milestones are unverified here.
Minimum A4 fits these inputs; general growing-sheet/geometry/resource coverage belongs to
the separately verified producer. Existing font readability guards remain intact; this
slice does not change a smaller viewport refusal to force success. No new golden or human
approval, production/old-test/dependency change, Git or Actions operation occurred.
All own Cargo/oracle/client/listener jobs are terminal.
