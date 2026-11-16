# Native editable SMILES paste

2026-11-16. Extends the [engine checkpoint](verification-smiles-paste-engine.md) with
the actual **Edit → Paste Special → SMILES…** form. GUIDE25 printed351 establishes the
menu role and printed78 the Windows Ctrl+Alt+P shortcut. The explicit dialog,48/72/72
initial numeric values and lifecycle policies are original Bondline choices. Exact26.1
dialog/default behavior and a Mac shortcut remain unknown; automatic clipboard capture
on invoking the command remains a separate requirement.

## Delivered contract and independent tests

The central control invokes the existing typed `ui.file` presentation action; Apply
invokes only `structure.paste_smiles`. There is no empty-notation placeholder or second
authoring registry. The command count stays31; default Drawing/SDF catalogues grow to
19/13 controls. Optional submenu and false-omitted Alt metadata preserve older bindings.
The Windows shortcut is wired; other-platform execution is not established on this Mac.

The multiline notation field reuses the grammar-free64KiB transactional text buffer.
Ordinary OS paste feeds it; input is not trimmed or reduced to its first line/component.
Explicit numeric geometry, empty/invalid input, Apply/Cancel and retained error drafts
use the real engine. Source/view/close/file changes detach a copyable draft and disable
Apply. Queued work retains its accepted intent while capacity is full; Cancel disarms it.
Errors after consumption require explicit Apply and rearming, never an automatic rebase.

`UiSmilesPasteContext` validates the source/page/view/close/file context under the origin
mutex, reserves the existing two-worker budget before consuming the accepted intent,
and rechecks that context atomically with finish after off-lock work. It has no FilePlan,
Save lease, file operation or trusted Save acknowledgment. Pending work, the real caller
thread and copyable forms participate in the native close barrier. A frame-start modal
fence prevents completion/Cancel pointer packets from drawing on the revealed canvas.

Actual-App menu baseline28908 compiles and fails0/1 at missing Paste Special. The frozen
five-test declaration22593B remains SHA256
`7746af8efc3cc616fb57b9debc83c37d1df239a529064a283efb7889be200468`.
First UI compilation31132 fails one E0433: UI has no direct IO dependency. The fix uses
the shared command-layer byte-cap alias; no dependency was added. All five then pass.

The initial native-worker declaration first reaches missing context/plan/method errors
and downstream inference errors; the unchanged nine cases then pass. A separate real
named Save A→B→A regression fails0/1: public path/format/revision/source can return to
their old values after intent consumption. An opaque Weak file-context identity rotated
by existing invalidation latches the change. All ten cases pass, including capacity,
drop/cancel/rearm, stale/foreign inputs, after-work context checks, poison and real threads.
Required receiving rustfmt changes only new-test whitespace; original declarations remain
archived. No old assertions or deadlines were relaxed.

A source review identifies the related **postcommit** presentation window. Its real-App
regression1676 fails0/1 after actual paste, Save B→A and independent file reads: the
successful dialog is wrongly dismissed. A Rust-only, serde-skipped result stamp is now
captured with finish under the same mutex, attached only to native paste work and checked
at capture and retirement. It is not Save authority and Debug exposes no address. All six
App cases pass unchanged, including retained/copyable historical success. Original five
tests remain an exact prefix; final28618B SHA256
`189e8e741cf07fccb4bb9cca45f41bc433e17c314e824455b454f3b644e7e36c`.

```sh
cargo test -p bondline-mcp --test ui_smiles_paste --locked
cargo test -p bondline-ui --lib view_tests::smiles_paste:: --locked
cargo xtask ci
```

Full gate79072 first exposes an old18-control expectation. Only the added control/count
expectations are updated. Final69758 exits0: **1245 passed/0 failed/1 ignored**,180 result
lines; rustfmt, warnings-denied workspace/vendor clippy/tests, layers, licenses and corpus
pass. Log151705B SHA256
`ae8fc643726caab92a5fa41f81cf2348809f11162907252bffc41fdb7a51a365`.
Rust1.94.1, incremental0/jobs2/dev+test debug0 and the literal pinned Python used.
Runtime closures155/166/221 plus2 assets; corpus8 files/5 bundles retains its omissions.
No parser/fuzz, dependency/lock, font, SDK, old fixture/oracle, corpus or golden change.
Actions remains disabled; historical MCP AddrInUse failures remain unexplained.

Final receiving checks retain eight pinned artifacts, the unchanged MCP shutdown test,
all 582 parity IDs in order and the 31-command table; 366 local documentation links and
Git whitespace pass. The first auxiliary ID selector omitted numeric R26 prefixes; its
corrected selector checks the complete unchanged sequence. No product/test change follows.
The ignored receipt is target/aromatic-authoring/ui-final-protection.json.

## Actual macOS execution and files

Build36938 exits0. Only the old Saved preview is quit and its existing bundle refreshed;
no obsolete preview is reopened. Binary20988368B SHA256
`cdecd57c69b81d91ae63ce70702094478147eb2fc8bb2c764ce2b2a5e1b96c14`.
Inline CUA AX/pixel observations show:

- Open the original C2/H3 native drawing: Saved2 atoms/1 bond/1 fragment.
- Open the real submenu/form; use OS clipboard paste and native geometry64/96/96.
- Apply `[NH4+]`: charge refusal, unchanged Saved2/1, retained draft/geometry. Replace
  through the OS clipboard with benzene and retry:8 atoms/7 bonds/2 fragments, dirty.
- One CmdZ restores Saved2/1; ShiftCmdZ restores8/7. F5Actual100 renders the complete scene.
- Clicking an existing Double bond with the Single tool reports the already-missing
  alignment behavior. Stationary numeric keys do not edit. After fresh pointer input,
  key1 changes that Double to Single/H2. Undo/Redo
  visibly restores H1 Double/H2 Single. Source eligibility requires a fresh positional
  event after the failed click; no input-policy fix or general OS-cause claim is made.
- Unicode Save As and explicit reopen succeed, retaining Saved8/7/2 at100%.

The actual1357-byte `native-edited-λ.cdxml` is SHA256
`73bb13116dcc5282e1c9a8ebc5e92e3e328779608f4d5c49796167a2592d9ab4`.
Independent XML parsing and the unchanged pinned RDKit Fragment consumer verify8 atoms,
7 bonds,2 components, C8H14 (pasted ring C6H8), exact old-fragment bytes and unchanged
435-byte original input. Ring centers start96/96 and all sides remain64pt within1e-9pt;
geometry is checked separately from the chemical consumer. XML wire IDs and model IDs
are not assumed interchangeable. Stereo/radicals remain absent.

Ignored artifacts: target/aromatic-authoring/native/. Receiving receipt4008B SHA256
`4dad7275bcb90a48383c96dd3396edfb0834fe73d361ac572e597a6204511976`.
The one current preview is PID39430 at the existing review-bundle path, Saved on that
file, Single selected, no control port requested. Submenu AX activation and empty-field
AX focus did not work on their first attempts; coordinate input worked. Their cause is
unverified. Native races/cancellation/poison, measured DPI/2×, Windows/Linux, automatic
clipboard capture, wider chemistry/layout and exact reference presentation remain open.
The approved old golden is unchanged; no new human-approved golden is claimed.
