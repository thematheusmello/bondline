use bondline_io::{
    mol::{MolFile, MolTransform},
    sdf::{DataItemInput, SdfFile, SdfRecordInput, write_v2000},
};
use bondline_model::Point2;
use serde_json::{Value, json};
use std::{
    io::Write,
    ops::Range,
    process::{Command, Stdio},
};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const SOURCE: &[u8] = include_bytes!("fixtures/sdf/ordered-fields-v2000.sdf");
const MOL: &[u8] = include_bytes!("fixtures/mol/ethane-v2000.mol");

fn oracle_raw(mode: &str, source: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(python)
        .arg("-B")
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/sdf_oracle.py"))
        .arg(mode)
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
        "external SDF oracle: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(result.stdout)
}

fn external(source: &[u8]) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(serde_json::from_slice(&oracle_raw("describe", source)?)?)
}

fn bounds(range: Range<usize>) -> [usize; 2] {
    [range.start, range.end]
}

fn span_table(file: &SdfFile<'_>) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    file.records()
        .iter()
        .map(|record| {
            let fields=record.fields().iter().map(|field|Ok(json!({
            "range":bounds(field.range()),"header_range":bounds(field.header_range()),
            "value_range":bounds(field.value_range()),"header":field.header(),
            "name":field.name(),"dt_number":field.dt_number(),
            "raw_value":std::str::from_utf8(field.raw_value())?,"value_lines":field.value_lines(),
        }))).collect::<Result<Vec<_>,Box<dyn std::error::Error>>>()?;
            Ok(
                json!({"range":bounds(record.range()),"mol_range":bounds(record.mol_range()),
            "delimiter_range":bounds(record.delimiter_range()),"fields":fields}),
            )
        })
        .collect()
}

fn assert_external_spans(source: &[u8]) -> Result<Value, Box<dyn std::error::Error>> {
    let independent = external(source)?;
    let mut expected = independent["records"]
        .as_array()
        .ok_or("independent records")?
        .clone();
    for record in &mut expected {
        record
            .as_object_mut()
            .ok_or("independent record")?
            .remove("rdkit");
    }
    assert_eq!(span_table(&SdfFile::parse(source)?)?, expected);
    Ok(independent)
}

#[test]
fn original_records_preserve_order_empty_multiline_duplicates_dt_and_all_source_spans() -> TestResult
{
    let file = SdfFile::parse(SOURCE)?;
    assert_eq!(file.as_bytes(), SOURCE);
    assert_eq!(file.as_bytes().as_ptr(), SOURCE.as_ptr());
    assert_eq!(file.records().len(), 2);
    let first = file.records().first().ok_or("first record")?;
    let second = file.records().get(1).ok_or("second record")?;
    assert_eq!(first.mol_bytes(), MOL);
    assert_eq!(first.range().start, 0);
    assert_eq!(first.range().end, second.range().start);
    assert_eq!(second.range().end, SOURCE.len());
    assert!(
        SOURCE
            .get(first.delimiter_range())
            .ok_or("delimiter")?
            .starts_with(b"$$$$BONDLINE_RECORD\n")
    );
    assert_eq!(
        first
            .fields()
            .iter()
            .map(|field| field.name())
            .collect::<Vec<_>>(),
        vec![
            Some("ID"),
            Some("EMPTY"),
            Some("NOTES"),
            None,
            Some("DUP"),
            Some("DUP"),
            Some("SPACES")
        ]
    );
    assert_eq!(
        first.fields().get(2).ok_or("combined")?.dt_number(),
        Some("12")
    );
    assert_eq!(first.fields().get(3).ok_or("DT")?.dt_number(), Some("0009"));
    assert!(
        first
            .fields()
            .get(1)
            .ok_or("empty")?
            .value_lines()
            .is_empty()
    );
    assert_eq!(
        first.fields().get(2).ok_or("multiline")?.value_lines(),
        ["alpha", "beta"]
    );
    assert_eq!(
        first
            .fields()
            .get(4)
            .ok_or("first duplicate")?
            .value_lines(),
        ["one"]
    );
    assert_eq!(
        first.fields().get(5).ok_or("last duplicate")?.value_lines(),
        ["two"]
    );
    assert_eq!(
        first.fields().get(6).ok_or("spaces")?.value_lines(),
        ["   "]
    );
    for record in file.records() {
        assert_eq!(
            SOURCE.get(record.mol_range()).ok_or("mol range")?,
            record.mol_bytes()
        );
        for field in record.fields() {
            let header = SOURCE.get(field.header_range()).ok_or("header range")?;
            assert_eq!(header, format!("{}\n", field.header()).as_bytes());
            assert_eq!(field.header_range().end, field.value_range().start);
            assert_eq!(
                SOURCE.get(field.value_range()).ok_or("value range")?,
                field.raw_value()
            );
            assert_eq!(
                SOURCE.get(field.value_range().end..field.range().end),
                Some(b"\n".as_slice())
            );
        }
        let projected = MolFile::parse(record.mol_bytes())?
            .to_fragment(MolTransform::new(24.0, Point2::new(0.0, 0.0)?)?)?;
        assert_eq!(projected.atoms().len(), 2);
        assert_eq!(projected.bonds().len(), 1);
    }
    assert_eq!(
        MolFile::parse(second.mol_bytes())?.header().first(),
        Some(&"")
    );
    assert!(second.fields().is_empty());
    Ok(())
}

