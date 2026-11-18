# Existing-bond order: independent application-frame acceptance

2026-11-14–15. Work in progress. Six tests are frozen; baseline behavior and
stationary-hotspot failures are preserved, and two narrow setup corrections are
separate from product fixes. All six workflows pass on the final real
implementation, including every14-context guard and external saved-file check.
UI all-target clippy and formatting pass. Root owns production, integration
and Git. This worker owns only the new `crates/ui/src/bond_order_tests.rs` and this
record. No native window/input, activation, pointer polling or hosted workflow is
used; the user's existing preview process is untouched.

## Public contract and original safeguards

The pinned public vendor-authored [GUIDE25](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf)
is17,721,447bytes, SHA-256
`3f1ebc429df07efdd47b6218a148dc346a215043f60bf89ee9a1cb90629bd5b3`.
Printed85 documents Single-tool conversion of an existing Single to Double;
further clicks concern Double placement. Printed88 documents selecting a bond
tool to convert an existing Single, with85–86 defining the Double/Triple tools.
Printed72–73 document absolute bond-hotspot keys1/2/3. Printed86 separately
documents endpoint-to-endpoint dragging across a Double to make a Triple.
These are version25 public facts, not observed26.1 behavior or assets.

The agreed first click map is Single+Single→Double, Double+Single→Double,
Triple+Single→Triple. Single/Double tools on an existing Double must show the
unsupported alignment gap. Other unproven old-order×tool combinations remain
visible refusals. No1→2→3 click cycle, invented alignment cycle or generic
Shift-click behavior is asserted. The guide's SHIFT+Click row on82 continues
with a saturated-double-ring-tool qualification on83.

Plain nonrepeat numeric presses, no modified shortcut capture, actual TextEdit
exclusion, fresh ordered hits, gesture/context/close/source checks and bounded
literal neutral-carbon H correction are original safety policies. The guide
does not define those event-level details. Button focus must remain compatible
with a hotspot operation; egui's `wants_keyboard_input` alone means any focus,
including an actual palette button. Pinned egui0.29.1 provides AccessKit Focus
processing and public `TextEdit::load_state` separately.

## Initial fixed tests

The actual `BondlineApp::update_workspace` runs in deterministic egui frames.
Fixtures use ordinary registry operations and no new production test hook.
Controls are discovered from actual AccessKit names/bounds. Independent expected
JSON retains every ID, position, unrelated atom/bond and page property, changing
only the specified existing bond and the two declared literal H values.

1. Actual Single/Double/Triple palette clicks convert nonfirst Bond2 of an
   original three-carbon vertical/horizontal graph. Expected H is[3,1,2] for
   Double or[3,0,1] for Triple. One case starts from actual saved/reopened native
   CDXML. Each conversion creates one Undo item, updates actual visible component
   vectors, keeps selected mode/epoch, and survives Save/Undo/Redo/reopen. Actual
   saved bytes face the unchanged Expat/Decimal oracle and pinned external
   RDKit2026.09.2 for topology, literal H and formula.
2. The real Triple palette button receives an actual SDK AccessKit Focus action;
   the test checks focus before numeric input. A previous-frame pointer on bondA
   followed by Key2→MoveB→Key3 in one event packet must convert A then B using
   current graph/revision, preserving selected mode. Absolute1 restores Single;
   a fresh same-order operation preserves ordinary state.

Initial two-test source is16,749bytes, SHA-256
`9b7eb71d76b5a2db5c8847f01d52d6534985c48c393e8e02b941a09e4ec79184`.
The initial722-file source identities, freeze and exact prefix copy are ignored
under `target/bond-order-ui-*`.

## Additive cases and observed RED

Three tests were appended without changing that16,749-byte prefix:

3. Fourteen distinct hotspot safety contexts each retain a later fresh positive
   control, so a disabled key handler cannot satisfy the test. Contexts include
   no fresh point, Escape, focus loss/return, pointer loss, tool A→B→A epoch,
   revision, DPI, host origin, foreground coverage, atom priority, key-up/repeat/
   modifiers, close, actual TextEdit character delivery, and a held gesture over
   another bond. The actual editor must receive Text2 while the document stays
   unchanged; button focus remains allowed by case2.
4. All six unsupported click combinations preserve complete state and display
   an actual clipped-in diagnostic; the two known Double alignment gaps must
   identify alignment. A separate actual original MOL Open remains read-only
   under click/key attempts and saves its complete original bytes exactly.
5. With Single selected, endpoint-to-endpoint dragging across a Double changes
   that existing edge to Triple in both directions. All IDs, coordinates,
   unrelated edges and literal H remain independently specified. Each operation
   produces one Undo item and survives Undo/Redo.

