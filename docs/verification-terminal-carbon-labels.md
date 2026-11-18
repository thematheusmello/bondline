# Terminal carbon labels — 2026-11-16

The [locked public-source contract](research/terminal-carbon-labels.md) follows
checkpoint `672744f`. This is a prerequisite for chain authoring. M0–M5 and chain
tools remain incomplete. The user's existing desktop bundle and draft are preserved.

## Delivered behavior

`Atom.show_terminal_carbon_labels` independently retains missing, false and true values.
Missing/null stays omitted in JSON and the legacy native-v0 profile. The node CDXML
attribute accepts only `yes`/`no` and is written before the nonterminal attribute.
Both model-to-MOL writers refuse every specified value; retained-SMILES replacement and
MOL regeneration reject metadata loss. Standalone parsers and source passthrough stay
unchanged. Root inheritance, child Text and foreign edit authority are not inferred.

One renderer selects the terminal flag at actual degree zero/one and the nonterminal
flag at higher degree. The other flag is dormant. Active false requires known literal/
resolved H and checked Single/Double/Triple weight plus H equal to four. Unbonded carbon
and actual isolated C2 components retain full labels. Existing near-collinear/two-Double
rules are unchanged. Hidden glyphs and masks both disappear; hits, IDs, geometry and all
real-ink guards remain. No H is inferred or erased.

Existing `atom.set_label` restores each false flag independently, including dormant flags.
Same element/H with false creates one ordinary edit; repeated true/None retains validated
no-op sharing, redo and close behavior. CLI, initialized stdio, authenticated loopback
and the existing editor share this operation. No command, parameter, control, authority,
dependency or asset is added. Registry 38 and control counts 31/20/14/10 stay unchanged.

## Test-first record

Root executes checks; workers provide declarations and bounded source reviews only.
Original declarations and logs are under ignored `target/terminal-carbon/`.

| Stage | Observed result |
| --- | --- |
| Model first / expanded | Runtime RED 0/1, then 0/3: field rejected. Separate E0609 API compilation is qualification, not runtime evidence. |
| Model `22363` | GREEN: three JSON cases and one typed API case. |
| IO first / `69200` | Runtime RED 0/1, then 0/3: missing projection/encoding and all four loss paths. Manual checker runs before generated files. |
| IO `36896` | GREEN 3/0: native-v0/source preservation and both MOL/retained-source loss refusals. |
| Render `61055` | Runtime RED 1/5 after data support. Mandatory isolated-carbon controls already pass. |
| Render `8872` | 5/1: a new tiny-Double test guesses the wrong existing error class; qualification below. |
| Render `76626` | GREEN 6/0 after correcting that new expectation and adding an independent pixel-guard control. |
| Engine `2033` / `72096` | First failure is a data boundary, not setter proof. With data support, intended setter RED 0/2 leaves terminal false. |
| Wire `53305` | Intended setter RED 0/2 through actual CLI, initialized stdio and authenticated loopback. |
| App | First filter selects zero tests. Correct `native_atom_label::terminal_label::` executes one and fails after Apply with terminal false instead of true. |
| Engine / wire / App GREEN | Engine 2/0; wire `40823` 2/0; App `33638` 1/0. Hidden endpoint paint/picking, Cancel, Apply, history and actual Unicode Save/reopen pass. |

There are 18 new cases: model 3, API 1, IO 3, renderer 6, Engine 2, adapters 2 and App 1.
Wire/App declarations are post-render-draft acceptance and pre-setter tests, not
pre-render TDD. Whole literal XML expectations precede the unchanged Expat/Decimal/RDKit
consumer. The IO checker qualifies nine manual positives/nine negatives before writer use.

Only the old JSON negative for the newly recognized field becomes a positive check on
the identical input, preserving old fields and exact serialization. Other aliases,
invalid values and old assertions remain. A new-test `Value` annotation is an equivalent
unused-import correction. Initial prefixes and pre-format sources are pinned in receiving.

The retained 0.5-point Double reaches the unchanged `9 * width > length` preflight,
returning `UnsupportedChemicalContent` before the separate pixel-scale check. The new
assertion now requires that exact error. An additional 48-point Double at an 80-pixel
viewport passes preflight but its lower arm misses pixel centers, requiring `BondScale`.
No product threshold, old input or guard is relaxed; see `render-guard-qualification.json`.
Bounded independent data, renderer and integration reviews find no material issue and
pin their reads. Integration review captures pre-format declarations; its recorded drift
is the subsequent root rustfmt, not a claim to have reviewed unseen final bytes.

Full local gate `19445` exits 0: **1,540 passed, 0 failed, 1 ignored** across 221 result
lines. Formatting, warnings-denied workspace/vendor clippy, workspace/vendor tests,
layers, strict licenses and pinned corpus all pass. Log 191,894 bytes, SHA-256
`477ba18d66a369247fb4149055b5e87b054aae6c01562749a54ab702dd2a1e1e`.
Native/CLI build `47945` exits 0; native SHA-256
`9a334fa99bd2bc82767ceeeb5b22d3619ebe7fe1de2445b2d840abf19a7e86b1`, CLI
`6bda7cb347e990d6388a15b1cdb65175e7eb67b3fea2c5d874e85be1f49049c3`.
The user's copied executable stays exact `d0873d6c…5c64`. Lock, workflow, old consumers,
standalone parsers and approved golden remain unchanged. The protected MCP test remains
`fd96b257cd6c00b52e97544d652ed69fdbaaaa454e1f3a9360d77f3292218fb0`.
This pass neither attributes nor repairs historical replacement-bind failures.

## Reproduction and limits

```sh
export BONDLINE_PYTHON="$PWD/.cache/oracles/format-python/bin/python"
cargo test -p bondline-model --test terminal_carbon_json --test terminal_carbon_api --locked
cargo test -p bondline-io --test terminal_carbon_formats --locked
cargo test -p bondline-render --test terminal_carbon_labels --locked
cargo test -p bondline-commands --test terminal_carbon --locked
cargo test -p bondline-cli --test sdf_capture_adapters --locked terminal_label_wire::
cargo test -p bondline-ui --lib --locked view_tests::native_atom_label::terminal_label::
cargo xtask ci
```

App execution at density 2 does not establish native OS/monitor input. No approved golden,
vendor defaults, settings, chain-tool or Windows/Linux parity is inferred. Unchanged
standalone parsers do not need repeat fuzzing here. Actions stays off; no publication.
