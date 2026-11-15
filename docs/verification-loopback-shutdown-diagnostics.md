# Loopback shutdown diagnostic context

2026-11-15. This changes only error context inside the existing
`shutdown_without_clients_releases_the_listener_address` test; no production fix or
socket-policy change is claimed.

Root full gate38859 fails the old case with macOS Os48/AddrInUse after740 passes. Its
89,289-byte log is SHA2563d90516f48c3924cc1aaed11e7390237580c7f81f208b7faaf3e366945583b65.
The original test does not label its fallible operations or chosen port, so the historical
phase, owner, listener lifetime, TIME_WAIT or port-reassignment cause cannot be identified
retrospectively. Source reading shows shutdown sends stop and joins its listener-owning
worker, but does not prove every OS outcome.

Before changing code, an independent worker declares a bounded investigation against the
same compiled MCP/Tokio dependency fingerprints: five exact isolated runs, three default
25-test contexts, then one phase-labeled probe of100 solo and200 concurrent lifecycles
alongside2048 connection attempts. All pass; failure-only socket snapshots never run.
These negative diagnostics do not replace the failed aggregate or establish its cause.

The one-function patch adds phase and known local-listener address to fallible errors.
It retains ErrorKind and original error display, not the wrapper's raw_os_error accessor.
Initial-bind errors report requested127.0.0.1:0 because no chosen address exists yet.
No token, data payload or unrelated process argument is recorded. Exact reconstruction
proves every byte outside the function remains original.

All socket operations, live exclusive-bind assertion, endpoint/protocol checks, two-second
shutdown and200ms exchange deadlines and default test parallelism remain. Accept/connect
still run concurrently. No retries, sleeps, reuse options, assertion removal or global
environment settings are added. The focused1/0 and default25/0 final annotated runs pass
once each; warning-denied direct rustc compilation and rustfmt pass. Root's later complete
local gate71585 passes1158/0/1, including the diagnostic case, with warnings-denied clippy.
That pass is verification, not a causal explanation for the preserved failure.

Ignored evidence is retained at target/smiles-ring/loopback-shutdown-investigation/.
The4163-byte final handoff is SHA256
9cb89497f3f67736fcae5ec3e6b464f9b308213804af1a99be78457082330d07;
the final76,126-byte test source is SHA256
fd96b257cd6c00b52e97544d652ed69fdbaaaa454e1f3a9360d77f3292218fb0.
All diagnostic jobs are terminal. Future naturally occurring failures now identify a
phase without weakening exclusive loopback ownership or security boundaries.
