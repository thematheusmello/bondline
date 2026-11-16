"""Original bounded raw-property/Decimal check; no chemistry normalization."""
import json
import sys
import xml.parsers.expat as expat
from decimal import Decimal

assert len(sys.argv) == 2
expected = json.loads(sys.argv[1])
raw = sys.stdin.buffer.read(1_048_577)
assert 0 < len(raw) <= 1_048_576
objects = []
parser = expat.ParserCreate()
parser.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
parser.StartElementHandler = lambda name, attrs: objects.append((name, attrs))
parser.Parse(raw, True)
atoms, bonds = expected['atoms'], expected['bonds']
assert [name for name, _ in objects] == ['CDXML', 'page', 'fragment'] + ['n'] * len(atoms) + ['b'] * len(bonds)
nodes = [attrs for name, attrs in objects if name == 'n']
edges = [attrs for name, attrs in objects if name == 'b']
assert len(nodes) == len(atoms) and len(edges) == len(bonds)
for i, (node, atom) in enumerate(zip(nodes, atoms)):
    fields = {'id', 'NodeType', 'Element', 'Charge', 'Radical', 'p', 'NumHydrogens'}
    for key, attribute in [('show_terminal_carbon_labels', 'ShowTerminalCarbonLabels'),
                           ('show_non_terminal_carbon_labels', 'ShowNonTerminalCarbonLabels')]:
        if key in atom:
            assert type(atom[key]) is bool
            fields.add(attribute)
            assert node[attribute] == ('yes' if atom[key] else 'no')
        else:
            assert attribute not in node
    assert set(node) == fields and int(node['id']) == i + 3
    assert atom['id'] == i + 1 and atom['element'] == 6
    assert atom['isotope'] is None and atom['charge'] == 0 and atom['radical'] == 'none'
    assert node['NodeType'] == 'Element' and node['Element'] == '6'
    assert node['Charge'] == '0' and node['Radical'] == 'None'
    assert Decimal(node['NumHydrogens']) == atom['explicit_hydrogens']
    point = list(map(Decimal, node['p'].split()))
    assert len(point) == 2
    for actual, axis in zip(point, ['x_pt', 'y_pt']):
        assert actual.is_finite() and abs(actual - Decimal(str(atom['position'][axis]))) <= Decimal('0.000000001')
for i, (edge, bond) in enumerate(zip(edges, bonds)):
    fields = {'id', 'B', 'E', 'Order'}
    if 'double_position' in bond:
        fields.add('DoublePosition')
        assert edge['DoublePosition'] == {'left': 'Left', 'right': 'Right', 'center': 'Center'}[bond['double_position']]
    else:
        assert 'DoublePosition' not in edge
    assert set(edge) == fields and int(edge['id']) == len(nodes) + i + 3
    assert bond['id'] == i + 1
    assert [int(edge['B']) - 2, int(edge['E']) - 2] == [bond['begin'], bond['end']]
    assert edge['Order'] == {'single': '1', 'double': '2', 'triple': '3'}[bond['order']]
print('{"raw_properties_and_decimal":true}')
