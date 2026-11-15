# Independent restricted SMILES conversion App acceptance

The six cases in `crates/ui/src/native_conversion_tests.rs` use the actual
`BondlineApp::update_workspace`, accessible controls, shared command engine and file
workflow. They start from `ac11ce6`. They are App input-frame and actual-file checks,
not native OS input/dialog, clipboard, Windows/Linux or complete reference parity.

The original conversion contract is pinned in the ignored receipt: 5,125 bytes, SHA256
`481cae901288e27198cd56b419cb140c332ed0f56f59655701ff7567f2cb45c8`.
It preserves the existing 30-command registry and closed eight-parameter
`file.convert_smiles_to_cdxml`, with `neutral_carbon_path_v1` and create-only CDXML.
The native presentation action and specialized registered-work bridge are adapters,
not a new format, parser, chemistry engine, overwrite mode or Save/close acknowledgment.

## Source and independent expectations

The [existing converter](verification-smiles-conversion-io.md),
[shared command](verification-smiles-conversion-command.md) and
[layout](verification-smiles-layout.md) retain the public OpenSMILES/IUPAC basis,
original geometry policies, independent expectations and supported subset. This UI
does not expand them. Its form, dialog labels and default 48 pt length, (72,72) pt
origin and A4 page are original Bondline policy, not reference-product defaults or
general SMILES import/Paste Special parity.

Input is an original `CC\n` file at a Unicode path. The default output must have
two carbon atoms at (72,72)/(120,72), literal H3/H3 and one Single bond. A custom run
sets all five numeric fields explicitly to length 40, origin (100,200), page (612,792).
The unchanged external conversion/Expat/Decimal/RDKit consumers inspect the actual
created files, including literal custom coordinates (100,200)/(140,200). No local
writer generates expected bytes or replaces the independent consumers.

Before conversion, the current native CDXML drawing is opened, then edited to Double,
leaving a dirty source-backed document, exact original file and undo/saved baseline.
Conversion must retain the complete current owner, its CaptureStamp, file/source data,
dirty state and history. Only the real presentation request/epoch is normalized when
comparing state. Undo/Redo after conversion verifies the saved baseline still exists.

## Five initial workflows and the additive cancellation regression

1. Actual File → `Convert SMILES file…` is reachable/enabled and opens
   `Convert SMILES file`, preserving the current drawing. The menu wait accounts for
   the pinned SDK's invisible sizing pass without a fixed settling delay.
2. Browse Unicode input/output through `Choose SMILES input`/`Choose new CDXML output`,
   using path selection only while the current drawing remains dirty. Verify defaults,
   default/custom geometry, successful real conversion, actual output path/counts and
   unchanged current/source/saved/history/input. Close the result before ordinary
   File/Open. Explicit `Discard and Open` owns the ordinary dirty replacement decision;
   edit the created bond through the actual canvas, Undo, Save and reopen it again.
3. Unsupported `CC=O`, an already-existing destination and nonfinite numeric text refuse
   without output/overwrite/current-state mutation. Each explicit retry corrects its
   own input/path/number and must create a new independently consumed file. These are
   deliberately pre-publication refusals; a generic failure heading is not proof that
   an arbitrary post-publication error created no file.
4. Begin with an actually hovered bond. Numeric key/text entry `123` belongs to the
   form, preserving both model and transient tool. Convert/Cancel/Close followed by
   pointer/key input in the same packet must not author underneath. The next fresh
   canvas packet must work and create a literal Single/H3 fragment. Real job retirement
   and late cancellation/publication are separately gated by producer tests.
5. Cancel before submitting creates no output. Source replacement or remote close
   before an old captured submit action must invalidate the unstarted request, dismiss
   its obsolete form/focus and preserve the replacement/close context. It cannot silently
   retarget or publish. This does not discard running/published work or historical truth.

6. After a real conversion form was painted, cancel its accepted `ui.file` request
   externally through the shared registry, without changing source or generation. Feed
   the old submit action plus a coherent canvas drag into the next frame: no conversion,
   output or authoring may occur. A fresh next-frame drag must author the expected
   Single/H3 at literal positions. This candidate passed on its first run (3958); it
   required no product fix and is not counted as another RED.

The interface labels/headings and exact actions were frozen before receiving product
code. All eight command fields remain explicit; the registry schema is still closed
and has eight required parameters. This slice adds no dependency, codec policy, fixture,
oracle, font, approved golden, App field or separate worker pool.

## Declared REDs and reviewed test changes

The minimal 2,067-byte source is SHA256
`f4c3f2c9992c40d9cb56f0fbff48b61ea09d97aa8d89e99a448ecaba9be19650`.
Actual run 86758 compiled and failed at the missing File control, with no future Rust
API involved. Log: 7,827 bytes, SHA256
`624428d4b5857fed869b66f46464defc5fe9eae9a4be647ae847a74b92dc15b3`.
Expanded baseline 17502 compiled all five cases and failed at the unavailable controls
(one immediate missing role; four bounded missing-form waits).

