//! Frozen existing-API declarations before the separately qualified Connected source grant.
use bondline_io::StoredDocument;
use bondline_model::PageSize;
use serde_json::{Value, json};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn source(name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let data: Value = serde_json::from_str(include_str!("fixtures/connected-styles/vectors.json"))?;
    assert_eq!(data["fonttools"], "4.61.1");
    assert_eq!(
        data["font_sha256"],
        "56a45233d29f11b4dfb86d248e921939d115778f87325e7ae8cc108383d6664d"
    );
    Ok(data["cases"]
        .as_array()
        .ok_or("cases")?
        .iter()
        .find(|case| case["name"] == name)
        .ok_or("case")?["source"]
        .as_str()
        .ok_or("source")?
        .to_owned())
}

fn display_contract(name: &str) -> TestResult {
    let raw = source(name)?;
    let stored = StoredDocument::from_cdxml(raw.as_bytes().into(), PageSize::A4)?;
    assert_eq!(
        serde_json::to_value(stored.model())?["drawing_style"]["margin_width_pt"],
        json!(1.6),
        "the data prerequisite must already project explicit MarginWidth"
    );
    assert!(!stored.is_fully_mapped());
    assert!(
        !stored.has_boxed_atom_labels(),
        "the foreign zero-bond Boxed contract remains exact"
    );
    assert_eq!(stored.to_cdxml()?.as_ref(), raw.as_bytes());
    assert!(
        stored.has_display_projection(),
        "fully proved connected styles need their separate display grant"
    );
    Ok(())
}

#[test]
fn explicit_water_single_style_has_a_separate_display_grant() -> TestResult {
    display_contract("water")
}

#[test]
fn explicit_carbon_dioxide_centered_double_style_has_a_separate_display_grant() -> TestResult {
    display_contract("carbon-dioxide")
}

#[test]
fn explicit_hydrogen_cyanide_triple_style_has_a_separate_display_grant() -> TestResult {
    display_contract("hydrogen-cyanide")
}

use bondline_model::{Element, Fragment};
use std::{
    borrow::Cow,
    io::Write,
    process::{Command, Stdio},
    sync::Arc,
};
type Out<T> = Result<T, Box<dyn std::error::Error>>;

