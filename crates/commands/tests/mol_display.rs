use base64::{Engine as _, engine::general_purpose::STANDARD};
use bondline_commands::{Cancellation, Command, Engine};
use bondline_io::StoredDocument;
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Document, Element, Fragment, FragmentId, Page, PageId,
    PageSize, Point2,
};
use bondline_render::{cpu_scene, graph_page_scene};
use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    process::{Command as Process, Stdio},
};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse only the existing isolated file directory")]
mod support;

type TestResult = Result<(), Box<dyn std::error::Error>>;
const ORIGINAL: &[u8] =
    include_bytes!("../../io/tests/fixtures/sdf-preview/valence-absent-mixed.sdf");

#[derive(Clone, Copy, Debug)]
enum Molecule {
    Ethanol,
    Methylamine,
}

impl Molecule {
    fn name(self) -> &'static str {
        match self {
            Self::Ethanol => "ethanol",
            Self::Methylamine => "methylamine",
        }
    }

    fn raw(self) -> Result<&'static [u8], Box<dyn std::error::Error>> {
        assert_eq!(ORIGINAL.len(), 740);
        ORIGINAL
            .get(match self {
                Self::Ethanol => 0..371,
                Self::Methylamine => 410..713,
            })
            .ok_or_else(|| "original literal MOL range".into())
    }

    // Independent expectations in the ordinary MOL frame, never SDF-packed coordinates.
    fn atoms(self) -> &'static [(u8, u16, f64, f64)] {
        match self {
            Self::Ethanol => &[(6, 3, 42.0, 60.0), (6, 2, 78.0, 60.0), (8, 1, 78.0, 24.0)],
            Self::Methylamine => &[(6, 3, 108.0, 108.0), (7, 2, 72.0, 72.0)],
        }
    }

    fn source_positions(self) -> Value {
        match self {
            Self::Ethanol => json!([[-1.25, 0.5, 0.0], [0.25, 0.5, 0.0], [0.25, 2.0, 0.0]]),
            Self::Methylamine => json!([[1.5, -1.5, 0.0], [0.0, 0.0, 0.0]]),
        }
    }

    fn format(self) -> &'static str {
        match self {
            Self::Ethanol => "mol_v3000",
            Self::Methylamine => "mol_v2000",
        }
    }
}

fn request(backend: &str) -> Value {
    json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":backend})
}

fn python() -> std::ffi::OsString {
    std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    })
}