The expanded 25,518-byte source, SHA256
`bdf0f6b1398881b9a4aa1696589f8710c059c6e3f72fd2700d436dce8136b8d0`,
received only Root-approved additive tool equality/Single/H3 assertions before product
receipt (26,274/`06c935c4…`). An unused helper import then caused warnings-denied clippy
28230 to fail; import-only cleanup and rustfmt produced 26,224 bytes, SHA256
`596b0597908d714de96ac30133c05a6d8f7f6dbb42f8d432c77b8bdf09221746`.
The exact inverse reconstructs both prior versions; all original assertions/inputs and
the complete 2,067-byte prefix remain intact. Clean pre-receipt clippy 50575 and fmt
87029 pass. This is strengthening/lint cleanup, not a behavior relaxation.

The first exact real-source run 9075 passed the menu/form case and stopped four cases
at same-frame InlineTextBox readback after accessible SetValue. Root-approved
metadata-only probe 65111 proved that the parent accessible value was immediately
exact while the painted child galley followed on the next frame, also in the unchanged
old file chooser. The probe then passed four workflows and exposed retained obsolete
form behavior in the fifth. Its 9,738-byte log is SHA256
`720e958693e0dd7573d58bcd2bc7036efd932f4fbd96d2f433022f0da67e32c1`.

The permanent new helper correction asserts the exact parent value immediately, uses
the same finite bound to await painted value, then retains the original exact child
assertion. It adds no input/action/state/history/graph change or longer timeout. Its
26,514-byte source is SHA256
`9b9af85d528691a338697554fdd7591d4a8da6d6dca1ac4c0b2fe896f60c20de`.
Removing just that added block reconstructs 26,224 bytes. The temporary probe and
original first failures remain archived. The producer's new-field repaint/order fix
is separate from this corrected snapshot oracle.

Root explicitly retained the fifth case's obsolete-form dismissal requirement. The
unstarted stale-form failure is a product behavior difference, not a test expectation
removed to accept a retained-disabled modal. Running/published result retention has a
different contract and separate finite evidence.

## Independent receiving and final checks

```sh
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo +1.94.1 test -p bondline-ui --lib view_tests::native_conversion --locked
```

Select the literal pinned interpreter through `BONDLINE_PYTHON` and a checkout-specific
absolute `CARGO_TARGET_DIR`. The first real UI/backend manifests and all 183 original
protected inputs were hashed before receiving; 175 outside explicit replacements remain
exact. The protected archive, source inverses, first REDs and actual consumer output are
kept under ignored `target/native-conversion-ui/`.

Independent run 67361 passes the unchanged five on the exact fixed manifest
`19c2c5956a520d98810b736ebb058cd5591a0ddd6f9fc58a3ce74f4ff551dcf1`.
The additional cancellation case then passes its first run 3958. The complete six-case
source is 30,232 bytes, SHA256
`7959533aa76f0400c59ce614b40544dd61a989b6602df6f2f6ceaa205bfb3113`;
its entire 26,514-byte five-case prefix is unchanged.

Root separately observed successful retry retaining the prior global error (51446).
The approved sole additional product change clears that message only after an explicit,
valid Convert request has actually queued. Its exact 200-byte source delta was verified
before receipt: `file_workflow/smiles_conversion.rs`, 26,728 bytes, SHA256
`1dc87cf366f8ef892b038060ba95b60e700ecfeb7fdf89bbf010a865c418cd46`.
The delta manifest is 1,581 bytes, SHA256
`b5f33adfda874f3c0c4af1b1ce9a7a75e46f483546ebfc349e6a85743871f2f5`.
Root/producer's separate recovery and finite worker tests are supplied evidence, not
executions by this independent worker. Their child test bridge was not needed here.

Final independent execution on that exact product/source combination:

- Run 29790: unchanged six App cases, **6 passed / 0 failed / 0 ignored**. The actual
  generated default/custom files pass the unchanged external consumers during the run.
- Run 62567: `cargo +1.94.1 clippy -p bondline-ui --all-targets --locked -- -D warnings`,
  exit 0. The earlier unused-import failure remains separately archived.
- `cargo +1.94.1 fmt -p bondline-ui -- --check`, exit 0. No source was reformatted during
  final receiving. All jobs are terminal.

All commands use this checkout's absolute target, incremental disabled, dev/test debug
info disabled, and the literal pinned `BONDLINE_PYTHON` interpreter. The final ignored
handoff binds the owned files, received production, protected 175 originals, source
inverses, all first failures/probe and final logs by SHA256. The eight explicitly
received replacements are classified separately from unchanged originals; the archive
preserves all 183 pre-port protected inputs.

These results establish the bounded App/actual-file workflows above. No full UI package,
workspace gate, native OS input/dialog/clipboard, hosted or interactive Windows/Linux
execution was performed by this independent worker. Root owns integration/native checks
and Git. No dependency, lock, asset, font, approved golden, old fixture, oracle or old
assertion was changed here.
