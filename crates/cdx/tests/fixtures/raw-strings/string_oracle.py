#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Independent reader for original fixture envelopes and raw CDXString queries."""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from dataclasses import dataclass, field
from pathlib import Path

SIGNATURE = b"VjCD0100\x04\x03\x02\x01"


def blob(value: bytes | memoryview) -> dict[str, object]:
    result: dict[str, object] = {
        "byte_length": len(value),
        "sha256": hashlib.sha256(value).hexdigest(),
    }
    if len(value) <= 128:
        result["hex"] = bytes(value).hex()
    else:
        result["prefix_hex"] = bytes(value[:32]).hex()
        result["suffix_hex"] = bytes(value[-32:]).hex()
    return result


class QueryError(ValueError):
    def __init__(self, detail: dict[str, object]) -> None:
        super().__init__(detail)
        self.detail = detail


def invalid_handle() -> QueryError:
    return QueryError({"kind": "Source", "error": "InvalidProperty"})


@dataclass(frozen=True)
class Record:
    owner: object = field(repr=False)
    index: int
    kind: str
    parent: int | None
    begin: int
    end: int
    tag: int | None = None
    identity: int | None = None
    payload_begin: int | None = None
    payload_end: int | None = None
    encoding: str | None = None
    matched: int | None = None


