//! Additive coverage: serialization of optional removals, not the baseline-equality fast path.
use bondline_io::StoredDocument;
use bondline_model::{BondOrder, DoublePosition, DrawingSettings, Fragment, PageSize};
use std::{
    io::Write,
    process::{Command, Stdio},
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn optional_removals_keep_adjacent_whitespace_and_unaffected_atoms_exact() -> TestResult {
    let source=b"<CDXML CreationProgram='Bondline 0.1.0' BondLength='48' ChainAngle='90'><page id='91' DrawingSpace='Poster' Width='612.00' Height='792.000'><fragment id='400'><n id='101' NodeType='Element' Element='6' Charge='0' Radical='None' p='100 200' Isotope='13' NumHydrogens='2' ShowTerminalCarbonLabels='no' ShowNonTerminalCarbonLabels='yes'/><n id='207' NodeType='Element' Element='6' Charge='0' Radical='None' p='140.00 200.000' NumHydrogens='2'/><b id='900' B='101' E='207' Order='2' DoublePosition='Right'/></fragment></page></CDXML>";
    let mut stored = StoredDocument::from_cdxml(source.as_slice().into(), PageSize::A4)?;
    assert!(stored.is_fully_mapped());
    let before = stored.clone();
    let page = stored.model().pages()[0].id();
    let f = stored.model().pages()[0].fragments()[0].as_ref();
    let mut atoms = f.atoms().to_vec();
    let mut bonds = f.bonds().to_vec();
    atoms[0].position = None;
    atoms[0].isotope = None;
    atoms[0].explicit_hydrogens = None;
    atoms[0].show_terminal_carbon_labels = None;
    atoms[0].show_non_terminal_carbon_labels = None;
    bonds[0].order = BondOrder::Triple;
    bonds[0].double_position = DoublePosition::Unspecified;
    let replacement = Fragment::new(f.id(), atoms, bonds)?;
    stored.model_mut().replace_fragment(page, replacement)?;
    stored
        .model_mut()
        .set_drawing_settings(DrawingSettings::default());
    stored
        .model_mut()
        .set_page_size(page, PageSize::new(640.0, 792.0)?)?;
    assert!(stored.is_fully_mapped());
    let output = stored.to_cdxml()?;
    assert_eq!(output.as_ref(),b"<CDXML CreationProgram='Bondline 0.1.0'  ><page id='91' DrawingSpace='Poster' Width='640' Height='792.000'><fragment id='400'><n id='101' NodeType='Element' Element='6' Charge='0' Radical='None'     /><n id='207' NodeType='Element' Element='6' Charge='0' Radical='None' p='140.00 200.000' NumHydrogens='2'/><b id='900' B='101' E='207' Order='3' /></fragment></page></CDXML>");
    assert_eq!(before.to_cdxml()?.as_ref(), source);
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child=Command::new(python).args(["-X","utf8","-B","-c",r#"
import sys,xml.parsers.expat as expat
from decimal import Decimal
nodes=[]
p=expat.ParserCreate(namespace_separator='|')
p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
p.StartElementHandler=lambda name,attrs:nodes.append((name,attrs))
p.Parse(sys.stdin.buffer.read(),True)
assert [n for n,a in nodes]==['CDXML','page','fragment','n','n','b']
assert nodes[0][1]=={'CreationProgram':'Bondline 0.1.0'}
assert nodes[3][1]=={'id':'101','NodeType':'Element','Element':'6','Charge':'0','Radical':'None'}
assert nodes[4][1]=={'id':'207','NodeType':'Element','Element':'6','Charge':'0','Radical':'None','p':'140.00 200.000','NumHydrogens':'2'}
assert nodes[5][1]=={'id':'900','B':'101','E':'207','Order':'3'}
assert Decimal(nodes[1][1]['Width'])==640 and Decimal(nodes[1][1]['Height'])==792
assert [Decimal(v) for v in nodes[4][1]['p'].split()]==[140,200]
"#]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    child.stdin.take().ok_or("stdin")?.write_all(&output)?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}
