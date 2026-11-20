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
    old = json.dumps(rows[:53], sort_keys=True, separators=(",", ":"), ensure_ascii=False)
    old_digest = ("8a25ff5a637e7e4bce0e81e9b41d9b72b0154090e6cefd7d09853b6e2552722b" if rows is cli
                  else "5d8d1358dac859440a50043fb899822f8605fa555857f7ce6be80cd4e3162d42")
    assert hashlib.sha256(old.encode()).hexdigest() == old_digest
    assert [r["name"] for r in rows[51:]] == ["analysis.formula", "ui.analysis", "analysis.properties", "file.export_png"]

epoch = {"type": "integer", "format": "uint64", "minimum": 0}
for i, title, description, other, value, read_only in [
    (51, "AnalysisFormula", "Read the Hill formula for the current selection or complete drawing. Reports skeletal CDX hydrogen defaults, unresolved hydrogens, partial fragments and unsupported source coverage.", "expected_selection_epoch", epoch, True),
    (53, "AnalysisProperties", "Read formula, molecular weight and monoisotopic exact mass for the whole drawing or selection. Partial fragments report uncapped contributions; unavailable mass data identify the element or isotope. Request elemental analysis for percent by weight or select one SDF molecule by record ID.", "expected_selection_epoch", epoch, True),
    (52, "UiAnalysis", "Show or hide the Analysis window for this session without changing the drawing or its history.", "visible", {"type": "boolean"}, False),
]:
    schema = {
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": title, "type": "object", "additionalProperties": False,
        "required": ["expected_revision", other],
        "properties": {"expected_revision": epoch, other: value},
    }
    if i == 53:
        schema["properties"]["include_elemental_analysis"] = {"type": "boolean", "default": False}
        schema["properties"]["sdf_record_id"] = {"type": ["integer", "null"], "format": "uint32", "minimum": 1}
    name = ["analysis.formula", "ui.analysis", "analysis.properties"][i - 51]
    assert cli[i] == {"name": name, "description": description, "read_only": read_only, "input_schema": schema}
    assert mcp[i] == {"name": name, "description": description, "inputSchema": schema,
                      "annotations": {"readOnlyHint": read_only, "openWorldHint": False}}

png_description = "Publish one complete drawing page as a new physical-resolution PNG file with opaque white or transparent paper. Preserve the document and its editable file association."
png_schema = {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "title": "ExportPng",
    "type": "object",
    "properties": {
        "expected_revision": epoch,
        "page_id": {"type": "integer", "format": "uint32", "minimum": 1},
        "dpi": {"type": "integer", "format": "uint16", "minimum": 1, "maximum": 65535},
        "transparent": {"type": "boolean"},
        "output_path": {"type": "string"},
    },
    "additionalProperties": False,
    "required": ["expected_revision", "page_id", "dpi", "transparent", "output_path"],
}
assert cli[54] == {"name": "file.export_png", "description": png_description,
                   "read_only": False, "input_schema": png_schema}
assert mcp[54] == {"name": "file.export_png", "description": png_description,
                   "inputSchema": png_schema,
                   "annotations": {"readOnlyHint": False, "openWorldHint": False}}