#[test]
fn writer_preserves_mol_bytes_and_raw_headers_and_writes_one_field_terminator() -> TestResult {
    let raw_mol = std::str::from_utf8(MOL)?
        .replace("M  END", "M  XYZ  1   1   7\nM  END")
        .replace('\n', "\r\n");
    let items = [
        DataItemInput {
            header: "> <EMPTY>",
            value_lines: &[],
        },
        DataItemInput {
            header: "> <DUP>",
            value_lines: &["one"],
        },
        DataItemInput {
            header: "> <DUP>",
            value_lines: &["two"],
        },
        DataItemInput {
            header: "> 7(BL-001)<NOTES>DT12",
            value_lines: &["alpha", "beta"],
        },
        DataItemInput {
            header: "> DT0009",
            value_lines: &["numbered"],
        },
    ];
    let encoded = write_v2000(&[SdfRecordInput {
        mol_bytes: raw_mol.as_bytes(),
        items: &items,
    }])?;
    let expected=[raw_mol.as_bytes(),b"> <EMPTY>\n\n> <DUP>\none\n\n> <DUP>\ntwo\n\n> 7(BL-001)<NOTES>DT12\nalpha\nbeta\n\n> DT0009\nnumbered\n\n$$$$\n"].concat();
    assert_eq!(
        encoded, expected,
        "new wrapper framing must not normalize or drop opaque MOL bytes"
    );
    let file = SdfFile::parse(&encoded)?;
    assert_eq!(
        file.records().first().ok_or("record")?.mol_bytes(),
        raw_mol.as_bytes()
    );
    assert!(
        MolFile::parse(raw_mol.as_bytes())?
            .to_fragment(MolTransform::new(24.0, Point2::new(0.0, 0.0)?)?)
            .is_err(),
        "SDF framing does not authorize a lossy molecule projection"
    );
    Ok(())
}

#[test]
fn incomplete_records_fields_non_ascii_and_unmarked_structure_padding_are_refused() -> TestResult {
    for bytes in [
        MOL.to_vec(),
        [MOL, b"> <VALUE>\nnot terminated\n$$$$\n"].concat(),
        [MOL, b"\n$$$$\n"].concat(),
        [MOL, b"\n> <VALUE>\na\n\n$$$$\n"].concat(),
        [MOL, b"$$$$\n\n"].concat(),
        [MOL, b"$$$$\nnot another complete molecule\n"].concat(),
        [MOL, b"> <BAD-NAME>\na\n\n$$$$\n"].concat(),
        [MOL, b"> (DT123)\na\n\n$$$$\n"].concat(),
        [MOL, b"> WORDT7\na\n\n$$$$\n"].concat(),
        [MOL, b"> <VALUE>\na\r\r\n\n$$$$\n"].concat(),
    ] {
        assert!(
            SdfFile::parse(&bytes).is_err(),
            "accepted malformed or unsupported framing"
        );
    }
    assert_eq!(
        SdfFile::parse(&[MOL, b"> <VALUE>\n\xff\n\n$$$$\n"].concat())
            .err()
            .ok_or("guessed encoding")?
            .code(),
        "unsupported_content"
    );
    assert!(SdfFile::parse(b"> <ONLY_DATA>\nvalue\n\n").is_err());
    assert!(SdfFile::parse(&[]).is_err());
    Ok(())
}

