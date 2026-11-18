"""Original MIT OR Apache-2.0 test-only graph authoring; external Indigo writer."""
import hashlib
import importlib.metadata
import json
from pathlib import Path
import platform
import sys
import struct

from indigo import Indigo

BASE = Path(__file__).parent
assert Path(sys.prefix) == BASE / "venv", "use only the separate investigation venv"
assert importlib.metadata.version("epam.indigo") == "1.47.0"
source = (BASE / "input-graphs.json").read_bytes()
graphs = json.loads(source)["graphs"]
output = BASE / "generated-positioned"
output.mkdir(exist_ok=False)
results = []
version = None
for graph in graphs:
    name = graph["name"]
    assert name.isascii() and name.isalpha()
    attempts = []
    for repetition in (1, 2):
        session = Indigo()
        version = session.version()
        molecule = session.createMolecule()
        atoms = [molecule.addAtom(symbol) for symbol in graph["symbols"]]
        for begin, end, order in graph["bonds"]:
            atoms[begin].addBond(atoms[end], order)
        # Public layout initializes coordinate presence; overwrite ALL generated
        # positions with the original declared literals before serialization.
        molecule.layout()
        for atom, position in zip(atoms, graph["positions"], strict=True):
            atom.setXYZ(*position)
        readback = [list(atom.xyz()) for atom in atoms]
        native_float = lambda value: struct.unpack("<f", struct.pack("<f", value))[0]
        assert readback == [[native_float(value) for value in point] for point in graph["positions"]]
        destination = output / f"{name}-{repetition}.cdx"
        result = molecule.saveCdx(str(destination))
        data = destination.read_bytes()
        attempts.append({"path": str(destination), "save_result": result,
                         "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest(),
                         "cdx_magic": data.startswith(b"VjCD0100")})
    results.append({"name": name, "attempts": attempts,
                    "declared_xyz_float32_readback": readback,
                    "repeat_exact": Path(attempts[0]["path"]).read_bytes()
                    == Path(attempts[1]["path"]).read_bytes()})

distribution = importlib.metadata.distribution("epam.indigo")
native = []
for entry in distribution.files or []:
    if str(entry).endswith(".dylib"):
        path = Path(distribution.locate_file(entry))
        data = path.read_bytes()
        native.append({"package_path": str(entry), "bytes": len(data),
                       "sha256": hashlib.sha256(data).hexdigest()})
receipt = {"package": "epam.indigo", "version": "1.47.0", "runtime_version": version,
           "python": sys.version, "executable": sys.executable, "prefix": sys.prefix,
           "platform": platform.platform(), "input_sha256": hashlib.sha256(source).hexdigest(),
           "generator_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
           "native_libraries": native, "results": results,
           "source_limit": "These are newly authored graph inputs, not independently authored upstream drawings; no corpus admission."}
(BASE / "generation-positioned-result.json").write_text(json.dumps(receipt, indent=2) + "\n")
print(json.dumps(receipt, indent=2))
