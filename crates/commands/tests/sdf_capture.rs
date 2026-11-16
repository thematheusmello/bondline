use base64::{Engine as _, engine::general_purpose::STANDARD};
use bondline_commands::{Cancellation, Command, Engine};
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Document, Element, Fragment, FragmentId, Page, PageId,
    Point2,
};
use bondline_render::{cpu_scene, graph_page_scene};
use serde_json::json;
use std::{
    fs,
    io::Write,
    process::{Command as Process, Stdio},
};
#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse only the existing isolated file directory")]
mod support;
type TestResult = Result<(), Box<dyn std::error::Error>>;

// Original fixed-column V2000 and explicit V3000: no VAL or HCOUNT declaration.
const ETHANOL: &[u8] = b"Bondline original ethanol\n  Bondline          2D\nVAL absent, source directions\n  3  2  0  0  0  0            999 V2000\n   -2.0000    5.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0\n    0.0000    5.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0\n    0.0000    3.0000    0.0000 O   0  0  0  0  0  0  0  0  0  0  0  0\n  1  2  1  0  0  0  0\n  2  3  1  0  0  0  0\nM  END\n";
const METHYLAMINE: &[u8] = b"Bondline original methylamine\n  Bondline          2D\nNonsequential IDs and opposite source ordering\n  0  0  0  0  0  0            999 V3000\nM  V30 BEGIN CTAB\nM  V30 COUNTS 2 1 0 0 0\nM  V30 BEGIN ATOM\nM  V30 17 N 4 -1 0 0\nM  V30 4 C 2 -2 0 0\nM  V30 END ATOM\nM  V30 BEGIN BOND\nM  V30 91 1 17 4\nM  V30 END BOND\nM  V30 END CTAB\nM  END\n";
const MIRRORED_METHYLAMINE: &[u8] = b"Bondline original methylamine\n  Bondline          2D\nNonsequential IDs and opposite source ordering\n  0  0  0  0  0  0            999 V3000\nM  V30 BEGIN CTAB\nM  V30 COUNTS 2 1 0 0 0\nM  V30 BEGIN ATOM\nM  V30 17 N 0 -1 0 0\nM  V30 4 C 2 -2 0 0\nM  V30 END ATOM\nM  V30 BEGIN BOND\nM  V30 91 1 17 4\nM  V30 END BOND\nM  V30 END CTAB\nM  END\n";
fn source() -> Vec<u8> {
    [
        ETHANOL,
        b"> <DUP>\nfirst\n\n> <DUP>\nsecond\n\n$$$$\n",
        METHYLAMINE,
        b"> DT0012\nopaque source data\n  \n\n$$$$\n",
    ]
    .concat()
}
fn request(backend: &str) -> serde_json::Value {
    json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":backend})
}

fn original(mirrored: bool) -> Vec<u8> {
    let second = if mirrored {
        MIRRORED_METHYLAMINE
    } else {
        METHYLAMINE
    };
    [
        ETHANOL,
        b"> <DUP>\nfirst\n\n> <DUP>\nsecond\n\n$$$$\n",
        second,
        b"> DT0012\nopaque source data\n  \n\n$$$$\n",
    ]
    .concat()
}

