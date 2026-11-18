# Portable independent clearance checks

These scripts find the repository through its pinned `assets/fonts/Roboto-Regular.ttf`;
no absolute home path is embedded.
All mathematical functions/classes are AST-identical to the frozen independent generator.
Only repository/input/output routing and CLI defaults changed. No product module, product
output, reference asset, fitted measurement constant or new dependency is imported.

The numeric generator and validator require Python plus external `fontTools==4.61.1`.
Optional painted-span regeneration requires external `Pillow==12.3.0`, without FontTools.
The checked system here is CPython3.13.0. Nothing is installed or shipped by these scripts.
The validator imports only its sibling independent `clearance_oracle.py`.

Defaults write ignored `target/connected-labels/` files, never the frozen renderer fixture.
Use explicit paths to compare with the tracked independent fixture after integration:

```
python3 xtask/oracles/clearance_oracle.py --output target/connected-labels/reproduced-clearance-vectors.json
python3 xtask/oracles/validate_clearance.py --vectors crates/render/tests/fixtures/connected-label-clearance.json --output target/connected-labels/clearance-self-checks.json
python3 xtask/oracles/clearance_pixels.py --vectors crates/render/tests/fixtures/connected-label-clearance.json --output-dir target/connected-labels/clearance-preview
```

The generator retains its original `elapsed_seconds` measurement. Compare every JSON
field/case exactly after removing that field only; do not rewrite the frozen fixture or
change its hash as an automatic build step. Local regeneration matches all semantic fields
of the128-case frozen fixture. No cross-platform byte-identical libm result is promised.

The portable validator also reproduces the frozen external numerical check record, and
portable Pillow reproduces the10 painted-span cases against the frozen input. The script
outputs include source-vector hashes, so a different timed regeneration has a different
input hash even when every semantic vector is equal. That is provenance, not a reason to
loosen numerical assertions. Original/frozen oracle code and rejected AABB countermodels
remain preserved outside this portable directory.

The earlier independent oracle's closed-zero-margin numerical qualification remains in
the validator result; no production defect or relaxed assertion is implied. Source/current/
Save/native and actual product rendering remain separately verified by Root's lanes.
