# macOS listener lifetime investigation

2026-11-16. The unchanged MCP integration test
`shutdown_without_clients_releases_the_listener_address` intermittently fails its one
replacement bind after shutdown. The atom checkpoint's full gate95827 reports
127.0.0.1:60475, `AddrInUse`/OS48, after848 passes. There is **no failure-time socket
identity or owner capture**. Neither a later passing gate nor the experiment below
establishes that failure's cause. Assertions, timing, retries, socket policy and test
concurrency remain unchanged.

The later DoublePosition data aggregate49615 naturally reproduces the same unchanged
test's replacement-bind failure at127.0.0.1:57773, AddrInUse/OS48, after909 passes.
No failure-time owner identity is captured, so this supplies no causal attribution.
It is preserved without retry; only not-started stages run separately. See
[the receiving record](../verification-double-position.md). No controlled probe or
exhausted diagnostic context is rerun.

The drawing-settings full gate94003 naturally fails the same unchanged replacement bind
at127.0.0.1:58634, AddrInUse/OS48, after998 passes. Log124425B SHA256
79d4749e2941a361845c481e3a5a8ff7a64b88a5bff5dadf3974ed53a3f25962.
Only not-started stages run separately (combined592/0/1); the failed aggregate is not
retried. One post-terminal `lsof -nP -iTCP:58634` exits1 with empty output. That late
observation identifies no failure-time socket/process owner and adds no causal evidence.
No collector, consumed probe, exhausted context or socket/assertion/timing policy changes.
See [receiving](../verification-drawing-settings-data.md) for the complete stage boundary.

Later evidence,2026-11-17: corrected settings-command full gate37811 fails the unchanged
replacement bind127.0.0.1:51300 after1012 passes, AddrInUse/OS48. Log125470B SHA256
336de6dbcb9b5aa05dcc5971bdbb38c02beac29c1a2832ee76a2bfa942910f85.
No owner identity/unchanged retry/collector/probe/policy change occurs. Its separate UI
test-readiness qualification does not attribute or repair this socket failure. See
[command receiving](../verification-drawing-settings-command.md). All consumed diagnostic
boundaries below remain unchanged.

## Source hypothesis and fixed experiment

