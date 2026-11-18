#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Original raw CDXString source generator. Stdlib only; no reader/product imports."""

from __future__ import annotations

import argparse
import hashlib
import json
import struct
import sys
from pathlib import Path

PREFIX = b"VjCD0100\x04\x03\x02\x01"
HEADER = PREFIX + bytes(16)
VALUE_TAG = 0x4001
FRAMES = {
    "implicit": "ImplicitDocument",
    "explicit": "ExplicitDocument",
    "header": "HeaderDocument",
    "implicit_file_end": "ImplicitDocumentFileEnd",
    "explicit_file_end": "ExplicitDocumentFileEnd",
}
RUNTIME = {
    "long_text_65553",
    "max_runs_65535_empty",
    "max_runs_65535_text",
    "invalid_max_count_following_padding",
}


def u16(value: int) -> bytes:
    return struct.pack("<H", value)


def prop(tag: int, payload: bytes, extended: bool = False) -> bytes:
    length = (
        b"\xff\xff" + struct.pack("<I", len(payload))
        if extended or len(payload) >= 65535
        else u16(len(payload))
    )
    return u16(tag) + length + payload


def obj(tag: int, identity: int, body: bytes) -> bytes:
    return struct.pack("<HI", tag, identity) + body + b"\0\0"


def payload(runs: list[tuple[int, int, int, int, int]], text: bytes) -> bytes:
    return u16(len(runs)) + b"".join(struct.pack("<5H", *run) for run in runs) + text


def wrap(
    value: bytes,
    frame: str = "implicit",
    *,
    extended: bool = False,
    tag: int = VALUE_TAG,
    nested: bool = False,
    following_padding: int = 0,
) -> bytes:
    item = prop(tag, value, extended)
    if nested:
        item = obj(0xC123, 149, prop(0x4007, b"nested-origin") + item)
    body = (
        prop(0x4002, b"Bondline original raw string\0")
        + item
        + prop(0x4003, b"following-property-boundary" + bytes(following_padding))
    )
    if frame == "implicit":
        return HEADER + body + b"\0\0"
    if frame == "explicit":
        return HEADER + obj(0x8000, 73, body)
    if frame == "header":
        return PREFIX + bytes(10) + obj(0x8000, 73, body) + b"\0\0"
    if frame == "implicit_file_end":
        return HEADER + body + b"\0\0\0\0"
    if frame == "explicit_file_end":
        return HEADER + obj(0x8000, 73, body) + b"\0\0"
    raise ValueError(frame)


def named_small_payloads() -> dict[str, bytes]:
    return {
        "zero_empty": payload([], b""),
        "zero_control_bytes": payload([], b"\0\r\n\x1b\x7f\x80\xe9\xff"),
        "zero_all_bytes": payload([], bytes(range(256))),
        "single_empty": payload([(65535, 42, 0, 240, 4)], b""),
        "single_size_240": payload([(0, 42, 0, 240, 4)], b"Bondline"),
        "single_size_241": payload([(0, 42, 0, 241, 4)], b"Bondline"),
        "literal_zero_fields": payload([(0, 0, 0, 0, 0)], b"\xff"),
        "full_uint16_fields": payload(
            [(65535, 65535, 65535, 65535, 65535)], b"\0\xff\r\n"
        ),
        "duplicate_starts": payload([(0, 7, 0, 240, 4), (0, 8, 65535, 241, 5)], b"AB"),
        "decreasing_starts": payload(
            [(8, 1, 0x60, 240, 2), (5, 2, 0, 241, 3), (0, 3, 0xFFFF, 0, 65535)], b"AB"
        ),
        "out_of_range_starts": payload(
            [(65535, 0, 0, 0, 0), (2, 65535, 65535, 65535, 65535)], b"X"
        ),
        "scalar_byte_witness": payload(
            [
                (start, 42, 0, 240 if i % 2 == 0 else 241, 4)
                for i, start in enumerate([0, 1, 2, 3, 4, 5, 7, 8, 65535])
            ],
            b"A\xc3\x85\xf0\x9f\x98\x80Z",
        ),
        "style_looking_remainder": payload(
            [(0, 7, 1, 240, 4)], struct.pack("<5H", 65535, 3, 5, 241, 2) + b"raw"
        ),
        "invalid_empty_payload": b"",
        "invalid_one_byte": b"\0",
        "invalid_one_missing": b"\x01\0",
        "invalid_one_nine": b"\x01\0" + bytes(9),
        "invalid_three_29": b"\x03\0" + bytes(29),
    }


