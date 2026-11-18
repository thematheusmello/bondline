//! Composition through existing registry commands; no same-target label policy is assumed.
use bondline_commands::Engine;
use bondline_io::{StoredDocument, cdxml::write_document};
use bondline_model::{Atom, AtomId, Bond, Element, Fragment, FragmentId, PageId, Point2};
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
            "bondline-carbon-flag-registry-{}-{}",
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
fn target(engine: &Engine) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(serde_json::to_value(
        &engine.document()?.pages()[0].fragments()[0].atoms()[0],
    )?)
}
fn consume(bytes: &[u8], stage: &str) -> TestResult {
    let program = r#"
import decimal,json,subprocess,sys,xml.parsers.expat as expat
from rdkit import Chem,rdBase
from rdkit.Chem import rdMolDescriptors
assert rdBase.rdkitVersion=='2026.09.2'
stage=sys.argv[2]
assert stage in ('propane','propene','vinyl_alcohol')
source=sys.stdin.buffer.read()
records=[]
p=expat.ParserCreate(namespace_separator='|')
p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
p.StartElementHandler=lambda name,attrs:records.append((name,attrs))
p.Parse(source,True)
assert [name for name,_ in records]==['CDXML','page','fragment','n','n','n','b','b']
nodes=[attrs for name,attrs in records if name=='n']
bonds=[attrs for name,attrs in records if name=='b']
double=stage!='propane'
oxygen=stage=='vinyl_alcohol'
h=['1','2','1' if oxygen else '3'] if double else ['2','3','3']
assert nodes==[
 {'id':'3','NodeType':'Element','Element':'6','Charge':'0','Radical':'None','p':'100 100','NumHydrogens':h[0],'ShowNonTerminalCarbonLabels':'no'},
 {'id':'4','NodeType':'Element','Element':'6','Charge':'0','Radical':'None','p':'60 100','NumHydrogens':h[1]},
 {'id':'5','NodeType':'Element','Element':'8' if oxygen else '6','Charge':'0','Radical':'None','p':'140 100','NumHydrogens':h[2]}]
assert bonds==[{'id':'6','B':'4','E':'3','Order':'2' if double else '1'},{'id':'7','B':'3','E':'5','Order':'1'}]
for node in nodes:
 assert all(decimal.Decimal(value).is_finite() for value in node['p'].split())
actual=subprocess.run([sys.executable,'-B',sys.argv[1]],input=source,stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=True)
graph=json.loads(actual.stdout)
assert graph==[{'id':1,'atoms':[
 {'id':1,'element':6,'position':{'x_pt':100.0,'y_pt':100.0},'isotope':None,'charge':0,'radical':'none','explicit_hydrogens':int(h[0])},
 {'id':2,'element':6,'position':{'x_pt':60.0,'y_pt':100.0},'isotope':None,'charge':0,'radical':'none','explicit_hydrogens':int(h[1])},
 {'id':3,'element':8 if oxygen else 6,'position':{'x_pt':140.0,'y_pt':100.0},'isotope':None,'charge':0,'radical':'none','explicit_hydrogens':int(h[2])}],
 'bonds':[{'id':1,'begin':2,'end':1,'order':'double' if double else 'single'},{'id':2,'begin':1,'end':3,'order':'single'}]}]
mols=Chem.MolsFromCDXML(source.decode('utf-8'),sanitize=True,removeHs=False)
assert len(mols)==1 and mols[0].GetNumAtoms()==3 and mols[0].GetNumBonds()==2
assert rdMolDescriptors.CalcMolFormula(mols[0])=={'propane':'C3H8','propene':'C3H6','vinyl_alcohol':'C2H4O'}[stage]
assert Chem.MolToSmiles(mols[0])=={'propane':'CCC','propene':'C=CC','vinyl_alcohol':'C=CO'}[stage]
"#;
    let mut child =
        Command::new(std::env::var_os("BONDLINE_PYTHON").ok_or("pinned BONDLINE_PYTHON")?)
            .args([
                "-B",
                "-c",
                program,
                concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../xtask/oracles/cdxml_graph.py"
                ),
                stage,
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
    child
        .stdin
        .take()
        .ok_or("consumer stdin")?
        .write_all(bytes)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "actual-file flag/graph/chemistry consumer: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}
#[test]
fn unicode_registry_save_open_order_noop_unrelated_label_history_and_mol_loss_guards_preserve_flag()
-> TestResult {
    let directory = Directory::new()?;
    let input = directory.0.join("original-炭-λ.cdxml");
    let output = directory.0.join("saved-炭-λ.cdxml");
    let mol = directory.0.join("must-stay-exact-炭.mol");
    const SENTINEL: &[u8] = b"preserve destination bytes\0";
    let mut atoms = Vec::new();
    for (id, x, h) in [(1, 100.0, 2), (2, 60.0, 3), (3, 140.0, 3)] {
        let mut atom = Atom::new(
            AtomId::new(id).ok_or("atom")?,
            Element::try_from(6)?,
            Some(Point2::new(x, 100.0)?),
        );
        atom.explicit_hydrogens = Some(h);
        atoms.push(atom);
    }
    let mut value = serde_json::to_value(&atoms[0])?;
    value["show_non_terminal_carbon_labels"] = json!(false);
    atoms[0] = serde_json::from_value(value)?;
    let bonds = [[2, 1], [1, 3]]
        .into_iter()
        .enumerate()
        .map(|(index, [begin, end])| {
            serde_json::from_value::<Bond>(
                json!({"id":index+1,"begin":begin,"end":end,"order":"single"}),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut native = StoredDocument::default();
    native.model_mut().append_fragments(
        PageId::new(1).ok_or("page")?,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            atoms,
            bonds,
        )?],
    )?;
    let original = write_document(native.model())?;
    consume(&original, "propane")?;
    fs::write(&input, &original)?;
    fs::write(&mol, SENTINEL)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_cdxml", json!({"path":input}))?;
    assert!(engine.stored_document()?.is_fully_mapped());
    assert!(!engine.state().file.dirty);
    assert_eq!(target(&engine)?["show_non_terminal_carbon_labels"], false);
    let before = serde_json::to_vec(&engine.state())?;
    let owner = engine.stored_document()?.model_snapshot();
    let no_op=engine.dispatch("bond.set_order",json!({"expected_revision":engine.state().revision,"page_id":1,"fragment_id":1,"bond_id":1,"order":"single","hydrogen_policy":"rectify_neutral_carbon"}))?;
    assert!(!no_op.changed);
    assert_eq!(serde_json::to_vec(&engine.state())?, before);
    assert!(Arc::ptr_eq(
        &owner,
        &engine.stored_document()?.model_snapshot()
    ));
    engine.dispatch("bond.set_order",json!({"expected_revision":engine.state().revision,"page_id":1,"fragment_id":1,"bond_id":1,"order":"double","hydrogen_policy":"rectify_neutral_carbon"}))?;
    assert_eq!(engine.state().undo_depth, 1);
    assert_eq!(target(&engine)?["explicit_hydrogens"], 1);
    assert_eq!(target(&engine)?["show_non_terminal_carbon_labels"], false);
    consume(engine.stored_document()?.to_cdxml()?.as_ref(), "propene")?;
    // Only the unrelated terminal atom is labeled. Same-target display/reset policy is deferred.
    engine.dispatch("atom.set_label",json!({"expected_revision":engine.state().revision,"page_id":1,"fragment_id":1,"atom_id":3,"element":8,"explicit_hydrogens":1}))?;
    assert_eq!(target(&engine)?["show_non_terminal_carbon_labels"], false);
    assert_eq!(target(&engine)?["explicit_hydrogens"], 1);
    assert_eq!(engine.state().undo_depth, 2);
    assert!(engine.state().file.dirty);
    let changed = serde_json::to_value(engine.document()?)?;
    let before = serde_json::to_vec(&engine.state())?;
    let owner = engine.stored_document()?.model_snapshot();
    let stamp = engine.capture_stamp();
    for version in ["v2000", "v3000"] {
        let error = engine
            .dispatch(
                "document.save_mol",
                json!({"path":mol,"overwrite":true,"version":version}),
            )
            .err()
            .ok_or("registry dropped flag")?;
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
    consume(&bytes, "vinyl_alcohol")?;
    assert_eq!(fs::read(&input)?, original);
    assert_eq!(fs::read(&mol)?, SENTINEL);
    assert!(!engine.state().file.dirty);
    let canonical = fs::canonicalize(&output)?;
    assert_eq!(engine.state().file.path.as_deref(), canonical.to_str());
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(target(&engine)?["show_non_terminal_carbon_labels"], false);
    consume(engine.stored_document()?.to_cdxml()?.as_ref(), "propene")?;
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(target(&engine)?["show_non_terminal_carbon_labels"], false);
    assert_eq!(engine.stored_document()?.to_cdxml()?.as_ref(), original);
    assert!(engine.state().file.dirty);
    engine.dispatch("history.redo", json!({}))?;
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(serde_json::to_value(engine.document()?)?, changed);
    assert!(!engine.state().file.dirty);
    let mut reopened = Engine::default();
    reopened.dispatch("document.open_cdxml", json!({"path":output}))?;
    assert_eq!(serde_json::to_value(reopened.document()?)?, changed);
    assert_eq!(target(&reopened)?["show_non_terminal_carbon_labels"], false);
    assert!(reopened.stored_document()?.is_fully_mapped());
    assert_eq!(
        (reopened.state().undo_depth, reopened.state().redo_depth),
        (0, 0)
    );
    assert!(!reopened.state().file.dirty);
    assert_eq!(reopened.stored_document()?.to_cdxml()?.as_ref(), bytes);
    Ok(())
}
