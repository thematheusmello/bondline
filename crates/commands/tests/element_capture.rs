use bondline_commands::Engine;
use serde_json::json;
use std::{
    fs,
    io::Write as _,
    process::{Command, Stdio},
};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse only the existing isolated test directory")]
mod support;

// Original CTfile vector: explicit published VAL arithmetic gives C-H3 and O-H1.
const METHANOL: &[u8] = b"Bondline original methanol\n  Bondline          2D\nLiteral neutral-element capture\n  0  0  0  0  0  0            999 V3000\nM  V30 BEGIN CTAB\nM  V30 COUNTS 2 1 0 0 0\nM  V30 BEGIN ATOM\nM  V30 10 C 0 0 0 0 VAL=4\nM  V30 40 O 4 0 0 0 VAL=2\nM  V30 END ATOM\nM  V30 BEGIN BOND\nM  V30 7 1 10 40\nM  V30 END BOND\nM  V30 END CTAB\nM  END\n";

fn independent(source: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(python)
        .args([
            "-B",
            "-c",
            r#"
import sys
from rdkit import Chem, rdBase
from rdkit.Chem import rdMolDescriptors
assert rdBase.rdkitVersion == '2026.09.2'
m = Chem.MolFromMolBlock(sys.stdin.buffer.read().decode('ascii'), removeHs=False, strictParsing=True)
assert m is not None and Chem.MolToSmiles(m) == 'CO'
assert rdMolDescriptors.CalcMolFormula(m) == 'CH4O'
assert [a.GetAtomicNum() for a in m.GetAtoms()] == [6, 8]
assert [a.GetNumExplicitHs() for a in m.GetAtoms()] == [3, 1]
assert [(b.GetBeginAtomIdx(), b.GetEndAtomIdx(), b.GetBondTypeAsDouble()) for b in m.GetBonds()] == [(0, 1, 1.0)]
assert [tuple(m.GetConformer().GetAtomPosition(i)) for i in range(2)] == [(0.0, 0.0, 0.0), (4.0, 0.0, 0.0)]
"#,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("oracle stdin")?
        .write_all(source)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

#[test]
fn registered_neutral_element_capture_preserves_actual_mol_source_and_reopens()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("methanol-λ.mol");
    let output = directory.path("unchanged-λ.mol");
    fs::write(&input, METHANOL)?;
    independent(&fs::read(&input)?)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_mol", json!({"path":input}))?;
    assert!(engine.stored_document()?.is_fully_mapped());
    let document = engine.document()?.clone();
    let atoms = document.pages()[0].fragments()[0].atoms();
    assert_eq!(
        atoms.iter().map(|a| a.element.get()).collect::<Vec<_>>(),
        [6, 8]
    );
    assert_eq!(
        atoms
            .iter()
            .map(|a| a.explicit_hydrogens)
            .collect::<Vec<_>>(),
        [Some(3), Some(1)]
    );
    assert_eq!(
        atoms
            .iter()
            .map(|a| a.position.map(|p| [p.x_pt(), p.y_pt()]))
            .collect::<Vec<_>>(),
        [Some([72.0, 72.0]), Some([168.0, 72.0])]
    );
    let before = serde_json::to_value(engine.state())?;
    let cpu = engine
        .dispatch(
            "render.capture",
            json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":"cpu"}),
        )?
        .capture
        .ok_or("CPU capture")?;
    let gpu = engine
        .dispatch(
            "render.capture",
            json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":"gpu"}),
        )?
        .capture
        .ok_or("GPU capture")?;
    assert_eq!(cpu.png_base64, gpu.png_base64);
    assert!(gpu.adapter.is_some());
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(engine.document()?, &document);
    assert_eq!(fs::read(&input)?, METHANOL);
    engine.dispatch("document.save_mol", json!({"path":output}))?;
    assert_eq!(fs::read(&output)?, METHANOL);
    independent(&fs::read(&output)?)?;
    assert_eq!(engine.state().revision, 1);
    assert_eq!(
        (engine.state().undo_depth, engine.state().redo_depth),
        (0, 0)
    );
    assert!(!engine.state().file.dirty);
    let mut reopened = Engine::default();
    reopened.dispatch("document.open_mol", json!({"path":output}))?;
    assert_eq!(reopened.document()?, &document);
    assert_eq!(reopened.stored_document()?.export_mol()?, METHANOL);
    Ok(())
}
