# Listener shutdown contract verification

This change corrects the shutdown regression test; it changes no runtime behavior.
The test now proves an accepted client's closure and exclusive listener-address release.
It passes in the normal parallel MCP package run. The deeper system cause of the
observed initial `connect_timeout` success followed by reset remains unknown.

## Preserved failures and diagnostic observations

Root's default workspace gate, session 10297, failed the unchanged
`shutdown_cancels_idle_clients_and_releases_the_listener` assertion at
`crates/mcp/tests/control.rs:1305`. Its ignored log,
`target/multiple-ci-fourth.log`, is 74,779 bytes with SHA-256
`f005e39be68db25a610206cb5ec3e57d173fd664b1052eb99dbe5051b3a0fde5`. The earlier session 66585 failure
is retained separately in `bondline-preview-work/target/preview-registered-work-final-tests.log`
(5,192 bytes, SHA-256
`c0c24f336c25fd2b4fbbd864056fd08c2a83ff0311e8343c6ce2cd35c2f4aec8`),
with its original unknown-cause qualification in
[the registered-work evidence](verification-preview-registered-work.md).
Neither failure is relabeled a pass.

Temporary diagnostics in an isolated worktree reproduced the same assertion with
normal test parallelism. They recorded only process/socket addresses, opaque
listener ordinals and I/O status; no token, request content or Engine Debug was
recorded. Source instrumentation tracked binds, accepts, worker return and join.
All instrumentation and bounded stress tests were removed from final source.

| Diagnostic | Result | Important limit |
|---|---|---|
| v1, session 32236 | 23 pass, 2 fail; original and stress iteration 81 | Local and peer ports differed; later `lsof` found no listener. This excludes exact self-connect for these samples, not transient foreign ownership. |
| v2, session 18868 | 25 pass, including 5,000 stress iterations | File writes before the probe changed timing. This is not a fix. |
| v3, session 64629 | 24 pass, 1 fail; stress iteration 3 | Lifecycle writes moved to a temporary channel/logger thread; no pre-probe file write. |
| v4, session 26205 | 23 pass, 2 fail; original and stress iteration 2 | Both unexpected streams transitioned from initial success to a reset in one bounded read. |
| Bare standard listener, session 64293 | 5,000 bind/drop/probe iterations, no unexpected success | No accepted idle client or MCP server; this negative control did not reproduce the mechanism. |

In v4's original test, listener 23 owned `127.0.0.1:62082`; the accepted idle
client used port 62083. Worker return and join were recorded before the late
probe began. The late stream used local port 62084 and peer port 62082, so it was
not self-connected. `take_error` initially returned none; immediate nonblocking
read returned WouldBlock. One subsequent read, bounded to the same 200 ms
probe budget, returned ConnectionReset (OS 54) in less than 1 ms; its peer query
then failed. The stress sample at port 62053 behaved the same way. No replacement
owned bind/accept of either address appears in the trace; later `lsof` found no
listener. These observations establish that initial success alone did not prove
a continuing endpoint owned by the stopped server. They do not establish the
kernel/network path that produced the reset or exclude every brief foreign event.

The existing production worker joins only after its listener future returns and
connection tasks are aborted and drained. It was restored byte-for-byte to the
receiving Root input: 29,756 bytes, SHA-256
`871277936b9d61f99b3270833d7a57573d0a58d70bad681c56489c461affa6d2`.
No production shutdown defect was demonstrated by this investigation.

## Stronger direct regression contract

Only the existing shutdown test function changes. It now:

1. Authenticates the client, proving the server actually accepted and serviced it,
   then leaves a partial command pending.
2. Requires a second ordinary bind to the live address to fail with AddrInUse.
3. Keeps the original shutdown duration bound below two seconds.
4. Reads the accepted client, requiring EOF, ConnectionReset or ConnectionAborted
   within 200 ms. WouldBlock, timeout, bytes or another error fail.
5. Immediately binds a new ordinary listener to the exact released address,
   without reuse-port configuration, retries or sleeps.
