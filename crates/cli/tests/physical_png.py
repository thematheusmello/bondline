"""Run the native file command, then inspect its published bytes outside Bondline."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile

cli = sys.argv[1]
root = Path(__file__).resolve().parents[3]
source = root / "crates/io/tests/fixtures/cdx-skeletal/propane.cdx"
def run(*commands):
    args = [cli, "run"]
    for name, params in commands:
        args += ["--cmd", json.dumps({"command": name, "params": params})]
    return subprocess.run(args, capture_output=True)
opened = ("document.open_cdx", {"path": str(source)})
with tempfile.TemporaryDirectory(prefix="bondline-published-page-") as work:
    path = Path(work) / "図 β.png"
    def export(revision):
        return ("file.export_png", {"expected_revision": revision, "page_id": 1, "dpi": 300,
                                   "transparent": True, "output_path": str(path)})
    stale = run(opened, export(0))
    assert stale.returncode != 0 and not path.exists()
    result = run(opened, export(1))
    assert result.returncode == 0, result.stderr
    state = json.loads(result.stdout)
    receipt = state["png_export"]
    assert receipt["output"]["path"] == str(path.resolve())
    assert receipt["output"]["byte_length"] == path.stat().st_size
    assert (receipt["page_id"], receipt["dpi"], receipt["transparent"], receipt["input_revision"]) == (1, 300, True, 1)
    before = json.loads(run(opened).stdout)
    for key in ("revision", "undo_depth", "redo_depth", "document", "file"):
        assert state[key] == before[key], key
    data = path.read_bytes()
    subprocess.run([sys.executable, "-B", str(root/"crates/io/tests/published_png_oracle.py"), "300", "true"], input=data, check=True)
    path.write_bytes(b"pre-existing user output")
    exists = run(opened, export(1))
    assert exists.returncode != 0 and path.read_bytes() == b"pre-existing user output"
