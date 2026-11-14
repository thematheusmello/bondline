use bondline_io::{
    mol::Version,
    rxn::{ComponentInput, Role, RxnFile, write_v2000},
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
use serde_json::{Value, json};
use std::{
    io::Write,
    ops::Range,
    process::{Command, Stdio},
};
const SOURCE: &[u8] = include_bytes!("fixtures/rxn/mapped-identity-v2000.rxn");
const MIXED: &[u8] = include_bytes!("fixtures/rxn/mixed-mol-v2000-wrapper.rxn");
fn external_raw(mode: &str, input: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(python)
        .arg("-B")
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/rxn_oracle.py"))
        .arg(mode)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("oracle stdin")?.write_all(input)?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "external RXN: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(result.stdout)
}
fn external(input: &[u8]) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(serde_json::from_slice(&external_raw("describe", input)?)?)
}
fn bounds(range: Range<usize>) -> [usize; 2] {
    [range.start, range.end]
}
fn role_name(role: Role) -> &'static str {
    match role {
        Role::Reactant => "reactant",
        Role::Product => "product",
        Role::Reagent => "reagent",
    }
}
fn check_independent(input: &[u8]) -> Result<Value, Box<dyn std::error::Error>> {
    let independent = external(input)?;
    let parsed = RxnFile::parse(input)?;
    assert_eq!(independent["header"], json!(parsed.header()));
    assert_eq!(
        independent["header_ranges"],
        json!(parsed.header_ranges().map(bounds))
    );
    assert_eq!(
        independent["counts_range"],
        json!(bounds(parsed.counts_range()))
    );
    let expected = independent["components"]
        .as_array()
        .ok_or("independent components")?;
    assert_eq!(parsed.components().len(), expected.len());
    for (component, other) in parsed.components().iter().zip(expected) {
        assert_eq!(other["role"], role_name(component.role()));
        assert_eq!(other["range"], json!(bounds(component.range())));
        assert_eq!(other["mol_range"], json!(bounds(component.mol_range())));
        assert_eq!(
            other["version"],
            match component.version() {
                Version::V2000 => "v2000",
                Version::V3000 => "v3000",
            }
        );
    }
    Ok(independent)
}
fn empty_mol() -> &'static [u8] {
    b"Original empty\nOriginal program\nOriginal metadata\n  0  0  0  0  0  0            999 V2000\nM  END\n"
}
fn wrapper(counts: &str, payloads: &[&[u8]]) -> Vec<u8> {
    let mut out = format!("$RXN\nOriginal vector\nOriginal program\nOriginal metadata\n{counts}\n")
        .into_bytes();
    for payload in payloads {
        out.extend_from_slice(b"$MOL\n");
        out.extend_from_slice(payload);
    }
    out
}

#[test]
fn published_counts_roles_names_maps_and_complete_mol_ranges_preserve_source_exactly() -> TestResult
{
    let file = RxnFile::parse(SOURCE)?;
    assert_eq!(file.as_bytes(), SOURCE);
    assert_eq!(file.as_bytes().as_ptr(), SOURCE.as_ptr());
    assert_eq!(file.container_version(), Version::V2000);
    assert_eq!(
        file.header(),
        [
            "Original mapped identity reaction",
            "  Bondline RXN research",
            "Original MIT OR Apache-2.0 vector"
        ]
    );
    let counts = file.counts();
    assert_eq!(
        (counts.reactants, counts.products, counts.reagents),
        (1, 1, 1)
    );
    assert_eq!(
        file.components()
            .iter()
            .map(|c| c.role())
            .collect::<Vec<_>>(),
        [Role::Reactant, Role::Product, Role::Reagent]
    );
    assert!(
        file.components()
            .iter()
            .all(|component| component.version() == Version::V2000)
    );
    let mut previous = file.counts_range().end;
    for component in file.components() {
        assert_eq!(component.range().start, previous);
        assert_eq!(
            SOURCE.get(component.mol_range()).ok_or("payload range")?,
            component.mol_bytes()
        );
        assert_eq!(
            SOURCE.get(component.range().start..component.mol_range().start),
            Some(b"$MOL\n".as_slice())
        );
        previous = component.range().end;
    }
    assert_eq!(previous, SOURCE.len());
    let inputs = file
        .components()
        .iter()
        .map(|c| ComponentInput {
            role: c.role(),
            mol_bytes: c.mol_bytes(),
        })
        .collect::<Vec<_>>();
    assert_eq!(write_v2000(file.header(), &inputs)?, SOURCE);
    Ok(())
}