fn vectors() -> Out<Value> {
    Ok(serde_json::from_str(include_str!(
        "fixtures/connected-styles/vectors.json"
    ))?)
}
fn stored(raw: &str) -> Out<StoredDocument> {
    Ok(StoredDocument::from_cdxml(
        raw.as_bytes().into(),
        PageSize::A4,
    )?)
}
fn connected(raw: &str) -> Out<StoredDocument> {
    let own = stored(raw)?;
    assert!(
        own.has_connected_atom_labels(),
        "qualified connected source: {raw}"
    );
    assert!(
        !own.has_boxed_atom_labels(),
        "foreign Boxed must remain zero-bond"
    );
    assert!(!own.is_fully_mapped());
    assert!(own.has_display_projection());
    assert_eq!(own.to_cdxml()?.as_ref(), raw.as_bytes());
    Ok(own)
}
fn edit_middle(base: &StoredDocument) -> Out<StoredDocument> {
    let page = base.model().pages().first().ok_or("page")?;
    let fragment = page.fragments().first().ok_or("fragment")?;
    let mut atoms = fragment.atoms().to_vec();
    let atom = atoms.get_mut(1).ok_or("middle atom")?;
    atom.element = Element::try_from(7)?;
    atom.explicit_hydrogens = Some(1);
    atom.authored_label = Some(Arc::new(bondline_render::position_atom_label(atom)?));
    let mut result = base.clone();
    result.model_mut().replace_fragment(
        page.id(),
        Fragment::new(fragment.id(), atoms, fragment.bonds().to_vec())?,
    )?;
    Ok(result)
}
fn opaque(raw: &str) -> TestResult {
    let base = stored(raw)?;
    assert!(
        !base.has_connected_atom_labels(),
        "unsupported connected source: {raw}"
    );
    assert!(!base.has_boxed_atom_labels());
    assert!(!base.has_display_projection());
    assert!(!base.is_fully_mapped());
    assert_eq!(base.to_cdxml()?.as_ref(), raw.as_bytes());
    let mut same = base.clone();
    *same.model_mut() = base.model().clone();
    assert!(!same.has_connected_atom_labels());
    assert!(matches!(same.to_cdxml()?, Cow::Borrowed(_)));
    let page = same.model().pages().first().ok_or("page")?.id();
    same.model_mut()
        .set_page_size(page, PageSize::new(513.0, 512.0)?)?;
    assert!(matches!(
        same.to_cdxml(),
        Err(bondline_io::DocumentError::UnsupportedEdit)
    ));
    assert_eq!(base.to_cdxml()?.as_ref(), raw.as_bytes());
    Ok(())
}
fn atom_xml(id: u32, element: u8, point: [f64; 2]) -> Out<String> {
    let data = vectors()?;
    let metric = data["cases"]
        .as_array()
        .ok_or("cases")?
        .iter()
        .flat_map(|case| case["nodes"].as_array().into_iter().flatten())
        .find(|node| node["element"] == element && node["h"] == 0)
        .ok_or("glyph metric")?;
    let dx = point[0] - metric["point"][0].as_f64().ok_or("metric x")?;
    let dy = point[1] - metric["point"][1].as_f64().ok_or("metric y")?;
    let bounds = metric["bounding_box"]
        .as_array()
        .ok_or("metric box")?
        .iter()
        .enumerate()
        .map(|(i, v)| {
            v.as_f64()
                .map(|v| (v + if i % 2 == 0 { dx } else { dy }).to_string())
                .ok_or("metric number")
        })
        .collect::<Result<Vec<_>, _>>()?
        .join(" ");
    let body = metric["text"].as_str().ok_or("metric body")?;
    Ok(format!(
        "<n id='{id}' NodeType='Element' Element='{element}' Charge='0' Radical='None' NumHydrogens='0' p='{} {}'><t id='21' BoundingBox='{bounds}' Justification='Left' LabelJustification='Left'><s font='42' size='12' face='0' color='4'>{body}</s></t></n>",
        point[0], point[1]
    ))
}
fn fragment_xml(id: u32, points: &[[f64; 2]], edges: &[(usize, usize, u8)]) -> Out<String> {
    let mut body = format!("<fragment id='{id}'>");
    for (i, point) in points.iter().enumerate() {
        body.push_str(&atom_xml(100 + u32::try_from(i)?, 8, *point)?);
    }
    for (i, (a, b, order)) in edges.iter().enumerate() {
        let ident = 900 + u32::try_from(i)?;
        let a = 100 + u32::try_from(*a)?;
        let b = 100 + u32::try_from(*b)?;
        let position = if *order == 2 {
            " DoublePosition='Center'"
        } else {
            ""
        };
        body.push_str(&format!(
            "<b id='{ident}' B='{a}' E='{b}' Order='{order}'{position}/>"
        ));
    }
    body.push_str("</fragment>");
    Ok(body)
}
fn geometry_source(points: &[[f64; 2]], edges: &[(usize, usize, u8)]) -> Out<String> {
    let raw = source("carbon-dioxide")?;
    let (before, inside) = raw
        .split_once("<fragment id='901'>")
        .ok_or("fragment start")?;
    let (_, after) = inside.split_once("</fragment>").ok_or("fragment end")?;
    Ok(format!(
        "{before}{}{after}",
        fragment_xml(901, points, edges)?
    ))
}
fn independent(raw: &[u8]) -> Out<Value> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child=Command::new(python).args(["-X","utf8","-B","-c",r#"
import json,sys,xml.parsers.expat as expat
from decimal import Decimal
from rdkit import Chem,rdBase
from rdkit.Chem import rdMolDescriptors
assert rdBase.rdkitVersion=='2026.09.2'
raw=sys.stdin.buffer.read(1048577);assert 0<len(raw)<=1048576
nodes=[];bonds=[];current=None;run=False;root=None
def start(name,a):
 global current,run,root
 if name=='CDXML':root={k:float(Decimal(a[k])) for k in ('LineWidth','MarginWidth')}
 if name=='n':
  current={'id':int(Decimal(a['id'])),'element':int(Decimal(a['Element'])),'h':int(Decimal(a['NumHydrogens'])),'point':[float(Decimal(v)) for v in a['p'].split()],'text':''};nodes.append(current)
 if name=='t' and current is not None:
  assert 'p' not in a
  assert a['Justification']==a['LabelJustification']=='Left'
  current['box']=[float(Decimal(v)) for v in a['BoundingBox'].split()]
 if name=='s' and current is not None:run=True
 if name=='b':bonds.append({'id':int(Decimal(a['id'])),'begin':int(Decimal(a['B'])),'end':int(Decimal(a['E'])),'order':int(Decimal(a['Order']))})
def end(name):
 global current,run
 if name=='s':run=False
 if name=='n':current=None
def chars(s):
 if run:current['text']+=s
p=expat.ParserCreate(namespace_separator='|');p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER);p.ExternalEntityRefHandler=lambda *args:0
p.StartElementHandler=start;p.EndElementHandler=end;p.CharacterDataHandler=chars;p.Parse(raw,True)
assert all(b['begin'] in {n['id'] for n in nodes} and b['end'] in {n['id'] for n in nodes} for b in bonds)
mols=Chem.MolsFromCDXML(raw.decode('utf8'),sanitize=False,removeHs=False);assert len(mols)==1
m=mols[0];m.UpdatePropertyCache(strict=False)
print(json.dumps({'nodes':nodes,'bonds':bonds,'root':root,'rdkit':rdBase.rdkitVersion,'atom_numbers':[a.GetAtomicNum() for a in m.GetAtoms()],'explicit_hydrogens':[a.GetNumExplicitHs() for a in m.GetAtoms()],'orders':[b.GetBondTypeAsDouble() for b in m.GetBonds()],'formula':rdMolDescriptors.CalcMolFormula(m)},separators=(',',':')))
"#]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    child.stdin.take().ok_or("consumer stdin")?.write_all(raw)?;
    let out = child.wait_with_output()?;
    if !out.status.success() {
        return Err(format!(
            "independent connected consumer: {}",
            String::from_utf8_lossy(&out.stderr)
        )
        .into());
    }
    Ok(serde_json::from_slice(&out.stdout)?)
}

