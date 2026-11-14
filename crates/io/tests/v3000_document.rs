use bondline_io::{
    StoredDocument,
    mol::{MolTransform, Version, v3000},
};
use bondline_model::{Document, Fragment, PageId, PageSize, Point2};
use serde_json::{Value, json};
use std::{
    borrow::Cow,
    io::Write,
    mem::size_of,
    process::{Command, Stdio},
    sync::Arc,
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const V2: &[u8] = include_bytes!("fixtures/mol/rdkit-CH3.mol");
const V3: &[u8] = include_bytes!("fixtures/v3000/noncontiguous-cc.mol");
const EXTENDED: &[u8] = include_bytes!("fixtures/v3000/continuations-and-extensions.mol");
fn frame() -> Result<MolTransform, Box<dyn std::error::Error>> {
    Ok(MolTransform::new(24.0, Point2::new(72.0, 72.0)?)?)
}
fn external(mode: &str, bytes: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(python)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/v3000_oracle.py"
        ))
        .arg(mode)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("oracle stdin")?.write_all(bytes)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "external RDKit: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(output.stdout)
}
fn describe(bytes: &[u8]) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(serde_json::from_slice(&external("describe", bytes)?)?)
}
fn edited(stored: &mut StoredDocument, position: Point2) -> TestResult {
    let fragment = stored
        .model()
        .pages()
        .first()
        .ok_or("page")?
        .fragments()
        .first()
        .ok_or("fragment")?;
    let mut atoms = fragment.atoms().to_vec();
    atoms.first_mut().ok_or("atom")?.position = Some(position);
    let changed = Fragment::new(fragment.id(), atoms, fragment.bonds().to_vec())?;
    let mut model = Document::default();
    model.append_fragments(PageId::new(1).ok_or("page")?, vec![changed])?;
    *stored.model_mut() = model;
    Ok(())
}

#[test]
fn auto_detected_v3000_owns_exact_source_and_shared_headers_without_changing_source_json()
-> TestResult {
    let bytes = external("emit", b"")?;
    let expected = bytes.clone();
    let source: Arc<[u8]> = bytes.into();
    let pointer = source.as_ptr();
    let stored = StoredDocument::from_mol(source, PageSize::A4, frame()?)?;
    assert_eq!(stored.mol_version(), Some(Version::V3000));
    assert_eq!(stored.mol_transform(), Some(frame()?));
    assert!(stored.is_fully_mapped());
    assert_eq!(stored.export_mol()?, expected);
    assert_eq!(stored.export_mol()?.as_ptr(), pointer);
    assert!(matches!(stored.to_mol(frame()?)?, Cow::Borrowed(_)));
    assert_eq!(
        stored.to_mol_as(Version::V3000, frame()?)?.as_ref(),
        expected
    );
    assert_eq!(
        serde_json::to_value(stored.source_info().ok_or("source")?)?,
        json!({"format":"mol","byte_length":314,"modified":false})
    );
    assert!(!format!("{stored:?}").contains("RDKit"));
    let clone = stored.clone();
    let once = StoredDocument::retained_bytes([&stored]);
    assert_eq!(StoredDocument::retained_bytes([&stored, &stored]), once);
    assert_eq!(
        StoredDocument::retained_bytes([&stored, &clone]) - once,
        size_of::<StoredDocument>()
    );
    assert!(stored.to_cdxml().is_err());
    Ok(())
}

#[test]
fn edited_v3000_keeps_its_headers_frame_version_and_full_precision_then_restores_exactly()
-> TestResult {
    let bytes = external("emit", b"")?;
    let mut stored = StoredDocument::from_mol(Arc::from(bytes.as_slice()), PageSize::A4, frame()?)?;
    let original = stored.clone();
    edited(&mut stored, Point2::new(100.0, 150.0)?)?;
    let ignored_native_frame = MolTransform::new(1.0, Point2::new(0.0, 0.0)?)?;
    let encoded = stored.to_mol(ignored_native_frame)?;
    assert!(matches!(encoded, Cow::Owned(_)));
    let raw = v3000::MolFile::parse(&encoded)?;
    assert_eq!(raw.header(), v3000::MolFile::parse(&bytes)?.header());
    assert_eq!(raw.atoms().first().ok_or("atom")?.x, 1.1666666666666667);
    assert_eq!(raw.atoms().first().ok_or("atom")?.y, -3.25);
    let external = describe(&encoded)?;
    assert_eq!(
        external["atoms"][0]["position"],
        json!([1.1666666666666667, -3.25, 0.0])
    );
    assert_eq!(external["atoms"][0]["isotope"], 13);
    assert!(stored.source_info().ok_or("source")?.modified);
    assert!(stored.export_mol().is_err());
    assert_eq!(
        stored
            .to_mol_as(Version::V2000, frame()?)
            .err()
            .ok_or("silent downgrade")?
            .code(),
        "unsupported_content"
    );
    assert_eq!(original.export_mol()?, bytes);
    *stored.model_mut() = original.model().clone();
    assert_eq!(stored, original);
    assert_eq!(stored.export_mol()?, bytes);
    Ok(())
}

