# Native SDF files and terminal-carbon drawing

Verified on source bb6d5b9d2e6dd0d34e9b326dc9bf8061ffaff5a2, macOS27.2 arm64,
Apple M3 Max/Metal, Rust1.94.1, default dev profile and incremental0. This is bounded
native evidence; M0/M1 and general chemical drawing remain incomplete.

The actual executable was18,235,928 bytes, SHA-256
316a719f1063ba880384b7a626cb4904e9783d26c0fe29d177873ce13c31a511.
Its396-file source manifest is68,760 bytes, SHA
3ddce1bc1a04373f25f9826c9668d276f49410caed7e534defb93997804c5935.
Root independently checked every source identity and the exact binary before editing.
The actual CLI reports18 commands; native controls are only contextual delivered bindings.

## Actual native actions

| Workflow | Observed result |
|---|---|
| Empty paper, label and atom-anchor dragging | C2, then C3 with H3/2/3, then C4 with H3/2/2/3; old IDs/anchors retained and one history step per extension |
| Stationary and one-pixel label actions | No unintended edit |
| Dirty Open Cancel, Undo/Redo | Exact previous graph/file/history retained; saved C3 clean versus C4 dirty restored |
| Unicode CDXML Save/Save As/reopen/edit/known Save | Actual653-byte C3 and811-byte C4 files; reopened C3 can extend and save as C4 |
| Mixed SDF Open/known Save/Unicode.sd Save As/reopen | All777 bytes, two records, V3000/V2000, fields6/1 and document:null retained |
| Misleading.mol with explicit SDF Open | Actual decoder choice succeeds; known Save keeps SDF association/path and exact777 bytes |
| Wrong SDF Save As suffix, malformed later record, CDXML conversion | Visible refusal, no target/current-data loss; all777 bytes retained |

Screens are actual own-window captures. Native actions/AX observations, public OS
captures and authenticated controller inspections are distinct evidence. SDF displays
a static record summary and explicitly unavailable chemical preview. This is an original
partial UI, different from the reference's observed all-visible molecular drawings.

Actual saved C3/C4 outputs face independent Expat graph/H/Decimal checks and external
RDKit2026.09.2: CCC/C3H8 and CCCC/C4H10. Decimal checks agree with the native authored
positions; the new point locations were not independently predicted from screen geometry.
No new point-placement precision claim follows.

Actual777-byte SDF outputs match the original SHA
165e6f5d5cff37dae6315a07843d84733aa42dbbc497b2920da1702bad9526a0.
Fixed raw literals verify every ordered field/header/DT/empty value and molecular span;
unchanged external V3000/MOL consumers read both actual saved payloads. The V2000-only
raw scanner was not changed or applied to mixed input. No lossy property map proves fidelity.

The published [C4 own-window capture](media/connected-chain.png) and
[mixed-SDF summary](media/sdf-summary.png) are exact unedited copies of these native
artifacts. [Media provenance](media/SOURCES.md) records their distinct build/state/hash;
neither is a later20-command build or a newly approved regression golden.

## Controller-only finish and limitations

At09:09:14.502779UTC another application took foreground. Input stopped before the
prepared native MOLV3000 Save. A mistaken approximate status timestamp was corrected;
audit/file UTCs are authoritative. No subsequent activation or input displaced that owner.

Authenticated cancellation retired only the owned pending picker. Controller-only MOLV3000,
CDX, bond/page and CPU/GPU capture refusals preserve all mixed source and create no file/fake
capture. Subsequent exact777-byte Save passes the same independent consumers. Controller-only
690-byte Open/Unicode Save/reopen/known overwrite preserves both V2000 records and fields7/0,
SHAf95838cbb55c107c649ce2eb99ac62e191f747ce6994a171e64019f8e6c53d1a. The unchanged raw
scanner/per-record RDKit pass; known full-supplier DT-only loss remains diagnostic.

These are not native chooser checks. Native690/MOL refusal, return-to-drawing/canvas-drag,
new placement precision, special2x/held/modal/origin/minimum-width cases remain unverified.
There is no SDF chemical preview, field editor, general inference, stereo or full parity.

## Exact reports, cleanup and copy correction

Ignored handoff: target/native-sdf-session-ui/handoff.md,11,016 bytes/SHA
1855b52d8e5a558ad167080e5cef99b850e077d7a1898f5f8b3f0d1d1bac6370.
Index: artifact-index.json,19,445 bytes/101 records/SHA
1ff6372c4d04b0b89600905ab33cb889a9caca295567c1fe147d8a31d661829c.
Root independently rechecked all101 records. Earlier setup stdin-EOF, unfocused drag,
unavailable optional launch API and incorrect first external XML-tag probe remain excluded.
No product fix was made to make those probes pass.

Guarded clean revision12 close completed: native78764, controller78720/session94655 and
helpers exited; windows absent, no live handles. CUA completion occurred exactly once after
observed cleanup. The four sampled reference windows/frames/layers and screens are preserved;
cursor exactly1320.02734375,810.21337890625; both left/Escape tables Up. Later foreign
foreground was preserved rather than restoring the initial foreground over it. No reference
application, account/TCC/display setting or proprietary asset was changed.

One actual UI copy defect was found: Open help omitted SDF despite its working decoder.
The subsequent one-line fix lists.sdf/.sd; rustfmt and all-target UI clippy pass. Its new
wording has not yet been rechecked natively while another GUI owner is active. No artificial
constant-mirroring test or native success is claimed for the wording change.

During later full-disk maintenance the original CLI cache was removed before its archive
could be written. The native binary/sources/other100 records stayed intact. Rebuilding with
the original workspace test profile (`cargo test --workspace --locked --no-run`) restores
the exact original18,630,816-byte CLI SHA
3b024d7724b2645e871f06f2b3af6f02b4a9e734b250c9361a50094eac17a167. It is now additionally
archived outside debug artifacts at target/native-sdf-session/cli-bb6d5b9; all101 original
index hashes then match again. Solo/workspace dev-build candidates differed and were never
relabeled as that original. The recovery record is target/native-cli-artifact-recovery-bb6.json.
This is build-artifact recovery, not a claim of universal bitwise reproducibility.
