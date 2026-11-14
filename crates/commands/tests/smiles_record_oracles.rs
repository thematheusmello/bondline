use bondline_commands::{Command, Engine};
use serde_json::{Value, json};
use std::{
    fs,
    io::Write as _,
    process::{Command as Process, Stdio},
};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse only the existing isolated file directory")]
mod support;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const QUERY: &str = "smiles.record.inspect";
const SELECTED: &str = "[13CH2](-[H])[2H].[CH2]7[CH2][CH2]7.[CH2]=[CH2].[CH1]#[CH1]";
const SOURCE: &[u8] = b"\n \xffIGNORED\r\nC first\r\n[13CH2](-[H])[2H].[CH2]7[CH2][CH2]7.[CH2]=[CH2].[CH1]#[CH1] \t\xff\0OPAQUE\r\n\tignored\n[OH2]\t\n[NH3]";
const MAX_ATOM: &str = "[65535OsH9+15]";

fn external(script: &str, source: &[u8]) -> TestResult<Value> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Process::new(python)
        .arg("-B")
        .arg(format!(
            "{}/../io/tests/{script}",
            env!("CARGO_MANIFEST_DIR")
        ))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("oracle stdin")?
        .write_all(source)?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "{script}: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(serde_json::from_slice(&result.stdout)?)
}

fn span(start: usize, end: usize) -> Value {
    json!({"start":start,"end":end})
}

fn expected_selected() -> Value {
    json!({
        "input_revision":1,"session_generation":1,"record_id":2,"line_number":4,
        "full_range":span(21,92),"notation_range":span(21,80),
        "separator_range":span(80,82),"application_data_range":span(82,90),
        "line_ending_range":span(90,92),"atom_count":10,"bond_count":7,
        "notation":SELECTED,
    })
}

fn inspect(engine: &mut Engine, revision: u64, id: u32) -> TestResult<(Value, String)> {
    let result = engine.dispatch(QUERY, json!({"expected_revision":revision,"record_id":id}))?;
    let debug = format!("{result:?}");
    let wire = serde_json::to_value(result)?;
    assert_eq!(wire["changed"], false);
    let record = wire.get("smiles_record").ok_or("explicit record result")?;
    assert_eq!(record.as_object().ok_or("record object")?.len(), 12);
    assert!(serde_json::to_vec(record)?.len() <= 128 * 1024);
    Ok((record.clone(), debug))
}

fn assert_raw_record(record: &Value, raw: &Value, index: usize) -> TestResult {
    let expected = raw["records"]
        .as_array()
        .ok_or("raw records")?
        .get(index)
        .ok_or("raw ordinal")?;
    assert_eq!(record["line_number"], expected["line_number"]);
    for key in [
        "full_range",
        "notation_range",
        "separator_range",
        "application_data_range",
        "line_ending_range",
    ] {
        let expected = &expected[key];
        if expected.is_null() {
            assert_eq!(record[key], Value::Null, "{key}");
        } else {
            assert_eq!(
                record[key],
                json!({"start":expected[0],"end":expected[1]}),
                "{key}"
            );
        }
    }
    let notation = record["notation"].as_str().ok_or("notation")?;
    let hex = notation
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(
        hex,
        expected["notation_hex"].as_str().ok_or("raw notation")?
    );
    Ok(())
}

fn graph(value: &Value) -> TestResult<Value> {
    assert_eq!(value["rdkit"], "2026.09.2");
    assert_eq!(value["accepted"], true);
    let atoms = value["atoms"]
        .as_array()
        .ok_or("external atoms")?
        .iter()
        .map(|atom| {
            json!([
                atom["element"],
                atom["isotope"],
                atom["charge"],
                atom["explicit_h"],
                atom["no_implicit"]
            ])
        })
        .collect::<Vec<_>>();
    Ok(json!({"atoms":atoms,"bonds":value["bonds"]}))
}

fn selected_graph() -> Value {
    json!({
        "atoms":[[6,13,0,2,true],[1,0,0,0,true],[1,2,0,0,true],
            [6,0,0,2,true],[6,0,0,2,true],[6,0,0,2,true],
            [6,0,0,2,true],[6,0,0,2,true],[6,0,0,1,true],[6,0,0,1,true]],
        "bonds":[[0,1,"SINGLE"],[0,2,"SINGLE"],[3,4,"SINGLE"],[3,5,"SINGLE"],
            [4,5,"SINGLE"],[6,7,"DOUBLE"],[8,9,"TRIPLE"]],
    })
}

