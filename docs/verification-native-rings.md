# Native Single-ring integration — receiving

2026-11-16. Work follows the locked [native contract](research/ring-native-plan.md)
and existing [ring command evidence](verification-ring-authoring.md). This record is
in progress; it does not establish native OS interaction, full ring parity or M2.
The desktop stays closed while the user uses it. GitHub Actions remains disabled.

## Initial independent RED and first draft

Root alone runs Cargo, changes live files and uses Git. The three workers declare
tests under ignored `target/ring-native/`; they do not execute Cargo or native input.
All commands use the pinned Rust1.94.1, incremental off, two build jobs, debug info off,
and the literal existing `.cache/oracles/format-python/bin/python` interpreter path.

On b6b1775's registry37, before product edits:

- Core12140: JSON-only `ui.ring_tool` fails0/1 with actual UnknownCommand. Frozen
  declaration2054B, SHA256`68619850da9b9cb43b3f9b8b61f562f73e3240fe3684f221ef486697853a649c`.
- App64310: actual App-frame test fails0/1: current actionable Cyclohexane control
  missing. It accepts the existing Button/Checkbox accessibility roles, without
  assuming a full visible label in the compact rail. Frozen1813B,
  SHA256`b01cc5f87556c39f23c1c89c2138ee6c1fd032e71c023f76e7638f18be0bb27f`.
- CLI80533: actual executable returns unknown_command;0/1. Frozen1400B,
  SHA256`4f6b308d1b26312d09f4fc51d056ca01fef336fa4e7193edfba3e4f903cf5915`.
- Separate API compilation fails on missing EffectiveTool, SelectRingTool,
  UiRingToolSelect, effective_tool, ring_context and preview_ring_create. This is
  missing-API evidence, distinct from runtime RED. Frozen2032B,
  SHA256`bf7ff86b534b29699a65720c3449dc24c9abf52b457c7f0ed8eb565ce260c5f1`.

The first shared mode/context/preview/controls/publication draft follows those REDs.
Core2716 passes both initial cases; App45300 passes the initial actionable/toggled
button case. No canvas ring gesture is implemented by that draft. These are focused
passes, not an aggregate or native workflow result.

The independent expanded JSON declaration arrives after that first draft. Root parks
six tracked production drafts, restores actual HEAD sources for receiving, and restores
the drafts byte-exact afterward. Expanded v1 uses nonexistent accept_ui_file and an
unused import;67173 and37715 are setup compilation failures, never runtime RED/GREEN.
Worker v2 changes only that call to the existing claim_ui_file and removes the unused
import; all assertions/inputs and the2054B initial prefix remain exact. v2 is15642B,
SHA256`790ed267f775b288eebc5dd8f5e05816837dab823fe6f2d34dd7674d92751ae0`.
Parked-HEAD96575 reaches actual0/4: three UnknownCommand and registry37/38 mismatch.
The already existing first draft is restored unchanged. This chronology is not a claim
that the expanded suite preceded the first draft.

Logs, declarations, draft snapshots and the pre-edit1064-file hash ledger are ignored
under `target/ring-native/`. The existing approved golden, source fixtures, consumers,
parser/chemistry/render code and dependency/asset policy remain authoritative.

## Acceptance boundary

The focused checks below do not replace the final local quality gate. Actual native
palette, drawing, history and Unicode persistence still require execution when the
desktop is available. Passing App frames or a Metal offscreen capture cannot substitute
for that native OS workflow or measured DPI/other-platform execution.

## Expanded shared and initial native receiving

Core88712 passes JSON4 plus initial API1 unchanged. Expanded context73727 passes2/1;
its final Running-Save setup reconstructs a logical fixture path instead of using the
accepted canonical SaveExisting params. Existing file_ui correctly refuses that selection.
Worker v2 reuses the claimed SaveExisting params directly; every source/phase/bytes/
no-ack assertion remains. v2 is15926B, SHA256
`b2b4022a68634135dd6ce73bcc7200c15b6c922ddbc983cb4ef3bcb595f73eb8`.
Core60240 then passes JSON4/context3. Private4722 passes both checked-overflow cases.
No file binder, epoch error, Save reservation or selection assertion is weakened.

The adapter expansion19038B SHA256
`4371691a6e186cc698e259c11adb7fe95cd3870930aafe6e6dac224e40e3bef8`
retains its initial1400B prefix. Parked-HEAD90218 reaches actual0/4: two UnknownCommand
and two37/38 discovery failures; all six draft sources restore byte-exact. Restored
64619 passes4/0, including initialized stdio and authenticated socket publication ACK/
activity release before changed-only mode/size notification. Old deadlines, listener
policy, control tests, admission bounds and generic consumers remain unchanged.

