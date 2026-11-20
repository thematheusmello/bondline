# Analysis window receiving checks

On 20 November 2026 the unmodified CLI on the parent drawing branch advertised 51 commands. Its View menu contained Actual size, Fit to window, Magnify and Reduce. Four new command tests failed on the unknown query or missing View control. After command implementation, an actual BondlineApp frame accepted the Show control but failed to paint the empty result. A loopback publication test separately failed on the absent native repaint.

## Results

| Receiver | Observation |
| --- | --- |
| Command engine | Empty drawing, methanol plus ethane `C3H10O`, complete selected methanol `CH4O`, selected O/H `HO`, and selected O/H plus ethane `C2H7O`. Undo cleared selection and restored `C4H12`. Successive same-revision selections retired earlier epochs. A numerically equal graph under a different retained content owner refused the old selection. No query changed the document, history, file state or capture identity. |
| Source and limits | Numbered carbon and signed charge produced `[13C]H3-`. Missing H identified its atom. The qualified two-fragment CDX contained independent N/H2 and Cl/H0 and yielded `ClH2N`. The detached and data-only CDX controls and an opaque CDXML source had no formula. 4,096 He atoms succeeded and 4,097 returned only the limit. |
| Discovery | The first 51 complete records remained identical in order and metadata. The canonical JSON SHA-256 was `d2d0c2d06c61edb1af9f749ee1edbaf3430284fa8c2d96c01ba38fde8a8bd8dd` for CLI and `beec5dc0f2b20aa596ff3d788ec63c4215b37e6ac61aacbb0a0f2c7f959a7d33` for initialized MCP. The independent oracle pins the two new records and their full schemas, descriptions and annotations. |
| Wire and publication | A real CLI created ethane and read `C2H6`. A real initialized stdio MCP session opened the panel, selected one carbon and obtained `CH3` with a partial-fragment indication. A stale request failed. Loopback wakes only after publication of a visibility change. Inspection and repeated Show produced no wake. |
| Native interaction | AccessKit-targeted pointer events in actual BondlineApp frames opened View and selected Show Analysis Window, produced and refreshed the readout, followed Undo to empty, then hid the window. The formula disappeared after the selection or window changed. With Single bond active, the header was dragged over a real ethane endpoint and its close button received a same-frame press and release above paper. Only window visibility changed. |

## Background macOS host

A private test bundle on arm64 macOS 27.2 ran the real desktop executable with an ephemeral authenticated loopback listener. The initial Launch Services background request still made its process frontmost. The foreground guard terminated that process before reading AX or sending input. The final test bundle used the macOS activation-prohibited property, polled its own AX tree and did not send physical mouse or keyboard input. [Apple activation policy](https://developer.apple.com/documentation/appkit/nsapplication/activationpolicy-swift.enum/prohibited).

PID 93937 published `C2H6` for the complete drawing and `CH3` for one selected carbon. Its native AX tree contained both the expected headings and values. After saving a temporary CDXML, removing exactly one written H count in a separate file, and reopening it, AX reported atom 1’s unspecified attached-hydrogen count. An opaque graphic displayed the source coverage message. The process exited after Close. Whole and partial formula queries preserved exact native session state, and none of the four final AX reads saw the test process frontmost.

The final unsigned executable SHA-256 was `d54a603e93b470b4fb7b5ad353a6f558042c7459279dea33b3648501f4158bd6`. The harness receipt was `60dc36cf6c3ccbe1d9ab67c7c98ae01f4bb35db9964014fa60676a24db0c2a60`. No Windows or Linux desktop interaction or new golden is claimed.

## Reproduce

```sh
cargo test -p bondline-commands --test analysis_formula
cargo test -p bondline-cli --test commands --test mcp analysis
cargo test -p bondline-cli --test analysis_cdx
cargo test -p bondline-mcp --lib analysis_visibility
cargo test -p bondline-ui --lib analysis_
cargo run -p bondline-cli -- run --cmd '{"command":"bond.create","params":{"page_id":1,"begin_x_pt":72,"begin_y_pt":72,"end_x_pt":120,"end_y_pt":72}}' --cmd '{"command":"analysis.formula","params":{"expected_revision":1,"expected_selection_epoch":0}}'
```

The public source scope, missing fields and prospective CDX profiles are recorded in the [Analysis window contract](research/analysis-window.md).

The standalone skeletal fixtures give propane with all three H counts declared, its central count omitted, or every count omitted. Before admission the new command test returned `unsupported_source` for complete propane. After admission the command and CLI return the declared formula or the corresponding missing atom. An unchanged native-source Save representation and a detached-model negative are checked in the engine.

## Skeletal and printed-label CDX receiving

The two profile extensions each first failed because a source with all H counts written returned `unsupported_source`. The final source gate reuses the exact complete-page certificates. Whole, partial and stale results were checked through command tests, actual CLI, initialized stdio MCP and actual BondlineApp frames. RDKit’s independent source reader confirms `C3H8` for propane and `CH4O` for the fully explicit methanol. It also infers H for the supplied comparison drawings. This analysis command continues to disclose H from the literal model.

An activation-prohibited macOS test host opened seven original CDX files. Native AX text showed `C3H8`, `CH4O`, a selected terminal `CH3`, and the expected missing-H messages for atoms 1 and 2 as each source was opened. The mixed printed-label source refused a selection request and preserved the previous state. Queries preserved exact file and history state. PID 15528 exited after the registered Close action. Executable SHA-256 was `68fd1ad3b08f4abd93af6f74508665d593af0606fd605cb92fef8768b9fcc381`. The source and AX receipt was `2262bcdf119cda71858b781ecf669fcd87e68dce7f0a3247544f10084cf2733b`.
