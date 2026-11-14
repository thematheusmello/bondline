# Original SMILES seeds

These small ASCII notations were independently authored for Bondline on2026-11-14.
They are MIT OR Apache-2.0 under the repository's licenses. No vendor sample, template,
external parser implementation or specification prose is copied. Each file has exactly
one final LF; Git attributes preserve its bytes even with core.autocrlf=true.

| File | Bytes | SHA-256 | Literal graph expectation |
|---|---:|---|---|
| isotopic-chain.smi |17|6c1fa14e6c1720eb3465e09c7450e2e520a5aa1a6438d279cbf528cdc40c7e9e|3atoms C/C/O,2Single bonds,1component; first isotope13; bracket H3/2/1 |
| two-ring-labels.smi |21|1528ac64991e737bfee28357012b3dd218780006b8b30ac1d25b222f46a5facb|7C/7Single bonds/2components: separate three/four-member rings with distinct73/74 labels |
| salt-branch.smi |21|357c9e0907672dba5acb80ed9e5263ca9f286cb889e5a221cd518603622dea1c|6atoms Na/O/4C,4Single bonds,2components; Na+1/O-1, bracket CH2 and three bare carbons |

Notation rules come from the pinned public
[OpenSMILES v1.0 source](https://github.com/opensmiles/OpenSMILES/blob/9240a2887158699503605bc838e1de9a9f7883f4/opensmiles.asciidoc).
These vectors are our own test data, not a copied example corpus or proof of full
conformance. Literal bracket H/isotope/charge and ring-label facts remain separate from
chemical perception and any eventual model normalization policy.

Before parser integration, actual bytes were independently consumed by external
RDKit2026.09.2 using sanitize=false, removeHs=false, parseName=false and allowCXSMILES=false.
Atom/bond/component counts, atomic numbers and isotope/charge/bracket-H values matched
the literal expectations above. No sanitization, property-cache update, canonical output
or coordinate generation was requested. Python exposes no skipCleanup field; its default
cleanup cannot be disabled by that binding. These checks do not establish implicit H,
radicals, chemical validity, layout or general parser conformance.

Fuzzing must COPY these seeds into an ignored mutation directory, never mutate this
committed directory. A bounded sanitizer run is not exhaustive coverage or an independent
chemical oracle for every accepted mutation.


## Aromatic notation seeds

The three additional original dual-licensed files below exercise source markers, not
chemical perception. CI copies all six committed.smi files into ignored mutation storage;
the fetched corpus remains the original three vectors at its unchangedeabe02f pin.

| File | Bytes | SHA-256 | Literal graph expectation |
|---|---:|---|---|
| aromatic-cycle.smi |9|3f1fc605b1087f50395dac8ce2981a09d3019a2f37ec61cdc0df692230570636|6lowercase carbon markers/6Aromatic edges/1component; bare H stays implicit, not inferred|
| explicit-aromatic-bond.smi |4|c96c5a85ca193233591215c2f253c1cee190a97413cba3cc8c70a093b81eee07|2uppercase carbon markers remain nonaromatic;1explicit Aromatic bond; normalization refuses this syntax-only graph|
| explicit-ring-orders.smi |18|3675e07cb329fc0ac1428f8d2f40856280f9a8287d770356e3508a2387717386|Two components: uppercase4-carbon ring with explicit Aromatic closure; lowercase6-carbon ring with explicit Single closure, all other orders literal |

Actual fixture bytes face literal marker/edge/EOF checks and the unchanged pinned
unsanitized RDKit consumer. A colon does not promote uppercase atom markers in the IR.
No canonical chemistry, valence-validity, aromaticity calculation or model conversion is
claimed; every aromatic marker/bond refuses the fixed-H normalizer. Original three source
files/metadata/hashes and all source pins remain unchanged. These new files are authored
regression data, not a copied vendor/sample library or a prospective fetched-source pin.
