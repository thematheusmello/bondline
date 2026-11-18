# Existing-bond edits in the native Mac window

2026-11-15. Root alone operates the user-authorized Bondline preview. No reference
program is launched or inspected. The current public-source-only clean-room boundary
and disabled GitHub Actions policy remain in force. This is a finite native check,
not full M0/M1/M2, style parity or Windows/Linux evidence.

## First native authoring build

The real Rust desktop executable uses egui/wgpu on Apple M3 Max / Metal, macOS arm64.
Its UI source is `c14a383a6c648040afefa69f0640abc7f1fae980a82362da628bd72358567021`;
commands sources are the committed backend `2b0f906`. Executable SHA-256:
`74be5aff7f50f7652fadcede054a7a2fc2e883dd8a70019d3c70718617cd0fb5`.
A disposable `.app` wrapper under `target/bond-order-native/` launches that binary
with `--control-port 0` and redirects its startup output to ignored files. Its shell
launcher is development packaging, not an application runtime dependency or release.
The preceding saved visual-review window exits normally using Cmd+Q before launch.

All application input uses CUA on the actual window. Authenticated loopback inspection
reads the resulting complete state after each operation; it does not substitute
registry authoring for native input. The one setup exception is explicit
`page.set_size` to420×420pt, recorded as such below. Each bond is authored with the
native pointer, and the Unicode file save uses the actual native chooser.

| Native input and observation | Readback |
|---|---|
| Empty A4: Single drag(420,350)→(555,350) | Revision1, one fragment/one Single/two C/H3; actual inspector2/1/1. |
| Short center click in that A4 view | No edit. Cause is unestablished; no native click success is inferred from this attempt. |
| Hotspot3 in the same view | Revision2, existing edgeTriple/two H1. The visible renderer reports its multiple-component pixel limit; no fallback or hidden guard relaxation. CmdZ restoresSingle at revision3. |
| Registry preparation only:420×420pt | Revision4, same positioned graph; provides readable scale. This does not verify a native custom-page or zoom workflow. |
| Single-tool body click(620,589),120ms | Revision5, same edgeDouble/two H2. IDs, positions and unrelated data stay exact. |
| Stationary3 then1 | Revisions6/7:Triple/H1 thenSingle/H3; same logical edge, no pointer retarget. |
| CmdZ, then2 without a fresh point | Revision8 restoresTriple/H1; the stale key preserves the entire state and bindings exactly. |
| CmdShiftZ | Revision9 restoresSingle/H3. |
| Actual Double selection and drag(300,300)→(450,300) | Revision10, new Double/two H2; old fragment unchanged. |
| Actual Triple selection and drag(300,420)→(450,420) | Revision11, new Triple/two H1; inspector6atoms/3bonds/3fragments. |
| SelectSingle; drag across Double endpoints(300,300)→(450,300) | Revision12, that same edgeTriple/two H1, one history entry, every ID/position/other fragment exact. Tool bindings stay exact. |
| CmdZ | Revision13 restores all three original graphs, Undo7/Redo1. |
| CmdShiftS; set Folder/Go and Unicode filename; Save | Actual chooser closes, Saved state and correct filename appear. Complete graph unchanged; file is1093bytes. |

Readback receipts are ignored under `target/bond-order-native/`: the named states,
`build-receipt.json`, `larger-view-setup.json` and `saved-consumer-receipt.json`.
The manual endpoint comparison initially overrequired equality of all control envelopes,
which correctly carry the current revision in file-command parameters. Its graph/H/one-
commit assertions had passed; the corrected check compares the unchanged three tool
bindings. That harness mistake changes neither production nor any native input/receipt.

## Independently consumed save

The native chooser writes `native-bonds-λ.cdxml`:1093bytes, SHA-256
`3940bfd4e27d5f8379f55e6c6270a85187a39614552202a35c1b11d588b5f0d8`.
The unchanged Expat/Decimal oracle independently recovers every actual saved fragment,
ID, endpoint direction, position, order and literal H from those bytes. External pinned
RDKit2026.09.2 reads three separate two-carbon structures with bond weights1/2/3,
H3/H2/H1 and formulasC2H6/C2H4/C2H2. These are actual native authoring/save bytes, not
an own-writer/own-reader-only round trip or pre-generated substitute.

The window screenshot is an untouched own-window buffer; it is documentation evidence,
not a new human-approved chemical golden. Startup tokens stay ignored and private.
No new dependencies, lockfiles, parser/renderer production source, old fixtures/oracles,
font or approved golden changed. Native multiple terminal extension, reversed endpoint
conversion, broader source editing, zoom and complete per-tool behavior remain unverified
or missing. Headless tests cover their separately stated scopes.


## Final menu-routing build and editable reopen

The final actual executable is20,626,504bytes, SHA-256
`aa38e594f35d6fb6ff23e4eb6cad63aa6ec20a78bc35fcad0dfb1e011e53292c`;
UI source is `fd39fe34002841c7e1835b3521b6fb97640b4e2170f82c83cbeb59dc6cb502d0`.
The saved first authoring build exits normally before this separate disposable bundle
launches. No custom-page setup or registry authoring is repeated in this build.

Actual CmdO/Folder/Go/file selection/Open loads the1093-byte native-authored file into
a clean source-backed session at revision1, Undo0/Redo0. Its complete document equals the
first build's saved scene exactly, and source is{cdxml,1093,modified:false}.
A native Single-tool click on the first bond setsDouble, stationary1 restoresSingle and
stationary2 setsDouble: revision4/Undo3/Redo0, same IDs/direction/positions/unrelated
fragments, H2 at the two endpoints, source.modified:true. This is positive native key2
and editable source-backed CDXML evidence, not just a stale-key refusal.

CmdS saves that actual edit. The unchanged Expat/Decimal oracle recovers the entire graph;
pinned RDKit reads formulasC2H4/C2H4/C2H2. The saved1093bytes have SHA-256
`b966edc862d25bab8a18040cd962e67e7830ded92d0ef0f7110c2a4c1226b41d`.
Its file baseline is clean while source.modified remains true, as designed.

Actual Edit→Undo restores the exact original whole graph at revision5/Undo2/Redo1,
source.modified:false and file.dirty:true. Edit→Redo restores the exact edited graph at
revision6/Undo3/Redo0 and the clean saved baseline. These are real menu activations on
the final source; the deterministic six-frame suite separately establishes same-packet
menu/keyboard ordering. CmdZ and CmdS then restore the exact original1093-byte saved file,
SHA3940bfd4…f0d8, at revision7/Undo2/Redo1 with both modified/dirty false.

The final saved preview stays intentionally open as owned PID83437, with no launch waiter
or worker Cargo jobs. Its bundle, build/state/consumer receipts, private startup credentials
and original capture buffer are ignored under `target/bond-order-menu-native/`.
The user-facing [actual capture](media/workspace-native-bonds.jpg) and
[public-safe provenance](media/workspace-native-bonds-provenance.json) contain no token.
CUA returns JPEG despite the initial disposable `.png` filename; the published extension
matches Pillow's independently verified JPEG1160×832 format. Bytes are not transcoded.
EXIF contains dimensions only; retained IPTC is empty. This is documentation media,
not a new golden approval. Current default-view, chemistry, alignment, multiple-extension
and platform limitations above still apply.
