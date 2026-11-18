# Source-preserving XML text receiving

2026-11-17, base8e030bf. [Contract](research/source-preserving-xml-text.md). Standalone
syntax only; imported-label authoring and text rendering remain missing.

## Independent tests

The separate author's first788B test, SHA256
`c4c5f5a346826680709897dcba8a850306d8659eb8871f2c7f0e59cafd32da7d`, fails with E0599:
Edit has no SetText. This is API/compiler RED, not runtime failure. The expanded ten-case
15008B declaration, SHA256 `fd6ac5cab4a82638a864008c267e03ab3bdecccf54a07d1348d4e4678ea2dd25`,
retains that prefix and also fails missing-API qualification before SetText implementation.

Helper regression7988 passes the old append/namespace/long-tag/resource cases before
adding the branch. Implemented suite93454 passes10/0 unchanged; full standalone35528
passes58/0 across13 result lines. No fake implementation stub or runtime RED is claimed for the
new variant. Later rustfmt changes test presentation only.

Expat is independently qualified on literal input and deliberately wrong text/attribute
controls before product output. Literal expected bytes and character data cover Unicode,
escaping, entity-looking text, CDATA/entities, fake closing tags inside CDATA, CRLF versus
referenced CR, NFC/NFD, clear/empty forms and unchanged trivia. Composition checks qualified
QNames, simultaneous attributes and descendant expansion before ancestor append at a
shared offset. Strict errors cover mixed content even on no-op, duplicate/content conflicts,
foreign/non-element ownership and invalid XML characters after valid edits. The old
attribute error stays exact. Long128KiB tags and independently assembled final XML exercise
escaped growth and propagation of the existing node-cap error.

## Fuzzing and local gate

ASan18323 passes10000 runs/2s using existing cdxml_edits. Added properties check decoded
values/no-ops, strict mixed-content/ownership/conflict errors and both-order composition;
old resource-error classifiers remain unchanged. The recipe adds ten original text seeds,
35 total, with independent source copies and a manifest outside the mutable corpus.
All35 original source pins still match. Initial seed reaches16384B; actual NEW/REDUCE
maximum is13164B. Final cov2951/ft8425/RSS447MiB. Log78935B/SHA256
`e6b371d10560d939054ae35946b759540de782f5e4fd0b232168195f8e03ac01`.
Committed fixtures are never mutated. This is bounded16KiB coverage, not full64MiB,
chemical, native or general foreign-file verification.

Aggregate CI88496 passes fmt/workspace warnings-denied clippy, then exits1 at the
historically intermittent MCP shutdown replacement bind (AddrInUse,127.0.0.1:64249).
It reports1164 passed/1 failed/0 ignored across190 result lines. Log147083B/SHA256
`e651b9c781a93f2e44392f5c0171731e036217040383a614785480d926136741`.
That is a failed aggregate, not a pass. No listener policy, test, timeout, retry or port
selection is changed, and the exhausted earlier diagnostic contexts are not repeated.

To finish coverage of the tests not reached by fail-fast, complete workspace21957 runs
once with `cargo test --workspace --locked --no-fail-fast`:1818 passed/0 failed/1 ignored
across269 result lines, exit0. Log226359B/SHA256
`1ebbe53e5f0f289b4dbc2ba8b067334dcc687d8caee5f2074e3de2a7c7df9862`.
Remaining38504 runs the exact not-started CI components: accesskit_macos lib tests3/0,
vendor warnings-denied clippy, layers, licenses/assets/notices and corpus, all exit0.
Log1885B/SHA256 `eabb26c93b54ca7fa31ff05df90c678adb9d6b87893be32874ec3cae0204bad0`.
Thus all component checks pass, with1821 total disjoint workspace/vendor tests and1
ignored; **the initial aggregate failure remains unexplained**. A later natural pass is
not a causal fix. No clean aggregate is fabricated by renaming component runs.

Pinned tools, CARGO_INCREMENTAL=0/CARGO_BUILD_JOBS=2, zero dev/test debug, existing
BONDLINE_PYTHON venv and fixed seed apply. Logs are ignored under target/cdxml-text/receiving.
Source review reports no material issue at its pinned readpoint, distinct from execution:
8866B/SHA256 `e09af58c8f749b2d8ccde1c7cd66058dc1fef35cebe77438117f613b77e6d4d8`.
Actual CLI46 output remains43338B/SHA256
`02bb76b9f909a3691fddbb5a1f3c2cc794a72f759f43194791b880213558a12c`.
No dependency/lock/vendor/font/asset/golden/workflow or protected listener policy/test
change occurs. Registry46 and catalogue16/124 retain their scope. This slice launches
no native app; user apps/drawings stay untouched. Full M0–M5, imported effective styles/
dependent Text/chemistry and fresh native/other-platform workflows remain required.
GitHub Actions stays OFF; no push/publication.
