//! Independent literal CDXML; no future Rust API needed for this first runtime RED.
use bondline_io::StoredDocument;
use bondline_model::PageSize;
use serde_json::json;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const SOURCE: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<CDXML BondSpacing='18.00' LineWidth='.500' CreationProgram="Bondline 0.1.0">
 <page id='1' DrawingSpace="Poster" Width='612.00' Height="792">
  <fragment id="2"><n id='3' NodeType="Element" Element='6' Charge="0" Radical='None' p="100.0 200" NumHydrogens='3'/><n id="4" NodeType='Element' Element="6" Charge='0' Radical="None" p='140 200.00' NumHydrogens="3"/><b id='5' B="3" E='4' Order="1"/></fragment>
 </page>
</CDXML>
"#;

#[test]
fn root_percent_is_complete_data_with_exact_original_bytes() -> TestResult {
    let stored = StoredDocument::from_cdxml(SOURCE.into(), PageSize::A4)?;
    assert_eq!(stored.to_cdxml()?.as_ref(), SOURCE);
    assert!(
        stored.is_fully_mapped(),
        "supported root spacing must not remain opaque"
    );
    assert_eq!(
        serde_json::to_value(stored.model())?["drawing_style"],
        json!({"line_width_pt":0.5,"bond_spacing_percent":18.0})
    );
    assert!(!stored.source_info().ok_or("source")?.modified);
    Ok(())
}

use bondline_io::{
    DocumentError, cdxml,
    mol::{MolTransform, Version},
    smiles::{RecordSelection, SmilesFileDocument},
};
use bondline_model::{DrawingStyle, Point2};
use bondline_render::Cancellation;
use std::{
    borrow::Cow,
    io::Write,
    process::{Command, Output, Stdio},
    sync::Arc,
};

const LEGACY: &[u8] = include_bytes!("fixtures/bondline-v0.cdxml");
const MOL2: &[u8] = include_bytes!("fixtures/mol/ethane-v2000.mol");
const MOL3: &[u8] = include_bytes!("fixtures/v3000/noncontiguous-cc.mol");
const CDX: &[u8] = include_bytes!("fixtures/cdx/metadata-explicit.cdx");