fn independent_molecule(molecule: Molecule, source: &[u8]) -> TestResult {
    assert_eq!(source, molecule.raw()?);
    let mut child = Process::new(python())
        .args([
            "-B",
            concat!(env!("CARGO_MANIFEST_DIR"), "/../io/tests/mol_oracle.py"),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("MOL oracle stdin")?
        .write_all(source)?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let result: Value = serde_json::from_slice(&result.stdout)?;
    let (smiles, formula) = match molecule {
        Molecule::Ethanol => ("CCO", "C2H6O"),
        Molecule::Methylamine => ("CN", "CH5N"),
    };
    assert_eq!(result["smiles"], smiles);
    assert_eq!(result["formula"], formula);
    let atoms = result["atoms"].as_array().ok_or("external atoms")?;
    assert_eq!(atoms.len(), molecule.atoms().len());
    for (actual, &(element, hydrogens, _, _)) in atoms.iter().zip(molecule.atoms()) {
        assert_eq!(actual["element"], element);
        assert_eq!(actual["hydrogens"], hydrogens);
        assert_eq!(actual["charge"], 0);
        assert_eq!(actual["isotope"], 0);
        assert_eq!(actual["radical_electrons"], 0);
    }
    assert_eq!(
        atoms
            .iter()
            .map(|atom| atom["position"].clone())
            .collect::<Vec<_>>(),
        molecule
            .source_positions()
            .as_array()
            .ok_or("source coordinates")?
            .clone()
    );
    let bonds = (0..atoms.len() - 1)
        .map(|index| json!({"begin":index,"end":index+1,"order":1.0}))
        .collect::<Vec<_>>();
    assert_eq!(result["bonds"], json!(bonds));
    Ok(())
}

#[test]
fn literal_mol_declarations_and_non_mol_drawings_keep_their_existing_policy() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let molecule = Molecule::Ethanol;
    for (valence, literal_h) in [(1, 0), (4, 3)] {
        let source = std::str::from_utf8(molecule.raw()?)?
            .replace("90 O 0.25 2 0 0", &format!("90 O 0.25 2 0 0 VAL={valence}"))
            .into_bytes();
        assert_ne!(source, molecule.raw()?);
        let input = directory.path(&format!("literal-h-{literal_h}.mol"));
        let output = directory.path(&format!("literal-copy-{literal_h}.mol"));
        fs::write(&input, &source)?;
        let mut engine = Engine::default();
        engine.dispatch("document.open_mol", json!({"path":input}))?;
        assert!(engine.stored_document()?.is_fully_mapped());
        let atoms = engine.document()?.pages()[0].fragments()[0].atoms();
        assert_eq!(atoms[0].explicit_hydrogens, None);
        assert_eq!(atoms[1].explicit_hydrogens, None);
        assert_eq!(atoms[2].explicit_hydrogens, Some(literal_h));
        let before = serde_json::to_value(engine.state())?;
        let capture = engine
            .dispatch("render.capture", request("cpu"))?
            .capture
            .ok_or("literal capture")?;
        assert_eq!(
            capture.png_base64,
            literal_png(molecule, Control::LiteralH(literal_h))?
        );
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(engine.stored_document()?.export_mol()?, source);
        engine.dispatch("document.save_mol", json!({"path":output}))?;
        assert_eq!(fs::read(&output)?, source);
    }

    // Absent CDXML H remains the separate literal compatibility policy; this feature
    // must not make every Drawing follow MOL's closed display derivation.
    for control in [Control::UnknownAuthored, Control::BareHeteroatom] {
        let page = literal_page(molecule, control)?;
        let mut source_free = StoredDocument::default();
        source_free.model_mut().append_fragments(
            PageId::new(1).ok_or("page")?,
            page.fragments()
                .iter()
                .map(|fragment| fragment.as_ref().clone())
                .collect(),
        )?;
        let cdxml = source_free.to_cdxml()?.into_owned();
        let imported = StoredDocument::from_cdxml(cdxml.clone().into(), PageSize::A4)?;
        assert!(imported.is_fully_mapped());
        for stored in [source_free, imported] {
            assert!(stored.mol_version().is_none());
            let mut engine = Engine::from_document(stored);
            let before = serde_json::to_value(engine.state())?;
            let capture = engine
                .dispatch("render.capture", request("cpu"))?
                .capture
                .ok_or("native literal capture")?;
            assert_eq!(capture.png_base64, literal_png(molecule, control)?);
            assert_eq!(serde_json::to_value(engine.state())?, before);
            assert_eq!(engine.stored_document()?.to_cdxml()?.as_ref(), cdxml);
        }
    }
    Ok(())
}

#[test]
fn deferred_mol_capture_owns_its_historical_source_and_preserves_history_on_cancel() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("old-ethanol.mol");
    let next = directory.path("new-methylamine.mol");
    fs::write(&input, Molecule::Ethanol.raw()?)?;
    fs::write(&next, Molecule::Methylamine.raw()?)?;
    independent_molecule(Molecule::Methylamine, &fs::read(&next)?)?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_mol", json!({"path":input}))?;
    let old_model = engine.document()?.clone();
    let first = engine.prepare(Command::parse("render.capture", request("cpu"))?)?;
    let second = engine.prepare(Command::parse("render.capture", request("cpu"))?)?;
    assert!(first.is_deferred() && second.is_deferred());
    engine.dispatch(
        "page.set_size",
        json!({"page_id":1,"width_pt":612,"height_pt":792}),
    )?;
    let edited = engine.content().clone();
    engine.dispatch("history.undo", json!({}))?;
    assert_eq!(engine.document()?, &old_model);
    assert_authored(&engine, Molecule::Ethanol)?;
    engine.dispatch("history.redo", json!({}))?;
    assert_eq!(engine.content(), &edited);
    let before = serde_json::to_value(engine.state())?;
    assert_eq!(before["source"]["modified"], true);
    assert_eq!(before["file"]["dirty"], true);
    let reply = engine.finish(first.complete()?)?;
    assert_eq!(reply.revision, 1);
    assert_eq!(reply.document.as_deref(), Some(&old_model));
    assert_eq!(reply.source.as_ref().ok_or("old source")?.byte_length, 371);
    assert!(!reply.source.as_ref().ok_or("old source")?.modified);
    assert!(!reply.file.dirty);
    check_pixels(
        &serde_json::to_value(reply.capture.ok_or("old capture")?)?,
        Molecule::Ethanol,
    )?;
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(engine.content(), &edited);

    fs::remove_file(&input)?;
    engine.dispatch(
        "document.open_mol",
        json!({"path":next,"discard_unsaved":true}),
    )?;
    assert_authored(&engine, Molecule::Methylamine)?;
    let revision = engine.state().revision;
    engine.dispatch(
        "document.close",
        json!({"expected_revision":revision,"action":"request"}),
    )?;
    let before = serde_json::to_value(engine.state())?;
    let owner = engine.content().clone();
    let reply = engine.finish(second.complete()?)?;
    assert_eq!(reply.revision, 1);
    assert_eq!(reply.document.as_deref(), Some(&old_model));
    assert_eq!(reply.source.as_ref().ok_or("old source")?.byte_length, 371);
    check_pixels(
        &serde_json::to_value(reply.capture.ok_or("old generation capture")?)?,
        Molecule::Ethanol,
    )?;
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(engine.content(), &owner);

    for late in [false, true] {
        let cancel = Cancellation::default();
        let work = engine.prepare(Command::parse(
            "render.capture",
            request(if late { "cpu" } else { "gpu" }),
        )?)?;
        let error = if late {
            let completed = work.complete_with_cancel(&cancel)?;
            cancel.cancel();
            engine
                .finish(completed)
                .err()
                .ok_or("late cancellation ignored")?
        } else {
            cancel.cancel();
            work.complete_with_cancel(&cancel)
                .err()
                .ok_or("early cancellation ignored")?
        };
        assert_eq!(error.info().code, "cancelled");
        assert_eq!(serde_json::to_value(engine.state())?, before);
        assert_eq!(engine.content(), &owner);
    }
    assert_eq!(fs::read(&next)?, Molecule::Methylamine.raw()?);
    Ok(())
}