#[test]
fn every_complete_profile_and_actual_source_face_direct_expat_decimal_and_rdkit_consumers()
-> TestResult {
    for (name, formula, numbers, orders) in [
        ("water", "H2O", vec![1, 8, 1], vec![1.0, 1.0]),
        ("carbon-dioxide", "CO2", vec![8, 6, 8], vec![2.0, 2.0]),
        ("hydrogen-cyanide", "CHN", vec![1, 6, 7], vec![1.0, 3.0]),
    ] {
        let raw = source(name)?;
        let base = connected(&raw)?;
        let facts = independent(raw.as_bytes())?;
        assert_eq!(facts["formula"], formula);
        assert_eq!(facts["atom_numbers"], json!(numbers));
        assert_eq!(facts["orders"], json!(orders));
        assert_eq!(facts["explicit_hydrogens"], json!([0, 0, 0]));
        assert_eq!(facts["root"], json!({"LineWidth":0.6,"MarginWidth":1.6}));
        assert_eq!(base.to_cdxml()?.as_ref(), raw.as_bytes());
    }
    let bad = source("water")?.replace("E='207'", "E='99999'");
    assert!(
        independent(bad.as_bytes()).is_err(),
        "qualify actual endpoint consumption"
    );
    Ok(())
}

#[test]
fn water_to_ammonia_changes_only_existing_element_h_body_and_independent_box() -> TestResult {
    let raw = source("water")?;
    let base = connected(&raw)?;
    let current = edit_middle(&base)?;
    assert!(current.has_connected_atom_labels());
    assert!(!current.has_boxed_atom_labels());
    assert!(!current.is_fully_mapped());
    let encoded = current.to_cdxml()?.into_owned();
    let facts = independent(&encoded)?;
    assert_eq!(facts["formula"], "H3N");
    assert_eq!(facts["atom_numbers"], json!([1, 7, 1]));
    assert_eq!(facts["explicit_hydrogens"], json!([0, 1, 0]));
    let data = vectors()?;
    let expected = &data["ammonia_middle"];
    assert_eq!(facts["nodes"][1]["box"], expected["bounding_box"]);
    assert_eq!(facts["nodes"][1]["text"], "NH");
    assert_eq!(facts["nodes"][1]["id"], 207);
    assert_eq!(facts["nodes"][1]["point"], json!([128.0, 100.0]));
    assert_eq!(facts["bonds"], independent(raw.as_bytes())?["bonds"]);
    let vector = data["cases"]
        .as_array()
        .ok_or("cases")?
        .iter()
        .find(|v| v["name"] == "water")
        .ok_or("water")?;
    let numbers = |v: &Value| -> Out<String> {
        Ok(v.as_array()
            .ok_or("numbers")?
            .iter()
            .map(|v| v.as_f64().map(|x| x.to_string()).ok_or("number"))
            .collect::<Result<Vec<_>, _>>()?
            .join(" "))
    };
    let old_box = numbers(&vector["nodes"][1]["bounding_box"])?;
    let new_box = numbers(&expected["bounding_box"])?;
    let literal = raw
        .replace(
            "id='207' NodeType='Element' Element='8'",
            "id='207' NodeType='Element' Element='7'",
        )
        .replace(
            "Element='7' Charge='0' Radical='None' NumHydrogens='0'",
            "Element='7' Charge='0' Radical='None' NumHydrogens='1'",
        )
        .replace(">O</s>", ">NH</s>")
        .replace(&old_box, &new_box);
    assert_eq!(
        encoded,
        literal.as_bytes(),
        "every byte outside four owned values remains exact"
    );
    let reopened = stored(std::str::from_utf8(&encoded)?)?;
    assert_eq!(reopened.model(), current.model());
    assert!(reopened.has_connected_atom_labels());
    assert_eq!(base.to_cdxml()?.as_ref(), raw.as_bytes());
    let mut same = base.clone();
    *same.model_mut() = base.model().clone();
    assert!(same.has_connected_atom_labels());
    assert!(matches!(same.to_cdxml()?, Cow::Borrowed(_)));
    assert_eq!(same.to_cdxml()?.as_ref(), raw.as_bytes());
    Ok(())
}