#[test]
fn wrapper_overhead_cannot_exceed_the_total_output_byte_limit() -> TestResult {
    // Empty metadata adds5 signature bytes+3 header LFs+10 counts bytes+5 marker
    // bytes:23 bytes beyond the complete MOL payload, independently of our writer.
    let header = b"a\nb\nc\n  0  0  0  0  0  0            999 V2000\n";
    let size = 64 * 1024 * 1024 - 22;
    let filler = size - header.len() - b"M  END\n".len();
    let mut mol = header.to_vec();
    for _ in 0..(filler / 4096) {
        mol.extend_from_slice(&vec![b'x'; 4095]);
        mol.push(b'\n');
    }
    let remainder = filler % 4096;
    if remainder > 0 {
        mol.extend_from_slice(&vec![b'x'; remainder - 1]);
        mol.push(b'\n');
    }
    mol.extend_from_slice(b"M  END\n");
    assert_eq!(mol.len(), size);
    assert_eq!(size + 23, 64 * 1024 * 1024 + 1);
    let error = write_v2000(
        ["", "", ""],
        &[ComponentInput {
            role: Role::Reactant,
            mol_bytes: &mol,
        }],
    )
    .err()
    .ok_or("oversized output escaped preflight")?;
    assert_eq!(error.code(), "resource_limit");
    Ok(())
}

#[test]
fn independent_raw_spans_and_reaction_consumer_preserve_roles_maps_unknown_data_and_mixed_versions()
-> TestResult {
    assert_eq!(
        serde_json::from_slice::<Value>(&external_raw("self-test", b"")?)?["default_folded_roles"],
        json!([3, 1, 0])
    );
    let original = check_independent(SOURCE)?;
    assert_eq!(
        original["source_sha256"],
        "cadf07801a6516c416bc43823c81b3e2f690c85bc0614d80efa13f5e138093c8"
    );
    assert_eq!(original["counts"], json!([1, 1, 1]));
    assert_eq!(original["components"][0]["maps"], json!([1]));
    assert_eq!(original["components"][1]["maps"], json!([1]));
    assert_eq!(original["components"][2]["maps"], json!([0]));
    let parsed = RxnFile::parse(SOURCE)?;
    assert!(
        std::str::from_utf8(parsed.components().get(2).ok_or("reagent")?.mol_bytes())?
            .contains("M  XYZ  1   1   7")
    );
    assert!(
        !bondline_io::mol::MolFile::parse(
            parsed.components().first().ok_or("reactant")?.mol_bytes()
        )?
        .unsupported()
        .is_empty(),
        "a mapped component must not be passed through strict model projection"
    );
    let mixed = check_independent(MIXED)?;
    assert_eq!(
        mixed["source_sha256"],
        "5b8fca7c1c008a272680557ea04ca37f5e48d7b198b4412bd9ab8b3fe33b8403"
    );
    assert_eq!(mixed["counts"], json!([1, 1, 0]));
    assert_eq!(mixed["components"][0]["version"], "v3000");
    assert_eq!(mixed["components"][1]["version"], "v2000");
    let parsed = RxnFile::parse(MIXED)?;
    let inputs = parsed
        .components()
        .iter()
        .map(|c| ComponentInput {
            role: c.role(),
            mol_bytes: c.mol_bytes(),
        })
        .collect::<Vec<_>>();
    let canonical = write_v2000(parsed.header(), &inputs)?;
    assert_eq!(
        canonical.len(),
        MIXED.len() + 3,
        "canonical writer includes the omitted zero reagent count"
    );
    let written = check_independent(&canonical)?;
    for (before, after) in mixed["components"]
        .as_array()
        .ok_or("original")?
        .iter()
        .zip(written["components"].as_array().ok_or("written")?)
    {
        assert_eq!(before["mol_sha256"], after["mol_sha256"]);
    }
    Ok(())
}

