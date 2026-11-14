# Independent source-backed MOL display acceptance

2026-11-14. Independent test owner, initially based on `70bf174` in the isolated
`bondline-mol-display-tests` checkout. This record covers headless command tests;
it does not establish native-window appearance or complete MOL/chemistry parity.
GitHub Actions stays disabled and no desktop input or activation is used.

## Independent inputs and expectations

The test reads the unchanged original 740-byte
[`valence-absent-mixed.sdf`](../crates/io/tests/fixtures/sdf-preview/valence-absent-mixed.sdf),
documented SHA-256
`e5cbdc14329c59bad55148c5de37058de90f651af10966c30cf8b89b3b5efd39`.
It takes literal raw MOL spans, without parsing/rebuilding the SDF to construct inputs:

| Span | Molecular version | Individual-MOL model positions, pt | Display H |
|---|---|---|---|
| `0..371` | V3000 ethanol | `(42,60), (78,60), (78,24)` | `3,2,1` |
| `410..713` | V2000 methylamine | `(108,108), (72,72)` | `3,2` |

These positions follow the existing explicit 24pt/unit, origin `(72,72)` and y inversion.
They are not SDF-packed positions. Literal expected pages use these independently declared
coordinates/counts and the protected pre-existing literal renderer. Actual captures use
the shared `document.open_mol` and `render.capture` registry entries. The unchanged external
`mol_oracle.py` checks the exact actual MOL source with pinned RDKit2026.09.2: source atoms,
bonds, positions, neutral metadata, total H, SMILES and formula. It never supplies depiction
counts back to production or rewrites the input.

Pillow12.3.0 validates actual PNG framing/mode/dimensions and exact RGBA against the literal
reference. Separate OH/NH2 crops and independently calculated A4/source-coordinate bond
midpoints distinguish missing H, a missing bond and mirrored geometry. CPU/GPU identity
requires a real reported GPU adapter. This compares production paths rather than supplying
an independent rasterizer; the existing font/FreeType evidence and approved golden retain
their separate scope. No golden, font, old oracle, fixture, threshold or dependency changes.

