# Shutdown test portability follow-up

The shutdown regression is now split into accepted-client closure and listener-only
address release. This is a test-only robustness correction informed by public source;
production behavior is unchanged. The package passes 50 tests with normal parallelism
on macOS. Windows/Linux execution remains unverified.

The [earlier diagnostic evidence](verification-listener-shutdown.md), its observed
initial connection success followed by reset, and the frozen 49-test handoff remain
unchanged. Their macOS results do not establish portable immediate rebind after an
accepted TCP connection.

## Source-backed portability concern

Pinned [Rust 1.94.1 listener construction](https://github.com/rust-lang/rust/blob/1.94.1/library/std/src/sys/net/connection/socket/mod.rs)
enables SO_REUSEADDR on non-Windows targets and deliberately omits it on Windows.
The [Windows socket constructor](https://github.com/rust-lang/rust/blob/1.94.1/library/std/src/sys/net/connection/socket/windows.rs)
does not substitute SO_EXCLUSIVEADDRUSE. Thus the same std API has different
address-reuse semantics across targets; Unix rebind success cannot establish the
Windows outcome.

Microsoft's [closesocket contract](https://github.com/MicrosoftDocs/sdk-api/blob/f1bce0b5a4909bd37281d5b0cfb4ce4c177ae776/sdk-api-src/content/winsock2/nf-winsock2-closesocket.md)
allows default graceful transport teardown to continue after handle closure.
[Its bind contract](https://github.com/MicrosoftDocs/sdk-api/blob/6799c0f3e3fdb30a59b694cc007a52583bb7b9ea/sdk-api-src/content/winsock2/nf-winsock2-bind.md)
allows address-in-use rejection when reuse is not enabled. Neither promises that
closing a listener and its accepted socket makes the address immediately bindable.
The [Winsock reuse guide](https://github.com/MicrosoftDocs/win32/blob/289d68fcbf23cf2e2500241e0f6433bb2daa000f/desktop-src/WinSock/using-so-reuseaddr-and-so-exclusiveaddruse.md)
includes a delayed-rebind example specifically using SO_EXCLUSIVEADDRUSE. That
example is not proof that our different default configuration always fails.

[RFC 9293 section 3.6 and Figure 12](https://www.rfc-editor.org/rfc/rfc9293.html#section-3.6)
distinguish two-way TCP closure from receiving a FIN. In a normal server-first
close, client EOF can coexist with client CLOSE-WAIT and server FIN-WAIT-2.
Closing the remaining client may complete the FIN exchange and leave the server
in TIME-WAIT. A reset is a different path. The test permits either EOF or reset,
so it must not rely on reset always removing transport state immediately.

Inference: the combined accepted-client/immediate-rebind test could reject a
correct Windows shutdown because of retained TCP state. This exact Windows
failure was not observed. `drop(idle)` cleans up the test's client handle and
allows a graceful exchange to finish; it is neither necessary to prove that the
server released its own Rust handles nor sufficient to guarantee immediate
rebind after server TIME-WAIT. No option, abortive close, unsafe boundary,
production workaround, dependency or wait-for-TIME-WAIT loop was added.

## Direct, separate contracts

`shutdown_cancels_an_authenticated_partial_client` authenticates before leaving
partial command input, preserving proof of an actually serviced accepted client.
It retains the two-second shutdown bound and requires EOF, ConnectionReset or
ConnectionAborted in the 200 ms read window. Bytes, timeout and other errors fail.
It explicitly drops the client after this observation. It makes no immediate
rebind assertion about that connection's transport state.

`shutdown_without_clients_releases_the_listener_address` creates no accepted
connection before stopping the server. A second bind must first fail with
AddrInUse. Shutdown remains bounded below two seconds, followed by an exact-address
ordinary bind and the existing 200 ms positive connect/accept/address/byte exchange.
There is no accepted connection from the stopped server to introduce TIME-WAIT
into this address-release assertion. This does not promise that no unrelated
process can race for a released port; such a failure is reported, never retried.

The two independent lifecycle requirements remain covered. All other control-test
bytes, the earlier verification document and all runtime code are unchanged.
This source-informed test split needed no artificial failing production change.

## Actual local checks and frozen records

Rust 1.94.1; macOS 27.2 (26B123), arm64; isolated absolute Cargo target;
incremental and dev/test debug disabled; unchanged root lock and pinned Python.
No test-thread override or test retries were used.

```text
cargo +1.94.1 test --locked -p bondline-mcp
cargo +1.94.1 clippy --locked -p bondline-mcp --all-targets -- -D warnings
cargo +1.94.1 fmt -p bondline-mcp -- --check
```

Package session 18261 passed all 50 tests: 19 library, one MOL ownership,
25 control and five registered UI-work tests. Clippy session 74168 and final
rustfmt exited zero. The first fmt check reported only the two new function
signature layouts; rustfmt corrected them before package execution. That failure
log is preserved, not counted as a behavior RED.

| Ignored log | Bytes | SHA-256 |
|---|---:|---|
| `listener-shutdown-portability-mcp-tests.log` | 5,783 | `a191697533c802cb2a0ca815fc3b37b787b727dc0a31edecb53d948f0e5db661` |
| `listener-shutdown-portability-clippy.log` | 184 | `219ada138d6fb41d65bab2e39b8e34a5fa7317149d7bcaf5c3f39c252b394c35` |
| `listener-shutdown-portability-fmt-first.log` | 1,012 | `44882d4acfa0298c2c9e3f74796d4d0250977a6bb326eb6048a9f2505678467a` |
| `listener-shutdown-portability-fmt-final.log` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

The ignored portability handoff includes exact public source bytes/hashes, the
focused diff against the previously received test, unchanged old handoff/doc and
protected-input identities. Production `control.rs` remains SHA-256
`871277936b9d61f99b3270833d7a57573d0a58d70bad681c56489c461affa6d2`.
The original 11,083-byte handoff remains SHA-256
`4fcfada3966767252c0aa3076cda8211bde215ce89480c7903df3d48086acc3a`.

No new Windows/Linux, hosted, GUI or native-window execution is claimed. Actions
stays disabled. Root's receiving aggregate remains separate. No Git action was
performed; all owned jobs have exited.