#[test]
fn actual_external_writers_with_separate_agents_and_thousand_atom_mixed_payloads_are_eligible()
-> TestResult {
    for mode in ["emit", "emit_mixed"] {
        let bytes = external_raw(mode, b"")?;
        let independent = check_independent(&bytes)?;
        let file = RxnFile::parse(&bytes)?;
        let inputs = file
            .components()
            .iter()
            .map(|c| ComponentInput {
                role: c.role(),
                mol_bytes: c.mol_bytes(),
            })
            .collect::<Vec<_>>();
        let encoded = write_v2000(file.header(), &inputs)?;
        let after = check_independent(&encoded)?;
        assert_eq!(after["consumer_roles"], independent["consumer_roles"]);
        for (a, b) in independent["components"]
            .as_array()
            .ok_or("before")?
            .iter()
            .zip(after["components"].as_array().ok_or("after")?)
        {
            assert_eq!(a["mol_sha256"], b["mol_sha256"]);
        }
        if mode == "emit_mixed" {
            assert_eq!(
                independent["components"][0]["elements"]
                    .as_array()
                    .ok_or("large graph")?
                    .len(),
                1000
            );
            assert_eq!(file.container_version(), Version::V2000);
            assert_eq!(
                file.components().first().ok_or("first")?.version(),
                Version::V3000
            );
        }
    }
    Ok(())
}

#[test]
fn fixed_metadata_context_crlf_and_unknown_constituents_are_not_split_or_normalized() -> TestResult
{
    let original = RxnFile::parse(SOURCE)?;
    let payload = original
        .components()
        .first()
        .ok_or("component")?
        .mol_bytes();
    // Free MOL metadata and an opaque property may equal wrapper markers.
    let rest = std::str::from_utf8(payload)?
        .lines()
        .skip(3)
        .collect::<Vec<_>>()
        .join("\n");
    let tricky = format!(
        "$MOL\nM  END\n$RXN V3000\n{}\n",
        rest.replace("M  END", "$MOL\nM  END")
    );
    assert!(!tricky.is_empty());
    let crlf = tricky.replace('\n', "\r\n");
    let encoded = write_v2000(
        ["$MOL", "M  END", "$RXN V3000"],
        &[ComponentInput {
            role: Role::Product,
            mol_bytes: crlf.as_bytes(),
        }],
    )?;
    let parsed = RxnFile::parse(&encoded)?;
    assert_eq!(parsed.header(), ["$MOL", "M  END", "$RXN V3000"]);
    assert_eq!(
        parsed.components().first().ok_or("component")?.mol_bytes(),
        crlf.as_bytes()
    );
    assert_eq!(parsed.counts().products, 1);
    let all_crlf = std::str::from_utf8(SOURCE)?.replace('\n', "\r\n");
    check_independent(all_crlf.as_bytes())?;
    let no_lf = empty_mol().strip_suffix(b"\n").ok_or("MOL LF")?;
    let encoded = write_v2000(
        ["", "", ""],
        &[ComponentInput {
            role: Role::Reactant,
            mol_bytes: no_lf,
        }],
    )?;
    assert_eq!(
        RxnFile::parse(&encoded)?
            .components()
            .first()
            .ok_or("component")?
            .mol_bytes(),
        empty_mol()
    );
    Ok(())
}

