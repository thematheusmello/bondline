"""Independent original Expat/Decimal consumer, not a Bondline writer/reader round trip."""
import json
import sys
import xml.parsers.expat as expat
from decimal import Decimal

raw = sys.stdin.buffer.read(1048577)
assert 0 < len(raw) <= 1048576
fonts = {}
colors = []
texts = []
stack = []
ids = set()
current = None
run = None

def number(s):
    d = Decimal(s)
    assert d.is_finite()
    return float(d)

def integer(s):
    d = Decimal(s)
    assert d.is_finite() and d == d.to_integral_value()
    return int(d)

def start(name, attrs):
    global current, run
    assert '|' not in name
    parent = stack[-1] if stack else None
    stack.append(name)
    if 'id' in attrs and name != 'font':
        n = integer(attrs['id'])
        assert n > 0 and n not in ids
        ids.add(n)
    if name == 'font':
        n = integer(attrs['id'])
        assert 0 <= n <= 65535 and n not in fonts
        fonts[n] = attrs['name']
    if name == 'color':
        rgb = [number(attrs[k]) for k in ('r', 'g', 'b')]
        assert all(0 <= n <= 1 for n in rgb)
        colors.append(rgb)
    if name == 't':
        assert parent in ('page', 'n')
        p = attrs.get('p')
        p = None if p is None else [number(n) for n in p.split()]
        assert p is None or len(p) == 2
        current = {'parent': parent, 'position': p,
                   'justification': attrs.get('Justification'),
                   'label_justification': attrs.get('LabelJustification'),
                   'label_alignment': attrs.get('LabelAlignment'), 'runs': []}
        texts.append(current)
    if name == 's':
        assert parent == 't' and current is not None
        font = fonts[integer(attrs['font'])]
        size = number(attrs['size'])
        face = integer(attrs['face'])
        color = integer(attrs['color'])
        assert size > 0 and 0 <= face <= 65535
        rgb = [0.0] * 3 if color == 0 else [1.0] * 3 if color == 1 else colors[color-2]
        run = {'text': '', 'font_family': font, 'size_pt': size, 'face': face, 'rgb': rgb}
        current['runs'].append(run)

def end(name):
    global current, run
    assert stack.pop() == name
    if name == 's':
        run = None
    if name == 't':
        assert current['runs']
        current = None

def chars(text):
    if run is not None:
        run['text'] += text
    else:
        assert not text.strip(' \t\r\n')

p = expat.ParserCreate(namespace_separator='|')
p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
p.ExternalEntityRefHandler = lambda *unused: 0
p.StartElementHandler = start
p.EndElementHandler = end
p.CharacterDataHandler = chars
p.Parse(raw, True)
print(json.dumps(texts, ensure_ascii=False, separators=(',', ':')))
