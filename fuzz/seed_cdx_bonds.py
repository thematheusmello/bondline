#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Freeze plain-bond CDX parser seeds outside the writable fuzz corpus."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import struct

REPO = Path(__file__).resolve().parent.parent
FIXTURES = REPO / "crates/io/tests/fixtures"
BONDS = FIXTURES / "cdx-plain-bonds"


def module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    root = parser.parse_args().directory
    root.mkdir(parents=True, exist_ok=False)
    immutable = root / "sources"
    corpus = root / "corpus"
    immutable.mkdir()
    corpus.mkdir()
    oracle = module("bond_seed_oracle", BONDS / "bond_oracle.py")
    negative = module("bond_seed_negatives", BONDS / "negative_bonds.py")
    seeds = {}
    sources = {}
    for directory, manifest, prefix in [
        (BONDS, "bond-vectors.json", "bond"),
        (FIXTURES / "cdx-literal-metadata", "literal-vectors.json", "literal"),
    ]:
        for case in json.loads((directory / manifest).read_text())["cases"]:
            raw = (directory / (case["name"] + ".cdx")).read_bytes()
            assert len(raw) == case["bytes"]
            assert hashlib.sha256(raw).hexdigest() == case["sha256"]
            seeds[prefix + "_" + case["name"]] = (raw, "frozen " + prefix)
            if prefix == "bond":
                sources[case["name"]] = raw
    facts = json.loads((BONDS / "bond-facts.json").read_text())
    for name, raw in sources.items():
        assert oracle.read(raw) == facts[name]
    negatives = negative.negative_sources(sources, facts)
    for name, (raw, expected) in negatives.items():
        try:
            oracle.read(raw)
        except (ValueError, struct.error, KeyError):
            pass
        else:
            raise AssertionError("independent reader admitted " + name)
        seeds["refuse_" + name] = (raw, expected)
    base = sources["manual_right"]
    at = next(r["end"] for r in facts["manual_right"]["records"] if r.get("tag") == 0x8000)
    padding = 131072 - len(base) - 8
    largest = base[:at] + struct.pack("<HHI", 0x400F, 65535, padding) + bytes(padding) + base[at:]
    assert len(largest) == 131072
    assert oracle.read(largest)["model"] == facts["manual_right"]["model"]
    seeds["max_input"] = (largest, "original opaque padding; same manual-right model")
    entries = []
    for name, (raw, kind) in seeds.items():
        filename = name + ".cdx"
        (immutable / filename).write_bytes(raw)
        (corpus / filename).write_bytes(raw)
        entries.append({"name": filename, "kind": kind, "bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest()})
    manifest = {"sources": entries, "bond_reader_positive": len(sources) + 1, "bond_reader_negative": len(negatives), "legacy_literal": 18}
    raw = (json.dumps(manifest, indent=2) + "\n").encode()
    (root / "sources.json").write_bytes(raw)
    print(json.dumps({"seeds": len(entries), "manifest_sha256": hashlib.sha256(raw).hexdigest()}))


if __name__ == "__main__":
    main()