#[test]
fn complete_prefix_delimiters_and_header_looking_value_lines_are_preserved() -> TestResult {
    let source = [
        MOL,
        b"> <VALUE>\n> text\n> <OTHER>\nM  END\n $$$$\n\n$$$$suffix",
    ]
    .concat();
    let parsed = SdfFile::parse(&source)?;
    let field = parsed
        .records()
        .first()
        .ok_or("record")?
        .fields()
        .first()
        .ok_or("field")?;
    assert_eq!(
        field.value_lines(),
        ["> text", "> <OTHER>", "M  END", " $$$$"]
    );
    assert_eq!(parsed.as_bytes(), source);
    let crlf = std::str::from_utf8(SOURCE)?.replace('\n', "\r\n");
    assert_eq!(SdfFile::parse(crlf.as_bytes())?.as_bytes(), crlf.as_bytes());
    Ok(())
}

#[test]
fn writer_refuses_a_delimiter_prefix_inside_otherwise_valid_opaque_mol() -> TestResult {
    for index in [1, 2] {
        let mut lines = std::str::from_utf8(MOL)?
            .lines()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        *lines.get_mut(index).ok_or("MOL header")? = "$$$$original_metadata".to_owned();
        let mol = format!("{}\n", lines.join("\n"));
        assert!(
            MolFile::parse(mol.as_bytes()).is_ok(),
            "the standalone MOL header is valid"
        );
        assert!(
            write_v2000(&[SdfRecordInput {
                mol_bytes: mol.as_bytes(),
                items: &[]
            }])
            .is_err(),
            "a column-zero delimiter cannot be escaped, normalized or emitted inside a MOL block"
        );
    }
    let property = std::str::from_utf8(MOL)?.replace("M  END", "$$$$original_property\nM  END");
    assert!(MolFile::parse(property.as_bytes()).is_ok());
    assert!(
        write_v2000(&[SdfRecordInput {
            mol_bytes: property.as_bytes(),
            items: &[]
        }])
        .is_err()
    );
    Ok(())
}

#[test]
fn independent_scanner_checks_every_span_and_rdkit_reads_each_raw_molecule() -> TestResult {
    assert_eq!(
        serde_json::from_slice::<Value>(&oracle_raw("self-test", b"")?)?["self_test"],
        "passed"
    );
    let original = assert_external_spans(SOURCE)?;
    assert_eq!(SOURCE.len(), 690);
    assert_eq!(
        original["source_sha256"],
        "f95838cbb55c107c649ce2eb99ac62e191f747ce6994a171e64019f8e6c53d1a"
    );
    assert_eq!(original["records"][0]["range"], json!([0, 424]));
    assert_eq!(original["records"][1]["range"], json!([424, 690]));
    for record in original["records"]
        .as_array()
        .ok_or("independent records")?
    {
        assert_eq!(record["rdkit"]["smiles"], "CC");
        assert_eq!(record["rdkit"]["atoms"].as_array().ok_or("atoms")?.len(), 2);
        assert_eq!(
            record["rdkit"]["bonds"],
            json!([{"begin":0,"end":1,"order":1.0}])
        );
    }
    // This exact normative DT-only vector exposes a pinned consumer limitation:
    // its full supplier loses DT0009 and later data/record, despite both raw MOLs reading.
    // Do not confuse independent ordered-field fidelity with its last-value map.
    assert_eq!(
        original["rdkit_records"]
            .as_array()
            .ok_or("consumer records")?
            .len(),
        1
    );
    assert_eq!(
        original["rdkit_records"][0]["properties"],
        json!({"ID":"first","EMPTY":"","NOTES":"alpha\nbeta"})
    );
    let crlf = std::str::from_utf8(SOURCE)?.replace('\n', "\r\n");
    assert_external_spans(crlf.as_bytes())?;
    Ok(())
}

