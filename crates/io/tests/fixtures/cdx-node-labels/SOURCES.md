# Original CDX Node label witnesses

These 34 original binaries and the independent struct generator are
MIT OR Apache-2.0. The [data contract](../../../../../docs/research/cdx-node-labels.md)
links the public binary definitions. Seven complete models and 27 refusal
controls are pinned in [cases.json](cases.json). The independent reader records
the CDX envelope, direct ownership, property spans and resource facts.
Long payloads are identified by SHA-256. The pre-label CLI refused all 34
originals and supplied the exact diagnostics for the controls.

The matching `connected_pair.cdx` graph also passed RDKit 2026.09.2's
binary CDX reader with two carbons, literal H2 at each, double order and
directed indices 1→0. RDKit reported C2H4. These observations concern
the numeric graph. They do not interpret placement, faces or the
deliberately conflicting label witness.

```sh
out=$(mktemp -d)
python3 -B crates/io/tests/fixtures/cdx-node-labels/generate_labels.py \
  --output-dir "$out/labels" --baseline-cli /path/to/pre-label/bondline-cli
diff -r -x SOURCES.md -x generate_labels.py crates/io/tests/fixtures/cdx-node-labels "$out/labels"
```

Manifest SHA-256 is `f050b95388f50c20af70b033646b14e3828fb21662202cb35aa02d5becbf214d`.