class RawString:
    def __init__(self, document: Document, record: Record) -> None:
        self.document = document
        self.record = record
        assert record.payload_begin is not None and record.payload_end is not None
        begin, end = record.payload_begin, record.payload_end
        if end - begin < 2:
            raise QueryError({"kind": "InvalidPayload", "record": record.index})
        self.count = int.from_bytes(document.raw[begin : begin + 2], "little")
        self.runs_begin = begin + 2
        self.text_begin = self.runs_begin + self.count * 10
        if self.text_begin > end:
            raise QueryError({"kind": "InvalidPayload", "record": record.index})
        self.end = end

    def text_bytes(self) -> memoryview:
        return memoryview(self.document.raw)[self.text_begin : self.end]

    def run(self, ordinal: int) -> dict[str, object]:
        if not 0 <= ordinal < self.count:
            raise IndexError(ordinal)
        start = self.runs_begin + ordinal * 10
        fields = [
            int.from_bytes(self.document.raw[start + i : start + i + 2], "little")
            for i in (0, 2, 4, 6, 8)
        ]
        return dict(
            zip(("start", "font", "face", "size_twentieths", "color"), fields),
            ordinal=ordinal,
            range=[start, start + 10],
        )

    def facts(self) -> dict[str, object]:
        selected = (
            list(range(self.count))
            if self.count <= 16
            else sorted({0, 1, self.count // 2, self.count - 2, self.count - 1})
        )
        return {
            "ok": True,
            "record": self.record.index,
            "tag": self.record.tag,
            "payload_range": [self.record.payload_begin, self.record.payload_end],
            "run_count": self.count,
            "runs_range": [self.runs_begin, self.text_begin],
            "runs_sha256": hashlib.sha256(
                memoryview(self.document.raw)[self.runs_begin : self.text_begin]
            ).hexdigest(),
            "runs_compact": self.count > 16,
            "runs": [self.run(i) for i in selected],
            "text_range": [self.text_begin, self.end],
            "text": blob(self.text_bytes()),
        }


class Document:
    def __init__(self, raw: bytes) -> None:
        if (
            len(raw) > 64 * 1024 * 1024
            or len(raw) < 28
            or not raw.startswith(SIGNATURE)
        ):
            raise ValueError("fixture header/size")
        self.raw = raw
        self.token = object()
        self.records: list[Record] = []
        # These generated envelopes are deliberately unambiguous. No general fallback
        # inference about arbitrary historical reserved-header bytes is asserted.
        embedded = raw[22:24] == b"\0\x80"
        self.header_length = 22 if embedded else 28
        cursor = self.header_length
        encoded = embedded or raw[cursor : cursor + 2] == b"\0\x80"
        stack: list[int] = []

        def word(width: int) -> int:
            nonlocal cursor
            end = cursor + width
            if end > len(raw):
                raise ValueError("truncated record")
            value = int.from_bytes(raw[cursor:end], "little")
            cursor = end
            return value

        while True:
            if len(self.records) >= 100000:
                raise ValueError("record limit")
            begin = cursor
            tag = word(2)
            parent = stack[-1] if stack else None
            index = len(self.records)
            if tag == 0:
                matched = stack.pop() if stack else None
                parent = stack[-1] if stack else None
                self.records.append(
                    Record(
                        self.token, index, "end", parent, begin, cursor, matched=matched
                    )
                )
                complete = (encoded and matched == 0) or (
                    not encoded and matched is None
                )
                if complete:
                    has_file_end = raw[cursor:] == b"\0\0"
                    if has_file_end:
                        self.records.append(
                            Record(
                                self.token,
                                len(self.records),
                                "file_end",
                                None,
                                cursor,
                                cursor + 2,
                            )
                        )
                        cursor += 2
                    if cursor != len(raw) or (embedded and not has_file_end):
                        raise ValueError("fixture root/file end")
                    self.framing = (
                        "HeaderDocument"
                        if embedded
                        else "ExplicitDocumentFileEnd"
                        if encoded and has_file_end
                        else "ExplicitDocument"
                        if encoded
                        else "ImplicitDocumentFileEnd"
                        if has_file_end
                        else "ImplicitDocument"
                    )
                    return
            elif tag & 0x8000:
                identity = word(4)
                if len(stack) >= 256 or (encoded and index == 0 and tag != 0x8000):
                    raise ValueError("fixture object scope")
                self.records.append(
                    Record(
                        self.token,
                        index,
                        "object",
                        parent,
                        begin,
                        cursor,
                        tag,
                        identity,
                    )
                )
                stack.append(index)
            else:
                size = word(2)
                encoding = "short"
                if size == 65535:
                    size = word(4)
                    encoding = "extended"
                payload_begin = cursor
                cursor += size
                if cursor > len(raw):
                    raise ValueError("truncated property")
                self.records.append(
                    Record(
                        self.token,
                        index,
                        "property",
                        parent,
                        begin,
                        cursor,
                        tag=tag,
                        payload_begin=payload_begin,
                        payload_end=cursor,
                        encoding=encoding,
                    )
                )

    def string(self, record: Record) -> RawString:
        if (
            not isinstance(record, Record)
            or record.owner is not self.token
            or not 0 <= record.index < len(self.records)
            or self.records[record.index] is not record
            or record.kind != "property"
        ):
            raise invalid_handle()
        return RawString(self, record)

    def record_facts(self, record: Record) -> dict[str, object]:
        value: dict[str, object] = {
            "index": record.index,
            "kind": record.kind,
            "parent": record.parent,
            "range": [record.begin, record.end],
        }
        if record.kind == "object":
            value.update(tag=record.tag, id=record.identity)
        elif record.kind == "property":
            value.update(
                tag=record.tag,
                encoding=record.encoding,
                payload_range=[record.payload_begin, record.payload_end],
                payload=blob(
                    memoryview(self.raw)[record.payload_begin : record.payload_end]
                ),
            )
        elif record.kind == "end":
            value["matched"] = record.matched
        return value

    def facts(self, record_index: int) -> dict[str, object]:
        record = self.records[record_index]
        try:
            query = self.string(record).facts()
        except QueryError as error:
            query = {"ok": False, "error": error.detail}
        return {
            "framing": self.framing,
            "byte_length": len(self.raw),
            "sha256": hashlib.sha256(self.raw).hexdigest(),
            "header_range": [0, self.header_length],
            "header_hex": self.raw[: self.header_length].hex(),
            "records": [self.record_facts(item) for item in self.records],
            "query_record": record_index,
            "query": query,
        }


def read(
    raw: bytes, record_index: int | None = None, query_tag: int = 0x4001
) -> dict[str, object]:
    document = Document(raw)
    if record_index is None:
        selected = [
            r.index
            for r in document.records
            if r.kind == "property" and r.tag == query_tag
        ]
        if len(selected) != 1:
            raise ValueError("fixture query selector")
        record_index = selected[0]
    return document.facts(record_index)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", nargs="?", default="-")
    parser.add_argument("--record", type=int)
    parser.add_argument("--tag", type=lambda value: int(value, 0), default=0x4001)
    args = parser.parse_args()
    raw = (
        sys.stdin.buffer.read() if args.input == "-" else Path(args.input).read_bytes()
    )
    result = read(raw, args.record, args.tag)
    sys.stdout.buffer.write(
        (json.dumps(result, ensure_ascii=True) + "\n").encode("ascii")
    )


if __name__ == "__main__":
    main()
