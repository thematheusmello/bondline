# Original printed-label creation vectors

Original Bondline test data, MIT OR Apache-2.0. Frozen on base `7113237` against
[the locked creation contract](../../../../../docs/research/printed-label-creation.md)
`126f99e`, before IO/caller producer changes. No reference executable, sample, export,
icon, font download or proprietary asset was used. The original five-node base is the
previous independently authored Mixed fixture; every new child/ID/expected byte is
declared here using independent Expat/Decimal splices and FontTools measurements.

The public Node/Text/IntroCDX/DTD and resource evidence is linked in the contract.
Child Text display precedence, four-byte object IDs versus UINT16 font IDs and explicit
style syntax supply evidence. Global fresh reservations, canonical body/no-p/Left+Left/
whole-element anchoring and exact append lexemes are original Bondline policies, not
claims of universal reference pixels or26.1 execution.

`creation-vectors.json` pins all20 source byte lengths/hashes, explicit black12 and
blue14 styles and stable baseline reservations. `creation-facts.json` records independent
source/model IDs, nodes/flags/neutral metadata, Text IDs/body/style/points/true outline
boxes, complete font/palette resources, graph, literal formulas, captions and comments.
`independent-checks.json` records actual20-source/64-box/eight-refusal qualification.

Cases cover middle CH2 creation, reverse3→2→4 versus2→3→4, partial/full states, blue14pt,
one-Apply N/H1 versus later old48 edit, existing endpoint edit, created/hidden/all point
moves, paired-node comments, dense IDs and caption IDs. Existing flag absence/yes/no,
literal H and every unaffected lexeme stay intact. Original formula is C3H9NO; central
C/H2→N/H1 gives literal C2H8N2O and OH→NH2 gives C3H10N2. Those are literal independently
parsed counts, without implicit H or a general chemical-validity claim.

For the dense input, chemical objects occupy1..13 and font14 is a distinct resource.
Global fresh reserved Text14/15/16 makes the first app3 creation use15; later app2 uses14
without colliding with font14. The caption case already uses Text1, so app3 reserves3.
Ordinary sparse source IDs73/97/109/137/149 are distinct from app IDs1..5. Original source
Text21/25 remain untouched. Existing colour0 resolves black, so deterministic lowest
matching black is0 even though explicit palette3 is also black; blue is4. No resource
table is changed or guessed.

The independent font oracle reads only the existing pinned `assets/fonts/Roboto-Regular.ttf`
(SHA25656a45233d29f11b4dfb86d248e921939d115778f87325e7ae8cc108383d6664d), via
FontTools4.61.1 and the original tracked `xtask/oracles/clearance_oracle.py`. It never
imports a product parser/writer/renderer. Actual Expat/Decimal plus independently constructed
and direct unsanitized RDKit2026.09.2 CDXML consumers agree on all20 graphs/formulas/SMILES.
All64 explicit label boxes independently match FontTools to1e-12pt. Strict reader controls
reject object collisions, font/face/body/child/H/metadata/ink failures; full facts detect
changed retained flags even when a changed file could represent another valid diagram.

Reproduction uses installed external test tools, no shipped dependency or new install:

```sh
python3 crates/io/tests/fixtures/created-labels/generate_created.py
python3 crates/io/tests/fixtures/created-labels/check_created.py
```

The first command requires FontTools4.61.1 and writes ignored
`target/label-creation/reproduced/`. The second requires FontTools4.61.1 plus
`BONDLINE_PYTHON` (or `.cache/oracles/format-python/bin/python`) with RDKit2026.09.2,
checks the frozen input directory, and writes ignored reproduced facts/checks.
`--input-dir`, `--output-dir` and `--python` permit explicit route changes. Defaults
never rewrite frozen references. Portable script adaptation changes paths only; mathematical,
lexical and semantic output bytes remain identical. Fonts/assets/old goldens/Actions and
existing48 command metadata are unchanged.

The shared test helper is `xtask/oracles/created_label_support.rs`; it reads these frozen
sources/facts and external reader. Generated labels do not add model atom/bond/caption IDs.
Actual application/native/render acceptance belongs to the receiving tests and records,
not these independent source-data oracles.
