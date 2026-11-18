//! Existing-public-API runtime declaration; no future style type is referenced.
use bondline_io::StoredDocument;
use bondline_model::PageSize;
use serde_json::json;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const SOURCE: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<CDXML LineWidth='.500' CreationProgram="Bondline 0.1.0">
 <!-- original lexical witness -->
 <page id='1' DrawingSpace="Poster" Width='612.00' Height="792">
  <fragment id="2"><n id='3' NodeType="Element" Element='6' Charge="0" Radical='None' p="100.0 200" NumHydrogens='3'/><n id="4" NodeType='Element' Element="6" Charge='0' Radical="None" p='140 200.00' NumHydrogens="3"/><b id='5' B="3" E='4' Order="1"/></fragment>
 </page>
</CDXML>
"#;

#[test]
fn explicit_point_line_width_is_a_complete_source_preserving_native_style() -> TestResult {
    let stored = StoredDocument::from_cdxml(SOURCE.into(), PageSize::A4)?;
    assert_eq!(stored.to_cdxml()?.as_ref(), SOURCE);
    assert!(
        stored.is_fully_mapped(),
        "the closed profile must model explicit root LineWidth"
    );
    assert_eq!(
        serde_json::to_value(stored.model())?["drawing_style"],
        json!({"line_width_pt":0.5})
    );
    let fragment = &stored.model().pages()[0].fragments()[0];
    assert_eq!((fragment.atoms().len(), fragment.bonds().len()), (2, 1));
    assert_eq!(
        fragment
            .atoms()
            .iter()
            .map(|a| a.explicit_hydrogens)
            .collect::<Vec<_>>(),
        [Some(3), Some(3)]
    );
    assert!(!stored.source_info().ok_or("source")?.modified);
    Ok(())
}

// Expanded acceptance retains the exact initial1,489-byte declaration prefix.
use bondline_io::{
    DocumentError,
    mol::{MolTransform, Version},
    smiles::{RecordSelection, SmilesFileDocument},
};
use bondline_model::{BondOrder, DrawingStyle, Fragment, Point2};
use bondline_render::Cancellation;
use serde_json::Value;
use std::{
    borrow::Cow,
    io::Write as _,
    process::{Command, Stdio},
    sync::Arc,
};

const LEGACY: &[u8] = include_bytes!("fixtures/bondline-v0.cdxml");
const MOL2: &[u8] = include_bytes!("fixtures/mol/ethane-v2000.mol");
const MOL3: &[u8] = include_bytes!("fixtures/v3000/noncontiguous-cc.mol");
const CDX: &[u8] = include_bytes!("fixtures/cdx/metadata-explicit.cdx");

fn set_style(stored: &mut StoredDocument, value: Option<f64>) -> TestResult {
    stored
        .model_mut()
        .set_drawing_style(DrawingStyle::new(value)?);
    Ok(())
}

fn double(stored: &mut StoredDocument) -> TestResult {
    let page = stored.model().pages()[0].id();
    let fragment = stored.model().pages()[0].fragments()[0].as_ref();
    let mut atoms = fragment.atoms().to_vec();
    for atom in &mut atoms {
        atom.explicit_hydrogens = Some(2);
    }
    let mut bonds = fragment.bonds().to_vec();
    bonds[0].order = BondOrder::Double;
    let replacement = Fragment::new(fragment.id(), atoms, bonds)?;
    stored.model_mut().replace_fragment(page, replacement)?;
    Ok(())
}

