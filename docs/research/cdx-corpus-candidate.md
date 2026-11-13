# Excluded CDX corpus candidate

Reviewed2026-11-13. One bounded public test-data investigation; no application binary was
acquired/run, no file was admitted to the corpus, and no parser assertion was relaxed.

[RDKit structure_4.cdx at the existing pin](https://github.com/rdkit/rdkit/blob/c686c2c945df370c2f95ebe20aae2872a36f7c17/Code/GraphMol/test_data/CDX/structure_4.cdx)
is402bytes, SHA256
`edd85ec0aafb98f011938e2ffc889d8e29b56a8117315db7524ff5c44bd7ccee`,
Git blob72c6f51d8dbfe99a8815fcb280a131117a260b86. The
[original contribution](https://github.com/rdkit/rdkit/commit/34e58b5edd0b633a43d19f11064f0e466dffa319)
was Brian Kelley's2025-08-12 CDX test addition, later squashed through
[PR8681](https://github.com/rdkit/rdkit/pull/8681). Upstream expects CCO.

The repository BSD-3-Clause notice matches the retained corpus notice, SHA256
daeb8d194502cbcf34c05c39541a0d02be65bc9bada5b891c1974cd24e9fca30. However, no candidate
notice/source statement establishing original file authorship was found in the reviewed
path, contribution, PR discussion or pinned Glysade metadata. Embedded writer metadata
mentions CS CDX driver3.0/2008. A contribution author is not necessarily the original drawing
author. **Exclude pending clearer provenance.** This is not proof that the file is unlicensed,
that a producer owns its contents, or that a repository license never covers test data.

Actual unmodified-byte probes establish a separate compatibility limit:

- External RDKit2026.09.2 CDX support accepts one ethanol molecule: three atoms, two bonds,
  CCO/C2H6O, matching upstream's expected structure.
- The freshly rustc1.94.1-compiled current standalone Bondline parser rejects TrailingData.
  Independent struct decoding from offset28 sees seven encoded objects/24 properties,
  reaches root end400, then sees two zero bytes. Starting at22 instead recognizes an
  explicit document record and eight objects, with the same root end/trailing bytes.
- No bytes were trimmed, normalized or modified. The current strict EOF policy and header/
  implicit-root distinction require a public-specification decision before changing them;
  consumer acceptance alone is insufficient.

The parser source SHA256 was4a56c7a7641e078d6b38e72f5940c02106329de3214bd3e283762fbe7c4044e4.
Full exact URLs, history responses and probe outputs remain ignored under
.cache/research/cdx-corpus/report.json and sibling files. No candidate bytes are committed.
Corpus coverage remains two existing pinned files and zero CDX entries. The bounded hunt
stopped at this candidate; the diagnostics do not establish general CDX compatibility.
