"""Compare both wire registries, including the exact ordered pre-analysis prefix."""
import hashlib
import json
import sys

cli, mcp = json.load(sys.stdin)
for rows, expected in [
    (cli, "d2d0c2d06c61edb1af9f749ee1edbaf3430284fa8c2d96c01ba38fde8a8bd8dd"),
    (mcp, "beec5dc0f2b20aa596ff3d788ec63c4215b37e6ac61aacbb0a0f2c7f959a7d33"),
]:
    prefix = json.dumps(rows[:51], sort_keys=True, separators=(",", ":"), ensure_ascii=False)
    assert hashlib.sha256(prefix.encode()).hexdigest() == expected
    assert [r["name"] for r in rows[51:]] == ["analysis.formula", "ui.analysis"]

epoch = {"type": "integer", "format": "uint64", "minimum": 0}
for i, title, description, other, value, read_only in [
    (51, "AnalysisFormula", "Read the Hill formula for the current selection or complete drawing. Reports skeletal CDX hydrogen defaults, unresolved hydrogens, partial fragments and unsupported source coverage.", "expected_selection_epoch", epoch, True),
    (52, "UiAnalysis", "Show or hide the Analysis window for this session without changing the drawing or its history.", "visible", {"type": "boolean"}, False),
]:
    schema = {
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": title, "type": "object", "additionalProperties": False,
        "required": ["expected_revision", other],
        "properties": {"expected_revision": epoch, other: value},
    }
    name = ["analysis.formula", "ui.analysis"][i - 51]
    assert cli[i] == {"name": name, "description": description, "read_only": read_only, "input_schema": schema}
    assert mcp[i] == {"name": name, "description": description, "inputSchema": schema,
                      "annotations": {"readOnlyHint": read_only, "openWorldHint": False}}