fn inspect(raw: &[u8]) -> TestResult<Value> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(python).args(["-X", "utf8", "-B", "-c", r#"
import json,sys,xml.parsers.expat as expat
from decimal import Decimal
raw=sys.stdin.buffer.read(1048577)
assert 0<len(raw)<=1048576
nodes=[]
p=expat.ParserCreate(namespace_separator='|');p.ordered_attributes=True
p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
p.StartElementHandler=lambda n,a:nodes.append((n,list(zip(a[::2],a[1::2]))))
p.Parse(raw,True)
assert [n for n,_ in nodes]==['CDXML','page','fragment','n','n','b']
root=nodes[0][1];attrs=dict(root)
width=str(Decimal(attrs['LineWidth'])) if 'LineWidth' in attrs else None
if width is not None: assert Decimal(width).is_finite()
atoms=[dict(a) for n,a in nodes if n=='n'];bonds=[dict(a) for n,a in nodes if n=='b']
assert [a['Element'] for a in atoms]==['6','6']
assert [a['id'] for a in atoms]==['3','4']
assert [tuple(map(Decimal,a['p'].split())) for a in atoms]==[(Decimal(100),Decimal(200)),(Decimal(140),Decimal(200))]
print(json.dumps({'root':root,'line_width':width,'hydrogens':[a['NumHydrogens'] for a in atoms],'bonds':[[b['B'],b['E'],b['Order']] for b in bonds]}))
"#]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    child.stdin.take().ok_or("Expat stdin")?.write_all(raw)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

#[test]
fn absent_style_is_not_a_default_and_canonical_output_orders_only_present_root_keys() -> TestResult
{
    // Qualify the independent reader on original manual bytes before writer output.
    assert_eq!(
        inspect(SOURCE)?,
        json!({"root":[["LineWidth",".500"],["CreationProgram","Bondline 0.1.0"]],"line_width":"0.500","hydrogens":["3","3"],"bonds":[["3","4","1"]]})
    );
    let alternate = std::str::from_utf8(SOURCE)?.replacen(".500", ".750", 1);
    assert_eq!(inspect(alternate.as_bytes())?["line_width"], "0.750");
    let mut native = StoredDocument::from_cdxml(LEGACY.into(), PageSize::A4)?;
    assert!(native.is_fully_mapped());
    assert!(
        serde_json::to_value(native.model())?
            .get("drawing_style")
            .is_none()
    );
    assert!(matches!(native.to_cdxml()?, Cow::Borrowed(_)));
    assert_eq!(native.to_cdxml()?.as_ref(), LEGACY);
    let pages = serde_json::to_value(native.model())?["pages"].clone();
    set_style(&mut native, None)?;
    assert_eq!(native.to_cdxml()?.as_ref(), LEGACY);
    assert!(
        serde_json::to_value(native.model())?
            .get("drawing_style")
            .is_none()
    );
    native
        .model_mut()
        .set_drawing_settings(bondline_model::DrawingSettings::new(
            Some(48.0),
            Some(120.0),
        )?);
    set_style(&mut native, Some(0.5))?;
    assert_eq!(serde_json::to_value(native.model())?["pages"], pages);
    let output = native.to_cdxml()?;
    let expected = std::str::from_utf8(LEGACY)?.replacen(
        "CreationProgram=\"Bondline 0.1.0\"",
        "CreationProgram=\"Bondline 0.1.0\" BondLength=\"48\" ChainAngle=\"120\" LineWidth=\"0.5\"",
        1,
    );
    assert_eq!(output.as_ref(), expected.as_bytes());
    assert_eq!(
        inspect(&output)?,
        json!({"root":[["CreationProgram","Bondline 0.1.0"],["BondLength","48"],["ChainAngle","120"],["LineWidth","0.5"]],"line_width":"0.5","hydrogens":["3","3"],"bonds":[["3","4","1"]]})
    );
    let reopened = StoredDocument::from_cdxml(output.into_owned().into(), PageSize::A4)?;
    assert!(reopened.is_fully_mapped());
    assert_eq!(reopened.model(), native.model());
    Ok(())
}

#[test]
fn preserved_updates_touch_only_changed_values_and_absent_key_slots_then_restore_original()
-> TestResult {
    let mut stored = StoredDocument::from_cdxml(SOURCE.into(), PageSize::A4)?;
    let original = stored.clone();
    let pages = stored.model().pages().to_vec();
    set_style(&mut stored, Some(2.0))?;
    let expected = std::str::from_utf8(SOURCE)?.replacen("LineWidth='.500'", "LineWidth='2'", 1);
    assert_eq!(stored.to_cdxml()?.as_ref(), expected.as_bytes());
    assert_eq!(stored.model().pages(), pages.as_slice());
    assert!(stored.source_info().ok_or("source")?.modified);
    assert_eq!(inspect(&stored.to_cdxml()?)?["line_width"], "2");
    set_style(&mut stored, None)?;
    let removed = std::str::from_utf8(SOURCE)?.replacen("LineWidth='.500'", "", 1);
    assert_eq!(stored.to_cdxml()?.as_ref(), removed.as_bytes());
    assert!(
        serde_json::to_value(stored.model())?
            .get("drawing_style")
            .is_none()
    );
    assert_eq!(inspect(&stored.to_cdxml()?)?["line_width"], Value::Null);
    set_style(&mut stored, Some(0.5))?;
    assert!(!stored.source_info().ok_or("source")?.modified);
    assert!(matches!(stored.to_cdxml()?, Cow::Borrowed(_)));
    assert_eq!(stored.to_cdxml()?.as_ref(), SOURCE);
    let mut absent = StoredDocument::from_cdxml(removed.as_bytes().into(), PageSize::A4)?;
    assert!(absent.is_fully_mapped());
    set_style(&mut absent, Some(0.5))?;
    let inserted = removed.replacen(
        "CreationProgram=\"Bondline 0.1.0\">",
        "CreationProgram=\"Bondline 0.1.0\" LineWidth=\"0.5\">",
        1,
    );
    assert_eq!(absent.to_cdxml()?.as_ref(), inserted.as_bytes());
    // Chemical edits remain source-qualified with the styled root intact.
    double(&mut stored)?;
    let edited = std::str::from_utf8(SOURCE)?
        .replace("NumHydrogens='3'", "NumHydrogens='2'")
        .replace("NumHydrogens=\"3\"", "NumHydrogens=\"2\"")
        .replace("Order=\"1\"", "Order=\"2\"");
    assert_eq!(stored.to_cdxml()?.as_ref(), edited.as_bytes());
    assert_eq!(
        inspect(&stored.to_cdxml()?)?["hydrogens"],
        json!(["2", "2"])
    );
    assert_eq!(
        inspect(&stored.to_cdxml()?)?["bonds"],
        json!([["3", "4", "2"]])
    );
    assert_eq!(original.to_cdxml()?.as_ref(), SOURCE);
    let flagged = std::str::from_utf8(SOURCE)?.replacen(
        "NumHydrogens='3'",
        "NumHydrogens='3' ShowTerminalCarbonLabels='no' ShowNonTerminalCarbonLabels='no'",
        1,
    );
    let mut styled = StoredDocument::from_cdxml(flagged.as_bytes().into(), PageSize::A4)?;
    assert!(styled.is_fully_mapped());
    let pages = styled.model().pages().to_vec();
    set_style(&mut styled, Some(2.0))?;
    assert_eq!(styled.model().pages(), pages.as_slice());
    assert_eq!(
        styled.to_cdxml()?.as_ref(),
        flagged
            .replacen("LineWidth='.500'", "LineWidth='2'", 1)
            .as_bytes()
    );
    let atom = &styled.model().pages()[0].fragments()[0].atoms()[0];
    assert_eq!(
        (
            atom.explicit_hydrogens,
            atom.show_terminal_carbon_labels,
            atom.show_non_terminal_carbon_labels
        ),
        (Some(3), Some(false), Some(false))
    );
    Ok(())
}

#[test]
fn invalid_nested_and_foreign_styles_remain_opaque_without_flattening_or_source_loss() -> TestResult
{
    let base = std::str::from_utf8(LEGACY)?;
    for (literal, expected) in [(".500", 0.5), ("1e-3", 0.001), ("100000", 100000.0)] {
        let raw = base.replacen("<CDXML ", &format!("<CDXML LineWidth='{literal}' "), 1);
        let stored = StoredDocument::from_cdxml(raw.as_bytes().into(), PageSize::A4)?;
        assert!(stored.is_fully_mapped(), "valid {literal}");
        assert_eq!(
            serde_json::to_value(stored.model())?["drawing_style"],
            json!({"line_width_pt":expected})
        );
        assert_eq!(stored.to_cdxml()?.as_ref(), raw.as_bytes());
    }
    let mut opaque = Vec::new();
    for literal in [
        "",
        "0",
        "-0",
        "-1",
        "100000.0001",
        "NaN",
        "INF",
        "not-a-number",
    ] {
        opaque.push((
            base.replacen("<CDXML ", &format!("<CDXML LineWidth='{literal}' "), 1),
            None,
        ));
    }
    for target in ["<n id=\"3\"", "<b id=\"5\""] {
        opaque.push((
            base.replacen(target, &format!("{target} LineWidth=\"2\""), 1),
            None,
        ));
    }
    opaque.push((
        base.replacen("<CDXML ", "<CDXML LineWidth='0.5' BoldWidth='2' ", 1),
        Some(0.5),
    ));
    opaque.push((
        base.replacen("Bondline 0.1.0", "Independent foreign producer", 1)
            .replacen("<CDXML ", "<CDXML LineWidth='0.5' ", 1),
        Some(0.5),
    ));
    for (raw, expected) in opaque {
        let mut stored = StoredDocument::from_cdxml(raw.as_bytes().into(), PageSize::A4)?;
        assert!(!stored.is_fully_mapped(), "opaque {raw}");
        let model = serde_json::to_value(stored.model())?;
        match expected {
            Some(value) => assert_eq!(model["drawing_style"], json!({"line_width_pt":value})),
            None => assert!(model.get("drawing_style").is_none()),
        }
        assert_eq!(stored.export_cdxml()?, raw.as_bytes());
        assert_eq!(stored.to_cdxml()?.as_ref(), raw.as_bytes());
        double(&mut stored)?;
        assert!(matches!(
            stored.to_cdxml(),
            Err(DocumentError::UnsupportedEdit)
        ));
    }
    Ok(())
}

#[test]
fn style_only_model_drift_cannot_reuse_mol_cdx_or_synchronized_smiles_bytes() -> TestResult {
    let frame = MolTransform::new(24.0, Point2::new(72.0, 72.0)?)?;
    // A4 removes the older page-layout refusal; no atom-display flag is present.
    let native = std::str::from_utf8(LEGACY)?.replace(
        "Width=\"612\" Height=\"792\"",
        &format!(
            "Width=\"{}\" Height=\"{}\"",
            PageSize::A4.width_pt(),
            PageSize::A4.height_pt()
        ),
    );
    let plain = StoredDocument::from_cdxml(native.as_bytes().into(), PageSize::A4)?;
    assert!(plain.is_fully_mapped());
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
        let mut styled = original.clone();
        set_style(&mut styled, Some(0.5))?;
        for version in [Version::V2000, Version::V3000] {
            assert!(matches!(
                styled.to_mol_as(version, frame),
                Err(DocumentError::UnsupportedMolLayout(_))
            ));
        }
    }
    for (raw, version) in [(MOL2, Version::V2000), (MOL3, Version::V3000)] {
        let original = StoredDocument::from_mol(raw.into(), PageSize::A4, frame)?;
        assert!(matches!(
            original.to_mol_as(version, frame)?,
            Cow::Borrowed(_)
        ));
        let mut changed = original.clone();
        set_style(&mut changed, Some(0.5))?;
        assert!(matches!(
            changed.to_mol_as(version, frame),
            Err(DocumentError::UnsupportedMolLayout(_))
        ));
        assert!(changed.export_mol().is_err());
        assert_eq!(original.to_mol_as(version, frame)?.as_ref(), raw);
        set_style(&mut changed, None)?;
        assert!(matches!(
            changed.to_mol_as(version, frame)?,
            Cow::Borrowed(_)
        ));
        assert_eq!(changed.to_mol_as(version, frame)?.as_ref(), raw);
    }
    let binary = StoredDocument::from_cdx(CDX.into(), PageSize::A4)?;
    let mut changed = binary.clone();
    set_style(&mut changed, Some(0.5))?;
    assert!(changed.export_cdx().is_err());
    assert!(changed.to_cdx().is_err());
    assert_eq!(binary.export_cdx()?, CDX);
    set_style(&mut changed, None)?;
    assert_eq!(changed.export_cdx()?, CDX);
    let raw = b"CC one\r\nO two\topaque-name\n";
    let projected = StoredDocument::from_smiles_records(
        SmilesFileDocument::parse(Arc::from(raw.as_slice()))?,
        RecordSelection::All,
        48.0,
        Point2::new(72.0, 72.0)?,
        PageSize::A4,
        &Cancellation::default(),
    )?;
    let mut changed = projected.clone();
    set_style(&mut changed, Some(0.5))?;
    assert!(changed.smiles_file().is_err());
    assert_eq!(projected.smiles_file()?.as_bytes(), raw);
    set_style(&mut changed, None)?;
    assert_eq!(changed.smiles_file()?.as_bytes(), raw);
    assert_eq!(plain.to_cdxml()?.as_ref(), native.as_bytes());
    Ok(())
}

#[test]
fn complete_owner_style_overrides_stale_camera_style_without_authoring() -> TestResult {
    use bondline_model::{BondId, FragmentId, PageId};
    use bondline_render::{PageView, Stroke};
    let source = std::str::from_utf8(SOURCE)?
        .replace("Width='612.00'", "Width='256'")
        .replace("Height=\"792\"", "Height=\"256\"")
        .replace("p=\"100.0 200\"", "p=\"32 64.125\"")
        .replace("p='140 200.00'", "p='96 64.125'");
    let mut stored = StoredDocument::from_cdxml(source.as_bytes().into(), PageSize::A4)?;
    assert!(stored.is_fully_mapped());
    let page = PageId::new(1).ok_or("page")?;
    let cancel = Cancellation::default();
    for (owner_width, stale_width) in [(0.5, 2.0), (2.0, 0.5)] {
        set_style(&mut stored, Some(owner_width))?;
        let snapshot = stored.model_snapshot();
        let view = PageView::fit(576, 576, stored.model().pages()[0].size())?
            .with_drawing_style(DrawingStyle::new(Some(stale_width))?);
        let expected = vec![Stroke {
            begin: [96.0, 160.25],
            end: [224.0, 160.25],
            width: (owner_width * 2.0) as f32,
            color: [0, 0, 0, 255],
        }];
        for scene in [
            bondline_io::display::drawing_page_scene(576, 576, &stored, page, &cancel)?,
            bondline_io::display::drawing_page_scene_with_view(&view, &stored, page, &cancel)?,
        ] {
            assert_eq!(
                scene
                    .bond_strokes(
                        FragmentId::new(1).ok_or("fragment")?,
                        BondId::new(1).ok_or("bond")?
                    )
                    .cloned()
                    .collect::<Vec<_>>(),
                expected
            );
            assert_eq!(scene.label_em_px(), Some(24.0));
        }
        assert!(Arc::ptr_eq(&snapshot, &stored.model_snapshot()));
    }
    Ok(())
}
