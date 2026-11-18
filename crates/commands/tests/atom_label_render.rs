//! Independent command/render composition control; not a declared new renderer RED.

use bondline_commands::Engine;
use serde_json::json;
use std::{
    fs,
    process::{Command as Process, Stdio},
    sync::Arc,
};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse only the existing isolated test directory")]
mod support;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

// Exact original literal from unchanged commands/tests/element_capture.rs. Its independent
// CTfile/RDKit test qualifies VAL4/2 -> C/H3, O/H1 and the captured 24pt/unit transform.
const METHANOL: &[u8] = b"Bondline original methanol\n  Bondline          2D\nLiteral neutral-element capture\n  0  0  0  0  0  0            999 V3000\nM  V30 BEGIN CTAB\nM  V30 COUNTS 2 1 0 0 0\nM  V30 BEGIN ATOM\nM  V30 10 C 0 0 0 0 VAL=4\nM  V30 40 O 4 0 0 0 VAL=2\nM  V30 END ATOM\nM  V30 BEGIN BOND\nM  V30 7 1 10 40\nM  V30 END BOND\nM  V30 END CTAB\nM  END\n";

fn capture_pair(engine: &mut Engine) -> TestResult<Vec<String>> {
    let state = serde_json::to_value(engine.state())?;
    let content = engine.content().clone();
    let stamp = engine.capture_stamp();
    let file = engine.ui_file_context_stamp();
    let model = engine.stored_document()?.model_snapshot();
    let camera = engine.view();
    let generation = engine.session_generation();
    let source = engine
        .stored_document()?
        .export_mol()
        .ok()
        .map(|bytes| (bytes.as_ptr(), bytes.len()));
    let mut pngs = Vec::new();
    for backend in ["cpu", "gpu"] {
        let result = engine.dispatch(
            "render.capture",
            json!({
                "page_id":1,"width_px":1024,"height_px":1024,"backend":backend
            }),
        )?;
        assert!(!result.changed);
        let capture = result.capture.ok_or("registered PNG capture")?;
        assert_eq!((capture.width_px, capture.height_px), (1024, 1024));
        assert_eq!(capture.mime_type, "image/png");
        if backend == "gpu" {
            let adapter = capture.adapter.as_ref().ok_or("actual GPU adapter")?;
            assert!(!adapter.name.is_empty());
            assert!(!adapter.backend.is_empty());
            assert!(!adapter.device_type.is_empty());
        } else {
            assert!(capture.adapter.is_none());
        }
        pngs.push(capture.png_base64);
        assert_eq!(serde_json::to_value(engine.state())?, state);
        assert_eq!(engine.content(), &content);
        assert_eq!(engine.capture_stamp(), stamp);
        assert!(engine.ui_file_context_stamp_matches(&file));
        assert!(Arc::ptr_eq(
            &model,
            &engine.stored_document()?.model_snapshot()
        ));
        assert_eq!(engine.view(), camera);
        assert_eq!(engine.session_generation(), generation);
        assert_eq!(
            engine
                .stored_document()?
                .export_mol()
                .ok()
                .map(|bytes| (bytes.as_ptr(), bytes.len())),
            source
        );
    }
    assert_eq!(pngs[0], pngs[1], "CPU/GPU PNG bytes differ");
    Ok(pngs)
}

#[test]
fn native_oh_label_capture_equals_the_independent_original_methanol_mol_capture() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("original-methanol-λ.mol");
    fs::write(&input, METHANOL)?;

    let mut reference = Engine::default();
    reference.dispatch("document.open_mol", json!({"path":input}))?;
    assert_eq!(reference.stored_document()?.export_mol()?, METHANOL);
    assert!(!reference.state().file.dirty);
    assert_eq!(
        (reference.state().undo_depth, reference.state().redo_depth),
        (0, 0)
    );

    let mut native = Engine::default();
    native.dispatch(
        "bond.create",
        json!({"page_id":1,
        "begin_x_pt":72,"begin_y_pt":72,"end_x_pt":168,"end_y_pt":72}),
    )?;
    native.dispatch(
        "atom.set_label",
        json!({"expected_revision":1,
        "page_id":1,"fragment_id":1,"atom_id":2,"element":8,"explicit_hydrogens":1}),
    )?;
    assert!(native.state().source.is_none());
    assert!(native.state().file.dirty);
    assert_eq!(
        (
            native.state().revision,
            native.state().undo_depth,
            native.state().redo_depth
        ),
        (2, 2, 0)
    );
    assert_eq!(native.document()?, reference.document()?);
    assert_eq!(
        serde_json::to_value(native.document()?)?["pages"][0]["fragments"],
        json!([
        {"id":1,"atoms":[
            {"id":1,"element":6,"position":{"x_pt":72.0,"y_pt":72.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":3},
            {"id":2,"element":8,"position":{"x_pt":168.0,"y_pt":72.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":1}],
         "bonds":[{"id":1,"begin":1,"end":2,"order":"single"}]}])
    );

    let native_pngs = capture_pair(&mut native)?;
    let reference_pngs = capture_pair(&mut reference)?;
    assert_eq!(
        native_pngs, reference_pngs,
        "native literal OH must have the same PNG as the independently imported original methanol"
    );
    assert_eq!(fs::read(&input)?, METHANOL);
    assert_eq!(reference.stored_document()?.export_mol()?, METHANOL);

    let mut pillow =
        Process::new(std::env::var_os("BONDLINE_PYTHON").ok_or("pinned external Python")?)
            .args([
                "-B",
                "-c",
                r#"
import base64,io,json,sys,PIL
from PIL import Image
assert PIL.__version__=='12.3.0'
p=json.load(sys.stdin)
assert set(p)=={'native','reference'} and len(p['native'])==len(p['reference'])==2
pixels=[]
pngs=[]
for name in ('native','reference'):
 for encoded in p[name]:
  raw=base64.b64decode(encoded,validate=True)
  with Image.open(io.BytesIO(raw)) as im: im.verify()
  with Image.open(io.BytesIO(raw)) as im:
   assert im.format=='PNG' and im.mode=='RGBA' and im.size==(1024,1024)
   rgba=im.tobytes()
   assert any(rgba[i:i+4]==bytes((0,0,0,255)) for i in range(0,len(rgba),4)), 'no molecular ink'
   pixels.append(rgba)
   pngs.append(raw)
assert all(raw==pngs[0] for raw in pngs[1:]), 'actual PNG bytes differ'
assert all(rgba==pixels[0] for rgba in pixels[1:]), 'actual CPU/GPU/native/reference RGBA differs'
"#,
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
    serde_json::to_writer(
        pillow.stdin.take().ok_or("Pillow stdin")?,
        &json!({"native":native_pngs,"reference":reference_pngs}),
    )?;
    let output = pillow.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}
