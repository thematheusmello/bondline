use super::*;

#[test]
fn current_capture_preserves_the_ordinary_mol_frame_at_one_and_two_x() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    for molecule in [Molecule::Ethanol, Molecule::Methylamine] {
        let input = directory.path(&format!("current-{}.mol", molecule.name()));
        let output = directory.path(&format!("retained-{}.mol", molecule.name()));
        fs::write(&input, molecule.raw()?)?;
        let mut engine = Engine::default();
        engine.dispatch("document.open_mol", json!({"path":input}))?;
        let before = serde_json::to_value(engine.state())?;
        let content = engine.content().clone();
        let stamp = engine.capture_stamp();
        engine.dispatch("ui.view", json!({"action":"actual"}))?;
        engine.dispatch(
            "ui.view",
            json!({"action":"pan","center_x_pt":100,"center_y_pt":144}),
        )?;
        for density in [1, 2] {
            let reply = engine.dispatch("render.capture", json!({
                "page_id":1,"width_px":848*density,"height_px":651*density,"backend":"cpu",
                "view":{"logical_width":848,"logical_height":651,"expected_view_epoch":engine.view().epoch}
            }))?;
            assert!(!reply.changed);
            assert!(
                !reply.view_target_prepared,
                "a Drawing does not prepare SDF bounds"
            );
            let capture = reply.capture.ok_or("current Drawing image")?;
            assert_eq!(capture.label_em_px, Some(12.0 * density as f32));
            let mode = match molecule {
                Molecule::Ethanol => "oxygen",
                Molecule::Methylamine => "nitrogen",
            };
            let mut child = Process::new(python()).args(["-B", "-c", r#"
import io,sys,PIL
from PIL import Image
assert PIL.__version__=='12.3.0'
d=int(sys.argv[1]); mode=sys.argv[2]
with Image.open(io.BytesIO(sys.stdin.buffer.read())) as im:
 assert im.mode=='RGBA' and im.size==(848*d,651*d)
 black=lambda b:sum(p==(0,0,0,255) for p in im.crop(tuple(v*d for v in b)).get_flattened_data())
 # Ordinary source frame is not SDF row packing. At100% its translation is(324,182).
 # Ethanol anchors=(366,242),(402,242),(402,206); methylamine=(432,290),(396,254).
 chords=[(384,242),(402,224)] if mode=='oxygen' else [(414,272)]
 for x,y in chords: assert black((x-1,y-1,x+2,y+2))>0,('ordinary chord',x,y,d)
 boxes=[(398,201,407,212),(390,201,398,212)] if mode=='oxygen' else [(392,249,401,260),(379,249,387,260),(387,255,392,263)]
 for box in boxes: assert black(box)>0,('source-aware O/H or N/H/subscript',box,d)
 if mode=='oxygen': assert black((398,189,407,199))==0,'horizontal-first H is not stacked above'
 else: assert black((411,233,418,240))==0,'source y direction is not reversed'
"#, &density.to_string(), mode])
                .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
            child
                .stdin
                .take()
                .ok_or("PNG consumer stdin")?
                .write_all(&capture.png_bytes()?)?;
            let consumed = child.wait_with_output()?;
            assert!(
                consumed.status.success(),
                "{}",
                String::from_utf8_lossy(&consumed.stderr)
            );
            assert_eq!(serde_json::to_value(engine.state())?, before);
            assert_eq!(engine.content(), &content);
            assert_eq!(engine.capture_stamp(), stamp);
        }
        engine.dispatch("document.save_mol", json!({"path":output}))?;
        assert_eq!(fs::read(&output)?, molecule.raw()?);
        assert_eq!(fs::read(&input)?, molecule.raw()?);
        independent_molecule(molecule, &fs::read(output)?)?;
    }
    Ok(())
}
