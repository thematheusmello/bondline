# SPDX-License-Identifier: MIT OR Apache-2.0
"""Independent original CDX resource/framing reader; standard library only."""

import hashlib, json, struct, sys

LIMIT = 64 * 1024 * 1024
PREFIX = b"VjCD0100\x04\x03\x02\x01"


def require(value, message):
    if not value:
        raise ValueError(message)


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def bytes_fact(raw):
    result = {"bytes": len(raw), "sha256": digest(raw)}
    if len(raw) <= 96:
        result["hex"] = raw.hex()
    else:
        result.update(prefix_hex=raw[:16].hex(), suffix_hex=raw[-16:].hex())
    return result


class QueryError(Exception):
    def __init__(self, kind, **fields):
        self.data = {"kind": kind, **fields}


def raw_records(source):
    require(30 <= len(source) <= LIMIT and source[:12] == PREFIX, "header/size")
    header = 22 if source[22:28] == struct.pack("<HI", 0x8000, 0) else 28
    offset = header
    encoded = struct.unpack_from("<H", source, offset)[0] == 0x8000
    records = []
    stack = []
    while True:
        require(
            len(records) < 100000 and offset + 2 <= len(source), "tag/record budget"
        )
        start = offset
        tag = struct.unpack_from("<H", source, offset)[0]
        offset += 2
        parent = stack[-1] if stack else None
        if tag == 0:
            matched = stack.pop() if stack else None
            records.append(
                {
                    "kind": "end",
                    "parent": stack[-1] if stack else None,
                    "matched": matched,
                    "range": [start, offset],
                }
            )
            if not stack and (encoded or matched is None):
                file_end = source[offset:] == bytes(2)
                if file_end:
                    records.append(
                        {
                            "kind": "file_end",
                            "parent": None,
                            "range": [offset, offset + 2],
                        }
                    )
                    offset += 2
                require(offset == len(source), "trailing/missing file end")
                if header == 22:
                    require(encoded and file_end, "header document file end")
                    framing = "HeaderDocument"
                elif encoded:
                    framing = (
                        "ExplicitDocumentFileEnd" if file_end else "ExplicitDocument"
                    )
                else:
                    framing = (
                        "ImplicitDocumentFileEnd" if file_end else "ImplicitDocument"
                    )
                return framing, header, records
        elif tag & 0x8000:
            require(offset + 4 <= len(source) and len(stack) < 256, "object/depth")
            identity = struct.unpack_from("<I", source, offset)[0]
            offset += 4
            records.append(
                {
                    "kind": "start",
                    "tag": tag,
                    "id": identity,
                    "parent": parent,
                    "range": [start, offset],
                }
            )
            stack.append(len(records) - 1)
        else:
            require(offset + 2 <= len(source), "length")
            length = struct.unpack_from("<H", source, offset)[0]
            offset += 2
            encoding = "short"
            if length == 65535:
                require(offset + 4 <= len(source), "extended length")
                length = struct.unpack_from("<I", source, offset)[0]
                offset += 4
                encoding = "extended"
            begin = offset
            offset += length
            require(offset <= len(source), "payload")
            records.append(
                {
                    "kind": "property",
                    "tag": tag,
                    "parent": parent,
                    "range": [start, offset],
                    "encoding": encoding,
                    "payload_range": [begin, offset],
                    "payload": bytes_fact(source[begin:offset]),
                }
            )


