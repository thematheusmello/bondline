# Development

Rust is pinned in rust-toolchain.toml. Cargo.lock pins the reviewed dependency closure;
do not regenerate it casually. Run the native app and headless CLI using README.md.

Independent render/format verification uses external Pillow 12.3.0 and RDKit 2026.09.2,
with NumPy 2.5.4, pinned in requirements-dev.txt. They are test tools, not Rust application
dependencies or bundled assets. To prepare an isolated oracle environment:

```sh
python3 -m venv .venv
.venv/bin/python -m pip install -r requirements-dev.txt
BONDLINE_PYTHON="$PWD/.venv/bin/python" cargo test --workspace --locked
BONDLINE_PYTHON="$PWD/.venv/bin/python" cargo xtask ci
```

On Windows, use `python -m venv .venv` and `.venv\Scripts\python.exe`. Set
`BONDLINE_PYTHON` to the pinned interpreter for workspace checks. Some independent CDX
checks require the explicit setting. Individual checks may fall back to system Python
and still require the pinned oracle versions.
The current Mac's new consumers are isolated under ignored .cache/oracles/format-python;
the system environment was not changed. Both documented .venv and BONDLINE_PYTHON selection
work the same way. RDKit is only an independent file consumer here; Bondline does not gain
Python runtime chemistry, linking or a general differential-chemistry claim by using it.
Use an absolute interpreter path: Cargo runs each test binary from its package directory,
so a workspace-relative executable path does not resolve consistently across the workspace.

Implemented gates: rustfmt, workspace/all-targets clippy, workspace tests, `cargo xtask layers`
and `cargo xtask licenses`, plus `cargo xtask corpus` (see corpus/README.md for exact subset).
`cargo xtask ci` runs these gates together; see ci.md for configured versus executed platforms. GPU tests
require an actual supported adapter and fail if it is absent. Source cross-compilation is
useful evidence but does not establish runtime behavior on that operating system.

Keep each worktree's default target directory separate. Reusing another checkout's target
produced a stale xtask binary and incompatible first-party metadata during actual integration.
The gate runner now uses read-only `cargo locate-project --workspace --frozen` at runtime;
CI prints the selected root before checks. A failed locator is an error, never a fallback to
the build-time checkout. This also applies to standalone asset/corpus lookup and works from
nested package directories. If artifacts were mixed, discard the affected first-party build
outputs and rerun the real gate; an earlier pass for another checkout is not evidence.

On Ubuntu 24.04, install the Linux packages listed in .github/workflows/ci.yml before
building/running the native app. X11 needs libxkbcommon-x11-0 at runtime in addition to
libxkbcommon-dev; the first hosted window launch exposed that missing dependency.
Mesa Vulkan and Xvfb support the headless CI smoke test, not a hardware-performance claim.

The native loopback channel is opt-in via `--control-port`. Do not broaden its bind address.
Every connection now authenticates with a fresh 256-bit session token before running any
command. Startup stdout is the owner's credential channel; do not redirect it to public logs
or commit tokens in artifacts. The UI/errors never show the token. Localhost is a network
boundary, not an operating-system user identity, and a compromised local account is outside
this token's protection. Current operations expose the temporary page/chemical model. HTTP
input is rejected and closed. Future file operations still require explicit authorization,
bounded input, validated paths and data-preserving save/recovery behavior.

The independent single-bond workflow is native on Mac and available through bond.create in
every adapter. CDXML Open/Save/Save As and safe native close/reopen now have Mac evidence;
see native-file-ui.md for exact scope and UI/keyboard/platform limitations. See rendering.md for
supported content, scale/primitive limits, exact golden scope and the missing connected tools.
The user-approved render reference lives in crates/render/tests/goldens with its coordinates,
hash and provenance. Do not refresh it to make a test pass without renewed human approval.
Pillow tests also compare terminal glyph masks with independent FreeType. FontTools is used
only to reproduce recorded metric vectors, not required by the regular CI/test command.
