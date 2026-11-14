use bondline_io::{
    cdxml::write_document,
    files::{
        SmilesConversionReceipt, convert_smiles_to_cdxml_file, read_cdxml_file, save_cdxml_file,
    },
    smiles::SmilesDocument,
};
use bondline_model::{Fragment, PageId, PageSize, Point2};
use bondline_render::Cancellation;
use serde_json::Value;
use std::{
    fs,
    io::Write as _,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

type TestResult = Result<(), Box<dyn std::error::Error>>;
struct Directory(PathBuf);
impl Directory {
    fn new() -> Result<Self, std::io::Error> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "bondline-smiles-convert-{}-{}",
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
fn origin() -> Result<Point2, bondline_model::Error> {
    Point2::new(72.0, 72.0)
}
fn python() -> std::ffi::OsString {
    std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    })
}
fn snapshot(path: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output=Command::new(python()).arg("-B").arg("-c").arg("import hashlib,json,os,sys; p=sys.argv[1]; s=os.stat(p); print(json.dumps({'sha256':hashlib.sha256(open(p,'rb').read()).hexdigest(),'bytes':s.st_size,'mode':s.st_mode,'mtime_ns':s.st_mtime_ns}))").arg(path).output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}
fn external(
    script: &str,
    source: &[u8],
    args: &[String],
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut child = Command::new(python())
        .arg("-B")
        .arg(format!("{}/{}", env!("CARGO_MANIFEST_DIR"), script))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("stdin")?.write_all(source)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "actual converted-file consumer: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(output.stdout)
}
fn assert_receipt(
    receipt: &SmilesConversionReceipt,
    input: &Path,
    output: &Path,
    bytes: usize,
    atoms: usize,
) -> TestResult {
    assert_eq!(receipt.input_path, fs::canonicalize(input)?);
    assert_eq!(receipt.input_byte_length, bytes);
    assert_eq!(receipt.atom_count, atoms);
    assert_eq!(receipt.bond_count, atoms - 1);
    assert_eq!(receipt.output.path, fs::canonicalize(output)?);
    assert_eq!(receipt.output.byte_length, fs::read(output)?.len());
    assert_eq!(receipt.clone(), *receipt);
    Ok(())
}

