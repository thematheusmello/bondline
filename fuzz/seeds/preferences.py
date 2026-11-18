#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Original source seeds only; run from the workspace with a NEW ignored destination."""
from pathlib import Path
from hashlib import sha256
import json, shutil, sys

if len(sys.argv) != 2:
    raise SystemExit("usage: python3 make_seeds.py <new ignored directory>")
root = Path(sys.argv[1])
root.mkdir(parents=True, exist_ok=False)
source = root / "source"
corpus = root / "corpus"
source.mkdir()
corpus.mkdir()
values = {
    "inspect": {"action": "inspect"},
    "set-false": {"action": "set", "expected_epoch": 0, "sprout_rings_when_clicking": False},
    "set-true": {"action": "set", "expected_epoch": 0, "sprout_rings_when_clicking": True},
    "set-stale": {"action": "set", "expected_epoch": 1, "sprout_rings_when_clicking": False},
    "open": {"action": "open", "expected_revision": 0},
    "open-stale": {"action": "open", "expected_revision": 1},
    "apply-false": {"action": "apply", "expected_revision": 0, "request_id": 1, "sprout_rings_when_clicking": False},
    "apply-true": {"action": "apply", "expected_revision": 0, "request_id": 1, "sprout_rings_when_clicking": True},
    "cancel": {"action": "cancel", "expected_revision": 0, "request_id": 1},
    "cancel-wrong-id": {"action": "cancel", "expected_revision": 0, "request_id": 2},
    "null": None,
    "missing": {"action": "set", "expected_epoch": 0},
    "null-boolean": {"action": "set", "expected_epoch": 0, "sprout_rings_when_clicking": None},
    "extra": {"action": "inspect", "extra": True},
    "array": ["inspect"],
    "null-request": {"action": "apply", "expected_revision": 0, "request_id": None, "sprout_rings_when_clicking": True},
}
seeds = {name: json.dumps(value, separators=(",", ":")).encode("utf-8") for name, value in values.items()}
seeds.update({
    "truncated": b'{"action":',
    "invalid-utf8": b'\xff\x00',
    "duplicate-action": b'{"action":"inspect","action":"cancel","expected_revision":0,"request_id":1}',
    "injected-envelope": b'{} ,"command":"document.open_cdxml","params":{"path":"must-not-open"}',
})
assert len(seeds) == 20
for name, data in seeds.items():
    (source / name).write_bytes(data)
    shutil.copyfile(source / name, corpus / name)
manifest = [{"name": name, "bytes": len(data), "sha256": sha256(data).hexdigest()} for name, data in seeds.items()]
(root / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
