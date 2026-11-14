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