fn external(script: &str, bytes: &[u8]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Process::new(python)
        .arg("-B")
        .arg(format!(
            "{}/../io/tests/{script}",
            env!("CARGO_MANIFEST_DIR")
        ))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("oracle stdin")?.write_all(bytes)?;
    let out = child.wait_with_output()?;
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    Ok(serde_json::from_slice(&out.stdout)?)
}
fn independent_source(bytes: &[u8], mirrored: bool) -> TestResult {
    let first_end = ETHANOL.len() + b"> <DUP>\nfirst\n\n> <DUP>\nsecond\n\n$$$$\n".len();
    let raw = external(
        "sdf_oracle.py",
        bytes.get(..first_end).ok_or("first complete record")?,
    )?;
    assert_eq!(raw["records"][0]["fields"][0]["header"], "> <DUP>");
    assert_eq!(
        raw["records"][0]["fields"][0]["value_lines"],
        json!(["first"])
    );
    assert_eq!(
        raw["records"][0]["fields"][1]["value_lines"],
        json!(["second"])
    );
    let first = external(
        "mol_oracle.py",
        bytes.get(..ETHANOL.len()).ok_or("first MOL")?,
    )?;
    assert_eq!(first["smiles"], "CCO");
    assert_eq!(first["formula"], "C2H6O");
    assert_eq!(
        first["atoms"]
            .as_array()
            .ok_or("atoms")?
            .iter()
            .map(|a| a["hydrogens"].clone())
            .collect::<Vec<_>>(),
        vec![json!(3), json!(2), json!(1)]
    );
    assert_eq!(
        first["atoms"]
            .as_array()
            .ok_or("atoms")?
            .iter()
            .map(|a| a["position"].clone())
            .collect::<Vec<_>>(),
        vec![
            json!([-2.0, 5.0, 0.0]),
            json!([0.0, 5.0, 0.0]),
            json!([0.0, 3.0, 0.0])
        ]
    );
    let second_len = METHYLAMINE.len();
    let second = bytes
        .get(first_end..first_end + second_len)
        .ok_or("second MOL")?;
    let molecule = external("mol_oracle.py", second)?;
    assert_eq!(molecule["smiles"], "CN");
    assert_eq!(molecule["formula"], "CH5N");
    assert_eq!(
        molecule["atoms"]
            .as_array()
            .ok_or("atoms")?
            .iter()
            .map(|a| a["hydrogens"].clone())
            .collect::<Vec<_>>(),
        vec![json!(2), json!(3)]
    );
    assert_eq!(
        molecule["atoms"][0]["position"],
        json!([if mirrored { 0.0 } else { 4.0 }, -1.0, 0.0])
    );
    assert_eq!(molecule["atoms"][1]["position"], json!([2.0, -2.0, 0.0]));
    assert_eq!(molecule["bonds"], json!([{"begin":0,"end":1,"order":1.0}]));
    let v3 = external("v3000_oracle.py", second)?;
    assert_eq!(v3["atoms"][0]["explicit_h"], 0);
    assert_eq!(v3["atoms"][1]["explicit_h"], 0);
    assert_eq!(
        bytes.get(first_end + second_len..),
        Some(b"> DT0012\nopaque source data\n  \n\n$$$$\n".as_slice())
    );
    Ok(())
}