Frozen five-test source is27,816bytes, SHA-256
`a20b095f2de92f4b75c5267e943f4674ec5338f1a5415f1b247af3286e6ed331`.
No new production Rust API or placeholder was required. Root added only the
test-module bridge for these baseline runs: UIlib SHA-256
`a6233cbda9c17c93f889b80ae1c7c88afb09374738fac2ab14c0faece934d6c3`;
removing that bridge reproduces exact baseline source
`98d0dd17b413a34b248a501f347372260634e8a94a333eb7d61ff8752965bca2`.

Run from this worktree, with its absolute `CARGO_TARGET_DIR`, incremental0,
development/test debug0, four build jobs and the existing pinned
`BONDLINE_PYTHON`:

```bash
cargo test -p bondline-ui --lib bond_order_tests --locked
```

| Run | Actual observation |
|---|---|
| Initial two, session81797 | Exit101,0pass/2fail/72filtered. Actual Single-tool click leaves Bond2Single/H[3,2,3], rather than Double/H[3,1,2]. Ordered numeric2/3 leaves both pairs Single/H3 rather than Double/H2 and Triple/H1. Actual AccessKit button-focus setup passed before those assertions. Root separately observed the same two runtime failures in7597. |
| Frozen five, session36707 | Exit101,0pass/5fail/72filtered. Initial failures persist. Unsupported Double+Single click has no diagnostic; first no-fresh guard correctly does nothing, but its required fresh positive rearm also does nothing; forward Double endpoint drag remains Double. Later loop iterations, reverse drag and MOL/refusal tail have not yet been reached and are not claimed exercised. |

Ignored logs:

| Artifact | Bytes / SHA-256 |
|---|---|
| `target/bond-order-ui-initial-red.log` | 13108 / `7e6d9920c7747158331e3027f1263e0ccf08b569a323ebfbcc5da498524597d6` |
| `target/bond-order-ui-five-red.log` | 12931 / `5c59ba39aa01b0b249e848283f688d1ba5a340ea1d1dfc91c6d70a425aa38021` |

No GREEN, full-package or lint claim applied to that baseline checkpoint.

## Stationary logical hotspot: separate product RED

A sixth test was appended with the entire27,816-byte five-test prefix unchanged.
That initial six-test source is32,443bytes, SHA-256
`bec3d858815cf0c8b07ecbd9195bdd1b0aafaf8944f02995bb63001f42c11d12`.
It enters actual Single ink once, then sends2→3→1 without another pointer move,
both across frames and within one event packet. It also enters an actual outer
Triple component and sends1→2. Each actual edit must retain the same IDs,
positions and unrelated fragments, correct both literal H values, create one
history item and preserve the selected mode. Fixture stroke width exceeds the
five-unit hit tolerance, so a changed Double's white center gap cannot conceal
target-loss behind a generous tolerance. External context invalidation still
belongs to the unchanged guard test.

Root's exact three-file current-draft port is recorded in
`target/bond-order-stationary-draft-port.json`,872bytes/SHA-256
`c109c262cbfbe4e08e628ee5881be1aa0373fb35ce2fed7ccd7409bb7b303f64`.
All three actual source hashes were checked before execution. The original six
were untouched when session65118 ran only the stationary test: exit101,
0pass/1fail/77filtered. The first2 produced Double/H2 correctly; the next
stationary3 remained Double/H2 rather than Triple/H1. Subsequent packet/outer
Triple iterations were not reached. This is the observed product failure,
separate from the setup corrections below. The4018-byte log has SHA-256
`00a7e2ecc4baddc38c924dea79dcb1de74435672802b64a38332ea358d40ca39`.
Root separately reported first-two GREEN33321 and a five-test draft run87259
with2pass/3fail, including endpoint/diagnostic behavior and the repeat setup.
Those are receiving-side observations, not local six-test GREEN.

## Two source-verified harness corrections

The pinned egui0.29.1 `src/input_state/mod.rs`,303–311, overwrites each pressed
key's repeat flag using the existing held-key set. Lines430–431 expose that
normalized vector through both `events` and `raw`. The pinned egui-winit0.29.1
`src/lib.rs`,777–782, always submits repeat:false for egui to determine. Actual
local primary-source hashes:

| Pinned source | Bytes / SHA-256 |
|---|---|
| egui0.29.1 `src/input_state/mod.rs` | 52771 / `d86068de276eaf59962e3ff6b2938692531d05ec11a4556ade606a9b5776fcf1` |
| egui-winit0.29.1 `src/lib.rs` | 73260 / `1a0ad59f3a3dd7e679a2fc16456a0cc55f985e1c70a8ffac74def9b02729a4cc` |