#[test]
fn all_twelve_actual_files_keep_independent_input_snapshot_and_reopen_editable() -> TestResult {
    let directory = Directory::new()?;
    for n in 1..=12 {
        let source = format!("{}\r\n", "C".repeat(n));
        let input = directory.0.join(format!("input-{n}-λ.smi"));
        let output = directory.0.join(format!("new-{n}-λ.cdxml"));
        fs::write(&input, source.as_bytes())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&input, fs::Permissions::from_mode(0o640))?;
        }
        let before = snapshot(&input)?;
        let receipt = convert_smiles_to_cdxml_file(
            &input,
            &output,
            48.0,
            origin()?,
            PageSize::A4,
            &Cancellation::default(),
        )?;
        assert_receipt(&receipt, &input, &output, source.len(), n)?;
        assert_eq!(snapshot(&input)?, before);
        assert_eq!(fs::read(&input)?, source.as_bytes());
        let owner = SmilesDocument::parse(Arc::from(source.as_bytes()))?;
        let expected = owner.layout_carbon_path(48.0, origin()?, PageSize::A4)?;
        let raw = fs::read(&output)?;
        assert_eq!(raw, write_document(&expected)?);
        let structural: Value =
            serde_json::from_slice(&external("../../xtask/oracles/cdxml_graph.py", &raw, &[])?)?;
        let expected_wire: Value =
            serde_json::from_slice(&serde_json::to_vec(expected.pages()[0].fragments())?)?;
        assert_eq!(structural, expected_wire);
        let coordinates = expected.pages()[0].fragments()[0]
            .atoms()
            .iter()
            .map(|atom| {
                let p = atom.position.ok_or("position")?;
                Ok([p.x_pt().to_string(), p.y_pt().to_string()])
            })
            .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
        let meaning: Value = serde_json::from_slice(&external(
            "tests/smiles_layout_oracle.py",
            &raw,
            &[serde_json::to_string(&coordinates)?],
        )?)?;
        assert_eq!(meaning["smiles"], "C".repeat(n));
        assert_eq!(
            meaning["formula"],
            if n == 1 {
                "CH4".to_owned()
            } else {
                format!("C{n}H{}", 2 * n + 2)
            }
        );
        if n == 2 {
            external("../../xtask/oracles/native_bond_rdkit.py", &raw, &[])?;
        }
        let mut loaded = read_cdxml_file(&output, &Cancellation::default())?.document;
        assert!(loaded.is_fully_mapped());
        assert_eq!(loaded.model(), &expected);
        assert_eq!(loaded.to_cdxml()?.as_ref(), raw);
        if n == 3 {
            let old = &expected.pages()[0].fragments()[0];
            let mut atoms = old.atoms().to_vec();
            let p = atoms[0].position.ok_or("first position")?;
            atoms[0].position = Some(Point2::new(p.x_pt() + 4.0, p.y_pt())?);
            loaded.model_mut().replace_fragment(
                PageId::new(1).ok_or("page")?,
                Fragment::new(old.id(), atoms, old.bonds().to_vec())?,
            )?;
            let edited = directory.0.join("edited-native.cdxml");
            save_cdxml_file(&edited, &loaded, false, &Cancellation::default())?;
            let reopened = read_cdxml_file(&edited, &Cancellation::default())?.document;
            assert!(reopened.is_fully_mapped());
            assert_eq!(reopened.model(), loaded.model());
            assert_eq!(fs::read(&output)?, raw);
            assert_eq!(snapshot(&input)?, before);
        }
    }
    Ok(())
}

#[test]
fn equivalent_notation_and_metadata_only_receipt_preserve_raw_spelling() -> TestResult {
    let directory = Directory::new()?;
    for (index, source) in ["C(C)C\r\n", "[CH2]([CH3])[CH3]\n", "C1.C1\n"]
        .into_iter()
        .enumerate()
    {
        let input = directory.0.join(format!("source-{index}.data"));
        let output = directory.0.join(format!("new-{index}.cdxml"));
        fs::write(&input, source)?;
        let before = snapshot(&input)?;
        let receipt = convert_smiles_to_cdxml_file(
            &input,
            &output,
            48.0,
            origin()?,
            PageSize::A4,
            &Cancellation::default(),
        )?;
        assert_eq!(snapshot(&input)?, before);
        assert!(!format!("{receipt:?}").contains(source.trim_end()));
        let expected = SmilesDocument::parse(Arc::from(source.as_bytes()))?.layout_carbon_path(
            48.0,
            origin()?,
            PageSize::A4,
        )?;
        let loaded = read_cdxml_file(&output, &Cancellation::default())?.document;
        assert_eq!(loaded.model(), &expected);
        assert_eq!(
            receipt.atom_count,
            expected.pages()[0].fragments()[0].atoms().len()
        );
    }
    Ok(())
}