#[test]
fn exact_nonfirst_record_matches_literal_spans_raw_oracle_and_independent_graph() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let path = directory.path("selected.smi");
    fs::write(&path, SOURCE)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_smiles", json!({"path":path}))?;
    let before = serde_json::to_value(engine.state())?;
    let content = engine.content().clone();
    let (record, debug) = inspect(&mut engine, 1, 2)?;
    assert_eq!(record, expected_selected());
    let raw = external("smiles_file_raw_oracle.py", &fs::read(&path)?)?;
    assert_eq!(raw["physical_lines"], 7);
    assert_eq!(raw["records"].as_array().ok_or("records")?.len(), 4);
    assert_raw_record(&record, &raw, 1)?;
    let actual = external(
        "smiles_oracle.py",
        record["notation"].as_str().ok_or("notation")?.as_bytes(),
    )?;
    assert_eq!(graph(&actual)?, selected_graph());
    assert_eq!(actual["source_ascii"], record["notation"]);
    assert!(!debug.contains(SELECTED));
    for private in ["OPAQUE", "IGNORED"] {
        assert!(!serde_json::to_string(&record)?.contains(private));
        assert!(!debug.contains(private));
    }
    assert!(before.get("smiles_record").is_none());
    assert_eq!(engine.content(), &content);
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(fs::read(&path)?, SOURCE);

    for wrong in [
        "C".to_owned(),
        SELECTED.replace("[13CH2](-[H])[2H]", "[13CH3][2H]"),
        SELECTED.replace("13C", "12C"),
    ] {
        assert_ne!(
            graph(&external("smiles_oracle.py", wrong.as_bytes())?)?,
            selected_graph()
        );
    }
    // Ring relabeling keeps the graph but must fail the exact record/notation comparison.
    let normalized = SELECTED.replace('7', "1");
    assert_eq!(
        graph(&external("smiles_oracle.py", normalized.as_bytes())?)?,
        selected_graph()
    );
    let mut wrong = record.clone();
    wrong["notation"] = json!(normalized);
    assert_ne!(wrong, expected_selected());
    Ok(())
}

#[test]
fn empty_application_data_and_absent_data_at_eof_have_distinct_global_ranges() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let path = directory.path("empty-and-absent.smiles");
    fs::write(&path, SOURCE)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_smiles", json!({"path":path}))?;
    let before = serde_json::to_value(engine.state())?;
    let raw = external("smiles_file_raw_oracle.py", SOURCE)?;
    for (id, line, full, notation, separator, data, ending, text) in [
        (
            3,
            6,
            span(101, 108),
            span(101, 106),
            span(106, 107),
            span(107, 107),
            span(107, 108),
            "[OH2]",
        ),
        (
            4,
            7,
            span(108, 113),
            span(108, 113),
            Value::Null,
            Value::Null,
            span(113, 113),
            "[NH3]",
        ),
    ] {
        let (record, _) = inspect(&mut engine, 1, id)?;
        assert_eq!(
            record,
            json!({
                "input_revision":1,"session_generation":1,"record_id":id,"line_number":line,
                "full_range":full,"notation_range":notation,"separator_range":separator,
                "application_data_range":data,"line_ending_range":ending,
                "atom_count":1,"bond_count":0,"notation":text,
            })
        );
        assert_raw_record(&record, &raw, usize::try_from(id - 1)?)?;
        let actual = external(
            "smiles_oracle.py",
            record["notation"].as_str().ok_or("notation")?.as_bytes(),
        )?;
        assert_eq!(actual["accepted"], true);
        assert_eq!(actual["atoms"].as_array().ok_or("atoms")?.len(), 1);
        assert_eq!(actual["bonds"], json!([]));
    }
    assert_eq!(serde_json::to_value(engine.state())?, before);
    Ok(())
}