#[test]
fn unsupported_mol_derivation_refuses_the_whole_capture_and_retains_source() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let original = std::str::from_utf8(Molecule::Ethanol.raw()?)?;
    for (name, source) in [
        (
            "unresolved-P",
            original.replace("90 O 0.25 2 0 0", "90 P 0.25 2 0 0"),
        ),
        ("double-bond", original.replace("7 1 10 40", "7 2 10 40")),
    ] {
        let input = directory.path(&format!("{name}.mol"));
        let output = directory.path(&format!("retained-{name}.mol"));
        fs::write(&input, source.as_bytes())?;
        let mut engine = Engine::default();
        engine.dispatch("document.open_mol", json!({"path":input}))?;
        assert!(engine.stored_document()?.is_fully_mapped());
        let before = serde_json::to_value(engine.state())?;
        let owner = engine.content().clone();
        for backend in ["cpu", "gpu"] {
            assert_eq!(
                engine
                    .dispatch("render.capture", request(backend))
                    .err()
                    .ok_or("partial unsupported MOL capture")?
                    .info()
                    .code,
                "unsupported_content"
            );
            assert_eq!(serde_json::to_value(engine.state())?, before);
            assert_eq!(engine.content(), &owner);
            assert_eq!(engine.stored_document()?.export_mol()?, source.as_bytes());
        }
        engine.dispatch("document.save_mol", json!({"path":output}))?;
        assert_eq!(fs::read(&input)?, source.as_bytes());
        assert_eq!(fs::read(&output)?, source.as_bytes());
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum Control {
    Expected,
    BareHeteroatom,
    MissingBond,
    Mirrored,
    LiteralH(u16),
    UnknownAuthored,
}

fn literal_page(molecule: Molecule, control: Control) -> Result<Page, Box<dyn std::error::Error>> {
    let mut document = Document::default();
    let mut atoms = Vec::new();
    let mut bonds = Vec::new();
    let min_x = molecule
        .atoms()
        .iter()
        .map(|atom| atom.2)
        .fold(f64::INFINITY, f64::min);
    let max_x = molecule
        .atoms()
        .iter()
        .map(|atom| atom.2)
        .fold(f64::NEG_INFINITY, f64::max);
    for (index, &(element, count, x, y)) in molecule.atoms().iter().enumerate() {
        let mut atom = Atom::new(
            AtomId::new(u32::try_from(index + 1)?).ok_or("atom ID")?,
            Element::new(element).ok_or("element")?,
            Some(Point2::new(
                if matches!(control, Control::Mirrored) {
                    min_x + max_x - x
                } else {
                    x
                },
                y,
            )?),
        );
        atom.explicit_hydrogens = match control {
            Control::UnknownAuthored => None,
            Control::BareHeteroatom if element != 6 => Some(0),
            Control::LiteralH(value) if element != 6 => Some(value),
            _ => Some(count),
        };
        atoms.push(atom);
        if index > 0
            && !(matches!(control, Control::MissingBond) && index + 1 == molecule.atoms().len())
        {
            bonds.push(Bond {
                id: BondId::new(u32::try_from(index)?).ok_or("bond ID")?,
                begin: AtomId::new(u32::try_from(index)?).ok_or("begin ID")?,
                end: AtomId::new(u32::try_from(index + 1)?).ok_or("end ID")?,
                order: BondOrder::Single,
            });
        }
    }
    document.append_fragments(
        PageId::new(1).ok_or("page ID")?,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment ID")?,
            atoms,
            bonds,
        )?],
    )?;
    Ok(document
        .pages()
        .first()
        .ok_or("literal expected page")?
        .clone())
}