fn expected_page(mirrored: bool, control: u8) -> Result<Page, Box<dyn std::error::Error>> {
    let mut document = Document::default();
    let mut fragments = Vec::new();
    for (index, rows) in [
        vec![
            (1, 6, 3, 72.0, 72.0),
            (2, 6, 2, 120.0, 72.0),
            (3, 8, if control == 2 { 0 } else { 1 }, 120.0, 120.0),
        ],
        vec![
            (
                4,
                7,
                if control == 2 { 0 } else { 2 },
                if mirrored { 72.0 } else { 120.0 },
                if control == 3 { 216.0 } else { 192.0 },
            ),
            (
                5,
                6,
                3,
                if mirrored { 120.0 } else { 72.0 },
                if control == 3 { 192.0 } else { 216.0 },
            ),
        ],
    ]
    .into_iter()
    .enumerate()
    {
        if index == 1 && control == 1 {
            continue;
        }
        let atoms = rows
            .into_iter()
            .map(|(id, z, h, x, y)| {
                let mut a = Atom::new(
                    AtomId::new(id).ok_or("atom")?,
                    Element::new(z).ok_or("element")?,
                    Some(Point2::new(x, y)?),
                );
                a.explicit_hydrogens = Some(h);
                Ok(a)
            })
            .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
        let edges = if index == 0 {
            vec![(1, 1, 2), (2, 2, 3)]
        } else {
            vec![(3, 4, 5)]
        };
        let bonds = edges
            .into_iter()
            .map(|(id, a, b)| {
                Ok(Bond {
                    double_position: bondline_model::DoublePosition::Unspecified,
                    id: BondId::new(id).ok_or("bond")?,
                    begin: AtomId::new(a).ok_or("begin")?,
                    end: AtomId::new(b).ok_or("end")?,
                    order: BondOrder::Single,
                })
            })
            .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
        fragments.push(Fragment::new(
            FragmentId::new(u32::try_from(index + 1)?).ok_or("fragment")?,
            atoms,
            bonds,
        )?);
    }
    document.append_fragments(PageId::new(1).ok_or("page")?, fragments)?;
    Ok(document.pages().first().ok_or("expected page")?.clone())
}
fn literal_reference(mirrored: bool, control: u8) -> Result<String, Box<dyn std::error::Error>> {
    let scene = graph_page_scene(1024, 1024, &expected_page(mirrored, control)?)?;
    Ok(STANDARD.encode(bondline_io::encode_png(&cpu_scene(scene.scene())?)?))
}
fn pixels(cpu: &serde_json::Value, gpu: &serde_json::Value, mirrored: bool) -> TestResult {
    let payload = json!({"cpu":cpu,"gpu":gpu,"reference":literal_reference(mirrored,0)?,"missing_row":literal_reference(mirrored,1)?,"wrong_h":literal_reference(mirrored,2)?,"wrong_y":literal_reference(mirrored,3)?,"mirrored":mirrored});
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut process=Process::new(python).args(["-B","-c",r#"
import base64,io,json,sys,PIL
from PIL import Image,ImageChops
assert PIL.__version__=='12.3.0'
p=json.load(sys.stdin)
def image(text):
 b=base64.b64decode(text,validate=True)
 with Image.open(io.BytesIO(b)) as im:
  assert im.format=='PNG' and im.mode=='RGBA' and im.size==(1024,1024)
  return im.copy()
a=image(p['cpu']['capture']['png_base64']);b=image(p['gpu']['capture']['png_base64']);r=image(p['reference'])
assert a.tobytes()==b.tobytes()==r.tobytes(), 'fullRGBA or complete literal two-row sheet differs'
assert p['cpu']['document'] is None and p['gpu']['document'] is None
assert p['cpu']['capture']['adapter'] is None and p['gpu']['capture']['adapter']['name']
rows=[(220,95,345,190),(220,235,345,300)]
black=lambda im:sum(tuple(v)==(0,0,0,255) for v in im.get_flattened_data())
assert all(black(a.crop(box))>80 for box in rows)
assert black(a.crop((220,194,345,230)))==0,'rows collapsed'
missing=image(p['missing_row']);wrongh=image(p['wrong_h']);wrongy=image(p['wrong_y'])
assert black(missing.crop(rows[1]))==0
assert all(ImageChops.difference(r.crop(box),wrongh.crop(box)).convert('RGB').getbbox() for box in rows),'missing OH/NH2 did not affect both rows'
assert a.tobytes()!=missing.tobytes() and a.tobytes()!=wrongh.tobytes() and a.tobytes()!=wrongy.tobytes()
# Analytic midpoint/quarterpoint tests are independent of the reference raster producer.
for x,y in [(281,114),(309,141),((267 if p['mirrored'] else 295),258)]:
 assert black(a.crop((x-1,y-1,x+2,y+2)))>0,('missing source-direction chord',x,y)
x=267 if p['mirrored'] else 295
assert black(a.crop((x-1,270,x+2,273)))==0,'source y direction inverted'
print(json.dumps({'rgba_bytes':len(a.tobytes()),'rows':2,'chords':3,'negative_controls':3}))
"#]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    serde_json::to_writer(process.stdin.take().ok_or("pixel stdin")?, &payload)?;
    let output = process.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

#[test]
fn complete_mixed_sdf_capture_displays_both_rows_without_becoming_a_drawing() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    for mirrored in [false, true] {
        let input = directory.path(&format!("original-{mirrored}.sdf"));
        let output = directory.path(&format!("copy-{mirrored}.data"));
        let bytes = original(mirrored);
        if !mirrored {
            assert_eq!(bytes, source());
        }
        fs::write(&input, &bytes)?;
        independent_source(&fs::read(&input)?, mirrored)?;
        let mut engine = Engine::default();
        engine.dispatch("document.open_sdf", json!({"path":input}))?;
        let before_queries = serde_json::to_value(engine.state())?;
        let first_query = serde_json::to_value(engine.dispatch(
            "sdf.record.inspect",
            json!({"expected_revision":1,"record_id":1}),
        )?)?;
        assert_eq!(first_query["sdf_record"]["fields"][0]["name"], "DUP");
        assert_eq!(
            first_query["sdf_record"]["fields"][0]["value_lines"],
            json!(["first"])
        );
        assert_eq!(first_query["sdf_record"]["fields"][1]["name"], "DUP");
        assert_eq!(
            first_query["sdf_record"]["fields"][1]["value_lines"],
            json!(["second"])
        );
        let second_query = serde_json::to_value(engine.dispatch(
            "sdf.record.inspect",
            json!({"expected_revision":1,"record_id":2}),
        )?)?;
        assert_eq!(second_query["sdf_record"]["fields"][0]["dt_number"], "0012");
        assert_eq!(
            second_query["sdf_record"]["fields"][0]["value_lines"],
            json!(["opaque source data", "  "])
        );
        assert_eq!(serde_json::to_value(engine.state())?, before_queries);
        engine.dispatch(
            "document.close",
            json!({"expected_revision":1,"action":"request"}),
        )?;
        let before = serde_json::to_value(engine.state())?;
        let owner = engine.content().clone();
        let mut results = Vec::new();
        for backend in ["cpu", "gpu"] {
            let result = engine.dispatch("render.capture", request(backend))?;
            assert!(!result.changed);
            assert!(result.document.is_none());
            assert_eq!(result.revision, 1);
            let mut value = serde_json::to_value(result)?;
            assert!(value.get("capture").is_some());
            let copy = value.clone();
            value.as_object_mut().ok_or("state")?.remove("capture");
            assert_eq!(value, before);
            results.push(copy);
            assert_eq!(serde_json::to_value(engine.state())?, before);
            assert_eq!(engine.content(), &owner);
            assert_eq!(engine.content().sdf()?.as_bytes(), bytes);
        }
        pixels(
            results.first().ok_or("cpu")?,
            results.get(1).ok_or("gpu")?,
            mirrored,
        )?;
        engine.dispatch("document.save_sdf", json!({"path":output}))?;
        assert_eq!(fs::read(&output)?, bytes);
        assert_eq!(fs::read(&input)?, bytes);
        independent_source(&fs::read(&output)?, mirrored)?;
        let mut reopened = Engine::default();
        reopened.dispatch("document.open_sdf", json!({"path":output}))?;
        assert_eq!(reopened.content().sdf()?.as_bytes(), bytes);
        assert!(reopened.state().document.is_none());
    }
    Ok(())
}

#[test]
fn virtual_page_and_cancellation_fail_without_mutating_the_complete_sdf() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("cancel.sdf");
    fs::write(&input, source())?;
    let mut engine = Engine::default();
    engine.dispatch("document.open_sdf", json!({"path":input}))?;
    let before = serde_json::to_value(engine.state())?;
    let owner = engine.content().clone();
    let mut missing = request("cpu");
    missing["page_id"] = json!(2);
    assert_eq!(
        engine
            .dispatch("render.capture", missing)
            .err()
            .ok_or("accepted absent virtualpage")?
            .info()
            .code,
        "not_found"
    );
    let cancel = Cancellation::default();
    cancel.cancel();
    let work = engine.prepare(Command::parse("render.capture", request("gpu"))?)?;
    assert!(work.is_deferred());
    assert_eq!(
        work.complete_with_cancel(&cancel)
            .err()
            .ok_or("cancel ignored")?
            .info()
            .code,
        "cancelled"
    );
    assert_eq!(engine.content(), &owner);
    assert_eq!(serde_json::to_value(engine.state())?, before);
    assert_eq!(fs::read(&input)?, source());
    Ok(())
}

#[test]
fn later_multiple_bond_renders_while_degenerate_or_empty_records_refuse_atomically() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let base = String::from_utf8(source())?;
    let empty=b"Empty original\n  Bondline          2D\nEmpty CTAB is not a drawing\n  0  0  0  0  0  0            999 V2000\nM  END\n";
    let empty_source = [ETHANOL, b"$$$$\n", empty, b"$$$$\n"].concat();
    for (case, bytes) in [
        base.replace("91 1 17 4", "91 2 17 4").into_bytes(),
        base.replace("4 C 2 -2 0 0", "4 C 4 -1 0 0").into_bytes(),
        empty_source,
    ]
    .into_iter()
    .enumerate()
    {
        let input = directory.path(&format!("refusal-{case}.sdf"));
        let output = directory.path(&format!("retained-{case}.sdf"));
        fs::write(&input, &bytes)?;
        let mut engine = Engine::default();
        engine.dispatch("document.open_sdf", json!({"path":input}))?;
        let before = serde_json::to_value(engine.state())?;
        let owner = engine.content().clone();
        let expected = if case == 0 {
            let second = String::from_utf8(METHYLAMINE.to_vec())?.replace("91 1 17 4", "91 2 17 4");
            let observed = external("mol_oracle.py", second.as_bytes())?;
            assert_eq!(observed["smiles"], "C=N");
            assert_eq!(observed["formula"], "CH3N");
            assert_eq!(observed["atoms"][0]["hydrogens"], 1);
            assert_eq!(observed["atoms"][1]["hydrogens"], 2);
            let mut expected = Document::default();
            for fragment in expected_page(false, 0)?.fragments() {
                let mut atoms = fragment.atoms().to_vec();
                let mut bonds = fragment.bonds().to_vec();
                if fragment.id().get() == 2 {
                    atoms[0].explicit_hydrogens = Some(1);
                    atoms[1].explicit_hydrogens = Some(2);
                    bonds[0].order = BondOrder::Double;
                }
                expected.append_fragments(
                    PageId::new(1).ok_or("page")?,
                    vec![Fragment::new(fragment.id(), atoms, bonds)?],
                )?;
            }
            let scene = graph_page_scene(1024, 1024, &expected.pages()[0])?;
            let png = STANDARD.encode(bondline_io::encode_png(&cpu_scene(scene.scene())?)?);
            assert_ne!(
                png,
                literal_reference(false, 0)?,
                "multiple record must not collapse to old Single/H2/H3"
            );
            Some(png)
        } else {
            None
        };
        for backend in ["cpu", "gpu"] {
            if let Some(expected) = &expected {
                let capture = engine
                    .dispatch("render.capture", request(backend))?
                    .capture
                    .ok_or("all-record multiple capture missing")?;
                assert_eq!(
                    capture.png_base64, *expected,
                    "both actual records, coordinates, H counts and bond components"
                );
                assert_eq!(capture.adapter.is_some(), backend == "gpu");
                assert_eq!((capture.width_px, capture.height_px), (1024, 1024));
            } else {
                assert!(
                    engine.dispatch("render.capture", request(backend)).is_err(),
                    "partial capture forcase{case}"
                );
            }
            assert_eq!(engine.content(), &owner);
            assert_eq!(serde_json::to_value(engine.state())?, before);
        }
        engine.dispatch("document.save_sdf", json!({"path":output}))?;
        assert_eq!(fs::read(&output)?, bytes);
        assert_eq!(fs::read(&input)?, bytes);
    }
    Ok(())
}
