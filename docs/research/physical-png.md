# Physical PNG page output

The output represents the selected chemical drawing page in authored points. One inch is 72 points and 25.4 millimetres. At a requested positive resolution in pixels per inch, each axis has `ceil(points × dpi / 72)` pixels. The origin is the top left. Document coordinates scale by the same factor on both axes. The PNG pHYs chunk records the rounded integral pixels per metre in both directions. [W3C absolute lengths](https://www.w3.org/TR/css-values-4/#absolute-lengths) · [W3C PNG physical pixel dimensions](https://www.w3.org/TR/png-3/#11pHYs).

The exporter builds the source-qualified chemical page, preserving the order of its ink, and rasterizes it with the existing CPU geometry. Paper can be opaque white or transparent. The output excludes the desktop viewport, border, shadow, selection and controls. A 300 dpi US Letter page is 2550 × 3300 pixels. The CPU page limit is 36 million pixels with at most 16,384 on either axis. The sum of clipped primitive bounding-box areas is limited to 160 million pixel visits. Existing preview capture keeps its 2048-pixel per-axis limit. Publication honours the drawing renderer’s existing typography and source-coverage checks.

The external test reads chunks and verifies their CRCs independently, then decodes the pixels using Pillow. It checks two fixed bond midpoints, empty paper, exact geometry-derived dimensions, alpha and physical resolution at 144 and 300 dpi. Additional checks cover Letter paper, excessive size, unknown source data, missing pages and cancellation.

The shared native `file.export_png` command requires the expected revision, page ID, DPI, transparency choice and destination. It captures an immutable drawing for the worker and uses the existing atomic create-only file publisher. A completed export returns the captured revision, page and format settings with the actual output path, length and directory-sync result. The editable document, its file association and history keep their previous values. The CLI test decodes the file with the independent PNG oracle, checks a Unicode pathname and verifies that an existing destination survives a second export.

```sh
cargo run -p bondline-cli -- run \
  --cmd '{"command":"document.open_cdx","params":{"path":"crates/io/tests/fixtures/cdx-skeletal/propane.cdx"}}' \
  --cmd '{"command":"file.export_png","params":{"expected_revision":1,"page_id":1,"dpi":300,"transparent":true,"output_path":"propane.png"}}'
```
