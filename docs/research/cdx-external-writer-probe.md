# Independent external CDX writer probe — 2026-11-15

This records an ignored investigation; its raw artifacts remain under target. It is not a corpus admission, format implementation or compatibility claim. No shipped dependency, Root environment, product source, existing oracle/fixture, corpus manifest or Git state was changed. Only the small standalone parser probe was built; no GUI, MCP, chemistry editor binary or hosted CI was used.

## Producer and original input provenance

- Official producer: [epam.indigo 1.47.0 on PyPI](https://pypi.org/project/epam.indigo/1.47.0/), version-specific metadata in `artifacts/pypi-1.47.0.json`. The downloaded macOS arm64 wheel is 5,844,288 bytes, SHA-256 `e3f097c9818dc282fe0ae436c4b4f98fd310d2ccba4eb13617ad0ef547cc6f1e`.
- Retained wheel METADATA declares Apache-2.0. It contains no separate LICENSE/NOTICE member; the exact released [LICENSE](https://github.com/epam/Indigo/blob/cc2ea85187433c253c0233a13f1859a6a5298d97/LICENSE) and source copyright headers are retained separately. No comprehensive binary dependency audit or reproducible-build attestation is claimed.
- Public annotated tag `indigo-1.47.0` points through tag object `fda5f80a8a2ea7bcf42a7c66c59462f86fff0ead` to source commit `cc2ea85187433c253c0233a13f1859a6a5298d97`. It is unsigned. Runtime reports `1.47.0.0-gcc2ea8518-aarch64-darwin-appleclang-17000013`, consistent with that recorded source prefix.
- `input-graphs.json`, both generators and probe callers are original MIT OR Apache-2.0 test work. The five declared graphs are ethane, ethanol, ethene, methylamine and cyclohexane, each with literal element/order/coordinate expectations. No upstream drawing, sample, template, icon, font asset, help prose or proprietary artifact is an input. The emitted font table contains producer-default font-name metadata, not font files.
- Producer installed only into local `venv/`, created with `--copies`. Installation used the local verified wheel, `--no-index --no-deps --no-cache-dir --require-hashes`. Root's format-python environment was used only as a separate external reader with `-B`. The original PyPI JSON, tag records, wheel, metadata, license, installation log and `pip freeze` are retained.

## Two preserved generation variants

`generate.py` called public createMolecule/addAtom/setXYZ/addBond/saveCdx. Although setXYZ returned the declared positions, the output omitted every atom position. The five original outputs remain untouched in `generated/`; each repeated generation was byte-identical.

The public release explains the missing positions: [indigoSetXYZ](https://github.com/epam/Indigo/blob/cc2ea85187433c253c0233a13f1859a6a5298d97/api/c/indigo/src/indigo_molecule.cpp#L1968) sets the coordinate vector but not `have_xyz`; the [saver](https://github.com/epam/Indigo/blob/cc2ea85187433c253c0233a13f1859a6a5298d97/core/indigo-core/molecule/src/molecule_cdxml_saver.cpp#L743) emits the node position only with that flag. The public [layout operation](https://github.com/epam/Indigo/blob/cc2ea85187433c253c0233a13f1859a6a5298d97/api/c/indigo/src/indigo_layout.cpp#L36) initializes coordinate presence.

`generate_positioned.py` therefore performs layout once, overwrites **every** atom position with the unchanged original coordinate declaration, then asserts each public XYZ readback equals the declared float32 value using stdlib struct conversion. No generated layout coordinates are retained as input. These separate outputs are in `generated-positioned/`; their repeats are also byte-identical. All atom nodes now have raw 0200 position properties.

| Graph | Positioned bytes | Positioned CDX SHA-256 |
| --- | ---: | --- |
| ethane | 258 | `856482269dbd85640b87c11f608104dba72ebd308a686f1333a893965ed560ba` |
| ethanol | 357 | `26d84130f681fb71a071503b884ff6977cdc382b110f5218bd74cba4ed5823da` |
| ethene | 264 | `3c682f35838c6a39a99fbae7ad4c504aab953ec04d081f4b453ea227388901e7` |
| methylamine | 324 | `b17e59cfb3fc428f634d38c89424e2ad506133dd5100d533dcbc1169f2426374` |
| cyclohexane | 458 | `2d3cd1924ff6eebbb4205517643be5f4a7a2409e528aee579e548a1fb99a3810` |

Indigo's input coordinate unit is **not** asserted to be a CDX document point. The producer scales/translates/flips coordinates: ethane input x=1.5 becomes raw x=45pt. Every emitted signed16.16 x/y and independently decoded RDKit conformer is recorded; neither original floating digits nor normative unit conversion is claimed.

## Actual independent inspection and refusals

Root's unchanged pinned RDKit 2026.09.2 CDX API read the original bytes in separate processes, not Indigo's own reader. `inspection-result.json` and `inspection-positioned-result.json` record all five successes per variant against literal elements, undirected edges/orders, H counts, charge/isotope/radical absence, canonical SMILES and formula. This establishes these bounded chemical vectors, not general stereo/geometry correctness or a Bondline chemistry engine.

The independent stdlib scan inventories the complete header, every object/property payload/reference/end offset and all position words, including unmatched terminal words. Each of the ten distinct files has the legacy 28-byte header, a closed implicit document tree and **exactly one additional 0000 word after the first implicit-document end**. Nothing was removed or repaired.

The unchanged old `xtask/oracles/cdx_oracle.py` generic describe path refuses every file with `ValueError: trailing bytes`. Its old `verify1explicit` canary also refuses this different scene as expected; that fixture-specific oracle remains exact. A new caller cannot reuse its old fixture-specific chemistry expectations as a general external-scene checker.

The tiny `probe/` Cargo project depends only on the unchanged `bondline-cdx` path from the 492905a research worktree and std. Its own target and lock stay under this ignored directory. Both runs completed normally, but **all five files per variant were refused with `TrailingData`**. Exact passthrough was consequently not reached. Probe exit 0 means the investigation ran, not that any CDX parsed or round-tripped.

The released [binary writer](https://github.com/epam/Indigo/blob/cc2ea85187433c253c0233a13f1859a6a5298d97/core/indigo-core/molecule/src/molecule_cdxml_saver.cpp#L2015) explains the bytes: writeBinaryElement emits a final zero at line2055, while endDocument writes the historical header/root placeholder, calls that function, then emits another zero at line2069. This is direct producer behavior evidence. It does not alone establish a normative format rule permitting arbitrary trailing zeros.

## Preserved setup correction and limits

The first ignored caller was named `inspect.py` and shadowed Python's standard library during NumPy import. Its source, partial scans/receipt and exit1 log remain in `initial-inspector-shadowing/` and `logs/inspect-pinned-rdkit.log`. Renaming the identical bytes to `cdx_external_probe.py` fixed only this new harness setup. The clean caller exited0; no reader, input, assertion or generated output changed for that correction. The positioned caller changes only its own variant paths.

No source snippet from Indigo was incorporated into Bondline. Its native wheel is an external investigation tool, not a shipped dependency. No claim is made about Windows/Linux generation, broadly identical bytes across releases/platforms, the unrelated RDKit upstream CDX file's unclear authorship, a native UI flow, or general imported CDX model support. Existing corpus pins/coverage and all old guards remain unchanged.

## Small actionable continuation

1. Use the positioned ethane as a provenance-clear **externally generated original** future test fixture, with the complete generator/input/license/package/source recipe and a separate external-graph oracle. Preserve both raw final zeros and all existing fixture-specific oracles/pins.
2. Before modifying parsing, resolve/document the public format evidence for historical implicit-document plus distinct-file-end framing. Add an independent exact-original RED and narrowly bounded framing behavior; retain arbitrary-trailing-data, malformed-root/end/count/limit negatives and exact-source round-trip assertions. Do not trim, scan for a plausible start, or weaken EOF globally.
3. Only after a real source-backed TDD change and review may the data be considered for a separately published/fetched corpus bundle. Publication, license admission, full CDX application mapping and foreign/general compatibility are not accomplished here.

`validation.json`, `artifact-manifest.json` and `final-receipt.json` record exact bytes, unchanged protected inputs, tool identities and terminal outcomes. The venv/build directories remain available but are excluded from the bulk artifact manifest; their executable/native-library identities are recorded separately.
