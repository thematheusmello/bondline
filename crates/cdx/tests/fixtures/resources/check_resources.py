# SPDX-License-Identifier: MIT OR Apache-2.0
"""Independent full-entry qualification for original CDX resources; stdlib only."""

from pathlib import Path
import argparse, hashlib, importlib.util, json, struct

SCRIPT = Path(__file__).resolve().parent
REPO = next(
    p
    for p in SCRIPT.parents
    if (p / "Cargo.toml").is_file() and (p / "crates/cdx").is_dir()
)
p = argparse.ArgumentParser(description=__doc__)
p.add_argument("--input-dir", type=Path, default=SCRIPT)
p.add_argument(
    "--output-dir", type=Path, default=REPO / "target/cdx-resources/reproduced-checks"
)
args = p.parse_args()
ROOT = args.input_dir
OUT = args.output_dir
OUT.mkdir(parents=True, exist_ok=True)
s = importlib.util.spec_from_file_location(
    "original_resource_oracle", SCRIPT / "resource_oracle.py"
)
oracle = importlib.util.module_from_spec(s)
s.loader.exec_module(oracle)
manifest = json.loads((ROOT / "resource-vectors.json").read_text())
facts = {}
sources = {}
for case in manifest["cases"]:
    raw = (ROOT / (case["name"] + ".cdx")).read_bytes()
    assert (
        len(raw) == case["bytes"] and hashlib.sha256(raw).hexdigest() == case["sha256"]
    )
    row = oracle.read(raw)
    assert row["query"]["ok"] == case["query_valid"], case["name"]
    facts[case["name"]] = row
    sources[case["name"]] = raw
    at = row["header_range"][1]
    for record in row["records"]:
        assert record["range"][0] == at
        at = record["range"][1]
    assert (
        at == len(raw)
        and raw[: row["header_range"][1]]
        + b"".join(raw[slice(*r["range"])] for r in row["records"])
        == raw
    )
frames = {
    "tables_implicit": "ImplicitDocument",
    "tables_explicit": "ExplicitDocument",
    "tables_header": "HeaderDocument",
    "tables_implicit_file_end": "ImplicitDocumentFileEnd",
    "tables_explicit_file_end": "ExplicitDocumentFileEnd",
}


def q(name):
    return facts[name]["query"]


def color(name, index):
    return next(x for x in q(name)["color_lookups"] if x["index"] == index)


def font(name, index):
    return next(x for x in q(name)["font_lookups"] if x["id"] == index)


for name, framing in frames.items():
    assert facts[name]["framing"] == framing
    assert [f["id"] for f in q(name)["fonts"]] == [0, 7, 65535] and [
        f["charset"] for f in q(name)["fonts"]
    ] == [0, 32768, 65535]
    assert [c["rgb"] for c in q(name)["colors"]] == [
        [12345, 23456, 34567],
        [40000, 50000, 60000],
        [0, 32768, 65535],
    ]
    assert (
        font(name, 0)["font"]["name"]["hex"] == ""
        and bytes.fromhex(font(name, 7)["font"]["name"]["hex"])
        == b"Bondline plain\0\xff"
    )
    assert (
        bytes.fromhex(font(name, 65535)["font"]["name"]["hex"])
        == b"\x80\xfeBondline wide"
    )
    for f in q(name)["fonts"]:
        raw = sources[name]
        start, end = f["range"]
        identity, charset, size = struct.unpack_from("<HHH", raw, start)
        assert (identity, charset, size) == (
            f["id"],
            f["charset"],
            f["name"]["bytes"],
        ) and [start + 6, end] == f["name_range"]
        assert (
            hashlib.sha256(raw[slice(*f["name_range"])]).hexdigest()
            == f["name"]["sha256"]
        )
    assert font(name, 42)["error"] == {"kind": "MissingFont", "id": 42}
for name in [c["name"] for c in manifest["cases"] if c["query_valid"]]:
    assert color(name, 0)["rgb"] == [0, 0, 0] and color(name, 1)["rgb"] == [
        65535,
        65535,
        65535,
    ]
    assert color(name, 4294967295)["error"] == {
        "kind": "MissingColor",
        "index": 4294967295,
    }
assert q("absent")["font_table"] is q("absent")["color_table"] is None
assert color("absent", 2)["rgb"] == [65535, 65535, 65535] and color("absent", 3)[
    "rgb"
] == [0, 0, 0]
assert q("user_tags_only") == q("absent")
assert (
    q("empty_tables")["font_platform"] == 1
    and q("font_empty_mac")["font_platform"] == 0
)
assert (
    q("empty_tables")["font_table"] is not None
    and q("empty_tables")["color_table"] is not None
)
for name in ["empty_tables", "palette_empty"]:
    assert color(name, 2)["error"] == {"kind": "MissingColor", "index": 2} and color(
        name, 3
    )["error"] == {"kind": "MissingColor", "index": 3}