#[test]
fn relevant_missing_overridden_unknown_and_ambiguous_styles_keep_opaque_unchanged_only_sources()
-> TestResult {
    let water = source("water")?;
    let double = source("carbon-dioxide")?;
    let triple = source("hydrogen-cyanide")?;
    let variants = vec![
        water.replace(" LineWidth='.60'", ""),
        water.replace(" MarginWidth='1.600'", ""),
        water.replace("MarginWidth='1.600'", "MarginWidth='-1e-9999'"),
        water.replace("<b ", "<b LineWidth='.60' "),
        water.replace("<b ", "<b MarginWidth='1.6' "),
        water.replace("<n ", "<n MarginWidth='1.6' "),
        water.replace("<n ", "<n LineWidth='.6' "),
        double.replace(" BondLength='48.000'", ""),
        double.replace(" BondSpacing='12.0'", ""),
        double.replace("<b ", "<b BondLength='48' "),
        double.replace("<b ", "<b BondSpacing='12' "),
        double.replace("<b ", "<b BondSpacingAbs='5.76' "),
        double.replace("<CDXML ", "<CDXML BondSpacingAbs='5.76' "),
        double.replace(" DoublePosition='Center'", ""),
        double.replace("DoublePosition='Center'", "DoublePosition='Left'"),
        double.replace("DoublePosition='Center'", "DoublePosition='Right'"),
        water.replace("<b ", "<b DoublePosition='Center' "),
        triple.replace("<b ", "<b DoublePosition='Center' "),
        water.replace("<b ", "<b Display='Dash' "),
        water.replace("<b ", "<b Display='Bold' "),
        double.replace("<b ", "<b Display='Wavy' "),
        double.replace("<b ", "<b Display2='Dash' "),
        water.replace("<b ", "<b BeginAttach='0' "),
        water.replace("<b ", "<b EndAttach='0' "),
        water.replace("<b ", "<b Future='retain' "),
        water.replace("<CDXML ", "<CDXML xmlns:unused='urn:unused' "),
        water.replace("<b ", "<b xml:color='3' "),
        water.replace("<t ", "<t xmlns='' "),
        water.replace("<CDXML ", "<CDXML color='3' "),
        water.replace("<page ", "<page color='3' "),
        water.replace("<fragment ", "<fragment Future='retain' "),
        water.replace(">O</s>", ">N</s>"),
        water.replace("face='0'", "face='96'"),
        water.replace("<t ", "<t p='128 100' "),
        water.replace("<b ", "<b Z='1' "),
        format!("<?future keep?>{water}"),
        format!("<!DOCTYPE CDXML>{water}"),
        water.replace(">O</s>", "><![CDATA[O]]></s>"),
    ];
    for raw in variants {
        opaque(&raw)?;
    }
    Ok(())
}