[BIOVIA2026](https://www.3ds.com/assets/invest/2026-01/biovia_chemistry_2026_ctfile_formats.pdf)
defines absent versus declared valence. [IUPAC GR2.1.1](https://iupac.qmul.ac.uk/drawing/drawing.html#GR211)
distinguishes bare heteroatom labels from those carrying H. The existing original neutral
display profile has [pinned OpenSMILES evidence](https://github.com/opensmiles/OpenSMILES/blob/9240a2887158699503605bc838e1de9a9f7883f4/opensmiles.asciidoc#L249-L283);
this is not universal CTfile valence perception. See the
[source investigation and implementation boundary](research/mol-display-plan.md).

## Test scope

The new `crates/commands/tests/mol_display.rs` contains four tests:

- Both original molecular versions: strict complete backing, dense IDs, exact model
  coordinates and authored `None`; real CPU/GPU captures; unchanged whole state, close,
  source/history/file association; Unicode/misleading-suffix exact Save and reopen.
- Declared H0 and an unusual literal H3 on O remain literal. The latter tests authored
  finite valence arithmetic and preservation, not normal-valence chemical certification.
  Source-free and byte-proven native CDXML retain the separate existing literal policy,
  including absent H and explicit H0. No general CDXML inference is added by this feature.
- Deferred captures retain their historical complete MOL context through page history,
  source-file removal and a new molecular session. Early/late cancellation preserves the
  current owner, source, saved state, history, association and close intent.
- A neutral P with unresolved H outside the closed profile and a Double bond outside
  renderer support refuse the whole capture. Their complete sources remain exactly savable;
  no partial scene replaces them.

Ordinary MOL stays an authored Drawing with unknown H; derived display facts neither
become authored counts nor relax editing/export guards. The 25-command registry and
transport schemas are unchanged. Existing adapter and worker/lifecycle tests retain
their coverage; this slice adds no duplicate transport server or synthetic native proof.

## Actual RED

Before any display implementation was received, the isolated command below ran against
the real unchanged `70bf174` production code:

```sh
CARGO_TARGET_DIR=/absolute/path/bondline-mol-display-tests/target \
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
BONDLINE_PYTHON=/absolute/path/bondline/.cache/oracles/format-python/bin/python \
cargo test --locked -p bondline-commands --test mol_display \
  ordinary_valence_absent_mol_displays_oh_and_nh2_without_authoring_h \
  -- --exact --nocapture
```

Handle36380 exited101. Original raw-span, unchanged external molecular consumer,
fully-mapped backing, unknown authored H and exact coordinate assertions passed. The
actual ethanol CPU PNG then failed the intended Pillow assertion:
`ethanol: ordinary MOL OH/NH2 label differs from literal-H expectation`.
Methylamine/GPU/Save later in that loop did not execute in this first failure.
This is a behavioral RED, not a missing API or compilation failure.

`target/mol-display-red.log`: 5,182 bytes, SHA-256
`1fb2a969a3c234db31a33b6898367728617c1d2d3f06ecbe084ce214a5da1478`.
The exact 13,242-byte RED test source is retained in
`target/mol-display-red-source.rs`, SHA-256
`d5b4dab4215fa94b13470c08a38c4c18cdfd658f8707c96faba5a7c9a3042b89`.
Its reconstruction from the additive later draft was hash-verified against the identity
recorded immediately after the failure; no expectation was relaxed.

The protected baseline records 274 old source/test/fixture/oracle/golden/manifest identities
in `target/mol-display-protected-baseline.json`, SHA-256
`d7d32a4e760d71dc59af563602437c7c48a2408dd989a5c884878e3a48013b62`.
Receiving production changes must be classified separately from this baseline.

## Actual GREEN and final source

Root supplied real complete-owner capture support `c788787` and shared IO implementation
`fd51ca17f36caba16c412d6ef8165a3175e48490`, followed by its actual command routing.
No backend stub was used. The first real-backend run executed all four tests unchanged:
handle12551 exited0, four passed, zero failed/ignored. Both original MOL sources, actual
CPU/GPU images, unchanged external consumers, exact Save/reopen and all drafted guards
therefore executed successfully. No test input, expected pixel, tolerance or assertion
needed correction after the initial RED.

The full command package on that received v1 source, handle79556, exited0:
205 passed, zero failed/ignored across32 result lines including doctests. Its log is
`target/mol-display-command-package.log`, 24,565 bytes, SHA-256
`cabfe14565ef7a797f19a7d69b89580777223a08f947e567b443ef0b6cac51fb`.
Workspace `cargo fmt --all -- --check`, handle7642, also exited0.

Root then supplied an additive `Engine::drawing_page_scene` delegate for the native
caller's existing dependency boundary, after its UI compilation exposed the unavailable
direct IO dependency. This leaves registered capture behavior unchanged. Final received
`crates/commands/src/capture.rs` has SHA-256
`d01a313674cf9099c8059a065196ef34db269869ceabea76d675e5a267a64b5a`;
`crates/commands/src/lib.rs` remains SHA-256
`0c5fa1ff939d095b308ec93ca0f8632e4c4d511736be3e5c90b80816ee2ff483`.
`target/mol-display-backend-port-v2.json` records the exact received files and superseded
v1 identities. The final focused run, handle84172, exited0 with all four tests passing
unchanged. Final all-target warnings-denied command clippy, handle35137, and final
workspace rustfmt check both exited0. Full205 is deliberately retained as v1 evidence;
Root's final aggregate covers the additive delegate together with the native UI caller.

```sh
cargo test --locked -p bondline-commands --test mol_display -- --nocapture
cargo test --locked -p bondline-commands
cargo clippy --locked -p bondline-commands --all-targets -- -D warnings
cargo fmt --all -- --check
```

Every Cargo command used this checkout's absolute unique target, incremental0,
dev/test debug0 and the pinned external interpreter, as in the RED command above.
The acceptance test remains exactly the frozen 21,963-byte pre-backend draft, SHA-256
`de2c0130841d4f949bb161aaf74762e1158848a37931cc31cfa200b1ba0ed653`.
Removing only the additive guard cases/helpers and restoring the initial helper branch
reconstructs the original RED source byte-for-byte; both initial and final hash checks
match `d5b4dab4215fa94b13470c08a38c4c18cdfd658f8707c96faba5a7c9a3042b89`.

Of the274 protected baseline paths,270 remain byte-exact. Four replacements are explicitly
Root-received implementation files: command capture/lib, IO lib and SDF preview factoring.
Three additional received inputs are shared IO display code, its IO tests and Root's
complete-owner command regression. The original molecular/other format fixtures, all old
oracles/goldens/fonts, model/renderer/codecs, manifests and lockfiles remain exact. Final
received identities are recorded separately from the baseline in
`target/mol-display-received-inputs-final.json`; these are not worker-authored changes.

## Bounded review and limits

A read-only review of Root's pending routing found no material behavioral defect: source
kind comes from the captured/current complete owner, not association or filename; both
initial and post-command graph paths use that owner; typed display errors delegate their
codes; historical/cancellation behavior and source/edit/Save guards remain. That was source
review, not compilation. Root's later native dependency correction above is recorded
separately; this worker did not compile or execute the native UI.

This owner ran the scoped commands/package/lint/format checks, not a new full workspace,
native-window or Windows/Linux run. The expected raster still uses the protected literal
renderer; external chemistry and analytic control evidence have the separate scope above.
No general valence engine, CDXML inference, broader bond rendering or whole-M1 parity is
claimed. GitHub Actions stayed disabled. No live worker jobs remain at final handoff.
