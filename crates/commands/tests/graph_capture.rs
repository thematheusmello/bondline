use bondline_commands::{Cancellation, Command, Engine};
use bondline_io::StoredDocument;
use bondline_model::PageSize;
use serde_json::json;
use std::{
    fs,
    io::Write,
    process::{Command as Process, Stdio},
};
#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(
    dead_code,
    reason = "the shared temp-directory helper also contains a CDXML-only oracle"
)]
mod support;

const PROPANE:&[u8]=b"Bondline original carbon chain\n  Bondline          2D\nOriginal graph-render acceptance\n  0  0  0  0  0  0            999 V3000\nM  V30 BEGIN CTAB\nM  V30 COUNTS 3 2 0 0 0\nM  V30 BEGIN ATOM\nM  V30 10 C 0 0 0 0 VAL=4\nM  V30 40 C 2 0 0 0 VAL=4\nM  V30 90 C 4 0 0 0 VAL=4\nM  V30 END ATOM\nM  V30 BEGIN BOND\nM  V30 7 1 10 40\nM  V30 20 1 40 90\nM  V30 END BOND\nM  V30 END CTAB\nM  END\n";
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn independent_propane(bytes: &[u8]) -> TestResult {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child=Process::new(python).args(["-B","-c",r#"
import sys
from rdkit import Chem,rdBase
from rdkit.Chem import rdMolDescriptors
assert rdBase.rdkitVersion=='2026.09.2'
m=Chem.MolFromMolBlock(sys.stdin.buffer.read().decode('ascii'),removeHs=False,strictParsing=True)
assert m is not None and Chem.MolToSmiles(m)=='CCC' and rdMolDescriptors.CalcMolFormula(m)=='C3H8'
assert [a.GetAtomicNum() for a in m.GetAtoms()]==[6,6,6]
assert [a.GetNumExplicitHs() for a in m.GetAtoms()]==[3,2,3]
assert [(b.GetBeginAtomIdx(),b.GetEndAtomIdx(),b.GetBondTypeAsDouble()) for b in m.GetBonds()]==[(0,1,1.0),(1,2,1.0)]
assert [tuple(m.GetConformer().GetAtomPosition(i)) for i in range(3)]==[(0.0,0.0,0.0),(2.0,0.0,0.0),(4.0,0.0,0.0)]
"#]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    child.stdin.take().ok_or("stdin")?.write_all(bytes)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

#[test]
fn registered_capture_opens_renders_and_saves_a_literal_imported_propane_without_changing_source()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("original-chain.mol");
    let copy = directory.path("copy.mol");
    fs::write(&input, PROPANE)?;
    independent_propane(&fs::read(&input)?)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_mol", json!({"path":input}))?;
    assert!(engine.stored_document()?.is_fully_mapped());
    let page = engine.document()?.pages().first().ok_or("page")?;
    let atoms = page.fragments().first().ok_or("fragment")?.atoms();
    assert_eq!(
        atoms
            .iter()
            .map(|a| a.explicit_hydrogens)
            .collect::<Vec<_>>(),
        [Some(3), Some(2), Some(3)]
    );
    assert_eq!(
        atoms
            .iter()
            .map(|a| a.position.map(|p| [p.x_pt(), p.y_pt()]))
            .collect::<Vec<_>>(),
        [Some([72.0, 72.0]), Some([120.0, 72.0]), Some([168.0, 72.0])]
    );
    let before = serde_json::to_vec(&engine.state())?;
    let cpu = engine.dispatch(
        "render.capture",
        json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":"cpu"}),
    )?;
    assert!(!cpu.changed);
    assert_eq!(cpu.revision, 1);
    let gpu = engine.dispatch(
        "render.capture",
        json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":"gpu"}),
    )?;
    assert_eq!(
        cpu.capture.ok_or("CPU capture")?.png_base64,
        gpu.capture.ok_or("GPU capture")?.png_base64
    );
    assert_eq!(serde_json::to_vec(&engine.state())?, before);
    let cancel = Cancellation::default();
    cancel.cancel();
    let work = engine.prepare(Command::parse(
        "render.capture",
        json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":"gpu"}),
    )?)?;
    assert_eq!(
        work.complete_with_cancel(&cancel)
            .err()
            .ok_or("cancel ignored")?
            .info()
            .code,
        "cancelled"
    );
    assert_eq!(serde_json::to_vec(&engine.state())?, before);
    engine.dispatch("document.save_mol", json!({"path":copy}))?;
    assert_eq!(fs::read(&copy)?, PROPANE);
    independent_propane(&fs::read(&copy)?)?;
    assert_eq!(engine.stored_document()?.export_mol()?, PROPANE);
    assert_eq!(
        (
            engine.state().revision,
            engine.state().undo_depth,
            engine.state().redo_depth
        ),
        (1, 0, 0)
    );
    assert!(!engine.state().file.dirty);
    let mut reopened = Engine::default();
    reopened.dispatch("document.open_mol", json!({"path":copy}))?;
    assert_eq!(reopened.document()?, engine.document()?);
    assert_eq!(reopened.stored_document()?.export_mol()?, PROPANE);
    Ok(())
}

#[test]
fn a_drawable_projection_never_unlocks_unmapped_source_display_or_export() -> TestResult {
    let original =
        std::str::from_utf8(include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml"))?;
    let extended = original
        .replace("<b ", "<b Display=\"WedgeBegin\" ")
        .into_bytes();
    assert_ne!(extended, original.as_bytes());
    let stored = StoredDocument::from_cdxml(extended.clone().into(), PageSize::A4)?;
    assert!(!stored.is_fully_mapped());
    let mut engine = Engine::from_document(stored);
    let before = serde_json::to_vec(&engine.state())?;
    assert_eq!(
        engine
            .dispatch(
                "render.capture",
                json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":"cpu"})
            )
            .err()
            .ok_or("unmapped display was hidden")?
            .info()
            .code,
        "unsupported_content"
    );
    assert_eq!(serde_json::to_vec(&engine.state())?, before);
    assert_eq!(engine.stored_document()?.to_cdxml()?.as_ref(), extended);
    assert!(!engine.stored_document()?.is_fully_mapped());
    Ok(())
}