SMALL_NAMES = (
    tuple(named_small_payloads())
    + tuple("plain_" + name for name in FRAMES)
    + (
        "extended_small",
        "opaque_tag_0700",
        "opaque_tag_0805",
        "nested_user_object",
        "invalid_extended_one_nine",
    )
)
NAMES = SMALL_NAMES + tuple(sorted(RUNTIME))


def case_source(name: str) -> bytes:
    small = named_small_payloads()
    base = payload([(0, 7, 0, 240, 4), (2, 65535, 0x60, 241, 3)], b"Raw\0\xff\r\n")
    if name in small:
        return wrap(small[name])
    if name.startswith("plain_") and name[6:] in FRAMES:
        return wrap(base, name[6:])
    if name == "extended_small":
        return wrap(base, "explicit", extended=True)
    if name == "opaque_tag_0700":
        return wrap(base, tag=0x0700)
    if name == "opaque_tag_0805":
        return wrap(base, tag=0x0805)
    if name == "nested_user_object":
        return wrap(base, "explicit", nested=True)
    if name == "invalid_extended_one_nine":
        return wrap(small["invalid_one_nine"], "explicit_file_end", extended=True)
    if name == "long_text_65553":
        text = (bytes(range(256)) * 257)[:65553]
        return wrap(
            payload([(65535, 65535, 65535, 241, 65535), (0, 0, 0, 240, 0)], text),
            "header",
        )
    if name in {"max_runs_65535_empty", "max_runs_65535_text"}:
        data = bytearray(u16(65535))
        for i in range(65535):
            data.extend(
                struct.pack(
                    "<5H",
                    65535 - i,
                    (73 * i + 7) & 65535,
                    (257 * i + 96) & 65535,
                    (29 * i + 240) & 65535,
                    (11 * i + 4) & 65535,
                )
            )
        if name.endswith("_text"):
            data.extend(b"\0\xff\x80A")
        return wrap(bytes(data), "explicit_file_end")
    if name == "invalid_max_count_following_padding":
        return wrap(b"\xff\xff" + bytes(9), following_padding=655350)
    raise KeyError(name)


def write_json(path: Path, value: object) -> None:
    path.write_bytes(
        (json.dumps(value, indent=2, ensure_ascii=True) + "\n").encode("ascii")
    )


def generate(destination: Path) -> dict[str, object]:
    destination.mkdir(parents=True, exist_ok=True)
    cases = []
    for name in NAMES:
        raw = case_source(name)
        stored = name not in RUNTIME
        if stored:
            (destination / (name + ".cdx")).write_bytes(raw)
        tag = (
            0x0700
            if name == "opaque_tag_0700"
            else 0x0805
            if name == "opaque_tag_0805"
            else VALUE_TAG
        )
        cases.append(
            {
                "name": name,
                "storage": "file" if stored else "runtime",
                "query_tag": tag,
                "query_valid": not name.startswith("invalid_"),
                "bytes": len(raw),
                "sha256": hashlib.sha256(raw).hexdigest(),
            }
        )
    manifest = {
        "method": "Original stdlib struct generation; raw fields only; runtime maxima are not bundled binary files.",
        "cases": cases,
    }
    write_json(destination / "string-vectors.json", manifest)
    return manifest


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=Path(__file__).with_name("data"))
    parser.add_argument("--raw-case", choices=NAMES)
    args = parser.parse_args()
    if args.raw_case:
        sys.stdout.buffer.write(case_source(args.raw_case))
    else:
        manifest = generate(args.output)
        print(
            json.dumps(
                {
                    "cases": len(manifest["cases"]),
                    "stored": len(SMALL_NAMES),
                    "runtime": len(RUNTIME),
                }
            )
        )


if __name__ == "__main__":
    main()
