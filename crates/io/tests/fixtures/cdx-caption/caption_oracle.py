# SPDX-License-Identifier: MIT OR Apache-2.0
"""Independent ORIGINAL CDX caption research reader; stdlib only, no product imports.

The immutable original resources oracle supplies raw framing/record/resource facts.
The caption profile below is a PROPOSAL witness, not an application success assertion.
Unknown/unsupported profile data stays in raw facts and produces preview_candidate=False.
"""

import hashlib
import importlib.util
import json
import struct
import sys
from pathlib import Path

# Integrated dce5e0f qualifies formatter/portable-LF/import changes by identical
# reproduction of all48 original resource binaries/facts/checks (Root receipt).
RESOURCE_SHA = "a5a2df85a9fc1a2df9ba57b1a7b5538c812a6f724e18d25a42a16d98d6ad5db2"


def resource_reader():
    starts = [Path.cwd()]
    if "__file__" in globals():
        starts.insert(0, Path(__file__).resolve().parent)
    for start in starts:
        for root in [start, *start.parents]:
            p = root / "crates/cdx/tests/fixtures/resources/resource_oracle.py"
            if p.is_file():
                raw = p.read_bytes()
                if hashlib.sha256(raw).hexdigest() != RESOURCE_SHA:
                    raise ValueError("immutable independent resources reader pin")
                spec = importlib.util.spec_from_file_location(
                    "original_resource_reader", p
                )
                module = importlib.util.module_from_spec(spec)
                spec.loader.exec_module(module)
                return module
    raise ValueError("independent resources reader not found from source or cwd")


RESOURCE = resource_reader()


def byte_data(raw):
    return {
        "length": len(raw),
        "hex": raw.hex(),
        "sha256": hashlib.sha256(raw).hexdigest(),
    }


def string_data(raw, begin):
    result = {"range": [begin, begin + len(raw)], "raw": byte_data(raw)}
    if len(raw) < 2:
        return {**result, "error": "missing_run_count"}
    count = struct.unpack_from("<H", raw)[0]
    result["run_count"] = count
    text_start = 2 + 10 * count
    if text_start > len(raw):
        return {**result, "error": "truncated_runs"}
    runs = []
    for ordinal in range(count):
        at = 2 + ordinal * 10
        start, font, face, size, color = struct.unpack_from("<HHHHH", raw, at)
        runs.append(
            {
                "ordinal": ordinal,
                "range": [begin + at, begin + at + 10],
                "start": start,
                "font": font,
                "face": face,
                "size_twentieths": size,
                "size_pt_decimal": str(size / 20),
                "color": color,
            }
        )
    body = raw[text_start:]
    result.update(
        runs=runs,
        text_range=[begin + text_start, begin + len(raw)],
        text_bytes=byte_data(body),
    )
    try:
        result["utf8"] = body.decode("utf-8", errors="strict")
    except UnicodeDecodeError:
        result["utf8"] = None
    if count == 0:
        result["zero_run_latin1"] = body.decode("iso-8859-1")
    return result


