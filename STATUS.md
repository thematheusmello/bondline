# Status — 2026-11-20

The full M0–M5 goal is **active**. Verified delivery lands into `main`.
[PR 5](https://github.com/thematheusmello/bondline/pull/5) landed direct-Page CDX
caption preview at `9f27fe9`. Its merge tree matches the reviewed tip `dbb821e`.
The remote feature ref was retired after the ancestry check.
[PR 6](https://github.com/thematheusmello/bondline/pull/6) landed ordered captions
at `a13a0f4`. The tree matches `7ed63cc` and its merged ref is retired.
[PR 7](https://github.com/thematheusmello/bondline/pull/7) landed explicit caption
runs at `4d8ae68`. Its tree matches `2808443` and the merged ref is retired.
[PR 8](https://github.com/thematheusmello/bondline/pull/8) landed owned CDX
atom labels at `3ac74a3`. Its tree matches `b999bfe` and the merged ref is
retired. [PR 9](https://github.com/thematheusmello/bondline/pull/9) landed the
[boxed-label preview](docs/verification-cdx-atom-label-preview.md) at `37d85f0`
and its feature ref is retired. The native/GPU license
blocker is resolved without broadening MIT/Apache/BSD. M0, M1 and M2–M5 remain
incomplete. **GitHub Actions stays OFF**; no enable/dispatch/rerun. Push each atomic
commit as the user requires. The user now expressly authorizes merging completed feature
PRs into main; release/publication and enabling Actions remain separate.

## Current recoverable work and evidence

Main integration is delivered: [PR1](https://github.com/thematheusmello/bondline/pull/1)
lands199 atomic commits as96794f4573237aa019e6fb1c28aa65113547c38e. Local/remote main match;
the merge tree exactly equals reviewed e7dcd85, whose code equals tested1606d62.
No resource WIP was staged.41 already-contained remote branches are retired;158 inactive
local heads are explicitly archived with IDs/worktrees/files preserved. Active Root59
resource files remain byte-exact.33 older Rust/Markdown variants are pushed under recovery
tag archive/legacy-worker-recovery-2026-11-18 (commit140754b), not merged as verified code.
All73 worktree directories remain. See [delivery/recovery policy](docs/branch-lifecycle.md)
and [ref ledger](docs/branch-inventory-2026-11-18.json). Full M0–M5 remains incomplete.

### Verified standalone CDX resources

[PR 3](https://github.com/thematheusmello/bondline/pull/3) merged the verified
resource commits into main at `b9527ab99fb2b5549652efc9f824b149edf5b8a9`. The
merge tree matches the reviewed feature head, including the original fixture,
producer, sanitizer and documentation commits. Local main fast-forwarded to it.

The [resource contract](docs/research/cdx-resources.md) at 391639c is implemented by
the frozen fixtures at 88cb7e9, query and boundary tests at dce5e0f, and sanitizer
coverage at ae329ed. The standalone API reads declared FontTable and ColorTable data
with strict owner, duplicate and payload checks. It retains raw name bytes, unsigned
charsets, RGB16 values and absent/empty/short palette distinctions. The data API
addresses index 65536. Font ID lookup leaves the CDXString index ambiguity unresolved.
The existing `serde_json` edge is dev-only. Registry 49 and catalogue 16/124 are unchanged.

[Resource verification](docs/verification-cdx-resources.md) records the API compiler
probe, runtime 5/7 RED and 12/0 GREEN, whole standalone 47/0, two file boundary
tests and one CLI boundary test. Exact unchanged Save and old authoring/capture/loss
refusals passed. The resource query remains opt-in. Native caption/chemical preview,
CDX editing and general styles are open.

After the user resumed, the first aggregate exposed missing `BONDLINE_PYTHON` in five
existing checks. The qualified `cargo xtask ci` passed **2,202/0/1** workspace tests and
**3/0** AccessKit tests, plus fmt, warnings-denied clippy, layers, licenses, seven assets
and the pinned eight-file corpus. The prior interrupted run and historical MCP listener
failures remain separate. Rust source and test assertions were unchanged between the
resumed runs. The dev commands now show the required Python setting (84a89b5).

ASan passed 10,000 runs in 2 seconds, with 85 immutable inputs and a largest observed
mutation of 393,481 bytes. All seed files reproduced exactly. The resumed CI and fuzz
jobs are terminal. Logs and seed receipts are under
`target/cdx-resources/resume.lQ1sz7`. No user app or drawing was re-opened. The isolated
resource worktree remains recoverable. GitHub Actions is off.

### Verified raw CDXString data

The [raw string reader](docs/verification-cdx-string.md) landed through
[PR 4](https://github.com/thematheusmello/bondline/pull/4) at `6565e58`. Raw property handles, UINT16
run fields and text bytes stay exact. An independent 32-source oracle checks all
five envelopes, 131,070 maximum-count entries, long text and invalid prefixes
confined to their own property. The API compiler probe preceded the four passing
runtime tests. Three test-only lint corrections retained the same expected errors.

The qualified local gate passed **2,206/0/1** workspace tests across 324 reports
and **3/0** AccessKit tests, plus fmt, clippy, layers, licenses, assets and corpus.
ASan passed 10,000 runs on 32 immutable seeds. Its maximum observed mutation was
623,179 bytes. The seed sets reproduced exactly. The verification page links the
reproducible recipe, fixture hashes and logs at
`target/cdx-resources/next-text/raw-query.HoJhSI`. The app Open/Save and rendering
production paths are unchanged for this standalone increment.
Standalone 51/0 and parser-library cross-checks for Windows MSVC, Linux GNU and
WebAssembly passed. App execution remains unverified there.
The resource and string feature branches are merged and their remote refs retired.
The prototype worktrees and user apps remain preserved.

### Direct-Page CDX captions

The [caption profile](docs/research/cdx-caption-preview.md) and original fixture
commit `a90e3b5` fix 12 independent positive models and 63 exact legacy refusals.
The implemented source path projects one explicit Roboto Regular caption on a white
Poster page. It grants read-only display for the exact source-derived model.
Model drift retires preview and exact Save, and Undo restores both. The CDXML
caption-edit and chemical source guards keep their earlier scope.

[Verification](docs/verification-cdx-caption-preview.md) records 1/3 runtime RED
and 4/0 on the same IO tests, actual CLI+initialized stdio+authenticated loopback,
Unicode known Save/Save As/reopen, exact CPU/Metal pixels and independent FreeType
placement. Local CI passed 2,210/0/1 and AccessKit 3/0 before adding the test-only
socket module. With that module included, all 115 adapter tests and warnings-denied
CLI all-targets clippy passed. ASan passed 10,000 runs with 75 original inputs and
158-byte maximum recorded mutation. Registry 49 stays byte-identical. Evidence
and actual PNGs are in `target/cdx-resources/next-text/preview-freeze.Ue3WwW`.
The additional two command tests pass with warnings-denied all-targets clippy.
Equal-model A→B→A source changes retire old weak file/capture contexts and preserve
the original committed output of an earlier Save. Offscreen Fit and Actual at 1.5
pixel density match CPU/Metal and independent FreeType placement.

Actual macOS Open, retained-tool pointer safety, Fit/Actual/112.5% zoom, known
Save, Unicode Save As and reopen pass. An unsupported palette shows the old
error, then Cancel recovers the prior caption. Independent readers confirm the
actual files and models. CPU and Metal captures at the explicit offscreen 1.5
density match at both tested views and across reopen. Physical display density
variation, CJK title glyph coverage, Windows, Linux, and browser execution
still need work. The owned review is closed. Evidence is under
`target/cdx-resources/next-text/native-review.hdS0xZ/native-evidence`.

The final gate on `cbaa4b0` passed **2,213/0/1** workspace tests and **3/0**
AccessKit tests, with fmt, clippy, layers, licenses, assets and corpus. The earlier
MCP listener rebind failure at 127.0.0.1:64951 remains unexplained.
See [verification](docs/verification-cdx-caption-preview.md) for both
runs. All feature execution jobs and review agents are terminal. Actions is off.

### Ordered CDX captions

The [ordered caption increment](docs/verification-cdx-caption-order.md) admits
multiple direct Text siblings on one Poster page. Signed depths establish paint
order independently of scoped caption identity. One omitted depth beneath
positive depths has published order. Equal depths and omission beside negative
depths need further proof.

Eight IO tests pass after three runtime REDs. The actual CLI, CPU/Metal,
independent binary reader and FreeType agree with the original witnesses.
The actual macOS app shows both disjoint captions and the blue top caption in
the overlap. Native Open, read-only input, known Save, Unicode Save As, tied-depth
refusal/Cancel and exact reopen pass. The owned review is closed. ASan passed
10,000 runs and preserved 93 seed hashes. Evidence is in
`target/cdx-resources/next-text/caption-order.8KvDal`.

The full local gate passed **2,218/0/1** workspace tests with three AccessKit
tests and all format, lint, layer, license, asset and corpus checks. The IO
library type-checks on Windows MSVC, Linux GNU and WebAssembly. Runtime
verification on those platforms remains open. Registry 49 stays byte-exact.
All ordered-caption execution jobs are terminal.

### Multiple explicit CDX caption runs

The [explicit run increment](docs/verification-cdx-caption-runs.md) keeps
resolved styles and unambiguous starts in one Text. It preserves blank runs,
colours, sizes and supported Unicode tails. Shared page budgets apply before
run allocations, and unresolved joins leave the older Open refusal intact.
The original `two_runs.cdx` has stronger model, pixel and source tests.

Actual CLI, initialized stdio MCP, independent FreeType and the isolated
macOS app pass the read-only preview and exact Unicode Save/reopen workflow.
The native ambiguous-offset refusal preserves the active file. The owned app
has exited. ASan passed 10,000 runs with 117 original seeds. Registry 49
remains byte-identical. The full local gate passed **2,223/0/1** plus **3/0**
AccessKit tests and all local checks. The IO crate type-checks on Linux,
Windows and WebAssembly. Other runtime platforms, broader typography and CDX
authoring remain open. Evidence is in
`target/cdx-resources/next-text/caption-runs.aTZunB`. All owned jobs are terminal.

### Owned CDX Node labels

The [Node label increment](docs/verification-cdx-node-labels.md) preserves a
direct child Text in the existing atom model. The seven complete original
models preserve their literal fields and all 27 refusal controls keep the
earlier diagnostics. The graph and stored document share the original bytes
for unchanged Save. Numeric fields and authored text remain independent.
Display, authoring, and conversion retain their previous authority checks.

Actual CLI, loopback and initialized stdio MCP recovered the complete model
and exact files. A separate macOS app verified Open, read-only canvas guards,
known Save, Unicode Save As, unsupported-source Cancel and clean reopen.
The independent binary reader confirmed both saved files. The owned review
has exited. ASan passed 10,000 runs with 151 immutable originals. Registry
49 stays byte-identical. The IO crate type-checks for Windows, Linux and
WebAssembly. Other runtime platforms remain open. The local receipts are
under `target/cdx-resources/next-text/node-labels.B6ZqBH`.
The full local gate passed **2,227/0/1** workspace tests, **3/0** AccessKit
tests, formatting, clippy, layers, licenses, assets and the pinned corpus.
All local execution jobs and the owned review are terminal. GitHub Actions
remains disabled.

### Source-qualified CDX atom label preview

The [preview increment](docs/verification-cdx-atom-label-preview.md)
certifies one explicit Poster page with boxed Roboto labels and qualified
plain Single, centered Double and Triple bonds. Six independent models,
39 source controls and exact earlier caption/data behavior pass. The
display proof requires exact current model equality. The original source
still governs unchanged Save and existing edit and conversion guards.

Actual CPU/Metal captures and independent FreeType masks pass. CLI, stdio
MCP and authenticated loopback verify capture and exact file reopening.
macOS Open, Actual size, Fit, read-only canvas, known Save and Unicode
Save As/reopen passed with independent actual-file readers. The signed
review exited. New runtime coverage on Windows, Linux and WebAssembly
remains open after IO type-checks on all three targets. Human golden
approval remains open.

The full local gate passed **2,232/0/1** workspace tests, **3/0** AccessKit
tests, and all fmt, Clippy, layer, license, asset and corpus checks.
ASan passed 10,000 runs with 196 immutable originals. All local jobs are
terminal. Receipts are under `target/cdx-resources/next-text/label-preview`.
GitHub Actions remains disabled.

### Source-preserving CDX existing-label edits

The [binary batch](docs/verification-cdx-batch-edits.md) at `a280f11` checks
all five supported framings and copies every unrelated record, including
unchanged extended lengths. The independent edit witnesses are at `110bee9`.
Qualified atoms with explicit Element and H and an owned boxed Text can
update the four existing properties. Re-import must certify the whole page
and match every model field. History and Save retain their exact snapshots,
with original provenance separate from current edited bytes.

[Verification](docs/verification-cdx-atom-label-edits.md) covers all three
independent file pairs, scoped repeated IDs, original and current byte
accounting, no-op and inverse edits, detached drift and the missing-Element
control. The shared command, CLI, MCP stdio and authenticated loopback pass
with CPU/Metal equality. The App harness verifies Alt-open, Cancel, Apply,
keyboard history, known Save and Unicode Save As/reopen at synthetic 2×.
The isolated macOS app was prepared while hardware was active. Actual
window input remains unverified for this increment. IO type-checks on
Windows MSVC, Linux GNU and WebAssembly. ASan completes 10,000 iterations
with 199 unchanged originals. Evidence is in
`target/cdx-resources/next-text/label-edits`.

[PR 10](https://github.com/thematheusmello/bondline/pull/10) carries the
reviewed producer `ca307af` and landed at `c8f4baf`. The final local gate passed **2,245/0/1**
workspace tests and **3/0** AccessKit tests, including the new App test.
Formatting, Clippy, layers, licenses, assets and corpus passed. The owned
jobs exited. The isolated review stayed unlaunched and Actions stayed off.

### CDX implicit Element editing

The [scoped binary insertion batch](docs/verification-cdx-property-insertions.md)
at `217968b` combines insertion and replacement under the existing limits.
Independent witnesses at `7c25dde` cover connected C→Si and two boxed
cases with reused source IDs. A carbon H-only change keeps omission.
The missing-H control retains the original file and its data projection.

[Verification](docs/verification-cdx-implicit-element.md) covers the
compiler and runtime REDs, exact models and saved bytes, omission-preserving
no-op and history, the existing Alt form and Unicode file workflow at
synthetic 2×, CLI/stdio/loopback and matching CPU/Metal PNGs. RDKit reads
the actual 503-byte output. ASan passes 10,000 runs with 205 unchanged
originals. IO type-checks for Windows MSVC, Linux GNU and WebAssembly.
The signed review stayed prepared during hardware activity. Live
macOS input remains unverified. Evidence is in
`target/cdx-resources/next-text/implicit-element`.

[PR 11](https://github.com/thematheusmello/bondline/pull/11) carries
reviewed producer `20f3644` and landed at `7b5e9c1`. The final local gate passed **2,254/0/1**
workspace tests and **3/0** AccessKit tests, including the new App case.
Fmt, Clippy, layers, licenses, assets and corpus passed. All owned jobs
exited and Actions stayed off.

### Editing CDX caption runs

The independent sources at `f6b71ad` cover single-run growth, earlier-run
growth and shrinkage, and a UTF-8 final run. Source and paint order can
differ, including anonymous Texts. The existing command updates one
owned Text property while preserving count, style words, placement, paint
order and unrelated bytes. It reimports the full caption model and
retains original and edited source snapshots.

The [verification](docs/verification-cdx-caption-edits.md) covers compiler
and runtime REDs, the exact 16-bit start boundary, four full-model/byte
witnesses and history, actual CLI/stdio/loopback, App repaint and file
controls. Independent FontTools agrees with actual coloured ink within
the declared three-pixel allowance. Before and after CPU/Metal captures
match. Registry49 changes one description and keeps schemas and order.
ASan passes 10,000 runs with 211 unchanged files. The signed desktop
review was prepared during hardware activity. Native caption input
remains open. Receipts are in
`target/cdx-resources/next-text/caption-edits`.

[PR 12](https://github.com/thematheusmello/bondline/pull/12) carries reviewed
producer `bdae9cb`. The final local gate passed **2,263/0/1** workspace
tests and **3/0** AccessKit tests, including the new encoder and App cases.
Fmt, Clippy, layers, licenses, assets and corpus passed. IO type-checks
on Windows MSVC, Linux GNU and WebAssembly. All owned jobs exited and
Actions stayed off.

### Text tool for existing captions

[PR 13](https://github.com/thematheusmello/bondline/pull/13) carries
the [public workflow contract](docs/research/native-caption-editing.md)
at `473aa31`, the independent painted-run hits at `149ad48`, and
the tested application and command integration at `b26f5d5`.
Choose Text or press `t` to edit a visible run in qualified CDXML or
CDX. The tool uses the current paint order and exact press/release
ownership. Apply routes through `text.set_run`, preserving styles and
whole-source history. The draft remains copyable if its owner goes
stale. Registry 50 appends `ui.caption` with the first 49 entries
byte-exact, and the catalogue has 17 bindings and 123 planned entries.

The [verification](docs/verification-native-caption-editing.md) records
eight App-frame cases, five renderer cases, three command cases and a
real authenticated loopback case. Exact independent CDX and CDXML
bytes pass Unicode Save As and reopen. The command crate type-checks
on Windows MSVC, Linux GNU and WebAssembly. Sanitizer checks pass
10,000 runs with 211 unchanged inputs. The full local gate on
`b26f5d5` passes **2,280/0/1** workspace tests and **3/0** AccessKit
tests, with format, lint, layer, license, asset and corpus checks.
Earlier runs stopped at superseded command and rail inventory
expectations. The product source did not change between those gates.
The signed review stayed unlaunched after recent desktop input.
Live platform input and exact 26.1 interaction remain open. Evidence
is under `target/native-caption-editing`. All owned check jobs exited.
GitHub Actions remains off.

### Native plain-caption creation

[PR 14](https://github.com/thematheusmello/bondline/pull/14) records this increment.
The [public contract and independent witnesses](docs/research/native-caption-creation.md)
landed on the feature branch at `c20707b`. The CDXML identity codec at
`223253d` retains sparse native Text IDs across exact save and re-import.
The shared command and native Text tool at `e4e5f56` create a single-line
caption at the blank-paper press baseline. Creation uses explicit 12 pt black
Roboto Regular and requires a complete single page with source-free or
byte-exact native CDXML authority. Apply certifies the file representation
and whole page, then commits one history edit. Undo retains the Text ID floor.
Registry 51 preserves the first 49 records byte-for-byte and adds one final
`ui.caption` action. The catalogue stays at 17 bindings and 123 planned entries.

[Verification](docs/verification-native-caption-creation.md) covers actual
CLI and stdio MCP, exact native/foreign IO boundaries, App frames at 1×/2×
and byte-identical CPU and Metal PNGs. The isolated macOS review passed
blank-paper creation, Cancel, Undo/Redo, Unicode Save and editable reopen.
Expat confirms the saved raw ID 3 and exact literal body. The review exited.
Command crate type-checks pass on Windows MSVC, Linux GNU and WebAssembly.
AddressSanitizer completed 10,000 runs on 48 unchanged seeds. Local records
are under `target/caption-create` and `target/caption-creation`.
The final local gate on `a98dc47` passed **2,297/0/1** workspace and **3/0**
AccessKit tests, plus formatting, Clippy, layering, licenses, assets and the
pinned corpus. Earlier gates found obsolete inventory assertions and one
unchanged listener replacement-bind failure. All owned caption jobs exited.
GitHub Actions stays off. Full M0–M5 remains active.

### Standalone CDX child insertion

[PR 15](https://github.com/thematheusmello/bondline/pull/15) records the
independent vectors at `b9674ce` and the checked splice at `dbc2f32`.
The [contract](docs/research/cdx-child-insertion.md) defines one bounded
property-only child splice at the matched physical end of an owned
encoded object. The [verification](docs/verification-cdx-child-insertion.md)
records independently checked exact bytes across five framings, eight
new ownership and bound regressions, 67 standalone CDX tests, and
10,000 AddressSanitizer runs over 18 unchanged inputs. The tool and
application registry retain their current source admission.
The full local gate passes **2,305/0/1** workspace and **3/0** AccessKit
tests, with formatting, Clippy, layers, licenses, assets and corpus checks.
Owned CDX splice jobs are terminal. GitHub Actions stays off.

### Creating captions in qualified CDX

[PR 16](https://github.com/thematheusmello/bondline/pull/16) landed at
`c1bdbdd`. The merge tree matches reviewed tip `fe4082c` and the merged
feature ref is retired.

The public contract at `84b60c6` pins six original page and live-ID
witnesses. `7b022b2` shares one effective live-ID map between reader
and run editing. `a1ab05f` appends a direct Page Text and its persistent
ObjectTag, then certifies the complete re-import and paper scene. All five
framings have exact writer checks. Choose Text or press `t`, then click
blank paper on a qualified caption-only Poster page with an existing
caption. Apply writes one 12 pt black Roboto Regular run. The same command
serves CLI and application paths. It retains the live-ID floor across Undo.
[Contract](docs/research/cdx-caption-creation.md),
[witnesses](crates/io/tests/fixtures/cdx-caption-create/SOURCES.md).

Independent readers verify exact Unicode Save and reopen, even with an
Undo gap leaving IDs 1 and 3. App frames pass at 1× and 2×. An isolated Mac
review used real Open, Text, Cancel, Undo/Redo and Save As, then reopened
and edited the new caption. Both captions resolve their own draft targets.
CPU and Metal output matches exactly in both CLI and Mac review. The
registry remains 51 entries with the first 50 byte-exact. The catalogue
stays 17/123. AddressSanitizer passes 10,000 runs with 66 unchanged seeds.
The command crate type-checks on Windows, Linux and WebAssembly. Native
runtime is verified on macOS for this workflow. The review exited.
[Evidence and limits](docs/verification-cdx-caption-creation.md).

The final local gate on `6c0689a` passed **2,320/0/1** workspace and
**3/0** AccessKit tests, plus formatting, Clippy, layers, licenses, assets
and the pinned corpus. The preceding MCP listener replacement-bind failure
is recorded separately. The retry passed with the listener code and test
unchanged. All owned caption-creation checks exited. GitHub Actions stays off.

### Creating the first caption on empty CDX paper

The implementation on `feat/cdx-empty-caption` follows source lock `c6f0a6f`.
A single empty, qualified Poster opens at its physical size. Text creates its
first caption at explicit signed depth zero. History, Unicode Save, reopening
the sparse branched ID and editing it pass the App, executable and adapter
checks. All six variants match independent complete files, with 19 unsupported
sources retaining their previous behavior. Physical Mac input reached the
draft. The background Mac host completed Apply, history, Save, reopen, existing
draft inspection and Cancel. CPU and Metal captures match. The qualified
10,000-run sanitizer check passes with 103 immutable seeds. Feature commit
`0728770` is pushed. Local `cargo xtask ci` passed 2,324 workspace tests with
one ignored, plus three AccessKit tests and all local gates. The strengthened
fallback assertions and their warnings-denied Clippy check also passed.
[Contract](docs/research/cdx-empty-caption.md), [verification](docs/verification-cdx-empty-caption.md).

[PR 17](https://github.com/thematheusmello/bondline/pull/17) landed the empty
Poster workflow at `0f76377`, with the verified head tree and feature ref retired.

### Mixed chemical structures and captions in CDX

Source fixtures `3f69901`, decoder `8d31595`, native viewing `53136e5`
and fuzz oracle `c5fa50a` are pushed on `feat/cdx-mixed-caption-preview`.
The file now opens as one complete read-only drawing when its caption depths
sit above the qualified chemical ink. The two source orders produced the
same CPU and Metal images at 100% and 200%. Both 368-byte sources survived
Unicode Save and reopen exactly. Fifteen App caption tests, the source
oracle, three mixed IO tests, the shared run budget boundary and 10,000
sanitizer trials passed. The actual Mac host was exercised in the background.
The full local retry passed 2,329 workspace tests with one ignored, three
AccessKit tests and all gates. The first attempt hit the existing intermittent
MCP replacement-bind error on port 55042. Its cause remains open.
[Scope and verification](docs/verification-cdx-mixed-caption.md).

[PR 18](https://github.com/thematheusmello/bondline/pull/18) landed the mixed
view at `a838462` with the reviewed head tree. Its feature ref is retired.

### Editing existing CDX captions on mixed pages

The mixed-run feature on `feat/cdx-mixed-caption-run` has independent sources
`8f23d38`, IO `29b4a3d`, command and UI `b83f50e`, and fuzz `1c2c9fa`.
Text targets the painted direct Page run above the chemical label. ID 19,
both atom-owned aliases, source styles, the second run's updated start and
whole-file Undo/Save/reopen are independently checked. The actual hidden Mac
host, CLI and stdio MCP agree on the edited files and 100%/200% CPU/Metal
images. The bounded sanitizer passed 10,000 trials with 137 immutable inputs.
Correction `f93778f` retires a pristine native form on external Apply and
preserves unsent local text. The final local gate passed 2,336 workspace tests
with one ignored, all three AccessKit tests, and the formatting, clippy, layer,
license, asset and corpus checks. GitHub Actions stays off.
[Run-edit evidence](docs/verification-cdx-mixed-caption-edits.md).

[PR 19](https://github.com/thematheusmello/bondline/pull/19) landed at
`c33987c`. Its tree equals the reviewed head and the merged feature ref is retired.

### Adding captions to imported chemical drawings

On `feat/cdx-chemical-caption-create`, the source contract is `4151329`,
writer and receiving tests `373b327`, independent file and history witnesses
`4f3144d`, sanitizer oracle `fa56c29`, native App input `d927dc5`, actual CLI
`fdbd710` and refusal cases `50ba05f`. On supported chemical-only pages,
a new caption sits above the chemical ink. On mixed pages it sits above all
the earlier captions. Undo restores the exact input. The CLI, MCP server
and native host now agree on the independently predicted files, colors and
IDs, including both 595-byte “Note β” hashes from the preceding checkpoint.
The new App tests cover ink selection and draft creation on blank paper.
Offline Chrome ran the Rust IO example from `1e43cd8` and reproduced all
four exact files. A deliberately wrong expected file failed. Final local CI
passed 2,343 workspace tests with one ignored, three AccessKit tests and all
gates. The earlier intermittent MCP bind failure on port 64219 remains
unexplained. GitHub Actions stays off.
[Acceptance and reproduction](docs/verification-cdx-chemical-caption-creation.md).

[PR 20](https://github.com/thematheusmello/bondline/pull/20) merged at
`7c21f89`. Local main matched the remote merge and the reviewed tree at
`a61fa56`. The completed feature ref is retired.

### Editing existing chemical labels after annotation

The public contract and 13 independent files are at `cb5b6f9`. IO `a031c62`
and command history `cc8cd6f` preserve every direct Page caption and the
other fragment when an existing atom label is changed. Native input `819e618`
supports exact Alt-primary immediately after caption creation while Text stays
selected. A painted caption shields an obscured atom. The actual CLI test is
`ff69629`, source-preserving fuzz oracle `fc93f5a` and verified Rust browser
example `7deea1f`. The Mac host, CLI and MCP passed seven exact output files
and fourteen CPU/Metal frame comparisons each. Offline Chrome checked the
codec against three patched files. Integrated local CI passed 2,353 workspace
tests, one ignored, all three AccessKit tests and all gates. The three
commands-crate cross-target checks passed. The temporary isolated checkout
was retired after checking all 33 files against the committed tree. Full
Windows/Linux desktop and browser UI workflows remain open.
[Acceptance and reproduction](docs/verification-cdx-mixed-label-edits.md).

[PR 21](https://github.com/thematheusmello/bondline/pull/21) merged the tested
mixed-label writer at `6fd6633`. Local and remote main matched the merge.
Its completed feature ref was retired.

### Skeletal CDX drawings on their original paper

The renderer at `868169c`, source reader at `0b3681f` and desktop controls at
`f6015a0` display a source-qualified neutral single-bond carbon chain on its
declared Poster. Five source encodings render identically while retaining their
individual hydrogen fields and carbon visibility flags. Offline Chrome runs the
same IO and CPU renderer at `895b067`. The sanitizer at `0ed5d77` checks the raw
Node owners, degree, H and flags, exact source output and detached-model refusal.
The separate ring regression at `00ec56f` verifies 1,776 black pixels, the
four retained omitted H fields, and atomic Save As and reopen. Both native
desktop targets type-checked. The browser UI build still reaches native MCP
networking and fails in mio. The full local gate on `00ec56f` passed 2,363
workspace tests with one ignored, all three AccessKit tests and all gates.
[Acceptance and local reproduction](docs/verification-cdx-skeletal.md).

[PR 22](https://github.com/thematheusmello/bondline/pull/22) merged the
verified skeletal viewer at `df7790a`. The merge tree matched `8af98c8`
and its completed feature branch was retired.

### Move selected skeletal atoms

The writer at `00ceb73` and native selection at `98f3edd` move owned Node
positions, preserve original length headers, and support exact Undo, Redo and
Unicode Save/reopen. Six independently predicted files include a long-form
position and repeated raw IDs in different fragments. Native drag and nudge
were exercised in App frames. CLI, MCP and macOS loopback all preserved the
expected saved bytes and produced 12 matching CPU/Metal pairs per surface.
The Rust WebAssembly file round trips and moved images passed in offline Chrome
at `76d3358`. The raw coordinate movement oracle at `2922cd9` passed 10,000
AddressSanitizer iterations from 245 verified seeds.
[Contract](docs/research/cdx-skeletal-movement.md) and [acceptance](docs/verification-cdx-skeletal-movement.md).

The integrated local gate at `e1a1cd3` passed 2,369 workspace tests with one
ignored, plus all three macOS AccessKit tests. Both x86-64 Windows and Linux
desktop binaries type-checked. Interactive tests on those systems remain open.

[PR 23](https://github.com/thematheusmello/bondline/pull/23) landed Hill
formula notation on main at `8f0e2f7`. The integrated gate at `db52782`
passed 2,364 workspace tests, one ignored, all three AccessKit tests and
all local checks. Main matched the tested tree and the feature ref was retired.
[Formula examples and ordering conventions](docs/research/hill-formula.md).

[PR 24](https://github.com/thematheusmello/bondline/pull/24) merged the
all-carbon movement workflow into main at `d68390c`. The merged tree matched
the tested feature with its final verification notes.

### Printed heteroatoms on skeletal CDX paper

[PR 25](https://github.com/thematheusmello/bondline/pull/25) adds native viewing
of methanol, ethanol and 3-aminopropan-1-ol drawn with hidden carbon vertices.
The four binary inputs include two encodings of methanol. Open, Unicode Save As
and reopen preserved the original bytes. Actual macOS, CLI and initialized MCP
returned matching CPU and Metal captures at 100% and 200%. FontTools verified
the printed label bounds and bond clearance, and RDKit checked each formula
and connectivity. The offline Chromium codec check and 10,000 AddressSanitizer
iterations on the integrated parser passed. The full local gate at `a0269d2`
passed 2,373 workspace tests with one ignored, all three macOS AccessKit tests
and all required validation steps. [Tests and scope](docs/verification-cdx-mixed-skeleton.md).

### Analysis window and literal molecular formulas

[PR 26](https://github.com/thematheusmello/bondline/pull/26) adds the native
View menu window and shared queries for a supported complete drawing or its
selected atoms. The first 51 published command records retained their order
and metadata across the CLI and initialized MCP. The two new commands are
`analysis.formula` and `ui.analysis`.

The frozen macOS run opened seven original skeletal CDX files and read the
actual window’s AX values. It showed `C3H8` for fully declared propane,
`CH4O` for fully declared methanol, the selected terminal contribution
`CH3`, and the first unresolved atom for omitted counts. The read-only
mixed profile retained its selection restriction. PID 15528 exited through
the registered Close command. [Scope and receiving evidence](docs/verification-analysis-window.md).

The combined full local gate on `3e4cd08` passed 2,390 workspace tests
with zero failures and one ignored. All three additional macOS AccessKit
tests, formatting, strict Clippy, layers, asset and runtime license checks,
and the eight-file pinned corpus passed. The log SHA-256 was
`d1cc50dc360a47b854bacab39979ff680043066ad25e3016f3b0c06d2431238f`.
PR 25 landed on main at `40455fd`. Its ancestry merge into PR 26 at
`6508ff2` changed two documentation files relative to that tested tree.
PR 26 landed on main at `7050792`. The local gate and native jobs are terminal.
GitHub Actions remains disabled.

### Formulas for skeletal CDX with omitted hydrogen counts

[PR 27](https://github.com/thematheusmello/bondline/pull/27) uses the
complete qualified CDX drawing to count hydrogens for the whole structure
or a selection. Selecting the middle carbon of propane gives `CH2`. Each
of eight files matched RDKit before and after Unicode Save As and reopen.
Every source byte, including the absence of hydrogen properties, survived.
The native Analysis window identifies how many atoms used a default.
Unsupported source chemistry still gives an explicit diagnostic.

The full local gate at `0efcab9` passed 2,391 workspace tests with one
ignored, all three additional macOS AccessKit tests, and every local audit.
The local gate and native verification jobs have finished.
[Acceptance and independent oracle](docs/verification-analysis-window.md).

### Next executable work

1. Integrate molecular weight and monoisotopic mass with independent reference
   data, explicit missing values and the existing whole-drawing and selection
   controls. Isotope distributions, elemental percentages and live analysis
   captions remain open.
2. `../bondline-cdx-mixed-skeletal-movement` is adding source-preserving
   movement to the printed-label drawings. Verify position and dependent
   bounding-box bytes together, then undo, redo and reopen.
3. The web host in `../bondline-web-workspace` is exercising actual egui/WebGPU
   startup, offline editing and history, and recovery across browser tabs.
   The codec test exercises a separate headless Wasm entry point. Interactive
   Windows/Linux validation and further work across M0–M5 remain outstanding.
   [Roadmap](ROADMAP.md) and [parity checklist](docs/parity-checklist.md).

Recoverable plain CDX bond workflow: basebaa3d90, contract6c9fff0, independent originals/
helperc29c1a1, IO37d094d, caller5afb434 and fuzz1606d62. Read
[contract](docs/research/cdx-plain-bonds.md),
[receiving](docs/verification-cdx-plain-bonds.md) and
[provenance](crates/io/tests/fixtures/cdx-plain-bonds/SOURCES.md).
This supersedes historical next-task/job paragraphs and the persistent goal's old
line-width first task. Every created atomic commit is pushed to the same origin branch.

The limited read-only graph now handles binary Single/Double/Triple, absent/explicit
Solid and six directed DoublePosition values. Query3 is never Triple. Missing position
stays Unspecified; automatic/manual mode remains exact immutable source data, not a new
model field. Explicit Display2/position on non-Double orders refuses even0. Both global/
local owner lists and exact duplicate/width/error policy stay strict. No full/display/
chemical-authoring/cross-format permission, command or grey tool is added. Existing
detached page edits/history still refuse CDX Save until Undo. Registry49 actual records/
schema/order and catalogue16/124 stay exact.

Original16 sources/149 strict-reader refusals reproduce exactly; pinned external RDKit
agrees on graph/order/direction/H2/H1 across three envelopes, without certifying visual
side/manual-mode/layout. IO initial0/4 then frozen2pass/10fail precede producer→12/0.
Exactly old Order2 becomes a stronger positive after its observed failure;0/3/FFFF and all
other negatives remain. FullIO541/0 passes. One new-test needless borrow and stronger exact
duplicate-record assertions are separately qualified by final12/clippy/fmt; no product
change follows fullIO. Root commands0/2→2/0, App0/1→1/0 and actual CLI/MCP/socket0/3→3/0
pass. Peer review strengthens known-Save request/success proof with qualified App1/0.

Separate own Mac review95769 passes AX-confirmed retained8Ring→Open manual-Right Double→
known Save→Unicode Save As→reopen, then Triple Open→Unicode Save As→reopen. Seven complete
models, clean history/source and actual-byte/struct/RDKit observations agree. Known Save
has public epoch3→4 plus controlled2000mtime→actual-publication proof. Actual files379B/
SHA25633727f121e74d276fbc93f22ed274e193118bc166f97a36ba79849efe42df7bc and367B/SHA256
79856421a31d22c75737236738a4a26e0af342cc77a127746b0afe551cf3c579 equal frozen sources.
CPU/GPU capture, same-order Triple edit and whole CDXML/MOL saves all refuse with unchanged
state/sentinel. The unmapped-source banner is honest; stale Ring inspector instructions,
initial batched tool/Open setup miss, AX truncation and CJK filename-glyph limits remain
explicit. No successful CDX preview/image or causal UI fix is claimed. Own review is quit,
exact bundle/original PID absent; user-viewed365 and older apps/drawings stay protected.

ASan13740 passes10000/1s, cov1321/ft2901/RSS232MiB,184 immutable/regenerated sources.
Initialmax131072B and final limit131072B differ from actual NEW/REDUCEmax104971B. Manifest
8438fb643a457f5ddd2c534914517980bbbf8d9597b51973e9f6dac2e52f6cda;
log25214B/SHA256447008f3d4939b8e718513be70a348fd28324e66892371d691b387cff184974c.

Fresh full local xtask CI2882 on1606d62 PASSES workspace2187/0/1 across321 reports plus
vendor3/0, workspace/fuzz fmt, warnings-denied workspace/vendor clippy, layers, strict
runtime licenses/seven assets and pinned eight-file/five-bundle corpus gates.
Log275214B/SHA256f3fdf9aaa2c25c4ab76d9ce03486ec6f34d9a0f372bfa0dd315c706855ebbb0c.
This is a clean local aggregate. Earlier1390's MCP replacement_bind51670 failure and
historical occurrences remain unexplained, not causally fixed; protected test/policy
stays exact. No dependency/lock/vendor/font/asset/golden/workflow change. Human NH2 golden
approval and new Windows/Linux/WASM execution remain unverified.

All owned implementation/native/sanitizer/gate jobs and workers are terminal. Root source/
tests are committed/pushed; isolated branch/worktree retains only the integrated candidate
and copied fixtures/helper for provenance, with no worker commit or unique pending code.
The next-resource read-only audit is complete: report8097B/SHA256
bc29b83cc3704e5ede4abea40940872300f112caca8750f09a760abd9302aebc and receipt manifest6375B/
SHA256a8574c064609a29687d1a4cc1632283766d4372cc1241607ced3fbe32adad9e4. Five fresh200
bodies, linked charset404 and five reused current receipts support the proposed next task;
the following [resource contract](docs/research/cdx-resources.md) now locks the standalone
API/policies before code. No resource decoder/Text permission is implemented by that audit. Actions OFF;
full M0–M5 remains active/incomplete.

Historical next-work plan for the delivered resource increment:

1. Read the locked [resource contract](docs/research/cdx-resources.md), source register/
   open questions and optional ignored audit. The planned standalone
   `bondline_cdx::Document::resources()` view retains platform/id/raw UINT16 charset/name
   bytes, strict count/length/duplicate/owner limits and palette presence. Binary empty
   tables stay present; a u32 colour data index preserves all entries through65536. Retain the
   font-style index-versus-declared-ID conflict and unknown name codec; do not resolve
   styled Text or infer a font from the apparently truncated published hex example.
2. Freeze independent original struct vectors/readers; distinguish the new API compiler
   probe from actual runtime RED as locked, then implement the
   checked standalone view with exact source retention and borrowed/owned budget policy.
   Missing tables remain distinct from present zero/short tables; no default-font/codec,
   style/H/placement inference, IO admission, registry/UI or display/edit grant follows.
   Cover raw charset high values, nonsequential IDs, malformed/duplicate/truncated/owner
   cases and boundary sizes, then parser fuzz/local gates and atomic commit+push.
3. Continue actual CDX Text/styles/resources/faithful preview/source-preserving edits,
   broader M1 interchange, all remaining M2 tools/selection/journal styles, M3 chemistry/
   reactions, M4 publishing/clipboard and M5 naming/templates/WASM/installers. M0 platform/
   input/accessibility/DPI/jobs/addressability and historical listener cause stay open.
   Do not repeat delivered scalar/plain-bond work or enable Actions at this checkpoint.

### Previous literal CDX metadata checkpoint

Recoverable literal CDX metadata: base42ccefb/clarification4af42c5, frozen originals9a56c97/
helper9b77891, IO4996ee7 (isolatedc018738), actual wired414c6a, command/App29e8831 and
fuzzf11a40c. Read [contract](docs/research/cdx-literal-atom-metadata.md),
[receiving](docs/verification-cdx-literal-atom-metadata.md) and
[fixture provenance](crates/io/tests/fixtures/cdx-literal-metadata/SOURCES.md).
This supersedes every historical next-task/job paragraph and the goal's old line-width
first task. Each atomic commit is pushed to the same origin branch.

The limited read-only graph now reads Node isotope/charge/radical/NumHydrogens into
existing fields. Both owner lists/duplicates/widths/domains are strict; missing H stays
None. Four-byte charge follows the SDK signedINT32>>24 instruction and preserves every
low byte. No full/display/chemical-authoring/cross-format authority, command or grey tool
is added. The old model-only page edit/history stays available but CDX Save refuses drift
until Undo. All49 actual command metadata and catalogue16/124 stay exact.

Eighteen independent original sources/36 strict-reader refusals reproduce exactly.
Actual RDKit agrees on qualified isotope/charge/H/doublet vectors;01000000 differs
(normative0/RDKit1), as does AABBCC7F (127/−86). Electron count cannot prove singlet/triplet.
Runtime IO0/4 and grouped3pass/10fail, commands0/3, App0/1 and actual adapters0/3 precede
production; GREEN13IO/3command/1App/3wire pass. WholeIO529/0 passes. Exactly six old valid
literal-field witnesses become stronger positives; all other unknown/query/label/stereo/
owner/refusal guards stay unchanged. Test declaration/helper/lint qualifications remain
separate from the actual runtime REDs in receiving.

Separate own Mac review86673 verifies retained Ring→Open isotope compound→known Save→
Unicode Save As→reopen, then four-byte-charge Open→Unicode Save As→reopen. Complete
independent models/actual files/struct/RDKit checks pass, with the legacy consumer
discrepancy preserved. Actual files353B/SHA256
25966d5b508ab7ef1d064d831f85bf79bbeec65f331b0cb62e27a6a91942f554 and230B/SHA256
b9a0a9f4b58226666d4aba20fa35fb669834ead30b202ad5c99531ae03cd57eb equal original sources.
CPU/GPU capture both refuse unsupported_content without state changes; no CDX image or
render parity is claimed. Retained Ring inspector instructions and CJK filename glyph
coverage remain presentation limits. Own review is quit, exact bundle/original PID absent;
user-viewed review365 and older apps/drawings remain protected.

ASan97553 passes10000/1s, cov1261/ft2818/RSS173MiB,43 immutable/regenerated seeds. Initial
max131072B, mutationlimit65746B and actual NEW/REDUCEmax777B stay distinct. Manifest
bb61c910883ed54656d174fba53e9fd72b85c4b194eeae49096f88b2a19cb026;
log29351B/SHA2564b178ddf322325c03628a64cac41dbe6de2a38ba46dfb51df68471729750a17b.

Full local xtask CI1390 onf11a40c fails1420/1/0 across222 reports at unchanged MCP
replacement_bind51670/AddrInUse. Log176572B/SHA256
af9fda65b0b4dbb69272ec51a431f1ed9da140cafcabf683a6dc11f33adb754e. Remaining11036 passes
751/0/1 across98 reports, including vendor3/0, and every previously unexecuted stage;
log98069B/SHA256f1316e84970dc580d04d0d4264a9dfad98ca9f812f6a35a6dc5c92151783f8db.
Combined workspace2168/1/1 across319 reports plus vendor3/0 is **not a clean aggregate
or causal listener fix**. The failed case is not retried or weakened. Workspace/fuzz fmt,
warnings-denied workspace/vendor clippy, layers, strict runtime licenses/seven assets
and pinned eight-file/five-bundle corpus pass. All implementation/execution jobs are
terminal; worker worktrees/branches remain for provenance. No dependency/lock/vendor/font/
asset/golden/workflow change. Human NH2 golden approval remains pending; new Windows/
Linux/WASM execution is unverified. Actions remains OFF; full M0–M5 stays incomplete.

Next executable work:

1. Source-lock a bounded plain CDX bond projection: binary Order bitmasks1/2/4,
   Solid Display/Display2 and DoublePosition presence/Center/Right/Left relative to B→E.
   Declare automatic/manual source distinctions, omitted position and non-Double rules
   before code; do not infer Center or render permission. Freeze independent original
   bytes/complete facts plus actual external graph observations, then observe runtime RED.
2. Implement only the agreed data prerequisite through both global/local owner lists,
   exact source/unchanged Save/model-drift/history/adapter/native workflows, parser fuzz
   and local gates. Preserve all existing source/full/display/loss guards and registry49/
   catalogue16/124. Faithful CDX styles/resources/Text/preview/editing remain separate
   requirements; general omitted-run typography is still unproved, not a default to invent.
3. Continue broader M1 interchange, all remaining M2 tools/selection/journal styles,
   M3 chemistry/reactions, M4 publishing/clipboard and M5 naming/templates/WASM/installers.
   M0 platforms/input/accessibility/DPI/jobs/addressability and historical listener cause
   remain open. Keep pushing every atomic commit and keep Actions OFF.

### Previous scalar-default metadata checkpoint

Recoverable scalar-default metadata workflow: base0aa97cd, contract8c2b608, original
fixtures deb078d, IO4e8470f (isolated47b021b), command/UI d8bc3ba, wire51fa159,
isolated owner-test qualification2e939f6, render e615cbf (isolated0f025d0) and fuzzc965607.
Read [contract](docs/research/label-default-metadata.md) and
[receiving](docs/verification-label-default-metadata.md). This supersedes historical
next-task/job paragraphs below and the persistent goal's old line-width first task.

Separate LabelDefaults authority covers checked unused root/n/b/t LabelFont/Size/Face
beside fully explicit runs. All owners/shadowed values/resources are checked; signed
scalar fields remain distinct from actual UINT16 run/font IDs. Unused different fonts,
face96/32767 and finite size1e308 stay source data, never glyph/default/preset input.
Every actual s font/size/face/color stays required. Existing48/49/Selection alone gain
the precise permission, with immutable metadata spans, current hidden/resource/source
proofs and a final page certificate. Old Boxed/Connected/Mixed/Flagged getters, fullfalse,
other authors/loss guards, all49 metadata and catalogue16/124 stay exact.

Twelve original sources/31 FontTools boxes/14 strict-reader negatives reproduce exactly.
IO516/0, command7, App1, actual CLI/initialized stdio/authenticated socket3, source-aware
render2/six whole CPU-GPU frames and renderer195/0 pass. Exactly three old metadata-only
IO negatives become stronger oldgetterfalse/newtrue/source/model/CPU equality positives;
all other negatives stay exact. Test-only Result alias, single-case loop and zero-match
App filter qualifications are recorded. Independent review strengthens A→B→A gesture
invalidation separately from form Open; no producer guard is weakened.

Fresh own Mac review40752 passes retained Ring→Open→100%→Alt49 visible-style/Cancel/
Apply→Undo/Redo→Alt48 N/H2→Marquee/Select All→all-five(24,-12)drag/history→Unicode Save/
reopen→existing-label Cancel. Actual2109B file equals frozen compound, SHA256
4b836da2349a33f2776a1355f739c65d6a2cfbd3ec84f34e9cee0226c8f4d7df; independent actual-file
Expat/Decimal/constructed+direct RDKit facts agree. Native1088 CPU/Metal PNGs are exact
29007B/SHA2568a21d02c21e044560c63d0f4d669f3dd1555ba9765adc572198b5ff9778dc71e.
AX100-row truncation/after-Quit own Untitled probe and CJK filename-glyph limits remain
explicit. Both own metadata instances are quit; originalPID40752 and bundle inventory
are absent. User-viewed creation review365 and older apps/drawings stay protected.

ASan76283 PASSES10000/391s, cov9099/ft22007/RSS488MiB,30 immutable/regenerated sources,
actual NEW16384B. Manifest682f24b1a5e599feb28ebcf67bd24e0219cfcfdee330ccfd4749014dafafb872;
log116952B/SHA256f2e2dc2d5e141ba6c1660e2aec61b3d9f5ebdd6b85aeb18539831c18d3264327.
Full local xtask CI98153 PASSES on code `c965607`: workspace2149/0/1 across317 reports,
vendor3/0 and all fmt/clippy/layers/licenses/seven-assets/pinned-corpus gates. Log270979B,
SHA256768d1dae09310a6d6b200f6347e0b974cf113318faf180eb2b994be74a6101cb. This is a clean
local aggregate; the historical MCP listener failure remains unexplained, not fixed.
Protected test/policy remains byte-exact. All workers/execution jobs are terminal.
The separate next-CDX source audit is complete: eight fresh primary receipts, canonical
type/owner/absence evidence and the documented four-byte Charge recovery rule are ready
for the next source lock. No CDX implementation or new external-chemistry result follows.
The final receiving checkpoint is dfedef8; keep pushing each next atomic commit.
Actions API enabledfalse and foundation workflow disabled_manually are verified.

Next executable work:

1. Read the now-locked [CDX literal-atom contract](docs/research/cdx-literal-atom-metadata.md),
   source register and existing cdx.rs/read_atom/Layout/property tests. Freeze independent
   Python-struct bytes/facts and qualify actual external RDKit before changing production;
   observe existing-API runtime RED for Node data and wrong owners. No new decoder exists yet.
2. Implement that M1 read-only isotope/charge/radical/NumHydrogens projection, including
   the explicit property-specific four-byte Charge recovery rule, with absent H unresolved.
   Keep source retention/display/edit/loss authority closed, duplicates strict and all49
   metadata/catalogue exact. Complete actual adapters/Unicode files/native scope, bounded
   fuzz and local gates before committing/pushing. The current public omitted-run evidence
   does not justify general
   typography inheritance; keep it required/open and continue other unblocked M1 work.
3. Broader CDX/styles/rich labels/interchange, remaining M2 drawing/selection/journal tools,
   M3 chemistry/reactions, M4 publishing/clipboard and M5 naming/templates/WASM/installers
   remain required. M0 platform/input/accessibility/DPI/jobs/addressability and historical
   listener cause remain open. Human NH2 golden approval remains pending. Full M0–M5 stays
   active/incomplete; push each atomic commit and keep Actions OFF.

### Previous printed-label creation checkpoint

Recoverable printed-label creation: base7113237, contract126f99e, renderer8f1225e
(isolated62b9841), original fixtures9acd1a5, IObb30dca (isolated33092ca), wire8c37996,
oracles38283d4 (isolatedd8b8d97), command/UI091328d and fuzz a186933. Read
[creation contract](docs/research/printed-label-creation.md) and
[receiving](docs/verification-printed-label-creation.md). This supersedes all historical
next-task/job paragraphs and the persistent goal's old line-width first task.

New49 creates a child label on a qualified current-hidden imported carbon with an
explicit style and existing resources. Flags/identity/geometry stay; Element/H/body/box
change together in one source/page-certified edit. Separate Flagged authority survives
partial/all-printed Save/reopen and further48 edits/movement. New source resource checks
and current hidden proofs precede the old native DATA codec; no fallback authoring.
Immutable-baseline reservations include child Text/captions but exclude font IDs, so
reverse creations never rename earlier children. Old Boxed/Connected/Mixed/full/loss
guards and every unrelated authoring refusal stay literal. Actual old48 byte/schema/
order prefix45486B/SHA256c861b31ce690ec5808e6262884dc7195e4e7e796f0e2ec6dcf47e082de9ad1da
and catalogue16/124 stay exact; no grey tool gains a binding.

Original20sources/64FontToolsboxes/eight-reader controls, renderer195/0, IO502/0, new
command8/App1/wire3/render3 and exact-source history pass. Source-aware rendering covers
15 whole CPU/GPU frames/60 bonds/five painted cases, with three predeclared uncertain
pixels and unchanged old masks. Native Mac retained8-Ring→Open→100%→hidden Alt creation/
Cancel/fresh/Apply→Undo/Redo→Unicode partial Save/reopen→old48 N/H1/history→remaining
creation→Selection/all-five(24,-12)drag/history→final Unicode Save/reopen passes exact
independent model/files. Final2279B SHA2565690e2fe8ce05ebdf5cb8d96399e49e1e86aef387ecd2aa2f677f2f848c92a7e;
CPU/Metal1088 PNGs byte-exact29110B/SHA256
d9b8c028f1bf76f6d3e7ccce195897ffb53134ea2c1cdfb59a6f490610318fb4.
AX truncation/coordinate-focus setup qualifications and CJK filename-glyph limits stay.

Fresh final27673 PASSES workspace2122/0/1 across314 reports plus vendor3/0 and all local
fmt/clippy/layers/licenses/seven-assets/pinned-corpus gates; fuzz fmt also passes.
Log266746B/SHA256d5c2ab9d8f6a2ede7761986453f0210e8981e624636d473dba5df417041e9e27.
Initial28685 has twelve stale CLI inventory failures; qualified51825 has one intentionally
superseded hidden-Open negative. Exact49 inventory/prefix and stronger nonauthoring
creation-style/Cancel checks correct them, with old48 hidden and other guards unchanged.
The earlier duplicate test module lint and wire slice lint are test-only qualifications.
No causal historical MCP listener fix is claimed; protected test/policy stays exact.
ASan22691 passes10000/305s, cov8796/ft20218/RSS513MiB,34 immutable sources and four actual
16KiB mutations. Manifest af870a2cab7b5632cb6650d70aee37be1c56e71d925dfdc9533d537fefe5eeaf
reproduces exactly; log111425B/SHA256fd25628a3e92a2d0a806402f6447dc7a8c200b67c9034e06b17898fb681cdd08.

All execution jobs/workers are terminal; isolated branches/worktrees remain for provenance.
The NEW own `target/label-creation/Bondline Label Review.app`, PID365, is LEFT OPEN:
the user manually changes zoom to158% after final reopen, then confirms they stopped.
Preserve this now-user-viewed review, its files and all older protected apps/drawings.
No final reopened-draft Cancel/quit is claimed. No dependency/lock/vendor/font/asset/
old golden/workflow change; full M0–M5 and human golden approval remain incomplete.

The user explicitly requires push-after-EVERY-atomic-commit. Code091328d and fuzz a186933
are pushed on `feat/multiple-bonds`, now tracking the same origin branch. Keep this
receiving record/next changes synced too; never force-push or infer merge/release approval.
Actions is confirmed OFF through repository API and stays OFF during development.

Next executable work:

1. Record any actual reply before promoting the older NH2 image to a human golden;
   approval remains pending and does not block independent implementation.
2. Source-lock effective foreign label typography: omitted Style-run font/size/face may
   use ONLY explicit, supported per-property LabelFont/LabelSize/LabelFace owners already
   represented by standalone lookup. Declare shadowed/unsupported/unresolved/resource/
   placement and exact-source/history/native vectors before widening complete authority.
   No external stylesheet/default,−1-as-inherit, universal baseline or producer exception.
   Broader resources/fonts/rich runs/child placement remain required; do not repeat the
   delivered child insertion/flag/ID workflow or weaken unknown-source/loss guards.
3. Continue general CDX/interchange, remaining M2 drawing/selection/journal tools, M3
   chemistry/reactions, M4 publishing/clipboard and M5 naming/templates/WASM/installers.
   M0 accessibility/input/DPI/platform/jobs/addressability and historical listener cause
   remain open. Keep the full goal active and push each tested atomic commit; Actions OFF.

### Last integrated Mixed skeletal workflow

Recoverable Mixed skeletal workflow: basef16b79f, contracts b30f0d6/5f58276, shared
renderer query bb719c9, independent fixtures71129ab, IOc3cc351 and render oracles1473353,
plus the receiving command/UI commit containing this record. Read
[contract](docs/research/mixed-skeletal-labels.md) and
[receiving](docs/verification-mixed-skeletal-labels.md). This supersedes every historical
next-task/job paragraph and the persistent goal's old line-width first task.

A separate Mixed capability admits explicit printed labels and explicitly/currently
hidden no-Text carbons, with literal C4 and neutral/natural/nonradical metadata. Source
children/keys/resources/ink/styles/scoped IDs and current geometry are fully checked.
Shared classification reuses old glyphless/mandatory rules; getter/writer re-prove after
point/neighbor changes so no default font can appear. Old foreign Boxed remains all-
explicit/zero-bond and Connected all-explicit; globalfullfalse and every other loss/
authoring guard stay. Existing48/form edits printed labels; Selection moves both kinds.
Hidden-node label creation stays refused. All48 actual metadata remain byte-exact45486B/
SHA256c861b31ce690ec5808e6262884dc7195e4e7e796f0e2ec6dcf47e082de9ad1da;
catalogue16/124 stays, no grey tool is enabled.

Independent source9/font boxes16/reader refusals11, renderer query5, IO14, command3
(including16 fixed-seed history sequences), App2 and actual CLI/initialized stdio/socket3
pass. Source-aware independent render checks cover21 whole CPU/GPU frames,75 bonds and
five painted cases with exactly one predeclared uncertain black pixel. The physical2x
1152 interior/actual1153 qualification stays explicit; no data/tolerance/old test changed.

Actual own Mac retained Ring→Open→Actual100%→hidden Alt refusal→printed OH→N/H2 Apply→
Marquee/CmdA→all5-atom hidden-anchor(24,-12) drag→Undo/Redo→Unicode Save As→explicit
reopen→printed draft Cancel passes. Native1736B output equals frozen edited_all_p24_m12,
SHA25676ebdbe5f11486e48719a552470388d7fe7ef2b97af7e6896846ca9467e27942, with independent
actual-file Expat/Decimal/constructed+direct RDKit and exact model/flags/IDs. Attached
CPU/Metal1088 PNGs are byte-exact28920B/SHA256
9c3492c014fc51846b1f99e1e22140fe99c1d3b403d6e4c080a5c6eff2345156. SDK key spelling/AX
truncation setup qualifications and CJK filename-glyph limits remain. Own review60177
is explicitly quit/PID absent; older protected apps/drawings remain untouched.

Fresh aggregate7935 PASSES all310 reports: workspace2091/0/1 plus vendor3/0; workspace
fmt/warnings-denied clippy, vendor clippy, layers, strict runtime licenses/seven assets
and pinned eight-file/five-bundle corpus. Separate fuzz fmt passes. Log264235B/SHA256
8282b884b83bb7377d19799b12ea979c3208f9f81fb5aba1ee4301b289480dc1. This is a clean local
aggregate; the preceding MCP bind failures remain unexplained, not causally repaired.
Protected control test/policy remains exact. Qualified ASan26920 passes10000/203s,
cov8392/ft17830/RSS523MiB, actual16KiB mutation and23 exact immutable sources; fresh seed
regeneration is exact. Its initial undeclared-render compiler miss is a harness-only
qualification using existing model constructors/frozen NH2 bounds, no new dependency.

All owned jobs/workers are terminal; own review is closed and isolated worktrees retained.
No dependency/lock/vendor/font/asset/old-golden/workflow change, push or publication.
Actions OFF, full M0–M5 stays active/incomplete.

Next executable work:

1. Record any actual reply before promoting the earlier NH2 image to a human golden;
   approval remains pending and does not block independent implementation.
2. Source-lock explicit printed-label creation on hidden imported carbons, including
   deliberate authored font/size/colour versus unresolved foreign defaults, public child
   insertion/IDs/resources and complete source/model/history/native transaction. Declare
   independent byte/layout/chemical/owner/refusal vectors before changing the current
   hidden-target refusal; preserve old48 schema/order and unknown/whole-format guards.
   Broader effective foreign fonts/defaults/styles remain a parallel required M1 workstream.
3. Finish CDX/general interchange, then all remaining M2 drawing/selection/journal tools,
   M3 chemistry/reactions, M4 publishing/clipboard and M5 naming/templates/WASM/installers.
   M0 accessibility/input/DPI/platform/jobs/addressability and the unexplained historical
   listener failure stay open. Do not repeat exhausted listener diagnosis or enable Actions.

### Preceding dependent-label movement checkpoint

Recoverable dependent-label movement: base9cf7103, IO783a632, renderer6bda2d9
(isolated321e1b2), SDF test-only readiness edbd837 and the receiving command/UI commit
containing this record. Read [contract](docs/research/dependent-label-translation.md)
and [receiving](docs/verification-dependent-label-translation.md). This supersedes earlier
connected-label and persistent-goal line-width first-task/job text; full scope is unchanged.

Existing selection/translation moves qualified Boxed/Connected atom points and CURRENT
raw label boxes together. Rounded-identical points preserve the entire atom/box/Arc/redo;
changed previews prove complete source and the final physical page before returning the
same commit candidate. Source writers splice existing p/label leaves and re-prove current
Connected geometry. Ring/chair/chain keep their old native predicate; other source/loss
and modal/file/view/input guards stay. Exact Alt-form targeting precedes Selection.
Actual48 metadata remain byte-exact45486B/SHA256
c861b31ce690ec5808e6262884dc7195e4e7e796f0e2ec6dcf47e082de9ad1da;
catalogue16/124 stays, no planned tool is enabled.

Independent IO476/0, focused commands8/App3/adapters3, source review, exact raw/chemical/
font consumers and native Mac subset/all drag/nudge/history/Unicode Save/reopen/Cancel
pass. Initial subset/all fixture naming is explicitly qualified without changing frozen
bytes. A native all-atom capture exposes nine real GPU edge pixels. Exact Fraction
oracles reproduce all nine; bounded analytic row geometry fixes them while preserving
CPU math and real ordered GPU compositing. Full renderer186/0 passes. Row data costs at
most64MiB host plus64MiB GPU, with other buffer/allocator overhead separate; no measured
acceleration claim. Final rebuilt native CPU/Metal1088 PNGs are byte-exact27397B, SHA256
eda39c081d72330fecab5c391222c3e72065a0219565d208a0657e4a1ed5805c, equal to the retained
pre-fix CPU output. Both own reviews are explicitly quit/PIDs87125 and35667 absent;
protected older apps/drawings are untouched. Filename CJK glyphs remain limited.

Final aggregate3289 fails1327/1 at unchanged MCP replacement_bind58656/AddrInUse.
Remaining32361 passes737/0/1 including vendor3/0 and every remaining gate. Exact workspace
artifact audit99124 additionally passes475/0/1 for20 not-yet-observed full-workspace
variants; one redundant example/variant runs are excluded from unique counts. All293
workspace executables and12 metadata-derived doc groups are observed: unique workspace
2061/1/1 plus vendor3/0. Workspace fmt/clippy, vendor clippy, layers, strict runtime
licenses/seven assets, pinned eight-file/five-bundle corpus and fuzz fmt pass. This is
NOT a clean aggregate or causal listener fix; protected control test/policy is unchanged.
Earlier obsolete-selection and SDF readiness qualifications remain in receiving.
ASan54835 passes10000/161s with28 immutable sources and actual16KiB; independent fresh
seed regeneration is exact. No dependency/lock/vendor/font/asset/old-golden/workflow change.

All owned execution jobs/workers are terminal. Isolated IO/render worktrees are retained;
no review window is left running. Actions OFF, no push/publication. The full goal stays active.

Next executable work:

1. Record an actual reply before promoting the earlier NH2 image to a human golden;
   approval remains pending and does not block independent implementation.
2. Continue M1 by source-locking mixed labelled/skeletal foreign diagrams and effective
   font/ink/default ownership. Declare independent source/layout/history/native vectors
   before widening a complete capability; preserve current explicit Boxed/Connected,
   arbitrary-child-p/style refusals and whole-format loss boundaries. Do not infer hidden
   carbon/font defaults from producer names or replace conflicting child Text numerically.
3. Finish broader styles/fonts/text, CDX/general interchange, then all remaining M2 drawing/
   selection/journal tools, M3 chemistry/reactions, M4 publishing/clipboard and M5 naming/
   templates/WASM/installers. M0 accessibility/input/DPI/platform/jobs/addressability and
   the unexplained listener failure remain open. Do not repeat exhausted listener diagnosis
   or enable Actions. Full M0–M5 remains active/incomplete.

### Preceding connected-label checkpoint

Recoverable connected-label workflow: data0014935, render0ba92cf, IO74375bd and
independent oracle7500599, plus the receiving command/UI commit containing this record.
Read [contract](docs/research/connected-label-styles.md) and
[receiving](docs/verification-connected-labels.md). This continuation supersedes the
persistent goal's historical line-width first-task text and older next-task/job paragraphs.

A separate Connected capability admits explicit all-labelled, noncrossing foreign
Single/centered Double/Triple diagrams with proven root ink/margin/spacing/colour.
Old foreign Boxed remains zero-bond; global full staysfalse. Actual-outline Euclidean
butt-strip shortening/common rail cuts replace the temporary margin refusal, preserving
old None pixels/glyph placement/Atom hits. Only existing command48/form may synchronize
Element/H/body/box, certify final source/page and commit once. Other mutators remain
refused. All48 metadata remain byte-exact; catalogue16/124 stays, no grey tool is enabled.

Independent FontTools128 cases, external line/curve/dense/Pillow controls, Expat/Decimal/
direct RDKit, exact-source generated history, actual CLI/stdio/socket and CPU/Metal pass.
Actual own Mac Open from retained Ring, Option-edit, history, Unicode Save As/reopen,
Cancel and read-only drag pass; actual saved1291B source equals the independent fixture.
Controller focus/typeText setup qualifications and CJK filename-glyph limits remain.
The disposable review is explicitly quit/PID50034 absent; protected apps are untouched.

Fresh aggregate12224 fails1300/1 at unchanged MCP replacement_bind56994/AddrInUse.
Not-yet-run components62118 pass732/0/1 including vendor3/0, all remaining gates and
fuzz fmt. Combined unique execution is workspace2029/1/1 plus vendor3/0. Workspace
fmt/clippy pass, but this is NOT a clean aggregate or a causal listener repair. Bounded
ASan63302 passes10000/151s, actual16KiB mutation and25 exact immutable sources. Full
pins, conditions and scope are in receiving; no dependency/lock/vendor/font/asset/old
golden/workflow change. All owned jobs/workers are terminal; isolated worktrees remain.

Next executable work:

1. Record an actual reply before promoting the earlier NH2 candidate to a human golden;
   approval remains pending and does not block independent implementation.
2. Continue M1 dependent labels: source-lock literal Node+ink-box translation together,
   declare independent exact-source/geometry/selection/history/refusal vectors, and extend
   supported movement through existing guards only after final page/source certification.
   Preserve both complete capabilities, unsupported child-p/styles/defaults and all other
   loss/authoring boundaries. Arbitrary chemical layout/face conventions remain unresolved.
3. Finish broader foreign styles/fonts/text, CDX/general interchange, then all remaining
   M2 drawing/selection/journal tools, M3 chemistry/reactions, M4 publishing/clipboard and
   M5 naming/templates/WASM/installers. M0 accessibility, input/DPI/platform acceptance,
   jobs/addressability and the unexplained listener-release failure remain open. Do not
   restart exhausted historical listener diagnosis or enable Actions. Full M0–M5 stays active.

### Preceding margin-data checkpoint

Recoverable data checkpoint: optional physical label-margin data on base61be8e6, in the
commit containing this record. Read [contract](docs/research/connected-label-styles.md)
and [data receiving](docs/verification-margin-data.md). Root-only MarginWidth now retains
independent optional0..100000pt values through strict model/JSON/native CDXML and per-field
source splices; true signed zero differs from absence, negative nonzero underflow stays
opaque. Generic MOL/CDX/SMILES loss guards include the field. Old48 actual registry
records and catalogue16/124 remain exact. Explicit margin on a bond-bearing page has a
temporary renderer refusal, so the new data is never silently ignored.

Actual existing-API IO0/1 RED and model missing-API qualification precede producers.
Fresh full97406 passes1999/0/1 across299 result lines and all local gates; bounded
ASan67587 passes10000/125s, actual16KiB,18 immutable sources exact. Historical MCP failure
remains unexplained, not fixed by this natural passing aggregate. No native/user app,
dependency/font/asset/old golden/workflow change; Actions OFF.

At that checkpoint the data did NOT complete the connected workflow. Three parallel lanes
continue the outline renderer, a distinct Connected source capability and independent
FontTools/paint oracles; all workers wait for this tested parent before producer edits.
No Root execution jobs remain live. Keep foreign Boxed literally all-explicit/zero-bond;
the future Connected grant needs explicit effective ink/margin/spacing/colour and pure
geometry proof. Implement actual-outline Euclidean butt-strip clearance/common rail cuts
from the contract, preserving old glyphs/hits/None pixels and real-ink guards. Wire existing
command48/form only after complete source/render/history/Unicode/native evidence. Exact
Roboto reference pixels, other fonts/defaults and the full M0–M5 target remain open.
The older boxed-label checkpoint below retains its full verification but not its next
source-work/job paragraph as the newest continuation.

### Preceding boxed-label workflow checkpoint

Recoverable state: the box-positioned label workflow commit containing this record, on
base3b4dc17. Model5bca3b0, render2b89f7a, IO1aecd5a, renderer fix5611a03 and
foreign-default fix44896bc are integrated.
This section supersedes every historical next-task/job paragraph and the stale line-width
continuation inside the persistent goal. Full M0–M5 scope is unchanged.

Read [contract](docs/research/boxed-atom-label-workflow.md) and
[receiving](docs/verification-boxed-atom-labels.md). Optional checked BoundingBox data,
exact curve/GPOS layout, authored colours/masks/Atom hits/em bounds and a separate complete
source capability now connect imported display to synchronized label editing. Only explicit
no-p/Left/Left/Roboto Regular face0 canonical Element/H labels qualify. Foreign scenes
require explicit labels on every atom and no bonds; missing styles never borrow native
defaults. Exact-native/source-free conventions remain richer. Global full
authority staysfalse; unqualified labels and all other dependent/page/style/caption edits
keep their refusals. Unknown data never gains authority from a producer name.

Registry48 appends atom.set_authored_label with the old AtomSetLabel shape. All old47
actual serialized records/schema/order retain their exact44405B literal prefix. Native
ui.atom_label chooses48 under the same weak owners and common dispatcher; the qualified
canvas takes event-local Alt-primary before a retained mode without authoring a ring/chain.
Catalogue16/124 stays, Text tool grey. Source edits change only existing Element/H/body/box,
certify re-import plus one final page and commit once. No-op/redo/Save lease/dirty/history
ownership and every other source/loss guard remain.

TDD model API/IO/command/App REDs become GREEN. Independent FontTools/Expat/Decimal/Pillow/
unsanitized no-implicit-H RDKit checks, actual CLI/initialized stdio/authenticated socket
and Unicode files pass. Seven command tests include24 fixed-seed generated edit sequences
with independent numeric/body consumption and exact whole-source undo/redo. Renderer175/0
includes two real combined text-work RED→GREEN fixes; independent source review confirms
closure. CPU/Metal1024 captures match exactly; human golden approval is pending.

Actual own signed macOS review verifies Ring→Open→Option-target→N/H2 Apply→Cmd-Z/Redo→
Unicode Save As→explicit reopen→draft Cancel→read-only drag→O/H1 Apply→known Save. Both
actual saved versions pass independent readers and exact four-field source comparisons.
The second603B output equals the original OH source exactly. Controller field-focus/key
setup and CJK filename-glyph limitations are qualified, not product fixes. The disposable
review is explicitly quit and its exact-path processes are absent; older user apps/drawings
remain untouched. No general native race/DPI/other-platform or full26.1 claim follows.

A final independent review catches foreign unlabelled glyph/bond defaults. IO actual
RED2/4→GREEN6/0 and command RED0/1→GREEN precede/qualify44896bc. Three old broad positives
are narrowed while retaining all negatives, plus four supported structural-drift controls.
No producer-string exception; source-free/exact-native richer controls remain positive.
The final rebuilt native app refuses an unproven701B diagram, then Option-edits a qualified
file and Save/reopens the exact596B NH2 output through its attached control adapter; both
actual-session CPU/GPU captures equal the displayed candidate. Both reviews are quit and
exact-path process checks report none. Earlier full native chooser/history/Cancel/Save
scope remains separately pinned to its preceding build, not silently relabelled.

Post-boundary76536 passes ALL local components: **workspace1989/0/1 plus vendor3/0**,
fmt/fuzz-fmt, warnings-denied workspace/vendor clippy, layers, strict runtime licenses,
seven assets and pinned eight-file/five-bundle corpus;297 result lines. Log251580B/SHA256
24f9c87e3182eab2543e8c8df4d6789184e73300785d922d52bb431d885d7ba7.
Final qualified structural controls additionally pass22/0 with IO clippy/fmt. The actual
cargo xtask ci9314 failure at unchanged replacement_bind55778/AddrInUse remains recorded,
1266/1. Earlier inventory-only qualifications remain too. Do not call this a clean xtask
aggregate or a causal listener repair; all final components pass separately.
Qualified final ASan84513 passes10000/119s, cov7520/ft14222/RSS503MiB, actual mutated16384B
and all42 immutable source hashes. Its79753B log SHA256 is
c64a3e8ac71b0897796d8ec20e7697badbea9bab7c4928a21f59b64895e473a3.
No dependency/lock/vendor/font/asset/old golden/workflow change. All owned jobs/workers
are terminal and isolated worktrees remain for provenance. Actions OFF, no push/publication.

Next executable work:

1. Record any actual reply for the displayed1024 NH2 candidate before promoting a human
   golden; older single-bond approval does not cover this scene. Do not block unrelated work.
2. Continue M1 with a connected, source-backed foreign label/style workflow. First lock
   explicit bond ink/gap/margin and relevant spacing/preferred-length ownership plus
   effective fonts for any unlabelled nodes against published sources. Declare independent
   connected foreign/render/hit/source/history/native vectors before admitting bonds or
   borrowed defaults. Preserve the current all-labelled/no-bond foreign boundary until
   that proof exists. Literal Node+box translation is a separate bounded dependent-label
   task through existing selection/history guards. Arbitrary child p, inherited defaults,
   formula/script faces and the full Text editor remain open; do not invent a baseline.
3. Finish full CDX/general interchange and effective foreign fonts/styles/text; all remaining
   M2 drawing/selection/journal styles, M3 chemistry/analysis/reactions, M4 publishing/
   clipboard/printing and M5 naming/templates/WASM/distribution remain required. M0 full
   inventory/addressability/jobs, accessibility/native races/DPI/platform evidence and the
   unexplained listener failure stay open. Full M0–M5 remains active/incomplete.

## Historical checkpoints

The following statements retain their checkpoint scope and do not replace the current
continuation, command inventory, source capability or job state above.

Recoverable state: the atomic owned Node child-Text integration containing this record,
based on `a546e46` with isolated model fdaec40 integrated as `1bf29d2`. Full M0–M5 remains
active/incomplete. This section supersedes every older next-task/job list and the stale
line-width continuation in the persistent goal history.

### Owned chemical label data integrated; fresh full local gate passes

Read [contract](docs/research/owned-atom-label-data.md) and
[receiving](docs/verification-owned-atom-label-data.md). Atoms optionally own literal
ordered text runs/raw optional placement and independent justification/alignment, separate
from numeric Element/H and Page caption identities. Combined logical text budgets and
globally deduplicated Arc/capacity history accounting are checked. Fragment/Document stay
Serialize-only; Atom/AtomLabel decoding is strict. Old absent-label JSON/native bytes stay.

Best-effort explicit Node child Text projects without new style/default/geometry guesses.
Unsupported/missing/unresolved/multiple text stays raw/opaque. Native encoding preserves
runs, resources, placement absence and positive source IDs. Any authored label revokes full
chemical authoring/display, even on source-free/exact native data. All current dependent
edits/page controls/caption edits and direct render paths refuse; MOL refuses loss, strict
retained-SMILES comparison includes labels and Preserved deltas cannot inject/drop them.
Raw native model encoding is a data codec, not registry/UI editing authority.

Actual model API, existing-API IO and PageSetSize runtime REDs precede their producers/
guards. Final model 8 / IO 9 / commands 2 / adapters 3 new cases pass, with independent literal JSON,
Expat/Decimal, all 23 enum names, absence/empty/namespace/opaque cases, full bounds/heap,
source/history/no-op/refusal and actual CLI/initialized stdio/authenticated socket Unicode
unchanged Save/reopen. A new test's wrong closed-source producer qualification is corrected
to sparse PageID91 without changing product policy. The compound pre-fix IO RED stops at
V2000; no distinct pre-fix renderer RED is claimed.

Fresh full 61058 exits0: **1941 passed/0 failed/1 ignored**,291 result lines; all fmt,
warnings-denied workspace/vendor clippy/tests, layers, strict declared runtime licenses,
seven assets/notices and eight-file/five-bundle corpus gates pass. Log245870B/SHA256
431e6e92810e8307c74f1f97a3083ba64dea8d814dcf0be42d07a999db5c4257.
Qualified ASan 5091 passes 10000/100s, cov6228/ft12893/RSS508MiB and actual mutated16384B.
All 54 original586..16384B seeds remain exact. Initial fuzz compile only qualifies the
new harness's scene export path; no producer/dependency change addresses it.
The preceding MCP failure stays unexplained, not fixed by this natural passing aggregate.

Actual CLI 47 discovery remains 44407B/SHA256
8d5a7031cfc0ce8e444878ed058ee6080dcff601f48cdfd8149648eee0b575df; catalogue 16/124 stays.
No new command/tool, dependency/lock, vendor/font/asset/golden/workflow or native window
change. Protected user apps/drawings are untouched. All Root jobs/workers are terminal;
isolated model worktree/branch retained for provenance. Actions OFF, no push/publication.

Next executable work:

1. Continue M1 with supported Node-child layout and numeric/text semantic agreement,
   using pinned public evidence rather than a universal p-to-baseline guess. Source-lock
   effective font/default/face and placement conditions; declare independent foreign/render/
   source/history/Save vectors before widening current authoring/display authority.
2. Implement synchronized source-preserving supported label edits through the shared
   registry. Every H/Element/position mutator (atom/bond/chain/ring/sprout/selection) must
   update dependent text or keep its explicit refusal. Prove actual masks/hits/em/DPI,
   source/current/saved/history ownership and real native Text editing before enabling
   the grey tool. Do not duplicate delivered resource/default/SetText/owned-data work.
3. Continue full CDX/general interchange, remaining M2 drawing/selection/styles, M3
   chemistry/analysis/reactions, M4 publishing/clipboard/printing and M5 naming/templates/
   WASM/distribution. M0 complete inventory/addressability/jobs, native races/accessibility/
   DPI/platform evidence and the unexplained listener failure remain open. Windows/Linux/
   browser/new native/release execution remains unverified. All M0–M5 stays active.

Recoverable state: the atomic source-backed label-default lookup integration containing
this record, based on7747111. Full M0–M5 remains active/incomplete; this section supersedes
all historical next-task/job lists and the stale line-width continuation in the goal history.

### Label-default provenance integrated; fresh full local gate passes

Read [contract](docs/research/label-default-lookup.md) and
[receiving](docs/verification-label-default-lookup.md). The standalone CDXML view resolves
LabelFont/LabelSize/LabelFace independently along legal containment owners, retaining
source nodes and decoded attribute values. Signed scalar selectors remain distinct from
unsigned run/resource IDs. Absence remains unresolved; positive finite XML size is an
original checked reader policy. Strict namespace/identity/unsupported-owner/shadowed-value
checks prevent guessed defaults. No model/IO/render/command/native authority or tool is added.

Source research establishes child Text precedence over redundant chemistry, missing-run
fallback gaps, DTD/archived justification conflict and conflicting qualified placement
conventions. Do not render authored child Text as derived Element/H or assume a universal
baseline. The Geometry SDK link serves ObjectTag; that lookup owner remains unsupported.
Primary/public source/license pins and limits are in the receiving/source registers.

Initial API compile RED precedes production. Nine independent literal/Expat/Decimal,
provenance, domain, namespace/ownership and exact-source edit cases pass. One new test's
None-only assertion for explicit empty namespace is qualified to ordinary None-or-empty
and strengthened with Expat; no producer change fixes it. Source review finds no material
issue. Fresh full40637 exits0: **1919 passed/0 failed/1 ignored**,288 result lines, all
fmt, warnings-denied workspace/vendor, layers, strict runtime licenses, seven assets/
notices and eight-file pinned corpus gates. Log242691B/SHA256
8da88225b3393901040181e8cc69a19af1cf32049f3deb8929ef871cdb91abf7.
The earlier MCP replacement-bind failure remains unexplained, not fixed by this natural pass.
ASan74732 passes10000/5s,54 immutable source seeds intact, maxseed16384B and actual
NEW/REDUCE maximum8885B. Do not call that16KiB mutation coverage.

Actual CLI47 discovery remains44407B/SHA256
8d5a7031cfc0ce8e444878ed058ee6080dcff601f48cdfd8149648eee0b575df; catalogue16/124 stays.
Model/IO/commands/UI/render, dependencies/locks, vendor/fonts/assets/goldens/workflow and
protected MCP policy/tests remain exact. No native window/user drawing is accessed;
all workers and Root jobs are terminal. Actions OFF; no push/publication.

Next executable work:

1. Integrate owned chemical Node child Text and effective style data separately from
   numeric Element/H. Use the new default/resource/SetText primitives without inferring
   omitted-run fallback or a universal baseline. First lock the supported placement/face/
   ownership and semantic-agreement contract against pinned public evidence, then declare
   independent foreign/source/retention/render/history/Save vectors before widening IO.
2. Every supported H/Element/position edit must synchronize dependent authored labels or
   explicitly refuse: atom setter, bond order/extension, chain/ring/sprout attachment and
   translation all matter. Native encoding, history heap budgets, MOL/SMILES loss guards,
   actual glyph masks/hits and both logical em endpoints need coverage. Keep old legacy
   atom-setter data-only behavior exact; do not add a blanket renderability restriction.
3. Finish native Text editing/style workflows and mixed foreign diagrams; continue CDX/
   general interchange, all remaining M2 drawing/selection/styles, M3 chemistry/analysis,
   M4 publishing/clipboard/printing and M5 naming/templates/WASM/distribution. M0 full
   inventory/addressability/jobs, native races/DPI/accessibility/platform acceptance and
   the unexplained listener failure remain open. Native width/spacing, Windows/Linux/
   browser and release checks still need actual evidence. Keep Actions OFF.



Recoverable state: the atomic styled-caption workflow integration containing this record,
based on `eff16ae` (the existing pinned GPOS dependency edge). The full persistent goal covers
**all M0–M5 open points**, not just this increment. This section supersedes every older
next-task/job paragraph, including the stale line-width continuation in the goal's history.

### Explicit plain styled captions integrated; components pass, aggregate failure retained

Read [contract](docs/research/styled-captions.md) and
[receiving](docs/verification-styled-captions.md), with the isolated model/parser/render
receiving records. Checked owned runs and native font/palette encoding now preserve caption
data. One complete explicit foreign caption-only profile can Open→render→edit run through
the shared registry→history→Save/reopen with every unrelated source byte intact. It keeps
global chemical completeness false; foreign page/style/chemical edits remain refused.
Source-free/native text has real encoding. Root source-aware, whole-MOL/CDX/retained-SMILES,
font/glyph/scale/paper/work, no-op/close/Save-lease and full-snapshot history guards stay.
A reviewed Caption→Canonical self-closing expansion refusal has actual RED→GREEN.

Registry47 appends text.set_run; actual old46 serialized metadata/schema/order prefix stays
byte-exact, catalogue16/124 unchanged. Three actual CLI/initialized stdio/authenticated
socket cases, independent Expat/Decimal/FontTools/Pillow, generated24-case history,
CPU/Metal and four App cases pass. Bounded actual Mac own-window Open, remote-edit repaint,
keyboard Undo/Redo, read-only drag and Unicode Save As/reopen pass. The391-byte saved file
has only the edited run delta and independent exact font/size/colour/position evidence.
The full native Text editor/tool, chemical formatting and mixed foreign diagrams are missing.

Initial full5056 fails1217/1/0 at the unchanged MCP replacement_bind58833 AddrInUse; log
154693B/SHA25656902ca0d24c9e403f7229cfcb2507b401495591acaa2e42bb5b712d104f0cd4.
Complete no-fail-fast workspace74577 records1906/1/1 at one obsolete UI registry46 assertion;
log238678B/SHA256548d5e10cf20c244bb3b190b129ebe19d6886458e3dfa6f44e4f6520debda3f6.
Its unchanged MCP test passes naturally, explaining no cause. The exact current UI count
is corrected; qualified11820 passes whole UI322/0/1, vendor3/0 plus clippy, workspace
warnings-denied clippy, rustfmt/layers/strict licenses/seven assets/notices/pinned corpus.
Remaining log40517B/SHA256c36f6ae8b8e8868685c9ed1a03fdaf9e3ddce6bd7ca49325e313955f1090a95b.
All components pass separately; **no clean aggregate or causal MCP fix** is claimed.
No listener policy, assertion, timeout, retry or exhausted diagnostic is changed.

Qualified parser ASan passes10000/6s and profile passes10000/87s; both actual NEW/REDUCE
reach16384B. All42 source seeds per directory stay exact. Initial new-harness namespace
expectation failure and its API qualification are retained; the product/qualified data
retention remains unchanged. Root/fuzz locks differ from b34 only by eff16ae's already
pinned ttf-parser edge, with no package/version changes. Workflow, protected MCP test,
vendor/fonts/assets and old goldens stay exact. All owned jobs/workers are terminal;
completed isolated worktrees remain provenance. The own review is observed closed;
older user apps/drawings stay untouched. Actions OFF; no push/publication.

Next executable work:

1. Continue M1 effective chemical LabelFont/LabelSize/LabelFace and Node child Text
   ownership. Source-lock inheritance/default/charset and chemical-display semantics;
   declare independent foreign/XML/retention/render/history vectors before widening
   the closed profile. Do not restart delivered caption/resource/SetText primitives.
2. Finish the real native Text tool/editor and richer supported style workflows through
   shared commands and source-aware complete candidates, with actual OS evidence. Mixed
   foreign diagrams, other fonts/faces/shaping/layout and CDX text mapping remain required.
3. Continue all broader CDX/interchange, M2 drawing/selection/styles, M3 chemistry/analysis,
   M4 publishing/clipboard/printing and M5 naming/templates/WASM/distribution. M0 full
   inventory/addressability/jobs and native race/DPI/accessibility/platform acceptance,
   including the unexplained MCP listener failure, remain open. Native width/spacing,
   Windows/Linux/browser and release checks need actual evidence. Keep Actions OFF.




Recoverable state: the atomic `feat(styles): support explicit document bond spacing`
commit containing this update, based onb377456. Current continuation supersedes all
historical task/live-job lists below.

### Explicit relative spacing integrated; full local gate passes

Read [contract](docs/research/document-bond-spacing.md) and
[receiving](docs/verification-document-bond-spacing.md). Optional root spacing and width
have independent source-preserving deltas. Shared rendering uses percentage × preferred
bond length, with actual ink/gap/paper/hit checks and unchanged label metrics/positions.
Absent spacing retains exact legacy behavior. Command46 keeps its old4 required keys
and adds optional spacing: omit preserves, null clears, number1..100 sets. Pair/Apply3
preserve it; the four-field form uses the same bounded ordered/currentness/Cancel rules.
The final settings/style candidate is certified once, including active reference-length
changes. Actual CLI/stdio/authenticated-loopback Unicode files, independent Expat/Decimal/
RDKit, history/source/no-op/error tests, App input and twelve analytic CPU/GPU captures pass.

Fresh full12360 exits0: **1861 passed/0 failed/1 ignored**,278 result lines, all fmt,
warnings-denied workspace/vendor, layers, strict license/asset/notice and pinned corpus
gates. Log234592B/SHA256
`b632b34f1886ad042d79de27991b05b9dff1f8f8e3cd0da44822f6a60f0b3927`.
First14853 stops238/1 at an old dialog-response expectation; two exact expectations now
include the new nullable field. The failed log is retained; no guard is weakened.
The historical MCP replacement-bind failure remains unexplained and is not claimed fixed.
ASan43341 passes10000 runs/65s, actual NEW/REDUCE16384B,37 immutable seeds unchanged.

Registry46 and catalogue16/124 remain. Old45 metadata, locks, workflow, listener tests/
policy, vendor, fonts/assets and existing goldens remain exact. Fresh controller startup
still fails; new native width/spacing interaction is **wired/unverified**. No user app or
drawing is touched. No owned live job or worker remains; isolated data/render worktrees
and commits remain completed provenance. Actions OFF; no push/publication.

Next executable work:

1. Continue M1 effective fonts/colours/Text and style ownership. Source-lock the public
   LabelFont/LabelSize/LabelFace/LabelColor and Caption/Text contracts against the SDK,
   existing resource/SetText primitives and bundled-font capabilities. Declare original
   licensed vectors and an external consumer that actually honors each tested property.
2. Implement a complete supported source/model/render/history/Save/reopen slice before
   widening foreign authoring. Keep Abs, local overrides, unknown font/default/charset
   semantics opaque; explicit native spacing is not a universal foreign default.
3. Continue broader CDX/interchange, chemistry/analysis and all M0–M5 roadmap requirements.
   Native/DPI/modifier/accessibility, Windows/Linux/browser acceptance and release work
   remain required; do not restart delivered primitives or enable Actions for evidence.



The following records describe their recorded revisions, not current tasks or live jobs.

### Resolved numeric composition integrated; fresh full local gate passes

Base6d9ce09; accompanying atomic integration commit is the recoverable state. Worker
41ab816 from isolated `bondline-formula` supplies the same code/tests, reviewed independently
before receiving. Read [contract](docs/research/formula-composition.md) and
[receiving](docs/verification-formula-composition.md). Model-only analyze_fragment returns
numeric element/isotope totals and formal charge, supports full100000-item bounds,
refuses unknown H and checks cancellation/arithmetic. Physical H and inline H are distinct;
natural isotope bins stay separate from numbered nuclides. No formula text, masses,
valence inference, chemical-validity/source certificate or Analysis command/UI is added.

Ten new tests include literal/full-bound controls and a prequalified independent RDKit
checker over13 reference molecules. Isolated chem/model57/0 and clippy/fmt pass. Root
review finds no material issue and confirms old neutral-authoring code byte-exact.
Fresh full6948 exits0:1831 passed/0 failed/1 ignored across272 result lines, with fmt,
warnings-denied workspace/vendor checks, layers, strict licenses, seven assets/notices
and pinned corpus. Log231502B/SHA256
248396a5c627c94bbb3fe6c74f905dded45715de022c4a5f789b6df3db8c5134.
CLI46 bytes/hash remain exact; catalogue16/124 and all source/command/native authority stay.

The preceding Text-stage MCP AddrInUse remains recorded/unexplained; this natural pass
does not fix it. Listener policy/tests/timing, locks/workflow/vendor/fonts/goldens remain
unchanged. All workers and Root execution jobs are terminal. Isolated formula branch
and worktree remain as completed provenance; do not restart or duplicate that worker.
No native app or user drawing is touched. Actions OFF; no push/publication.

Next return to M1 effective styles/fonts/colour/dependent Text and actual source/model/
render/history/Save integration. The Text and resource primitives below are delivered,
not a general foreign profile. Independent public spacing evidence supports centreline
separation; read the durable [evidence](docs/research/bond-spacing-evidence.md) and verify
the full source contract before implementing relative-spacing
data/render/commands, retaining Abs/inheritance/default unknowns. Actual source-aware
composition/inference/notation/mass/UI, general CDX/interchange, native/other-platform
acceptance and full M0–M5 remain required. The persistent goal stays active/incomplete.

### Source-preserving text edits; components pass, aggregate bind failure retained

Base8e030bf; accompanying atomic format commit is the recoverable state. Read
[contract](docs/research/source-preserving-xml-text.md) and
[receiving](docs/verification-source-preserving-xml-text.md). Owned SetText edits decoded
data in empty/text-only elements while preserving exact no-op lexemes, attributes/QName,
unrelated bytes and batch atomicity. Mixed child elements/comments/PIs refuse. Shared
append helpers retain old validation and descendant-first planning. No complete IO,
model/render, command/schema, native binding or imported-label authority is expanded.

Independent declarations precede the implementation; missing-API RED is compile evidence,
not a runtime failure claim. Ten new literal/Expat tests pass on first implementation,
all58 standalone cases pass and ASan18323 passes10000/2s (initial16KiB, NEW/REDUCE max13164B,
cov2951/ft8425/RSS447MiB). All35 separate original seed source hashes remain intact.
Source review finds no material issue. Actual CLI46 bytes/hash stay exact; catalogue16/124.

Initial aggregate88496 fails at the old MCP shutdown replacement bind:1164/1/0,
127.0.0.1:64249 AddrInUse. Preserve log147083B/SHA256
e651b9c781a93f2e44392f5c0171731e036217040383a614785480d926136741.
Complete no-fail-fast workspace21957 then passes1818/0/1; remaining38504 passes vendor3/0,
clippy/layers/licenses/assets/notices/corpus. All components pass, not a clean aggregate.
The natural later shutdown pass explains no cause; listener policy/tests/timing and
exhausted prior diagnostics remain untouched. Locks/workflow/vendor/fonts/goldens stay.
All Root execution jobs are terminal; no native app/user drawing is touched. Actions OFF.

At this Text checkpoint, the isolated formula worker was running on8e030bf. The newer
integration section above supersedes that live-job state; its completed branch/worktree
are retained for provenance. No Analysis workflow is advertised by this library work.

Next complete effective font/colour/style and synchronized dependent Text through model,
render, commands/history/Save/reopen before widening foreign authoring. Independent public
open-source spacing research now supports a centreline convention, while Abs/ancestry and
exact26.1 remain unverified; see the pinned source report under target/bond-spacing/independent-research.
General CDX/interchange/chemistry, fresh native and other-platform acceptance and all
M0–M5 remain required. The full persistent goal stays active/incomplete.

### Standalone CDXML font/colour resources; local gate passes

Base70e6f27; the accompanying atomic format commit is the recoverable state. Read
[contract](docs/research/cdxml-resources.md) and
[receiving](docs/verification-cdxml-resources.md). Checked borrowed resource tables now
support unique UINT16 font IDs, raw charset/name data, ordered RGB palettes, intrinsic
colours and absent-only defaults. A present one-entry palette leaves foreground3 unresolved.
Raw nodes drive existing owned edits without disturbing unrelated bytes. Syntax admission,
complete IO authority, model/render behaviour, registry46 and catalogue16/124 stay unchanged.

Three actual namespace/numeric REDs precede the fixes. Ten original resource cases and
one legacy-writer qualification use public domain/default rules, literal bytes and
independent Expat/Decimal. Initial ASan exposes an older harness local-name readback bug;
the unchanged writer passes exact/Expat proof. The corrected harness also asserts every
qualified attribute survives. Fixed replay passes; fresh qualified10000-run ASan passes
in3s, actual16KiB, cov2834/ft6839/RSS381MiB. All19 original source pins remain intact.

Final full local5632 exits0:1811 passed/0 failed/1 ignored across269 result lines, plus
fmt, warnings-denied workspace/vendor checks, layers, strict runtime licenses, seven
assets/notices and pinned corpus. Log227941B/SHA256
54c007d42f8433f18d40eb03e60b99bcf8b8e9012c3bc12c813013f4c5123aa4.
Actual CLI46 bytes/hash remain exact. Source review finds no outstanding codec issue.
Locks, protected MCP listener/test, workflow, vendor/fonts/assets/goldens stay unchanged.
All Root execution jobs are terminal; no native app is launched or inspected. Both
older user apps/drawings stay untouched. Actions stays OFF; no push or publication.

This prerequisite is not foreign-file appearance or authoring. Next add a checked owned
text-only SetText syntax operation: retain decoded no-op lexemes; replace complete inner
content rather than roxmltree's potentially partial text-node source range; reject element/
comment/PI mixed content; preserve attributes/QName; cover CDATA/entities/CRLF, self-closing
expansion, duplicate/container conflicts, bounds and atomicity with independent Expat tests
before implementation. Then integrate effective font/colour/label styles and synchronized
dependent Text through real model/render/commands/history/Save/reopen. Do not broaden the
closed IO profile prematurely. Spacing centreline/ink-gap and inherited/default evidence,
general CDX/interchange/chemistry, fresh native/other-platform acceptance and full M0–M5
remain required. The persistent goal remains active/incomplete.

### Explicit document line width; full local gate passes

The user recreated/resumed the complete M0–M5 goal after pausing/clearing the earlier goal.
Base `ab912a2`; the accompanying atomic feature commit is the recoverable state. Read
[contract](docs/research/document-line-width.md) and
[receiving](docs/verification-document-line-width.md). Optional root width now reaches
source-preserving CDXML, the pure model, actual shared CPU/GPU ink, complete-owner/stale-
camera display, native previews, paste/settings certificates, history and registered
capture. Original glyph/spacing metrics, empty JSON/native bytes and unknown-source guards
stay. Whole-MOL refuses explicit style; CDX/SMILES drift guards remain.

One appended combined settings command commits nullable length/angle/width atomically;
the old pair API/schema retains style. The existing form has three bounded fields with
the original prefix/stale/cancel/modal safeguards. No planned tool gains a binding.
Four initial actual IO/JSON/render/App REDs and a later numeric scale RED precede fixes.
Independent Expat/Decimal/RDKit/Pillow, eight CPU/Metal render cases, real CLI/initialized
stdio/authenticated socket/Unicode files and exact history/lease/guard cases pass. Source
review finds no additional material issue. Source-only declarations remain distinct from
executions. Two old LineWidth9 negatives become stronger positives while unmodeled
styles/unknown metadata retain their refusals. Old current inventories and presentation
expectations are updated explicitly; historical prefixes and old45 schemas remain exact.

First aggregate attempts fail at obsolete inventory/presentation expectations; complete
workspace15849 reports1795/1/1 at the final native-profile refusal. After its reviewed
positive upgrade, fresh full71103 exits0: **1800 passed/0 failed/1 ignored across268 result
lines**, with fmt, warnings-denied workspace/vendor checks, layers, assets/notices, strict
runtime licenses and pinned corpus. Log226886B/SHA256
58f0d907e4348e30ccf1eb4ee4d944518a879e1144c8cce12f133afb2543e782.
ASan60878 passes10000 runs/51s;26 original seeds, initial16KiB, actual NEW/REDUCE max9731B,
cov4413/ft8905/RSS473MiB. No claim of full-size/general/parser/chemistry coverage.

Actual CLI46 records occupy43338B/SHA25602bb76b9f909a3691fddbb5a1f3c2cc794a72f759f43194791b880213558a12c;
the first45 complete records/schema/order remain equal. Catalogue16/124 stays. Locks,
workflow, protected listener test/policy, vendor/fonts/assets/golden/corpus stay unchanged.
Fresh CUA startup still fails; no app/window is selected or changed. Both older user
apps/drawings remain protected. New native width and Windows/Linux interaction are
unverified. Actions OFF; no hosted run, push or publication. All owned jobs are terminal.

Next finish broader M1 source-faithful styles/font/text/default resolution using independent
foreign-source fixtures and explicit effective-property proof before expanding authority.
Then continue full CDX mapping/writing and remaining MOL/SDF/SMILES workflows alongside
necessary chemistry/model work. Revisit actual native width Open/Apply/history/Save/reopen
when controller access returns. Full M0–M5 remains active/incomplete; earlier checkpoints
below retain their own revision/evidence scope.

### User visual request: logo and planned-tool catalogue delivered

Read-only review on base `3b3c7d6` confirms the original five SVG masters, native header/
window-icon geometry and complete documented tool catalogue already delivered in `f077ed2`
and subsequent chain/chair commits. The current split is 140 entries covering 124 source
roles, with 16 delivered bindings and 124 disabled planned entries. Do not restart this
visual increment or enable tools merely because their icons exist. See the current
[identity](docs/design/brand.md), [catalogue](docs/design/tool-catalogue.md) and
[review evidence](docs/verification-branding-and-tools.md#2026-11-17-delivery-review).

The logo proof sheet and earlier actual native catalogue image were inspected from disk;
both committed native image hashes match their provenance record. No new native launch,
capture or platform claim follows. No producer changed, so tests were not repeated; the
full local 1766/0/1 checkpoint below remains the latest execution evidence. The full M0–M5
goal stays active/incomplete, Actions OFF, and public style research remains the next
implementation work. The displayed earlier capture retains its original 12/128 split.

### Source-aware closed-v0 CDXML editing; full local gate passes

Base03d5205; the accompanying atomic source/test/docs commit is the recoverable work.
The source-aware IO implementation follows [contract](docs/research/source-preserving-cdxml-profile.md)
and [receiving](docs/verification-source-preserving-cdxml-profile.md). Equivalent native-v0
whitespace/quotes/comments, numeric lexemes, sparse/scoped IDs and interleaved forward
references now support every delivered typed update/append without canonicalizing old
source. Canonical stays byte-exact first; Preserved validates every node/key/known value
and current typed prefix/domain; Opaque stays unchanged-only. Original source IDs/B/E
and unrelated bytes are retained, with final explicit ID/semantic certification. New
generated-only XML IDs are deterministic per snapshot, not persistently stable.

Actual existing-API IO and registered bond-edit REDs precede the producer. Thirteen IO
cases, independent Expat/Decimal/RDKit, actual CLI/initialized stdio/authenticated socket/
Unicode Save/history/reopen and paired current authoring families pass. The original464B
C2 file changes only Order and two H value bytes, CC/C2H6→C=C/C2H4. CPU captures of equal
canonical/Preserved models are identical and non-authoring. All31 opaque variants,12
valid-model identity/order/membership/direction drifts and canonical-only whole-MOL
controls pass. Explicit graph-only SMILES export declares geometry/context loss and
preserves the session. Four new lexical-fact tests retain all33 older syntax cases.

Three old lexical-only refusal expectations become explicit stronger positives; all
semantic/other-format negatives remain. SDK QName/root-empty-namespace and private
helper lifetime qualifications are recorded separately. First full75528 fails369/1 at
the last whitespace-only expectation. Fresh qualified full48986 exits0: **1766 passed/
0 failed/1 ignored across263 result lines**, plus fmt, warnings-denied workspace/vendor
checks, layers, seven assets/notices, strict runtime licenses and pinned corpus. Log
222486B/SHA2565436dbb7b99972b071208f0968dce919d51cdfe7ea318b88bbc704809777226b.
New IO ASan69021 passes10000/33s, actual16KiB; unchanged parser84686 passes10000/2s,
observed NEW/REDUCE maximum15502B of16KiB. Frozen review finds no material issue.

Actual rebuilt CLI45 records/schema/order remain byte-exact to the prior5dbb59f… hash;
controls39/22/16/12 and catalogue16/124 stay. No planned entry gains a binding. Locks, workflow,
protected MCP test/listener policy, vendor/font/assets/corpus/golden and both protected
user binaries retain their hashes. Natural passing tests do not explain historical
AddrInUse. Fresh CUA startup still fails; no app/window/drawing is selected or changed.
The new native path is wired, **unverified**; new Windows/Linux execution is unverified.
Actions stays OFF; no hosted run, push or publication. All owned jobs/agents are terminal.

Full M0–M5 remains active/incomplete. Next research/lock public CDXML style/default data
(line width/spacing, labels/fonts/colour and inheritance) and original foreign-program
fixtures with independent source/field/render expectations before widening the profile.
General foreign files, labels/objects, full CDX/chemistry/interchange and stable generated
IDs remain required. Revisit actual native Open/edit/history/Save/reopen when desktop
access returns; do not treat headless/capture evidence as native workflow verification.

### Earlier checkpoint: batch XML prerequisite and pre-implementation contract

The complete M0–M5 goal stays active. On base3240a1f, committed format feature1176039
adds standalone SetAttribute/RemoveAttribute/AppendChildren operations with one checked
immutable byte plan. Source quotes, unrelated/unknown bytes, semantic string no-op lexemes
and original qualified tag names stay exact. Duplicate/foreign ownership, namespace
capture and existing resource errors fail atomically. Metadata is indexed once per
element; parser admission and legacy edit/writer behavior stay unchanged. Read
[contract](docs/research/source-preserving-xml-edits.md) and
[receiving](docs/verification-source-preserving-xml-edits.md).

Eighteen new cases and all33 cdxml cases pass independent exact-byte/Expat/resource
expectations, also with the Python override unset. Shared-boundary and namespace defects
have actual runtime RED→GREEN. Test-helper lint/Root syntax corrections and the first
fuzz constant's None→Some("") API qualification remain explicit. Corrected ASan4982
passes10000 runs with actual16KiB reached. Frozen source review finds no further issue.
Fresh full70957 exits0: **1735 passed/0 failed/1 ignored across253 result lines**, plus
fmt, warnings-denied workspace/vendor checks, layers, seven assets/notices, strict
runtime license closures and pinned corpus. Log218681B/SHA256
d1ab9b942d053d3ad1793ccdc2aa5f3641baeb38d8bd1a3074089e5c744d55ae.

This is a syntax prerequisite, not imported-file authoring. Registry45, controls39/22/16/12
and catalogue16/124 stay. The original464B noncanonical CLI bond edit still refuses;
preserve that RED, source guards and whole-MOL loss permission. Next complete source↔
baseline bindings, strict supported-profile proof, typed source-aware field edits and
new-object appends for every current authoring operation, with independent file/history/
adapter/native acceptance before broadening authority. General foreign styles/defaults,
unknown semantic objects and full M1–M5 remain required. No owned execution job remains
at this primitive checkpoint. Actions OFF; no push/publication/hosted, lock/dependency/
vendor/font/golden/corpus change. Both protected user apps/drawings stay untouched.

At that checkpoint the independently prepared next [IO contract](docs/research/source-preserving-cdxml-profile.md)
was reviewed and locked, not implemented. It covers strict all-node/key/value/prolog/
namespace proof, original scoped ID bindings, current-prefix drift guards, all-current
typed edits/appends, optional-field add/remove, lowest-unused generated IDs even with
u32::MAX input and canonical-only whole-MOL permission. Per-snapshot generated-only ID
renumbering remains explicit; stronger stability and general foreign styles remain in
scope. First declare the existing-public-API positive/unknown/drift/loss tests and observe
RED. The proof also needs four lexical facts (DTD/PI/CDATA/namespace declarations) that
ordinary roxmltree nodes alone cannot establish; collect them without changing parser
admission. No IO producer or new authority is enabled by this contract. All agents and
Root execution jobs are terminal; ignored evidence remains under target/noncanonical-cdxml/.

### Logo and planned-tool recheck

The user-requested original bond-loop B, five SVG variants and native header/icon are
already delivered. The current catalogue covers all 124 documented TD/TQ/TB roles as 140
entries: 16 delivered bindings and 124 disabled planned entries. Fresh logo, inventory/font,
catalogue, foreground and workspace tests pass: 17 executions over 13 distinct cases. See
[recheck evidence](docs/verification-branding-and-tools.md#2026-11-17-current-state-recheck).
No tool is enabled by presentation alone. A fresh desktop preview cannot run this turn:
CUA reports native-pipe startup failure/browser host unavailable. Earlier actual native
captures retain their original binary/date scope. No user app/drawing is touched.

### Native selection acceptance and stale-drag fixes; full local gate passes

Base0c9e6af; the accompanying atomic source/test/docs commit is the recoverable state.
Read [lock](docs/research/native-selection-acceptance.md) and
[receiving](docs/verification-native-selection.md). The preceding goal turn is progress:
committed sprout/preferences and independently verified desktop/catalogue work. This turn
adds bounded native Mac Marquee acceptance and fixes two independently reproduced input
authority defects. Existing Atom/settings-open epochs/raw ownership now qualify selection
proofs; shared canvas cancellation retires disposable selection input before equal
surface restoration. Persistent selection, source/model/history and schemas stay intact.

Actual preimplementation command11533/App27637/supplement66346 fail at the intended
hidden-Atom/zero-surface/active-modal assertions, after ordinary no-loss controls. Unchanged
assertions then pass command46899:4/0 and App19297:14/0, retaining all old cases. Fresh full
36217 exits0: **1717 passed/0 failed/1 ignored across249 result lines**, plus fmt,
warnings-denied workspace/vendor checks, layers, seven assets, unchanged155/166/221 strict
runtime closures and pinned corpus. Log215726B/SHA256
ba1eab4167faebed304afd0ecc67593970911abc6be01658b3a0979d3db83338.
Bounded frozen producer review finds no material issue. No listener policy/test change,
exhausted probe retry or attribution of the historical MCP AddrInUse failure.

The baseline actual Mac workflow verifies atom/Shift/bond/rectangle/component/All/Deselect/
Escape, precise drag, separate nudges/history and editable Unicode Save/reopen. Six691B
outputs pass independent Expat/Decimal/RDKit CH4O+C2H6 checks. Final build26772 succeeds;
its executable SHA256 is0fa9d4ff4e8bc50da033a82c9519dedc66636653c56a6a1531b8b59a64ca20c2.
The rebuilt app additionally verifies disconnected components within one Fragment, visible
Atom/settings arrow/Cancel fences, fresh drag/history/nudges and editable Unicode reopening.
Five663B fixture/native files pass independent exact fields/coordinates/CC.CO/C3H10O;
cancelled/input are byte-exact. Both own reviews are Quit and later observed absent.
Both older user apps/drawings remain protected; no other user window is inspected.

Rebuilt CLI45-command output is byte-exact to its predecessor; controls39/22/16/12 and
catalogue16 available/124 planned stay. No new tool is enabled. A manually combined valid
XML fixture still refuses the current canonical source profile. The guard is retained;
a separate existing-registry-authored canonical fixture supplies the component test.
The initial system-Python RDKit import miss is setup only, corrected with the pinned venv.
No owned jobs remain. Actions stays OFF; no dependency/font/vendor/lock/corpus/workflow,
golden, new fuzz/parser, hosted run or publication change.

Native held-button races/additive Shift-marquee/horizontal10pt arrows, measured2×/DPI,
Windows/Linux, broader object/source editing, Lasso/transforms/styles/stereo and exact26.1
behavior remain unverified or missing. Full M0–M5 remains active/incomplete. Next return
to the user's M1 priority: inspect io/document.rs canonical-profile gate, cdxml mapping/
writer and the actual refused fixture; cite public XML/CDXML identity/order rules; lock
source-preserving support for equivalent noncanonical elemental documents. Write independent
whitespace/ID/order/unknown-data preservation and unsupported-source REDs before changing
mapping/writing authority. Do not loosen byte equality alone or promote preview/editing
without complete mappings, actual file consumers/history and native evidence.

## Earlier checkpoints

### Native ring sprout and session Preferences; full local gate passes

Base6996862; the accompanying atomic source/test/docs commit is the recoverable state.
Read [lock](docs/research/native-ring-sprout.md) and
[receiving](docs/verification-native-ring-sprout.md). All ten ring tools now route
atom-targeted Alt or preference-driven sprout through the same guarded preview/command.
File > Preferences… has one real current-session checkbox, ordered local Defaults/OK/
Cancel, stale drafts, exact cancellation and modal/retirement/ABA fences. No chemical-file
attribute, Save receipt, close ACK or restart persistence is invented.

Registry45 preserves all44 earlier complete actual CLI records/schema/order. Controls
39/22/16/12 add only Preferences; the original logo and140-entry/124-role catalogue stay
16 delivered/124 planned disabled. Nineteen new independent cases pass. Actual runtime
REDs, new fixture/closed-Inspect qualifications and the two deliberately upgraded old
Alt-atom assertions remain explicit. All unsupported old vectors/guards stay. Fresh full
42232 exits0: **1713 passed/0 failed/1 ignored across249 result lines**, plus fmt,
warnings-denied workspace/vendor checks, layers, seven assets, strict155/166/221 runtime
closures and pinned corpus. New ASan88252 passes10000 runs, max reached92B of16KiB.
Historical MCP listener failure remains unattributed; no listener policy/test change.

An isolated actual Mac app verifies Alt-sprout, Undo/Redo, preference-driven Chair2 sprout,
confirmed64pt creation setting, Unicode Save/reopen, retained session preference, Defaults/
Cancel and unchanged OK without dirtying. Its actual3897B20/21/two-fragment CDXML passes
independent Expat/Decimal and pinned RDKit C12H16/C8H16 checks; old48pt benzene stays.
The catalogue visibly shows16/124; grey Curved arrow cannot activate or change the clean
drawing. Initial background-focus/input and first filename setup misses are qualified,
not product fixes. Probe and normal review are explicitly closed and later observed absent.
Both older user apps stay running with unchanged binaries; their drawings remain untouched.

No owned jobs remain. No new golden, dependency/font/vendor/lock/corpus/workflow change,
hosted run or publication. Native all-tool/modifier/race coverage, measured2×/DPI, Windows/
Linux, CJK glyph coverage, full styles/preferences/stereo and exact26.1 policies remain
unverified or missing. Next take the highest unblocked labels/styles/selection acceptance
item from ROADMAP/parity, lock its public behavior, write independent RED expectations,
then finish its complete existing-registry/native/file workflow before changing a grey
binding. At that checkpoint the next acceptance item is TD-002/M2 Marquee: read its cited guide
pages and existing selection tests, exercise marquee/click/toggle/drag/nudge/Undo/Redo in
a separate own native app, then independently consume its Unicode Save/reopen file. Keep
the existing protected apps and report any failing first observation before fixing code.
Full M0–M5 remains active and incomplete; Actions stays OFF.

### Atomic headless sprout; full local gate passes

Baseec84717; the accompanying atomic command/test/docs commit is the recoverable state.
Read [lock](docs/research/native-ring-sprout.md) and
[receiving](docs/verification-ring-sprout.md). One ring.sprout command/preview appends an
entirely new3–8 cycle, either named ring or either chair, through one Single in the same
Fragment and one history edit. All43 old actual CLI records/schema/order stay exact;
registry44 leaves native controls38/21/15/11 and catalogue16 available/124 planned unchanged.

Thirteen new independent core/property/render/file/actual-CLI/MCP/socket cases pass.
Actual UnknownCommand REDs precede producer edits; the real positional-array schema
gap has unchanged RED/GREEN assertions. Negative-square occupancy and notation atom-order
setup corrections preserve the discovered refusal and all old consumers/thresholds.
Full local22063 exits0: **1694 passed/0 failed/1 ignored across248 result lines**, plus
fmt, warnings-denied workspace/vendor checks, layers, seven assets, strict155/166/221
runtime closures and corpus. New ASan55045 passes10000 runs, max reached264B of16KiB.
Current inventory changes are positive exact additions; historical arrays and protected
MCP policy/tests stay unchanged. The historical listener failure remains unattributed.

This is headless delivery; native Alt-sprout and application preferences remain missing.
Next immediately adopt the independently frozen existing-App/JSON tests, observe their
runtime REDs and finish the locked session preference/modal policy, ordered native form,
same-candidate click/drag and adapter repaint. Verify in a fresh own native Mac app and
consume actual saved files independently before enabling/claiming those flows. Logo and
all140 catalogue entries remain; grey entries never gain fake bindings. No user app/
drawing, dependency/font/vendor/lock/corpus/workflow, golden or publication changes.
At that checkpoint Root execution jobs are terminal; source-only next-stage preparation
continues into the separately recorded native increment above.
Full M0–M5 stays active and incomplete. Earlier native/platform/style limits remain.

### Cyclohexane-chair tools; full local gate passes

Base59e3403; the accompanying atomic source/test/docs commit is the recoverable state.
Read [lock](docs/research/native-chair-workflow.md) and
[receiving](docs/verification-native-chairs.md). Both original constitutional depictions
now have horizontal/rotated drawing, known BondLength application, terminal/spiro/Single-
edge attachment, contextual9/0 and one-edit history through the existing registry.
Source/H/ID/full-ink/paper guards remain. No stereo or conformer solver is inferred.

Registry43 preserves41 unrelated actual CLI records plus both old branches/definitions
of the two extended schemas. Controls38/21/15/11; catalogue140/124 roles is16 delivered/
124 planned. Ring weak contexts prevent hidden modal Open/Cancel ABA. Root observes the
preimplementation JSON/control REDs and ordinary-ring ABA2 RED/GREEN. A supporting-edge
RED corrects unshipped reference indexing without changing the ideal chair/camera or
weakening old geometry/targets/tolerances. Twenty-four new cases pass. Future-API,
vertical-pose, real held-key repeat, required-field and zero-case selector qualifications
remain distinct from product failures; no production repeat patch is needed.

Initial full37099/23146 stop at new test-helper lints;25793 fails176/1 at the missed
current control-ID set,54493 fails267/1 at the missed current Tools cardinality. Exact
new-ID/count corrections preserve old predicates/history arrays. Fresh full1866 exits0:
**1681 passed/0 failed/1 ignored across247 result lines**, plus formatting, warnings-
denied workspace/vendor checks, layers, seven assets, unchanged strict155/166/221 runtime
closures and pinned corpus. Log210478B/SHA256
9831752c4b58d412f454026c9fee37c4e8c1be0ef396552b0b15fa2eaa3a4ea6.
ASan2423 passes10,000 bounded ring-JSON inputs; max reached length262B of configured16KiB.
No MCP socket-policy/test change or attribution follows; exhausted probes are not repeated.
Bounded producer/fuzz source reviews find no material issue; static inventory omissions
remain explicitly recorded rather than hidden by the later executing gate.

Native/CLI build26106 and final normal build pass; copied/rebuilt native SHA256
143337fe49a2ee6df409af5aad781ab08b9b1ba07c0a5999dd3c09a826ca8330.
The isolated actual Mac app checks both horizontal/near-vertical chairs, default-labelled
terminal targets,9/0 Single-edge fusion with mode retained, Shift refusal/plain recovery,
Undo/Redo and Unicode editable Save/reopen. The first numeric entry is not confirmed;
its actual9508-byte50/50/8 file has absent preferences/48pt fallback, not64pt evidence.
A fresh coordinate-focused pass confirms64/137.5 before Apply and after reopening:
2798-byte14/13/3 file has correct root settings,64pt chairs and unchanged old C2 geometry.
Both actual files pass independent raw Expat/Decimal geometry/flags/orders/directions and
RDKit formulas. The catalogue shows16 available/124 planned; grey Curved arrow cannot
activate or change the clean drawing. Both review runs are explicitly Quit and later
inventory observes only that review stopped. Both older user apps/drawings stay untouched.

Native direct chair-tool bond-click/spiro, race paths, measured2×/DPI, Windows/Linux,
exact26.1 defaults/coordinates/hotkeys, stereo/ring flips and CJK glyph coverage remain
unverified or limited. Alt-sprout/preference, wider labels/styles and full M0–M5 stay
required. No new golden, hosted run, publication, dependency/font/vendor/lock/corpus/
workflow change. No owned execution jobs remain. Next lock the missing ring sprout/
preference flow from public sources and implement a complete guarded registry/native
slice, or take the next unblocked labels/styles item. Preserve the existing user bundles.

### Native carbon chains; full local gate passes

Base `f6f1cac`; the accompanying atomic source/test/docs commit is the recoverable state.
Read [interaction lock](docs/research/native-chain-interactions.md) and
[receiving](docs/verification-native-chains.md). Acyclic/Snaking now have native controls,
full previews/count, Add Chain, count repeat, supported-carbon attachment, conditional
ordered modifiers and one-edit history. Stored48/90 then48/120 creation preferences
affect later chains while old geometry stays. Source/H/ID/full-ink/paper guards remain.

Registry43 preserves all old42 actual CLI records/schema/order. Native controls36;
retained/SDF/raw stay21/15/11. Catalogue140/124 roles is14 delivered/126 planned. Only
Acyclic/Snaking gain bindings; both chairs remain disabled. Weak modal-open epochs prevent
hidden Open/Cancel ABA; successful Add captures repeat POST-commit under one Engine lock.
Surface loss cannot revive repeat, and same-packet inactive events retain refusals.
Exact Cancel/stale copyable drafts, bounded input and modal/retirement/close fences stay.
No worker, FilePlan, Save receipt or close ACK is added.

Actual JSON/App REDs precede implementation. Independent acceptance adds28 unique cases.
Actual counter and two repeat/refusal defects have unchanged RED/GREEN assertions;
compile/lint/setup and one rustc StorageFull qualification remain distinct. First full
10212 exits0: **1657 passed/0 failed/1 ignored across245 result lines**, including fmt,
warnings-denied workspace/vendor checks, layers, seven assets, unchanged strict runtime
closures155/166/221 and pinned corpus. Log208096B/SHA256
`a8bb8526c8bb5efe1b33def053c6f72b6302848c2bad5f1b5f1f6339ee540660`.
This is not causal repair of historical MCP AddrInUse failures; no exhausted probe repeats.
The bounded source delta review finds no material issue; its earlier two candidates are
the actual App regressions above. A rejected source-only inventory proposal briefly
misbound chairs; Root catches it before application, and corrected v2 excludes that file.

Native/CLI build61504 passes. The isolated actual Mac app checks right-angle drawing,
turning Snaking, invalid/valid Add, Alt repeat, supported-carbon Add attachment, invalid
chord refusal, Fixed Lengths OFF, Undo/Redo and Unicode Save/reopen. The actual5024-byte
file has23 atoms/18 bonds/5 fragments. Independent Expat/Decimal/RDKit checks old geometry,
preferences, literal flags/H/orders,48pt fixed and45.455pt variable lengths, snapped endpoint
and formulas C5H12/C6H14/three C4H10. Native catalogue shows14/126 and a disabled chair click
cannot activate. Explicit Quit and a later list observe the disposable review gone;
an earlier immediate shutdown observation is inconclusive. Both older user apps remain
running with their copied binary hashes unchanged; their drawings are untouched.

Supported native Control-drag/reversal/backtracking, hardware/measured2×/DPI, Windows/Linux,
full styles/inheritance and exact26.1 defaults/hotkeys remain unverified or missing.
CJK filename glyph coverage and historical MCP attribution remain unresolved. No new
golden, hosted run, publication, dependency/font/lock/corpus/workflow change. Scoped cache
cleanup preserves evidence and protected bundles. All owned execution jobs are terminal.
Next source-lock and independently implement missing drawing tools before binding them;
chair/sprout, broader labels/styles, native acceptance and full M0–M5 remain required.

### Verified native two-preference form; full local gate passes

Base `b6e1b18`; the accompanying atomic source/test/docs commit is the recoverable state.
Read [contract](docs/research/native-drawing-settings.md) and
[receiving](docs/verification-native-drawing-settings.md). File > Document Settings…
now edits/clears stored bond length and interior chain angle, using the existing atomic
command under one Engine guard. Blank stays absent; existing geometry/view stay unchanged.
Weak source/association/File/close/view/tool proofs, explicit request-cancel identity,
copyable stale drafts,384-byte fields and modal/action-prefix/close fences are checked.
No FilePlan, worker, Save receipt, close ACK or source authority is added.

Registry42 retains all old41 actual CLI records/schema/order. Controls32/21/15/11;
catalogue124 roles/140 entries stays12 delivered/128 disabled. Sixteen new cases pass;
all17 old Atom cases pass before/after the mechanical shared input-router extraction.
Actual runtime UnknownCommand/missing-menu/missing-repaint REDs remain separate from
compile, filter and new-consumer setup qualifications. Exact cancellation fixes the
remote-Cancel retained form; ordinary context loss still preserves a copyable draft.

First full14206 fails167/1 after fmt/lint at one newly added headless-control reason.
Its explicit unavailable-host premise and source-qualified expectation correction pass
focused30846, then fresh full84512 exits0: **1629 passed/0 failed/1 ignored across242
result lines**. Fmt, warnings-denied workspace/vendor lint/tests, layers, seven assets,
unchanged155/166/221 strict runtime closures and pinned corpus pass. Log203657B/SHA256
`c5e046fd7ee237e3065a9c379f615cfdd480d185e85cbc57d25e57e476bf6c82`.
The existing ignored offscreen artifact case stays ignored. This does not explain or
repair historical MCP listener failures; no exhausted diagnostic experiment is retried.

Native/CLI build10841 passes. The isolated actual Mac app creates/saves a bond, sets
64.25pt/137.5°, checks Undo/Redo, clears angle, saves/reopens exact Unicode filenames and
applies unchanged without dirtying. Independent root/body/geometry/RDKit readers confirm
only BondLength64.25 is added and the existing80pt C2H6 geometry stays unchanged. CJK
filename glyphs are missing visually despite correct AX names and IO; this is recorded,
not complete Unicode UI support. CUA AX focus/diff qualifications are not product fixes.
The disposable review app is observed closed. Both older user apps remain running with
unchanged executable hashes, and their drawings are untouched. No new golden, hosted run,
publication, dependency/font/lock/corpus/workflow change or other-platform claim follows.

Both chain tools stay grey. Next lock the original complete native chain interactions,
then implement drag/count, Add Chain, repeat, Snaking and preference application through
existing candidates/history/registry, with full previews, physical/source/modal/retirement
guards and actual acceptance. Inheritance, remaining settings/styles, native2×/DPI,
Windows/Linux and the remaining M0–M5 workflows stay open. No owned live jobs remain.

### Ordered native input; its fresh local gate passes

This atomic source/test/docs increment follows071f34c. Previous goal turn is PROGRESS;
full M0–M5 remains active. Read [contract](docs/research/ordered-native-modifiers.md) and
[receiving](docs/verification-ordered-native-modifiers.md). One appended egui observation
and the existing common eframe wrapper preserve actual native modifier reports after
unchanged platform mapping. Original EventResponse/global snapshots, old gestures,
source/modal/worker guards and per-viewport queues stay. No fake keys/resets, new
dependency, unsafe, model/command/file authority or native chain control. Registry41,
controls31/20/14/10 and catalogue12 delivered/128 disabled remain unchanged.

First native probe omits empty-event snapshots and fails Control-up setup: inconclusive.
Qualified actual Mac OS baseline establishes focused true→false but zero ordered reports;
the desired oracle fails before producer edits. SDK96800 E0599 is separate API qualification.
Producer46709 passes original3 SDK tests; supplemental97410 adds3 SDK cases/1 privacy pass.
Exact order/filter/take/append, five bits, press snapshot, focus, no replay/fake keys,
duplicates, viewport separation and discard are synthetic SDK evidence, not a chain consumer.

Actual native75511 builds the real app and a separate observation wrapper. On macOS27.2/
AppleM3Max, CUA OS-injected Control+click yields6 trace rows/10 events/4 modifier reports;
the unchanged oracle passes. Release frame11 retains ctrl=true on the button, then NONE
at its actual event position despite final ctrl=false. The original logo, grey rail and
12 available/128 planned catalogue are inspected in that native app. Final dev probe uses
a typed whitelist and forwards the app hook, qualified separately by privacy tests; the
native evidence is not relabeled as that later probe binary. No hardware/DPI/other-platform
or complete chain-flow claim follows. All three disposable review apps are observed closed.

First fresh full cargo xtask ci50402 exits0: **1613 passed/0 failed/1 ignored across240
result lines**. Fmt, warnings-denied workspace lint/tests, existing vendor tests/lint,
layers, seven assets, unchanged155/166/221 strict runtime license closures and pinned
corpus pass. Log202013B SHA256
8e12c6ca45b048fbe33073c89747e35430e00ad55e85bf87d4ab1c188b61f5f7.
The existing ignored offscreen application-frame artifact case stays ignored. Historical
MCP/UI failed runs remain recorded; this pass is not an identity-based causal repair.
No full gate is retried or exhausted probe/context, assertion, socket/timing policy changed.
Bounded source/evidence review finds no material production issue; final doc/comment
qualification passes formatting, diff hygiene and local links without another full run.
Scoped Cargo cleanup removes only two inspected inactive1.6GiB compile caches, preserving
current builds, separate evidence/fixtures and protected bundles; this gate has no disk error.

Native product build a36cc6ea44742cc840119b435aeffca9fe09bb07e45628fd2c0a893198d0f5e3
contains the input patch. Both user apps17806/96449 are observed live with their original
copied binary hashes; neither current drawing is inspected, saved, closed or refreshed.
No new chemical golden, hosted run, push or publication. All managed execution jobs are terminal.

Next exact task: lock original full native chain pose/count/snapping/backtracking/reversal/
repeat policies, then implement their source/context-qualified controller and authored
settings form/application through existing candidate/history/registry paths. Finish Add
Chain/drag counter/Alt repeat/Snaking/full previews/modal/retirement/DPI and independent
App/files/adapters plus actual native acceptance per native-chain-plan.md steps4–6.
Keep both chain tools grey until corresponding complete workflows are checked. Windows/
Linux/Glow/Web modifier behavior, human hardware/measured DPI, styles, broader chemistry/
formats/publishing/distribution and proper MCP identity-based diagnosis remain required.

### Verified atomic settings command; full gate remains failed

This atomic source/test/docs increment follows b6497dd, contract dede5ee and separate
test-only readiness correction e516dd9. Previous goal turn is PROGRESS. Read
[receiving](docs/verification-drawing-settings-command.md). One appended
document.set_drawing_settings requires all three flat keys and explicit nulls to clear;
source authority precedes domain/no-op. One normal edit preserves old pages/H/geometry,
view focus/tool/generation/IDs, with normal history/selection/close/file guards. No-op
retains model sharing, redo and contexts. No worker, receipt or native binding is added.
Registry41 retains every old40 record/schema/order in actual CLI comparison; controls
31/20/14/10 and catalogue12 delivered/128 disabled stay. Native form/tool application,
effective inheritance/defaults and full styles remain missing; both chain controls stay grey.

First JSON35719 REDUnknownCommand0/1 precedes producer18321 GREEN1/0. Expanded core79634
passes9/0, private6151 1/0, captured-Save48981 1/0 and actual adapters1536 3/0. Later
serialization7219 RED shows typed Some(NaN) becoming null/clear; a finite-value serializer
gives46704 GREEN1/0, with unchanged incoming/domain/source/schema behavior. Fifteen new
cases pass, including64 generated history pairs with seed671308429. Existing
independent root checks precede unchanged graph/chemical consumers. Captured Save truth
and both registered MOL versions' positive controls/loss refusals are verified. Inventory
audit positively updates46 cardinalities/16 exact-name comparisons across44 files;
old predicates/OLD_* constants/request IDs stay. Initial source review/format qualification
finds no material issue at that read point; the later serialization safety fix has its
own actual RED/GREEN and separate bounded review.

First aggregate79447 stops at that new-test lint, before workspace tests. Corrected full
gate37811 passes fmt/workspace lint then fails **1012/1/0 across161 result lines** at
unchanged MCP replacement bind51300, AddrInUse/OS48. No unchanged retry/owner identity.
Remaining24093 fails486/1/1 at the old SMILES order UI ready wait. [e516dd9's qualification](docs/verification-smiles-order-readiness.md)
strengthens actual-owner retirement and timeout diagnostics, preserving every old
predicate/assertion/action, two warm frames, ten-second deadline and production fence.
It does not attribute the uncaptured failure schedule or establish a product defect.
Modified UI passes268/0/1; the next selector repeats that library and adds50 UI binary/
integration passes, then stops on invalid xtask --lib selection. Then-unstarted74317
passes55/0/0 with fmt, UI/vendor lint, layers/licenses/corpus. Duplicate UI passes add no
unique coverage. Both failed test runs remain failed; **there is no green aggregate**.

The serializer's current fmt/workspace lint passes, then97163 hits StorageFull in six
file tests after482 passes. Cargo cleanup reclaims4.4GiB only from an old unused dependency
cache; affected and then-unstarted42816 passes51/0, gates and a fresh native/CLI build.
All41 schema records still equal the pre-serializer output. No old cache evidence,
source, current build output or protected app bundle is removed. This is environmental
recovery with prior failures retained, not a green aggregate or semantic test relaxation.

Initial native/CLI build89296 passes; final42816 native4170ce64…3d08 and CLI0fd9a2cc…e092
include the serialization guard. Additional actual
Unicode CLI settings/bond/history/Save/no-op/reopen preserves current/saved/original truth;
independent Expat/Decimal and RDKit read the actual427-byte C2H4/C=C file with only root
angle137.5. No native launch, copied-bundle/draft change, current-drawing inspection,
new platform/golden, hosted run, push or publication. Protected apps17806/96449 are observed
live and both copied binary hashes match their pins. All managed execution jobs are terminal.

Next executable task: implement ordered native Control transitions at the input boundary,
then full source/context-qualified settings/chain forms, drag/count/Add Chain/repeat/
Snaking/poses/real-ink workflows and actual acceptance per native-chain-plan.md steps3–6.
Keep frame-global modifiers from being applied retroactively and keep grey tools disabled
until complete workflows are verified. Full M0–M5, journal styles, broader interoperability/
chemistry/publishing/distribution and proper identity-based MCP diagnosis remain required.

### Verified drawing-settings data — aggregate gate failed

This atomic data/format increment follows contract538df6c on65af201. Read
[the lock](docs/research/drawing-settings-data.md) and
[receiving evidence](docs/verification-drawing-settings-data.md). Optional document-root
BondLength/ChainAngle preserve absence and independent presence, XML points/degrees and
CDX signed fixed16.16. Invalid/unsupported/ambiguous source values remain opaque and
byte-preserved; exact native re-encoding remains the authority proof. Both whole-MOL
versions refuse settings loss before raw/fragment reuse or publication. Existing-source
drift guards and explicitly lossy fragment export stay distinct. Stored preferences do
not move or rescale existing geometry and are not yet applied by native creation tools.

Actual XML82691 and CDX3733 runtime REDs precede producer98821 GREEN2/0. Seventeen new
focused cases pass, with independently authored vectors and an Expat/Decimal root reader;
old chemical readers cannot establish root-setting preservation. Bounded source review
and post-format qualification find no material issue. No dependency, command registry,
source-authority, fixture, golden or socket-policy change. Registry40, controls31/20/14/10
and catalogue12 delivered/128 disabled stay unchanged; full settings workflows stay missing.

Full cargo xtask ci94003 exits1 at the unchanged MCP replacement bind127.0.0.1:58634,
AddrInUse/OS48: **998 passed/1 failed** across156 result lines. Formatting and
warnings-denied workspace lint passed before that failure. No unchanged retry is run.
Only not-started targets/stages pass separately:61803 **583/0/1**, then a target-coverage
audit identifies three still-unstarted compatibility packages;22798 passes **9/0/0**.
Combined remaining **592/0/1** is not a green aggregate. Vendor lint/tests, layers,
strict licenses and pinned corpus pass there. A single post-terminal lsof returns no
owner; it is not failure-time identity evidence. The natural failure remains unattributed;
no exhausted probe/context or socket policy is changed.

Unchanged XML/CDX projection harnesses each pass10000 ASan runs with fresh original
ignored seeds; no new crash is found within those bounds. Actual native/CLI build88017
passes. CLI Unicode Open→bond edit→Undo/Redo→Save→reopen preserves both root values,
editable geometry and clean baseline; independent Expat/Decimal plus RDKit verify the
actual700-byte output. No native launch, copied-bundle refresh, current user drawing
inspection or new platform/golden claim. Protected apps17806/96449 are observed live and
both copied binary hashes remain unchanged. All current execution jobs are terminal.

Next increment at that checkpoint: lock and implement the source-qualified atomic drawing-settings
command through registry/history/adapters, preserving no-op/invalid/source/close/ID guards.
Then ordered native modifier transitions, the settings form and full chain modes/count/
repeat/poses/real-ink flows and actual acceptance per native-chain-plan.md steps2–6.
Keep both native chain controls grey. Full M0–M5, journal styles and broader format/
chemistry/publishing/distribution work remain required. Actions OFF; no push/publication.

### Verified literal path/preview prerequisite

This increment follows a433c3c and dependency contract b4c6ae1; its atomic source/test/
docs commit accompanies this record. Previous turn is PROGRESS (fixed chain1563/0/1).
Read docs/research/native-chain-plan.md and docs/verification-chain-paths.md. New
chain.create_path commits one literal turning path, with free2..64 or1..63 attached new
points. Closed point objects reuse model validation without a model schema dependency.
Fixed and literal paths share final source/H/metadata/ID/history rules. Both new previews
use the same builders without edits/reservations; path vectors are borrowed/bounded
before resolved allocation. Old fixed types/semantics and all39 old command records/
schemas/order remain equal; actual old/current CLI comparison proves that compatibility.
Registry40 leaves native controls31/20/14/10 and catalogue12/128 unchanged.

First JSON35032 REDUnknownCommand0/1 precedes producer77658 GREEN1/0. Independent
post-draft expanded29473 passes8/0 (JSON5/API3); actual adapter42759 returns2/1 because
a new test assumes malformed serde input keeps its socket open. Existing protocol closes
after replying to Error::InvalidParams. Explicit EOF assertion/fresh authentication
preserves all old error/state/publication/source checks and five-second settings;
qualified22336 passes3/0. No production transport change or bind retry. Eleven new
cases cover literal geometry/180°/tiny/crossings, sparse weighted attachments/metadata,
source/ID/history/late-cancel and preview purity/exact commit. Actual Unicode files pass
unchanged raw two-flag/Decimal/directed integer/style checks before RDKit C5H12/C7H16.

First full local gate20061 exits0: **1574/0/1 across225 result lines**, fmt, warnings-denied
workspace/vendor clippy/tests, layers, seven assets, same155/166/221 runtime license
closures and pinned corpus. Log195218B SHA256
67b4b7ac2f2b5c22caf59bb4949a0951a517acb81ee0f861ac0d29ab316f5015.
Native/CLI build44866 passes; native8cc3bf81829105e3c3d09c55001dbd27622f3b400d0784af32c6de030d8fcba1,
CLI3b3be33df0c81b831105128aeec1a422918342d04da4bdafa9c0c84529cd871a.

New isolated chain_path_request fuzz uses existing dependencies/unchanged locks. ASan
38563 passes10000 runs, seed274810359,2s, RSS137MiB, final adaptive length1741;
eight original4208B seeds become200 retained corpus entries. It prepares wrapped JSON
and checks error atomicity/accepted graph/UndoRedo, never completes file/render work.
Empty Engine has no successful attachment targets. This does not cover the full16KiB
setting/1MiB wire budget or establish a security audit. Fuzz fmt and clippy93727 pass.
No new crash is found in this bounded run. Bounded producer review finds no material
issue; review is source evidence, not an independent execution or native acceptance.

No native launch, copied-bundle/window/draft change, golden or new platform claim.
Protected processes17806/96449 are observed live; current drawings are not re-inspected.
No Root/worker execution job remains live. Actions OFF; no hosted run, push or publication.
Historical MCP replacement-bind failures remain unattributed; these passes do not repair
or explain them, and exhausted diagnostic boundaries remain.

Next task at that checkpoint: research normative public SDK BondLength/ChainAngle numeric
encoding, defaults/inheritance and actual existing model/IO loss boundaries; lock a
preserving authored drawing-style increment before independent tests/code. Then preserve
ordered native modifier events and implement complete chain modes/drag counter/Add Chain/
repeat/settings/poses/context/real-ink flows and actual acceptance per dependency steps2–6.
Both native chain controls stay grey. Full M0–M5 and all broader format/chemistry/render/
publishing/distribution work remain required; do not replace the intended app with this
headless prerequisite or infer modern/source/platform defaults.

### Verified headless chain increment

The chain engine follows locked bf4d937 on branding base f077ed2; its atomic source/
tests/docs commit accompanies this receiving record. Read docs/research/chain-authoring-plan.md
and docs/verification-chain-authoring.md. One typed chain.create command adds explicit
count2..64 including anchor, angle1..179° and first-edge/side placement. Free and
shared-atom candidates preserve old metadata, directed orders/styles, geometry and
unrelated owners. New C retains literal H with both false label flags. Target-local
weighted neutral-C4 proof is shared with rings; candidate/IDs/history/source guards use
the existing immediate lane. Registry39; controls31/20/14/10 and catalogue12/128 stay.
The native chain tools remain disabled, with no drag/form/repeat/settings implementation.

Twelve new independent cases pass: engine7/API2/actual CLI+stdio+loopback3. First JSON
44120 fails UnknownCommand before producer; first3299 passes. Later declarations are
honestly post-producer acceptance. Separate source-ID/application-ID, close-context and
three-parenthesis new-test setup/syntax corrections preserve all behavioral predicates
and producer policy. The initial inventory search misses legacy exact sets/counts;
three full gates stop on those assertions (18165/82455/21143). Complete literal-set
audit positively adds only chain.create/39, retains old names/RPC IDs/source offsets,
and affected targets60340 pass35/0. No behavior/source/history guard is weakened.

Final local gate52340 exits0: **1563/0/1 across223 result lines**, fmt, warnings-denied
workspace/vendor clippy/tests, layers, seven-asset strict licenses, same155/166/221
runtime closures and pinned corpus. Log193078B SHA256
1218931ff544d8733e7a98c9ce45c02eac7e49140b0632cd7843809d6fee70a4.
Actual Unicode saved files pass raw Expat/Decimal both-flags/IDs/directed integer
orders/styles before unchanged RDKit chemistry: propane/isobutane/propene/toluene,
C3H8/C4H10/C3H6/C7H8. History, source authority, floors, atomic refusals and immediate
cancellation/context truth pass. Bounded source reviews find no material issue; their
pinned scope is distinct from execution and the complete legacy-inventory audit.

Native/CLI build16149 exits0; native SHA256
1711ed7a965563527accd9cac0f5f5e2b3fc8e4487db3814f39bffc8243a67fd,
CLI08f1c0df0d3532eff097bab4af063287f25e4fc534139c7b54431e84791cf490.
No launch, copied-bundle refresh, window/draft modification, native-chain or new golden
claim. Both protected processes17806/96449 are observed live; current user drawing state
is not re-inspected. No Root/worker Cargo or execution job remains live. Actions OFF;
no enable/dispatch/rerun, push or publication. Historical MCP replacement-bind failures
remain unattributed; this aggregate pass is not a causal repair.

Next: lock the complete native chain workflow and original unknown-default policies
before independent App tests: Add Chain/drag counter, repeat lifecycle, Snaking/Control,
fixed lengths/angles and modifier order, recommended automatic poses including linear
Single+Triple/two-Double centers, shared-anchor count wording, and context/real-ink
guards. Continue remaining M1–M5; chain UI remains grey until actually implemented and
checked. Preserve the earlier File-menu first-click uncertainty, incomplete modern
inventory, platform gaps and exhausted MCP diagnostic boundaries.

### Verified branding/catalogue increment

The original logo and planned-tool request is implemented and verified within the scope
below, on base `4948cec` after terminal-label `1cd8671`; this receiving record accompanies
the atomic visual commit. Read docs/verification-branding-and-tools.md and design/brand.md,
design/tool-catalogue.md. The visual commit includes its code, tests, assets and documentation.

Original MIT OR Apache-2.0 logo masters, header, generated icon and README banner are
present. All124 documented TD/TQ/TB roles map to140 display entries: twelve delivered
bindings and128 disabled/unselected entries with no command. Working rail order/geometry
and its twelve original assertions remain; grey entries have explicit tooltips/AX names.
Search/scroll and current-control replay use one existing registry (38 commands; delivered
control counts31/20/14/10). A grey icon never promotes its missing feature to implemented.
Foreground File/Atom/SDF forms own input; catalogue Escape preserves persistent selection.

Eleven new cases include actual-state/source/history/chemical-paint guards, independent
source-role/variant inventory, modal priority and actual bundled-font coverage. Real
runtime REDs precede producer/fixes; separate SDK/File-menu fixture qualifications stay
explicit. Source review exposes SDF Escape theft; its actual regression fails all three
priority assertions before the shared eligibility fix. Native review finds two absent
font characters; the new font regression identifies exactly both, then geometry fixes them.

First full gate39983 passes1550/0/1 before that last glyph finding. Final gate20773 passes
1551/0/1 across221 result lines: fmt, warnings-denied workspace/vendor clippy/tests,
layers, seven-asset strict licenses, same155/166/221 runtime closures and pinned corpus.
Final log191502B/SHA256a63e67348f9c69fd618fb64d4e6cee9fff9f535c0e5a31e3eaf88297d4638bc2.
Build3564 passes: native c1b8c8849cf44c097ad07e915cfb495ba41c3300e13c5db1537c5f17c53a04bd;
CLI6bda7cb347e990d6388a15b1cdb65175e7eb67b3fea2c5d874e85be1f49049c3.
This does not attribute or repair historical MCP replacement-bind failures.

Bounded macOS27.2/M3Max review uses a separate Bondline Preview bundle, preserving the
old user app/draft. Original2287B command-created seed is independently C6H6+C5H6,
11 atoms/11 bonds/two fragments, SHA25669ba4b4253032bcce65f30a046f363bb4c855366376481f252837163eb39fe8b.
Native Cmd+O/path/Open, grey Lasso/DNA, search/real Double selection, scroll to HELM and
Escape work. Preliminary native Benzene paper click yields17/17/three fragments/dirty;
Cmd+Z restores Saved11/11/two fragments. The initial File-menu Open click closes without
a chooser; its cause is unresolved, so that first-click flow is not called verified.
This is not native authoring of the two-ring seed, full attachment/Shift/ring parity,
measured monitor DPI or Windows/Linux acceptance. Final glyph-corrected native review
reopens the seed and visibly verifies the corrected replacement/angle icons. Two exact
native JPEGs and complete provenance are in docs/media/branding-native-provenance.json.
They are actual own-window images, not generated mockups or approved chemical goldens.

Final preview process17806 remains open on target/desktop-review-branding-4948cec/Bondline.app
with the Saved original scene and no listener. Preliminary process3334 is closed cleanly
before its own disposable bundle refresh. Old user process96449 and copied d0873d6c…5c64
bundle/draft remain untouched. Revalidate newer handles/state before interacting. No Root
Cargo job or worker job remains live; workers completed ignored proposals/reviews only.
No core/format/parser/font/shader/dependency/workflow/approved-golden change. Actions OFF;
no push or publication. Full M0–M5 remains active and incomplete.

Next: lock the existing chain proposal
before independent tests/code. Continue complete Add Chain/repeat/Snaking/settings and the
remaining M1–M5 scope. Preserve native first-click uncertainty and all cross-platform/
modern-inventory gaps; do not repeat exhausted MCP diagnostic contexts or weaken guards.

### Verified terminal-label checkpoint

The terminal-label prerequisite follows locked contracts `3aa093f`/`662c72c` and is
committed `1cd8671`. Read docs/verification-terminal-carbon-labels.md.
Optional node data, strict format-loss refusals, actual-degree display and independent
two-flag restoration are delivered through the shared renderer/registry/editor. Eighteen
new independent cases pass. Root observes intended model/IO/render/setter REDs first;
new-test error-class, type annotation and zero-filter qualifications remain separate.

Full local gate 19445 exits 0: 1,540/0/1 across 221 result lines, including fmt, workspace/
vendor clippy/tests, layers/licenses/corpus. Log 191894B, SHA256
477ba18d66a369247fb4149055b5e87b054aae6c01562749a54ab702dd2a1e1e.
Native/CLI build 47945 passes; native 9a334fa9…e86b1, CLI 6bda7cb3…49c3.
Registry 38 and control counts 31/20/14/10 stay unchanged. No new style/chain control,
source authority, dependency, parser, asset or golden. Bounded source reviews find no
material issue; the integration review explicitly captures pre-format test declarations.

The preceding goal turn made visual-review progress: CUA raises the user's existing
Bondline window and observes the intact unsaved 17-atom/19-bond drawing. It does not
reload the older copied build or verify the newly built features. The subsequent user request prioritizes original branding and visible disabled planned
tools; its current receiving record appears above. Root retains execution/mutation ownership.

## Previous native named-ring checkpoint

Native named-ring integration is the current source/test/docs checkpoint after command89aa03d and
locked native contracta795b1d. Read docs/verification-native-unsaturated-rings.md.
Two controls now share ui.ring_tool, exact effective mode/context, original procedural
icons, the same candidate/preview/release and press-latched Shift. Registry38 stays;
native controls31, retained20/SDF14/raw10. The old sized APIs/receipt/preview and all
scene/source/input guards remain. No desktop bundle/draft change or native OS claim.

Actual runtime REDs precede first producer adoption; missing-type compile qualification
is separate. Core4/API1 and actual adapter2 pass. Initial App3 cases pass; two later
acceptance families bring5 current passing App cases including files, attachments,
Shift/raw chronology/modal and physical refusal/recovery. A disabled first-layout Cancel
and Root’s overstrong Default-bit readiness were test setup errors; current enabled+
Click-verb readiness preserves the original packet/assertions. CLI operation-vs-snapshot
changed flag is separately qualified by a one-line inspect setup correction. No producer
fix or old assertion relaxation for either. Source reviews find no material issue.

Full gate10421 stops at two Copy-test needless borrows. Equivalent by-value correction
changes no assertions. Gate43930 stops at an old CLI exact inventory missing the two
new IDs; only those IDs and the old Tools10→12 count are added. Final gate19064 exits0:
1522/0/1 across216 result lines, all formatting/workspace/vendor lint/tests and layers/
licenses/corpus. Log187496B/SHA25669e5875ac67f3fe9db8f888e299d8f40a7ec1369109cb8367b2ce84b43e51be1.
Native/CLI build85439 passes; native843aa979…7064, CLI baf94aab…8a67. Explicit ignored
actual-App GPU capture77304 passes; the inspected1600×1000 three-bond frame shows the
new twelve-tool rail. PNG52889B/SHA256807ccc47…151f under target/visual-capture/.
This is offscreen actual-App evidence, not a native screenshot/named gesture recording
or approved golden. The user bundle remains exact d0873d6c…5c64 and is not reloaded.

## Previous named-ring command checkpoint

The named benzene/cyclopentadiene command increment is committed89aa03d after contract105b220.
Read [the lock](docs/research/unsaturated-ring-authoring.md) and
[receiving](docs/verification-unsaturated-rings.md). Existing ring.create now accepts
closed sized/named alternatives, bounded2/5 phase ranking against proven literal H,
inward new Doubles/false new flags and exact old metadata. One builder/history path
serves Engine/CLI/initialized stdio/authenticated loopback; registry38 and native
controls29/20/14/10 stay. Named native tools are not yet implemented.

Actual runtime REDs precede adoption; separate test borrow/API/closure compile
qualifications and positive legacy-schema upgrades retain old expectations. An exposed
pre-existing registry array-parameter mismatch is fixed at the shared object boundary.
The first integrated core18/1 failure is a new consumer setup error: sanitized benzene
was passed to an unchanged Single-only helper. A new child-only raw-integer/unsanitized/
sanitized checker passes five manual positives/twelve negatives before generated-file
use. Corrected66740 passes parameters3/core19/types1; actual wire tests pass3/0. The
old parent/consumers, locks, workflow, approved golden and MCP control test stay exact.
fmt76358 passes. Full local gate18337 exits0:1510/0/1 across214 result lines, including
all workspace/vendor lint/tests and layers/licenses/corpus. Log186029B SHA256
18c24e4ae36954323417c2a9a7e48abe2dd9c01ab1272dbf7633475ba2c3e339.
Native/CLI build75233 passes; native e435ed0e…439, CLI cab34aee…6de. Independent bounded
source review finds no material issue and qualifies final formatting/legacy preservation.
Actual CLI authoring saves an original2303B two-ring CDXML scene and captures1024×768
CPU/Metal-GPU frames with exact whole RGBA equality. Independent Expat/RDKit checks
C6H6/C5H6 and literal H/weights/flags/sides. Actual GPU PNG visually inspected under
target/unsaturated-rings/visual/:21506B/SHA256d4d1a4ac…46a2. Not native tool evidence or
an approved golden. No change to the user's old bundle/draft.

## Previous skeletal checkpoint

The skeletal display/explicit-label increment is committed5fa23d1, following locked
contractfb17db9, data caa8d60 and desktop checkpoint27b87a5. This is the recoverable
source/test/docs checkpoint. Read
[the contract](docs/research/skeletal-carbon-display.md) and
[receiving evidence](docs/verification-skeletal-carbon.md).

The shared renderer keeps None/dormant legacy pixels, requires known H/checked actual
C4 before active false suppression, and preserves identical near-collinear/two-Double
mandatory labels. Hidden glyphs and masks disappear together; stable atom hits and
actual ink/gap/paper/scale/cancel guards remain. No H inference, source-authority override,
parser/dependency/asset/font/shader change. Existing atom.set_label restores false to
true, including same fields, through normal history; None/true no-ops remain exact.

Twenty-four new cases cover analytic Single/Kekulé components/pixels/full CPU-Metal-GPU,
known/resolved/physical H, mandatory labels, source/no-op/history and actual Unicode
files through Engine/CLI/initialized stdio/authenticated loopback/actual-App editor.
Only the prior DATA blanket refusal is positively upgraded on identical old inputs;
legacy/cancel and other old assertions stay. Five renderer supplements are post-draft
acceptance. Their one failed exact-dot precondition is a numeric fixture setup issue;
a Decimal-derived Y-only correction keeps every assertion/threshold and producer exact.
The command missing-prolog setup and two zero-test filters remain separately qualified.

The first aggregate51082 stops at one new-test bool-assert lint. An equivalent assert!
correction changes no test meaning or product. Final aggregate42092 exits0:1494 passed/
0 failed/1 ignored across212 result lines, including fmt, warnings-denied workspace/
vendor clippy/tests, layers/licenses/corpus. Log184162B SHA256
06a217eafd4b03a53d3733798c1663b616dfe2053975e4ef29f6af3c876f97e4.
Native/CLI build63733 exits0. No native window is opened/replaced for this increment.
Independent bounded source review finds no material product issue; exact identities,
correction proofs and reproduction commands are in receiving. Root/isolated locks,
old native-v0/consumers/goldens and unchanged MCP test remain pinned.

Headless CLI76145 additionally produces an original six-carbon skeletal Kekulé preview;
actual adapter Metal/Apple M3 Max/IntegratedGpu and complete CPU/GPU RGBA agree.
Independent Expat/RDKit checks C6H6 and exact H1/no source nodes. The actual GPU PNG is
visually inspected under target/skeletal-display/visual. It is not a native screenshot,
named-ring-tool workflow or approved golden; no asset is shipped or user draft changed.

## Existing application and remaining parity

Registry42 serves UI, CLI, stdio MCP and authenticated loopback. Native controls32,
retained SMILES21, SDF15 and raw SMILES11. Earlier bounded Mac bonds, atom labels,
history/source files and SMILES/SDF workflows retain their individually scoped evidence.
Selection/translation and six3–8 Single-ring tools have independent command/adapter/App
checks; most actual native OS interaction remains unverified. The two-preference form
above has its separate bounded native Mac acceptance.

Explicit Left/Right Double retains unchanged authored axes, bisector-shortened secondary
strokes, finite-cap/strict-edge gaps, role-qualified joins, distinct arm pixels and actual
CPU/Metal-GPU frame agreement. Its prior full gate95915 passes1451/0/1. See
[inset receiving](docs/verification-inset-doubles.md); dormant non-Double still refuses.

Named-tool native OS acceptance, false-setting/default-style controls, alignment, broader
imported editing, chemistry/reactions, publishing/clipboard, naming/biopolymers,
WebAssembly/installers, modern inventory and cross-platform interaction remain required.
Do not shrink M0–M5 or equate persisted fields with a finished authoring workflow.

## Unresolved historical test failure

The earlier data aggregate49615 fails909/1 at unchanged MCP replacement bind
127.0.0.1:57773, AddrInUse/OS48, without failure-time owner identity. Separate84887
passes521/0/1 plus remaining stages. Later1451/1470/1494 passes neither attribute nor repair it.
Earlier atom95827 fails848/1 at port60475. Controlled probes establish a mechanism only;
see [listener research](docs/research/macos-listener-lifetime.md). Both probes and20
exhausted diagnostic contexts are consumed. Never retry/serialize/weaken that test for
green. A new causal investigation needs known failure-time socket/process identity,
without observer queries in shutdown→bind. No socket policy or test-source change lands.

## Desktop, ownership and jobs

At the user's explicit request, the current Bondline build is now OPEN on the desktop.
Bundle target/desktop-review-caa8d60/Bondline.app contains the exact verified native
executable recorded in receiving; no control listener was requested. CUA observed the
real window with tool rail, page and inspector, then a user-edited unsaved17-atom/19-bond
drawing. Leave this window and draft intact; do not clear, save, close or relaunch it
without a relevant user request. This visual observation is not a verified native tool/
history/persistence workflow or an approved golden. Earlier unused-browser-tab cleanup
remains intact; iTerm tabs remain unchecked because access is restricted. No proprietary
reference binary/asset is opened or inspected. Native OS/DPI/Windows/Linux workflow
acceptance for inset/selection/rings remains unverified.

Root owns Cargo/Git/CUA/live mutation. Full gate84512, build10841 and all focused settings
checks/reviews are terminal; no owned job or disposable review app is live. Two protected
user processes are revalidated without inspecting drawings: PID96449 at
target/desktop-review-caa8d60/Bondline.app, executable SHA256
`d0873d6c812187d936c915265137d8da6be658720e4dc88a7a747f680c7b5c64`, and PID17806 at
target/desktop-review-branding-4948cec/Bondline.app, SHA256
`c1b8c8849cf44c097ad07e915cfb495ba41c3300e13c5db1537c5f17c53a04bd`.
Do not replace, close or lose either draft. Revalidate handles before restarting;
never retry an unchanged failed socket test for green. Independent workers supplied
ignored proposals/source reviews only; Root integrated and executed checks.

## Historical chain continuation steps (superseded above)

1. Read docs/research/native-chain-plan.md and docs/research/native-drawing-settings.md.
   Existing fixed/path candidates, ordered native modifier transport, optional settings
   data and the actual two-preference form are delivered. GUIDE25 printed104–105 covers
   drag/count, Snaking/Control, attachment, Add Chain and Alt repeat. Lock original
   count/quantization/poses/snapping/backtrack/reversal/repeat policies before independent
   native-controller/App REDs. Preserve literal H; never infer26.1 defaults from these APIs.
2. Continue full Add Chain/repeat/Snaking/chair/sprout/style/chemistry/publishing/backlog
   workflows, with independent core/wire/App/native evidence. Implement preference
   application/default policy as part of complete chain tools. Preserve unsupported-source
   authority, preview IDs/history and all physical/density/input/modal/retirement guards.
   Both tools remain grey until corresponding complete workflows are independently checked.
3. Preserve initial native File-menu first-click uncertainty for a bounded own-app
   investigation; Cmd+O is verified. Fused/spiro/Shift/density and other-platform native
   acceptance remain required. GitHub Actions stays disabled; no unrequested publication.

### Analysis molecular weight and exact mass

[PR 29](https://github.com/thematheusmello/bondline/pull/29) contains chemistry
`b2ea49b`, shared command `2939458`, native panel `0f909f9` and main/#27 ancestry
`2e92757`. The Analysis window now shows both mass quantities for complete
structures and uncapped selected-atom contributions. Tabulated data, convention,
licensing and unavailable reasons are documented. The 25 chemistry, 10 focused
command, three CLI, one MCP and four App-frame tests passed. An isolated macOS
background host verified the values through its own accessibility tree and
closed. The native licenses and four-target dependency delta passed.
[Acceptance and replay](docs/verification-analysis-masses.md).

Scoped tests finished and the PR awaits its combined workspace gate and main
integration. Adjustable decimals, isotope distribution, elemental percentages,
overrides and live captions remain open. Actions stays off and M0–M5 remains active.
