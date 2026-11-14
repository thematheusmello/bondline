# Contributing to Bondline

Contributions are welcome: code, bug reports, original test cases, documentation,
accessibility improvements and public-source research. AI-assisted contributions
follow the same review, verification and licensing requirements as any other work.

Start with [STATUS.md](STATUS.md) for what works today, [ROADMAP.md](ROADMAP.md) for
the delivery order and the [parity checklist](docs/parity-checklist.md) for missing,
wired and verified capabilities. Small fixes can go straight to a pull request.
For a larger change, an issue describing the workflow and intended scope helps
avoid duplicate work. See [SUPPORT.md](SUPPORT.md) for questions and bug reports.

GitHub Actions is intentionally disabled during development to control hosted costs.
Run the local checks below; do not enable or rerun hosted workflows. See [CI policy](docs/ci.md).

## Set up and run

Follow the [development guide](docs/development.md) for the pinned Rust toolchain,
platform packages and isolated Python test tools. Python is used for independent
test oracles; it is not part of the application runtime.

```sh
cargo run --locked -p bondline-ui --bin bondline
cargo run --locked -p bondline-cli -- commands
```

Keep a separate Cargo target directory for each worktree. Shared build outputs
have caused checks to run against the wrong checkout.

## Implement and verify a change

1. Read the relevant source, callers, tests and public specification. Check
   [AGENTS.md](AGENTS.md) and the [architecture](docs/architecture.md) before
   changing component boundaries.
2. For behavior changes, write a focused test with an independent expectation.
   Run it and observe the intended failure (RED), implement the change, then run
   it again to observe success (GREEN). Refactor after it passes.
3. Exercise failure, malformed-input, resource-bound and data-preservation cases
   relevant to the change. A round trip through only our own reader and writer
   does not establish interoperability. Use published vectors, an independent
   parser, external chemistry tools, analytic cases or approved images as appropriate.
4. Record the actual commands, results and platform in
   [docs/verification.md](docs/verification.md). UI changes also need the actual
   native workflow checked; input tests and screenshots alone do not prove it.
5. Update the relevant documentation and parity status. Distinguish implemented,
   wired and verified behavior, and state what remains untested.

Add authoring operations to the existing typed command registry and use it from
the UI, CLI, MCP and loopback adapters. Keep core libraries independent of the GUI.
Preserve complete source-backed documents and unknown format data through edits,
history and saving. Do not replace them with a partial model projection.

Use typed errors and bounded input handling. Production code must not introduce
panic, unwrap, expect or placeholder paths. Unsafe code is restricted to the
documented, reviewed OS boundaries. See [AGENTS.md](AGENTS.md) for the full rules.

After setting up the test interpreter, run the aggregate gate for code changes:

```sh
BONDLINE_PYTHON="$PWD/.venv/bin/python" cargo xtask ci
```

Use an absolute interpreter path. The [development guide](docs/development.md)
describes Windows setup and the individual formatting, lint, test, layer, license
and corpus commands. GPU checks require a supported adapter. If a platform or
native check is unavailable, state that limitation and report the checks you did run.
Documentation-only changes need relevant link, spelling or configuration checks;
they do not need artificial tests or a full Rust test run. Parser fuzzing has its
own [pinned setup](fuzz/README.md).

## Sources, assets and licenses

Submit original code under [MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE).
Shipped dependencies and bundled assets must permit use under MIT, Apache-2.0,
BSD-2-Clause or BSD-3-Clause. Retain their notices and provenance; do not broaden
the allowlist or change a license label to satisfy a check. See the
[dependency policy](docs/dependency-policy.md) for the exact rules and known audit gaps.
GPL, LGPL and AGPL tools may only run as external test programs, never be linked
or copied into the shipped application.

Use public specifications, documentation, screenshots, videos or authorized
artifact observations. Record exact sources, versions, platforms and observations
in [sources.md](docs/clean-room/sources.md), and unresolved behavior in
[open-questions.md](docs/clean-room/open-questions.md). Do not bypass access controls
or transplant proprietary code, help prose, icons, fonts, templates or sample libraries.
Create original redistributable assets matching the documented roles.

Test fixtures also need a clear redistribution license or original authorship.
Follow the [corpus guidance](docs/corpus/README.md). Keep downloaded references,
research captures, caches and build output in ignored directories. Never include
credentials or session tokens. Do not regenerate the approved single-bond golden
to make a test pass; changing that reference requires renewed human approval.

## Send a pull request

Keep commits small and use conventional messages such as `feat(commands): ...`
or `fix(cdxml): ...`. Keep one command or format feature with its tests, provenance
and relevant documentation in the same commit. Record RED evidence without leaving
the proposed final change failing.

Explain the problem, resulting behavior, verification and remaining limitations.
Link an issue or parity item when relevant. Include source and license information
for new dependencies, fixtures or assets. Do not include disposable research files,
build outputs or secrets. The pull request template provides a short review outline.