The three first actual-App gesture cases are frozen before RingWorkflow/events:
6300B SHA256`365f20d1902d59209f7f2a611fdc10c87ab31eae58ab35a42207ff64c08b187c`.
App53575 reaches1/3: the palette passes; Free has0/1 fragment, shared terminal atom2/7
atoms, shared bond2/6 atoms. The first gesture draft then makes7002 pass4/0 unchanged.
It uses actual graph hits, exact authored targets, one shared builder and complete
candidate rendering before each native release, without a converter/file fallback.

Four additional preview/attachment cases follow that gesture draft. App2529 passes6/2:
all literal graph/paint/anchor/H checks pass, but two saved-file comparisons fail in
its Rust JSON test bridge by one ULP. Root qualifies this independently in ignored
float-bridge/: actual CLI Save and unchanged Expat/Decimal consumer compare exactly
in external Python. A standalone probe links the already compiled default serde_json:
95.99999999999997 has std f64 bits4057fffffffffffe versus JSON4057fffffffffffd;
119.99999999999999 has std405dffffffffffff versus JSON405e000000000000.
This is not a CDXML writer failure or a reason to alter a shipped dependency/consumer.

Worker v2 moves the exact full-dictionary comparison into the same pinned external
Python process before that approximate JSON bridge, then retains the RDKit assertions.
No numeric assertion/tolerance/model/input/paint/history expectation is relaxed and
no existing consumer changes. The suffix16168B has SHA256
`3283ee21b6a44a0758e74b6576f055314a3e857a2687db7fc976f3fab6799cfc`;
its predecessor15667B/f1d431ec remains archived with the exact setup diff. App40403
passes8/0, including complete held-candidate glyph/stroke geometry, free/atom/fused/
spiro construction, both bond-drag sides, actual Unicode Save/history/reopen and exact
Expat/Decimal plus pinned RDKit chemistry. Logical6-em new labels refuse despite
physical12-em at2×;2×/4× candidates pass actual vector geometry, including width>2048.
This is App vector evidence, not native measured DPI or a new approved golden.

## Source-review regressions

The independent reviewer freezes exactly two actual-App regressions before either
narrow fix:6887B SHA256
`b373b41d7836b386fe51f47a3fab54e705b87b372dba399822de791c4be43392`.
App91509 reaches actual0/2. A legal leftward50pt triangle from99975/500 on a100000pt
page at99% cannot start because the press prematurely validates its x+48 click default.
A valid held ring on a60000pt page survives the real1×→2× invalid-viewport refusal→1×
path and incorrectly commits on the old release. Both failures are product defects.

Only after RED, default endpoints remain raw finite coordinates until the existing
candidate builder validates them; a genuine drag can replace an invalid default. No
out-of-domain click becomes valid. Existing invalidated-surface/early-return boundaries
now also cancel disposable Ring ownership. No physical/logical extent, point, font,
label, paper/ink or geometry limit is weakened. App29231 passes both regressions with
their full state/source/Arc/bytes and subsequent fresh-gesture positive controls exact.
One duplicate cancel is removed afterward without changing those assertions.

## Final App input declaration and setup qualifications

The original owned15-case declaration42819B/89be4622 plus the independent6887B reviewer
suffix runs as17 cases in71866:16 pass, one cancellation-loop setup refuses AlreadyExists.
Its B→A direct Save omitted the established explicit overwrite flag. Worker v2 adds
only overwrite:true; no file policy/assertion changes. Single-case42942 then reaches
"explicit camera" during its fresh-gesture recovery. That log has no printed loop
branch trace; no exact historical branch attribution is claimed. Source independently
shows retained-record import resets to Fit and Open preserves it. The helper requires
an explicit zoom. v3 restores Actual only after old-release state/source/count assertions
and source restoration, before the fresh recovery gesture. Every cancellation assertion,
variant, deadline and original request is otherwise retained. Neither setup qualification
is represented as a Ring product failure or fixed by changing application view policy.

The final owned15 declaration42903B has SHA256
`2b1e09b8649af834bab8d019368acb781ffafd760a1c545559045d90370fc72a`;
its corrected eight-case24281B prefix is exact. Two setup-only inverses reconstruct
both archived predecessors. The two independent source-review tests remain separate.
Coverage includes six actionable polygon tools and their selected size/first anchors,
same-mode no-op during an armed gesture, latched out/back and off-canvas cancellation,
weak source/mode/size/view/file/close ABA and replacement sources, modifier/ineligible-hit
refusals without Alt-form or detached fallback, actual atom-modal/text/dismissal fences,
ordered release/tool/Undo/Save and two complete same-packet current-source commits.
All final live declaration differences are rustfmt only.