The pinned Rust1.94.1 Darwin [socket constructor](https://github.com/rust-lang/rust/blob/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/sys/net/connection/socket/unix.rs#L65)
creates a socket before a separate [close-on-exec operation](https://github.com/rust-lang/rust/blob/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/sys/fd/unix.rs#L566).
Its [process implementation](https://github.com/rust-lang/rust/blob/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/sys/process/unix/unix.rs#L691)
does not request Darwin's close-all spawn flag. A concurrent spawn could therefore inherit
an initially unbound socket; later parent bind/listen would affect that same open socket.
This is a source-derived hypothesis, not evidence that such overlap occurred in the test.

An original ignored standalone external diagnostic fixes exactly two arms: spawn before
FIOCLEX and spawn after FIOCLEX, once each. It uses the reviewed constructor/options,
one listener owner, one child, zero accepted clients, known PID/FD only, one bind while
the child remains and one after its explicit close/reap. Budgets remain2s I/O,8s per arm
and10s watchdogs. No retries, broad process/descriptor scans or reuse-policy changes.
The probe is not linked into Bondline or added to its dependency/unsafe allowlist.

The original sole run was inconclusive: before-flag transport failed with EINVAL22;
after-flag introspection failed with ENOPROTOOPT42. Neither reached a replacement bind.
Both children were reaped. Darwin exposes SO_ACCEPTCONN as an internal option bit but
the reviewed getter switch does not implement that query. A queued child ERROR followed
by peer close is source-consistent with the transport failure, but was not observed.
Do not recast that first run as a falsification or a positive result.

A separately frozen v2 replaces only that unsupported observation and the ERROR/CLOSED
transport handshake. Its original test-only C shim uses the actual MacOSX27.2 SDK
`socket_fdinfo` layout, calls `proc_pidfdinfo` for its own known descriptor, and requires
TCP/family/type plus agreement of the accept bit and LISTEN state. The public
[field definitions](https://github.com/apple-oss-distributions/xnu/blob/f6217f891ac0bb64f3d375211650a4c1ff8ca1ea/bsd/sys/proc_info.h#L541),
[state population](https://github.com/apple-oss-distributions/xnu/blob/f6217f891ac0bb64f3d375211650a4c1ff8ca1ea/bsd/kern/socket_info.c#L103)
and [libproc return convention](https://github.com/apple-oss-distributions/xnu/blob/f6217f891ac0bb64f3d375211650a4c1ff8ca1ea/libsyscall/wrappers/libproc/libproc.c#L160)
guide this observer. Those sources do not attest the installed kernel; actual own-FD
startup observations qualify it. No kernel pointers are logged. All original semantic
predicates, socket operations, arm counts and budgets remain exact.

## Observed result and practical limit

Root compiles with the pinned Rust commit, libc0.2.191 artifact and actual Apple SDK;
the C compiler checks the scalar ABI and792-byte structure. The one v2 invocation passes:

| Arm | Child before bind | Child after parent join | Bind while child lives | Bind after explicit close/reap |
|---|---|---|---|---|
| Spawn before FIOCLEX | Unbound, not close-on-exec | Same endpoint, LISTEN | AddrInUse/OS48 | Success |
| Spawn after FIOCLEX | Descriptor absent | Descriptor absent | Success | Success |

Both listener owners join and both children exit0 and are reaped. This qualifies the
**controlled inheritance mechanism only**. It does not identify who owned60475 or prove
that the actual test encountered this interleaving. No production or test-policy fix is
adopted from this result. The earlier exhausted20-context run and falsified outbound-socket
hypothesis are not repeated.

Ignored artifacts remain under `target/native-smiles-records/socket-source-review-v2/`:

| Artifact | SHA256 |
|---|---|
| Original source,22657B | `4e607e9bbe1c041b17a8eb9ae0625d97b3ca4ca6608123aa6ccfb768df11abec` |
| Original result,839B | `47c7a835a6615bad1de60c08ff08ce26a56900cd454d114e2342bba48617cb29` |
| v2 Rust source,26090B | `92aef4d4e49068606ad200dc0b9d75cd85789891320aef1731747b69b6fe09aa` |
| Original C observer,1614B | `ef6434a25960cb048da756ceff7844ca6aa938e729c71d3638021db4bef6e9c3` |
| v2 executable,849664B | `5f34045e33d511f713188dd1fac2a5aff83c2e5843e5ade6a58437d33c24d5ab` |
| v2 result,2870B | `01e1596d73f5a34062a58795c8d3abad5207562a165c366ba634ab2c972dbfb8` |

Build commands, SDK/source pins, declaration preservation, consumed invocation markers
and cleanup records accompany those artifacts. They are disposable local evidence,
not shipped assets. Do not rerun either consumed invocation.

Next causal investigation needs a new natural failure's identity: capture the original
known descriptor while certainly owned, register actual children at existing spawn sites,
and inspect only after a failed replacement bind. Match socket identity/generation and
process birth, not port or PID alone. Do not query sockets in the shutdown→bind interval:
the observer itself briefly holds a kernel reference. The collector is designed only,
not implemented or run. A close-all spawn policy can prevent persistent inheritance but
does not promise zero transient fork references; no examined wrapper has been adopted.

## A spawn begins after the listener is published, 2026-11-20

PR 34 recorded two natural replacement bind failures on ports 52920 and
59323, OS error 48. The logs identify their addresses and phase. Their
historical owners remain unknown.

A separate bounded experiment used the real `LoopbackServer`, zero connected
clients and an ordinary concurrent `posix_spawn` after `bind` returned.
An ordered child file action created a marker, then waited for a private FIFO
before exec. The test called the normal server shutdown and tried one exact
replacement bind while the child remained in that interval. It released the
FIFO and reaped the child before evaluating the assertion. The earlier
pre-close-on-exec experiment above concerns a different interval.

| Measured on macOS 27.2 arm64 | Before listener change | With close-on-fork |
| --- | --- | --- |
| Original live address | Exclusive | Exclusive |
| Shutdown and join | 211 µs | 2,010 µs |
| First replacement bind while spawn is paused | AddrInUse (48) | Succeeded |
| Bind after child exec and reap | Succeeded | Succeeded |

These single-run times identify the observed calls. They are not performance
benchmarks. An independent control with an anonymous pipe used no TCP ports.
With only close-on-exec the pipe still had a writer while spawn was suspended.
With both close-on-exec and close-on-fork it immediately had none. Both arms
observed EOF after exec. These experiments establish causality for this
controlled overlap; the cause of PR 34's historical failure is unconfirmed.

Bondline preserves existing descriptor flags and requests `FD_CLOFORK` on
its own listener before publishing it. The MacOSX27.2 SDK's public
`sys/fcntl.h` defines the value as 2 and its `fcntl(2)` manual documents
close-on-fork. Public XNU sources distinguish copying a process's file
descriptors during spawn from discarding descriptors on exec.
[Apple spawn implementation](https://github.com/apple-oss-distributions/xnu/blob/f6217f891ac0bb64f3d375211650a4c1ff8ca1ea/bsd/kern/kern_exec.c) · [Apple descriptor-table copy](https://github.com/apple-oss-distributions/xnu/blob/f6217f891ac0bb64f3d375211650a4c1ff8ca1ea/bsd/kern/kern_descrip.c)

There is a brief interval between the standard socket constructor and the
flag operation. A spawn that has already copied the descriptor by then may
retain its reference. An unrelated process may also claim a released port.
Neither case is covered by the controlled comparison.

This repository sets no explicit macOS deployment target. The pinned Rust
compiler reports defaults of 11.0 for arm64 and 10.12 for x86_64. The local
arm64 desktop executable's `LC_BUILD_VERSION` reports minimum 11.0 and SDK
27.2. Both historical XNU implementations below accept `F_SETFD` and process
only the close-on-exec bit. On such systems the listener retains its prior
behavior. An explicit unsupported-flag error (`EINVAL`, `ENOTSUP`, or
`EOPNOTSUPP`) also preserves ordinary startup. Other errors propagate.
Older macOS execution remains unverified.
[XNU 3789](https://github.com/apple-oss-distributions/xnu/blob/88cc0b975a863932db8d475bd87c96c73292f9a3/bsd/kern/kern_descrip.c) · [XNU 7195](https://github.com/apple-oss-distributions/xnu/blob/bb611c8fecc755a0d8e56e2fa51513527c5b7a0e/bsd/kern/kern_descrip.c)

The regression is `cargo test --locked -p bondline-mcp --test macos_listener_spawn`.
The unmodified service failed it with OS48. The changed service passed. On
older kernels the test measures support on a regular descriptor before
exercising the stronger guarantee. Two unit tests check listener flags,
unsupported errors and propagation of other descriptor errors. The original
live exclusive bind, two-second shutdown and 200 ms replacement endpoint and
byte-exchange checks are unchanged. Their test source retains SHA-256
`fd96b257cd6c00b52e97544d652ed69fdbaaaa454e1f3a9360d77f3292218fb0`.

The direct macOS dependency is the same rustix 1.1.6 already shipped through
Bondline IO. The safe POSIX spawn wrapper from MIT-licensed nix 0.32.0 is a
macOS test dependency. The MCP normal package names and versions match the
previous revision for arm64/x86_64 macOS, Windows, Linux and WebAssembly.
[Locked rustix](https://docs.rs/rustix/1.1.6/rustix/io/fn.fcntl_setfd.html) · [Test-only spawn API](https://docs.rs/nix/0.32.0/nix/spawn/fn.posix_spawn.html)

At `59028ee`, the complete default-parallel MCP package ran once and passed
98 tests, zero failed and zero ignored. The original 25-case control binary
and the new deterministic macOS regression both passed. Its log SHA-256 is
`49f35c5a75f20db8f211ff1d7af7c880f95a41579787b257cf1020f6661affab`.
All MCP Clippy targets, rustfmt, layers and the configured runtime license
and asset checks also passed.

The full local `cargo xtask ci` at `8923167` exited successfully. The
workspace reported 2,417 passed, zero failed and one ignored test, followed
by three passing AccessKit tests. The original control and the new held-spawn
test both passed. The build was briefly suspended for disk space before any
workspace test began. The same process then completed all checks.
Receipt `target/analysis-ci/listener-8923167.log` in the analysis-window
checkout is 311,132 bytes, SHA-256
`5e27c5bb161c46573d6326c2f50b905f1cf13672bbfc368274f0aabe8736e3a1`.
