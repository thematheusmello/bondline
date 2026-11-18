#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Copy the pinned original CDXString sources into a fresh fuzz corpus."""

import argparse
import hashlib
import json
import runpy
from pathlib import Path

FIXTURES = (
    Path(__file__).resolve().parent.parent / "crates/cdx/tests/fixtures/raw-strings"
)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    root = parser.parse_args().directory
    root.mkdir(parents=True, exist_ok=False)
    sources = root / "sources"
    corpus = root / "corpus"
    sources.mkdir()
    corpus.mkdir()

    generate = runpy.run_path(str(FIXTURES / "generate_strings.py"))["case_source"]
    data_dir = FIXTURES / "data"
    manifest = (data_dir / "string-vectors.json").read_bytes()
    cases = json.loads(manifest)["cases"]
    for case in cases:
        name = case["name"]
        filename = name + ".cdx"
        data = generate(name)
        assert len(data) == case["bytes"]
        assert hashlib.sha256(data).hexdigest() == case["sha256"]
        if case["storage"] == "file":
            assert data == (data_dir / filename).read_bytes()
        for directory in (sources, corpus):
            (directory / filename).write_bytes(data)
    (root / "sources.json").write_bytes(manifest)
    print(
        json.dumps(
            {
                "seeds": len(cases),
                "max_bytes": max(case["bytes"] for case in cases),
                "manifest_sha256": hashlib.sha256(manifest).hexdigest(),
            }
        )
    )


if __name__ == "__main__":
    main()