## Local integration gate qualifications

Aggregate80277 stops at two new-test clippy lints, before workspace tests. Root simplifies
an equivalent boolean in the test's expected-result branch (truth-table checked) and
replaces its constant bound assertion with an assertion against the actual100000pt page
width plus the same out-of-domain default check. This strengthens setup verification;
no lint suppression, input, tolerance or behavioral assertion is removed. The exact
inverse restores both owned raw declarations. Final combined raw49882B SHA256
`06fc915877185aa6b15d0c53cea3a344ee55dc2b6c1201d60afdd474f4d094a4`.

70000 then stops at the old CLI37-name discovery list.90680 stops at the old23-control
ID set.41603 stops at the old New command's37-name set; a complete static literal-array
audit locates five such remaining sets.26594 reaches245/2/1 in UIlib; both failures are
its shared old four-tool label catalogue. The two expected lists now include the six
new tools and extend the existing bounds/non-overlap checks to all ten. These are missed
catalogue integration edits, not Ring geometry/history failures; every full failed log
is retained. No command body, format, source fixture, chemical/geometry expectation,
control role, deadline or socket policy changes to turn those gates green.

Source checks preserve626 tracked runtime/parser/model/IO/chem/render/xtask/vendor/
compat/assets/fuzz/config files, all582 parity IDs/order and the old MCP control tests.
Removing only the new changed Ring receipt predicate reconstructs the MCP production
control file exactly. Two source-only reviews find no further material issue and retain
actual input hashes. Five final live test units reproduce the qualified raw declarations
through rustfmt only. Local Markdown links pass359 checks before the final status update;
whitespace checks pass. No hosted, native OS, measured-DPI or new parser-fuzz evidence
is inferred from these checks.

## Final full local gate

Aggregate48193 completes successfully with **1412 passed,0 failed,1 ignored**,197
result lines. Its172093B log SHA256 is
`8622085323cbf4f5bf048fa47d05a7ea9fce272493ceb4cc109e39dc53576bbc`.
This includes rustfmt, warnings-denied workspace/vendor clippy, workspace/native vendor
tests, layers, licenses and corpus; the log names this exact workspace root. All17 new
App cases, core9 and actual-wire4 pass. The unchanged old MCP replacement-bind test also
passes in this invocation; no causal repair or historical ownership is inferred.
The earlier five aggregate failures remain distinct in this record and ignored logs.

Native/CLI build76735 also passes with --locked without launching a window. Runtime
closures and bundled assets are unchanged. Corpus remains eight pinned files/five source
bundles; foreign/general/application CDX and Bondline formula/mass/aromaticity/stereo/
SMILES/InChI computations are explicitly missing. No dependency/lock/font/golden/source
fixture/parser/IO/chem/render policy, Actions setting or old socket test changes.
Full M0–M5 remains incomplete; actual native selection/ring and platform acceptance
remain outstanding.

## Actual App frame and binaries

Unchanged ignored capture1726 passes1/0 and writes the actual1600×1000 App frame on
Apple M3 Max/Metal, using the existing three-bond/Double-selected scene. Root inspects
all ten visible rail controls and unclipped paper/inspector. It is not a ring-gesture
screenshot, native input/window, or approved golden. No old golden/fixture is replaced.
Ignored PNG51588B SHA256
`88f2f16c8a7efc19f06624e9de42d0bceb9c1fd310e303944bac8a0464de25dd`;
artifact `target/visual-capture/native-ring-tools-20261116.png`, with JSON/RGBA receipts.

The refreshed native executable22670752B SHA256
`ea11399e134e92d2bdf47c2b8d68cf0053f1545bd0a0413c9d672f5a080c21ac`;
CLI20676096B SHA256
`af1df680d0e8b7f1e2599f0ed7638cec9eba449e69e9d7ca3fb2366795a567b1`.
Actual CLI discovery retains the old37 declaration-order prefix and adds ui.ring_tool
last; all38 names have protocol rows. No bundle is refreshed or window opened. All
Cargo, oracle, build and capture handles for this increment are terminal; worker
reviews/declarations are terminal. Local Git checkpoints carry the continuation.

Source/tests/provenance checkpoint: e869490. The final local link/protection check
passes360 links and the staged62-file scope excludes target/.cache outputs. GitHub
Actions remains disabled; no push/publication. STATUS.md records the source revision,
terminal handles and concrete next native/public-source work without claiming M0–M5.
