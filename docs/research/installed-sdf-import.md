# Installed SDF import: bounded observation

Observed2026-11-14,05:29–05:38UTC, installed26.1.0.6801 on macOS27.2arm64,
existingPID50732 and exact /Applications bundle; active license tier remains unknown.
This supplements the installed feature map. It implements no Bondline SDF workflow.

## Observed workflow

Both independently authored two-record inputs open into **one document with two
separate molecular drawings visible on the displayed page**. The ordinary V2000 input
contains CC and CCC; the mixed V3000/V2000 input contains two C2 structures. Neither
Open presents an additional record/import-choice dialog or initial record browser/table.
This does not establish internal fragment grouping or the number of unviewed pages.

The first local File>Open panel initially shows All Supported Formats; MDL SDfile is
an enabled filter with no displayed V2000/V3000 distinction. Selecting that filter
opens the ordinary input. The second panel again initially shows All Supported Formats;
the mixed input opens under that state. These are current-session observations,
not asserted factory defaults or a generic Save-version rule. No cloud/account route
was followed and no reference export was attempted.

On the first owned import, Document Properties has an empty document-level table.
Object>Attach Data is disabled without a selection and enabled after Select All; its
new-attachment sheet has empty name/data/tag, no search operator, automatic position
and an unchecked chemical-significance flag. Both dialogs were cancelled without an
edit. An empty document-properties table and a blank **new** attachment sheet do not
establish whether imported per-record fields were retained, collapsed or discarded.

## Inputs and evidence

Only original, openly licensed inputs were opened; no proprietary sample/template input.

| Input | Exact bytes / SHA256 | Independent input check |
|---|---|---|
| Original named-only V2000 pair |760 /3e6013ab33a06bd64938cd3801f35f88ee670ab6e21e5b5dfa43d8bda3609c43|Pinned external RDKit2026.09.2 writes CC/CCC molecular blocks; separate raw count and consumer verify both records. Ordered duplicate/empty/multiline fields exist in input.|
| Existing original mixed fixture |777 /165e6f5d5cff37dae6315a07843d84733aa42dbbc497b2920da1702bad9526a0|Exact copy of crates/io/tests/fixtures/sdf/mixed-v2000-v3000.sdf; raw ordered fields and each molecular block checked independently.|

RDKit's duplicate-name dictionary and documented DT-only full-supplier limitation are
not preservation oracles. Input validation does not prove the reference retained fields.
CUA performed menu/keyboard/AX actions; four passive REA6.9.0/Node26.12.1 captures record
only their after-states. Parent-window captures and separate OS sheet images are labeled
separately. Raw/screenshots remain ignored; no code, icon, font or help prose is reused.

Root target/reference-sdf-handoff.md SHA
 d0c53eb8ea053fe057658c75eedae4b35dfee397560dd8326d360f33ca609ccf
lists the four exact REA EvidenceIDs/raw hashes and ordered methods/actions/results.
The41-file evidence index hashes to
65ef68646f4c3354c577322ff15e10cd98b672a5adee913f3e893873a893a165;
Root independently rehashed every entry. Flow ledger SHA
f23c6944e4594425cbe93267a4e710e07332de9c9dcb7b53f57943158221d115;
cleanup SHA c7ace82ad135263e558ab0587a1830713e55afe42bab2c1aadc8925b2cc7bb87.
Two harmless probes remain excluded: a wrong cached CLI subpath and an ambiguous bundle
ID corrected to the exact installed path. No install/config/access change occurred.

Both owned imports closed cleanly without a Save prompt by05:38:17UTC. The four sampled
original window IDs/frames/layers and cursor are preserved; PID50732 stays running.
No owned document/helper/debugger/job remains; CUA completed once after cleanup.
This is sampled preservation, not continuous observation or a rewrite of the earlier
historical window-restoration incident.

## Implementation implications and unknowns

Retain every SDF record and ordered field; first-record-only projection cannot represent
this observed Open result. Record boundaries remain source semantics even if multiple
molecules are displayed on one page. A new Bondline record browser/data editor is an
original policy until independently evidenced, not established26.1 UI parity.

Imported field retention/attachment, internal object grouping, precise layout rules,
record-selection correspondence, Save/export versions/fidelity and deeper data controls
remain unknown. No missing Bondline feature becomes verified through this reference pass.
The implementation ticket remains M1-05; the source-preserving SDF library is unchanged.
