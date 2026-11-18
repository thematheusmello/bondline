# SPDX-License-Identifier: MIT OR Apache-2.0
"""Check actual ORIGINAL CDX files with independent struct facts, no product code."""

import argparse
import copy
import hashlib
import importlib.util
import json
import struct
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = next(p for p in HERE.parents if (p / "Cargo.toml").is_file())
spec = importlib.util.spec_from_file_location(
    "independent_caption_reader", HERE / "caption_oracle.py"
)
oracle = importlib.util.module_from_spec(spec)
spec.loader.exec_module(oracle)


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def require(condition, message):
    if not condition:
        raise ValueError(message)


def save_json(path, value):
    path.write_bytes(
        (json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode("utf-8")
    )


def model_drifts(model):
    mutations = [
        (
            "point_x",
            lambda value: value["pages"][0]["texts"][0]["position"].__setitem__(
                "x_pt", 81.25
            ),
        ),
        (
            "point_y",
            lambda value: value["pages"][0]["texts"][0]["position"].__setitem__(
                "y_pt", 99.75
            ),
        ),
        (
            "body",
            lambda value: value["pages"][0]["texts"][0]["runs"][0].__setitem__(
                "text", "Changed"
            ),
        ),
        (
            "size",
            lambda value: value["pages"][0]["texts"][0]["runs"][0]["style"].__setitem__(
                "size_pt", 12.05
            ),
        ),
        (
            "face",
            lambda value: value["pages"][0]["texts"][0]["runs"][0]["style"].__setitem__(
                "face", 1
            ),
        ),
        (
            "color",
            lambda value: value["pages"][0]["texts"][0]["runs"][0]["style"].__setitem__(
                "rgb", [0.0, 0.0, 1.0]
            ),
        ),
        (
            "page_width",
            lambda value: value["pages"][0]["size"].__setitem__("width_pt", 513.0),
        ),
        ("page_id", lambda value: value["pages"][0].__setitem__("id", 2)),
        ("text_id", lambda value: value["pages"][0]["texts"][0].__setitem__("id", 2)),
    ]
    rows = [
        {
            "name": "equal_model_clone",
            "model": copy.deepcopy(model),
            "equals_baseline": True,
            "planned_current_preview": True,
            "planned_current_save": True,
        }
    ]
    for name, mutation in mutations:
        value = copy.deepcopy(model)
        mutation(value)
        require(value != model, "declared model drift must change model")
        rows.append(
            {
                "name": name,
                "model": value,
                "equals_baseline": False,
                "planned_current_preview": False,
                "planned_current_save": False,
            }
        )
    return {
        "scope": "proposed immutable-source preview; no app/command execution",
        "source": "caption_ascii",
        "cases": rows,
        "same_model_source_replacement": {
            "first": "caption_ascii",
            "second": "caption_ascii_explicit",
            "models_equal": True,
            "source_bytes_equal": False,
            "old_source_bound_proof_valid": False,
        },
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input-dir", type=Path, default=HERE)
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=REPO / "target/cdx-resources/next-text/reproduced-caption-facts",
    )
    args = parser.parse_args()
    args.output_dir.mkdir(parents=True, exist_ok=True)
    manifest_raw = (args.input_dir / "caption-vectors.json").read_bytes()
    manifest = json.loads(manifest_raw)
    facts = {}
    for vector in manifest["cases"]:
        name = vector["name"]
        raw = (args.input_dir / vector["file"]).read_bytes()
        require(
            len(raw) == vector["byte_length"] and digest(raw) == vector["sha256"],
            "declared original bytes: " + name,
        )
        fact = oracle.read(raw)
        require(fact["source"]["sha256"] == vector["sha256"], "reader source identity")
        require(
            fact["preview"]["candidate"] == vector["preview_candidate"],
            "declared proposed eligibility: " + name,
        )
        if vector["preview_candidate"]:
            require(
                fact["model"] == vector["model"], "independent complete model: " + name
            )
            require(
                fact["identities"]["page"]["source_id"] == 90
                and fact["identities"]["text"]["source_id"] == 73
                and fact["identities"]["page"]["model_id"] == 1
                and fact["identities"]["text"]["model_id"] == 1,
                "source/app namespace: " + name,
            )
        else:
            require(
                fact["model"] is None and fact["preview"]["issues"],
                "fail closed proposed profile: " + name,
            )
        facts[name] = fact
    baseline = facts["caption_ascii"]["model"]
    require(
        facts["caption_ascii_explicit"]["model"] == baseline,
        "equal-model envelope pair",
    )
    require(
        facts["caption_ascii"]["source"]["sha256"]
        != facts["caption_ascii_explicit"]["source"]["sha256"],
        "different source identity pair",
    )
    require(
        facts["caption_ascii"]["source"]["sha256"]
        == "6e94388187d08ba852e7058938e2414c0f19cf1c0a42f168884e1610a4dd8660"
        and facts["caption_ascii_explicit"]["source"]["sha256"]
        == "31a26e5bb5a5ca764bb4a9bc29d99cc37ac6b3c287685e197a129f6d2ddf8bd2",
        "early positive pair remains immutable",
    )
    facts_path = args.output_dir / "caption-facts.json"
    save_json(facts_path, facts)
    drift_path = args.output_dir / "model-drift.json"
    save_json(drift_path, model_drifts(baseline))
    checks = {
        "scope": "original independent source/facts checks, NOT app acceptance",
        "source_count": len(facts),
        "planned_positive": len(manifest["positive_names"]),
        "planned_refused": len(manifest["negative_names"]),
        "all_raw_syntax_valid": True,
        "all_declared_models_match": True,
        "manifest_sha256": digest(manifest_raw),
        "facts_sha256": digest(facts_path.read_bytes()),
        "model_drift_sha256": digest(drift_path.read_bytes()),
        "reader_sha256": digest((HERE / "caption_oracle.py").read_bytes()),
        "generator_sha256": digest((HERE / "generate_captions.py").read_bytes()),
        "resource_reader_sha256": oracle.RESOURCE_SHA,
        "equal_model_source_pairs": [
            ["caption_ascii", name]
            for name in [
                "caption_ascii_explicit",
                "caption_ascii_header",
                "caption_ascii_implicit_file_end",
                "caption_ascii_explicit_file_end",
                "caption_extended",
                "caption_short_white",
                "caption_font_one",
            ]
        ],
        "rejected_with_same_nominal_model_as_base": [
            name
            for name in manifest["negative_names"]
            if facts[name]["nominal_model"] == baseline
        ],
    }
    for _, name in checks["equal_model_source_pairs"]:
        require(facts[name]["model"] == baseline, "declared equal-model pair: " + name)
    # Outer framing errors are different from valid syntax with refused preview.
    base_raw = (args.input_dir / "caption_ascii.cdx").read_bytes()
    raw_bad = {
        "empty": b"",
        "header_short": base_raw[:12],
        "header_magic": b"X" + base_raw[1:],
        "truncated_end": base_raw[:-1],
        "trailing_byte": base_raw + b"X",
        "property_length_beyond_source": base_raw[:30] + b"\xfe\xff" + base_raw[32:],
    }
    for name, data in raw_bad.items():
        try:
            oracle.read(data)
        except (ValueError, struct.error):
            pass
        else:
            raise ValueError("malformed framing must not return preview facts: " + name)
    checks["raw_invalid_controls"] = list(raw_bad)
    require(
        facts["invalid_utf8"]["strings"][0]["utf8"] is None,
        "no replacement decoding of malformed UTF8",
    )
    require(
        facts["embedded_nul"]["strings"][0]["utf8"] == "A\0B",
        "counted text is not NUL terminated",
    )
    require(
        facts["font_nul_name"]["resources"]["fonts"][0]["name"]["hex"]
        == b"Roboto\0".hex(),
        "raw font names are not NUL terminated",
    )
    save_json(args.output_dir / "independent-checks.json", checks)
    print(json.dumps(checks, indent=2))


if __name__ == "__main__":
    main()
