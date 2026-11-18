# Refreshed workspace verification

2026-11-14. The actual Bondline application now uses an original light header, compact
two-column bond rail, white chemical paper and a right document inspector. It follows
the hierarchy studied in [pinned public references](research/visual-direction.md), with
original Rust/layout/glyphs and existing permitted typography. No dependency, lock,
chemical golden, font, old fixture or external oracle changed.

## Independent frame checks and regressions

The two presentation tests declare64/248-unit side panels at1280×900 and760×520,
actual accessible tools, truthful6atom/3bond/3fragment counts and exact authored/chemical
geometry preservation on real Double activation. Initial60381 fails the missing rail.
Root21748 exposes an over-wide76-unit Grid;97892/34745 then expose a new test locator
matching a tooltip Label instead of a Button. Requiring its existing Button role fixes
only that locator; original inputs/assertions remain. The original source and failed
logs are retained. The first independent GREEN on the earlier styled source is22936.

Broader unchanged tests found additional real regressions:

- Root58127: footer policy clipping and host100001 dimensions masked by side panels.
  Validate the outer viewport before layout and clear stale gestures/hover on refusal.
- Root93079: an auto-height multi-row footer changes paper coordinates between passes.
  Fixed84 height passes16/17 attachment tests22291; first-frame policy remains absent.
- Pinned egui's new Grid requests a discarded first pass; Painter suppresses later text
  while that flag is set. A read-only probe proves the second-frame policy fits its clip.
  Automatic wrapped tool scopes then incorrectly expand the rail to94 (30384). Explicit
  two-button horizontal rows avoid both problems without changing bindings or event order.
- Root84097:71 library and17 attachment tests pass, but the old batched-drawing case
  correctly exposes another layout regression. The84-unit footer shrinks A4's label em
  below the unchanged8-unit guard at1000×700. Move tool policy/errors into the existing
  inspector and keep a fixed24-unit page/job footer; do not weaken the guard or test.

Final Root94657 passes118 UI tests, zero failures, one ignored development-only capture
utility. All17 attachment and11 drawing tests are unchanged. The final independent replay
passes both presentation tests on the same binary/source. UI source SHA-256:
`98d0dd17b413a34b248a501f347372260634e8a94a333eb7d61ff8752965bca2`.
Read-only diagnosis is retained in the worker's ignored
`target/workspace-attachment-review/review.md`, SHA-256
`e894a0006894ffc72646c9a18059a1beb5fb43d2e3d9b79070d2e38217196f42`.

Root aggregate52371 passes `cargo xtask ci`:891 tests, zero failures, one ignored explicit
capture helper across138 result lines. This includes workspace tests, formatting,
warnings-denied all-target clippy, the3 Mac bridge tests, layers, declared runtime licenses,
two assets and the8-file/5-bundle pinned corpus. Log `target/multiple-ci-final.log`:
107345bytes, SHA-256`b501300c736fc66eab376f6cefd7620340c376899fe748858939ffe25c7d5403`.
No new Windows/Linux or hosted execution follows from this local pass. Earlier aggregate
failures for stale exact command/control discovery lists and the historical listener
assertion remain recorded in the authoring and shutdown evidence.

After checking intermediate trees, shared build output produced old-API metadata errors
in20355 and Root87867 despite unchanged coherent source. The intermediate backend passes
883 tests under isolated CI58584. Prior-export dep-info is present in the shared cache;
the exact Cargo freshness mechanism remains unproven. Preserve both failed logs. Root
archives the complete old debug directory as `target/debug-pre-index-recovery`, with
`target/debug-index-recovery-relocation.json`, then rebuilds the normal default target
from scratch. No source or assertion changes are needed. Final clean Root1724 passes
the complete891-test gate again, with zero failures/one ignored utility. Log
`target/multiple-clean-final-ci.log`:119044bytes, SHA-256
`f8f05260270fba0f3ce65ab80c49c194070c0318fc940c934e6acea846afd192`.
Use separate absolute targets for future index exports, just as for worktrees. The
archive retains the old captured test binaries and fingerprint evidence without deletion.

## Actual desktop review

The user explicitly authorized opening Bondline for this review. Root alone launched the
actual macOS app using the audited wgpu backend: Apple M3 Max / Metal. No reference-product
binary was inspected. The final native binary SHA-256 is
`bbf429b73856327394478913723230b4b5b1d2690188b38edfffe86cb5f8827f`;
its UI source is the final98d0 hash above. The initial84-footer/Grid build and its logs
are retained separately; it was saved and cleanly closed before refreshing.

The original420×280pt scene contains Single/Double/Triple carbon pairs, literal H3/H2/H1,
six atoms and three fragments. It was created through the authenticated shared registry,
saved as native CDXML and reopened in the final executable. Actual native inspection
shows all components/labels, correct inspector/file state and Double selected. Native
accessibility activation changes Single→Double and its contextual policy. File→Open
shows CDXML, CDX, MOL V2000/V3000, SDF and SMILES choices; Cancel returns to the retained
drawing. These are finite verified native interactions, not full format compatibility.

Two early CUA drag attempts produced no authored change. They are not native drawing
passes; their event/focus-delivery cause is unresolved. The final scene was registry-seeded,
not claimed to have been drawn through those attempts. Existing independent real-frame,
file, renderer and adapter tests retain their own scope. Full native Double/Triple
authoring/attachment/history/save acceptance, Windows/Linux and reference style parity
remain unverified. No screenshot is a new human-approved chemical golden.

Unedited actual native JPEGs are published with [media provenance](media/SOURCES.md).
The app is intentionally left open for the user's visual review; native launch waiter89286
and its authenticated loopback listener remain live. Credentials are only in ignored startup
output, never these screenshots/docs. CUA completion was recorded once after the final
native screenshot. Future agents must check user availability before further interaction.