#[test]
fn full_source_refusals_never_publish_or_change_input_or_old_target() -> TestResult {
    let directory = Directory::new()?;
    let input = directory.0.join("source.smi");
    let target = directory.0.join("new.cdxml");
    let old = directory.0.join("keep.cdxml");
    fs::write(&old, b"existing target")?;
    for (source, code) in [
        ("", "invalid_smiles"),
        ("CC name\n", "invalid_smiles"),
        ("CC\nC", "invalid_smiles"),
        ("CC\n\n", "invalid_smiles"),
        ("C(", "invalid_smiles"),
        ("c1ccccc1", "unsupported_content"),
        ("[cH]1ccccc1", "unsupported_content"),
        ("C:C", "unsupported_content"),
        ("CC(C)C", "unsupported_content"),
        ("C1CC1", "unsupported_content"),
        ("CO", "unsupported_content"),
        ("C[H]", "unsupported_content"),
        ("[13CH3]C", "unsupported_content"),
        ("[NH4+]", "unsupported_content"),
        ("[CH3:7]", "unsupported_content"),
        ("[C@H](F)(Cl)Br", "unsupported_content"),
        ("C=C", "unsupported_content"),
        ("C#C", "unsupported_content"),
        ("CC.CC", "unsupported_content"),
        ("ＣＣ", "unsupported_content"),
        ("CCCCCCCCCCCCC", "resource_limit"),
    ] {
        fs::write(&input, source)?;
        let before = snapshot(&input)?;
        for output in [&target, &old] {
            let error = convert_smiles_to_cdxml_file(
                &input,
                output,
                48.0,
                origin()?,
                PageSize::A4,
                &Cancellation::default(),
            )
            .err()
            .ok_or_else(|| format!("accepted {source:?}"))?;
            assert_eq!(error.code(), code, "{source:?}: {error}");
            assert_eq!(error.stage(), "decode");
            assert_eq!(error.path(), fs::canonicalize(&input)?);
            assert!(!error.published());
            assert!(error.recovery_path().is_none());
        }
        assert!(!target.exists());
        assert_eq!(fs::read(&old)?, b"existing target");
        assert_eq!(snapshot(&input)?, before);
        assert_eq!(fs::read_dir(&directory.0)?.count(), 2);
    }
    Ok(())
}

#[test]
fn layout_errors_and_true_read_limit_are_not_relabelled_as_native_64mib() -> TestResult {
    let directory = Directory::new()?;
    let input = directory.0.join("source.smi");
    let output = directory.0.join("new.cdxml");
    fs::write(&input, b"CC\n")?;
    for (length, origin, page, code) in [
        (0.0, origin()?, PageSize::A4, "invalid_params"),
        (-1.0, origin()?, PageSize::A4, "invalid_params"),
        (f64::NAN, origin()?, PageSize::A4, "invalid_params"),
        (f64::INFINITY, origin()?, PageSize::A4, "invalid_params"),
        (1.0, origin()?, PageSize::A4, "unsupported_content"),
        (
            48.0,
            Point2::new(2.0, 72.0)?,
            PageSize::A4,
            "unsupported_content",
        ),
        (
            48.0,
            origin()?,
            PageSize::new(1.0, 100_000.0)?,
            "unsupported_content",
        ),
    ] {
        let before = snapshot(&input)?;
        let error = convert_smiles_to_cdxml_file(
            &input,
            &output,
            length,
            origin,
            page,
            &Cancellation::default(),
        )
        .err()
        .ok_or("accepted bad layout")?;
        assert_eq!(error.code(), code);
        assert_eq!(error.stage(), "decode");
        assert!(!error.published());
        assert_eq!(snapshot(&input)?, before);
        assert!(!output.exists());
    }
    fs::File::create(&input)?.set_len(65_537)?;
    let before = snapshot(&input)?;
    let error = convert_smiles_to_cdxml_file(
        &input,
        &output,
        48.0,
        origin()?,
        PageSize::A4,
        &Cancellation::default(),
    )
    .err()
    .ok_or("oversized input")?;
    assert_eq!(error.code(), "file_too_large");
    assert_eq!(error.stage(), "read");
    assert!(error.to_string().contains("64 KiB"));
    assert!(!error.to_string().contains("64 MiB"));
    assert_eq!(snapshot(&input)?, before);
    assert!(!output.exists());
    fs::write(&input, "C".repeat(65_536))?;
    let error = convert_smiles_to_cdxml_file(
        &input,
        &output,
        48.0,
        origin()?,
        PageSize::A4,
        &Cancellation::default(),
    )
    .err()
    .ok_or("parse atoms bound")?;
    assert_eq!(error.code(), "resource_limit");
    assert_eq!(error.stage(), "decode");
    assert!(!output.exists());
    Ok(())
}

