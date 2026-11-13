# CI contract and execution evidence

Latest verified application source `c879f4c95964617a86cc2d1e466515c5aeb31f29` passes every job in
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