An ignored2092-byte Rust probe, SHA-256
`823d132b62a228b4033b5e653470d0a71cc64c8e5c61ca08d59837d610465a8e`,
links only the already built pinned egui rlib using rustc1.94.1. It creates real
headless Context inputs and exits0. Its actual results for `events`, `raw` and
held state are:

| Input | Observed pressed repeat flags / final held state |
|---|---|
| Original unmatched key-up, rawrepeat:true press, key-up | `[false]` / `[false]` / false |
| First press off-hotspot | `[false]` / `[false]` / true |
| Move while still held | `[]` / `[]` / true |
| Another rawrepeat:false press, then release | `[true]` / `[true]` / false |

The original `non_plain` input was therefore a first press to the application,
not an observable repeat. Only its setup changed: an actual first press occurs
off-hotspot, moves onto the bond while still held, and then sends the repeat
and release. It retains an unmatched key-up and all four modifier pairs. New
evidence assertions verify off-hotspot setup made no edit or mode selection,
the key is actually held, and real normalized pressed flags are exactly
`[true,false,false,false,false]`. All existing14 no-edit and later positive-rearm
assertions remain unchanged. The probe's241-byte log has SHA-256
`f0eea8161116ab1ea2d0ab0d657032632df481fbc728e035f1cf4902f6fec0bf`.

That repeat-only corrected source was34,087bytes/SHA-256
`e6da8aadcbad37652306d0480c8295f17bdddb318d5cd69b0fe9e97445715da3`.
Filtered guard run80160 advanced through all11 cases including repeat and its
positive control, then failed on previously unreached close setup: the unchanged
`CloseDocument` requires `expected_revision`. The885-byte error log has SHA-256
`ad6591f576876711e86e6eb13d746b06b04203ddd71c37d5a0e52db3101433a3`.
Only current expected_revision was added to the existing request/cancel setup
calls. No close or other behavioral assertion changed. Neither setup mismatch
is reported as a product bug or product fix.

## Prior corrected-source freeze

Corrected six-test source is34,279bytes, SHA-256
`d330f663894328d6fdb40331e90c672be12cef0ed986d41f764be7ab3e3d0dce`.
At that checkpoint the entire initial16,749-byte two-test prefix was exact. The
later semantic locator correction below is the additional narrow exception.
The stationary sixth function remains exact relative to the original32,443-byte source. Both
original five/six, intermediate repeat-only source, original verification
record and exact setup diff remain ignored under `target/bond-order-ui-*`.
The3465-byte setup diff has SHA-256
`f149e3c912cf7f9a684e6eada3fd7924da7ee86c37e5c37a2206fcd83ae6ea75`.

Filtered guard run59976 exits0,1pass/77filtered. All14 negative contexts and
every fresh positive rearm complete, including actual editor text delivery and
held gesture suppression. Its486-byte log has SHA-256
`95987d85dcdd3726afbe309880581a0b33cd4c54cd166bd85e3a35335f6bbf1c`.
That filtered run does not establish complete six-test or native-window GREEN.

## First receiving GREEN (c14)

Root supplied four exact implementation/history-test inputs through
`target/bond-order-ui-final-port.json`,760bytes/SHA-256
`364c34dbd010123d68062e5abc8c45c1ddd696760da91a415a5d7a621f4f11e5`.
All four actual hashes and the unchanged corrected six-test source were checked
before execution. The UI input is SHA-256
`c14a383a6c648040afefa69f0640abc7f1fae980a82362da628bd72358567021`.
The final six retain the same assertions as their recorded RED/setup sequence;
no expectation, threshold, font, old fixture or external oracle was weakened.

| Command / run | Actual outcome |
|---|---|
| `cargo test -p bondline-ui --lib bond_order_tests --locked`; session2187 | Exit0,6pass/0fail/77filtered. All three click choices, native-CDXML variant, Expat/RDKit saved files, Undo/Redo/reopen, ordered absolute keys/button focus,14 guards with positive rearm, six visible unsupported combinations/MOL exact Save, forward/reverse Double drag, and all four stationary-pointer sequences complete. |
| `cargo clippy -p bondline-ui --all-targets --locked -- -D warnings`; session43839 | Exit0, no warnings; own absolute target and pinned environment. |
| `cargo fmt -p bondline-ui -- --check` | Exit0, no source mutation. |

