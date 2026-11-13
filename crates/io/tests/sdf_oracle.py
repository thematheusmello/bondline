"""Original MIT OR Apache-2.0 external SDF oracle; never linked or shipped.

Raw spans/ordered data are scanned independently of Bondline. RDKit's last-value
property map is diagnostic only. BIOVIA Chemistry 2026, printed pages 97-99:
https://www.3ds.com/assets/invest/2026-01/biovia_chemistry_2026_ctfile_formats.pdf
"""
import hashlib
import io
import json
import re
import sys

from rdkit import Chem, rdBase

MAX_BYTES = 64 * 1024 * 1024
MAX_RECORDS = 1024
MAX_FIELDS = 65536
MAX_LINES = 1_000_000
MAX_LINE_BYTES = 4096


def header_identity(header):
    """Validate the explicit ASCII identity subset; leave other text untouched."""
    assert header.startswith(">"), "data header must start in column zero"
    names, numbers = [], []
    depth, offset = 0, 1
    while offset < len(header):
        character = header[offset]
        if character == "(":
            depth += 1
        elif character == ")":
            assert depth, "unmatched registry parenthesis"
            depth -= 1
        elif depth == 0 and character == "<":
            end = header.find(">", offset + 1)
            assert end != -1, "unterminated field name"
            name = header[offset + 1:end]
            assert re.fullmatch(r"[A-Za-z][A-Za-z0-9_]*", name), "invalid field name"
            names.append(name)
            offset = end + 1
            continue
        elif depth == 0 and character == ">":
            raise AssertionError("unmatched field-name close")
        elif depth == 0 and header.startswith("DT", offset):
            before = header[offset - 1]
            match = re.match(r"DT([0-9]+)(?![A-Za-z0-9_])", header[offset:])
            if not (before.isalnum() or before == "_") and match:
                numbers.append(match.group(1))
                offset += len(match.group(0))
                continue
        offset += 1
    assert depth == 0, "unclosed registry parenthesis"
    assert len(names) <= 1 and len(numbers) <= 1, "ambiguous field identity"
    assert names or numbers, "header has no supported field identity"
    return (names[0] if names else None, numbers[0] if numbers else None)


def scan(source):
    """Report exact byte ranges and ordered items, without a chemical parser."""
    assert 0 < len(source) <= MAX_BYTES
    assert all(byte in (10, 13) or 32 <= byte <= 126 for byte in source), "ASCII slice only"
    spans = []
    for match in re.finditer(rb"[^\n]*\n|[^\n]+$", source):
        assert len(spans) < MAX_LINES, "line budget"
        spans.append(match.span())

    def content(index):
        start, end = spans[index]
        line = source[start:end]
        if line.endswith(b"\n"):
            line = line[:-1]
            if line.endswith(b"\r"):
                line = line[:-1]
        assert b"\r" not in line and len(line) <= MAX_LINE_BYTES, "line/encoding bound"
        return line

    records, offset, field_count = [], 0, 0
    while offset < len(spans):
        assert len(records) < MAX_RECORDS
        begin = offset
        assert offset + 4 <= len(spans), "truncated Molfile header"
        assert content(offset + 3)[33:39] == b" V2000", "V2000 oracle subset"
        offset += 4
        while offset < len(spans) and content(offset).rstrip(b" ") != b"M  END":
            assert not content(offset).startswith(b"$$$$"), "delimiter before M END"
            offset += 1
        assert offset < len(spans), "missing M END"
        mol_end = spans[offset][1]
        offset += 1
        fields = []
        while offset < len(spans) and not content(offset).startswith(b"$$$$"):
            if content(offset) == b"":
                assert fields, "padding before the first field/structure-only delimiter"
                offset += 1
                continue
            field_count += 1
            assert field_count <= MAX_FIELDS
            item_start = spans[offset][0]
            header = content(offset).decode("ascii")
            name, number = header_identity(header)
            header_range = list(spans[offset])
            offset += 1
            value_start = header_range[1]
            values = []
            while offset < len(spans) and content(offset) != b"":
                line = content(offset)
                assert not line.startswith(b"$$$$"), "missing blank before delimiter"
                assert len(line) <= 200, "data line exceeds 200 ASCII characters"
                values.append(line.decode("ascii"))
                offset += 1
            assert offset < len(spans), "unterminated data item"
            value_end = spans[offset][0]
            fields.append({
                "range": [item_start, spans[offset][1]],
                "header_range": header_range,
                "value_range": [value_start, value_end],
                "header": header,
                "name": name,
                "dt_number": number,
                "raw_value": source[value_start:value_end].decode("ascii"),
                "value_lines": values,
            })
            offset += 1
        assert offset < len(spans), "missing final record delimiter"
        delimiter_range = list(spans[offset])
        records.append({
            "range": [spans[begin][0], delimiter_range[1]],
            "mol_range": [spans[begin][0], mol_end],
            "delimiter_range": delimiter_range,
            "fields": fields,
        })
        offset += 1
    assert records and records[-1]["range"][1] == len(source)
    return records


