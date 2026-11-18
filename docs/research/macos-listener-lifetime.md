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