#[test]
fn explicit_v2000_to_v3000_preserves_source_and_known_fields_but_v3000_downgrade_refuses()
-> TestResult {
    let v2 = StoredDocument::from_mol(Arc::from(V2), PageSize::A4, frame()?)?;
    assert_eq!(v2.mol_version(), Some(Version::V2000));
    assert!(v2.is_fully_mapped());
    assert_eq!(v2.to_mol(frame()?)?.as_ref(), V2);
    let converted = v2.to_mol_as(Version::V3000, frame()?)?;
    assert!(matches!(converted, Cow::Owned(_)));
    let raw = v3000::MolFile::parse(&converted)?;
    assert_eq!(raw.header(), bondline_io::mol::MolFile::parse(V2)?.header());
    assert_eq!(describe(&converted)?["atoms"][0]["explicit_h"], 3);
    assert_eq!(v2.export_mol()?, V2);
    assert_eq!(v2.mol_version(), Some(Version::V2000));
    let v3 = StoredDocument::from_mol(Arc::from(V3), PageSize::A4, frame()?)?;
    assert_eq!(
        v3.to_mol_as(Version::V2000, frame()?)
            .err()
            .ok_or("downgrade")?
            .code(),
        "unsupported_content"
    );
    assert_eq!(v3.export_mol()?, V3);
    let native = StoredDocument::default();
    assert_eq!(native.mol_version(), None);
    assert!(bondline_io::mol::MolFile::parse(&native.to_mol(frame()?)?).is_ok());
    assert!(v3000::MolFile::parse(&native.to_mol_as(Version::V3000, frame()?)?).is_ok());
    Ok(())
}

#[test]
fn v2000_rounding_proof_cannot_authorize_a_v3000_conversion_after_frame_precision_loss()
-> TestResult {
    // This is a valid f10.4 coordinate. IEEE754 at100 has an ULP of2^-46: applying
    // 1e-10 scale then subtracting the origin gives0.00014210854715202004, not0.0001.
    // V2000's4dp re-encoding rounds it back, whereas full-precision V3000 must refuse.
    let bytes = std::str::from_utf8(V2)?.replace("   -1.2375    2.1509", "    0.0001    0.0000");
    let tiny = MolTransform::new(1e-10, Point2::new(100.0, 200.0)?)?;
    let stored = StoredDocument::from_mol(Arc::from(bytes.as_bytes()), PageSize::A4, tiny)?;
    assert!(
        stored.is_fully_mapped(),
        "V2000's own format precision proof is valid"
    );
    assert_eq!(
        stored
            .to_mol_as(Version::V3000, frame()?)
            .err()
            .ok_or("lossy frame was advertised convertible")?
            .code(),
        "unsupported_content"
    );
    assert_eq!(stored.export_mol()?, bytes.as_bytes());
    let error = stored
        .to_mol_as(Version::V3000, frame()?)
        .err()
        .ok_or("conversion")?;
    assert!(!stored.source_info().ok_or("source")?.modified);
    assert!(
        !error.to_string().contains("the model changed"),
        "an unchanged conversion refusal must not falsely report an authored edit"
    );
    Ok(())
}

