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
Aggregate CI remains missing. GPU tests
require an actual supported adapter and fail if it is absent. Source cross-compilation is
useful evidence but does not establish runtime behavior on that operating system.

The native loopback channel is opt-in via `--control-port`. Do not broaden its bind address.
Before exposing file access or private chemical documents, add session authentication and
authorization; localhost is a network boundary, not an operating-system user identity.
Current operations expose only the temporary page model. HTTP input is rejected and closed.
