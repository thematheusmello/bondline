# SMILES file UI wiring and headless verification

2026-11-14. This slice connects complete SMILES files to the existing file chooser,
shared file worker and nondrawing workspace. Native interaction is **unverified**:
the user was using the computer, so no window, pointer, keyboard, activation or
screen-capture workflow was run. These results are egui/input/accessibility and
actual filesystem/command-worker tests, not native acceptance or reference parity.

## Delivered scope

Open and Save As expose one SMILES family for `.smi` and `.smiles`, including
case-insensitive aliases. An unnamed Save As defaults to `.smi`; a supplied
`.smiles` name stays intact. An explicitly chosen decoder remains separate from
the filename. Known Save uses the associated format and path. Failed operations
retain their request, selected family and exact attempted path for explicit retry.

The file workflow calls the real registered `document.open_smiles` and
`document.save_smiles` operations through the existing executor and save reservation.
It does not select a record, infer chemistry, regenerate notation or implement another
publisher. The actual backend preserves all source bytes, including mixed endings,
opaque metadata and ignored lines; that backend's broader evidence is separate.

The workspace shows only record count, byte count and an explicit unavailable-preview
message. It uses the compact `smiles_summary()` result, without a record vector or
metadata strings. A zero-record file remains a source summary with no synthetic drawing.
Opening a SMILES file clears held drawing gestures and hover state, and removes the old
paper/canvas. Reopening a drawing does not rearm stale pointer state. Ordinary labels
also expose the summary to AccessKit without publishing notation/application data.

Native presentation, decoder selection by actual controls, Save As/overwrite/close
interaction and platform accessibility remain unverified. No record editor, molecular
preview, clipboard workflow or new chemical authoring tool is delivered here.

## Five new tests

| Test | What its evidence establishes |
| --- | --- |
| `smiles_chooser_has_one_source_family_aliases_and_exact_failed_save_retry` | Chooser helper family/labels, both suffixes, default name, wrong-format Save As refusal and exact failed known-path retry. This is not an executed native picker. |
| `smiles_worker_open_save_as_retry_known_save_and_close_retain_the_whole_file` | Actual shared worker Open with an explicit SMILES decoder on a `.mol` filename; Unicode `.smiles` Save As, existing-destination retention and authorized retry, known Save, exact complete source bytes, and clean close blocked until the pending Save job drains. |
| `malformed_later_smiles_record_preserves_the_open_session_and_decoder_retry` | A malformed later notation cannot replace the complete current content, revision or association; the failed picker retains the SMILES decoder/request for retry. |
| `smiles_open_clears_held_gesture_hover_and_paper_without_disclosing_record_bytes` | Ordered egui hover/press, real SMILES Open and later release clear stale canvas/gesture data. Counts and unavailable-preview text appear in shapes and AccessKit; opaque private markers do not. Drawing reopen stays clear across idle frames and an old release. |
| `zero_record_smiles_is_a_source_summary_without_a_synthetic_drawing` | An ignored-only source shows zero records and no paper/dummy drawing, retains state, and does not disclose its metadata. |

All sources are original inline test bytes. Literal expected counts/bytes and a complete
content comparison are independent expectations. The UI does not add an interchange
oracle: existing lower-layer raw-record and external chemistry consumers keep that role.
The immutable file has no public editing command, so no dirty-SMILES close-Save scenario
is fabricated. The close test covers a clean file and an already pending real Save job;
general trusted close-Save validation remains in the existing shared engine tests.

## Actual verification and qualifications

The five UI source/test files were frozen before the real backend port in
`target/ui-smiles-source-freeze.json`:1708B/SHA256
`f16d531b06d5c18c5ee0cb82319a07d0562bb5aac31f8d6a455a552227436ade`.
Every hash remained exact through the first real-backend GREEN; no production or test
correction was needed to obtain that pass. This new verification document is separate
from those five frozen files.

- UI attempt79466 exits101 in `target/ui-smiles-red.log`: missing actual
  `SmilesFileDocument`, `read_smiles_file` and `save_smiles_file` APIs. This is a
  **dependency-integration compile RED**; UI assertions did not execute. Root's separate
  runtime missing-command RED94529 belongs to the command tests, not these UI tests.
- After Root ported real codec `ca11b2f` and IO backend at supplied `28c2238`, UI
  test60698 exits0: **98 passed,0 failed,0 ignored**, across8 result suites including
  the five new tests. Log: `target/ui-smiles-green.log`.
- UI all-target warnings-denied clippy2620 exits0.
  Log: `target/ui-smiles-clippy.log`.
- UI package formatting check exits0.
  Log: `target/ui-smiles-fmt-final.log`.

Commands use this worktree's absolute isolated
`CARGO_TARGET_DIR=/Users/sarenaberg/Workspace/doozle/open-source-world/bondline-render-labels/target/ui-smiles`,
`CARGO_INCREMENTAL=0`, dev/test debug0 and the existing pinned Root `BONDLINE_PYTHON`.
No shared target, stub, dependency, asset, golden, parser or writer was changed.

```bash
cargo test --locked -p bondline-ui
cargo clippy --locked -p bondline-ui --all-targets -- -D warnings
cargo fmt -p bondline-ui -- --check
```

The ignored handoff records exact source/log hashes. All own jobs are terminal. Root owns
the command/backend edits, receiving workspace checks, public status/provenance and Git;
these UI checks do not establish a full-workspace pass, new Windows/Linux execution,
native UI acceptance, complete M1 or completed application parity. GitHub Actions remains
disabled under the user's development policy.
