"""Original independent ordered CDXML root and Decimal reader; no defaults."""
import json
import sys
import xml.parsers.expat as expat
from decimal import Decimal

raw = sys.stdin.buffer.read(1_048_577)
assert 0 < len(raw) <= 1_048_576
root = None
depth = 0


def start(name, attributes):
    global root, depth
    if depth == 0:
        assert name == 'CDXML' and root is None
        root = list(zip(attributes[::2], attributes[1::2]))
    depth += 1


def end(_):
    global depth
    depth -= 1


parser = expat.ParserCreate()
parser.ordered_attributes = True
parser.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
parser.StartElementHandler, parser.EndElementHandler = start, end
parser.Parse(raw, True)
assert root is not None and depth == 0
numbers = {}
for name, raw_value in root:
    if name in ('BondLength', 'ChainAngle'):
        value = Decimal(raw_value)
        assert value.is_finite()
        numbers[name] = str(value)
print(json.dumps({'root': root, 'decimal': numbers}, separators=(',', ':')))