#[test]
fn named_item_writer_is_readable_by_external_full_supplier_without_losing_empty_or_multiline_values()
-> TestResult {
    let items = [
        DataItemInput {
            header: "> <EMPTY>",
            value_lines: &[],
        },
        DataItemInput {
            header: "> <MULTI>",
            value_lines: &["line one", "line two"],
        },
        DataItemInput {
            header: "> <DUP>",
            value_lines: &["first"],
        },
        DataItemInput {
            header: "> <DUP>",
            value_lines: &["second"],
        },
        DataItemInput {
            header: "> <SPACES>",
            value_lines: &["   "],
        },
    ];
    let encoded = write_v2000(&[
        SdfRecordInput {
            mol_bytes: MOL,
            items: &items,
        },
        SdfRecordInput {
            mol_bytes: MOL,
            items: &[],
        },
    ])?;
    let independent = assert_external_spans(&encoded)?;
    assert_eq!(
        independent["records"][0]["fields"]
            .as_array()
            .ok_or("ordered fields")?
            .len(),
        5
    );
    let consumer = independent["rdkit_records"]
        .as_array()
        .ok_or("consumer records")?;
    assert_eq!(consumer.len(), 2);
    assert_eq!(
        consumer.first().ok_or("first molecule")?["properties"],
        json!({"EMPTY":"","MULTI":"line one\nline two","DUP":"second","SPACES":"   "})
    );
    assert!(consumer.iter().all(|molecule| molecule["smiles"] == "CC"));
    Ok(())
}

#[test]
fn externally_generated_empty_value_padding_is_preserved_then_canonically_reframed() -> TestResult {
    let emitted = oracle_raw("emit", b"")?;
    let independent = assert_external_spans(&emitted)?;
    assert_eq!(independent["rdkit_records"][0]["smiles"], "CO");
    let parsed = SdfFile::parse(&emitted)?;
    assert_eq!(parsed.as_bytes(), emitted);
    let record = parsed.records().first().ok_or("record")?;
    let fields = record.fields();
    assert_eq!(fields.len(), 3);
    assert_eq!(
        fields.first().ok_or("EMPTY")?.value_lines(),
        [] as [&str; 0]
    );
    assert_eq!(fields.get(1).ok_or("MULTI")?.value_lines(), ["one", "two"]);
    let items = fields
        .iter()
        .map(|field| DataItemInput {
            header: field.header(),
            value_lines: field.value_lines(),
        })
        .collect::<Vec<_>>();
    let rewritten = write_v2000(&[SdfRecordInput {
        mol_bytes: record.mol_bytes(),
        items: &items,
    }])?;
    let rewritten_file = SdfFile::parse(&rewritten)?;
    let rewritten_record = rewritten_file.records().first().ok_or("rewritten record")?;
    assert_eq!(
        rewritten_record.mol_bytes(),
        record.mol_bytes(),
        "external MOL bytes are opaque to SDF writing"
    );
    let reread = assert_external_spans(&rewritten)?;
    assert_eq!(reread["rdkit_records"], independent["rdkit_records"]);
    for (before, after) in fields.iter().zip(rewritten_record.fields()) {
        assert_eq!(before.header(), after.header());
        assert_eq!(before.name(), after.name());
        assert_eq!(before.value_lines(), after.value_lines());
    }
    assert!(
        rewritten.len() < emitted.len(),
        "only external extra item padding is omitted by new framing"
    );
    Ok(())
}

#[test]
fn header_identities_are_unambiguous_and_registry_parentheses_are_complete() -> TestResult {
    let digits = "0".repeat(1000);
    let header = format!("> 17(REF-DT42)<FIELD_9>DT{digits} extra");
    let bytes = [MOL, header.as_bytes(), b"\nv\n\n$$$$\n"].concat();
    let file = SdfFile::parse(&bytes)?;
    let field = file
        .records()
        .first()
        .ok_or("record")?
        .fields()
        .first()
        .ok_or("field")?;
    assert_eq!(field.dt_number(), Some(digits.as_str()));
    assert_eq!(field.name(), Some("FIELD_9"));
    for header in [
        "> <A>(unfinished",
        "> <A>)",
        "> <A><B>",
        "> DT1 DT2",
        "> DT7suffix",
        "> <9NAME>",
    ] {
        let source = [MOL, header.as_bytes(), b"\nx\n\n$$$$\n"].concat();
        assert!(SdfFile::parse(&source).is_err(), "accepted {header:?}");
        assert!(
            write_v2000(&[SdfRecordInput {
                mol_bytes: MOL,
                items: &[DataItemInput {
                    header,
                    value_lines: &["x"]
                }]
            }])
            .is_err(),
            "wrote {header:?}"
        );
    }
    for header in ["> <A>(unfinished", "> <A>)"] {
        assert_eq!(
            SdfFile::parse(&[MOL, header.as_bytes(), b"\nx\n\n$$$$\n"].concat())
                .err()
                .ok_or("unsupported metadata")?
                .code(),
            "unsupported_content"
        );
    }
    Ok(())
}

