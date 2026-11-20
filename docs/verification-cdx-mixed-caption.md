# Viewing chemical structures and captions together in CDX

Verified on macOS on 2026-11-20, from `0f76377`. The prior CLI refused all
29 independent inputs. The new reader admits 12. The other 17 still produce
the same error code and original record number. Both pure profiles retain
their existing workflows. [Source contract](research/cdx-mixed-caption.md),
[originals and oracle](../crates/io/tests/fixtures/cdx-mixed-caption/SOURCES.md).

The 12 positive inputs span five framings, boxed labels, water, cyanide,
two Center encodings, anonymous and sparse caption IDs, multiple captions,
and both physical sibling orders. Tests compare the complete imported model,
the exact borrowed CDX bytes, and source authority after a detached edit. The
model can be viewed and captured. Its separate mixed-page grant excludes both
chemical and caption writers. One small run-budget test exercises a real
drawing with one caption run and two atom-label runs. Its decoder succeeds
at three and refuses at two. The application limit is 4,096.
[Decoder](../crates/io/src/cdx/mixed.rs), [IO tests](../crates/io/tests/cdx_mixed_caption.rs).

A blue caption intersects a red atom label. The expected sample coordinates
come from the pinned font outlines. At 100% the overlap is at (166, 122), and
at 200% it is at (332, 245). The samples include red-only and blue-only ink.
The visible foreground must stay blue when the sibling order is reversed.
The IO tests also exercise the case with two captions.

The App test initially showed an active drawing tool in the inspector.
After correcting the inspector, the same test caught a bond gesture starting
on read-only paper. All 15 adjacent caption App tests passed with both changes.
The mixed-page test opens through the chooser, checks the inspector and tool
availability, exercises gestures and zoom, then saves and reopens a Unicode
copy. [App test](../crates/ui/src/native_caption_tests.rs).

The actual signed macOS test executable ran hidden, controlled through its
authenticated native host. It opened both source orders, refused five
authoring requests without changing state, saved both 368-byte drawings
exactly and reopened them. Each of the two views agreed across CPU and
Apple M3 Max Metal, including all 24 color samples. Source order also left
the complete images identical. The host exited on approved Close.

| View | PNG bytes | SHA-256 |
| --- | ---: | --- |
| 512 × 512 at 100% | 6,609 | `c5e5da2413f3434813b3e889d16a9187de396e168052f7f0d23a2540dde87ca9` |
| 1,024 × 1,024 at 200% | 23,175 | `47311fe9f5bdf74cb6f6cda59461f3e703a1d73f8f2c1586aa05741f907219d2` |

The final AddressSanitizer run completed 10,000 trials in one second.
All 132 immutable seed hashes matched after the run. The largest newly
retained or reduced input was 1,397 bytes, and peak reported RSS was 238 MiB.
The oracle checks original ownership, declared identities, run content
and source retention, including denials after detachment. The commands crate
type-checked for Windows MSVC, Linux GNU and wasm32. Wasm emitted eight
warnings. Physical OS input for mixed drawings and runtime verification on
other platforms remain open.

The local evidence directory is `target/cdx-mixed-preparation` in the primary
checkout. The native receipt is `native/run-nwd0eu3r/verified.json`. The final
sanitizer log SHA-256 is `65f7213154fea944457574c606b4b126d49492edd42b1f728095e6082d0857a5`.

```sh
python3 -B crates/io/tests/fixtures/cdx-mixed-caption/generate.py --check crates/io/tests/fixtures/cdx-mixed-caption
cargo test --locked -p bondline-io --test cdx_mixed_caption
cargo test --locked -p bondline-io --lib page_caption_and_both_node_labels_share_one_run_budget
cargo test --locked -p bondline-ui --lib native_caption_tests
```

The fixture replay uses Python with FontTools 4.61.1. Full mixed editing,
captions behind chemistry, ties, arbitrary styles and other object classes
remain in the parity backlog.

The first full local CI run stopped at the existing MCP listener replacement
bind on 127.0.0.1:55042, after 1,556 passes and one failure. The same local
gate was rerun with the listener and assertion intact. The retry passed
2,329 workspace tests with one ignored, all three AccessKit tests, rustfmt,
warnings-denied Clippy, layers, licenses, assets and corpus verification.
The retry log SHA-256 is `68b24f9fedfcd8386faaf46cd21723c6ff2a3915a346b395e5b7ac1b2e12c763`.
The intermittent bind cause remains open. GitHub Actions stayed disabled.
