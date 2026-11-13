"""Independent Expat/Decimal projection of the documented elemental subset, test-only."""
import decimal
import json
import sys
import xml.parsers.expat as expat

orders = dict(zip(map(decimal.Decimal, ["1", "2", "3", "4", "5", "6", "0.5", "1.5", "2.5", "3.5", "4.5", "5.5"]),
                  ["single", "double", "triple", "quadruple", "quintuple", "sextuple", "half", "aromatic", "two_and_half", "three_and_half", "four_and_half", "five_and_half"]))
words = {"dative": "dative", "ionic": "ionic", "hydrogen": "hydrogen", "threecenter": "three_center"}
fragments = []
current = None
atom_count = 0
bond_count = 0


def integer(value):
    numerator, denominator = decimal.Decimal(value).as_integer_ratio()
    assert denominator == 1
    return numerator


def start(name, attributes):
    global current, atom_count
    if name == "fragment":
        assert current is None
        current = {"id": len(fragments) + 1, "atoms": [], "bonds": [], "references": {}, "pending": []}
        fragments.append(current)
    elif name == "n" and current is not None:
        assert attributes.get("NodeType", "Element") == "Element"
        atom_count += 1
        source_id = integer(attributes.get("id", "0"))
        assert 0 <= source_id <= 0xffffffff
        if source_id:
            assert source_id not in current["references"]
            current["references"][source_id] = atom_count
        position = None
        if "p" in attributes:
            x, y = map(float, attributes["p"].split())
            position = {"x_pt": x, "y_pt": y}
        current["atoms"].append({
            "id": atom_count, "element": integer(attributes.get("Element", "6")),
            "position": position, "isotope": integer(attributes.get("Isotope", "0")) or None,
            "charge": integer(attributes.get("Charge", "0")),
            "radical": attributes.get("Radical", "None").lower(),
            "explicit_hydrogens": integer(attributes["NumHydrogens"]) if "NumHydrogens" in attributes else None,
        })
    elif name == "b" and current is not None:
        current["pending"].append(attributes)


def end(name):
    global current, bond_count
    if name != "fragment":
        return
    for attributes in current.pop("pending"):
        bond_count += 1
        order = attributes.get("Order", "1")
        kind = words[order] if order in words else orders[decimal.Decimal(order)]
        current["bonds"].append({
            "id": bond_count, "begin": current["references"][integer(attributes["B"])],
            "end": current["references"][integer(attributes["E"])], "order": kind,
        })
    del current["references"]
    current = None


parser = expat.ParserCreate(namespace_separator="|")
parser.StartElementHandler = start
parser.EndElementHandler = end
parser.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
parser.Parse(sys.stdin.buffer.read(), True)
json.dump(fragments, sys.stdout, ensure_ascii=True, allow_nan=False)