6. Within 200 ms, positively connects and accepts a new pair, checks both endpoint
   addresses and exchanges exact independent `replacement-owner` / `released`
   bytes through that new owner.

The positive exchange uses the already-pinned Tokio runtime/network APIs so both
accept and I/O are bounded. It is not a retry of the failed old-address probe.
The live bind conflict plus successful replacement bind directly test ownership;
actual client EOF/reset tests the closure missing from the earlier assertion.
A leaked listener, retained accepted client, wrong replacement endpoint or failed
exchange now fails decisively. No global mutex, test serialization, additional
public API, dependency, longer deadline or runtime change was added. All other
control-test bytes are unchanged.

## Execution and evidence

Platform: macOS 27.2 (26B123), arm64. Root port receipt
`target/shutdown-diagnosis-root-port.json` verified 35 receiving source/test inputs
before diagnostics (SHA-256
`731433963d8073803c8b9a4364a7652a1085a71529f7293bbdae705ff17f2939`).
Every Cargo run used Rust 1.94.1, this worktree's absolute `target/`,
`CARGO_INCREMENTAL=0`, dev/test debug 0, the existing pinned `BONDLINE_PYTHON`,
and the unchanged root lock. No test-thread override was used in the final run.

```text
cargo +1.94.1 test --locked -p bondline-mcp --test control shutdown_cancels_idle_clients_and_releases_the_listener
cargo +1.94.1 test --locked -p bondline-mcp
cargo +1.94.1 clippy --locked -p bondline-mcp --all-targets -- -D warnings
cargo +1.94.1 fmt -p bondline-mcp -- --check
```

The first stronger-contract run, session 60866, passed. The default parallel
package run, session 84699, passed all 49 tests: 19 library, one MOL ownership,
24 control and five registered UI-work tests; doc tests had no cases. Clippy
session 84374 and rustfmt both exited zero.

| Final ignored log | Bytes | SHA-256 |
|---|---:|---|
| `listener-shutdown-strong-contract-first.log` | 441 | `0350f5b3937dc8181b64f2710f5188aaf3c237523eef7129419030fccdbb2c5c` |
| `listener-shutdown-mcp-tests-final.log` | 5,723 | `d54d2cc61c0b0eea0027ab2542a5dea9f02c520156fd46750c07ce8bed3bc53c` |
| `listener-shutdown-clippy-final.log` | 2,910 | `e430af25dc741fc02a147f09f2fac1754b9234fbb4bf251cd711ab8541d31862` |
| `listener-shutdown-fmt-first.log` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

The frozen ignored handoff contains all diagnostic source/log hashes, exact test
diff and protected-input checks. Of 308 pre-task source/fixture/oracle/lock/tooling
identities, only the intended test file changes; the new verification document
is additive. The final test-file size and SHA-256 are recorded in that handoff.

## Primary technical sources and limits

The pinned [Rust 1.94.1 Unix socket implementation](https://github.com/rust-lang/rust/blob/1.94.1/library/std/src/sys/net/connection/socket/unix.rs)
contains immediate nonblocking-connect success and a separate poll path. Its
exact downloaded source is 25,222 bytes, SHA-256
`0a196515a392a124277b9d5f13e7585e8d3a79beb223f10192d901440c118fd0`.
This explains why an initial result and independently observed endpoint behavior
are distinct evidence; it does not identify our actual syscall path.
The primary [socket2 issue 652](https://github.com/rust-lang/socket2/issues/652)
reports Apple completion behavior with remote IPv6/VPN conditions. It is a
research lead, not a reproduction of our loopback case or proof of its cause.
No socket2/runtime/library upgrade or copied upstream code is included here.

Root's receiving workspace aggregate remains a separate required gate. New
Windows/Linux execution, native UI, deeper kernel diagnosis and a universal
network-closure guarantee were not checked. GitHub Actions remains disabled;
no GUI/input/activation, Git operation, dependency, asset, oracle, golden or
fixture change was performed. All diagnostic/Cargo child processes have exited.
