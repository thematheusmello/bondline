# Rotated methine-star attachment control

This is a separate original fixed-geometry control for the single-ring work. It
does not use the new ring/search API or alter its frozen nine IO cases, the ring
CH2 regression, any existing overlap control, font or golden.

The fixture is C4H10: central C/H1 at (128,128) pt, three C/H3 terminals 88 pt
away on 10°, 130° and 250° rays, and three directed Single bonds. The 256 pt
square paper is rendered at 576×576 pixels, giving the existing exact transform
32+2×coordinate. The 120° relationship is analytic; this orientation is an
original test choice, not a reference-product default.

`literal_rotated_star_attachment.rs` checks the literal graph/counts and exact
lengths/directions, public literal versus same-count attachment-safe scene,
three Bond hits and actual black incident-bond pixels 20 pixels from the central
C, visible central H ink outside C and the bond corridors, complete immutable
Document equality, and actual CPU/GPU frame equality for both scenes. The C
corridor exclusion uses the conservative 16×24 pixel box around the existing
pinned Roboto C; no reference font or asset is introduced. The retained explicit
counts are [1,3,3,3]; they are never replaced with unknown H.

The first runtime, session **22018**, exits0 with **1 passed/0 failed/0 ignored**.
Warnings-denied scoped clippy74760 and the owned-file rustfmt check also exit0.
The exact 10° star therefore does **not** reproduce the hypothesized limitation
of the four existing H placements. This is passing-first-run coverage, not a
manufactured RED and not a renderer fix. It does not establish that every
rotation, the producer's distinct 12-carbon bulky candidate, or general label
congestion is supported. The ring CH2 RED25524 and its verified degree-two
attachment correction remain separately recorded in
[the ring attachment evidence](verification-literal-ring-attachment.md).

Before runtime, source inspection found that Point2 getters are `x_pt`/`y_pt`,
not the draft's `x`/`y`. The original draft and exact accessor-only correction
are retained under ignored `target/rotated-star-attachment/`; no test input or
expectation changed. No compile/runtime failure is claimed for that draft.

Commands use Rust1.94.1, this worktree's absolute target, incremental0 and
dev/test debug0:

```sh
cargo +1.94.1 test -p bondline-render --test literal_rotated_star_attachment --locked
cargo +1.94.1 clippy -p bondline-render --test literal_rotated_star_attachment --locked -- -D warnings
rustup run 1.94.1 rustfmt --edition 2024 --check crates/render/tests/literal_rotated_star_attachment.rs
```

The runtime test is 5,205 bytes, SHA256
`cd8050a962c3bbf96cbeb0b5f3705a3de7cf607116a0b41e707bf0ecc68ac193`.
The first log is510 bytes, SHA256
`08fe0059dfa45d792abf068f36b8ee86974c49702dcff2f5d6cec5f8d677a0a4`.
The renderer input remains the authorized 33,741-byte degree-two correction,
SHA256 `5e618385af247189e634de5c557f0a01c78c21f5a1351a1a3323069f78a27973`.
The frozen prior receipts and all39 snapshotted protected inputs remain exact.
No GUI/native input, hosted run, dependency, old test, old oracle or product
source change occurs in this control.
