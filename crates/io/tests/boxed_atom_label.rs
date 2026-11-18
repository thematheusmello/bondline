//! Frozen existing-API runtime declarations before IO projection/display changes.
use bondline_io::StoredDocument;
use bondline_model::PageSize;
use serde_json::json;
type TestResult = Result<(), Box<dyn std::error::Error>>;
const SOURCE: &[u8] = br#"<CDXML CreationProgram="Original box-label oracle"><fonttable><font id="42" name="Roboto"/></fonttable><colortable><color r="1" g="1" b="1"/><color r="0" g="0" b="0"/><color r="0.2" g="0.4" b="0.8"/></colortable><page id="90" DrawingSpace="Poster" Width="512" Height="512"><fragment id="51"><n id="73" NodeType="Element" Element="8" Charge="0" Radical="None" NumHydrogens="1" p="84.1220703125 95.734375"><t id="21" BoundingBox="80.69140625 91.3515625 95.796875 100.1171875" Justification="Left" LabelJustification="Left"><s font="42" size="12" face="0" color="4">OH</s></t></n></fragment></page></CDXML>"#;

#[test]
fn explicit_box_is_owned_and_text_point_absence_stays_absent() -> TestResult {
    let stored = StoredDocument::from_cdxml(SOURCE.into(), PageSize::A4)?;
    let model = serde_json::to_value(stored.model())?;
    assert_eq!(
        model["pages"][0]["fragments"][0]["atoms"][0]["authored_label"],
        json!({
            "position":null,"bounding_box":[80.69140625,91.3515625,95.796875,100.1171875],
            "justification":"left","label_justification":"left","label_alignment":null,
            "runs":[{"text":"OH","style":{"font_family":"Roboto","size_pt":12.0,"face":0,"rgb":[0.2,0.4,0.8]}}]
        })
    );
    assert!(!stored.is_fully_mapped());
    assert_eq!(stored.to_cdxml()?.as_ref(), SOURCE);
    Ok(())
}

#[test]
fn explicit_agreeing_boxed_label_has_a_real_current_owner_scene_without_full_authority()
-> TestResult {
    let stored = StoredDocument::from_cdxml(SOURCE.into(), PageSize::A4)?;
    assert!(!stored.is_fully_mapped());
    assert!(
        stored.has_display_projection(),
        "complete boxed label must support actual display"
    );
    let page = stored.model().pages().first().ok_or("page")?;
    let scene = bondline_io::display::drawing_page_scene(
        1024,
        1024,
        &stored,
        page.id(),
        &Default::default(),
    )?;
    assert!(scene.label_em_px().is_some());
    assert_eq!(stored.to_cdxml()?.as_ref(), SOURCE);
    Ok(())
}

use bondline_model::{Element, Fragment};
use serde_json::Value;
use std::{
    borrow::Cow,
    io::Write,
    process::{Command, Stdio},
    sync::Arc,
};
type Out<T> = Result<T, Box<dyn std::error::Error>>;

