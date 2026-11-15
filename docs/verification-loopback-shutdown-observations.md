# Loopback shutdown: bounded observations

2026-11-15. No production MCP code or existing test changes. The earlier
[diagnostic-only phase labels](verification-loopback-shutdown-diagnostics.md) now
identify a natural failure, but do not identify its cause.

## Actual failure and final receiving pass

Root full CI97729 stops after754 passes at
`shutdown_without_clients_releases_the_listener_address`:
`replacement_bind 127.0.0.1:51670`, macOS AddrInUse/Os48. Its91080-byte log is SHA256
`78153a7443ff803009751be506278666a9b200a823b07f755cf8d40edc1e4a88`.
A delayed empty lsof query does not describe kernel state at the failed bind.

After the bounded independent investigation, Root declares exactly one final receiving
`cargo xtask ci` run81324. It passes1172/0/1 across169 result lines, including all later
groups omitted by97729, formatting, warnings-denied workspace/vendor clippy/tests,
layers, licenses and corpus. Its141627-byte log is SHA256
`a59098b56b4293de7e7da3f81b26ff1f2e61c6137d6ceea256cc6509b2e8ded6`.
This is receiving evidence, not a causal fix or a retry-until-pass policy.

The original76126-byte control test stays SHA256
`fd96b257cd6c00b52e97544d652ed69fdbaaaa454e1f3a9360d77f3292218fb0`.
All socket operations, live exclusive-bind assertion, endpoint/protocol checks,
two-second shutdown and200ms exchange deadlines and default concurrency remain.
No retries, sleeps, reuse options, assertion changes, environment serialization or
security-policy changes are introduced. The selected51670 address remains unexplained.

## Fixed20 default contexts

An independent worker declares20 natural default-parallel full-control contexts,
25 tests each; runner64662 exits0 with all500 passes. The fixed bound is exhausted.
An isolated test copy adds failure-only filtered selected-port diagnostics plus
mechanical paths to unchanged original child modules/fixtures. It links the same nine
Root dependency fingerprints. The first copy fails compilation because five
include_bytes paths were not routed; that setup error is retained and counts as no
runtime result. Only routing is corrected before the20 contexts run.

No context fails, so the collector never executes. These observations yield no
LISTEN/ESTABLISHED/TIME_WAIT/PID snapshot and do not establish a listener lifetime,
port reassignment, kernel-state or CDX cause. Its filtering is not runtime-qualified.

## One contradicted hypothesis

A separately declared single trial98404 checks whether a held outbound connection
claiming the released address prevents replacement binding. Actual shutdown257µs,
the non-reusing TcpSocket bind to127.0.0.1:56125, connection to a separate56124 sink,
all local/peer assertions and the200ms byte exchange pass. **The replacement std
listener also binds successfully.** The trial exits1 at its anticipated-collision
expectation. This contradicts the simple held-ESTABLISHED-outbound hypothesis on
this host; it is not a product failure or reproduction of97729.

The trial is preserved without correction/retry. Its failure-only collector is not
reached either. Other owners/states remain unproven; no competing-LISTEN trial runs.

## Source facts and continuation

- Actual Rust1.94.1 commit
  [e408947bfd200af42db322daf0fadfe7e26d3bd1](https://github.com/rust-lang/rust/blob/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/net/tcp.rs)
  documents OS-selected port0 and close-on-drop. Its
  [Unix bind implementation](https://github.com/rust-lang/rust/blob/e408947bfd200af42db322daf0fadfe7e26d3bd1/library/std/src/sys/net/connection/socket/mod.rs#L536)
  already sets SO_REUSEADDR. Adding that option is not a qualified fix.
- Exact locked Tokio1.54.1/Mio1.2.4 packaged sources transfer ownership in from_std;
  no listener clone appears in that conversion. Root sends stop and joins the
  listener-owning worker. Source lifetime is consistent with release, but does not
  reserve a global address after the last owner closes.
- [Apple bind(2)](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/bind.2.html)
  defines EADDRINUSE without identifying its owner/state. Historical public manuals
  do not attest every current Darwin rule. The observed49152..65535 ephemeral
  allocation range includes51670; that alone is not causal evidence.

Do not change binding policy, assertions or deadlines based on these observations.
A future durable selected-port collector needs bounded child termination/reaping,
reviewed platform parsing and privacy controls while preserving the original error.
The isolated helper has only an outer process-group ceiling and is not ported to Root.
Holding a clone defeats the release test; a hardcoded port merely moves possible
conflicts into setup. No portable post-close reservation is supplied here.

All investigation jobs30122/64662/27118/98404 and receiving81324 are terminal. Raw
sources, exact dependency fingerprints and logs remain ignored development evidence
under the research worktree's `target/loopback-shutdown-port-investigation/`; Root
retains its two aggregate logs in `target/cdx-file-end/`.

| Receipt | Bytes | SHA256 |
|---|---:|---|
| Independent REPORT.md |8104|`abaf620f17309b5c292eea7e288183ad5b0e1774cd25b56b63a08f59643ed954`|
| final-handoff.json |5273|`2d33fc9ca1347be8124fc9c58a89e44ddaf4edfdc933a0e13213cfca50c6c553`|
| results20.json |11042|`02f6915728ff27ed5d7be4472dbebfa40f87b8f1af16d2e2b97e9c5c028ddbf2`|
| counterexample-first-result.json |437|`dd8d029e765731e7161992220ef16c5751b09afe44c0ad02abbcb953b2370ea6`|
| logs/counterexample-first.log |249|`52073a0fb017c5a8ac6371ff9b622be521a6bfc85c048da4447e2df2c0f85e7f`|

Actions stays disabled. This diagnostic record establishes no hosted/platform parity
or completion of M0–M5.
