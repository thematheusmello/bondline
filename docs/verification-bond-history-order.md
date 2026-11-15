# Bond editing and ordered history input

2026-11-15. Independent regression owner in `bondline-order-history`, created by Root
from checkpoint `1322290` with the real in-progress command/UI inputs recorded in
`target/history-draft-port.json`. This is headless egui evidence, not native input.
GitHub Actions remains disabled. This owner changes only the new tests and this record.

The source review found that the existing global Edit shortcut handler executes before
the ordered canvas events. A packet containing an order edit followed by Undo can
therefore undo the seed before applying that earlier edit. Undo/Redo availability also
changes within a packet. The fix must preserve history for content without a drawing
and refresh page/viewport context after history, which can restore a different page size.

`crates/ui/src/bond_history_tests.rs` uses the actual `show_workspace`, registry, shared
immutable owners and egui input. It adds no public hook, replacement renderer, fake
loader or duplicate command implementation. All key taps include their real key-up:
the pinned egui input normalizes repeats from held-key state rather than trusting a raw
repeat flag. Pointer positions come from the actual canvas and an explicitly declared
page; literal document coordinates, H counts, identities and history counts are checked.
The existing PageViewport is input-generation support, not a new independent math oracle.

The five declared workflows are:

1. Hover a Single, press 2, then Undo in the same packet. The complete prior owner must
   return with literal H[3,3], Single order, the same IDs/positions/source/file state and
   one redo entry. Both source-free and saved/reopened native CDXML cases run. A later
   numeric key without a fresh pointer must not reuse a history-invalidated hotspot;
   a fresh position followed by 3 must author Triple/H[1,1].
2. Click-release conversion followed by Undo must revert the click and retain the seed,
   with two revision advances, unchanged prior history depth and one redo entry.
3. On reopened native CDXML with initially empty history, 2 -> Undo -> Redo in one packet
   must end at Double/H[2,2], one undo entry, no redo and three revision advances. This
   catches shortcuts enabled only after an earlier event; original-source metadata stays
   distinct from the dirty edited model.
4. Undo a 256×128 pt page back to 128×128 pt, then create a bond in the same packet using
   the restored page's pointer coordinates. The new atoms must be at (80,100)/(104,100),
   with H3 and fresh IDs 3/4. This catches a new graph paired with an obsolete viewport.
5. Open the protected original two-record 690-byte SDF, edit record 1's field index 5,
   then Undo/Redo in one packet. Complete source/current content, ordered record summaries,
   source/file metadata and document:null remain exact; history must work outside the
   drawing branch. The on-disk original source is never changed by these edits.

These are UI routing/history assertions with literal expectations. Existing independent
CDXML/MOL/RDKit command tests retain their separate chemical/interoperability evidence;
this slice does not add a chemistry engine, native verification or a new approved golden.
Old tests, fixtures, oracles, font, dependencies and locks are not changed.

## Initial checkpoint

The tests are declared before Root's history scheduling fix. No runtime result is claimed
at this checkpoint; the module bridge and intended baseline RED are pending. Actual logs,
source hashes and subsequent results will be appended after execution.

Reproducible scoped command, using a separate absolute target and existing pinned Python:

```sh
CARGO_TARGET_DIR=/absolute/path/bondline-order-history/target \
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
BONDLINE_PYTHON=/absolute/path/bondline/.cache/oracles/format-python/bin/python \
cargo test --locked -p bondline-ui --lib bond_history_tests:: -- --nocapture
```

No live worker job remains at this draft checkpoint. Root owns production changes,
module bridges, integration, Git and later native execution. Full M0-M5 stays active.

## Observed baseline RED

The actual scoped command above ran in the declared isolated workspace, session 4801,
and exited 101: 5 tests, 1 passed, 4 failed; no compiler/setup failure. The unchanged
page-restoration case passed. The other failures were:

- Numeric 2 then Undo and click-release then Undo both failed exact whole-content
  restoration. The numeric test stops at its source-free variant on this baseline;
  its saved/reopened native variant remains a required later GREEN check.
- Numeric 2 -> Undo -> Redo on reopened native CDXML ended at revision 4 rather than 6,
  showing that history newly available within the packet was not applied.
- SDF Undo/Redo retained equal content but stayed at revision 2 rather than 4. Exact
  history/revision assertions catch the missed operations even when their net content
  would be unchanged.

The test source remains 12994 bytes, SHA-256
`1b95902598b3495604e15afcf1c2bfdabb6fc2a7ab274079af406c1a01011860`.
Root's real bridged UI input is SHA-256
`6ea4811823e4149d36b7260c5eda99866e1e5a3b8b6028ac85d3ab973fb6ac82`.
The log is `target/bond-history-ui-baseline-red.log`, 9365 bytes, SHA-256
`1fea4abae5e92ffc11aa1678742a944a5e3d7d6b1c0fb298293c46f80260dfc3`. A 171-file protected-input ledger and exact initial
author-file copies are retained in ignored `target/bond-history-evidence/`. No native
workflow, chemistry calculation or full-workspace GREEN follows from this RED run.

## First real fix and menu follow-up

