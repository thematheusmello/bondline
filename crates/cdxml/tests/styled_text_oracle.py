"""Independent test-only Expat/Decimal view; no Bondline parser or writer."""
import decimal
import json
import re
import sys
import xml.parsers.expat as expat

assert len(sys.argv) == 2
expected = json.loads(sys.argv[1], parse_float=decimal.Decimal)
assert isinstance(expected, list)
for row in expected:
    assert isinstance(row, list) and len(row) == 5 and isinstance(row[0], str)
    for at in [1, 3, 4]:
        assert row[at] is None or (type(row[at]) is int and 0 <= row[at] <= 65535)
    assert row[2] is None or (
        type(row[2]) in (int, decimal.Decimal)
        and decimal.Decimal(row[2]).is_finite() and row[2] > 0
    )

def integer(attrs, key):
    if key not in attrs:
        return None
    raw = attrs[key].strip(" \t\r\n")
    assert re.fullmatch(r"[+-]?[0-9]+(?:\.0+)?", raw)
    value = decimal.Decimal(raw)
    assert 0 <= value <= 65535
    return int(value)

def size(attrs):
    if "size" not in attrs:
        return None
    value = decimal.Decimal(attrs["size"].strip(" \t\r\n"))
    assert value.is_finite() and value > 0
    return value

rows, stack = [], []
parser = expat.ParserCreate(namespace_separator="|")
parser.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
parser.ExternalEntityRefHandler = lambda *unused: 0
def start(name, attrs):
    if stack and stack[-1] == "t":
        assert name == "s", name
        rows.append(["", integer(attrs, "font"), size(attrs),
                     integer(attrs, "face"), integer(attrs, "color")])
    elif "s" in stack:
        raise AssertionError("mixed run")
    stack.append(name)
def chars(text):
    if stack and stack[-1] == "s":
        rows[-1][0] += text
    elif stack and stack[-1] == "t":
        assert not text.strip(" \t\r\n"), repr(text)
def end(name):
    assert stack.pop() == name
def trivia(*unused):
    assert "t" not in stack, "unexpected Text trivia"
parser.StartElementHandler = start
parser.CharacterDataHandler = chars
parser.EndElementHandler = end
parser.CommentHandler = trivia
parser.ProcessingInstructionHandler = trivia
source = sys.stdin.buffer.read(65537)
assert len(source) <= 65536
parser.Parse(source, True)
assert rows == expected, (rows, expected)
print("independent styled-run values agree")
