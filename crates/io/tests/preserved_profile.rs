//! Original exact-byte counterexample declared before profile implementation.
use bondline_io::{DocumentError, StoredDocument};
use bondline_model::{BondOrder, Fragment, PageId, PageSize};
use std::{
    io::Write,
    process::{Command, Stdio},
};

const SOURCE: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<CDXML CreationProgram="Bondline 0.1.0">
 <page id='91' Width='612.00' Height="792.000" DrawingSpace='Poster'>
  <fragment id="400">
   <b id='900' E='207' Order='1' B='101' />
   <n Radical='None' id='101' p='100.00 200.000' NumHydrogens='3' Charge='0' Element='6' NodeType='Element'/>
   <n Element="6" NumHydrogens="3" NodeType="Element" p="140 200" id="207" Radical="None" Charge="0"></n>
  </fragment>
 </page>
</CDXML>
"#;

#[test]
fn sparse_forward_referenced_source_changes_only_order_and_hydrogen_values()
-> Result<(), Box<dyn std::error::Error>> {
    let mut stored = StoredDocument::from_cdxml(SOURCE.into(), PageSize::A4)?;
    let before = stored.clone();
    assert_eq!(stored.to_cdxml()?.as_ref(), SOURCE);
    assert!(!stored.source_info().ok_or("source info")?.modified);
    let original = stored
        .model()
        .pages()
        .first()
        .and_then(|p| p.fragments().first())
        .ok_or("fragment")?;
    let mut atoms = original.atoms().to_vec();
    for atom in &mut atoms {
        atom.explicit_hydrogens = Some(2);
    }
    let mut bonds = original.bonds().to_vec();
    bonds.first_mut().ok_or("bond")?.order = BondOrder::Double;
    let replacement = Fragment::new(original.id(), atoms, bonds)?;
    stored
        .model_mut()
        .replace_fragment(PageId::new(1).ok_or("page")?, replacement)?;
    let written = stored.to_cdxml()?;
    let expected = std::str::from_utf8(SOURCE)?
        .replace("Order='1'", "Order='2'")
        .replace("NumHydrogens='3'", "NumHydrogens='2'")
        .replace("NumHydrogens=\"3\"", "NumHydrogens=\"2\"");
    assert_eq!(written.as_ref(), expected.as_bytes());
    assert_eq!(before.to_cdxml()?.as_ref(), SOURCE);
    assert!(stored.is_fully_mapped());
    assert!(stored.source_info().ok_or("source info")?.modified);
    assert!(matches!(
        stored.export_cdxml(),
        Err(DocumentError::UnsupportedEdit)
    ));
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
nodes=[]
p=expat.ParserCreate(namespace_separator='|')
p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
p.StartElementHandler=lambda name,attrs:nodes.append((name,attrs))
p.Parse(sys.stdin.buffer.read(),True)
assert [n for n,_ in nodes]==['CDXML','page','fragment','b','n','n']
assert nodes[0][1]=={'CreationProgram':'Bondline 0.1.0'}
assert nodes[1][1]=={'id':'91','Width':'612.00','Height':'792.000','DrawingSpace':'Poster'}
assert nodes[2][1]=={'id':'400'}
assert nodes[3][1]=={'id':'900','B':'101','E':'207','Order':'2'}
for (_,a),identity,pos in zip(nodes[4:],('101','207'),(('100.00','200.000'),('140','200'))):
    assert a==dict(id=identity,Element='6',NodeType='Element',Radical='None',Charge='0',NumHydrogens='2',p=' '.join(pos))
    assert tuple(map(Decimal,a['p'].split()))==tuple(map(Decimal,pos))
"#]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    child
        .stdin
        .take()
        .ok_or("Expat stdin")?
        .write_all(&written)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    *stored.model_mut() = before.model().clone();
    assert_eq!(stored.to_cdxml()?.as_ref(), SOURCE);
    Ok(())
}