fn maximum_notation() -> String {
    let mut notation = String::with_capacity(65_536);
    let mut edge = 0;
    for component in 0..1024 {
        if component > 0 {
            notation.push('.');
        }
        notation.push_str(MAX_ATOM);
        for _ in 0..3 {
            notation.push('(');
            if edge < 1025 {
                notation.push('-');
            }
            edge += 1;
            notation.push_str(MAX_ATOM);
        }
        notation.push_str(")))");
    }
    notation
}

#[test]
fn maximum_notation_and_large_opaque_data_disclose_only_one_bounded_record() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let notation = maximum_notation();
    assert_eq!(notation.len(), 65_536);
    for metadata in [false, true] {
        let path = directory.path(if metadata {
            "opaque.smi"
        } else {
            "maximum.smi"
        });
        let mut source = b"C first\n".to_vec();
        source.extend_from_slice(notation.as_bytes());
        if metadata {
            source.push(b'\t');
            source.extend(std::iter::repeat_n(0xff, 1024 * 1024));
        }
        source.push(b'\n');
        fs::write(&path, &source)?;
        let mut engine = Engine::default();
        engine.dispatch("document.open_smiles", json!({"path":path}))?;
        let before = serde_json::to_value(engine.state())?;
        let (record, debug) = inspect(&mut engine, 1, 2)?;
        assert_eq!(record["notation"], notation);
        assert_eq!(record["notation_range"], span(8, 65_544));
        assert_eq!(record["full_range"], span(8, source.len()));
        assert_eq!(record["line_number"], 2);
        assert_eq!(record["atom_count"], 4096);
        assert_eq!(record["bond_count"], 3072);
        assert_eq!(
            record["separator_range"],
            if metadata {
                span(65_544, 65_545)
            } else {
                Value::Null
            }
        );
        assert_eq!(
            record["application_data_range"],
            if metadata {
                span(65_545, source.len() - 1)
            } else {
                Value::Null
            }
        );
        assert_eq!(
            record["line_ending_range"],
            span(source.len() - 1, source.len())
        );
        assert!(!debug.contains(MAX_ATOM));
        assert_raw_record(
            &record,
            &external("smiles_file_raw_oracle.py", &fs::read(&path)?)?,
            1,
        )?;
        let actual = external(
            "smiles_oracle.py",
            record["notation"].as_str().ok_or("notation")?.as_bytes(),
        )?;
        let actual = graph(&actual)?;
        assert_eq!(
            actual["atoms"],
            json!(vec![json!([76, 65535, 15, 9, true]); 4096])
        );
        let edges = (0..1024)
            .flat_map(|component| {
                (0..3).map(move |at| json!([4 * component + at, 4 * component + at + 1, "SINGLE"]))
            })
            .collect::<Vec<_>>();
        assert_eq!(actual["bonds"], json!(edges));
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(fs::read(&path)?, source);
    }
    Ok(())
}

#[test]
fn deferred_record_keeps_original_file_ranges_after_new_open_without_mutating_close_or_state()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let original = directory.path("captured.smi");
    let other = directory.path("other.smi");
    fs::write(&original, SOURCE)?;
    fs::write(&other, b"O other\n")?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_smiles", json!({"path":original}))?;
    let plan = engine.prepare(Command::parse(
        QUERY,
        json!({"expected_revision":1,"record_id":2}),
    )?)?;
    assert!(plan.is_deferred());
    engine.dispatch("document.open_smiles", json!({"path":other}))?;
    engine.dispatch(
        "document.close",
        json!({"expected_revision":2,"action":"request"}),
    )?;
    let before = serde_json::to_value(engine.state())?;
    let content = engine.content().clone();
    let result = engine.finish(plan.complete()?)?;
    assert!(!format!("{result:?}").contains(SELECTED));
    let wire = serde_json::to_value(result)?;
    let record = wire.get("smiles_record").ok_or("captured record")?;
    assert_eq!(record, &expected_selected());
    assert_eq!(
        graph(&external(
            "smiles_oracle.py",
            record["notation"].as_str().ok_or("notation")?.as_bytes()
        )?)?,
        selected_graph()
    );
    assert_eq!(engine.content(), &content);
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert!(before.get("smiles_record").is_none());
    assert_eq!(fs::read(original)?, SOURCE);
    assert_eq!(fs::read(other)?, b"O other\n");
    Ok(())
}