#[test]
fn strict_counts_complete_eof_and_format_boundaries_refuse_consumer_data_loss_cases() -> TestResult
{
    let source = std::str::from_utf8(SOURCE)?;
    for bad in [
        source.replace("  1  1  1\n", "  1  1\n"),
        format!("{source}ignored\n"),
        format!("{source}\n"),
        source.replace("  1  1  1", "  2  1  1"),
        source.replace("$MOL\n", " $MOL\n"),
        source.replace("  1  1  1", " -1  1  1"),
        source.replace("  1  1  1", "1e1  1  1"),
        source.replace("  1  1  1", "  1  1 1"),
        source.replace('\n', "\r\r\n"),
    ] {
        assert!(RxnFile::parse(bad.as_bytes()).is_err());
    }
    for end in 0..SOURCE.len().saturating_sub(2) {
        assert!(
            RxnFile::parse(SOURCE.get(..end).ok_or("prefix")?).is_err(),
            "accepted truncated file {end}"
        );
    }
    assert!(RxnFile::parse(&[SOURCE, b"\0"].concat()).is_err());
    assert!(RxnFile::parse(&[SOURCE, b"\xff"].concat()).is_err());
    assert_eq!(
        RxnFile::parse(source.replacen("$RXN", "$RXN V3000", 1).as_bytes())
            .err()
            .ok_or("V3 wrapper")?
            .code(),
        "unsupported_content"
    );
    let padded = [empty_mol(), b"\n"].concat();
    assert!(bondline_io::mol::MolFile::parse(&padded).is_ok());
    assert_eq!(
        write_v2000(
            ["", "", ""],
            &[ComponentInput {
                role: Role::Reactant,
                mol_bytes: &padded
            }]
        )
        .err()
        .ok_or("padding trimmed")?
        .code(),
        "unsupported_content"
    );
    assert!(write_v2000(["name\n$MOL", "", ""], &[]).is_err());
    assert!(
        write_v2000(
            ["", "", ""],
            &[
                ComponentInput {
                    role: Role::Product,
                    mol_bytes: empty_mol()
                },
                ComponentInput {
                    role: Role::Reactant,
                    mol_bytes: empty_mol()
                }
            ]
        )
        .is_err()
    );
    let empty = write_v2000(["", "", ""], &[])?;
    assert_eq!(empty, b"$RXN\n\n\n\n  0  0  0\n");
    check_independent(&empty)?;
    Ok(())
}

#[test]
fn component_count_and_physical_resource_limits_are_explicit_and_checked_before_allocating_from_counts()
-> TestResult {
    let components = (0..1024)
        .map(|index| ComponentInput {
            role: if index < 999 {
                Role::Reactant
            } else {
                Role::Product
            },
            mol_bytes: empty_mol(),
        })
        .collect::<Vec<_>>();
    let maximum = write_v2000(["", "", ""], &components)?;
    let parsed = RxnFile::parse(&maximum)?;
    assert_eq!(
        (
            parsed.counts().reactants,
            parsed.counts().products,
            parsed.components().len()
        ),
        (999, 25, 1024)
    );
    assert_eq!(
        RxnFile::parse(&wrapper("999 26  0", &[]))
            .err()
            .ok_or("declared components")?
            .code(),
        "resource_limit"
    );
    assert_eq!(
        write_v2000(
            ["", "", ""],
            &vec![
                ComponentInput {
                    role: Role::Reactant,
                    mol_bytes: empty_mol()
                };
                1000
            ]
        )
        .err()
        .ok_or("role overflow")?
        .code(),
        "resource_limit"
    );
    let title = "x".repeat(4096);
    let encoded = write_v2000([&title, "", ""], &[])?;
    assert_eq!(
        RxnFile::parse(&encoded)?.header().first(),
        Some(&title.as_str())
    );
    assert_eq!(
        write_v2000([&"x".repeat(4097), "", ""], &[])
            .err()
            .ok_or("header bound")?
            .code(),
        "resource_limit"
    );
    assert_eq!(
        RxnFile::parse(&vec![b' '; 64 * 1024 * 1024 + 1])
            .err()
            .ok_or("input bound")?
            .code(),
        "resource_limit"
    );
    let blank = wrapper("     1   ", &[empty_mol()]);
    assert!(
        RxnFile::parse(&blank).is_err(),
        "present counts need numeric evidence, not the separate MOL blank rule"
    );
    Ok(())
}