Root supplied the actual UI production input, 72935 bytes, SHA-256
`c14a383a6c648040afefa69f0640abc7f1fae980a82362da628bd72358567021`,
and its separately owned final order tests. The five frozen regressions passed unchanged
in session 48882: 5 passed, 0 failed. UI all-target warnings-denied clippy and workspace
rustfmt check then passed in session 18154. These are isolated owner checks, not the
parent's whole-workspace or native evidence.

The requested source review then found that `control_menu` still returns a selected Edit
command which is executed before all canvas events. An additive sixth actual-frame test
opened Edit through a real button click, then sent this packet: pointer over the Single,
numeric 2 tap, pointer over the visible Undo menu item, primary click. The intended result
is the original Single/H3 whole owner, two revision advances and one redo entry. It fails
exact whole-owner equality on the real UI input above: session 50805, exit 101, one failed
test. This is a runtime menu-ordering defect, rather than an inferred source concern.

The sixth test is appended to the unchanged 12994-byte original five-test prefix. The
current test source is 14866 bytes, SHA-256
`79f8e629f8a224bdc90d0ee9e26d95f00d407431eba87c4ab8d1850b80d66a7f`.
No production changes or expectation reductions were made. The first five are GREEN;
the menu path remains RED pending a real routing correction. No live job remains at
this checkpoint.

## Final real-backend GREEN

Root supplied the indexed-menu implementation in `crates/ui/src/lib.rs`, 73635 bytes,
SHA-256 `fd39fe34002841c7e1835b3521b6fb97640b4e2170f82c83cbeb59dc6cb502d0`.
The received identity is recorded in `target/bond-history-menu-green-port.json`.
The six tests ran unchanged in the isolated owner workspace, session 86383:

| Check | Actual result | Owner log under `target/` |
|---|---|---|
| `cargo test --locked -p bondline-ui --lib bond_history_tests:: -- --nocapture` | 6 passed, 0 failed, 0 ignored | `bond-history-six-final-green.log` |
| `cargo clippy --locked -p bondline-ui --all-targets -- -D warnings` | Exit 0 | `bond-history-six-final-clippy.log` |
| `cargo fmt --all -- --check` | Exit 0 | `bond-history-six-final-fmt.log` |

Both source-free and saved/reopened native CDXML numeric-history variants now complete.
The exact original native source returns after Undo, then a fresh positional event permits
the next edit. Both keyboard and real menu-click ordering cases retain the complete prior
owner. Dynamic Undo/Redo availability, fresh restored-page coordinates and whole-SDF
history also meet the unchanged assertions. Original five-test bytes are still an exact
prefix of the additive six-test file; no fixture, oracle, golden or threshold was changed.

The bounded read-only review found no additional confirmed production defect. Indexed
Edit/Page actions retain a control ID and event index, resolve current enabled commands
at that event, and share the existing whole-owner/page/viewport/scene refresh. Non-drawing
and invalid-viewport fallbacks retain keyboard history; the former also receives indexed
menu intent. Close/file blocking, text focus and positional-context guards remain. The
common activation predicate is the former tool predicate extracted without semantic
changes. This is source review of Page and accessibility menu paths, not runtime coverage
of every such activation or several distinct menu actions within one packet.

Root's first menu reproduction used a wrong test selector and ran **zero tests**; its
`target/bond-history-menu-root-red.log` is not RED or GREEN evidence. Root's corrected
`target/bond-history-menu-root-actual-red.log` ran the named case and failed exact owner
equality, independently reproducing this owner's session 50805. These Root records stay
separate from the owner logs above.

Another owner's toolbar-focus test had a separate name-only accessibility locator. Root
reports its runtime probe established that the selected ID was a stale, unreachable Label,
while the current Button supports Focus. Correcting that locator preserves the actual
focus/state assertions; it is not a production fix or a change to these six tests. This
owner inspected the locator source but did not run that probe or its corrected case.

## Public policy and limits

[The public 2025 guide](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf),
printed pages 72–73, 85–86 and 88, supplies the absolute bond-order hotspot roles and the
bounded click/drag distinctions. Page 99 separates H rectification from label display.
Its pinned SHA-256 is
`3f1ebc429df07efdd47b6218a148dc346a215043f60bf89ee9a1cb90629bd5b3`.
This remains version-25 documentation, not observed version-26.1 input behavior.
[Pinned OpenSMILES](https://github.com/opensmiles/OpenSMILES/blob/9240a2887158699503605bc838e1de9a9f7883f4/opensmiles.asciidoc#L275-L312)
supplies neutral carbon's valence four. The registry's required
`rectify_neutral_carbon` policy changes proved literal authored H counts; this test slice
adds no general H perception, preference default or foreign-source edit authority.

Ordering, fresh-context invalidation and exact history restoration are original Bondline
safety contracts. Clicks requesting missing Double alignment refuse; an explicit same-order
setter retains its validated no-op policy. Native OS input, accessibility menu activation
and visual parity remain separate gates. No new approved image or chemistry engine is claimed.

The 171-input baseline ledger has 169 exact preserved entries and two explicitly received
Root replacements: UI production and the separately owned order tests. All fixtures,
oracles, manifests and locks in that ledger are exact. This worker changed no assets. The final
handoff includes those identities and every actual local log. This worker owns only the
new history test file and this verification record. All jobs are terminal; Root owns Git,
whole-workspace integration, global status and native evidence. Actions stays disabled,
and the complete M0–M5 goal remains active.
