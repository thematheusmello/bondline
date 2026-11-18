use bondline_io::StoredDocument;
use bondline_model::{DrawingStyle, PageId, PageSize};
use serde_json::json;
use std::{
    borrow::Cow,
    io::Write,
    process::{Command, Stdio},
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const ORIGINAL: &str = include_str!("fixtures/bondline-v0.cdxml");

fn consume(raw: &[u8], expected: Option<&str>) -> TestResult {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| "python3".into());
    let mut child = Command::new(python)
        .args([
            "-B",
            "-c",
            r#"
import sys,xml.parsers.expat as expat
from decimal import Decimal
root=[];p=expat.ParserCreate(namespace_separator='|')
p.StartElementHandler=lambda n,a:root.append((n,a))
p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
p.ExternalEntityRefHandler=lambda *unused:0
p.Parse(sys.stdin.buffer.read(),True)
assert root[0][0]=='CDXML'
actual=root[0][1].get('MarginWidth');expected=sys.argv[1]
assert (actual is None)==(expected=='absent')
if actual is not None:assert Decimal(actual).is_finite() and Decimal(actual)==Decimal(expected)
assert [name for name,_ in root]==['CDXML','page','fragment','n','n','b']
"#,
        ])
        .arg(expected.unwrap_or("absent"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("oracle stdin")?.write_all(raw)?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}

#[test]
fn explicit_root_margin_projects_and_survives_native_and_source_aware_edits() -> TestResult {
    let source = ORIGINAL.replacen("<CDXML ", "<CDXML MarginWidth='2.0000' ", 1);
    let mut stored = StoredDocument::from_cdxml(source.as_bytes().into(), PageSize::A4)?;
    assert_eq!(
        serde_json::to_value(stored.model().drawing_style())?,
        json!({"margin_width_pt":2.0})
    );
    assert!(stored.is_fully_mapped());
    assert!(matches!(stored.to_cdxml()?, Cow::Borrowed(_)));
    consume(stored.to_cdxml()?.as_ref(), Some("2"))?;
    stored
        .model_mut()
        .set_page_size(PageId::new(1).ok_or("page")?, PageSize::new(400.0, 600.0)?)?;
    let edited = stored.to_cdxml()?.into_owned();
    let expected = source
        .replace("Width=\"612\"", "Width=\"400\"")
        .replace("Height=\"792\"", "Height=\"600\"");
    assert_eq!(edited, expected.as_bytes());
    consume(&edited, Some("2"))?;
    let mut native = StoredDocument::default();
    *native.model_mut() = stored.model().clone();
    let bytes = native.to_cdxml()?.into_owned();
    consume(&bytes, Some("2"))?;
    let reopened = StoredDocument::from_cdxml(bytes.into(), PageSize::A4)?;
    assert_eq!(reopened.model(), native.model());
    Ok(())
}

#[test]
fn margin_deltas_preserve_other_lexemes_and_omission_is_still_exact_legacy() -> TestResult {
    let source = ORIGINAL.replacen(
        "<CDXML ",
        "<CDXML LineWidth='1.5000' BondSpacing='15.000' ",
        1,
    );
    let mut stored = StoredDocument::from_cdxml(source.as_bytes().into(), PageSize::A4)?;
    consume(stored.to_cdxml()?.as_ref(), None)?;
    let style: DrawingStyle = serde_json::from_value(
        json!({"line_width_pt":1.5,"bond_spacing_percent":15.0,"margin_width_pt":0.125}),
    )?;
    stored.model_mut().set_drawing_style(style);
    let edited = stored.to_cdxml()?.into_owned();
    assert!(std::str::from_utf8(&edited)?.contains("LineWidth='1.5000' BondSpacing='15.000'"));
    consume(&edited, Some("0.125"))?;
    stored.model_mut().set_drawing_style(serde_json::from_value(
        json!({"line_width_pt":1.5,"bond_spacing_percent":15.0}),
    )?);
    assert_eq!(stored.to_cdxml()?.as_ref(), source.as_bytes());
    let empty = StoredDocument::from_cdxml(ORIGINAL.as_bytes().into(), PageSize::A4)?;
    assert_eq!(empty.to_cdxml()?.as_ref(), ORIGINAL.as_bytes());
    Ok(())
}

#[test]
fn unsupported_margin_lexemes_and_owners_remain_opaque_and_byte_exact() -> TestResult {
    for value in ["-1", "-1e-9999", "NaN", "inf", "100001", "unknown"] {
        let source = ORIGINAL.replacen("<CDXML ", &format!("<CDXML MarginWidth='{value}' "), 1);
        let mut stored = StoredDocument::from_cdxml(source.as_bytes().into(), PageSize::A4)?;
        assert!(!stored.is_fully_mapped(), "{value}");
        assert_eq!(stored.to_cdxml()?.as_ref(), source.as_bytes());
        stored
            .model_mut()
            .set_page_size(PageId::new(1).ok_or("page")?, PageSize::A4)?;
        assert!(stored.to_cdxml().is_err());
    }
    for source in [
        ORIGINAL.replacen("<n ", "<n MarginWidth='2' ", 1),
        ORIGINAL.replacen("<b ", "<b MarginWidth='2' ", 1),
        ORIGINAL.replacen(
            "<CDXML ",
            "<CDXML xmlns:q='urn:future' q:MarginWidth='2' ",
            1,
        ),
    ] {
        let stored = StoredDocument::from_cdxml(source.as_bytes().into(), PageSize::A4)?;
        assert!(!stored.is_fully_mapped());
        assert_eq!(stored.to_cdxml()?.as_ref(), source.as_bytes());
    }
    Ok(())
}

#[test]
fn margin_only_drift_refuses_whole_mol_and_other_retained_source_reuse() -> TestResult {
    use bondline_io::{
        DocumentError,
        mol::{MolTransform, Version},
        smiles::{RecordSelection, SmilesFileDocument},
    };
    use bondline_model::Point2;
    use bondline_render::Cancellation;
    use std::sync::Arc;
    let frame = MolTransform::new(24.0, Point2::new(72.0, 72.0)?)?;
    for (raw, version) in [
        (
            include_bytes!("fixtures/mol/ethane-v2000.mol").as_slice(),
            Version::V2000,
        ),
        (
            include_bytes!("fixtures/v3000/noncontiguous-cc.mol").as_slice(),
            Version::V3000,
        ),
    ] {
        let original = StoredDocument::from_mol(raw.into(), PageSize::A4, frame)?;
        assert_eq!(original.to_mol_as(version, frame)?.as_ref(), raw);
        for margin in [0.0, 2.0] {
            let mut changed = original.clone();
            changed
                .model_mut()
                .set_drawing_style(DrawingStyle::default().with_margin_width(Some(margin))?);
            assert!(matches!(
                changed.to_mol_as(version, frame),
                Err(DocumentError::UnsupportedMolLayout(_))
            ));
            assert!(changed.export_mol().is_err());
            changed
                .model_mut()
                .set_drawing_style(DrawingStyle::default());
            assert_eq!(changed.to_mol_as(version, frame)?.as_ref(), raw);
        }
    }
    let raw = include_bytes!("fixtures/cdx/metadata-explicit.cdx").as_slice();
    let mut binary = StoredDocument::from_cdx(raw.into(), PageSize::A4)?;
    binary
        .model_mut()
        .set_drawing_style(DrawingStyle::default().with_margin_width(Some(0.0))?);
    assert!(binary.to_cdx().is_err());
    binary
        .model_mut()
        .set_drawing_style(DrawingStyle::default());
    assert_eq!(binary.to_cdx()?.as_ref(), raw);
    let raw = b"CC one\r\nO two\topaque-name\n";
    let mut smiles = StoredDocument::from_smiles_records(
        SmilesFileDocument::parse(Arc::from(raw.as_slice()))?,
        RecordSelection::All,
        48.0,
        Point2::new(72.0, 72.0)?,
        PageSize::A4,
        &Cancellation::default(),
    )?;
    smiles
        .model_mut()
        .set_drawing_style(DrawingStyle::default().with_margin_width(Some(0.0))?);
    assert!(smiles.smiles_file().is_err());
    smiles
        .model_mut()
        .set_drawing_style(DrawingStyle::default());
    assert_eq!(smiles.smiles_file()?.as_bytes(), raw);
    Ok(())
}
