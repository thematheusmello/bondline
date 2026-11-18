//! New-object IDs are independently checked, never assumed to equal application IDs.
use bondline_io::StoredDocument;
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, DoublePosition, Element, Fragment, FragmentId, PageSize,
    Point2,
};
use std::{
    io::Write,
    process::{Command, Stdio},
    sync::Arc,
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn carbon(id: u32, x: f64, h: u16) -> TestResult<Atom> {
    let mut a = Atom::new(
        AtomId::new(id).ok_or("atom")?,
        Element::new(6).ok_or("C")?,
        Some(Point2::new(x, 96.0)?),
    );
    a.explicit_hydrogens = Some(h);
    Ok(a)
}
fn single(id: u32, begin: u32, end: u32) -> TestResult<Bond> {
    Ok(Bond {
        id: BondId::new(id).ok_or("bond")?,
        begin: AtomId::new(begin).ok_or("begin")?,
        end: AtomId::new(end).ok_or("end")?,
        order: BondOrder::Single,
        double_position: DoublePosition::Unspecified,
    })
}
fn oracle(bytes: &[u8], mode: &str, source: &str, prefix: &str, suffix: &str) -> TestResult {
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
raw=sys.stdin.buffer.read(); nodes=[]
p=expat.ParserCreate(namespace_separator='|')
p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
p.StartElementHandler=lambda name,attrs:nodes.append((name,attrs))
p.Parse(raw,True)
mode,source,prefix,suffix=sys.argv[1:]
for name,a in nodes:
    if 'id' in a: assert 0<int(a['id'])<=4294967295
    if 'p' in a:
        assert len(a['p'].split())==2
        assert all(Decimal(v).is_finite() for v in a['p'].split())
def n(i,x,h,flags=False):
    return '<n id="'+i+'" NodeType="Element" Element="6" Charge="0" Radical="None" p="'+x+' 96" NumHydrogens="'+h+'"'+(' ShowTerminalCarbonLabels="no" ShowNonTerminalCarbonLabels="no"' if flags else '')+'/>'
def b(i,a,z): return '<b id="'+i+'" B="'+a+'" E="'+z+'" Order="1"/>'
if mode=='scoped':
    assert [name for name,_ in nodes]==['CDXML','page','fragment','b','n','n','n','b','fragment','n','fragment','n']
    assert nodes[1][1]['id']=='4294967295'
    new_atom,new_bond,new_fragment,methane=[nodes[i][1] for i in (6,7,10,11)]
    ids=[a['id'] for a in (new_atom,new_bond,new_fragment,methane)]
    assert len(set(ids))==4 and not set(map(int,ids))&{4294967295,7,8,101,207,900}
    assert all(i==str(int(i)) for i in ids)
    assert new_atom=={'id':ids[0],'NodeType':'Element','Element':'6','Charge':'0','Radical':'None','p':'168 96','NumHydrogens':'3','ShowTerminalCarbonLabels':'no','ShowNonTerminalCarbonLabels':'no'}
    assert new_bond=={'id':ids[1],'B':'207','E':ids[0],'Order':'1'}
    assert new_fragment=={'id':ids[2]}
    assert methane=={'id':ids[3],'NodeType':'Element','Element':'6','Charge':'0','Radical':'None','p':'240 96','NumHydrogens':'4'}
    old=source.replace("p='120 96' NumHydrogens='3'","p='120 96' NumHydrogens='2'",1)
    expected=old.replace('</fragment>',n(ids[0],'168','3',True)+b(ids[1],'207',ids[0])+'</fragment>',1)
    expected=expected.replace('</page>','<fragment id="'+ids[2]+'">'+n(ids[3],'240','4')+'</fragment></page>',1)
    assert raw==expected.encode(),(raw,expected)
else:
    names=['CDXML','page','fragment','n','n','b']
    assert [name for name,_ in nodes]==names
    fid,a,z,bond=[nodes[i][1]['id'] for i in (2,3,4,5)]
    generated=[a,z,bond]+([fid] if mode=='page' else [])
    reserved={17} | ({7} if mode=='fragment' else set())
    assert len(set(generated))==len(generated) and not set(map(int,generated))&reserved
    assert all(i==str(int(i)) for i in generated)
    assert nodes[3][1]=={'id':a,'NodeType':'Element','Element':'6','Charge':'0','Radical':'None','p':'72 96','NumHydrogens':'3'}
    assert nodes[4][1]=={'id':z,'NodeType':'Element','Element':'6','Charge':'0','Radical':'None','p':'120 96','NumHydrogens':'3'}
    assert nodes[5][1]=={'id':bond,'B':a,'E':z,'Order':'1'}
    children=n(a,'72','3')+n(z,'120','3')+b(bond,a,z)
    if mode=='fragment': assert fid=='7'
    else: children='<fragment id="'+fid+'">'+children+'</fragment>'
    assert raw==(prefix+children+suffix).encode(),raw
"#,mode,source,prefix,suffix]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    child.stdin.take().ok_or("stdin")?.write_all(bytes)?;
    let out = child.wait_with_output()?;
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    Ok(())
}

#[test]
fn scoped_repeated_ids_and_u32_max_do_not_block_sparse_model_appends_or_retarget_old_bonds()
-> TestResult {
    let source = "<CDXML CreationProgram='Bondline 0.1.0'><page id='4294967295' Width='612.00' DrawingSpace='Poster' Height='792.000'><fragment id='7'><!--a--><b id='900' E='207' B='101' Order='1'/><n id='101' NodeType='Element' Element='6' Charge='0' Radical='None' p='72.00 96.000' NumHydrogens='3'/><n id='207' NodeType='Element' Element='6' Charge='0' Radical='None' p='120 96' NumHydrogens='3'/></fragment><fragment id='8'><n id='101' NodeType='Element' Element='8' Charge='0' Radical='None' p='192 192' NumHydrogens='2'/></fragment></page></CDXML>";
    let mut stored = StoredDocument::from_cdxml(source.as_bytes().into(), PageSize::A4)?;
    assert!(stored.is_fully_mapped());
    let original = stored.clone();
    let page = stored.model().pages()[0].id();
    let untouched = Arc::clone(&stored.model().pages()[0].fragments()[1]);
    let f = stored.model().pages()[0].fragments()[0].as_ref();
    let mut atoms = f.atoms().to_vec();
    let mut bonds = f.bonds().to_vec();
    assert_eq!(atoms.iter().map(|a| a.id.get()).collect::<Vec<_>>(), [1, 2]);
    atoms[1].explicit_hydrogens = Some(2);
    let mut added = carbon(900_001, 168.0, 3)?;
    added.show_terminal_carbon_labels = Some(false);
    added.show_non_terminal_carbon_labels = Some(false);
    atoms.push(added);
    bonds.push(single(700_000, 2, 900_001)?);
    let replacement = Fragment::new(f.id(), atoms, bonds)?;
    stored.model_mut().replace_fragment(page, replacement)?;
    stored.model_mut().append_fragments(
        page,
        vec![Fragment::new(
            FragmentId::new(600_000).ok_or("fragment")?,
            vec![carbon(900_002, 240.0, 4)?],
            vec![],
        )?],
    )?;
    assert!(stored.is_fully_mapped());
    assert!(Arc::ptr_eq(
        &untouched,
        &stored.model().pages()[0].fragments()[1]
    ));
    let output = stored.to_cdxml()?;
    assert_eq!(output, stored.to_cdxml()?);
    oracle(&output, "scoped", source, "", "")?;
    assert_eq!(original.to_cdxml()?.as_ref(), source.as_bytes());
    assert_eq!(
        stored.model().pages()[0].fragments()[0].atoms()[2].id.get(),
        900_001
    );
    Ok(())
}

#[test]
fn existing_empty_fragment_both_forms_accept_new_atoms_and_bonds_without_rewriting_trivia()
-> TestResult {
    for (source, prefix, suffix) in [
        (
            "<CDXML CreationProgram='Bondline 0.1.0'><page id='17' DrawingSpace='Poster' Width='612.00' Height='792.000'><fragment id='7' /></page></CDXML>",
            "<CDXML CreationProgram='Bondline 0.1.0'><page id='17' DrawingSpace='Poster' Width='612.00' Height='792.000'><fragment id='7' >",
            "</fragment></page></CDXML>",
        ),
        (
            "<CDXML CreationProgram='Bondline 0.1.0'><page id='17' DrawingSpace='Poster' Width='612.00' Height='792.000'><fragment id='7'><!--kept--></fragment></page></CDXML>",
            "<CDXML CreationProgram='Bondline 0.1.0'><page id='17' DrawingSpace='Poster' Width='612.00' Height='792.000'><fragment id='7'><!--kept-->",
            "</fragment></page></CDXML>",
        ),
    ] {
        let mut stored = StoredDocument::from_cdxml(source.as_bytes().into(), PageSize::A4)?;
        assert!(stored.is_fully_mapped());
        let original = stored.clone();
        let page = stored.model().pages()[0].id();
        let fid = stored.model().pages()[0].fragments()[0].id();
        stored.model_mut().replace_fragment(
            page,
            Fragment::new(
                fid,
                vec![carbon(100_000, 72.0, 3)?, carbon(200_000, 120.0, 3)?],
                vec![single(300_000, 100_000, 200_000)?],
            )?,
        )?;
        oracle(&stored.to_cdxml()?, "fragment", source, prefix, suffix)?;
        assert_eq!(original.to_cdxml()?.as_ref(), source.as_bytes());
    }
    Ok(())
}

#[test]
fn existing_empty_page_both_forms_accept_a_new_fragment_without_canonicalizing_the_page()
-> TestResult {
    for (source, prefix, suffix) in [
        (
            "<CDXML CreationProgram='Bondline 0.1.0'><page id='17' DrawingSpace='Poster' Width='612.00' Height='792.000' /></CDXML>",
            "<CDXML CreationProgram='Bondline 0.1.0'><page id='17' DrawingSpace='Poster' Width='612.00' Height='792.000' >",
            "</page></CDXML>",
        ),
        (
            "<CDXML CreationProgram='Bondline 0.1.0'><page id='17' DrawingSpace='Poster' Width='612.00' Height='792.000'><!--kept--></page></CDXML>",
            "<CDXML CreationProgram='Bondline 0.1.0'><page id='17' DrawingSpace='Poster' Width='612.00' Height='792.000'><!--kept-->",
            "</page></CDXML>",
        ),
    ] {
        let mut stored = StoredDocument::from_cdxml(source.as_bytes().into(), PageSize::A4)?;
        assert!(stored.is_fully_mapped());
        let original = stored.clone();
        let page = stored.model().pages()[0].id();
        stored.model_mut().append_fragments(
            page,
            vec![Fragment::new(
                FragmentId::new(900_000).ok_or("fragment")?,
                vec![carbon(100_000, 72.0, 3)?, carbon(200_000, 120.0, 3)?],
                vec![single(300_000, 100_000, 200_000)?],
            )?],
        )?;
        oracle(&stored.to_cdxml()?, "page", source, prefix, suffix)?;
        assert_eq!(original.to_cdxml()?.as_ref(), source.as_bytes());
    }
    Ok(())
}