def read(source):
    raw = RESOURCE.read(source)
    records = raw["records"]
    query = raw["query"]
    root = None if raw["framing"].startswith("Implicit") else 0
    issues = []

    def issue(kind, record=None, **extra):
        item = {"kind": kind, **extra}
        if record is not None:
            item["record"] = record
        if item not in issues:
            issues.append(item)

    def payload(record):
        a, b = records[record]["payload_range"]
        return source[a:b]

    def properties(owner):
        return [
            (i, record)
            for i, record in enumerate(records)
            if record["kind"] == "property" and record["parent"] == owner
        ]

    def first(owner, tag):
        found = [i for i, r in properties(owner) if r["tag"] == tag]
        if len(found) != 1:
            issue("missing_or_duplicate_field", owner, tag=tag, count=len(found))
            return None
        return found[0]

    def unpack(owner, tag, fmt):
        record = first(owner, tag)
        if record is None:
            return None
        data = payload(record)
        if len(data) != struct.calcsize(fmt):
            issue("wrong_field_width", record, tag=tag, width=len(data))
            return None
        values = struct.unpack(fmt, data)
        return values[0] if len(values) == 1 else list(values)

    header = source[: raw["header_range"][1]]
    if header != b"VjCD0100\x04\x03\x02\x01" + bytes(len(header) - 12):
        issue("unadmitted_header_bytes")

    objects = [(i, r) for i, r in enumerate(records) if r["kind"] == "start"]
    pages = [(i, r) for i, r in objects if r["tag"] == 0x8001 and r["parent"] == root]
    if len(pages) != 1:
        issue("exactly_one_direct_page", count=len(pages))
    page = pages[0][0] if len(pages) == 1 else None
    texts = [(i, r) for i, r in objects if r["tag"] == 0x8006 and r["parent"] == page]
    if len(texts) != 1:
        issue("exactly_one_direct_text", count=len(texts))
    text = texts[0][0] if len(texts) == 1 else None
    for i, r in objects:
        good = (
            i == root
            and r["tag"] == 0x8000
            and r["parent"] is None
            or i == page
            and r["tag"] == 0x8001
            and r["parent"] == root
            or i == text
            and r["tag"] == 0x8006
            and r["parent"] == page
        )
        if not good:
            issue("unadmitted_object_or_owner", i, tag=r["tag"], parent=r["parent"])

    owners = {root: {0x0100, 0x0300}}
    if page is not None:
        owners[page] = {0x0811, 0x0812, 0x0813}
    if text is not None:
        owners[text] = {0x0200, 0x080C, 0x0700}
    seen = set()
    for i, record in enumerate(records):
        if record["kind"] != "property":
            continue
        key = (record["parent"], record["tag"])
        if key in seen:
            issue("duplicate_field", i, tag=record["tag"])
        seen.add(key)
        if record["tag"] not in owners.get(record["parent"], set()):
            issue(
                "unadmitted_property_or_owner",
                i,
                tag=record["tag"],
                parent=record["parent"],
            )
    if not query["ok"]:
        issue("invalid_resources", error=query["error"])

    def color(index):
        if not query["ok"]:
            return None
        if index in (0, 1):
            return [0, 0, 0] if index == 0 else [65535, 65535, 65535]
        if query["color_table"] is None:
            if index in (2, 3):
                return [65535, 65535, 65535] if index == 2 else [0, 0, 0]
            return None
        return next(
            (entry["rgb"] for entry in query["colors"] if entry["index"] == index), None
        )

    background = color(2)
    if background != [65535, 65535, 65535]:
        issue("missing_or_nonwhite_palette2")
    if query["ok"] and query["font_platform"] != 1:
        issue("explicit_platform1_required", value=query["font_platform"])

    width_raw = unpack(page, 0x0812, "<i") if page is not None else None
    height_raw = unpack(page, 0x0813, "<i") if page is not None else None
    drawing = unpack(page, 0x0811, "<b") if page is not None else None
    if drawing != 1:
        issue("explicit_poster_required", page)
    dimensions = None
    if width_raw is not None and height_raw is not None:
        dimensions = [width_raw / 65536, height_raw / 65536]
        if min(dimensions) <= 0:
            issue("positive_page_extent_required", page)

    point_raw = unpack(text, 0x0200, "<ii") if text is not None else None
    justification = unpack(text, 0x080C, "<b") if text is not None else None
    if justification != 0:
        issue("explicit_caption_left_required", text)
    strings = []
    for i, record in enumerate(records):
        if record["kind"] == "property" and record["tag"] in (0x0700, 0x0709):
            data = string_data(payload(i), record["payload_range"][0])
            strings.append(
                {"record": i, "tag": record["tag"], "parent": record["parent"], **data}
            )
    selected = first(text, 0x0700) if text is not None else None
    string = next((entry for entry in strings if entry["record"] == selected), None)
    run = None
    font = None
    rgb = None
    body = None
    if string is not None:
        if "error" in string:
            issue("invalid_string_layout", selected, reason=string["error"])
        elif string["run_count"] != 1:
            issue(
                "exactly_one_explicit_run_required", selected, count=string["run_count"]
            )
        else:
            run = string["runs"][0]
            if run["start"] != 0:
                issue("explicit_run_start0_required", selected, value=run["start"])
            if run["face"] != 0:
                issue("plain_face0_required", selected, value=run["face"])
            if run["size_twentieths"] == 0:
                issue("positive_run_size_required", selected)
            if query["ok"]:
                font = next(
                    (entry for entry in query["fonts"] if entry["id"] == run["font"]),
                    None,
                )
            if font is None:
                issue("missing_selected_declared_font", selected)
            elif font["ordinal"] != run["font"]:
                issue(
                    "font_id_ordinal_disagreement",
                    selected,
                    declared=font["id"],
                    ordinal=font["ordinal"],
                )
            elif font["charset"] != 65001 or font["name"].get("hex") != b"Roboto".hex():
                issue("explicit_utf8_roboto_required", selected)
            rgb = color(run["color"])
            if rgb is None:
                issue("missing_inline_color", selected, value=run["color"])
            body = string.get("utf8")
            if not body:
                issue("nonempty_strict_utf8_required", selected)
            elif any(
                not (0x20 <= ord(ch) <= 0x7E or 0x370 <= ord(ch) <= 0x3FF)
                for ch in body
            ):
                issue("single_line_ascii_greek_scope", selected)

    nominal = None
    if (
        dimensions is not None
        and min(dimensions) > 0
        and point_raw is not None
        and run is not None
        and run["size_twentieths"] > 0
        and body is not None
        and font is not None
        and font["name"].get("hex") == b"Roboto".hex()
        and rgb is not None
    ):
        nominal = {
            "pages": [
                {
                    "id": 1,
                    "size": {"width_pt": dimensions[0], "height_pt": dimensions[1]},
                    "fragments": [],
                    "texts": [
                        {
                            "id": 1,
                            "position": {
                                "x_pt": point_raw[1] / 65536,
                                "y_pt": point_raw[0] / 65536,
                            },
                            "runs": [
                                {
                                    "text": body,
                                    "style": {
                                        "font_family": "Roboto",
                                        "size_pt": run["size_twentieths"] / 20,
                                        "face": run["face"],
                                        "rgb": [value / 65535 for value in rgb],
                                    },
                                }
                            ],
                        }
                    ],
                }
            ]
        }
    candidate = not issues and nominal is not None
    return {
        "source": {
            key: raw[key]
            for key in (
                "framing",
                "header_range",
                "header_hex",
                "byte_length",
                "sha256",
                "records",
            )
        },
        "resources": query,
        "strings": strings,
        "identities": {
            "document": {"record": root, "source_id": records[root]["id"]}
            if root is not None
            else None,
            "page": {"record": page, "source_id": records[page]["id"], "model_id": 1}
            if page is not None
            else None,
            "text": {"record": text, "source_id": records[text]["id"], "model_id": 1}
            if text is not None
            else None,
        },
        "page_fields": {
            "drawing_space_raw": drawing,
            "width_fixed": width_raw,
            "height_fixed": height_raw,
            "background_rgb16": background,
        },
        "text_fields": {
            "position_yx_fixed": point_raw,
            "caption_justification_raw": justification,
            "selected_font": font,
            "selected_rgb16": rgb,
        },
        "preview": {
            "candidate": candidate,
            "issues": issues,
            "scope": "original proposed profile only; no app evidence",
        },
        "model": nominal if candidate else None,
        "nominal_model": nominal,
    }


if __name__ == "__main__":
    value = json.dumps(
        read(sys.stdin.buffer.read(64 * 1024 * 1024 + 1)),
        ensure_ascii=False,
        separators=(",", ":"),
    )
    sys.stdout.buffer.write(value.encode("utf-8") + b"\n")
