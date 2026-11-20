# Mixed fragment and caption CDX witnesses

The 29 original, dual-licensed MIT OR Apache-2.0 files exercise one Poster Page that contains both Fragment objects and direct Text. The [Page](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Page.htm) admits both children. [Fragment](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Fragment.htm), [Node](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Node.htm) and [Text](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Text.htm) distinguish an atom's owned label from a free-standing caption. [Binary object identity](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDX.htm) scopes nonzero raw IDs to their enclosing object.

The candidate rendering profile combines already qualified chemical objects and direct captions. Each caption has an explicit positive signed depth and every chemical Node, Bond and child Text uses its original absent depth. Building the chemical drawing first and painting the captions in distinct increasing depth then agrees with the documented rules even when caption ink covers chemistry. A Fragment's storage position supplies no paint depth. Other interleavings and equal-depth overlaps need more evidence. [ZOrder](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/ZOrder.htm).

The generator uses the six original [label models and physical ink positions](../cdx-label-preview/SOURCES.md), the independently authored [caption encoding](../cdx-caption/SOURCES.md) and the [binary record oracle](../../../../cdx/tests/fixtures/resources/resource_oracle.py). The 12 receiving positives cover all five framings, before/after storage order, boxed and connected structures, reordered input, multiple captions, a raw ID reused across different containers, anonymous ID zero and a persistent marker carrying live Text ID 19. The two explicitly centered double-bond modes have different input bytes. The 17 controls exercise unresolved order, identity and ownership conflicts, opaque peers, malformed typography/geometry and incomplete chemical styles.

For the overlap pair the first atom's authored red NH2 supplies a 24 pt Roboto H at (155.947265625, 131.65625) pt. A blue caption H at the same baseline shifts three points to the right. FontTools 4.61.1 reads the pinned Roboto Regular H outline. Its horizontal and vertical edges let the oracle certify complete interior and exterior pixel cells with a 1/32-pixel guard. The frozen samples identify overlap, caption-only ink and label-only ink at both 100% and 200%. Use the square viewport size, centered zoom and exact RGB values in each optical_probes entry. Whole-frame CPU/GPU agreement on the receiving mixed documents still needs integration testing.

On 2026-11-20 the pre-mixed Bondline CLI refused every candidate with unsupported_content. The independent model and ink observations in cases.json come from the source geometry and intended ownership. The separately captured legacy-open.json records the actual prior code and record index for each file. Unchanged CDX Save must reproduce each admitted input byte-for-byte.

From the repository root with Python and FontTools 4.61.1 available, run the byte, model and pixel oracle.

```sh
python3 -B crates/io/tests/fixtures/cdx-mixed-caption/generate.py \
  --check crates/io/tests/fixtures/cdx-mixed-caption
```

To regenerate both fixtures and historical CLI refusals, use --output-dir with a new directory and --baseline-cli with a preserved pre-mixed CLI. cases.json SHA-256 is eed57fe172f9238a0c295c8687e072c37e9bef630d5e2cc5132efe7d840dc2fd. legacy-open.json SHA-256 is 9d89c4c5d1e75ba36f6a52446e8ffb7623ed722c2b4de0f326f159d9b10575db.