#[test]
fn solid_spellings_black_colour_resolution_and_missing_vs_short_palette_are_independent()
-> TestResult {
    let water = source("water")?;
    connected(&water.replace("<b ", "<b Display='Solid' Display2='Solid' "))?;
    for raw in [
        water.replace("<b ", "<b color='0' "),
        water.replace("<b ", "<b color='3.000' "),
    ] {
        connected(&raw)?;
    }
    let red = water.replace("<color r='0' g='0' b='0'/>", "<color r='1' g='0' b='0'/>");
    opaque(&red)?;
    connected(&red.replace("<b ", "<b color='0' "))?;
    for colour in ["3", "4", "65535", "-1", "-1e-9999"] {
        opaque(&red.replace("<b ", &format!("<b color='{colour}' ")))?;
    }
    let black_runs = water.replace("color='4'", "color='0'");
    let palette = "<colortable><color r='1' g='1' b='1'/><color r='0' g='0' b='0'/><color r='.2' g='.4' b='.8'/></colortable>";
    connected(&black_runs.replace(palette, ""))?;
    let short = black_runs.replace(
        palette,
        "<colortable><color r='1' g='1' b='1'/></colortable>",
    );
    opaque(&short)?;
    connected(&short.replace("<b ", "<b color='0' "))?;
    opaque(&water.replace("<color r='1' g='1' b='1'/>", "<color r='.9' g='1' b='1'/>"))?;
    Ok(())
}

#[test]
fn pure_geometry_refuses_single_crossing_incident_overlap_vertex_on_unrelated_edge_and_coincidence()
-> TestResult {
    for (points, edges) in [
        (
            vec![[80., 80.], [180., 180.], [80., 180.], [180., 80.]],
            vec![(0, 1, 1), (2, 3, 1)],
        ),
        (
            vec![[80., 200.], [128., 200.], [176., 200.]],
            vec![(0, 1, 1), (0, 2, 1)],
        ),
        (
            vec![[80., 300.], [176., 300.], [128., 300.]],
            vec![(0, 1, 1)],
        ),
        (
            vec![[80., 380.], [80., 380.], [128., 380.]],
            vec![(0, 2, 1)],
        ),
    ] {
        opaque(&geometry_source(&points, &edges)?)?;
    }
    let first = fragment_xml(901, &[[80., 80.], [180., 180.]], &[(0, 1, 1)])?;
    let second = fragment_xml(902, &[[80., 180.], [180., 80.]], &[(0, 1, 1)])?;
    let sample = source("carbon-dioxide")?;
    let (before, inside) = sample.split_once("<fragment id='901'>").ok_or("start")?;
    let (_, after) = inside.split_once("</fragment>").ok_or("end")?;
    opaque(&format!("{before}{first}{second}{after}"))?;
    Ok(())
}

#[test]
fn pure_source_geometry_retains_signed_coordinates_and_does_not_depend_on_fixed_capture_scale()
-> TestResult {
    for points in [
        vec![[-120., -50.], [-72., -50.], [-24., -50.]],
        vec![[10000., 10000.], [10048., 10000.], [10096., 10000.]],
        vec![[80., 100.], [80.0001, 100.], [80.0002, 100.]],
        vec![[80., 100.], [128., 100.], [176., 100.]],
    ] {
        connected(&geometry_source(&points, &[(0, 1, 1), (1, 2, 1)])?)?;
    }
    Ok(())
}

