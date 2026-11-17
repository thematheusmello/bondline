//! Independent literal-field coverage, not a valence/chemical-validity claim.
use bondline_io::StoredDocument;
use bondline_model::{
    BondOrder, DoublePosition, DrawingSettings, Element, Fragment, PageSize, Point2, Radical,
};
use std::{
    borrow::Cow,
    io::Write,
    num::NonZeroU16,
    process::{Command, Stdio},
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn oracle(bytes: &[u8], expected: &str) -> TestResult {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(python).args(["-X", "utf8", "-B", "-c", r#"
import json, sys, xml.parsers.expat as expat
from decimal import Decimal
nodes=[]
p=expat.ParserCreate(namespace_separator='|')
p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
p.StartElementHandler=lambda name,attrs:nodes.append([name,attrs])
p.Parse(sys.stdin.buffer.read(),True)
assert nodes==json.loads(sys.argv[1]), nodes
for name,attrs in nodes:
    for key in ('BondLength','ChainAngle','Width','Height','Element','Charge','Isotope','NumHydrogens'):
        if key in attrs: assert Decimal(attrs[key]).is_finite(), (name,key)
    if 'p' in attrs:
        coordinates=attrs['p'].split()
        assert len(coordinates)==2 and all(Decimal(x).is_finite() for x in coordinates)
"#, expected]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    child.stdin.take().ok_or("oracle stdin")?.write_all(bytes)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

#[test]
fn complete_existing_scalar_delta_changes_only_its_literal_attribute_ranges() -> TestResult {
    let source = b"<?xml version='1.0' encoding='UTF-8'?>\r\n<CDXML ChainAngle=\"120.000\" CreationProgram='Bondline &#48;.1.0' BondLength='48.00'><!--root--><page Height='792.000' DrawingSpace='Poster' id='91' Width='612.00'><fragment id='400'><!--inside--><b E='207' id='900' Order='1' B='101' DoublePosition='Center'/><n NumHydrogens='&#51;' id='101' NodeType='Element' Element='6' p='100.00 200.000' Isotope='13' Charge='0' Radical='None' ShowTerminalCarbonLabels='yes' ShowNonTerminalCarbonLabels='no'></n><n id='207' NodeType='Element' Element='6' Charge='0' Radical='None' p='140 200' NumHydrogens='3'/></fragment></page></CDXML>";
    let mut stored = StoredDocument::from_cdxml(source.as_slice().into(), PageSize::A4)?;
    assert!(stored.is_fully_mapped());
    let original = stored.clone();
    let page = stored.model().pages()[0].id();
    let fragment = stored.model().pages()[0].fragments()[0].as_ref();
    let mut atoms = fragment.atoms().to_vec();
    let mut bonds = fragment.bonds().to_vec();
    atoms[0].element = Element::new(8).ok_or("oxygen")?;
    atoms[0].isotope = Some(NonZeroU16::new(18).ok_or("isotope")?);
    atoms[0].charge = -1;
    atoms[0].radical = Radical::Doublet;
    atoms[0].position = Some(Point2::new(125.5, 240.25)?);
    atoms[0].explicit_hydrogens = Some(1);
    atoms[0].show_terminal_carbon_labels = None;
    atoms[0].show_non_terminal_carbon_labels = Some(true);
    bonds[0].order = BondOrder::Double;
    bonds[0].double_position = DoublePosition::Right;
    let replacement = Fragment::new(fragment.id(), atoms, bonds)?;
    stored.model_mut().replace_fragment(page, replacement)?;
    stored
        .model_mut()
        .set_page_size(page, PageSize::new(640.0, 800.0)?)?;
    stored
        .model_mut()
        .set_drawing_settings(DrawingSettings::new(Some(64.0), None)?);
    assert!(stored.is_fully_mapped());
    let output = stored.to_cdxml()?;
    assert!(matches!(output, Cow::Owned(_)));
    let expected=b"<?xml version='1.0' encoding='UTF-8'?>\r\n<CDXML  CreationProgram='Bondline &#48;.1.0' BondLength='64'><!--root--><page Height='800' DrawingSpace='Poster' id='91' Width='640'><fragment id='400'><!--inside--><b E='207' id='900' Order='2' B='101' DoublePosition='Right'/><n NumHydrogens='1' id='101' NodeType='Element' Element='8' p='125.5 240.25' Isotope='18' Charge='-1' Radical='Doublet'  ShowNonTerminalCarbonLabels='yes'></n><n id='207' NodeType='Element' Element='6' Charge='0' Radical='None' p='140 200' NumHydrogens='3'/></fragment></page></CDXML>";
    assert_eq!(output.as_ref(), expected);
    assert_eq!(original.to_cdxml()?.as_ref(), source);
    assert!(stored.source_info().ok_or("source info")?.modified);
    oracle(
        &output,
        r#"[["CDXML",{"CreationProgram":"Bondline 0.1.0","BondLength":"64"}],["page",{"Height":"800","DrawingSpace":"Poster","id":"91","Width":"640"}],["fragment",{"id":"400"}],["b",{"E":"207","id":"900","Order":"2","B":"101","DoublePosition":"Right"}],["n",{"NumHydrogens":"1","id":"101","NodeType":"Element","Element":"8","p":"125.5 240.25","Isotope":"18","Charge":"-1","Radical":"Doublet","ShowNonTerminalCarbonLabels":"yes"}],["n",{"id":"207","NodeType":"Element","Element":"6","Charge":"0","Radical":"None","p":"140 200","NumHydrogens":"3"}]]"#,
    )
}

#[test]
fn absent_optional_fields_can_be_inserted_and_exact_baseline_restoration_is_borrowed() -> TestResult
{
    let source=b"<CDXML CreationProgram='Bondline 0.1.0'><page id='5' DrawingSpace='Poster' Width='612.00' Height='792.000'><fragment id='7'><n id='9' NodeType='Element' Element='6' Charge='0' Radical='None' /></fragment></page></CDXML>";
    let mut stored = StoredDocument::from_cdxml(source.as_slice().into(), PageSize::A4)?;
    assert!(stored.is_fully_mapped());
    let baseline = stored.model_snapshot();
    let page = stored.model().pages()[0].id();
    let f = stored.model().pages()[0].fragments()[0].as_ref();
    let mut atoms = f.atoms().to_vec();
    atoms[0].position = Some(Point2::new(72.0, 96.0)?);
    atoms[0].isotope = Some(NonZeroU16::new(13).ok_or("isotope")?);
    atoms[0].explicit_hydrogens = Some(4);
    atoms[0].show_terminal_carbon_labels = Some(false);
    atoms[0].show_non_terminal_carbon_labels = Some(true);
    let replacement = Fragment::new(f.id(), atoms, f.bonds().to_vec())?;
    stored.model_mut().replace_fragment(page, replacement)?;
    stored
        .model_mut()
        .set_drawing_settings(DrawingSettings::new(Some(48.0), Some(90.0))?);
    let output = stored.to_cdxml()?;
    assert_eq!(output.as_ref(),b"<CDXML CreationProgram='Bondline 0.1.0' BondLength=\"48\" ChainAngle=\"90\"><page id='5' DrawingSpace='Poster' Width='612.00' Height='792.000'><fragment id='7'><n id='9' NodeType='Element' Element='6' Charge='0' Radical='None'  p=\"72 96\" Isotope=\"13\" NumHydrogens=\"4\" ShowTerminalCarbonLabels=\"no\" ShowNonTerminalCarbonLabels=\"yes\"/></fragment></page></CDXML>");
    oracle(
        &output,
        r#"[["CDXML",{"CreationProgram":"Bondline 0.1.0","BondLength":"48","ChainAngle":"90"}],["page",{"id":"5","DrawingSpace":"Poster","Width":"612.00","Height":"792.000"}],["fragment",{"id":"7"}],["n",{"id":"9","NodeType":"Element","Element":"6","Charge":"0","Radical":"None","p":"72 96","Isotope":"13","NumHydrogens":"4","ShowTerminalCarbonLabels":"no","ShowNonTerminalCarbonLabels":"yes"}]]"#,
    )?;
    *stored.model_mut() = baseline.as_ref().clone();
    assert!(!stored.source_info().ok_or("source info")?.modified);
    assert!(matches!(stored.to_cdxml()?, Cow::Borrowed(_)));
    assert_eq!(stored.to_cdxml()?.as_ref(), source);
    Ok(())
}