assert (
    color("font_empty_mac", 2)["origin"] == "absent_default"
    and color("font_empty_mac", 3)["origin"] == "absent_default"
)
assert (
    color("palette_one", 2)["rgb"] == [4369, 32768, 65535]
    and color("palette_one", 3)["error"]["kind"] == "MissingColor"
)
assert color("palette_nondefault", 2)["rgb"] == [65535, 0, 32768] and color(
    "palette_nondefault", 3
)["rgb"] == [32768, 65535, 1]
assert [f["id"] for f in q("tables_reordered")["fonts"]] == [65535, 0, 7] and [
    f["ordinal"] for f in q("tables_reordered")["fonts"]
] == [0, 1, 2]
assert [
    r["tag"]
    for r in facts["tables_reordered"]["records"]
    if r.get("tag") in (0x0100, 0x0300)
] == [0x0300, 0x0100]
assert all(
    r["encoding"] == "extended"
    for r in facts["tables_extended"]["records"]
    if r.get("tag") in (0x0100, 0x0300)
)
maximum_name = font("max_name", 7)["font"]
raw = sources["max_name"]
name = raw[slice(*maximum_name["name_range"])]
assert (
    q("max_name")["font_platform"] == 0
    and len(name) == 65535
    and name == bytes((i * 53 + 29) & 255 for i in range(65535))
)
assert (
    struct.unpack_from("<H", raw, maximum_name["range"][0] + 4)[0] == 65535
    and maximum_name["name"]["sha256"] == hashlib.sha256(name).hexdigest()
)
assert (
    q("max_fonts")["font_count"] == 65535
    and q("max_fonts")["fonts_compact"]
    and len(q("max_fonts")["fonts"]) == 5
)
raw = sources["max_fonts"]
start, end = q("max_fonts")["font_entries_range"]
assert end - start == 65535 * 6
ids = [0, 65535] + [i for i in range(1, 65535) if i != 32768]
for ordinal, identity in enumerate(ids):
    assert struct.unpack_from("<HHH", raw, start + 6 * ordinal) == (
        identity,
        (0, 32768, 65535)[ordinal % 3],
        0,
    )
assert font("max_fonts", 32768)["error"] == {"kind": "MissingFont", "id": 32768}
assert (
    q("max_colors")["color_count"] == 65535
    and q("max_colors")["colors_compact"]
    and len(q("max_colors")["colors"]) == 5
)
raw = sources["max_colors"]
start, end = q("max_colors")["color_entries_range"]
assert end - start == 65535 * 6
for ordinal in range(65535):
    assert struct.unpack_from("<HHH", raw, start + 6 * ordinal) == (
        (ordinal * 73 + 11) & 65535,
        (ordinal * 257 + 32768) & 65535,
        65535 - ordinal,
    )
assert color("max_colors", 65536)["rgb"] == [65401, 32254, 1] and color(
    "max_colors", 65537
)["error"] == {"kind": "MissingColor", "index": 65537}
expected = {}
for name in [
    "invalid_font_header",
    "invalid_font_count",
    "invalid_font_name",
    "invalid_font_trailing",
    "invalid_font_max_count",
]:
    expected[name] = {"kind": "InvalidTable", "tag": 0x0100, "record": 1}
for name in [
    "invalid_color_header",
    "invalid_color_count",
    "invalid_color_trailing",
    "invalid_color_max_count",
]:
    expected[name] = {"kind": "InvalidTable", "tag": 0x0300, "record": 1}
for value in [2, 65535]:
    expected["unsupported_platform_" + str(value)] = {
        "kind": "UnsupportedPlatform",
        "record": 1,
        "value": value,
    }
expected.update(
    {
        "duplicate_font_id": {
            "kind": "DuplicateFontId",
            "record": 1,
            "id": 7,
            "ordinal": 1,
        },
        "duplicate_font_late": {
            "kind": "DuplicateFontId",
            "record": 1,
            "id": 0,
            "ordinal": 65534,
        },
        "duplicate_font_table": {"kind": "DuplicateTable", "tag": 0x0100, "record": 2},
        "duplicate_color_table": {"kind": "DuplicateTable", "tag": 0x0300, "record": 2},
        "duplicate_before_invalid_font": {
            "kind": "DuplicateTable",
            "tag": 0x0100,
            "record": 2,
        },
        "owner_before_invalid_font": {
            "kind": "UnsupportedTable",
            "tag": 0x0300,
            "record": 4,
        },
    }
)
for name, row in facts.items():
    if not name.startswith("owner_") or name == "owner_before_invalid_font":
        continue
    record_index = next(
        i
        for i, r in enumerate(row["records"])
        if r["kind"] == "property" and r["tag"] in (0x0100, 0x0300)
    )
    tag = row["records"][record_index]["tag"]
    expected[name] = {"kind": "UnsupportedTable", "tag": tag, "record": record_index}
assert len(expected) == 31
for name, error in expected.items():
    assert q(name) == {"ok": False, "error": error}, name
# Outer framing truncation is separately syntax-invalid, never a query-only error.
for raw in [
    sources["tables_explicit"][:-1],
    sources["tables_extended"][:39],
    sources["tables_header"][:-1],
]:
    try:
        oracle.read(raw)
    except (ValueError, struct.error):
        pass
    else:
        raise AssertionError("raw truncation accepted")
(OUT / "resource-facts.json").write_text(json.dumps(facts, indent=2) + "\n")
checks = {
    "status": "passed",
    "sources": len(facts),
    "positive_queries": 17,
    "query_errors": 31,
    "raw_syntax_valid": len(facts),
    "framings": frames,
    "full_fonts_checked": 65535,
    "full_colors_checked": 65535,
    "max_name_bytes_checked": 65535,
    "last_color_index": 65536,
    "last_color_rgb": [65401, 32254, 1],
    "query_error_expectations": expected,
    "raw_truncation_controls": 3,
    "no_font_codec_or_style_resolution": True,
    "no_external_rdkit_claim": True,
    "product_execution": False,
}
(OUT / "independent-checks.json").write_text(json.dumps(checks, indent=2) + "\n")
print(
    json.dumps(
        {
            "status": "passed",
            "sources": len(facts),
            "positive_queries": 17,
            "query_errors": 31,
            "facts_sha256": hashlib.sha256(
                (OUT / "resource-facts.json").read_bytes()
            ).hexdigest(),
        }
    )
)
