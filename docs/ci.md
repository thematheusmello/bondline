# CI contract and execution evidence

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
