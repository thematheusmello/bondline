#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Qualify the independent reader and freeze facts for original raw String sources."""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import sys
from pathlib import Path

sys.dont_write_bytecode = True


def module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec is not None and spec.loader is not None
    value = importlib.util.module_from_spec(spec)
    sys.modules[name] = value
    spec.loader.exec_module(value)
    return value


def json_bytes(value: object) -> bytes:
    return (json.dumps(value, indent=2, ensure_ascii=True) + "\n").encode("ascii")


def must_fail(reader, document, record) -> dict[str, object]:
    try:
        document.string(record)
    except reader.QueryError as error:
        return error.detail
    raise AssertionError("invalid query was accepted")


def qualify_literal_reader(reader) -> dict[str, object]:
    # Complete literal bytes; neither the generator nor any product writer is called.
    literal = (
        b"VjCD0100\x04\x03\x02\x01"
        + bytes(16)
        + bytes.fromhex("0140 0d00 0100 3412 7856 bc9a f100 ffff 41 0000")
    )
    document = reader.Document(literal)
    value = document.string(document.records[0])
    expected = {
        "start": 0x1234,
        "font": 0x5678,
        "face": 0x9ABC,
        "size_twentieths": 241,
        "color": 65535,
        "ordinal": 0,
        "range": [34, 44],
    }
    assert value.count == 1 and value.run(0) == expected
    assert bytes(value.text_bytes()) == b"A" and (value.text_begin, value.end) == (
        44,
        45,
    )
    assert value.text_bytes().obj is literal
    # The property itself is one style-byte short; the following property is ample.
    short = (
        b"VjCD0100\x04\x03\x02\x01"
        + bytes(16)
        + bytes.fromhex("0140 0b00 0100 000000000000000000")
        + bytes.fromhex("0340 2000")
        + bytes(32)
        + b"\0\0"
    )
    document = reader.Document(short)
    assert must_fail(reader, document, document.records[0]) == {
        "kind": "InvalidPayload",
        "record": 0,
    }
    return {
        "literal_little_endian_and_ten_byte_run": True,
        "literal_payload_boundary_one_byte_short": True,
    }