#[test]
fn create_only_aliases_missing_input_and_cancellation_preserve_every_file() -> TestResult {
    let directory = Directory::new()?;
    let input = directory.0.join("source.smi");
    let output = directory.0.join("keep.cdxml");
    fs::write(&input, b"CCC\r\n")?;
    fs::write(&output, b"old output")?;
    let before = snapshot(&input)?;
    let alias = directory.0.join("same-inode.cdxml");
    fs::hard_link(&input, &alias)?;
    for destination in [&input, &output, &alias] {
        let error = convert_smiles_to_cdxml_file(
            &input,
            destination,
            48.0,
            origin()?,
            PageSize::A4,
            &Cancellation::default(),
        )
        .err()
        .ok_or("clobbered destination")?;
        assert_eq!(error.code(), "already_exists");
        assert!(!error.published());
    }
    assert_eq!(snapshot(&input)?, before);
    assert_eq!(fs::read(&alias)?, b"CCC\r\n");
    assert_eq!(fs::read(&output)?, b"old output");
    let missing = directory.0.join("missing.smi");
    assert_eq!(
        convert_smiles_to_cdxml_file(
            &missing,
            &output,
            48.0,
            origin()?,
            PageSize::A4,
            &Cancellation::default()
        )
        .err()
        .ok_or("missing source")?
        .code(),
        "not_found"
    );
    assert_eq!(
        convert_smiles_to_cdxml_file(
            &directory.0,
            &output,
            48.0,
            origin()?,
            PageSize::A4,
            &Cancellation::default()
        )
        .err()
        .ok_or("directory input")?
        .code(),
        "not_regular_file"
    );
    let cancel = Cancellation::default();
    cancel.cancel();
    let error =
        convert_smiles_to_cdxml_file(&input, &output, 48.0, origin()?, PageSize::A4, &cancel)
            .err()
            .ok_or("cancelled convert ran")?;
    assert_eq!(error.code(), "cancelled");
    assert!(!error.published());
    assert_eq!(snapshot(&input)?, before);
    assert_eq!(fs::read(&output)?, b"old output");
    assert_eq!(fs::read_dir(&directory.0)?.count(), 3);
    Ok(())
}

#[cfg(unix)]
#[test]
fn reading_symlink_keeps_its_target_while_output_symlink_is_refused() -> TestResult {
    let directory = Directory::new()?;
    let input = directory.0.join("source.smi");
    fs::write(&input, b"CC\n")?;
    let alias = directory.0.join("chosen.smi");
    std::os::unix::fs::symlink(&input, &alias)?;
    let output = directory.0.join("new.cdxml");
    let before = snapshot(&input)?;
    let receipt = convert_smiles_to_cdxml_file(
        &alias,
        &output,
        48.0,
        origin()?,
        PageSize::A4,
        &Cancellation::default(),
    )?;
    assert_receipt(&receipt, &input, &output, 3, 2)?;
    assert_eq!(fs::read_link(&alias)?, input);
    assert_eq!(snapshot(&input)?, before);
    let target = directory.0.join("other");
    fs::write(&target, b"keep other")?;
    let link = directory.0.join("link.cdxml");
    std::os::unix::fs::symlink(&target, &link)?;
    let error = convert_smiles_to_cdxml_file(
        &input,
        &link,
        48.0,
        origin()?,
        PageSize::A4,
        &Cancellation::default(),
    )
    .err()
    .ok_or("followed output symlink")?;
    assert_eq!(error.code(), "symlink_target");
    assert!(!error.published());
    assert_eq!(fs::read(&target)?, b"keep other");
    assert_eq!(snapshot(&input)?, before);
    Ok(())
}