def describe(source):
    records = scan(source)
    for record in records:
        begin, end = record["mol_range"]
        molecule = Chem.MolFromMolBlock(source[begin:end].decode("ascii"),
                                       sanitize=True, removeHs=False, strictParsing=True)
        assert molecule is not None, "independent consumer rejected the Molfile"
        conformer = molecule.GetConformer()
        record["rdkit"] = {
            "smiles": Chem.MolToSmiles(molecule),
            "atoms": [{"element": atom.GetAtomicNum(), "charge": atom.GetFormalCharge(),
                       "isotope": atom.GetIsotope(), "hydrogens": atom.GetTotalNumHs(),
                       "radical_electrons": atom.GetNumRadicalElectrons(),
                       "position": list(conformer.GetAtomPosition(atom.GetIdx()))}
                      for atom in molecule.GetAtoms()],
            "bonds": [{"begin": bond.GetBeginAtomIdx(), "end": bond.GetEndAtomIdx(),
                       "order": bond.GetBondTypeAsDouble()} for bond in molecule.GetBonds()],
        }
    supplier = Chem.ForwardSDMolSupplier(io.BytesIO(source), sanitize=True,
                                        removeHs=False, strictParsing=True)
    supplier.SetProcessPropertyLists(False)
    # Keep None entries and lost/overwritten properties visible; never zip or filter.
    consumer = [None if molecule is None else {
        "smiles": Chem.MolToSmiles(molecule),
        "properties": {key: molecule.GetProp(key) for key in
                       molecule.GetPropNames(includePrivate=False, includeComputed=False)},
    } for molecule in supplier]
    return {"source_sha256": hashlib.sha256(source).hexdigest(),
            "records": records, "rdkit_records": consumer}


def emit():
    molecule = Chem.MolFromSmiles("CO")
    assert molecule is not None
    molecule.SetProp("EMPTY", "")
    molecule.SetProp("MULTI", "one\ntwo")
    molecule.SetProp("ORDINARY", "original external vector")
    output = io.StringIO()
    writer = Chem.SDWriter(output)
    writer.SetProps(["EMPTY", "MULTI", "ORDINARY"])
    writer.SetForceV3000(False)
    writer.write(molecule)
    writer.flush()
    source = output.getvalue().encode("ascii")
    writer.close()
    return source


def self_test():
    mol = (b"Original SDF record\n  Bondline research  2D\nOriginal MIT OR Apache-2.0 vector\n"
           b"  1  0  0  0  0  0            999 V2000\n"
           b"    0.0000    0.0000    0.0000 C   0  0  0  0  0  4  0  0  0  0  0  0\nM  END\n")
    source = mol + b"> <EMPTY>\n\n> <DUP>\nfirst\n\n> <DUP>\nsecond\n\n> <MULTI>\nline one\nline two\n\n$$$$\n"
    assert len(source) == 271
    assert hashlib.sha256(source).hexdigest() == "04b62164b7cf86dc96a5f76b7437a9410a3e1199f8296da52dbaedaa321c9be6"
    result = describe(source)
    record = result["records"][0]
    assert record["range"] == [0, 271] and record["mol_range"] == [0, 195]
    assert record["delimiter_range"] == [266, 271]
    assert [(field["name"], field["value_lines"]) for field in record["fields"]] == [
        ("EMPTY", []), ("DUP", ["first"]), ("DUP", ["second"]), ("MULTI", ["line one", "line two"])]
    assert result["rdkit_records"][0]["properties"]["DUP"] == "second"
    assert record["fields"][0]["header_range"] == [195, 205]
    assert record["fields"][0]["value_range"] == [205, 205]
    assert record["fields"][0]["range"] == [195, 206]
    assert record["fields"][3]["raw_value"] == "line one\nline two\n"
    assert header_identity("> 7(REF-DT42)<MASS>DT00019") == ("MASS", "00019")
    assert header_identity("> DT12") == (None, "12")
    for header in ["> (DT123)", "> WORDT7", "> <HAS-DASH>", "> <A><B>", "> DT1 DT2"]:
        try:
            header_identity(header)
        except AssertionError:
            continue
        raise AssertionError("invalid/ambiguous header accepted")
    for bad in [mol + b"\n$$$$\n", source[:-5], source + b"\n",
                mol + b"> <X>\nvalue\n$$$$\n", mol + b"> <X>\n" + b"a" * 201 + b"\n\n$$$$\n"]:
        try:
            scan(bad)
        except AssertionError:
            continue
        raise AssertionError("malformed/incomplete source accepted")
    crlf = source.replace(b"\n", b"\r\n")
    crlf_record = scan(crlf)[0]
    assert crlf_record["range"] == [0, 290]
    assert crlf_record["fields"][3]["raw_value"] == "line one\r\nline two\r\n"
    assert [record["range"] for record in scan((mol + b"$$$$\n") * 2)] == [[0, 200], [200, 400]]
    assert scan(mol + b"> <X>\n" + b"a" * 200 + b"\n\n$$$$\n")[0]["fields"][0]["value_lines"] == ["a" * 200]
    generated = describe(emit())
    assert generated["records"][0]["fields"][0]["value_lines"] == []
    return {"self_test": "passed"}


if __name__ == "__main__":
    assert rdBase.rdkitVersion == "2026.09.2", "use the pinned BONDLINE_PYTHON interpreter"
    mode = sys.argv[1] if len(sys.argv) == 2 else "describe"
    assert len(sys.argv) <= 2 and mode in ("describe", "emit", "self-test")
    if mode == "emit":
        sys.stdout.buffer.write(emit())
    else:
        result = self_test() if mode == "self-test" else describe(sys.stdin.buffer.read(MAX_BYTES + 1))
        sys.stdout.buffer.write(json.dumps(result, ensure_ascii=True, allow_nan=False,
                                          separators=(",", ":")).encode("ascii") + b"\n")
