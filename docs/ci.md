# CI contract and execution evidence

## Current policy: Actions disabled during development

On2026-11-14 the user requested that hosted GitHub Actions stop during development because
of its cost, and resume only when the project is complete. Repository Actions was disabled
through GitHub's permissions API, and Rust foundation workflow417397204 was disabled
manually. Readback at11:26:45UTC confirms enabled=false and disabled_manually; a paginated
check found zero running or queued runs, so no cancellation request was necessary.
No other repository, billing setting or completed evidence was changed.

Do not dispatch, rerun or re-enable Actions for development commits or milestone checkpoints.
Keep the workflow YAML for final verification. Continue local cargo fmt/clippy/test,
cargo xtask ci and relevant bounded fuzzing. Passing local gates permit integration during
this policy; older hosted-before-main rules below are historical and superseded. Missing
new-source Windows/Linux/hosted execution remains explicitly unverified, never inferred
from previous successful runs. Only full-project completion or a new user instruction can
change this policy. The README badge reflects the disabled state.

The records below remain evidence of runs that actually happened before disablement.
They are not instructions to start another run or a promise of coverage for later source.

SMILES notation/fuzz integration8e0e92b7437e5123e5a5ebd4a7fb981968194d69 passes every
[43573987299](https://github.com/thematheusmello/bondline/actions/runs/43573987299) job:
Mac132384119167:568 (565workspace+3bridge), Linux132384119069:566,
Windows132384118953:558 and ASAN132384119211:all seven10000-run targets. Every actual
window smoke passes with the same qualified Metal/Vulkan/Dx12 adapters below. Main
advances only after matching GREEN. This is notation/source evidence, not application
SMILES. Subsequent SDF session sourceb842e36 has local receiving590 but still needs its
own hosted/native evidence;8e0e92b's results are not substituted.

Documentation checkpoint3ec8b4cb372e3a3c297829c382bb2ff2b687cc5b passes its automatic
[43571525559](https://github.com/thematheusmello/bondline/actions/runs/43571525559):
Mac132376759120:559, Linux132376759149:557, Windows132376758857:549, all three actual
windows and ASAN132376759095:six10000-run targets. Its runtime code matches15ecbc7.
The subsequently received SMILES codec/fuzz/fixture source has local568/ASAN10000
evidence in verification-smiles.md and its own hosted run above. Seven parser targets
are configured and actually pass there; later sources still need their own execution.

Owned-SDF integration9436bf4c9ce0bb4a4425112a0bbfb2a1b7f1a484 passes every
[43568995448](https://github.com/thematheusmello/bondline/actions/runs/43568995448) job:
Mac132369037648:549 tests (546workspace+3bridge), Linux132369037687:547,
Windows132369037361:539 and ASAN132369037587:six10000-run targets. Every actual native
window smoke passes. Adapters remain Metal/Apple Paravirtual device, Vulkan/llvmpipe CPU
and Dx12/Microsoft Basic Render Driver CPU. This is complete-source library ownership,
not SDF application Open/Save, data editing, layout or a native collection workflow.

Field-value integration15ecbc7f1ca543f2875e76e6f536fd8051245ad9 has its own
[43570071575](https://github.com/thematheusmello/bondline/actions/runs/43570071575):
Mac132372310260 passes559 (556workspace+3bridge), Linux132372310080 passes557,
Windows132372310208 passes549 and ASAN132372310315 passes all six10000-run targets.
All three actual window smokes pass with the same adapters. The run finishes GREEN at
07:27:58UTC; main advances only afterward, to a documentation child of this exact source.
No old run substitutes for new code, and no completed passing job has been requested again.
This verifies library field-value editing, not application/native SDF editing. Later
whole-collection session/Open/Save work must pass its own checks and native acceptance.

Attachment sourcef0910ca09e2cfa12691dd03d965e3053e482928e now passes
[43563825506](https://github.com/thematheusmello/bondline/actions/runs/43563825506), after
one failed-job-only retry. Original Mac132353167345 passes540 tests/windows and
Windows132353167364 passes530/windows; ASAN132353167332 passes all six10000-run targets.
Original Linux132353167418 fails before any application test/window: rustup reports a
preinstalled bin/cargo-clippy conflict while installing the pinned1.94.1 components.
That is infrastructure failure, not Linux feature verification. The failed result remains.

Only failed jobs were requested again with gh run rerun --failed. Attempt2 Linux
132358002259 passes538 tests and the actual Vulkan/llvmpipe window on a fresh runner,
without source/toolchain-pin changes. GitHub's latest-attempt listing assigns retained
successful jobs IDs132358003812/132358039105/132358027355; their original execution
timestamps/output are unchanged. No completed passing execution was manually repeated.
Adapters remain Metal/Apple Paravirtual device and Dx12/Microsoft Basic Render Driver CPU
for the other platforms. Sourcef0910ca's actual Mac pass verifies only new empty→C2
painting; terminal attachment native cases were safely stopped on unowned input and
remain unverified. See verification-bond-attachment-ui.md; no window smoke substitutes.

Terminal-extension/CDX source501a315f2fd92a05702cc4518b5d33c8f72abb7c passes every
[43560603081](https://github.com/thematheusmello/bondline/actions/runs/43560603081) job:
Mac132343193738:523 tests (520workspace+3bridge), Linux132343193548:521,
Windows132343193736:513 and ASAN132343193667:six10000-run targets. All actual native
window smokes pass. Adapters remain Metal/Apple Paravirtual device, Vulkan/llvmpipe CPU
and Dx12/Microsoft Basic Render Driver CPU. This verifies the16-command headless engine
and existing native window, not the subsequently developed attachment gesture.

Guarded CDX application source450962cdcbad2abd5aa74b514ef77d58923d3d8f separately passes
every [43559974203](https://github.com/thematheusmello/bondline/actions/runs/43559974203)
job: Mac132341249329:502 tests (499workspace+3bridge), Linux132341249192:500,
Windows132341249443:492 and ASAN132341249397:six10000-run targets. Every actual window
smoke passes with the same adapters. Main advanced from00e5b02 to this exact source only
after matching GREEN. Actual bounded Mac CDX workflow acceptance uses its frozen binary
e9491a72…5749 and379-source manifest, separately recorded in
[verification-cdx-application.md](verification-cdx-application.md).

Documentation-only acceptance updates after501a315 change no tested runtime, dependency,
fixture or configuration. Earlier failures remain below; no completed job was manually
rerun. Interactive Windows/Linux chemistry/file workflows remain unverified.

Corrected carbon/metadata sourcea3613c0968dbce126ba0e1632d36db3d034be14f passes every
[43559699996](https://github.com/thematheusmello/bondline/actions/runs/43559699996) job:
Mac132340407449:490 tests (487workspace+3bridge), Linux132340407514:488,
Windows132340407526:480 and ASAN132340407087:six10000-run parser targets. Every actual
native window smoke passes. Adapters: Metal/Apple Paravirtual device, Vulkan/llvmpipe CPU,
Dx12/Microsoft Basic Render Driver CPU. The original0.90 font threshold/negative cases,
production/font and approved golden remain unchanged; BASIC with independent design-unit
positions fixes the reproduced test-only layout dependence. Both previously failing
label tests pass in Mac and Windows aggregates. Failed983ffee remains below.

This verifies the source-only metadata and carbon renderer/input integration; it does
not verify subsequent CDX application source450962c or the terminal-carbon command/UI.
Those newer sources and their separate actual native scope are recorded above. No
completed passing job was restarted; new feature results stay distinct.

Carbon source983ffee6951e0b3d8c17e5135645cb2cbd3830a3 has a real failed
[43557351022](https://github.com/thematheusmello/bondline/actions/runs/43557351022).
Linux132333092682 passes477 tests and its actual Vulkan/llvmpipe window;
ASAN132333092803 passes all six10000-run targets. Mac132333092799 and Windows132333092520
fail both FreeType label comparisons at the unchanged0.90 ink-IoU threshold: right-facing
CH3 is0.8668951728 and authoredH1 is0.8883125864. Their aggregates stop in render unit
tests before the window smoke; neither is a platform pass. Local479/native Mac acceptance
remain separate exact evidence. The generalized Python oracle's layout-dependent advances
were reproduced and corrected test-only at a3613c0 as recorded above; no renderer/font/
golden/threshold change or failed-run relabeling. No passing job was restarted.

Corrected CDX snapshot/filesystem source00e5b02fb6cd5b845a6343ee6bcd34b3add95d7e passes all
[43550955963](https://github.com/thematheusmello/bondline/actions/runs/43550955963) jobs:
Mac132313031167:446 tests (443workspace+3bridge), Linux132313031270:444,
Windows132313031269:436 and ASAN132313031015:six10000-run targets. All actual native
window smokes pass. Windows's previously failing excess-connection test now passes.
Adapters are Metal/Apple Paravirtual IntegratedGpu, Vulkan/llvmpipe Cpu and Dx12/Microsoft
Basic Render Driver Cpu. Main advanced to this exact source only after matching GREEN.
The failed earlier run remains below; no passing job was restarted. This is library
CDX/source/filesystem evidence, not registered/native application CDX or interactive
Windows/Linux chemical workflows. Later graph/hover/metadata sources need their own runs.

Owned CDX source5e6b644ba56b43e95f0306f032a8c4a39c46fe0b has a terminal failed
[43549658341](https://github.com/thematheusmello/bondline/actions/runs/43549658341):
Mac132308919149 passes441 tests (438workspace+3bridge), Linux132308918944 passes439,
both actual windows pass, and ASAN132308919199 passes all six10000-run targets.
Mac uses Metal/Apple Paravirtual IntegratedGpu; Linux uses Vulkan/llvmpipe Cpu.
Windows132308919222 fails the existing control test
excess_connections_are_closed_instead_of_spawning_unbounded_tasks: read returns OS10053,
ErrorKind::ConnectionAborted, excluded by the assertion. Its15-test binary has14 passes/
one failure; the aggregate stops before native window launch. This is not Windows GREEN.
Main remains0830a23. The test-only platform correction and later CDX filesystem source
need a new matching run; no passing job is rerun or the observed failure erased.
The follow-up test-only correction accepts the documented definitive ConnectionAborted
outcome in all three closed-read assertions. Positive bytes, live/time-out/would-block reads
still fail; production networking is unchanged. Local15 control tests and443workspace+
3bridge aggregate pass. Corrected source requires its own actual hosted Windows result.

CDX header-document framing integration6447c6ca2d83405503417a6d0e141fa6387f3538 passes every
[43542026529](https://github.com/thematheusmello/bondline/actions/runs/43542026529) job:
Mac132284521204:420 tests (417workspace+3bridge), Linux132284520959:418,
Windows132284521318:410 and ASAN132284521169:six bounded targets. Every actual native
window smoke passes. Adapters remain Metal/Apple Paravirtual IntegratedGpu, Vulkan/
llvmpipe Cpu and Dx12/Microsoft Basic Render Driver Cpu. Main advanced only after GREEN,
to docs-only child0830a23. This verifies syntax/framing and its regression suite, not CDX
model/application interoperability or interactive Windows/Linux workflows. Later owned
source snapshots require their own hosted run; no passing job was restarted.

RXN/mixed-SDF integrationc351bb839ee183f2b8a069642b39de32a8ad52ab passes every
[43539538831](https://github.com/thematheusmello/bondline/actions/runs/43539538831) job: Mac132276586398:413
(410workspace+3bridge), Linux132276586426:411, Windows132276586389:403 and ASAN132276586297.
All actual windows and six bounded parser targets pass, including RXN and mixed-SDF seeds.
Main advanced to validated docs-only child291c3a6 after this source passed. CDX framing
research/fixes and future model mapping are separate; no native SDF/reaction editor claim.

Earlier mixed-SDF integration23a43aa passes [43538842553](https://github.com/thematheusmello/bondline/actions/runs/43538842553):
Mac132274352536:404 (401workspace+3bridge), Linux132274352484:402, Windows132274352683:394,
ASAN132274352264, all actual windows/five targets. Isolated source6aeb9f2 also passes
[43538499719](https://github.com/thematheusmello/bondline/actions/runs/43538499719): Mac132273225712:382,
Linux132273225750:380, Windows132273225700:374 and ASAN132273225445/five targets.
Isolated RXN4cbc89b passes [43538648520](https://github.com/thematheusmello/bondline/actions/runs/43538648520):
Mac132273712642:376, Linux132273712487:374, Windows132273712656:368 and ASAN132273712226/
six targets, all actual windows. These source checks are not substituted for the receiving
integration above. Adapters remain Metal/Apple Paravirtual IntegratedGpu, Vulkan/llvmpipe
Cpu and Dx12/Microsoft Basic Render Driver Cpu. No passing job was restarted.

Accepted application/CDX/corpus integration85bf5ebfd53580b66cde9726a374658fc9befcee passes every
[43537753933](https://github.com/thematheusmello/bondline/actions/runs/43537753933) job: Mac132270775971:397
tests (394workspace+3bridge), Linux132270775923:395, Windows132270776068:387 and ASAN132270775722.
All actual windows and five bounded parser targets pass; adapters remain Metal/Apple
Paravirtual IntegratedGpu, Vulkan/llvmpipe Cpu and Dx12/Microsoft Basic Render Driver Cpu.
Main was fast-forwarded fromc797262 only after those results. Later SDF/RXN integrations
require their own source runs; native interactive Windows/Linux and full M0/M1 remain open.

CDX writer plus fetched original V3000 corpus source7596c777e2a3453dadeefc2b9c82144d7b8850a3
is terminal GREEN in [43536671159](https://github.com/thematheusmello/bondline/actions/runs/43536671159):
Mac132267252777:370 tests (367workspace+3bridge), Linux132267252875:368, Windows132267252821:360
and ASAN132267252522. All actual native window smokes and five bounded targets pass. Adapters
remain Metal/Apple Paravirtual IntegratedGpu, Vulkan/llvmpipe Cpu and Dx12/Microsoft Basic
Render Driver Cpu. The new-tree writer and one original V3000 corpus vector do not establish
application CDX, general V3000 files or Bondline chemistry. Later application/container
integration sources require their own gates. No passing job was restarted.

Corrected fetched-SDF corpus source `c797262d23524dd949047b3bbe3b4b4022d37fbc` is terminal
GREEN in [run43518162847](https://github.com/thematheusmello/bondline/actions/runs/43518162847):
macOS `132205357329`: 334 tests (331 workspace plus three bridge), Linux `132205357320`:
332, Windows `132205357409`: 324 and ASAN `132205356923`. All three actual window smokes
and all four bounded parser targets pass. Adapters remain Metal/Apple Paravirtual,
Vulkan/llvmpipe Cpu and Dx12/Microsoft Basic Render Driver Cpu. Main was fast-forwarded
only after these checks passed. The failed earlier run remains recorded below. This
does not verify native SDF authoring, foreign-file CDX or interactive Windows/Linux use.

First fetched-SDF corpus run
[43515232724](https://github.com/thematheusmello/bondline/actions/runs/43515232724) at504be5f
is terminal: macOS132195407757, Linux132195408230 and ASAN132195408337 passed; Windows
132195408100 failed the original manifest's immutable-byte assertion after checkout CRLF
conversion. This is a failure, not a platform pass. The narrow manifest -text correction
changes no parser, license/source pin, manifest byte or assertion. An isolated real Git
clone reproduces the exact bad hash then the original good hash; see
research/corpus-manifest-checkout.md. Corrected hosted execution follows publication.

SDF codec source `fb86832a0c9b7744fb3ef8f8115c7ed296164505` passes every job in
[run43512236206](https://github.com/thematheusmello/bondline/actions/runs/43512236206):
macOS `132185104536`: 327 tests (324 workspace plus three bridge), Linux `132185104144`:
325 tests, Windows `132185104480`: 319 tests and ASAN `132185104428`. All three actual
windows and all four CDXML/CDX/MOL/SDF bounded sanitizer targets pass. Adapters remain
Metal/Apple Paravirtual IntegratedGpu, Vulkan/llvmpipe Cpu and Dx12/Microsoft Basic Render
Driver Cpu. The narrow meaningful-whitespace attribute/doc child2d665de changes no codec
or fixture bytes. Fetched SDF corpus tooling is a separate subsequent feature; its local
aggregate passes331 workspace plus three bridge tests and hosted execution follows publication.
No native SDF workflow or complete M1 claim follows. Source run43512236206 is terminal GREEN.

Earlier MOL application source `c879f4c95964617a86cc2d1e466515c5aeb31f29` passes every job in
[run43508330802](https://github.com/thematheusmello/bondline/actions/runs/43508330802):
macOS job `132171702163`: 315 tests (312 workspace plus three bridge), Linux job `132171702255`:
313 tests and Windows job `132171702213`: 307 tests, plus ASAN job `132171702124`. All three actual
window smokes pass; the sanitizer job runs CDXML/CDX/MOL 10000 times each. Adapters remain Metal/Apple Paravirtual
IntegratedGpu, Vulkan/llvmpipe Cpu and Dx12/Microsoft Basic Render Driver Cpu. No interactive
Windows/Linux workflow or hardware-performance claim follows. Independent native Mac MOL
acceptance names its separate owned binary in native-file-ui.md.

Standalone codec correction `670178b943ce2e9c7ea477e8064adf09f98eac58` also passes every job in
[run43507660824](https://github.com/thematheusmello/bondline/actions/runs/43507660824):
macOS job `132169417129`: 284 tests, Linux job `132169417270`: 282, Windows job `132169417113`: 276
and ASAN job `132169416475`, all actual windows and all three parser targets. First codec
[run43504072080](https://github.com/thematheusmello/bondline/actions/runs/43504072080)
failed Windows fixture checkout byte identity, not a hidden pass. The narrow Git attribute
correction leaves parser/assertions/oracle input unchanged; see research/mol-fixture-checkout.md.
Both corrected/source runs above are terminal GREEN; do not restart them.

`cargo xtask ci` runs real rustfmt, all-targets workspace clippy with warnings denied,
locked workspace tests, layer checks, native declared-license/asset checks and the corpus
gate. It stops at the first failure. Corpus coverage remains the explicit subset in
corpus/README.md, never an all-format/chemistry claim. A valid cache makes its corpus
stage offline; a missing reviewed entry is fetched by the developer-only Python helper.

The aggregate command has passed locally on macOS arm64/Metal. A deliberate formatting
defect was rejected before clippy; the exact source was restored and the full command
rerun. Hosted execution is recorded below; configuration alone is not a pass.

The native matrix targets ubuntu-24.04, windows-2025 and macos-15 (arm64 according to the
reviewed runner reference). It selects Rust 1.94.1 and Python 3.13.2, installs the pinned
Pillow/NumPy/RDKit external consumers from requirements-dev.txt, runs
the aggregate checks and launches the actual window's three-frame smoke mode. Linux gets
Mesa Vulkan and Xvfb for window tests; a software driver validates the graphics API path,
not hardware acceleration or performance. GPU tests fail if an adapter is absent. X11 also
requires libxkbcommon-x11-0 at runtime; libxkbcommon-dev alone does not supply that library.
Infrastructure failures must be recorded or moved to suitable runners, never hidden by
skipping assertions.

A separate Linux job configures the same isolated ASan parser smoke used locally: pinned
nightly/cargo-fuzz, committed fuzz lock, original seed in an ignored mutation corpus,
10000 runs with a fixed seed and explicit length/time bounds. It checks that dependency
locks were retained. This is a smoke run, not exhaustive fuzzing or a security certification.

The workflow uses contents:read permissions, PR/push/manual triggers and full action commit
pins. Checkout credentials are not persisted. Reviewed releases: checkout v7.0.1 at
3d3c42e5aac5ba805825da76410c181273ba90b1; setup-python v7.1.0 at
6d4fd22b20465ccc0f7354476588e8df7c9ca0e0. Both published action manifests use Node 24;
these CI tools are not application dependencies. No publication, artifact upload, signing,
deployment or hosted service is configured. Repository publication explicitly requested by
the user triggered the first hosted workflow.

Static validation: PyYAML 6.0.3 parses the workflow; triggers/matrix were checked against
the public runner documentation. actionlint was unavailable.

## First hosted run, 2026-11-13

[Run 43448214670](https://github.com/thematheusmello/bondline/actions/runs/43448214670)
tested product checkpoint 3d3dd44. All three platforms passed the real aggregate gates,
including 147 workspace tests and CPU/GPU checks. macos-15 and windows-2025 also passed
the actual three-frame native-window launch. The separate Linux CDXML ASan job passed
its bounded 10000-run check. These results do not verify interactive drawing on Windows
or replace the existing manually inspected Mac workflow.

The Linux window step failed while xkbcommon-dl loaded libxkbcommon-x11.so.0/.so.
Its pinned source and the Ubuntu noble package index identify the omitted runtime package
libxkbcommon-x11-0. The workflow now installs it, retaining the actual window test without
fallback or skipped assertions. A corrective hosted run is required before claiming that
the Linux native launch passes; the first run's overall conclusion remains failure.

Correction 8672de0 triggered
[run 43451706668](https://github.com/thematheusmello/bondline/actions/runs/43451706668),
which completed successfully on all three platforms, including the actual native windows
and the separate 10000-run Linux ASan check. Smoke output identifies macOS Metal on Apple
Paravirtual device (IntegratedGpu), Windows Dx12 on Microsoft Basic Render Driver (Cpu),
and Linux Vulkan on llvmpipe LLVM 20.1.2 (Cpu). Software-driver success verifies those API
paths, not hardware acceleration or performance. No interactive Windows/Linux drawing or
new file-workflow claim follows from this run; its product code is still 3d3dd44.

## Native files and current integration

[Run43458663480](https://github.com/thematheusmello/bondline/actions/runs/43458663480) at
34d577d passes Windows, Ubuntu and Linux ASan. Windows executes seven io unit and six file
integration tests, including protected owner DACL/exclusive temporary, DACL/Zone.Identifier
preservation and recovery, then launches the real window. Its earlier exact-DACL test RED
was corrected against Microsoft's documented AI control flag, retaining all access checks.

The Mac job131999452071 never started: provider annotations record multiple acquisition
attempts and hosted arm64 capacity constraints, then cancellation with no runner/steps.
The overall run is unsuccessful. This is not a Mac test failure or successful execution;
see [provider annotations](https://api.github.com/repos/thematheusmello/bondline/check-runs/131999452071/annotations).
Local Mac checks and native UI evidence remain separate.

Complete file/UI/accessibility integration at ccf8c50 is now in
[run43463687047](https://github.com/thematheusmello/bondline/actions/runs/43463687047), confirmed
complete with all four jobs successful. Actual aggregate test totals are196 on Mac
(193 workspace plus3 native bridge),194 on Linux and188 on Windows; differences are
target-specific tests. All actual native window launches pass, as does Linux ASan.
Mac's aggregate executes the patched accessibility tests/clippy from the root lock.
The tested feature branch was fast-forwarded into main. Final native Mac interaction has
the exact-source binary hash recorded in native-file-ui.md; interactive Windows/Linux
file gestures remain unverified despite their automated engine/UI/window checks.
Saved ignored full log: target/github-files-43463687047.log.

## Standalone CDX integration

[Run43475705005](https://github.com/thematheusmello/bondline/actions/runs/43475705005) at
8547f4d passed Mac/Linux and both Linux parser sanitizer targets, but failed Windows's
independent record-table comparison: Python text stdout emitted CRLF rather than the exact
LF byte protocol. The producer was corrected without normalizing binary CDX or relaxing
assertions, and a forced-CRLF regression reproduces the failure on any host.

Corrected sourcea73bed7 passes all four jobs in
[run43480423381](https://github.com/thematheusmello/bondline/actions/runs/43480423381):
Mac210 tests, Linux208, Windows202, every actual three-frame native window and both bounded
Linux ASan targets. The Mac total includes three native bridge tests. Adapters remain Metal/
Apple Paravirtual IntegratedGpu, Dx12/Microsoft Basic Render Driver Cpu and Vulkan/llvmpipe
Cpu. These verify those execution paths, not hardware performance, interactive Windows/Linux
editing, general chemical CDX coverage or application CDX import/export. Main was updated
only after this GREEN; the integration/status commit changes documentation alone.

## Shared close integration

[Run43488771976](https://github.com/thematheusmello/bondline/actions/runs/43488771976) at
72742fb passes all four jobs: Mac236 tests (233 workspace plus3 native bridge), Linux234,
Windows228, all actual native three-frame windows and both bounded Linux parser ASan
targets. Each aggregate printed the invoked checkout root, exercising the runtime locator
fix. Adapters remain Metal/Apple Paravirtual IntegratedGpu, Dx12/Microsoft Basic Render
Driver Cpu and Vulkan/llvmpipe Cpu. This verifies shared-close engine/adapter/UI tests and
window launch, not interactive Windows/Linux closing or hardware performance. Main was
fast-forwarded after GREEN, with documentation-only status updates afterward. No job
remains live and no release was published.

## Shared file-control integration

[Run43496010615](https://github.com/thematheusmello/bondline/actions/runs/43496010615) at
65b5fb8 passes all four jobs. Aggregate tests: Mac268 (265 workspace plus3 native bridge),
Linux266, Windows260. Every actual native three-frame window launch and both bounded Linux
parser ASan targets pass. Each aggregate printed its invoked workspace root. Adapters are
Metal/Apple Paravirtual IntegratedGpu, Vulkan/llvmpipe Cpu and Dx12/Microsoft Basic Render
Driver Cpu. Interactive native acceptance is separately Mac/Apple M3 Max/Metal, exact owned
binary4a5425b0...; no interactive Windows/Linux or hardware-performance claim follows.
Source was fast-forwarded into main after GREEN, with documentation-only updates after the
tested source. All jobs are terminal; do not rerun unchanged source or substitute an older
run for future changes. No release, signing or installer publication occurred.

## Original CDX fixture and standalone V3000

[Run43520991476](https://github.com/thematheusmello/bondline/actions/runs/43520991476) at
30ae30e passes Mac335 tests (332 workspace plus3 bridge), Linux333 and Windows325.
Jobs132214903242/132214903791/132214903835 execute the real aggregate and actual native
window; sanitizer job132214903870 runs CDXML/CDX/MOL/SDF10000 iterations each. CDX now
includes the complete65750-byte original positive seed under max_len131072.

[Run43519338382](https://github.com/thematheusmello/bondline/actions/runs/43519338382) at
5caf6ce separately passes Mac341 (338 workspace plus3 bridge), Linux339 and Windows333.
Jobs132209347432/132209347777/132209347969 execute their aggregates and native windows;
132209347869 passes all five10000-run parser sanitizer targets, including V3000. This
source predates the fetched SDF/CDX corpus integration and V3000 application workflow.

All jobs are terminal. Mac uses Metal/Apple Paravirtual IntegratedGpu; Linux uses Vulkan/
llvmpipe Cpu; Windows uses Dx12/Microsoft Basic Render Driver Cpu. These are launch and
automated execution results, not interactive Windows/Linux workflows or performance claims.

## Fetched CDX and merged standalone V3000

[Run43526004010](https://github.com/thematheusmello/bondline/actions/runs/43526004010) at
98bc59a passes all four jobs. Mac132231799739 reports 338 tests (335 workspace plus3bridge),
Linux132231799721 reports 336, Windows132231799481 reports 328. Each executes the actual
native window and four-file/three-bundle corpus. ASAN132231799170 passes the four parser
targets 10000 iterations each. This verifies the original authored CDX vector, not foreign
files, a chemistry engine or application CDX.

[Run43526538340](https://github.com/thematheusmello/bondline/actions/runs/43526538340) at
226396e combines that source with unchanged standalone V3000 codec5caf6ce and passes all
jobs. Mac132233587981 reports 352 tests (349 workspace plus3bridge), Linux132233587509
reports 350 and Windows132233587877 reports 342. Five-target ASAN132233587865 passes,
including the positive CDX seed/bound and V3000. All actual window smokes pass. These do not
verify application V3000 or the later save-order/receipt correction, which needs its own run.
Adapters remain Metal/Apple Paravirtual IntegratedGpu, Vulkan/llvmpipe Cpu and Dx12/
Microsoft Basic Render Driver Cpu. Every job is terminal; no completed job was restarted.


## Hosted receiving-source acceptance, 2026-11-14

Exact sourcee6471961bc88eeb97a0bb7a8fe115c1bed2392e7 passes
[run43583494325](https://github.com/thematheusmello/bondline/actions/runs/43583494325),
terminal success10:15:28UTC. Root inspected all four latest-attempt job outcomes and
successful logs: Mac132413004015:634 (631workspace+3bridge), Linux132413003718:632,
Windows132413003985:624, plus ASAN132413003970:seven10000-run parser targets. The normal
hosted SMILES target includes the final source-to-normalized/unique-marker export checks;
its three starting seeds are distinct from the five-seed local run. Both locks remain exact.
All three actual window launches report Metal/Apple Paravirtual device, Vulkan/llvmpipe
(LLVM20.1.2) CPU and Dx12/Microsoft Basic Render Driver CPU. These are real window/API
smokes, not interactive Windows/Linux chemistry or hardware-performance measurements.
Main advanced to this exact source only after terminal GREEN. No new native20-command
field editor, application SMILES flow, golden approval or complete M0/M1 claim follows.
