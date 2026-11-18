# SPDX-License-Identifier: MIT OR Apache-2.0
"""Independent test-only Expat/Decimal owner/value reader; no Bondline code or fonts."""
import decimal
import json
import math
import re
import sys
import xml.parsers.expat as expat

assert len(sys.argv) == 2
expected = json.loads(sys.argv[1], parse_float=decimal.Decimal)
assert isinstance(expected, list) and 0 < len(expected) <= 64
assert all(isinstance(row, dict) and isinstance(row.get("id"), str) for row in expected)
FIELDS = [("LabelFont", "font"), ("LabelSize", "size_pt"), ("LabelFace", "face")]
ROLES = {"n", "b", "t", "table", "spectrum", "constraint", "tlcplate"}
INTEGER = re.compile(r"[+-]?[0-9]+(?:\.0+)?\Z")
REAL = re.compile(r"[+-]?(?:[0-9]+(?:\.[0-9]*)?|\.[0-9]+)(?:[eE][+-]?[0-9]+)?\Z")

source = sys.stdin.buffer.read(65537)
assert len(source) <= 65536
nodes, stack, identifiers = [], [], {}
parser = expat.ParserCreate(namespace_separator="|")
parser.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
parser.ExternalEntityRefHandler = lambda *unused: 0

def start(name, attrs):
    node = {"name": name, "attrs": attrs, "parent": stack[-1] if stack else None}
    index = len(nodes)
    nodes.append(node)
    if "id" in attrs:
        assert attrs["id"] not in identifiers
        identifiers[attrs["id"]] = index
    stack.append(index)

def end(name):
    assert nodes[stack.pop()]["name"] == name

parser.StartElementHandler = start
parser.EndElementHandler = end
parser.Parse(source, True)
assert nodes and nodes[0]["name"] == "CDXML" and not stack

def legal_owner(index):
    name = nodes[index]["name"]
    return index == 0 or name in ROLES

def number(property_name, raw):
    value = raw.strip(" \t\r\n")
    if property_name != "LabelSize":
        assert INTEGER.fullmatch(value)
        exact = decimal.Decimal(value)
        assert -32768 <= exact <= 32767
        return int(exact)
    assert REAL.fullmatch(value)
    exact = decimal.Decimal(value)
    binary = float(value)
    assert exact.is_finite() and exact > 0 and math.isfinite(binary) and binary > 0
    return exact

def resolve(target_id):
    index = identifiers[target_id]
    if not legal_owner(index):
        return {"id": target_id, "error": ["InvalidElement"]}
    values = {key: None for _, key in FIELDS}
    while index is not None:
        node = nodes[index]
        for property_name, key in FIELDS:
            # Expanded qualified names remain unrelated raw attributes in Expat.
            if property_name not in node["attrs"]:
                continue
            if not legal_owner(index):
                return {"id": target_id, "error": ["UnsupportedOwner", property_name]}
            raw = node["attrs"][property_name]
            try:
                value = number(property_name, raw)
            except (AssertionError, ValueError, OverflowError, decimal.InvalidOperation):
                return {"id": target_id, "error": ["InvalidProperty", property_name]}
            if values[key] is None:
                values[key] = [value, node["name"], node["attrs"].get("id", ""), raw]
        index = node["parent"]
    return {"id": target_id, **values}

actual = [resolve(row["id"]) for row in expected]
assert actual == expected, (actual, expected)
print("independent label-default owners and values agree")
