use bondline_io::files::export_smiles_fragment_file;
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Element, Fragment, FragmentId, Point2, Radical,
};
use bondline_render::Cancellation;
use std::{
    fs,
    io::Write as _,
    num::NonZeroU16,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
};

type TestResult = Result<(), Box<dyn std::error::Error>>;
struct Directory(PathBuf);
impl Directory {
    fn new() -> Result<Self, std::io::Error> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "bondline-smiles-export-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path)?;
        Ok(Self(path))
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn atom(id: u32, element: u8, hydrogens: u16) -> Result<Atom, Box<dyn std::error::Error>> {
    let mut value = Atom::new(
        AtomId::new(id).ok_or("atom ID")?,
        Element::new(element).ok_or("element")?,
        Some(Point2::new(-17.25, 999.125)?),
    );
    value.explicit_hydrogens = Some(hydrogens);
    Ok(value)
}
fn fragment(
    atoms: Vec<Atom>,
    edges: &[(u32, u32, BondOrder)],
) -> Result<Fragment, Box<dyn std::error::Error>> {
    let bonds = edges
        .iter()
        .enumerate()
        .map(|(index, &(left, right, order))| {
            Ok(Bond {
                double_position: bondline_model::DoublePosition::Unspecified,
                id: BondId::new(u32::try_from(index + 1)?).ok_or("bond ID")?,
                begin: AtomId::new(left).ok_or("begin")?,
                end: AtomId::new(right).ok_or("end")?,
                order,
            })
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    Ok(Fragment::new(
        FragmentId::new(19).ok_or("fragment ID")?,
        atoms,
        bonds,
    )?)
}
fn external(path: &Path) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let file = fs::read(path)?;
    assert!(file.len() <= bondline_io::smiles::MAX_SMILES_BYTES + 1);
    assert!(file.is_ascii());
    let notation = file.strip_suffix(b"\n").ok_or("missing final LF")?;
    assert!(!notation.is_empty());
    assert!(!notation.contains(&b'\n') && !notation.contains(&b'\r'));
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut process = Command::new(python)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/smiles_write_oracle.py"
        ))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    process
        .stdin
        .take()
        .ok_or("oracle stdin")?
        .write_all(notation)?;
    let output = process.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

#[test]
fn real_unicode_export_has_exact_literal_bytes_receipt_and_external_ethane_meaning() -> TestResult {
    let directory = Directory::new()?;
    let path = directory.0.join("export-化学.data");
    let input = fragment(
        vec![atom(1, 6, 3)?, atom(2, 6, 3)?],
        &[(1, 2, BondOrder::Single)],
    )?;
    let before = input.clone();
    let receipt = export_smiles_fragment_file(&path, &input, &Cancellation::default())?;
    assert_eq!(input, before);
    assert_eq!(fs::read(&path)?, b"[CH3][CH3]\n");
    assert_eq!(receipt.path, fs::canonicalize(&path)?);
    assert_eq!(receipt.byte_length, 11);
    let value = external(&path)?;
    assert_eq!(value["notation"]["source_ascii"], "[CH3][CH3]");
    assert_eq!(value["notation"]["source_bytes"], 10);
    assert_eq!(value["meaning"]["smiles"], "CC");
    assert_eq!(value["meaning"]["formula"], "C2H6");
    assert_eq!(value["meaning"]["radicals"], serde_json::json!([0, 0]));
    assert_eq!(value["meaning"]["conformers"], 0);
    Ok(())
}

#[test]
fn every_existing_writer_capability_reaches_actual_files_without_coordinate_or_source_rules()
-> TestResult {
    use BondOrder::{Double as D, Single as S, Triple as T};
    let directory = Directory::new()?;
    let mut isotope = atom(1, 6, 2)?;
    isotope.isotope = NonZeroU16::new(13);
    let mut deuterium = atom(3, 1, 0)?;
    deuterium.isotope = NonZeroU16::new(2);
    let mut maximum_mass = atom(1, 6, 4)?;
    maximum_mass.isotope = NonZeroU16::new(u16::MAX);
    let mut cases = vec![
        (
            fragment(
                vec![
                    atom(1, 6, 1)?,
                    atom(2, 6, 3)?,
                    atom(3, 6, 3)?,
                    atom(4, 6, 3)?,
                ],
                &[(1, 2, S), (1, 3, S), (1, 4, S)],
            )?,
            "[CH1]([CH3])([CH3])[CH3]".to_owned(),
        ),
        (
            fragment(vec![atom(1, 6, 2)?, atom(2, 6, 2)?], &[(1, 2, D)])?,
            "[CH2]=[CH2]".to_owned(),
        ),
        (
            fragment(vec![atom(1, 6, 1)?, atom(2, 7, 0)?], &[(1, 2, T)])?,
            "[CH1]#[NH0]".to_owned(),
        ),
        (
            fragment(vec![atom(1, 6, 3)?, atom(2, 8, 1)?], &[(1, 2, S)])?,
            "[CH3][OH1]".to_owned(),
        ),
        (
            fragment(vec![atom(1, 6, 4)?, atom(2, 8, 2)?], &[])?,
            "[CH4].[OH2]".to_owned(),
        ),
        (
            fragment(
                vec![isotope, atom(2, 1, 0)?, deuterium],
                &[(1, 2, S), (1, 3, S)],
            )?,
            "[13CH2]([H])[2H]".to_owned(),
        ),
        (fragment(vec![maximum_mass], &[])?, "[65535CH4]".to_owned()),
        (
            fragment(
                (1..=6).map(|id| atom(id, 6, 2)).collect::<Result<_, _>>()?,
                &[
                    (1, 2, S),
                    (2, 3, S),
                    (3, 4, S),
                    (4, 5, S),
                    (5, 6, S),
                    (6, 1, S),
                ],
            )?,
            "[CH2]0[CH2][CH2][CH2][CH2][CH2]0".to_owned(),
        ),
    ];
    for (element, h, token) in [
        (7, 3, "[NH3]"),
        (8, 2, "[OH2]"),
        (9, 1, "[FH1]"),
        (17, 1, "[ClH1]"),
        (35, 1, "[BrH1]"),
        (53, 1, "[IH1]"),
    ] {
        cases.push((fragment(vec![atom(1, element, h)?], &[])?, token.to_owned()));
    }
    for (index, (model, literal)) in cases.into_iter().enumerate() {
        let before = model.clone();
        let path = directory.0.join(format!("capability-{index}-λ.smi"));
        let receipt = export_smiles_fragment_file(&path, &model, &Cancellation::default())?;
        assert_eq!(fs::read(&path)?, format!("{literal}\n").as_bytes());
        assert_eq!(receipt.byte_length, literal.len() + 1);
        assert_eq!(model, before);
        let actual = external(&path)?;
        assert_eq!(actual["notation"]["source_ascii"], literal);
        let atoms = actual["notation"]["atoms"]
            .as_array()
            .ok_or("external atoms")?;
        assert_eq!(atoms.len(), model.atoms().len());
        for (got, expected) in atoms.iter().zip(model.atoms()) {
            assert_eq!(got["element"], expected.element.get());
            assert_eq!(got["isotope"], expected.isotope.map_or(0, NonZeroU16::get));
            assert_eq!(
                got["explicit_h"],
                expected.explicit_hydrogens.ok_or("literal H")?
            );
            assert_eq!(got["charge"], 0);
            assert_eq!(got["no_implicit"], true);
        }
        let mut bonds = model
            .bonds()
            .iter()
            .map(|bond| {
                let token = match bond.order {
                    S => "SINGLE",
                    D => "DOUBLE",
                    T => "TRIPLE",
                    _ => "not in this fixture",
                };
                let a = bond.begin.get() - 1;
                let b = bond.end.get() - 1;
                serde_json::json!([a.min(b), a.max(b), token])
            })
            .collect::<Vec<_>>();
        bonds.sort_by_key(|value| (value[0].as_u64(), value[1].as_u64()));
        assert_eq!(actual["notation"]["bonds"], serde_json::json!(bonds));
        assert_eq!(
            actual["meaning"]["radicals"],
            serde_json::json!(vec![0; model.atoms().len()])
        );
        assert_eq!(actual["meaning"]["conformers"], 0);
    }
    Ok(())
}

#[test]
fn the_filesystem_adapter_does_not_inherit_the_converter_twelve_atom_or_layout_limits() -> TestResult
{
    let directory = Directory::new()?;
    let count = 4096_u32;
    let atoms = (1..=count)
        .map(|id| atom(id, 6, if id == 1 || id == count { 3 } else { 2 }))
        .collect::<Result<_, _>>()?;
    let edges = (1..count)
        .map(|id| (id, id + 1, BondOrder::Single))
        .collect::<Vec<_>>();
    let input = fragment(atoms, &edges)?;
    let before = input.clone();
    let path = directory.0.join("large-λ.smi");
    let receipt = export_smiles_fragment_file(&path, &input, &Cancellation::default())?;
    let literal = format!("[CH3]{}[CH3]", "[CH2]".repeat(4094));
    assert_eq!(fs::read(&path)?, format!("{literal}\n").as_bytes());
    assert_eq!(receipt.byte_length, 4096 * 5 + 1);
    assert_eq!(input, before);
    let actual = external(&path)?;
    assert_eq!(
        actual["notation"]["atoms"]
            .as_array()
            .ok_or("external atoms")?
            .len(),
        4096
    );
    assert_eq!(
        actual["notation"]["bonds"]
            .as_array()
            .ok_or("external bonds")?
            .len(),
        4095
    );
    assert_eq!(actual["meaning"]["formula"], "C4096H8194");
    Ok(())
}

#[test]
fn unsupported_or_over_budget_fragment_errors_are_typed_encode_refusals_without_files() -> TestResult
{
    let directory = Directory::new()?;
    let path = directory.0.join("must-not-exist.smi");
    let mut cases = vec![(fragment(vec![], &[])?, "invalid_smiles_export")];
    for kind in [
        "unknown_h",
        "charge",
        "radical",
        "element",
        "valence",
        "physical_h",
    ] {
        let mut value = atom(1, 6, 4)?;
        match kind {
            "unknown_h" => value.explicit_hydrogens = None,
            "charge" => value.charge = 1,
            "radical" => value.radical = Radical::Doublet,
            "element" => value.element = Element::new(15).ok_or("P")?,
            "valence" => value.explicit_hydrogens = Some(0),
            "physical_h" => {
                value.element = Element::new(1).ok_or("H")?;
                value.explicit_hydrogens = Some(0);
            }
            _ => return Err("fixture kind".into()),
        }
        cases.push((fragment(vec![value], &[])?, "unsupported_content"));
    }
    cases.push((
        fragment(
            vec![atom(1, 6, 3)?, atom(2, 6, 3)?],
            &[(1, 2, BondOrder::Aromatic)],
        )?,
        "unsupported_content",
    ));
    cases.push((
        fragment(
            (1..=4097)
                .map(|id| atom(id, 6, 4))
                .collect::<Result<_, _>>()?,
            &[],
        )?,
        "resource_limit",
    ));
    for (model, code) in cases {
        let before = model.clone();
        let error = export_smiles_fragment_file(&path, &model, &Cancellation::default())
            .err()
            .ok_or("accepted bad fragment")?;
        assert_eq!(error.code(), code);
        assert_eq!(error.stage(), "encode");
        assert_eq!(
            error.path(),
            fs::canonicalize(&directory.0)?.join("must-not-exist.smi")
        );
        assert!(!error.published());
        assert!(error.recovery_path().is_none());
        assert_eq!(model, before);
        assert_eq!(fs::read_dir(&directory.0)?.count(), 0);
    }
    Ok(())
}

#[test]
fn existing_or_cancelled_targets_are_not_replaced_and_original_fragment_stays_exact() -> TestResult
{
    let directory = Directory::new()?;
    let path = directory.0.join("original.smi");
    fs::write(&path, b"keep original\r\n")?;
    let before = fs::metadata(&path)?;
    let model = fragment(vec![atom(1, 6, 4)?], &[])?;
    let original = model.clone();
    let error = export_smiles_fragment_file(&path, &model, &Cancellation::default())
        .err()
        .ok_or("overwrote target")?;
    assert_eq!(error.code(), "already_exists");
    assert_eq!(error.stage(), "validate");
    assert!(!error.published());
    assert_eq!(fs::read(&path)?, b"keep original\r\n");
    let after = fs::metadata(&path)?;
    assert_eq!(before.permissions(), after.permissions());
    assert_eq!(before.modified()?, after.modified()?);
    let cancelled = Cancellation::default();
    cancelled.cancel();
    let new = directory.0.join("cancelled.smi");
    let error = export_smiles_fragment_file(&new, &model, &cancelled)
        .err()
        .ok_or("ignored cancellation")?;
    assert_eq!(error.code(), "cancelled");
    assert!(!error.published());
    assert!(!new.exists());
    assert_eq!(model, original);
    assert_eq!(fs::read_dir(&directory.0)?.count(), 1);
    Ok(())
}

#[cfg(unix)]
#[test]
fn output_symlinks_refuse_without_touching_the_target() -> TestResult {
    let directory = Directory::new()?;
    let target = directory.0.join("keep.smi");
    fs::write(&target, b"keep")?;
    let link = directory.0.join("chosen.smi");
    std::os::unix::fs::symlink(&target, &link)?;
    let model = fragment(vec![atom(1, 6, 4)?], &[])?;
    let error = export_smiles_fragment_file(&link, &model, &Cancellation::default())
        .err()
        .ok_or("followed output link")?;
    assert_eq!(error.code(), "symlink_target");
    assert!(!error.published());
    assert_eq!(fs::read(&target)?, b"keep");
    assert_eq!(fs::read_link(&link)?, target);
    Ok(())
}