fn probe(raw: &[u8], width: &str, spacing: &str) -> TestResult<Output> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(python).args(["-X", "utf8", "-B", "-c", r#"
import sys,xml.parsers.expat as expat
from decimal import Decimal
rows=[]
p=expat.ParserCreate(namespace_separator='|')
p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
p.StartElementHandler=lambda n,a:rows.append((n,a))
p.Parse(sys.stdin.buffer.read(),True)
assert [n for n,_ in rows]==['CDXML','page','fragment','n','n','b']
root=rows[0][1]
for name,value in [('LineWidth',sys.argv[1]),('BondSpacing',sys.argv[2])]:
    if value=='absent': assert name not in root
    else:
        number=Decimal(root[name])
        assert number.is_finite() and number==Decimal(value),(name,root.get(name),value)
atoms=[a for n,a in rows if n=='n'];bonds=[a for n,a in rows if n=='b']
assert [[a['id'],a['Element'],a['NumHydrogens']] for a in atoms]==[['3','6','3'],['4','6','3']]
assert [tuple(map(Decimal,a['p'].split())) for a in atoms]==[(Decimal(100),Decimal(200)),(Decimal(140),Decimal(200))]
assert [[a['B'],a['E'],a['Order']] for a in bonds]==[['3','4','1']]
"#, width, spacing]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    child.stdin.take().ok_or("stdin")?.write_all(raw)?;
    Ok(child.wait_with_output()?)
}

fn inspect(raw: &[u8], width: &str, spacing: &str) -> TestResult {
    let out = probe(raw, width, spacing)?;
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    Ok(())
}

#[test]
fn spacing_and_width_deltas_preserve_each_others_original_numeric_lexemes() -> TestResult {
    // Qualify the external checker against independently authored XML first.
    inspect(SOURCE, ".5", "18")?;
    assert_eq!(probe(SOURCE, ".5", "19")?.status.code(), Some(1));
    let mut stored = StoredDocument::from_cdxml(SOURCE.into(), PageSize::A4)?;
    let original = stored.clone();
    let original_style = stored.model().drawing_style();
    stored
        .model_mut()
        .set_drawing_style(original_style.with_bond_spacing_percent(Some(24.0))?);
    let expected =
        std::str::from_utf8(SOURCE)?.replacen("BondSpacing='18.00'", "BondSpacing='24'", 1);
    inspect(expected.as_bytes(), ".5", "24")?;
    assert_eq!(stored.to_cdxml()?.as_ref(), expected.as_bytes());
    inspect(&stored.to_cdxml()?, ".5", "24")?;
    assert!(stored.is_fully_mapped());
    assert!(stored.source_info().ok_or("source")?.modified);
    assert_eq!(original.to_cdxml()?.as_ref(), SOURCE);
    stored
        .model_mut()
        .set_drawing_style(original_style.with_line_width(Some(2.0))?);
    let expected = std::str::from_utf8(SOURCE)?.replacen("LineWidth='.500'", "LineWidth='2'", 1);
    assert_eq!(stored.to_cdxml()?.as_ref(), expected.as_bytes());
    inspect(&stored.to_cdxml()?, "2", "18")?;
    stored
        .model_mut()
        .set_drawing_style(original_style.with_bond_spacing_percent(None)?);
    let expected = std::str::from_utf8(SOURCE)?.replacen("BondSpacing='18.00'", "", 1);
    assert_eq!(stored.to_cdxml()?.as_ref(), expected.as_bytes());
    inspect(&stored.to_cdxml()?, ".5", "absent")?;
    stored
        .model_mut()
        .set_drawing_style(original_style.with_line_width(None)?);
    let expected = std::str::from_utf8(SOURCE)?.replacen("LineWidth='.500'", "", 1);
    assert_eq!(stored.to_cdxml()?.as_ref(), expected.as_bytes());
    inspect(&stored.to_cdxml()?, "absent", "18")?;
    stored.model_mut().set_drawing_style(original_style);
    assert!(matches!(stored.to_cdxml()?, Cow::Borrowed(_)));
    assert_eq!(stored.to_cdxml()?.as_ref(), SOURCE);
    assert!(!stored.source_info().ok_or("source")?.modified);
    Ok(())
}

#[test]
fn native_bytes_and_absence_stay_exact_and_new_present_attributes_have_stable_order() -> TestResult
{
    let legacy = StoredDocument::from_cdxml(LEGACY.into(), PageSize::A4)?;
    assert!(legacy.is_fully_mapped());
    assert!(legacy.model().drawing_style().is_empty());
    assert_eq!(legacy.to_cdxml()?.as_ref(), LEGACY);
    assert_eq!(cdxml::write_document(legacy.model())?, LEGACY);
    let source = std::str::from_utf8(SOURCE)?.replacen("BondSpacing='18.00' ", "", 1);
    let mut stored = StoredDocument::from_cdxml(source.as_bytes().into(), PageSize::A4)?;
    let style = stored
        .model()
        .drawing_style()
        .with_bond_spacing_percent(Some(0.0))?;
    stored.model_mut().set_drawing_style(style);
    // Existing root attributes retain slots/quotes; a missing spacing is appended.
    let expected = source.replacen(
        "CreationProgram=\"Bondline 0.1.0\">",
        "CreationProgram=\"Bondline 0.1.0\" BondSpacing=\"0\">",
        1,
    );
    assert_eq!(stored.to_cdxml()?.as_ref(), expected.as_bytes());
    inspect(expected.as_bytes(), ".5", "0")?;
    let generated = cdxml::write_document(stored.model())?;
    assert!(std::str::from_utf8(&generated)?.contains(
        "<CDXML CreationProgram=\"Bondline 0.1.0\" LineWidth=\"0.5\" BondSpacing=\"0\">"
    ));
    inspect(&generated, ".5", "0")?;
    let reopened = StoredDocument::from_cdxml(generated.into(), PageSize::A4)?;
    assert!(reopened.is_fully_mapped());
    assert_eq!(reopened.model().drawing_style(), style);
    assert_eq!(stored.model().pages(), legacy.model().pages());
    Ok(())
}

#[test]
fn supported_root_literals_are_retained_but_invalid_abs_and_owner_styles_stay_opaque() -> TestResult
{
    for (literal, value) in [
        ("0", 0.0),
        ("-0.000", 0.0),
        ("1e-3", 0.001),
        ("18.000", 18.0),
        ("3276.7", 3276.7),
    ] {
        let source = std::str::from_utf8(SOURCE)?.replacen("18.00", literal, 1);
        let stored = StoredDocument::from_cdxml(source.as_bytes().into(), PageSize::A4)?;
        assert!(stored.is_fully_mapped(), "{literal}");
        assert_eq!(
            stored.model().drawing_style().bond_spacing_percent(),
            Some(value)
        );
        assert_eq!(stored.to_cdxml()?.as_ref(), source.as_bytes());
    }
    let base = std::str::from_utf8(SOURCE)?;
    let mut opaque = Vec::new();
    for literal in [
        "",
        "-0.001",
        "3276.7001",
        "6553.5",
        "NaN",
        "INF",
        "not-a-number",
    ] {
        opaque.push((base.replacen("18.00", literal, 1), None));
    }
    for target in ["<CDXML ", "<page ", "<fragment ", "<n ", "<b "] {
        opaque.push((
            base.replacen(target, &format!("{target}BondSpacingAbs='3' "), 1),
            Some(18.0),
        ));
    }
    for target in ["<page ", "<fragment ", "<n ", "<b "] {
        opaque.push((
            base.replacen(target, &format!("{target}BondSpacing='9' "), 1),
            Some(18.0),
        ));
    }
    opaque.push((
        base.replace(
            "BondSpacing='18.00'",
            "xmlns:f='urn:foreign' f:BondSpacing='18'",
        ),
        None,
    ));
    opaque.push((
        base.replace(
            "CreationProgram=\"Bondline 0.1.0\"",
            "CreationProgram=\"Foreign\"",
        ),
        Some(18.0),
    ));
    opaque.push((
        base.replacen("<CDXML ", "<CDXML UnknownStyle='x' ", 1),
        Some(18.0),
    ));
    opaque.push((base.replacen("</page>", "<opaque/></page>", 1), Some(18.0)));
    for (raw, expected) in opaque {
        let mut stored = StoredDocument::from_cdxml(raw.as_bytes().into(), PageSize::A4)?;
        assert!(!stored.is_fully_mapped(), "{raw}");
        assert_eq!(
            stored.model().drawing_style().bond_spacing_percent(),
            expected
        );
        assert_eq!(stored.to_cdxml()?.as_ref(), raw.as_bytes());
        assert_eq!(stored.export_cdxml()?, raw.as_bytes());
        let page = stored.model().pages()[0].id();
        stored
            .model_mut()
            .set_page_size(page, PageSize::new(613.0, 792.0)?)?;
        assert!(matches!(
            stored.to_cdxml(),
            Err(DocumentError::UnsupportedEdit)
        ));
    }
    Ok(())
}

#[test]
fn any_present_spacing_is_refused_by_whole_mol_cdx_and_retained_smiles_loss_guards() -> TestResult {
    let frame = MolTransform::new(24.0, Point2::new(72.0, 72.0)?)?;
    let native = std::str::from_utf8(LEGACY)?.replace(
        "Width=\"612\" Height=\"792\"",
        &format!(
            "Width=\"{}\" Height=\"{}\"",
            PageSize::A4.width_pt(),
            PageSize::A4.height_pt()
        ),
    );
    let plain = StoredDocument::from_cdxml(native.as_bytes().into(), PageSize::A4)?;
    let mut free = StoredDocument::default();
    let page = free.model().pages()[0].id();
    free.model_mut().append_fragments(
        page,
        vec![plain.model().pages()[0].fragments()[0].as_ref().clone()],
    )?;
    for original in [&plain, &free] {
        for version in [Version::V2000, Version::V3000] {
            assert!(original.to_mol_as(version, frame).is_ok());
        }
        for spacing in [0.0, 18.0] {
            let mut changed = original.clone();
            changed.model_mut().set_drawing_style(
                DrawingStyle::default().with_bond_spacing_percent(Some(spacing))?,
            );
            for version in [Version::V2000, Version::V3000] {
                assert!(matches!(
                    changed.to_mol_as(version, frame),
                    Err(DocumentError::UnsupportedMolLayout(_))
                ));
            }
        }
    }
    for (raw, version) in [(MOL2, Version::V2000), (MOL3, Version::V3000)] {
        let original = StoredDocument::from_mol(raw.into(), PageSize::A4, frame)?;
        assert!(matches!(
            original.to_mol_as(version, frame)?,
            Cow::Borrowed(_)
        ));
        let mut changed = original.clone();
        changed
            .model_mut()
            .set_drawing_style(DrawingStyle::default().with_bond_spacing_percent(Some(0.0))?);
        assert!(changed.to_mol_as(version, frame).is_err());
        assert!(changed.export_mol().is_err());
        changed
            .model_mut()
            .set_drawing_style(DrawingStyle::default());
        assert_eq!(changed.to_mol_as(version, frame)?.as_ref(), raw);
    }
    let binary = StoredDocument::from_cdx(CDX.into(), PageSize::A4)?;
    let mut changed = binary.clone();
    changed
        .model_mut()
        .set_drawing_style(DrawingStyle::default().with_bond_spacing_percent(Some(0.0))?);
    assert!(changed.to_cdx().is_err());
    assert!(changed.export_cdx().is_err());
    assert_eq!(binary.export_cdx()?, CDX);
    let raw = b"CC one\r\nO two\topaque\n";
    let projected = StoredDocument::from_smiles_records(
        SmilesFileDocument::parse(Arc::from(raw.as_slice()))?,
        RecordSelection::All,
        48.0,
        Point2::new(72.0, 72.0)?,
        PageSize::A4,
        &Cancellation::default(),
    )?;
    let mut changed = projected.clone();
    changed
        .model_mut()
        .set_drawing_style(DrawingStyle::default().with_bond_spacing_percent(Some(0.0))?);
    assert!(!changed.has_smiles_projection());
    assert!(changed.smiles_file().is_err());
    let page = changed.model().pages()[0].id();
    let fragment = changed.model().pages()[0].fragments()[0].as_ref().clone();
    assert!(
        changed
            .with_smiles_fragment(page, fragment, &Cancellation::default())
            .is_err()
    );
    assert_eq!(projected.smiles_file()?.as_bytes(), raw);
    changed
        .model_mut()
        .set_drawing_style(DrawingStyle::default());
    assert_eq!(changed.smiles_file()?.as_bytes(), raw);
    Ok(())
}

#[test]
fn negative_nonzero_underflow_cannot_be_certified_as_numeric_zero() -> TestResult {
    // Supplemental source-boundary declaration after the first five cases passed.
    // Expat/Decimal distinguishes real negative magnitude from f64 rounding to -0.
    let base = std::str::from_utf8(SOURCE)?;
    for (literal, decoded) in [
        ("-1e-9999", "-1e-9999"),
        ("-.0001e-9999", "-.0001e-9999"),
        ("&#45;1e-9999", "-1e-9999"),
        (" -1e-9999 ", "-1e-9999"),
    ] {
        let raw = base.replacen("18.00", literal, 1);
        inspect(raw.as_bytes(), ".5", decoded)?;
        let mut stored = StoredDocument::from_cdxml(raw.as_bytes().into(), PageSize::A4)?;
        assert!(
            !stored.is_fully_mapped(),
            "negative nonzero source {literal}"
        );
        assert_eq!(stored.model().drawing_style().bond_spacing_percent(), None);
        assert_eq!(stored.to_cdxml()?.as_ref(), raw.as_bytes());
        let page = stored.model().pages()[0].id();
        stored
            .model_mut()
            .set_page_size(page, PageSize::new(613.0, 792.0)?)?;
        assert!(matches!(
            stored.to_cdxml(),
            Err(DocumentError::UnsupportedEdit)
        ));
    }
    for literal in ["-0e-9999", "-0.000e-9999"] {
        let raw = base.replacen("18.00", literal, 1);
        inspect(raw.as_bytes(), ".5", "0")?;
        let stored = StoredDocument::from_cdxml(raw.as_bytes().into(), PageSize::A4)?;
        assert!(stored.is_fully_mapped());
        assert_eq!(
            stored.model().drawing_style().bond_spacing_percent(),
            Some(0.0)
        );
        assert_eq!(stored.to_cdxml()?.as_ref(), raw.as_bytes());
    }
    Ok(())
}