fn literal_png(molecule: Molecule, control: Control) -> Result<String, Box<dyn std::error::Error>> {
    let scene = graph_page_scene(1024, 1024, &literal_page(molecule, control)?)?;
    Ok(STANDARD.encode(bondline_io::encode_png(&cpu_scene(scene.scene())?)?))
}

fn check_pixels(capture: &Value, molecule: Molecule) -> TestResult {
    let payload = json!({
        "capture":capture,"name":molecule.name(),"atoms":molecule.atoms(),
        "reference":literal_png(molecule,Control::Expected)?,
        "bare":literal_png(molecule,Control::BareHeteroatom)?,
        "missing_bond":literal_png(molecule,Control::MissingBond)?,
        "mirrored":literal_png(molecule,Control::Mirrored)?
    });
    let mut process = Process::new(python()).args(["-B", "-c", r#"
import base64,io,json,sys,PIL
from PIL import Image,ImageChops
assert PIL.__version__=='12.3.0'
p=json.load(sys.stdin)
def image(value):
    raw=base64.b64decode(value,validate=True)
    with Image.open(io.BytesIO(raw)) as im:
        im.verify()
    with Image.open(io.BytesIO(raw)) as im:
        assert im.format=='PNG' and im.mode=='RGBA' and im.size==(1024,1024)
        return im.copy()
a=image(p['capture']['png_base64']);r=image(p['reference']);bare=image(p['bare'])
# Independently derive the existing centered A4 viewport and source-point positions.
pw=210*72/25.4;ph=297*72/25.4
w=round(pw/ph*960);h=960;left=(1024-w)//2;top=(1024-h)//2
def point(atom):
    return (left+atom[2]/pw*w,top+atom[3]/ph*h)
x,y=point(p['atoms'][-1]);box=(int(x)-42,int(y)-18,int(x)+43,int(y)+19)
assert ImageChops.difference(r.crop(box),bare.crop(box)).convert('RGB').getbbox(), 'missing-H control is ineffective'
assert a.crop(box).tobytes()==r.crop(box).tobytes(), p['name']+': ordinary MOL OH/NH2 label differs from literal-H expectation'
assert a.tobytes()==r.tobytes(), p['name']+': complete source-direction RGBA differs'
black=lambda im:sum(tuple(pixel)==(0,0,0,255) for pixel in im.get_flattened_data())
missing=image(p['missing_bond']);mirrored=image(p['mirrored'])
assert r.tobytes()!=missing.tobytes() and r.tobytes()!=mirrored.tobytes()
begin,end=map(point,p['atoms'][-2:]);mx,my=((begin[0]+end[0])/2,(begin[1]+end[1])/2)
chord=(int(mx)-1,int(my)-1,int(mx)+3,int(my)+3)
assert black(a.crop(chord))>0,'missing actual source-coordinate bond midpoint'
assert black(missing.crop(chord))==0,'missing-bond control is ineffective'
print(json.dumps({'name':p['name'],'rgba_bytes':len(a.tobytes()),'heteroatom_region':box,'controls':3}))
"#]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    serde_json::to_writer(process.stdin.take().ok_or("pixel oracle stdin")?, &payload)?;
    let output = process.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

fn assert_authored(engine: &Engine, molecule: Molecule) -> TestResult {
    assert!(engine.stored_document()?.is_fully_mapped());
    assert_eq!(engine.stored_document()?.export_mol()?, molecule.raw()?);
    let document = engine.document()?;
    let page = document.pages().first().ok_or("opened page")?;
    let fragment = page.fragments().first().ok_or("opened fragment")?;
    assert_eq!(page.fragments().len(), 1);
    assert_eq!(fragment.id().get(), 1);
    assert_eq!(fragment.atoms().len(), molecule.atoms().len());
    for (index, (actual, &(element, _, x, y))) in
        fragment.atoms().iter().zip(molecule.atoms()).enumerate()
    {
        assert_eq!(actual.id.get(), u32::try_from(index + 1)?);
        assert_eq!(actual.element.get(), element);
        assert_eq!(actual.position, Some(Point2::new(x, y)?));
        assert_eq!(
            actual.explicit_hydrogens, None,
            "display must not author inferred H"
        );
    }
    let state = serde_json::to_value(engine.state())?;
    assert_eq!(
        state["source"],
        json!({"format":"mol","byte_length":molecule.raw()?.len(),"modified":false})
    );
    assert_eq!(state["file"]["format"], molecule.format());
    assert!(state.get("collection").is_none());
    Ok(())
}

#[test]
fn ordinary_valence_absent_mol_displays_oh_and_nh2_without_authoring_h() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    for molecule in [Molecule::Ethanol, Molecule::Methylamine] {
        let input = directory.path(&format!("original-λ-{}.mol", molecule.name()));
        let output = directory.path(&format!("exact-λ-{}.data", molecule.name()));
        fs::write(&input, molecule.raw()?)?;
        independent_molecule(molecule, &fs::read(&input)?)?;
        let mut engine = Engine::default();
        engine.dispatch("document.open_mol", json!({"path":input}))?;
        assert_authored(&engine, molecule)?;
        engine.dispatch(
            "document.close",
            json!({"expected_revision":1,"action":"request"}),
        )?;
        let before = serde_json::to_value(engine.state())?;
        let owner = engine.content().clone();
        let mut cpu = None;
        for backend in ["cpu", "gpu"] {
            let result = engine.dispatch("render.capture", request(backend))?;
            let mut result = serde_json::to_value(result)?;
            let capture = result
                .as_object_mut()
                .ok_or("result object")?
                .remove("capture")
                .ok_or("capture")?;
            assert_eq!(result, before);
            assert_eq!(capture["backend"], backend);
            if backend == "cpu" {
                assert_eq!(capture["adapter"], Value::Null);
                cpu = Some(capture["png_base64"].clone());
            } else {
                assert!(
                    capture["adapter"]["name"]
                        .as_str()
                        .is_some_and(|name| !name.is_empty())
                );
                assert_eq!(Some(&capture["png_base64"]), cpu.as_ref());
            }
            check_pixels(&capture, molecule)?;
            assert_eq!(serde_json::to_value(engine.state())?, before);
            assert_eq!(engine.content(), &owner);
            assert_authored(&engine, molecule)?;
        }
        engine.dispatch("document.save_mol", json!({"path":output}))?;
        assert_eq!(fs::read(&input)?, molecule.raw()?);
        assert_eq!(fs::read(&output)?, molecule.raw()?);
        independent_molecule(molecule, &fs::read(&output)?)?;
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
        reopened.dispatch("document.open_mol", json!({"path":output}))?;
        assert_authored(&reopened, molecule)?;
        assert_eq!(reopened.document()?, engine.document()?);
    }
    Ok(())
}