def check(source_dir: Path, facts_dir: Path, verify: bool = False) -> dict[str, object]:
    scripts = Path(__file__).resolve().parent
    generator = module("original_raw_string_generator", scripts / "generate_strings.py")
    reader = module("independent_raw_string_reader", scripts / "string_oracle.py")
    manifest = json.loads((source_dir / "string-vectors.json").read_bytes())
    assert [case["name"] for case in manifest["cases"]] == list(generator.NAMES)
    facts: dict[str, object] = {}
    raw_cases: dict[str, bytes] = {}
    accepted = rejected = 0
    for case in manifest["cases"]:
        name = case["name"]
        raw = (
            (source_dir / (name + ".cdx")).read_bytes()
            if case["storage"] == "file"
            else generator.case_source(name)
        )
        assert (
            len(raw) == case["bytes"]
            and hashlib.sha256(raw).hexdigest() == case["sha256"]
        )
        value = reader.read(raw, query_tag=case["query_tag"])
        assert value["query"]["ok"] == case["query_valid"], name
        document = reader.Document(raw)
        record = document.records[value["query_record"]]
        if case["query_valid"]:
            accepted += 1
            string = document.string(record)
            assert string.text_bytes().obj is raw
            assert string.runs_begin == record.payload_begin + 2
            assert string.text_begin == string.runs_begin + string.count * 10
            assert string.end == record.payload_end
        else:
            rejected += 1
            assert value["query"]["error"] == {
                "kind": "InvalidPayload",
                "record": record.index,
            }
            available = record.payload_end - record.payload_begin
            assert (
                available < 2
                or 2
                + 10
                * int.from_bytes(
                    raw[record.payload_begin : record.payload_begin + 2], "little"
                )
                > available
            )
        raw_cases[name] = raw
        facts[name] = value

    checks = qualify_literal_reader(reader)

    def string(name: str):
        document = reader.Document(raw_cases[name])
        return document.string(document.records[facts[name]["query_record"]])

    assert string("single_size_240").run(0)["size_twentieths"] == 240
    assert string("single_size_241").run(0)["size_twentieths"] == 241
    assert string("literal_zero_fields").run(0) == {
        "start": 0,
        "font": 0,
        "face": 0,
        "size_twentieths": 0,
        "color": 0,
        "ordinal": 0,
        "range": facts["literal_zero_fields"]["query"]["runs"][0]["range"],
    }
    assert all(
        string("full_uint16_fields").run(0)[name] == 65535
        for name in ("start", "font", "face", "size_twentieths", "color")
    )
    assert [string("duplicate_starts").run(i)["start"] for i in range(2)] == [0, 0]
    assert [string("decreasing_starts").run(i)["start"] for i in range(3)] == [8, 5, 0]
    assert [string("out_of_range_starts").run(i)["start"] for i in range(2)] == [
        65535,
        2,
    ]
    assert len(string("single_empty").text_bytes()) == 0
    assert (
        bytes(string("zero_control_bytes").text_bytes())
        == b"\0\r\n\x1b\x7f\x80\xe9\xff"
    )
    assert bytes(string("zero_all_bytes").text_bytes()) == bytes(range(256))
    witness = string("scalar_byte_witness")
    original = bytes(witness.text_bytes())
    decoded_witness = original.decode("utf-8")
    assert (
        len(original),
        len(decoded_witness),
        len(decoded_witness.encode("utf-16-le")) // 2,
    ) == (8, 4, 5)
    assert [witness.run(i)["start"] for i in range(witness.count)] == [
        0,
        1,
        2,
        3,
        4,
        5,
        7,
        8,
        65535,
    ]
    checks["scalar_byte_witness"] = {
        "raw_bytes": 8,
        "scalars_if_utf8": 4,
        "utf16_units_if_utf8": 5,
        "codec_or_offset_interpretation_authorized": False,
    }
    assert string("style_looking_remainder").count == 1
    assert len(string("style_looking_remainder").text_bytes()) == 13
    assert len(string("long_text_65553").text_bytes()) == 65553
    assert string("long_text_65553").count == 2
    checked_runs = 0
    for name in ("max_runs_65535_empty", "max_runs_65535_text"):
        value = string(name)
        assert value.count == 65535
        for ordinal in range(65535):
            run = value.run(ordinal)
            expected = (
                65535 - ordinal,
                (73 * ordinal + 7) & 65535,
                (257 * ordinal + 96) & 65535,
                (29 * ordinal + 240) & 65535,
                (11 * ordinal + 4) & 65535,
            )
            assert (
                tuple(
                    run[key]
                    for key in ("start", "font", "face", "size_twentieths", "color")
                )
                == expected
            )
            assert run["range"] == [
                value.runs_begin + ordinal * 10,
                value.runs_begin + (ordinal + 1) * 10,
            ]
            checked_runs += 1
    assert len(string("max_runs_65535_empty").text_bytes()) == 0
    assert bytes(string("max_runs_65535_text").text_bytes()) == b"\0\xff\x80A"

    # Plenty of following file bytes must not rescue an undersized property prefix.
    raw = raw_cases["invalid_max_count_following_padding"]
    doc = reader.Document(raw)
    record = doc.records[facts["invalid_max_count_following_padding"]["query_record"]]
    assert len(raw) - record.payload_begin >= 2 + 65535 * 10
    assert record.payload_end - record.payload_begin == 11
    assert must_fail(reader, doc, record) == {
        "kind": "InvalidPayload",
        "record": record.index,
    }

    raw = raw_cases["plain_explicit_file_end"]
    doc, foreign = reader.Document(raw), reader.Document(bytes(bytearray(raw)))
    target = facts["plain_explicit_file_end"]["query_record"]
    controls = {}
    for kind in ("object", "end", "file_end"):
        selected = next(record for record in doc.records if record.kind == kind)
        error = must_fail(reader, doc, selected)
        assert error == {"kind": "Source", "error": "InvalidProperty"}
        controls[kind] = {"record": selected.index, "error": error}
    error = must_fail(reader, doc, foreign.records[target])
    assert error == {"kind": "Source", "error": "InvalidProperty"}
    controls["foreign_same_bytes"] = {"record": target, "error": error}

    checks.update(
        sources=len(manifest["cases"]),
        stored_sources=sum(case["storage"] == "file" for case in manifest["cases"]),
        runtime_sources=sum(case["storage"] == "runtime" for case in manifest["cases"]),
        query_accepted=accepted,
        query_invalid_prefix=rejected,
        all_raw_syntax_valid=True,
        maximum_run_entries_individually_checked=checked_runs,
        property_local_max_count_refusal=True,
        zero_duplicate_decreasing_and_out_of_range_starts_preserved=True,
        no_product_code_or_external_consumer_used=True,
    )
    facts_dir.mkdir(parents=True, exist_ok=True)
    for name, value in (
        ("string-facts.json", facts),
        (
            "handle-controls.json",
            {"source": "plain_explicit_file_end", "controls": controls},
        ),
        ("independent-checks.json", checks),
    ):
        destination = facts_dir / name
        content = json_bytes(value)
        if verify:
            assert destination.read_bytes() == content, name
        else:
            destination.write_bytes(content)
    return checks


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--source-dir", type=Path, default=Path(__file__).with_name("data")
    )
    parser.add_argument("--facts-dir", type=Path)
    parser.add_argument("--verify", action="store_true")
    args = parser.parse_args()
    print(
        json.dumps(
            check(args.source_dir, args.facts_dir or args.source_dir, args.verify)
        )
    )


if __name__ == "__main__":
    main()