def selected(count):
    return (
        list(range(count))
        if count <= 16
        else sorted({0, 1, count // 2, count - 2, count - 1})
    )


def table_handle(index, record):
    return {
        "record": index,
        "tag": record["tag"],
        "range": record["range"],
        "payload_range": record["payload_range"],
    }


def query(source, framing, records):
    tables = {}
    expected_parent = None if framing.startswith("Implicit") else 0
    for index, record in enumerate(records):
        if record["kind"] != "property" or record["tag"] not in (0x0100, 0x0300):
            continue
        tag = record["tag"]
        if record["parent"] != expected_parent:
            raise QueryError("UnsupportedTable", tag=tag, record=index)
        if expected_parent == 0 and not (
            records[0]["kind"] == "start" and records[0]["tag"] == 0x8000
        ):
            raise QueryError("UnsupportedTable", tag=tag, record=index)
        if tag in tables:
            raise QueryError("DuplicateTable", tag=tag, record=index)
        tables[tag] = (index, record)
    font_entries = []
    font_ids = {}
    font_platform = None
    font_bounds = None
    color_entries = []
    color_bounds = None
    if 0x0100 in tables:
        index, record = tables[0x0100]
        begin, end = record["payload_range"]
        at = begin

        def invalid_font():
            raise QueryError("InvalidTable", tag=0x0100, record=index)

        if end - at < 4:
            invalid_font()
        font_platform, count = struct.unpack_from("<HH", source, at)
        at += 4
        if font_platform not in (0, 1):
            raise QueryError("UnsupportedPlatform", record=index, value=font_platform)
        entry_begin = at
        for ordinal in range(count):
            if end - at < 6:
                invalid_font()
            start = at
            identity, charset, name_size = struct.unpack_from("<HHH", source, at)
            at += 6
            name_start = at
            at += name_size
            if at > end:
                invalid_font()
            if identity in font_ids:
                raise QueryError(
                    "DuplicateFontId", record=index, id=identity, ordinal=ordinal
                )
            entry = (
                ordinal,
                identity,
                charset,
                start,
                at,
                name_start,
                name_size,
                index,
            )
            font_ids[identity] = entry
            font_entries.append(entry)
        if at != end:
            invalid_font()
        font_bounds = [entry_begin, end]
    if 0x0300 in tables:
        index, record = tables[0x0300]
        begin, end = record["payload_range"]
        if end - begin < 2:
            raise QueryError("InvalidTable", tag=0x0300, record=index)
        count = struct.unpack_from("<H", source, begin)[0]
        at = begin + 2
        if count * 6 != end - at:
            raise QueryError("InvalidTable", tag=0x0300, record=index)
        for ordinal in range(count):
            start = at
            rgb = list(struct.unpack_from("<HHH", source, at))
            at += 6
            color_entries.append((ordinal + 2, rgb, start, at, index))
        color_bounds = [begin + 2, end]

    def font_fact(entry):
        ordinal, identity, charset, start, end, name_start, name_size, index = entry
        name = source[name_start : name_start + name_size]
        return {
            "id": identity,
            "charset": charset,
            "ordinal": ordinal,
            "range": [start, end],
            "name_range": [name_start, name_start + name_size],
            "name": bytes_fact(name),
            "record": index,
        }

    def color_fact(entry):
        index, rgb, start, end, record = entry
        return {"index": index, "rgb": rgb, "range": [start, end], "record": record}

    font_lookups = []
    for identity in [0, 1, 7, 42, 32768, 65535]:
        if identity in font_ids:
            font_lookups.append({"id": identity, "font": font_fact(font_ids[identity])})
        else:
            font_lookups.append(
                {"id": identity, "error": {"kind": "MissingFont", "id": identity}}
            )
    color_lookups = []
    for index in [0, 1, 2, 3, 4, 65535, 65536, 65537, 4294967295]:
        if index < 2:
            color_lookups.append(
                {
                    "index": index,
                    "rgb": [0, 0, 0] if index == 0 else [65535, 65535, 65535],
                    "origin": "reserved",
                }
            )
        elif 0x0300 not in tables and index in (2, 3):
            color_lookups.append(
                {
                    "index": index,
                    "rgb": [65535, 65535, 65535] if index == 2 else [0, 0, 0],
                    "origin": "absent_default",
                }
            )
        elif 2 <= index < len(color_entries) + 2:
            color_lookups.append(
                {"index": index, "rgb": color_entries[index - 2][1], "origin": "entry"}
            )
        else:
            color_lookups.append(
                {"index": index, "error": {"kind": "MissingColor", "index": index}}
            )
    return {
        "ok": True,
        "font_table": table_handle(*tables[0x0100]) if 0x0100 in tables else None,
        "color_table": table_handle(*tables[0x0300]) if 0x0300 in tables else None,
        "font_platform": font_platform,
        "font_count": len(font_entries),
        "color_count": len(color_entries),
        "fonts_compact": len(font_entries) > 16,
        "colors_compact": len(color_entries) > 16,
        "fonts": [font_fact(font_entries[i]) for i in selected(len(font_entries))],
        "colors": [color_fact(color_entries[i]) for i in selected(len(color_entries))],
        "font_entries_range": font_bounds,
        "color_entries_range": color_bounds,
        "font_entries_sha256": digest(source[slice(*font_bounds)])
        if font_bounds is not None
        else None,
        "color_entries_sha256": digest(source[slice(*color_bounds)])
        if color_bounds is not None
        else None,
        "font_lookups": font_lookups,
        "color_lookups": color_lookups,
    }


def read(source):
    framing, header, records = raw_records(source)
    try:
        resources = query(source, framing, records)
    except QueryError as error:
        resources = {"ok": False, "error": error.data}
    return {
        "framing": framing,
        "header_range": [0, header],
        "header_hex": source[:header].hex(),
        "byte_length": len(source),
        "sha256": digest(source),
        "records": records,
        "query": resources,
    }


if __name__ == "__main__":
    print(json.dumps(read(sys.stdin.buffer.read(LIMIT + 1)), separators=(",", ":")))