fn stored(raw: &[u8]) -> Out<StoredDocument> {
    Ok(StoredDocument::from_cdxml(raw.into(), PageSize::A4)?)
}
fn replace_label(base: &StoredDocument, element: u8, hydrogens: u16) -> Out<StoredDocument> {
    let page = base.model().pages().first().ok_or("page")?;
    let fragment = page.fragments().first().ok_or("fragment")?;
    let mut atoms = fragment.atoms().to_vec();
    let atom = atoms.first_mut().ok_or("atom")?;
    atom.element = Element::try_from(element)?;
    atom.explicit_hydrogens = Some(hydrogens);
    atom.authored_label = Some(Arc::new(bondline_render::position_atom_label(atom)?));
    let mut result = base.clone();
    result.model_mut().replace_fragment(
        page.id(),
        Fragment::new(fragment.id(), atoms, fragment.bonds().to_vec())?,
    )?;
    Ok(result)
}
fn fixture() -> Out<Value> {
    Ok(serde_json::from_str(include_str!(
        "../../render/tests/fixtures/boxed-atom-font-vectors.json"
    ))?)
}
fn independent(raw: &[u8]) -> Out<Value> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut process = Command::new(python).args(["-X","utf8","-B","-c",r#"
import json,sys,xml.parsers.expat as expat
from decimal import Decimal
raw=sys.stdin.buffer.read(1048577);assert 0<len(raw)<=1048576
nodes=[];current=None;label=None;run=False;fonts={};colours=[]
def start(name,attrs):
 global current,label,run
 if name=='font':fonts[int(attrs['id'])]=attrs['name']
 if name=='color':colours.append([float(Decimal(attrs[c])) for c in ('r','g','b')])
 if name=='n':
  current={'id':int(attrs['id']),'element':int(Decimal(attrs['Element'])),'h':int(Decimal(attrs['NumHydrogens'])),'p':[float(Decimal(x)) for x in attrs['p'].split()]};nodes.append(current)
 if name=='t' and current is not None:
  assert 'p' not in attrs and 'LabelAlignment' not in attrs
  assert attrs['Justification']==attrs['LabelJustification']=='Left'
  label={'id':int(attrs['id']),'box':[float(Decimal(x)) for x in attrs['BoundingBox'].split()],'text':''};current['label']=label
 if name=='s' and label is not None:
  run=True;label.update(font=fonts[int(attrs['font'])],size=float(Decimal(attrs['size'])),face=int(Decimal(attrs['face'])))
  color=int(Decimal(attrs['color']));label['rgb']=[0.0]*3 if color==0 else [1.0]*3 if color==1 else colours[color-2]
def end(name):
 global run,label,current
 if name=='s':run=False
 if name=='t':label=None
 if name=='n':current=None
def chars(text):
 if run:label['text']+=text
p=expat.ParserCreate(namespace_separator='|');p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
p.ExternalEntityRefHandler=lambda *unused:0
p.StartElementHandler=start;p.EndElementHandler=end;p.CharacterDataHandler=chars;p.Parse(raw,True)
print(json.dumps(nodes,separators=(',',':')))
"#]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    process.stdin.take().ok_or("Expat stdin")?.write_all(raw)?;
    let out = process.wait_with_output()?;
    if !out.status.success() {
        return Err(format!(
            "independent box consumer: {}",
            String::from_utf8_lossy(&out.stderr)
        )
        .into());
    }
    Ok(serde_json::from_slice(&out.stdout)?)
}

#[test]
fn original_fonttools_matrix_owns_boxes_and_preserves_the_complete_source_without_full_authority()
-> TestResult {
    let data = fixture()?;
    assert_eq!(data["fonttools"], "4.61.1");
    assert_eq!(
        data["font_sha256"],
        "56a45233d29f11b4dfb86d248e921939d115778f87325e7ae8cc108383d6664d"
    );
    let cases = data["cases"].as_array().ok_or("cases")?;
    assert_eq!(cases.len(), 156);
    for case in cases {
        let raw = original_source(case)?;
        let own = stored(raw.as_bytes())?;
        assert!(own.has_boxed_atom_labels(), "{}", case["name"]);
        assert!(own.has_display_projection());
        assert!(!own.is_fully_mapped());
        assert!(!own.has_caption_projection());
        assert_eq!(own.to_cdxml()?.as_ref(), raw.as_bytes());
        let model = serde_json::to_value(own.model())?;
        let atom = &model["pages"][0]["fragments"][0]["atoms"][0];
        assert_eq!(atom["authored_label"]["bounding_box"], case["bounding_box"]);
        assert_eq!(atom["authored_label"]["runs"][0]["text"], case["text"]);
        assert_eq!(
            atom["position"],
            json!({"x_pt":case["node_point"][0],"y_pt":case["node_point"][1]})
        );
    }
    Ok(())
}
#[test]
fn independently_consumed_delta_patches_only_element_h_text_and_box_and_keeps_source_ids()
-> TestResult {
    let base = stored(SOURCE)?;
    let expected = json!([{"id":73,"element":8,"h":1,"p":[84.1220703125,95.734375],"label":{"id":21,"box":[80.69140625,91.3515625,95.796875,100.1171875],"text":"OH","font":"Roboto","size":12.0,"face":0,"rgb":[0.2,0.4,0.8]}}]);
    assert_eq!(independent(SOURCE)?, expected);
    let bad = std::str::from_utf8(SOURCE)?.replace("font=\"42\"", "font=\"43\"");
    assert!(
        independent(bad.as_bytes()).is_err(),
        "qualify independent font lookup"
    );
    let current = replace_label(&base, 7, 2)?;
    assert!(current.has_boxed_atom_labels());
    assert!(!current.is_fully_mapped());
    let encoded = current.to_cdxml()?.into_owned();
    let data = fixture()?;
    let vector = data["cases"]
        .as_array()
        .ok_or("cases")?
        .iter()
        .find(|v| v["name"] == "NH2-12pt-80-100")
        .ok_or("NH2 vector")?;
    let dx = 84.1220703125 - vector["node_point"][0].as_f64().ok_or("x")?;
    let expected_box: Vec<_> = vector["bounding_box"]
        .as_array()
        .ok_or("box")?
        .iter()
        .enumerate()
        .map(|(i, v)| {
            v.as_f64()
                .map(|x| x + if i % 2 == 0 { dx } else { 0.0 })
                .ok_or("number")
        })
        .collect::<Result<_, _>>()?;
    let mut expected = expected;
    expected[0]["element"] = json!(7);
    expected[0]["h"] = json!(2);
    expected[0]["label"]["text"] = json!("NH2");
    expected[0]["label"]["box"] = json!(expected_box);
    assert_eq!(independent(&encoded)?, expected);
    let box_string = expected_box
        .iter()
        .map(f64::to_string)
        .collect::<Vec<_>>()
        .join(" ");
    let literal = std::str::from_utf8(SOURCE)?
        .replace("Element=\"8\"", "Element=\"7\"")
        .replace("NumHydrogens=\"1\"", "NumHydrogens=\"2\"")
        .replace(">OH</s>", ">NH2</s>")
        .replace("80.69140625 91.3515625 95.796875 100.1171875", &box_string);
    assert_eq!(
        encoded,
        literal.as_bytes(),
        "all bytes outside the four owned fields remain exact"
    );
    let reopened = stored(&encoded)?;
    assert_eq!(reopened.model(), current.model());
    assert!(reopened.has_boxed_atom_labels());
    assert_eq!(base.to_cdxml()?.as_ref(), SOURCE);
    Ok(())
}
#[test]
fn qualified_noop_keeps_entities_quotes_numeric_spellings_and_borrowed_source() -> TestResult {
    let raw = std::str::from_utf8(SOURCE)?
        .replace("Element=\"8\"", "Element='8.000'")
        .replace("NumHydrogens=\"1\"", "NumHydrogens='+001'")
        .replace(">OH</s>", ">&#79;H</s>")
        .replace(
            "80.69140625 91.3515625 95.796875 100.1171875",
            "80.69140625000 91.3515625000 95.796875000 100.117187500",
        );
    let own = stored(raw.as_bytes())?;
    assert!(own.has_boxed_atom_labels());
    let mut same = own.clone();
    let snapshot = same.model_snapshot();
    let value = same.model().clone();
    *same.model_mut() = value;
    assert!(matches!(same.to_cdxml()?, Cow::Borrowed(_)));
    assert_eq!(same.to_cdxml()?.as_ref(), raw.as_bytes());
    assert_eq!(own.model(), same.model());
    assert!(Arc::ptr_eq(&snapshot, &own.model_snapshot()));
    Ok(())
}
#[test]
fn unknown_or_inconsistent_source_never_acquires_the_boxed_capability_or_rewrite_permission()
-> TestResult {
    let original = std::str::from_utf8(SOURCE)?;
    let variants = [
        original.replace("<CDXML ", "<CDXML Future='retain' "),
        original.replace("<page ", "<page Future='retain' "),
        original.replace("<fragment ", "<fragment Future='retain' "),
        original.replace("<n ", "<n Future='retain' "),
        original.replace("<t ", "<t Future='retain' "),
        original.replace("<s ", "<s Future='retain' "),
        original.replace("<font ", "<font charset='UTF-8' "),
        original.replace("<color ", "<color Future='retain' "),
        original.replace("<CDXML ", "<CDXML xmlns:unused='urn:unused' "),
        original.replace("<t ", "<t xmlns='' "),
        original.replace(
            "BoundingBox=\"80.69140625 91.3515625 95.796875 100.1171875\"",
            "BoundingBox='80 90 96 101'",
        ),
        original.replace(">OH</s>", ">NH2</s>"),
        original.replace("face=\"0\"", "face='96'"),
        original.replace("name=\"Roboto\"", "name='Other'"),
        original.replace("<t ", "<t p='80 100' "),
        original.replace("<t ", "<t LabelAlignment='Auto' "),
        original.replace(
            "Justification=\"Left\" LabelJustification=\"Left\"",
            "Justification='Left'",
        ),
        original.replace("<n ", "<n ShowTerminalCarbonLabels='yes' "),
        original.replace("id=\"21\"", "id='0'"),
        original.replace("<s font=\"42\"", "<s font='43'"),
        original.replace(
            "<color r=\"1\" g=\"1\" b=\"1\"/>",
            "<color r='0.9' g='1' b='1'/>",
        ),
        original.replace(">OH</s>", "><![CDATA[OH]]></s>"),
        format!("<?future keep?>{original}"),
        format!("<!DOCTYPE CDXML>{original}"),
    ];
    for raw in variants {
        let mut own = stored(raw.as_bytes())?;
        assert!(!own.has_boxed_atom_labels(), "must remain opaque: {raw}");
        assert!(!own.has_display_projection());
        assert_eq!(own.to_cdxml()?.as_ref(), raw.as_bytes());
        let page = own.model().pages()[0].id();
        own.model_mut()
            .set_page_size(page, PageSize::new(513.0, 512.0)?)?;
        assert!(matches!(
            own.to_cdxml(),
            Err(bondline_io::DocumentError::UnsupportedEdit)
        ));
    }
    Ok(())
}
#[test]
fn complete_boxed_proof_allows_arbitrary_or_absent_producer_but_not_legacy_profile_widening()
-> TestResult {
    let original = std::str::from_utf8(SOURCE)?;
    for raw in [
        original.replace("Original box-label oracle", "Unrecognized producer"),
        original.replace(" CreationProgram=\"Original box-label oracle\"", ""),
    ] {
        let own = stored(raw.as_bytes())?;
        assert!(own.has_boxed_atom_labels());
        assert!(!own.is_fully_mapped());
        assert_eq!(own.to_cdxml()?.as_ref(), raw.as_bytes());
        assert!(replace_label(&own, 7, 2)?.to_cdxml().is_ok());
    }
    Ok(())
}
#[test]
fn pure_native_data_requires_a_qualified_box_and_delta_preserves_all_other_fields() -> TestResult {
    let base = stored(SOURCE)?;
    let mut pure = StoredDocument::default();
    *pure.model_mut() = base.model().clone();
    assert!(pure.has_boxed_atom_labels());
    let bytes = pure.to_cdxml()?.into_owned();
    let native = stored(&bytes)?;
    assert!(native.has_boxed_atom_labels());
    assert!(!native.is_fully_mapped());
    assert_eq!(native.model(), pure.model());
    let mut changed = native.clone();
    let page = changed.model().pages()[0].id();
    changed
        .model_mut()
        .set_page_size(page, PageSize::new(513.0, 512.0)?)?;
    assert!(!changed.has_boxed_atom_labels());
    let mut malformed = pure.clone();
    let fragment = malformed.model().pages()[0].fragments()[0].clone();
    let mut atoms = fragment.atoms().to_vec();
    let a = atoms.first_mut().ok_or("atom")?;
    a.authored_label = Some(Arc::new(
        a.authored_label
            .as_ref()
            .ok_or("label")?
            .with_bounding_box(None)?,
    ));
    malformed
        .model_mut()
        .replace_fragment(page, Fragment::new(fragment.id(), atoms, vec![])?)?;
    assert!(!malformed.has_boxed_atom_labels());
    assert!(!malformed.has_display_projection());
    Ok(())
}
#[test]
fn mixed_captions_native_graph_and_root_settings_are_complete_but_stay_readonly_in_the_delta()
-> TestResult {
    let raw=std::str::from_utf8(SOURCE)?.replace("<CDXML ","<CDXML BondLength='48.000' ChainAngle='120' LineWidth='1.00' BondSpacing='12.000' ")
        .replace("</fragment></page>","<n id='99' NodeType='Element' Element='6' Charge='0' Radical='None' NumHydrogens='4' p='200 200'/></fragment><t id='101' p='72 180' Justification='Left'><s font='42' size='12' face='0' color='3'>caption</s></t></page>");
    let own = stored(raw.as_bytes())?;
    assert!(own.has_boxed_atom_labels());
    let changed = replace_label(&own, 7, 2)?;
    let bytes = changed.to_cdxml()?.into_owned();
    assert_eq!(stored(&bytes)?.model(), changed.model());
    for kind in 0..4 {
        let mut drift = own.clone();
        let page = drift.model().pages()[0].id();
        match kind {
            0 => drift.model_mut().set_page_size(page, PageSize::A4)?,
            1 => drift
                .model_mut()
                .set_drawing_settings(bondline_model::DrawingSettings::default()),
            2 => drift
                .model_mut()
                .set_drawing_style(bondline_model::DrawingStyle::default()),
            _ => {
                let text =
                    drift.model().pages()[0].texts()[0].with_run_text(0, "changed".into())?;
                drift.model_mut().replace_text(page, text)?;
            }
        }
        assert!(!drift.has_boxed_atom_labels(), "drift{kind}");
        assert!(matches!(
            drift.to_cdxml(),
            Err(bondline_io::DocumentError::UnsupportedEdit)
        ));
    }
    Ok(())
}

#[test]
fn valid_geometry_does_not_authorize_position_style_or_unlabeled_atom_drift() -> TestResult {
    let raw=std::str::from_utf8(SOURCE)?.replace("</n></fragment>","</n><n id='99' NodeType='Element' Element='6' Charge='0' Radical='None' NumHydrogens='4' p='200 200'/></fragment>");
    let base = stored(raw.as_bytes())?;
    assert!(base.has_boxed_atom_labels());
    for kind in 0..3 {
        let mut drift = base.clone();
        let page = drift.model().pages()[0].id();
        let fragment = drift.model().pages()[0].fragments()[0].clone();
        let mut atoms = fragment.atoms().to_vec();
        if kind == 2 {
            atoms.get_mut(1).ok_or("unlabeled")?.element = Element::try_from(7)?;
        } else {
            let atom = atoms.first_mut().ok_or("atom")?;
            if kind == 0 {
                let p = atom.position.ok_or("p")?;
                atom.position = Some(bondline_model::Point2::new(
                    p.x_pt() + 16.0,
                    p.y_pt() + 16.0,
                )?);
            } else {
                let old = atom.authored_label.as_ref().ok_or("label")?;
                let run = old.runs().first().ok_or("run")?;
                let style =
                    bondline_model::TextStyle::new("Roboto".into(), 24.0, 0, [0.8, 0.1, 0.3])?;
                let label = bondline_model::AtomLabel::new(
                    None,
                    old.justification(),
                    old.label_justification(),
                    None,
                    vec![bondline_model::TextRun::new(run.text().into(), style)?],
                )?;
                atom.authored_label = Some(Arc::new(label));
            }
            atom.authored_label = Some(Arc::new(bondline_render::position_atom_label(atom)?));
            bondline_render::validate_atom_label(atom)?;
        }
        drift
            .model_mut()
            .replace_fragment(page, Fragment::new(fragment.id(), atoms, vec![])?)?;
        assert!(
            !drift.has_boxed_atom_labels(),
            "forbidden drift{kind} with otherwise qualified geometry"
        );
        assert!(matches!(
            drift.to_cdxml(),
            Err(bondline_io::DocumentError::UnsupportedEdit)
        ));
        assert_eq!(base.to_cdxml()?.as_ref(), raw.as_bytes());
    }
    Ok(())
}

#[test]
fn combined_render_work_bound_revokes_display_capability_without_truncating_model_data()
-> TestResult {
    let base = stored(SOURCE)?;
    let prototype = base.model().pages()[0].fragments()[0].atoms()[0].clone();
    let atoms = (1..=2049)
        .map(|id| {
            let mut a = prototype.clone();
            a.id = bondline_model::AtomId::new(id).ok_or("id")?;
            Ok(a)
        })
        .collect::<Out<Vec<_>>>()?;
    let mut pure = StoredDocument::default();
    let page = pure.model().pages()[0].id();
    pure.model_mut().append_fragments(
        page,
        vec![Fragment::new(
            bondline_model::FragmentId::new(1).ok_or("fragment")?,
            atoms,
            vec![],
        )?],
    )?;
    assert!(pure.model().has_authored_atom_labels());
    assert_eq!(pure.model().pages()[0].fragments()[0].atoms().len(), 2049);
    assert!(
        !pure.has_boxed_atom_labels(),
        "4098 label glyphs exceed the existing4096 shared render-work bound"
    );
    assert!(!pure.has_display_projection());
    Ok(())
}

#[test]
fn source_scope_and_malformed_box_guards_keep_original_bytes_without_promoting_coverage()
-> TestResult {
    let original = std::str::from_utf8(SOURCE)?;
    let mut cases = vec![
        original.replace("id=\"90\"", "id='0'"),
        original.replace("id=\"51\"", "id='0'"),
        original.replace("id=\"73\"", "id='0'"),
        original
            .replace("<fragment ", "<group id='89'><fragment ")
            .replace("</fragment>", "</fragment></group>"),
        original.replace("</page>", "<graphic id='89'/></page>"),
        original.replace("<n ", "<n x:Future='1' xmlns:x='urn:future' "),
        original.replace("<color r=\"0\"", "<color r='-1e-9999'"),
    ];
    for raw_box in [
        "",
        "80 90 96",
        "80 90 96 101 102",
        "NaN 90 96 101",
        "80 90 inf 101",
        "100 90 96 101",
        "80 102 96 101",
        "80 90 80 101",
        "80 90 96 90",
        "100001 90 100002 101",
    ] {
        cases.push(original.replace("80.69140625 91.3515625 95.796875 100.1171875", raw_box));
    }
    for raw in cases {
        let own = stored(raw.as_bytes())?;
        assert!(!own.has_boxed_atom_labels(), "unsupported source: {raw}");
        assert!(!own.has_display_projection());
        assert_eq!(own.to_cdxml()?.as_ref(), raw.as_bytes());
    }
    let duplicate = original.replace("</n></fragment>", "</n><n id='73' NodeType='Element' Element='6' Charge='0' Radical='None' p='200 200'/></fragment>");
    assert!(
        StoredDocument::from_cdxml(duplicate.into_bytes().into(), PageSize::A4).is_err(),
        "the pre-existing same-scope duplicate-ID import refusal stays a hard error"
    );
    let two_pages = original.replace(
        "</CDXML>",
        "<page id='91' DrawingSpace='Poster' Width='512' Height='512'/></CDXML>",
    );
    assert!(
        matches!(
            StoredDocument::from_cdxml(two_pages.into_bytes().into(), PageSize::A4),
            Err(bondline_io::DocumentError::Mapping(
                bondline_io::cdxml::MappingError::UnsupportedPageLayout
            ))
        ),
        "the pre-existing multiple-Page projection refusal stays a hard error"
    );
    Ok(())
}

#[test]
fn multiple_labels_batch_preserves_local_text_ids_and_directed_bonds_and_refuses_structure_drift()
-> TestResult {
    let second = r#"<n id='97' NodeType='Element' Element='8' Charge='0' Radical='None' NumHydrogens='1' p='184.1220703125 95.734375'><t id='21' BoundingBox='180.69140625 91.3515625 195.796875 100.1171875' Justification='Left' LabelJustification='Left'><s font='42' size='12' face='0' color='4'>OH</s></t></n><b id='900' B='97' E='73' Order='1'/>"#;
    let raw = std::str::from_utf8(SOURCE)?.replace("</fragment>", &format!("{second}</fragment>"));
    let base = stored(raw.as_bytes())?;
    assert!(
        base.has_boxed_atom_labels(),
        "the same child Text ID is local to two distinct Node scopes"
    );
    let page = base.model().pages()[0].id();
    let fragment = base.model().pages()[0].fragments()[0].clone();
    let mut atoms = fragment.atoms().to_vec();
    for atom in &mut atoms {
        atom.element = Element::try_from(7)?;
        atom.explicit_hydrogens = Some(2);
        atom.authored_label = Some(Arc::new(bondline_render::position_atom_label(atom)?));
    }
    let mut edited = base.clone();
    edited.model_mut().replace_fragment(
        page,
        Fragment::new(fragment.id(), atoms, fragment.bonds().to_vec())?,
    )?;
    assert!(edited.has_boxed_atom_labels());
    let encoded = edited.to_cdxml()?.into_owned();
    let facts = independent(&encoded)?;
    for (i, id) in [73, 97].into_iter().enumerate() {
        assert_eq!(facts[i]["id"], id);
        assert_eq!(facts[i]["element"], 7);
        assert_eq!(facts[i]["h"], 2);
        assert_eq!(facts[i]["label"]["id"], 21);
        assert_eq!(facts[i]["label"]["text"], "NH2");
    }
    assert!(std::str::from_utf8(&encoded)?.contains("<b id='900' B='97' E='73' Order='1'/>"));
    assert_eq!(stored(&encoded)?.model(), edited.model());
    assert_eq!(base.to_cdxml()?.as_ref(), raw.as_bytes());
    for kind in 0..8 {
        let mut atoms = fragment.atoms().to_vec();
        let mut bonds = fragment.bonds().to_vec();
        let mut id = fragment.id();
        match kind {
            0 => atoms.swap(0, 1),
            1 => id = bondline_model::FragmentId::new(17).ok_or("fragment ID")?,
            2 => {
                let bond = bonds.first_mut().ok_or("bond")?;
                std::mem::swap(&mut bond.begin, &mut bond.end);
            }
            3 => bonds.first_mut().ok_or("bond")?.order = bondline_model::BondOrder::Double,
            4 => {
                bonds.first_mut().ok_or("bond")?.id =
                    bondline_model::BondId::new(17).ok_or("bond ID")?
            }
            5 => bonds.clear(),
            6 => atoms.first_mut().ok_or("atom")?.authored_label = None,
            _ => atoms.first_mut().ok_or("atom")?.isotope = std::num::NonZeroU16::new(18),
        }
        let replacement = Fragment::new(id, atoms, bonds)?;
        let mut drift = base.clone();
        if id == fragment.id() {
            drift.model_mut().replace_fragment(page, replacement)?;
        } else {
            *drift.model_mut() = bondline_model::Document::default();
            drift
                .model_mut()
                .set_page_size(page, PageSize::new(512.0, 512.0)?)?;
            drift
                .model_mut()
                .append_fragments(page, vec![replacement])?;
        }
        assert!(
            !drift.has_boxed_atom_labels(),
            "forbidden structure drift {kind}"
        );
        assert!(matches!(
            drift.to_cdxml(),
            Err(bondline_io::DocumentError::UnsupportedEdit)
        ));
    }
    Ok(())
}

fn original_source(case: &Value) -> Out<String> {
    let point = case["node_point"]
        .as_array()
        .ok_or("node point")?
        .iter()
        .map(|v| v.as_f64().map(|n| n.to_string()).ok_or("point number"))
        .collect::<Result<Vec<_>, _>>()?
        .join(" ");
    let bounds = case["bounding_box"]
        .as_array()
        .ok_or("box")?
        .iter()
        .map(|v| v.as_f64().map(|n| n.to_string()).ok_or("box number"))
        .collect::<Result<Vec<_>, _>>()?
        .join(" ");
    let element = case["element"].as_u64().ok_or("element")?;
    let h = case["explicit_hydrogens"].as_u64().ok_or("hydrogens")?;
    let size = case["size_pt"].as_f64().ok_or("size")?;
    let body = case["text"].as_str().ok_or("text")?;
    Ok(format!(
        r#"<CDXML CreationProgram="Original box-label oracle"><fonttable><font id="42" name="Roboto"/></fonttable><colortable><color r="1" g="1" b="1"/><color r="0" g="0" b="0"/><color r="0.2" g="0.4" b="0.8"/></colortable><page id="90" DrawingSpace="Poster" Width="512" Height="512"><fragment id="51"><n id="73" NodeType="Element" Element="{element}" Charge="0" Radical="None" NumHydrogens="{h}" p="{point}"><t id="21" BoundingBox="{bounds}" Justification="Left" LabelJustification="Left"><s font="42" size="{size}" face="0" color="4">{body}</s></t></n></fragment></page></CDXML>"#
    ))
}
