//! Independent registry composition: persistent depiction is not a new command or chemical inference.
use bondline_commands::Engine;
use bondline_io::{StoredDocument, cdxml::write_document};
use bondline_model::{Atom, AtomId, Bond, Element, Fragment, FragmentId, PageId};
use serde_json::{Value, json};
use std::{
    fs,
    io::Write as _,
    path::PathBuf,
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Directory(PathBuf);
impl Directory {
    fn new() -> Result<Self, std::io::Error> {
        let path = std::env::temp_dir().join(format!(
            "bondline-sided-registry-{}-{}",
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

fn current_bond(engine: &Engine) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(serde_json::to_value(
        engine.document()?.pages()[0].fragments()[0].bonds()[0],
    )?)
}
fn consume(bytes: &[u8], single: bool) -> TestResult {
    let code = r#"
import decimal,io,json,subprocess,sys,xml.parsers.expat as expat
from rdkit import Chem,rdBase
from rdkit.Chem import rdMolDescriptors
assert rdBase.rdkitVersion=='2026.09.2'
single=sys.argv[2]=='single'
source=sys.stdin.buffer.read()
attrs=[]
p=expat.ParserCreate(namespace_separator='|')
p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
p.StartElementHandler=lambda name,value:attrs.append((name,value))
p.Parse(source,True)
nodes=[value for name,value in attrs if name=='n']
bonds=[value for name,value in attrs if name=='b']
assert len(nodes)==2 and len(bonds)==1
h='3' if single else '2'
assert nodes==[
 {'id':'3','NodeType':'Element','Element':'6','Charge':'0','Radical':'None','p':'72.125 96.25','NumHydrogens':h},
 {'id':'4','NodeType':'Element','Element':'6','Charge':'0','Radical':'None','p':'168.5 96.25','NumHydrogens':h}]
assert bonds==[{'id':'5','B':'3','E':'4','Order':'1' if single else '2','DoublePosition':'Right'}]
for value in nodes:
 assert all(decimal.Decimal(token).is_finite() for token in value['p'].split())
raw=subprocess.run([sys.executable,'-B',sys.argv[1]],input=source,stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=True)
graph=json.loads(raw.stdout)
assert graph==[{'id':1,'atoms':[
 {'id':1,'element':6,'position':{'x_pt':72.125,'y_pt':96.25},'isotope':None,'charge':0,'radical':'none','explicit_hydrogens':int(h)},
 {'id':2,'element':6,'position':{'x_pt':168.5,'y_pt':96.25},'isotope':None,'charge':0,'radical':'none','explicit_hydrogens':int(h)}],
 'bonds':[{'id':1,'begin':1,'end':2,'order':'single' if single else 'double'}]}]
mols=Chem.MolsFromCDXML(source.decode('utf-8'),sanitize=True,removeHs=False)
assert len(mols)==1 and mols[0].GetNumAtoms()==2 and mols[0].GetNumBonds()==1
assert rdMolDescriptors.CalcMolFormula(mols[0])==('C2H6' if single else 'C2H4')
assert Chem.MolToSmiles(mols[0])==('CC' if single else 'C=C')
"#;
    let mut child =
        Command::new(std::env::var_os("BONDLINE_PYTHON").ok_or("pinned BONDLINE_PYTHON")?)
            .args([
                "-B",
                "-c",
                code,
                concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../xtask/oracles/cdxml_graph.py"
                ),
                if single { "single" } else { "double" },
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
    child.stdin.take().ok_or("oracle stdin")?.write_all(bytes)?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "actual-file Expat/Decimal plus unchanged graph oracle/RDKit: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}

#[test]
fn registry_unicode_save_open_order_history_and_failed_mol_publication_keep_persistent_direction_metadata()
-> TestResult {
    let directory = Directory::new()?;
    let input = directory.0.join("original-右-λ.cdxml");
    let output = directory.0.join("saved-右-λ.cdxml");
    let mol = directory.0.join("must-preserve-λ.mol");
    const SENTINEL: &[u8] = b"existing MOL destination remains exact\0";
    let mut atoms = Vec::new();
    for (id, x) in [(1, 72.125), (2, 168.5)] {
        let mut atom = Atom::new(
            AtomId::new(id).ok_or("atom")?,
            Element::try_from(6)?,
            Some(bondline_model::Point2::new(x, 96.25)?),
        );
        atom.explicit_hydrogens = Some(2);
        atoms.push(atom);
    }
    let bond: Bond = serde_json::from_value(
        json!({"id":1,"begin":1,"end":2,"order":"double","double_position":"right"}),
    )?;
    let mut native = StoredDocument::default();
    native.model_mut().append_fragments(
        PageId::new(1).ok_or("page")?,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            atoms,
            vec![bond],
        )?],
    )?;
    let original = write_document(native.model())?;
    consume(&original, false)?;
    fs::write(&input, &original)?;
    fs::write(&mol, SENTINEL)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_cdxml", json!({"path":input}))?;
    assert!(engine.stored_document()?.is_fully_mapped());
    assert_eq!(engine.state().undo_depth, 0);
    assert!(!engine.state().file.dirty);
    assert_eq!(
        current_bond(&engine)?,
        json!({"id":1,"begin":1,"end":2,"order":"double","double_position":"right"})
    );
    let owner = engine.stored_document()?.model_snapshot();
    let before = serde_json::to_vec(&engine.state())?;
    let no_op=engine.dispatch("bond.set_order",json!({"expected_revision":engine.state().revision,"page_id":1,"fragment_id":1,"bond_id":1,"order":"double","hydrogen_policy":"rectify_neutral_carbon"}))?;
    assert!(!no_op.changed);
    assert_eq!(serde_json::to_vec(&engine.state())?, before);
    assert!(Arc::ptr_eq(
        &owner,
        &engine.stored_document()?.model_snapshot()
    ));
    let change=engine.dispatch("bond.set_order",json!({"expected_revision":engine.state().revision,"page_id":1,"fragment_id":1,"bond_id":1,"order":"single","hydrogen_policy":"rectify_neutral_carbon"}))?;
    assert!(change.changed);
    assert_eq!(engine.state().undo_depth, 1);
    assert!(engine.state().file.dirty);
    assert_eq!(
        current_bond(&engine)?,
        json!({"id":1,"begin":1,"end":2,"order":"single","double_position":"right"})
    );
    assert!(
        engine.document()?.pages()[0].fragments()[0]
            .atoms()
            .iter()
            .all(|atom| atom.explicit_hydrogens == Some(3))
    );
    let changed = serde_json::to_value(engine.document()?)?;
    let before = serde_json::to_vec(&engine.state())?;
    let stamp = engine.capture_stamp();
    let owner = engine.stored_document()?.model_snapshot();
    for version in ["v2000", "v3000"] {
        let error = engine
            .dispatch(
                "document.save_mol",
                json!({"path":mol,"overwrite":true,"version":version}),
            )
            .err()
            .ok_or("registry silently dropped dormant placement")?;
        assert_eq!(error.info().code, "unsupported_content");
        assert_eq!(fs::read(&mol)?, SENTINEL);
        assert_eq!(fs::read_dir(&directory.0)?.count(), 2);
        assert_eq!(serde_json::to_vec(&engine.state())?, before);
        assert!(engine.capture_stamp_matches(&stamp));
        assert!(Arc::ptr_eq(
            &owner,
            &engine.stored_document()?.model_snapshot()
        ));
    }
    engine.dispatch("document.save_cdxml", json!({"path":output}))?;
    let bytes = fs::read(&output)?;
    consume(&bytes, true)?;
    assert_eq!(fs::read(&input)?, original);
    assert_eq!(fs::read(&mol)?, SENTINEL);
    assert!(!engine.state().file.dirty);
    let canonical_output = fs::canonicalize(&output)?;
    assert_eq!(
        engine.state().file.path.as_deref(),
        canonical_output.to_str()
    );
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(
        current_bond(&engine)?,
        json!({"id":1,"begin":1,"end":2,"order":"double","double_position":"right"})
    );
    assert!(
        engine.document()?.pages()[0].fragments()[0]
            .atoms()
            .iter()
            .all(|atom| atom.explicit_hydrogens == Some(2))
    );
    assert!(engine.state().file.dirty);
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(serde_json::to_value(engine.document()?)?, changed);
    assert!(!engine.state().file.dirty);
    let mut reopened = Engine::default();
    reopened.dispatch("document.open_cdxml", json!({"path":output}))?;
    assert_eq!(serde_json::to_value(reopened.document()?)?, changed);
    assert_eq!(
        current_bond(&reopened)?,
        json!({"id":1,"begin":1,"end":2,"order":"single","double_position":"right"})
    );
    assert!(reopened.stored_document()?.is_fully_mapped());
    assert!(!reopened.state().file.dirty);
    assert_eq!(reopened.state().undo_depth, 0);
    assert_eq!(reopened.state().redo_depth, 0);
    assert_eq!(reopened.stored_document()?.to_cdxml()?.as_ref(), bytes);
    Ok(())
}
