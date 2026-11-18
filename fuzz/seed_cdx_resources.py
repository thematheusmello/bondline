#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Seed the CDX resource query with independently checked original files."""

import argparse
import hashlib
import importlib.util
import json
import struct
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
RESOURCES = REPO / "crates/cdx/tests/fixtures/resources"
IO_FIXTURES = REPO / "crates/io/tests/fixtures"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    root = parser.parse_args().directory
    root.mkdir(parents=True, exist_ok=False)
    immutable = root / "sources"
    corpus = root / "corpus"
    immutable.mkdir()
    corpus.mkdir()

    spec = importlib.util.spec_from_file_location(
        "resource_seed_oracle", RESOURCES / "resource_oracle.py"
    )
    oracle = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(oracle)
    facts = json.loads((RESOURCES / "resource-facts.json").read_text())
    entries = []

    def add(name, raw, kind):
        name += ".cdx"
        for folder in (immutable, corpus):
            (folder / name).write_bytes(raw)
        entries.append(
            {
                "name": name,
                "kind": kind,
                "bytes": len(raw),
                "sha256": hashlib.sha256(raw).hexdigest(),
            }
        )

    cases = json.loads((RESOURCES / "resource-vectors.json").read_text())["cases"]
    for case in cases:
        name = case["name"]
        raw = (RESOURCES / (name + ".cdx")).read_bytes()
        assert oracle.read(raw) == facts[name]
        assert len(raw) == case["bytes"]
        assert hashlib.sha256(raw).hexdigest() == case["sha256"]
        add(
            "resource_" + name,
            raw,
            "query succeeds" if case["query_valid"] else "query refuses",
        )

    for name, end in [
        ("tables_explicit", -1),
        ("tables_extended", 39),
        ("tables_header", -1),
    ]:
        raw = (RESOURCES / (name + ".cdx")).read_bytes()[:end]
        try:
            oracle.raw_records(raw)
        except (ValueError, struct.error):
            add("truncated_" + name, raw, "framing refuses")
        else:
            raise AssertionError(f"independent reader accepted truncated {name}")

    for folder, manifest, prefix in [
        ("cdx-plain-bonds", "bond-vectors.json", "bond"),
        ("cdx-literal-metadata", "literal-vectors.json", "literal"),
    ]:
        directory = IO_FIXTURES / folder
        for case in json.loads((directory / manifest).read_text())["cases"]:
            name = case["name"]
            raw = (directory / (name + ".cdx")).read_bytes()
            assert len(raw) == case["bytes"]
            assert hashlib.sha256(raw).hexdigest() == case["sha256"]
            add(prefix + "_" + name, raw, "previously frozen " + prefix)

    manifest = (json.dumps({"sources": entries}, indent=2) + "\n").encode()
    (root / "sources.json").write_bytes(manifest)
    print(
        json.dumps(
            {
                "seeds": len(entries),
                "max_bytes": max(entry["bytes"] for entry in entries),
                "manifest_sha256": hashlib.sha256(manifest).hexdigest(),
            }
        )
    )


if __name__ == "__main__":
    main()
