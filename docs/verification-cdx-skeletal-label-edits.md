# Editing a printed atom on skeletal CDX paper

The existing atom label form changes the element and attached hydrogen count of a printed heteroatom beside hidden carbons. The writer uses the current position, recomputes the printed text and bounding box, reimports the entire drawing and checks the paper before creating one history entry. Hidden vertices, foreign styles and unsupported label geometry retain their existing restrictions. The tool catalogue and command list stay at their previous counts.

| Real CLI workflow | Saved bytes | Independent RDKit result |
| --- | ---: | --- |
| Ethanol OH → NH₂ | 371 | CCN, C₂H₇N |
| Methanol OH → Cl | 314 | CCl, CH₃Cl |
| Moved methanol OH → NH₂ | 315 | CN, CH₅N |
| Second of two methanol fragments OH → NH₂ | 505 | CO and CN, combined C₂H₉NO |

Each workflow compares all bytes after edit and Redo to independently constructed files, and every byte after Undo to the original. Each saved file reopens and reports the expected formula. RDKit 2026.09.2 reads the saved binary directly. The repeated-fragment case exercises duplicate raw IDs in different containers. The fresh expectations use a struct reader and FontTools outline bounds. [Original sources and fixture generation](../crates/io/tests/fixtures/cdx-skeletal-label-edit/SOURCES.md).

The first CLI run refused the ethanol edit. After enabling the qualified profile a second test found that the changed 15-byte Text payload lost its extended length header. The opt-in standalone property writer now retains that header. Its tests include short lengths on either side of 65,535. The original canonical replacement API retains its original encoding contract.

The command tests also check the wrong fragment, a stale revision, an unlabelled carbon, a new printed carbon, and a valid no-op while Redo is available. The macOS App-frame test Alt-clicks and cancels an exposed OH label, moves it, reopens it, changes it to NH₂ by editing both fields, then applies and traverses history. Stdio MCP opens the repeated-fragment drawing, edits its second fragment, saves to a Unicode filename and reopens the exact expected binary.

Three drawing cases ran on an actual background-only macOS host. The authenticated command channel targeted the existing label. macOS accessibility set the actual Element and Attached hydrogens fields and pressed Apply in the background app. Each edit entered history once. The host then saved and reopened the file and displayed the resulting formula in the Analysis window. CPU and Metal produced identical 512 × 512 captures in all three cases. The final native receipt was recorded in `target/skeletal-label/proof-gdf3v9qv/receipt.json`, SHA-256 `4a9f1b5163e5210fb83e75582b4af7d4713fafb3f0229f19487b04f068c90b26`. That host closed normally. An earlier remote Cancel invalidated the command request and retained a copyable, stale form in the native window. Pressing Apply in the actual form exercised the normal user workflow. A macOS Alt-click on the physical desktop and interactive Windows/Linux execution remain unverified.

## Replay

```sh
cargo test -p bondline-cdx --test edit
cargo test -p bondline-io --test cdx_mixed_skeleton
cargo test -p bondline-commands --test cdx_skeletal_move
cargo test -p bondline-ui --lib mixed_skeletal_binary_moves
cargo test -p bondline-cli --test analysis_cdx --test sdf_capture_adapters -- printed
```

Set `BONDLINE_PYTHON` to the pinned RDKit environment for CLI tests.

## Browser and sanitizer checks

A real headless Chromium session ran the maintained WebAssembly codec example, changed the second fragment of the twins drawing, compared all saved bytes with the independent file, reopened the model and compared the CPU paper. The module was 1,739,401 bytes, SHA-256 `d78f4abd61efbb33ca5cf6590f16cdb99d49009a3362350e3e33863da250178e`. Browser file import and a pointer-driven browser label edit require separate verification.

AddressSanitizer with cargo-fuzz 0.13.2 and nightly 2026-04-01 completed 10,000 runs in two seconds. All 356 distinct original seeds retained their SHA-256 hashes. The largest reached input limit was 45,173 bytes and the final run produced zero crash artifacts. The initial seed pass exposed an obsolete harness assertion that had prohibited all mixed-label edits. Its 314-byte seed was replayed after changing the assertion to require the exact editing profile and retain the complete-mapping refusal. The final log SHA-256 is `c57796ff8101b5a078047a9e828f50df735c53f8d19afd684733279062510ec9`.

## Integration with the macOS control listener

The full `cargo xtask ci` gate passed on `804c72a`, including workspace rustfmt, warnings-denied Clippy, 2,421 passing tests with one ignored, three additional macOS AccessKit tests, layer and license checks, assets and the pinned corpus. The test that immediately rebinds the released socket passed. Two previous runs on `7df66a2` had failed at that assertion. The integrated log SHA-256 is `f63be5200831c0fcf3b30a8d1130bb1465fe9bfbc2e0940eacfd3691aae5bfcc`.