#[test]
fn character_and_line_boundaries_are_checked_without_trimming_or_encoding_guesses() -> TestResult {
    for length in [199, 200] {
        let value = "v".repeat(length);
        let source = [MOL, b"> <VALUE>\n", value.as_bytes(), b"\n\n$$$$\n"].concat();
        assert_eq!(
            SdfFile::parse(&source)?
                .records()
                .first()
                .ok_or("record")?
                .fields()
                .first()
                .ok_or("field")?
                .value_lines(),
            [value.as_str()]
        );
        write_v2000(&[SdfRecordInput {
            mol_bytes: MOL,
            items: &[DataItemInput {
                header: "> <VALUE>",
                value_lines: &[value.as_str()],
            }],
        }])?;
    }
    let long = "v".repeat(201);
    let source = [MOL, b"> <VALUE>\n", long.as_bytes(), b"\n\n$$$$\n"].concat();
    assert_eq!(
        SdfFile::parse(&source).err().ok_or("long input")?.code(),
        "resource_limit"
    );
    assert_eq!(
        write_v2000(&[SdfRecordInput {
            mol_bytes: MOL,
            items: &[DataItemInput {
                header: "> <VALUE>",
                value_lines: &[long.as_str()]
            }]
        }])
        .err()
        .ok_or("long output")?
        .code(),
        "resource_limit"
    );
    for length in [4096, 4097] {
        let header = format!("> <A>{}", "x".repeat(length - 5));
        let source = [MOL, header.as_bytes(), b"\nx\n\n$$$$\n"].concat();
        let input = SdfFile::parse(&source);
        let output = write_v2000(&[SdfRecordInput {
            mol_bytes: MOL,
            items: &[DataItemInput {
                header: &header,
                value_lines: &["x"],
            }],
        }]);
        if length == 4096 {
            assert!(input.is_ok());
            assert!(output.is_ok());
        } else {
            assert_eq!(input.err().ok_or("header input")?.code(), "resource_limit");
            assert_eq!(
                output.err().ok_or("header output")?.code(),
                "resource_limit"
            );
        }
    }
    for value in [
        "",
        "line\nbreak",
        "line\rbreak",
        "$$$$prefix",
        "\0",
        "\u{7f}",
        "é",
    ] {
        assert!(
            write_v2000(&[SdfRecordInput {
                mol_bytes: MOL,
                items: &[DataItemInput {
                    header: "> <VALUE>",
                    value_lines: &[value]
                }]
            }])
            .is_err(),
            "accepted injection/unsupported value {value:?}"
        );
    }
    // A tab is preserved ASCII data in this profile, while spaces-only stays nonempty.
    let tabbed = [MOL, b"> <VALUE>\n\t\n   \n\n$$$$\n"].concat();
    assert_eq!(
        SdfFile::parse(&tabbed)?
            .records()
            .first()
            .ok_or("record")?
            .fields()
            .first()
            .ok_or("field")?
            .value_lines(),
        ["\t", "   "]
    );
    assert!(write_v2000(&[]).is_err());
    assert!(
        write_v2000(&[SdfRecordInput {
            mol_bytes: &[MOL, b"\n"].concat(),
            items: &[]
        }])
        .is_err(),
        "MOL padding is never trimmed away"
    );
    let no_final_lf = MOL.strip_suffix(b"\n").ok_or("final MOL LF")?;
    assert_eq!(
        write_v2000(&[SdfRecordInput {
            mol_bytes: no_final_lf,
            items: &[]
        }])?,
        [MOL, b"$$$$\n"].concat()
    );
    Ok(())
}

