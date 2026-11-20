# Editing a chemical label after annotating a CDX drawing

Verified on 2026-11-20. From a chemical drawing, add a page caption, change
NH₂ to OH, change Cl to Br, then edit the caption again. Each action gets one
Undo entry. Undo and Redo recover the exact bytes, entire model and dirty
state at all four stages. Unicode Save As, reopen and a subsequent inverse edit
also passed. The two atoms and a standalone caption can each own a Text with
raw ID 73. The writer locates the atom through its fragment and Node.
[Source contract](research/cdx-mixed-label-edits.md),
[command workflow](../crates/commands/tests/cdx_mixed_label_workflow.rs).

The pre-change CLI opened the independently constructed edited files but
refused the six attempted writes. Four IO cases and the command workflow
also failed against the previous writer. Thirteen original binary fixtures
now pin the expected chemical edits and refusals. An independent Python
record reader checks untouched records and the FontTools oracle supplies the
label geometry. RDKit 2026.09.2 separately recovered the expected chemical
graphs. [Fixtures and reproduction](../crates/io/tests/fixtures/cdx-mixed-label-edit/SOURCES.md),
[IO checks](../crates/io/tests/cdx_mixed_label_edits.rs).

| Starting source | Applied change | Expected CDX bytes |
| --- | --- | ---: |
| Chemical drawing after its first caption | NH₂ → OH | 431 |
| Mixed drawing after an additional caption | NH₂ → OH, then Cl → Br | 599 |
| Caption precedes both chemical fragments | NH₂ → OH | 367 |
| Water with two connected bonds and a caption | Central atom becomes NH | 529 |
| Direct caption at maximum signed depth | NH₂ → OH | 599 |

The off-page control changes Cl at x = 12 pt to Mg. The expected left box
edge moves from 2.888671875 pt to −3.404296875 pt. Rendering the full
candidate rejects this change while preserving the previous file, model,
view and Redo. Editing a label requires no free caption depth or object ID.
The separate caption-creation overflow checks still apply.

The native App tests keep Text selected throughout the create-and-edit
workflow. Exact Alt-primary on an exposed chemical target opens its label
form. A frontmost painted caption blocks selection of the obscured atom.
Ordinary Text clicks still edit the painted caption. The App suite passed
361 tests with one ignored. The actual macOS host, CLI and initialized
stdio MCP each produced the same seven files as the independent fixtures.
All fourteen frames on each surface matched CPU and Apple M3 Max Metal at
100% and 200%. The background host approved Close and exited. The App
harness supplied pointer events. The receiving host used authenticated
loopback. [Pointer tests](../crates/ui/src/native_caption_tests.rs),
[continuous Text workflow](../crates/ui/src/native_caption_creation_tests.rs).

The receiving receipt for `7deea1f` is
`target/cdx-mixed-label-edit-preparation/receiving/summary.json`, SHA-256
`8d90a04d96173d7f6d4e71f49aa9c625dd94f6dcb37e2aad814857d41cba1b7a`.
AddressSanitizer passed 10,000 trials with 208 unchanged input files. The
oracle compared all records outside the four permitted target properties.
Final reported RSS was 429 MiB. [Fuzz reproduction](../fuzz/README.md#editing-atom-labels-beside-cdx-page-captions).

Offline Chrome 157 ran the Rust IO example with the newly annotated
chemical page and the mixed page. It compared three edited outputs with
the independent files and reopened each, checking both the edited atom and
every direct caption. The Wasm module produced by wasm-bindgen 0.2.130 was
1,257,067 bytes with SHA-256
`b41b5b443e9ea195fa0cdcf408c7bd9ccaa5589f364bcd0a18d57e2128300558`.
The browser check covers the codec. The drawing UI and desktop input on
Windows and Linux remain unverified for this change.
[Rust browser example](../crates/io/examples/cdx_caption_browser.rs).

The integrated `cargo xtask ci` passed 2,353 workspace tests with one ignored,
all three AccessKit tests, warnings-denied clippy, formatting, layers,
licenses, assets and corpus checks. Its log is
`target/cdx-mixed-label-edit-preparation/workspace.log`, SHA-256
`fc548231185e4d9e7c955778bc6b91b35cb6cba828d800802a2257e2ef0f101f`.
The commands crate also type-checked on Windows MSVC, Linux GNU and wasm32.
The serialized 51-command registry retained SHA-256
`3da5b0fa925dfa00fff8aa4a4d1851574d0e83c93957607333b2a48ab4b05079`.

```sh
cargo test --locked -p bondline-io --test cdx_mixed_label_edits
cargo test --locked -p bondline-commands --test cdx_mixed_label_workflow
cargo test --locked -p bondline-ui --lib mixed_cdx
cargo test --locked -p bondline-cli --test sdf_capture_adapters cli_preserves_the_new_caption
python3 xtask/oracles/cdx_caption_browser.py
```