The1305-byte six-suite log, `target/bond-order-ui-six-first-green.log`, has
SHA-256`477462a389b67e20109bbc720c93e155e0bbea965bcd26575a1aefa871493f34`.
The5952-byte clippy log has SHA-256
`53b57b2275d76f4c6704ba5e72d20940f5780fad6dfdb4b302f31de27313972d`.
The format log is empty/SHA-256
`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
Root separately reported whole UI130pass/0fail/1ignored in54148 and five history
tests GREEN82150; this worker did not repeat those receiving-side full suites.
The complete workspace gate remains Root's responsibility.

At that receiving checkpoint,719 of the original722 files were exact. The three changed
originals and one new history-test file are exact Root imports; the other two
new files are this worker's test and this record. No baseline path is missing or
new path unclassified. Existing six multiple-bond tests, all old fixtures,
oracles, fonts, notices, locks and approved Single golden remain exact. The
ignored final handoff lists source/import/protected/log/probe/setup identities.
All jobs are terminal. This is real headless App/frame and independent-file
evidence, not native-window acceptance or completed M0–M5. No native interaction,
Git mutation, hosted workflow or new golden approval occurred.

## Menu-order integration: semantic focus locator

Root's later menu-order source,73,635bytes/SHA-256
`fd39fe34002841c7e1835b3521b6fb97640b4e2170f82c83cbeb59dc6cb502d0`,
exposed an ambiguous name-only test locator. Root's library run had82pass/1fail/
1ignored; a targeted replay also failed at the unchanged exact-focus assertion:
requested Node387546315756561404, actual focus Node13826925289788388. Copied
failure logs are retained as9887bytes/SHA-256
`6747fc8adc0f7474b6fe1ce3804ec61befa91762f250076f5c62779282fc775a`
and1073bytes/SHA-256
`99b058c2b8058183946895a3be519ca49a96d945a6d966c8c6309314cf30e2f1`.
Static comparison found equivalent palette activation logic; no production
workaround was applied.

An isolated ignored export snapshotted730 exact Root source inputs. A diagnostic
was appended only to its copied test file; Root and the owned frozen six stayed
unchanged during diagnosis. Session57000 identified these actual cached nodes:

| Node ID | Actual role / name | Focus / current-tree reachability |
|---|---|---|
| 387546315756561404 | Label / Triple bond | No Focus action; unreachable stale tooltip label, bounds(9,159)–(72,176). |
| 13826925289788388 | Window / unnamed | Current accessibility root; no focused widget. |
| 3475799789266076766 | Button / Triple bond | Focus action; reachable current control, bounds(3,121)–(31,149). |

The original name-only HashMap lookup chose the real button in that diagnostic
run and focusing it succeeded. This exposes arbitrary selection among cached
same-name candidates, rather than a failed menu/focus workflow. The real button
reports Default-action support:false, so the correction must not require that
unadvertised action. The9161-byte probe log has SHA-256
`dfae792462be4eeb7d2bfa56d27d2cb525c140dd4f9bfa3de4346d5916c80bcf`.

Only `Fixture::node`'s predicate was corrected: exact name, actual Role::Button,
and supported Focus action. This targets the real semantic control. No test
assertion, raw input sequence, model expectation or threshold changed. The
entire file before `node` and after `choose` is byte-for-byte identical to the
preceding d330 source; that proves all six test bodies and other helpers remain
exact. Original d330 source and0c9b verification record are preserved under
`target/*before-focus-locator*`. The old initial-prefix equality claim therefore
belongs to the preceding freeze, while this explicit helper exception applies
to the current source.

Isolated fd39 verification39607 passes all six acceptance tests, with only the
additional diagnostic skipped. Root then supplied the exact fd39 UI and its
additive menu/history tests through446-byte
`target/bond-order-ui-menu-port.json`, SHA-256
`fc002c2461aa179e0b522923eb2d80adef94d8c22ea84464354ed70e0b5f35fc`.
Actual owned-worktree run78458 passes6/0/0,78filtered; all-target UI clippy99797
and package fmt also exit0. The corrected six are34,440bytes, SHA-256
`81d986b87a2423d0ef23f3119adc1d51c51d9604efd5ebb1cc2cad020e044272`.

| Latest ignored log | Bytes / SHA-256 |
|---|---|
| `target/bond-order-focus-isolated-six-green.log` | 1198 / `12e9adaf010c3bdbb8c0dcf7852aefccb5fcc86c1bc7a9dece5d54d7d5ea8758` |
| `target/bond-order-ui-menu-six-green.log` | 1061 / `5a12dbf683687afe96ea3f787c6d0c78d15a4b015f6b2233683a8d3f7320ae28` |
| `target/bond-order-ui-menu-clippy.log` | 175 / `541e515406d1e48476026967ca83c65d112309e5a785d3404b08ad1e480d23f1` |
| `target/bond-order-ui-menu-fmt.log` | 0 / `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

The separate menu handoff preserves the preceding handoff and every failure/
probe/source receipt. All four latest Root input identities are checked after
the gates;719 protected originals, old fixtures/oracles/fonts/locks/golden
remain exact. All jobs are terminal. Root owns full-workspace/native acceptance
and Git integration; this investigation used no native input or production edit.
