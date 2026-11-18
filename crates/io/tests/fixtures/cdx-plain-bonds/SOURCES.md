# Original plain CDX bond vectors

Original Bondline test data, MIT OR Apache-2.0, frozen on base `baa3d90` against
[the locked plain-bond contract](../../../../../docs/research/cdx-plain-bonds.md) `6c9fff0`.
Every byte is independently authored with Python `struct`; expected files/models never
come from a Bondline parser, writer, executable or renderer. No reference executable,
export, artwork, font, downloaded binary or proprietary sample is inspected or copied.

The public SDK [Order](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Bond_Order.htm),
[Display](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Bond_Display.htm),
[Display2](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Bond_Display2.htm),
[DoublePosition](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Bond_DoublePosition.htm),
[Begin](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Bond_Begin.htm),
[End](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Bond_End.htm) and
[IntroCDX](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDX.htm) supply the tagged,
little-endian, owner-specific format facts. The source register/contract pin the fresh
public evidence. Referenced documentation is not bundled/relicensed as an asset.

Order0600 is a bitmask:1/2/4 mean Single/Double/Triple;3 is a query combination. Missing
Order means Single. The other modeled order kinds and query/unspecified values are outside
this original bounded projection, not universally malformed files. Display0601 and
Display2 0602 are signed16; only absent/explicit Solid0 is admitted here. Display2 absence
inherits Display. The model has no non-Solid style field, and no source-backed display
permission is inferred from these plain values.

DoublePosition0603 is signed16. Values0/1/2 are automatic Center/Right/Left;256/257/258
are the manual variants. Absence stays Unspecified, distinct from explicit Center0.
Side is relative to authored Begin-to-End. Equal modeled automatic/manual pairs retain
different raw source values and complete source identities. The manual bit is preserved
only in the immutable CDX source/facts, without inventing a model field or re-encoding it.
Explicit Display2 or position on non-Double bonds, INCLUDING0, is refused by the contract's
original support boundary. This does not claim such dormant format data is always illegal.

`bond-vectors.json` pins16 complete original files/lengths/SHA256 hashes. Source IDs are
Page90/Fragment51/Node73+149/Bond900, distinct from caller A4 page1/fragment1/atoms1+2/bond1.
The bond deliberately precedes the second Node in source order. Fixed-point points are
(96.25,160.5)/(144.25,160.5), stored y before x. Original root/page/fragment/atom/bond/custom
opaque properties include null/high bytes; custom IDs400000001/2 remain distinct. Full
framing records preserve owner indices, offsets, tag order, payload hex and length encoding.
No opaque data supplies chemistry, style or layout defaults.

The source names cover absent/explicit Single-Solid equal models, valid etheneH2/ethyneH1,
all six explicit positions plus absence, reversed Begin/End, reversed property order,
small extended payload framing, explicit/implied/embedded envelopes and absent inline H.
The extended marker around a small payload exercises the existing tolerant framing,
without changing the two-byte numeric payload. The existing embedded22-byte envelope's
prior public/open-source evidence remains in `docs/research/cdx-file-end-plan.md` and
`xtask/oracles/cdx_header_oracle.py`; it is not claimed as the literal SDK28-byte header.

`bond-facts.json` comes from the independent standard-library `bond_oracle.py`. It records
all source atoms/bonds/fields and a COMPLETE expected A4 model, assembled as JSON data
without model constructors or product imports. Expected Atom fields retain absent HNone,
isotopeNone, charge0 and radicalNone. Bond facts additionally preserve source endpoints,
app endpoints, raw presence/values, effective plain displays, side and the raw manual flag.
The model omits Unspecified's optional serialized field, keeps explicit Center and retains
reversed source149-to73 as app2-to1 without converting Right into Left. No caption, drawing
settings or style field is fabricated. Different source bytes remain different owners
although automatic/manual or absent/explicit defaults may have equal models.

`negative_bonds.py` exposes the original fixed `negative_sources(sources,facts)` controls
as a standard-library function returning name-to-(bytes,contract-error) pairs. The149
checks cover every admitted field on root/Document/Page/Group/Fragment/Node/custom wrong
owners, unsupported order bit/combinations, every non-Solid style1..14 plus reserved/
negative controls, unknown positions, unsupported widths, equal/different duplicates,
all six positions and Display2=0 on Single/Triple, and short/extended truncations.
Expected product error names are independently declared by the contract: old Order width
InvalidProperty; new field width/enum/non-Double UnsupportedProperty; duplicate known
fields DuplicateProperty; unsupported order UnsupportedBondOrder; truncation syntax errors.
The reader itself throws standard Python errors and does not simulate product diagnostics.

Actual installed external RDKit2026.09.2 reports binary-CDX support and qualifies all16
original sources for graph/order/direction across the three envelopes. Known H2/H1 agrees,
ethene yields C2H4/C=C and ethyne C2H2/C#C; reversed source endpoints remain reversed
consumer bond indices. For missing H, RDKit's own inferred formula is recorded only as an
external graph observation: expected authoring H remains None. Raw-coordinate/y-reversal
behavior is explicitly recorded rather than mistaken for point-space layout agreement.
No side/automatic-manual state, rendering, stereo or spin inference follows from these
ordinary molecule observations. Source bytes/independent struct/public enums establish
that data only. `rdkit-observed.json` and `independent-checks.json` freeze the exact results.

Reproduce with existing tools and no product build or network:

```sh
python3 crates/io/tests/fixtures/cdx-plain-bonds/generate_bonds.py
.cache/oracles/format-python/bin/python crates/io/tests/fixtures/cdx-plain-bonds/check_bonds.py
```

The generator uses standard Python and writes ignored `target/cdx-bonds/reproduced/`.
The checker requires the existing pinned external RDKit interpreter, reads frozen inputs
and writes ignored `target/cdx-bonds/reproduced-checks/`. `--input-dir`/`--output-dir` allow
explicit routes; defaults never overwrite references. Sources, manifest, complete facts,
observations and checks reproduce byte-for-byte. Reader `read(source)` and fixed negative
controls can be loaded by standard-library `importlib` for receiving/fuzz tests.

`xtask/oracles/cdx_bond_support.rs` exposes source/facts/model/read. It includes the prior
immutable49-command receipt from `cdx-literal-metadata/old49.json`, SHA256
`3cfd51a6e72891137cf0cc56fd756ff33652cfc685e5aaf877634b259cc41e4f`, without creating or
recapturing another snapshot. The old18 literal sources/helpers and their pins are unchanged.
This original data is not implementation, native acceptance, display/chemical-authoring
permission, golden approval or completion of CDX/M1/M0–M5. Actions remains OFF.
