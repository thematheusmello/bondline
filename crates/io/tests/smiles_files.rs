use bondline_io::{
    files::{export_smiles_fragment_file, read_smiles_file, save_smiles_file},
    smiles::file::RecordId,
};
use bondline_render::Cancellation;
use std::{
    fs,
    io::Write as _,
    num::NonZeroU16,
    path::PathBuf,
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const MIXED: &[u8] = b"\r\n \tignored [not valid]\r\n[13CH3][OH1]\tfirst name = x\t  \r\nc1ccccc1 benzene\tclassification\n\tignored ???\n[2H][H]\tlast opaque !";

struct Directory(PathBuf);
impl Directory {
    fn new() -> std::io::Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "bondline-smiles-file-{}-{}",
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

#[test]
fn complete_mixed_file_keeps_every_record_metadata_and_raw_byte_on_save() -> TestResult {
    let directory = Directory::new()?;
    let input = directory.0.join("original-化学.cdx");
    let output = directory.0.join("copy-λ.data");
    fs::write(&input, MIXED)?;
    let loaded = read_smiles_file(&input, &Cancellation::default())?;
    assert_eq!(loaded.path, fs::canonicalize(&input)?);
    assert_eq!(loaded.document.as_bytes(), MIXED);
    assert_eq!(loaded.document.records().len(), 3);
    for (info, (id, line, full, notation, separator, data, ending, atoms, bonds)) in
        loaded.document.records().iter().zip([
            (1, 3, 25..57, 25..37, 37..38, 38..55, 55..57, 2, 1),
            (2, 4, 57..89, 57..65, 65..66, 66..88, 88..89, 6, 6),
            (3, 6, 102..123, 102..109, 109..110, 110..123, 123..123, 2, 1),
        ])
    {
        assert_eq!(info.id().get(), id);
        assert_eq!(info.line_number(), line);
        assert_eq!(info.full_range(), full);
        assert_eq!(info.notation_range(), notation);
        assert_eq!(info.separator_range(), Some(separator));
        assert_eq!(info.application_data_range(), Some(data));
        assert_eq!(info.line_ending_range(), ending);
        assert_eq!(info.atom_count(), atoms);
        assert_eq!(info.bond_count(), bonds);
    }
    for (id, notation, elements, isotopes, hydrogens, bonds) in [
        (
            1,
            b"[13CH3][OH1]".as_slice(),
            vec![6, 8],
            vec![13, 0],
            vec![3, 1],
            serde_json::json!([[0, 1, "SINGLE"]]),
        ),
        (
            2,
            b"c1ccccc1".as_slice(),
            vec![6; 6],
            vec![0; 6],
            vec![0; 6],
            serde_json::json!([
                [0, 1, "AROMATIC"],
                [0, 5, "AROMATIC"],
                [1, 2, "AROMATIC"],
                [2, 3, "AROMATIC"],
                [3, 4, "AROMATIC"],
                [4, 5, "AROMATIC"]
            ]),
        ),
        (
            3,
            b"[2H][H]".as_slice(),
            vec![1, 1],
            vec![2, 0],
            vec![0, 0],
            serde_json::json!([[0, 1, "SINGLE"]]),
        ),
    ] {
        let record = loaded
            .document
            .read_record(RecordId::new(id).ok_or("record ID")?)?;
        assert_eq!(record.as_bytes(), notation);
        assert_external(notation, &elements, &isotopes, &hydrogens, &bonds)?;
    }
    let original = loaded.document.clone();
    let receipt = save_smiles_file(&output, &loaded.document, false, &Cancellation::default())?;
    assert_eq!(receipt.path, fs::canonicalize(&output)?);
    assert_eq!(receipt.byte_length, MIXED.len());
    assert_eq!(fs::read(&output)?, MIXED);
    assert_eq!(fs::read(&input)?, MIXED);
    assert_eq!(loaded.document, original);
    assert_eq!(
        read_smiles_file(&output, &Cancellation::default())?.document,
        original
    );
    Ok(())
}

fn external(notation: &[u8]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    assert!(notation.is_ascii());
    assert!(!notation.iter().any(u8::is_ascii_whitespace));
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
            "/tests/smiles_oracle.py"
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
    let value: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(value["rdkit"], "2026.09.2");
    assert_eq!(value["accepted"], true);
    assert_eq!(value["source_ascii"], std::str::from_utf8(notation)?);
    assert_eq!(value["source_bytes"], notation.len());
    Ok(value)
}

fn assert_external(
    notation: &[u8],
    elements: &[u8],
    isotopes: &[u16],
    hydrogens: &[u16],
    bonds: &serde_json::Value,
) -> TestResult {
    let actual = external(notation)?;
    let atoms = actual["atoms"].as_array().ok_or("external atom array")?;
    assert_eq!(atoms.len(), elements.len());
    assert_eq!(atoms.len(), isotopes.len());
    assert_eq!(atoms.len(), hydrogens.len());
    for (((atom, element), isotope), hydrogens) in
        atoms.iter().zip(elements).zip(isotopes).zip(hydrogens)
    {
        assert_eq!(atom["element"], *element);
        assert_eq!(atom["isotope"], *isotope);
        assert_eq!(atom["explicit_h"], *hydrogens);
        assert_eq!(atom["charge"], 0);
    }
    assert_eq!(&actual["bonds"], bonds);
    Ok(())
}

#[test]
fn whole_file_over_64kib_is_allowed_without_enlarging_notation_limits() -> TestResult {
    let directory = Directory::new()?;
    let input = directory.0.join("metadata.txt");
    let output = directory.0.join("copied.smi");
    let mut bytes = b"C\t".to_vec();
    bytes.extend(std::iter::repeat_n(b'x', 70 * 1024));
    bytes.extend_from_slice(b"  \r\n[OH2]\n");
    assert!(bytes.len() > bondline_io::smiles::MAX_SMILES_BYTES);
    fs::write(&input, &bytes)?;
    let loaded = read_smiles_file(&input, &Cancellation::default())?;
    assert_eq!(loaded.document.as_bytes(), bytes);
    assert_eq!(loaded.document.records().len(), 2);
    let first = loaded.document.records().first().ok_or("first record")?;
    assert_eq!(first.notation_range(), 0..1);
    assert_eq!(first.application_data_range(), Some(2..(2 + 70 * 1024 + 2)));
    assert_eq!(loaded.document.read_record(first.id())?.as_bytes(), b"C");
    let receipt = save_smiles_file(&output, &loaded.document, false, &Cancellation::default())?;
    assert_eq!(receipt.byte_length, bytes.len());
    assert_eq!(fs::read(&output)?, bytes);
    assert_external(b"C", &[6], &[0], &[0], &serde_json::json!([]))?;
    assert_external(b"[OH2]", &[8], &[0], &[2], &serde_json::json!([]))?;
    Ok(())
}

#[test]
fn empty_and_ignored_only_files_are_retained_and_source_owns_bytes_after_input_removal()
-> TestResult {
    let directory = Directory::new()?;
    for (index, bytes) in [
        b"".as_slice(),
        b"\r\n\n ignored text @@@\n\tanything\r\n\rignored also\n",
    ]
    .into_iter()
    .enumerate()
    {
        let input = directory.0.join(format!("input-{index}.data"));
        let output = directory.0.join(format!("copy-{index}.data"));
        fs::write(&input, bytes)?;
        let loaded = read_smiles_file(&input, &Cancellation::default())?;
        assert_eq!(loaded.document.as_bytes(), bytes);
        assert!(loaded.document.records().is_empty());
        fs::remove_file(&input)?;
        let receipt = save_smiles_file(&output, &loaded.document, false, &Cancellation::default())?;
        assert_eq!(receipt.byte_length, bytes.len());
        assert_eq!(fs::read(&output)?, bytes);
    }
    Ok(())
}

#[test]
fn malformed_or_unsupported_later_record_refuses_the_whole_read_at_canonical_decode_path()
-> TestResult {
    let directory = Directory::new()?;
    let input = directory.0.join("later-error.smi");
    let valid = directory.0.join("valid.smi");
    fs::write(&valid, MIXED)?;
    let old = read_smiles_file(&valid, &Cancellation::default())?.document;
    let baseline = old.clone();
    for (later, code) in [
        (b"C(".as_slice(), "invalid_smiles"),
        (b"[C@H]".as_slice(), "unsupported_content"),
        (b"#comment".as_slice(), "invalid_smiles"),
        (b"\x0bC".as_slice(), "invalid_smiles"),
    ] {
        let mut bytes = b"CC first\r\n\n".to_vec();
        bytes.extend_from_slice(later);
        fs::write(&input, &bytes)?;
        let error = read_smiles_file(&input, &Cancellation::default())
            .err()
            .ok_or("partial read accepted")?;
        assert_eq!(error.code(), code);
        assert_eq!(error.stage(), "decode");
        assert_eq!(error.path(), fs::canonicalize(&input)?);
        assert!(!error.published());
        assert!(error.recovery_path().is_none());
        assert_eq!(fs::read(&input)?, bytes);
        assert_eq!(old, baseline);
    }
    let mut notation_over = vec![b'0'; bondline_io::smiles::MAX_SMILES_BYTES + 1];
    notation_over[0] = b'C';
    fs::write(&input, &notation_over)?;
    let error = read_smiles_file(&input, &Cancellation::default())
        .err()
        .ok_or("notation byte cap")?;
    assert_eq!(error.code(), "resource_limit");
    assert_eq!(error.stage(), "decode");
    assert_eq!(old, baseline);
    Ok(())
}

#[test]
fn source_and_destination_checks_cancellation_and_no_clobber_leave_existing_bytes_exact()
-> TestResult {
    let directory = Directory::new()?;
    let input = directory.0.join("source.smi");
    let output = directory.0.join("existing.smi");
    fs::write(&input, MIXED)?;
    fs::write(&output, b"original bytes")?;
    let loaded = read_smiles_file(&input, &Cancellation::default())?;
    let error = save_smiles_file(&output, &loaded.document, false, &Cancellation::default())
        .err()
        .ok_or("clobbered file")?;
    assert_eq!(error.code(), "already_exists");
    assert_eq!(error.stage(), "validate");
    assert!(!error.published());
    assert_eq!(fs::read(&output)?, b"original bytes");
    let cancel = Cancellation::default();
    cancel.cancel();
    for error in [
        read_smiles_file(&input, &cancel)
            .err()
            .ok_or("read ignored cancellation")?,
        save_smiles_file(&output, &loaded.document, true, &cancel)
            .err()
            .ok_or("save ignored cancellation")?,
    ] {
        assert_eq!(error.code(), "cancelled");
        assert_eq!(error.stage(), "validate");
        assert!(!error.published());
        assert!(error.recovery_path().is_none());
    }
    assert_eq!(fs::read(&output)?, b"original bytes");
    let receipt = save_smiles_file(&output, &loaded.document, true, &Cancellation::default())?;
    assert_eq!(receipt.byte_length, MIXED.len());
    assert_eq!(fs::read(&output)?, MIXED);
    assert_eq!(fs::read(&input)?, MIXED);
    assert_eq!(fs::read_dir(&directory.0)?.count(), 2);
    let not_regular = read_smiles_file(&directory.0, &Cancellation::default())
        .err()
        .ok_or("directory accepted")?;
    assert_eq!(not_regular.code(), "not_regular_file");
    let missing = read_smiles_file(&directory.0.join("missing"), &Cancellation::default())
        .err()
        .ok_or("missing source accepted")?;
    assert_eq!(missing.code(), "not_found");
    assert_eq!(missing.stage(), "read");
    Ok(())
}

#[test]
fn native_64mib_source_bound_is_enforced_before_decode() -> TestResult {
    let directory = Directory::new()?;
    let input = directory.0.join("oversized.smi");
    fs::File::create(&input)?.set_len(64 * 1024 * 1024 + 1)?;
    let error = read_smiles_file(&input, &Cancellation::default())
        .err()
        .ok_or("unbounded source")?;
    assert_eq!(error.code(), "file_too_large");
    assert_eq!(error.stage(), "read");
    assert_eq!(error.path(), fs::canonicalize(&input)?);
    assert!(error.to_string().contains("64 MiB"));
    assert!(!error.to_string().contains("64 KiB"));
    assert!(!error.published());
    Ok(())
}

#[cfg(unix)]
#[test]
fn selected_read_symlink_is_resolved_but_saving_through_a_symlink_is_refused() -> TestResult {
    let directory = Directory::new()?;
    let input = directory.0.join("source.smi");
    let alias = directory.0.join("alias-化学.cdx");
    fs::write(&input, MIXED)?;
    std::os::unix::fs::symlink(&input, &alias)?;
    let loaded = read_smiles_file(&alias, &Cancellation::default())?;
    assert_eq!(loaded.path, fs::canonicalize(&input)?);
    assert_eq!(loaded.document.as_bytes(), MIXED);
    let error = save_smiles_file(&alias, &loaded.document, true, &Cancellation::default())
        .err()
        .ok_or("followed write symlink")?;
    assert_eq!(error.code(), "symlink_target");
    assert!(!error.published());
    assert_eq!(fs::read(&input)?, MIXED);
    assert!(fs::symlink_metadata(&alias)?.file_type().is_symlink());
    assert_eq!(fs::read_dir(&directory.0)?.count(), 2);
    Ok(())
}

#[test]
fn actual_current_export_file_reopens_without_narrowing_any_supported_writer_feature() -> TestResult
{
    use bondline_model::{Atom, AtomId, Bond, BondId, BondOrder, Element, Fragment, FragmentId};
    let directory = Directory::new()?;
    let input = directory.0.join("all-writer-features.smi");
    let output = directory.0.join("all-features-copy.cdxml");
    let elements = [
        6, 6, 6, 6, 6, 1, 1, 6, 6, 6, 6, 6, 6, 7, 8, 9, 17, 35, 53, 7, 6,
    ];
    let isotopes = [
        0, 0, 0, 0, 13, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 65535,
    ];
    let hydrogens = [
        1, 3, 3, 3, 2, 0, 0, 2, 2, 2, 2, 2, 1, 0, 2, 1, 1, 1, 1, 3, 4,
    ];
    let mut atoms = Vec::new();
    for (index, ((element, isotope), hydrogens)) in
        elements.iter().zip(isotopes).zip(hydrogens).enumerate()
    {
        let mut atom = Atom::new(
            AtomId::new(u32::try_from(index + 1)?).ok_or("atom ID")?,
            Element::new(*element).ok_or("element")?,
            None,
        );
        atom.isotope = NonZeroU16::new(isotope);
        atom.explicit_hydrogens = Some(hydrogens);
        atoms.push(atom);
    }
    use BondOrder::{Double as D, Single as S, Triple as T};
    let bonds = [
        (1, 2, S),
        (1, 3, S),
        (1, 4, S),
        (5, 6, S),
        (5, 7, S),
        (8, 9, S),
        (9, 10, S),
        (10, 8, S),
        (11, 12, D),
        (13, 14, T),
    ]
    .into_iter()
    .enumerate()
    .map(|(index, (begin, end, order))| {
        Ok(Bond {
            id: BondId::new(u32::try_from(index + 1)?).ok_or("bond ID")?,
            begin: AtomId::new(begin).ok_or("begin")?,
            end: AtomId::new(end).ok_or("end")?,
            order,
        })
    })
    .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    let graph = Fragment::new(FragmentId::new(71).ok_or("fragment ID")?, atoms, bonds)?;
    let before = graph.clone();
    let expected = b"[CH1]([CH3])([CH3])[CH3].[13CH2]([H])[2H].[CH2]0[CH2][CH2]0.[CH2]=[CH2].[CH1]#[NH0].[OH2].[FH1].[ClH1].[BrH1].[IH1].[NH3].[65535CH4]\n";
    let exported = export_smiles_fragment_file(&input, &graph, &Cancellation::default())?;
    assert_eq!(fs::read(&input)?, expected);
    assert_eq!(exported.byte_length, expected.len());
    assert_eq!(graph, before);
    let loaded = read_smiles_file(&input, &Cancellation::default())?;
    assert_eq!(loaded.document.as_bytes(), expected);
    assert_eq!(loaded.document.records().len(), 1);
    let info = loaded.document.records().first().ok_or("exported record")?;
    assert_eq!(info.atom_count(), 21);
    assert_eq!(info.bond_count(), 10);
    assert_eq!(info.separator_range(), None);
    assert_eq!(info.application_data_range(), None);
    assert_eq!(info.notation_range(), 0..expected.len() - 1);
    assert_eq!(info.line_ending_range(), expected.len() - 1..expected.len());
    assert_eq!(
        loaded.document.read_record(info.id())?.as_bytes(),
        &expected[..expected.len() - 1]
    );
    let receipt = save_smiles_file(&output, &loaded.document, false, &Cancellation::default())?;
    assert_eq!(receipt.byte_length, expected.len());
    assert_eq!(fs::read(&output)?, expected);
    assert_eq!(
        read_smiles_file(&output, &Cancellation::default())?.document,
        loaded.document
    );
    assert_external(
        &expected[..expected.len() - 1],
        &elements,
        &isotopes,
        &hydrogens,
        &serde_json::json!([
            [0, 1, "SINGLE"],
            [0, 2, "SINGLE"],
            [0, 3, "SINGLE"],
            [4, 5, "SINGLE"],
            [4, 6, "SINGLE"],
            [7, 8, "SINGLE"],
            [7, 9, "SINGLE"],
            [8, 9, "SINGLE"],
            [10, 11, "DOUBLE"],
            [12, 13, "TRIPLE"]
        ]),
    )?;
    Ok(())
}

#[test]
fn opaque_non_utf8_nul_and_bare_cr_application_bytes_survive_read_and_save() -> TestResult {
    let directory = Directory::new()?;
    let input = directory.0.join("opaque-化学.smi");
    let output = directory.0.join("opaque-copy.data");
    let bytes = b"CC\t \xff\x00\xc3\xa9\rprivate-opaque-flag\r\n[OH2]\t\x80tail\r";
    fs::write(&input, bytes)?;
    let loaded = read_smiles_file(&input, &Cancellation::default())?;
    assert_eq!(loaded.document.as_bytes(), bytes);
    assert_eq!(loaded.document.records().len(), 2);
    for (info, expected) in loaded.document.records().iter().zip([
        b"\xff\x00\xc3\xa9\rprivate-opaque-flag".as_slice(),
        b"\x80tail\r".as_slice(),
    ]) {
        let range = info.application_data_range().ok_or("application data")?;
        assert_eq!(bytes.get(range), Some(expected));
    }
    assert!(!format!("{:?}", loaded.document).contains("private-opaque-flag"));
    let receipt = save_smiles_file(&output, &loaded.document, false, &Cancellation::default())?;
    assert_eq!(receipt.byte_length, bytes.len());
    assert_eq!(fs::read(&output)?, bytes);
    assert_eq!(
        read_smiles_file(&output, &Cancellation::default())?.document,
        loaded.document
    );
    assert_external(
        b"CC",
        &[6, 6],
        &[0, 0],
        &[0, 0],
        &serde_json::json!([[0, 1, "SINGLE"]]),
    )?;
    assert_external(b"[OH2]", &[8], &[0], &[2], &serde_json::json!([]))?;
    Ok(())
}