#[test]
fn record_and_field_budgets_accept_the_boundary_and_reject_the_next_item() -> TestResult {
    let molecule = [MOL, b"$$$$\n"].concat();
    let maximum_records = molecule.repeat(1024);
    assert_eq!(SdfFile::parse(&maximum_records)?.records().len(), 1024);
    assert_eq!(
        SdfFile::parse(&[maximum_records.as_slice(), molecule.as_slice()].concat())
            .err()
            .ok_or("record cap")?
            .code(),
        "resource_limit"
    );
    let record = SdfRecordInput {
        mol_bytes: MOL,
        items: &[],
    };
    assert_eq!(write_v2000(&vec![record; 1024])?, maximum_records);
    assert_eq!(
        write_v2000(&vec![record; 1025])
            .err()
            .ok_or("writer record cap")?
            .code(),
        "resource_limit"
    );

    let fields = b"> <X>\n\n".repeat(65536);
    let source = [MOL, fields.as_slice(), b"$$$$\n"].concat();
    assert_eq!(
        SdfFile::parse(&source)?
            .records()
            .first()
            .ok_or("record")?
            .fields()
            .len(),
        65536
    );
    let excess = [MOL, fields.as_slice(), b"> <X>\n\n$$$$\n"].concat();
    assert_eq!(
        SdfFile::parse(&excess).err().ok_or("field cap")?.code(),
        "resource_limit"
    );
    let item = DataItemInput {
        header: "> <X>",
        value_lines: &[],
    };
    assert_eq!(
        write_v2000(&[SdfRecordInput {
            mol_bytes: MOL,
            items: &vec![item; 65536]
        }])?,
        source
    );
    assert_eq!(
        write_v2000(&[SdfRecordInput {
            mol_bytes: MOL,
            items: &vec![item; 65537]
        }])
        .err()
        .ok_or("writer field cap")?
        .code(),
        "resource_limit"
    );
    // The field count is global, not reset for each molecule.
    let half = vec![item; 32769];
    assert_eq!(
        write_v2000(
            &[SdfRecordInput {
                mol_bytes: MOL,
                items: &half
            }; 2]
        )
        .err()
        .ok_or("global writer cap")?
        .code(),
        "resource_limit"
    );
    let half_source = [MOL, b"> <X>\n\n".repeat(32769).as_slice(), b"$$$$\n"].concat();
    assert_eq!(
        SdfFile::parse(&half_source.repeat(2))
            .err()
            .ok_or("global input cap")?
            .code(),
        "resource_limit"
    );
    Ok(())
}

#[test]
fn total_line_and_byte_budgets_bound_parsing_and_preflight_new_output() -> TestResult {
    let fixed_lines = MOL.iter().filter(|byte| **byte == b'\n').count() + 3;
    let padding = vec![b'\n'; 1_000_000 - fixed_lines];
    let source = [MOL, b"> <E>\n\n", padding.as_slice(), b"$$$$\n"].concat();
    assert_eq!(SdfFile::parse(&source)?.records().len(), 1);
    let over = [MOL, b"> <E>\n\n", padding.as_slice(), b"\n$$$$\n"].concat();
    assert_eq!(
        SdfFile::parse(&over).err().ok_or("line cap")?.code(),
        "resource_limit"
    );
    let lines = vec!["x"; 1_000_000];
    assert_eq!(
        write_v2000(&[SdfRecordInput {
            mol_bytes: MOL,
            items: &[DataItemInput {
                header: "> <E>",
                value_lines: &lines
            }]
        }])
        .err()
        .ok_or("writer line cap")?
        .code(),
        "resource_limit"
    );
    let oversized = vec![b' '; 64 * 1024 * 1024 + 1];
    assert_eq!(
        SdfFile::parse(&oversized).err().ok_or("byte cap")?.code(),
        "resource_limit"
    );
    let value = "x".repeat(200);
    let values = vec![value.as_str(); 64 * 1024 * 1024 / 201 + 1];
    assert_eq!(
        write_v2000(&[SdfRecordInput {
            mol_bytes: MOL,
            items: &[DataItemInput {
                header: "> <E>",
                value_lines: &values
            }]
        }])
        .err()
        .ok_or("writer byte cap")?
        .code(),
        "resource_limit"
    );
    Ok(())
}