#[test]
fn box_only_native_and_source_free_capabilities_stay_separate_from_foreign_connected() -> TestResult
{
    let raw = source("water")?;
    let base = connected(&raw)?;
    let mut pure = StoredDocument::default();
    *pure.model_mut() = base.model().clone();
    assert!(pure.has_boxed_atom_labels());
    assert!(!pure.has_connected_atom_labels());
    assert!(!pure.is_fully_mapped());
    let encoded = pure.to_cdxml()?.into_owned();
    let native = stored(std::str::from_utf8(&encoded)?)?;
    assert!(native.has_boxed_atom_labels());
    assert!(!native.has_connected_atom_labels());
    assert!(!native.is_fully_mapped());
    let no_bond = raw
        .replace("<b id='700' B='71' E='207' Order='1'/>", "")
        .replace("<b id='702' B='207' E='13' Order='1'/>", "");
    let boxed = stored(&no_bond)?;
    assert!(boxed.has_boxed_atom_labels());
    assert!(!boxed.has_connected_atom_labels());
    let unlabeled=raw.replace("</fragment>","<n id='99' NodeType='Element' Element='6' Charge='0' Radical='None' NumHydrogens='4' p='250 250'/></fragment>");
    opaque(&unlabeled)?;
    let mut repaired = stored(&unlabeled)?;
    *repaired.model_mut() = base.model().clone();
    assert!(!repaired.has_connected_atom_labels());
    assert!(!repaired.has_boxed_atom_labels());
    assert!(matches!(
        repaired.to_cdxml(),
        Err(bondline_io::DocumentError::UnsupportedEdit)
    ));
    for raw in [
        raw.replace(
            "Original independent connected label vectors",
            concat!("Bondline ", env!("CARGO_PKG_VERSION")),
        ),
        raw.replace(
            " CreationProgram='Original independent connected label vectors'",
            "",
        ),
    ] {
        connected(&raw)?;
    }
    Ok(())
}

#[test]
fn certified_position_changes_preserve_connected_context_and_other_drift_refusals() -> TestResult {
    let raw = source("water")?;
    let base = connected(&raw)?;
    for kind in 0..8 {
        let mut drift = base.clone();
        let page = base.model().pages()[0].id();
        let fragment = base.model().pages()[0].fragments()[0].clone();
        if kind == 0 {
            drift
                .model_mut()
                .set_page_size(page, PageSize::new(513., 512.)?)?;
        } else if kind == 1 {
            drift
                .model_mut()
                .set_drawing_style(base.model().drawing_style().with_margin_width(Some(2.))?);
        } else {
            let mut atoms = fragment.atoms().to_vec();
            let mut bonds = fragment.bonds().to_vec();
            match kind {
                2 => {
                    let atom = atoms.get_mut(1).ok_or("middle")?;
                    atom.position = Some(bondline_model::Point2::new(128., 101.)?);
                    atom.authored_label =
                        Some(Arc::new(bondline_render::position_atom_label(atom)?));
                }
                3 => {
                    let b = bonds.first_mut().ok_or("bond")?;
                    std::mem::swap(&mut b.begin, &mut b.end);
                }
                4 => bonds.first_mut().ok_or("bond")?.order = bondline_model::BondOrder::Double,
                5 => atoms.swap(0, 1),
                6 => atoms.first_mut().ok_or("atom")?.authored_label = None,
                _ => bonds.clear(),
            }
            drift
                .model_mut()
                .replace_fragment(page, Fragment::new(fragment.id(), atoms, bonds)?)?;
        }
        if kind == 2 {
            assert!(drift.has_connected_atom_labels());
            assert!(!drift.has_boxed_atom_labels());
            assert!(!drift.is_fully_mapped());
            assert!(drift.has_display_projection());
            let expected = raw.replace("p='128 100'", "p='128 101'").replace(
                "124.5693359375 95.6171875 131.4306640625 104.3828125",
                "124.5693359375 96.6171875 131.4306640625 105.3828125",
            );
            let encoded = drift.to_cdxml()?;
            assert_eq!(encoded.as_ref(), expected.as_bytes());
            let mut facts = independent(raw.as_bytes())?;
            facts["nodes"][1]["point"] = json!([128.0, 101.0]);
            facts["nodes"][1]["box"] =
                json!([124.5693359375, 96.6171875, 131.4306640625, 105.3828125]);
            assert_eq!(independent(encoded.as_ref())?, facts);
        } else {
            assert!(!drift.has_connected_atom_labels(), "drift{kind}");
            assert!(!drift.has_boxed_atom_labels());
            assert!(!drift.has_display_projection());
            assert!(matches!(
                drift.to_cdxml(),
                Err(bondline_io::DocumentError::UnsupportedEdit)
            ));
        }
        assert_eq!(base.to_cdxml()?.as_ref(), raw.as_bytes());
    }
    Ok(())
}