#[test]
fn large_v3000_and_invalid_imports_respect_source_layout_and_regeneration_guards() -> TestResult {
    let bytes = external("emit1000", b"")?;
    let mut stored = StoredDocument::from_mol(Arc::from(bytes.as_slice()), PageSize::A4, frame()?)?;
    assert_eq!(
        stored
            .model()
            .pages()
            .first()
            .ok_or("page")?
            .fragments()
            .first()
            .ok_or("fragment")?
            .atoms()
            .len(),
        1000
    );
    assert!(stored.is_fully_mapped());
    assert_eq!(stored.export_mol()?, bytes);
    let original = stored.clone();
    edited(&mut stored, Point2::new(100.0, 150.0)?)?;
    let rewritten = stored.to_mol(frame()?)?;
    assert_eq!(
        describe(&rewritten)?["atoms"]
            .as_array()
            .ok_or("external atom count")?
            .len(),
        1000
    );
    assert_eq!(
        v3000::MolFile::parse(&rewritten)?.header(),
        v3000::MolFile::parse(&bytes)?.header()
    );
    stored
        .model_mut()
        .set_page_size(PageId::new(1).ok_or("page")?, PageSize::new(612.0, 792.0)?)?;
    assert_eq!(
        stored.to_mol(frame()?).err().ok_or("layout loss")?.code(),
        "unsupported_content"
    );
    for bytes in [
        b"invalid MOL".to_vec(),
        std::str::from_utf8(V3)?
            .replace("COUNTS 2 1", "COUNTS 3 1")
            .into_bytes(),
        EXTENDED.to_vec(),
        std::str::from_utf8(V3)?
            .replace("V3000", "V2000")
            .into_bytes(),
    ] {
        let before = stored.clone();
        assert!(StoredDocument::from_mol(bytes.into(), PageSize::A4, frame()?).is_err());
        assert_eq!(stored, before);
    }
    let collapsed = StoredDocument::from_mol(
        Arc::from(V3),
        PageSize::A4,
        MolTransform::new(1e-300, Point2::new(100.0, 200.0)?)?,
    )?;
    assert!(!collapsed.is_fully_mapped());
    assert_eq!(collapsed.export_mol()?, V3);
    assert_eq!(original.export_mol()?, bytes);
    Ok(())
}

#[test]
fn empty_v3000_document_keeps_headers_and_version_through_its_first_authored_fragment() -> TestResult
{
    let bytes=b"Original empty V3000\n  Original          2D\nOriginal MIT OR Apache-2.0 header\n  0  0  0  0  0  0            999 V3000\nM  V30 BEGIN CTAB\nM  V30 COUNTS 0 0 0 0 0\nM  V30 BEGIN ATOM\nM  V30 END ATOM\nM  V30 END CTAB\nM  END\n";
    let mut stored = StoredDocument::from_mol(Arc::from(bytes.as_slice()), PageSize::A4, frame()?)?;
    assert_eq!(stored.mol_version(), Some(Version::V3000));
    assert!(stored.is_fully_mapped());
    assert!(
        stored
            .model()
            .pages()
            .first()
            .ok_or("page")?
            .fragments()
            .is_empty()
    );
    assert!(
        describe(bytes)?["atoms"]
            .as_array()
            .ok_or("external empty")?
            .is_empty()
    );
    let fragment = v3000::MolFile::parse(V3)?.to_fragment(frame()?)?;
    stored
        .model_mut()
        .append_fragments(PageId::new(1).ok_or("page")?, vec![fragment])?;
    let encoded = stored.to_mol(MolTransform::new(1.0, Point2::new(0.0, 0.0)?)?)?;
    let raw = v3000::MolFile::parse(&encoded)?;
    assert_eq!(
        raw.header(),
        [
            "Original empty V3000",
            "  Original          2D",
            "Original MIT OR Apache-2.0 header"
        ]
    );
    assert_eq!(
        describe(&encoded)?["atoms"]
            .as_array()
            .ok_or("external atoms")?
            .len(),
        2
    );
    Ok(())
}

#[test]
fn native_cdxml_defaults_to_v2000_but_explicit_v3000_requires_the_complete_profile() -> TestResult {
    let native = include_bytes!("fixtures/bondline-v0.cdxml");
    let mut known = StoredDocument::from_cdxml(Arc::from(native.as_slice()), PageSize::A4)?;
    assert!(known.is_fully_mapped());
    known
        .model_mut()
        .set_page_size(PageId::new(1).ok_or("page")?, PageSize::A4)?;
    assert!(bondline_io::mol::MolFile::parse(&known.to_mol(frame()?)?).is_ok());
    let encoded = known.to_mol_as(Version::V3000, frame()?)?;
    assert_eq!(
        describe(&encoded)?["atoms"]
            .as_array()
            .ok_or("external atoms")?
            .len(),
        2
    );
    assert_eq!(known.mol_version(), None);
    let unknown = include_bytes!("../../cdxml/tests/fixtures/unknown.cdxml");
    let opaque = StoredDocument::from_cdxml(Arc::from(unknown.as_slice()), PageSize::A4)?;
    let before = opaque.clone();
    assert_eq!(
        opaque
            .to_mol_as(Version::V3000, frame()?)
            .err()
            .ok_or("lost unknown XML")?
            .code(),
        "unsupported_content"
    );
    assert_eq!(opaque, before);
    assert_eq!(opaque.export_cdxml()?, unknown);
    Ok(())
}