fn original_graph_mol(atoms: usize, with_cage_bonds: bool) -> Vec<u8> {
    let bond_count = if with_cage_bonds {
        atoms + atoms / 2
    } else {
        0
    };
    let mut text = format!(
        "Original resource graph\n  Bondline          2D\nOriginal MIT OR Apache-2.0\n  0  0  0  0  0  0            999 V3000\nM  V30 BEGIN CTAB\nM  V30 COUNTS {atoms} {bond_count} 0 0 0\nM  V30 BEGIN ATOM\n"
    );
    for index in 1..=atoms {
        text.push_str(&format!("M  V30 {index} C 0 0 0 0\n"));
    }
    text.push_str("M  V30 END ATOM\n");
    if with_cage_bonds {
        text.push_str("M  V30 BEGIN BOND\n");
        for index in 1..=atoms {
            text.push_str(&format!("M  V30 {index} 1 {index} {}\n", index % atoms + 1));
        }
        for index in 1..=atoms / 2 {
            text.push_str(&format!(
                "M  V30 {} 1 {index} {}\n",
                atoms + index,
                index + atoms / 2
            ));
        }
        text.push_str("M  V30 END BOND\n");
    }
    text.push_str("M  V30 END CTAB\nM  END\n");
    text.into_bytes()
}

#[test]
fn aggregate_molecular_bounds_apply_across_components_instead_of_resetting_per_molecule()
-> TestResult {
    let mol = original_graph_mol(1000, false);
    let maximum = write_v2000(
        ["", "", ""],
        &vec![
            ComponentInput {
                role: Role::Reactant,
                mol_bytes: &mol
            };
            100
        ],
    )?;
    assert_eq!(RxnFile::parse(&maximum)?.components().len(), 100);
    assert_eq!(
        write_v2000(
            ["", "", ""],
            &vec![
                ComponentInput {
                    role: Role::Reactant,
                    mol_bytes: &mol
                };
                101
            ]
        )
        .err()
        .ok_or("aggregate atom writer bound")?
        .code(),
        "resource_limit"
    );
    let refs = vec![mol.as_slice(); 101];
    let error = RxnFile::parse(&wrapper("101  0  0", &refs))
        .err()
        .ok_or("aggregate atom reader bound")?;
    assert_eq!(error.code(), "resource_limit");
    assert!(error.to_string().contains("atoms"));
    // A cycle plus diametrical edges gives1000 atoms/1500 distinct edges:67
    // molecules exceed100000 bonds while remaining below the atom budget.
    let cage = original_graph_mol(1000, true);
    let error = write_v2000(
        ["", "", ""],
        &vec![
            ComponentInput {
                role: Role::Reactant,
                mol_bytes: &cage
            };
            67
        ],
    )
    .err()
    .ok_or("aggregate bond writer bound")?;
    assert_eq!(error.code(), "resource_limit");
    assert!(error.to_string().contains("bonds"));
    let refs = vec![cage.as_slice(); 67];
    let error = RxnFile::parse(&wrapper(" 67  0  0", &refs))
        .err()
        .ok_or("aggregate bond reader bound")?;
    assert_eq!(error.code(), "resource_limit");
    assert!(error.to_string().contains("bonds"));
    Ok(())
}

#[test]
fn total_line_budget_is_independent_from_each_constituents_own_limit() -> TestResult {
    let text = std::str::from_utf8(empty_mol())?
        .replace("M  END", &format!("{}M  END", "X\n".repeat(99_990)));
    let maximum = write_v2000(
        ["", "", ""],
        &vec![
            ComponentInput {
                role: Role::Reactant,
                mol_bytes: text.as_bytes()
            };
            10
        ],
    )?;
    assert_eq!(RxnFile::parse(&maximum)?.components().len(), 10);
    assert_eq!(
        write_v2000(
            ["", "", ""],
            &vec![
                ComponentInput {
                    role: Role::Reactant,
                    mol_bytes: text.as_bytes()
                };
                11
            ]
        )
        .err()
        .ok_or("writer line budget")?
        .code(),
        "resource_limit"
    );
    let refs = vec![text.as_bytes(); 11];
    assert_eq!(
        RxnFile::parse(&wrapper(" 11  0  0", &refs))
            .err()
            .ok_or("reader line budget")?
            .code(),
        "resource_limit"
    );
    Ok(())
}