#[test]
fn mixed_explicit_captions_and_styles_remain_readonly_while_label_values_edit() -> TestResult {
    let caption = "<t id='101' p='72 320' Justification='Left'><s font='42' size='12' face='0' color='3'>read-only caption</s></t>";
    let raw = source("water")?.replace("</page>", &format!("{caption}</page>"));
    let base = connected(&raw)?;
    let changed = edit_middle(&base)?;
    let encoded = changed.to_cdxml()?.into_owned();
    assert!(std::str::from_utf8(&encoded)?.contains(caption));
    assert_eq!(
        stored(std::str::from_utf8(&encoded)?)?.model(),
        changed.model()
    );
    for kind in 0..3 {
        let mut drift = base.clone();
        let page = base.model().pages()[0].id();
        match kind {
            0 => {
                let text = base.model().pages()[0].texts()[0]
                    .with_run_text(0, "changed caption".into())?;
                drift.model_mut().replace_text(page, text)?;
            }
            1 => drift
                .model_mut()
                .set_drawing_settings(bondline_model::DrawingSettings::new(Some(48.), None)?),
            _ => drift
                .model_mut()
                .set_drawing_style(base.model().drawing_style().with_line_width(Some(1.))?),
        }
        assert!(!drift.has_connected_atom_labels());
        assert!(matches!(
            drift.to_cdxml(),
            Err(bondline_io::DocumentError::UnsupportedEdit)
        ));
    }
    Ok(())
}

#[test]
fn explicit_solid_and_black_resource_spelling_remain_exact_through_the_shared_writer() -> TestResult
{
    let raw = source("water")?.replace("<b ", "<b Display='Solid' Display2='Solid' color='0.000' ");
    let base = connected(&raw)?;
    let changed = edit_middle(&base)?;
    let bytes = changed.to_cdxml()?.into_owned();
    assert_eq!(
        std::str::from_utf8(&bytes)?
            .matches("Display='Solid' Display2='Solid' color='0.000'")
            .count(),
        2
    );
    assert_eq!(independent(&bytes)?["formula"], "H3N");
    let reopened = stored(std::str::from_utf8(&bytes)?)?;
    assert_eq!(reopened.model(), changed.model());
    assert!(reopened.has_connected_atom_labels());
    Ok(())
}

#[test]
fn complete_equal_reimport_can_become_exact_native_only_after_actual_codec_certification()
-> TestResult {
    let base = connected(&source("water")?)?;
    let mut own = StoredDocument::default();
    *own.model_mut() = base.model().clone();
    let native = String::from_utf8(own.to_cdxml()?.into_owned())?;
    let lexical = native.replace("Element=\"8\"", "Element=\"8.000\"");
    assert_ne!(lexical, native);
    let source = connected(&lexical)?;
    let edited = edit_middle(&source)?;
    let bytes = edited.to_cdxml()?.into_owned();
    let expected = edit_middle(&own)?.to_cdxml()?.into_owned();
    assert_eq!(
        bytes, expected,
        "the complete native codec, not producer text, proves this transition"
    );
    let reopened = stored(std::str::from_utf8(&bytes)?)?;
    assert_eq!(reopened.model(), edited.model());
    assert!(reopened.has_boxed_atom_labels());
    assert!(!reopened.has_connected_atom_labels());
    assert!(!reopened.is_fully_mapped());
    assert_eq!(source.to_cdxml()?.as_ref(), lexical.as_bytes());
    Ok(())
}

#[test]
fn explicit_zero_margin_is_distinct_from_absence_and_keeps_its_lexeme_through_edits() -> TestResult
{
    let water = source("water")?;
    for value in ["0", "-0.000", "+0.000", "0e-20"] {
        let raw = water.replace("MarginWidth='1.600'", &format!("MarginWidth='{value}'"));
        let base = connected(&raw)?;
        assert_eq!(base.model().drawing_style().margin_width_pt(), Some(0.));
        let changed = edit_middle(&base)?;
        let bytes = changed.to_cdxml()?.into_owned();
        assert!(std::str::from_utf8(&bytes)?.contains(&format!("MarginWidth='{value}'")));
        assert_eq!(
            stored(std::str::from_utf8(&bytes)?)?.model(),
            changed.model()
        );
    }
    opaque(&water.replace(" MarginWidth='1.600'", ""))?;
    Ok(())
}
