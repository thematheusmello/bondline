# Development

Rust is pinned in rust-toolchain.toml. Cargo.lock pins the reviewed dependency closure;
do not regenerate it casually. Run the native app and headless CLI using README.md.

Independent PNG verification uses Pillow 12.3.0 as an external process. Python/Pillow are
test tools only, not application dependencies. To prepare an isolated oracle environment:

```sh
python3 -m venv .venv
.venv/bin/python -m pip install -r requirements-dev.txt
BONDLINE_PYTHON=.venv/bin/python cargo test --workspace --locked
```

On Windows, use `python -m venv .venv` and `.venv\Scripts\python.exe`; set BONDLINE_PYTHON
to that interpreter in the invoking shell. If it is unset, tests use python3 on Unix and
python on Windows. A missing or different oracle version fails explicitly, never skips.
The current Mac already has Pillow 12.3.0; this work did not change its system environment.

Implemented gates: rustfmt, workspace/all-targets clippy, workspace tests, `cargo xtask layers`
and `cargo xtask licenses`, plus `cargo xtask corpus` (see corpus/README.md for exact subset).
`cargo xtask ci` runs these gates together; see ci.md for configured versus executed platforms. GPU tests
require an actual supported adapter and fail if it is absent. Source cross-compilation is
useful evidence but does not establish runtime behavior on that operating system.

The native loopback channel is opt-in via `--control-port`. Do not broaden its bind address.
Every connection now authenticates with a fresh 256-bit session token before running any
command. Startup stdout is the owner's credential channel; do not redirect it to public logs
or commit tokens in artifacts. The UI/errors never show the token. Localhost is a network
boundary, not an operating-system user identity, and a compromised local account is outside
this token's protection. Current operations expose the temporary page/chemical model. HTTP
input is rejected and closed. Future file operations still require explicit authorization,
bounded input, validated paths and data-preserving save/recovery behavior.

The independent single-bond workflow is native on Mac and available through bond.create in
every adapter. It is session-local: no save/reopen command exists yet. See rendering.md for
supported content, scale/primitive limits, exact golden scope and the missing connected tools.
The user-approved render reference lives in crates/render/tests/goldens with its coordinates,
hash and provenance. Do not refresh it to make a test pass without renewed human approval.
Pillow tests also compare terminal glyph masks with independent FreeType. FontTools is used
only to reproduce recorded metric vectors, not required by the regular CI/test command.
