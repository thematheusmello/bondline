# SDF current-view core review

Read-only independent source review, 2026-11-15. **No material defects found in the
exact captured scope.** This is not native/UI acceptance or broad feature-parity evidence.
No product/test changes, builds, tests, Git, hosted Actions or native interaction ran.

## Exact inputs

Twenty files were snapshotted before reading under ignored
`target/view-core-review/inputs/`; `inputs.json` is 3,475 bytes, SHA256
`263bfc5384807bbc1e9ef9108ba81955aea23e89a6ec1cd4d6cdc3483b723eed`.
It records every source/test/document size and hash. Principal implementation pins:

| Source | SHA256 |
|---|---|
| commands/capture.rs | `d4602ec09766652ed7eb86ebadf0a1fc2b703ad51caf60354caf794940addc7b` |
| commands/view.rs | `a4d341526582e36ff20e37974799ee4e02d88ccfc9a4b91f745480e1baab5c4a` |
| commands/controls.rs | `167eba548e89ed8c9c0e016f767077d003024b371470dbf4dc093a8f6f045a3e` |
| commands/files.rs | `8616aa454b8d676e021203ee8bc9f07e1468e9eec6b9047887256992aee8c4a9` |
| commands/lib.rs | `12702b280360662f1c8fc66660634098cf291c51fd8db7561c73148cf2173737` |
| mcp/work.rs | `bc06217cb8c2097b072cc98e3b6d52c3d6c8749300c34ef8bc44a9af73418319` |
| mcp/control.rs | `21d28fa7f2177d57ab321ed86d816d1afa6168f5b17c199ec189afe1a1058f14` |

The received renderer matches the frozen producer exactly: view.rs
`e68580393a94c621aa412a83271b8e3a00eb9d2430f3f8775090944331aec27e`,
page.rs `29811c04fecf23975a7406332fe9f4551e0efe642ac0f0ada583c3a7316a854a`.

## Connected behavior checked

- Optional capture context snapshots validated logical dimensions, current view epoch,
  complete source owner and Weak source/revision/generation identity. SDF still requires
  virtual page1; Drawing still requires the exact page and completely mapped source.
  All-record preparation, source-aware display-H and fresh physical raster generation
  remain below the UI, with no first-record/empty Drawing substitution.
- Extent installation occurs only after successful scene rendering and PNG encoding.
  Errors cannot produce its completion; early/late cancellation checks precede install.
  Installation checks current SDF kind, Open close state/epoch, view epoch and exact
  source stamp. Historical/foreign replies may return their image but install nothing.
  The cache holds only Weak identity plus PageSize, outside authored history/saved data.
- Actual/percentage/explicit relative zoom can bootstrap SDF without guessing bounds.
  Fit-relative actions and Pan still require real current bounds as appropriate.
  Cache validity depends on source, not later camera changes; source edits, history or
  identical-byte Open cannot revive it merely through geometry equality.
- Native `prepare_ui_capture_for_close` validates captured close epoch and source under
  the originating mutex before worker reservation. Its originating Engine and shared
  two-slot permit survive actual completion/finish; errors/drop release them. It does
  not bind/ack a native file request, release a Save reservation or grant close approval.
- `view_target_prepared` defaults false, is skipped in public JSON and becomes true only
  for a newly installed current extent. Loopback publication considers it for repaint,
  releases response activity after write/flush, then notifies even on failed delivery.
  Repeated equal extents do not manufacture another view change.
- Absent/null `view` retains legacy whole-page rendering, independent of window camera,
  with no extent side effect. Opted-in logical Fit intentionally differs at higher DPI.

## Test substance and remaining boundary

Read all seven command acceptance cases and both new MCP cases, plus the connected
legacy view/capture lifecycle tests. Their expectations include literal 1x source
coordinates/label regions through pinned Pillow; twelve actual source records with
independent first/last MOL consumers; last-record unsupported chemistry off-camera;
source/view/close ABA and foreign finish; early/late cancellation; closed nested schema;
Save/source/wrong-kind/native-receipt authority; and notification/close-epoch behavior.
The old SDF Actual refusal was explicitly replaced with an authored-state/source/stamp/
whole-Save-preserving positive and unavailable-bounds Pan negative. No test execution
was repeated here; passing results are Root's evidence.

Changing UI files, wheel handling, strict-helper adoption, actual logical-readability
checks, preview-key invalidation, job retirement after decode and native input remain
outside this review. They require their own receiving tests/native gate. Before/after
source hashes and any concurrent drift are recorded in the ignored final receipt.
All own jobs are terminal.
