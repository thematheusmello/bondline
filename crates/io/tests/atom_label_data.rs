//! Independent declaration before the producer exists; only established public APIs.
use bondline_io::cdxml::ElementalGraph;
use serde_json::json;

type TestResult = Result<(), Box<dyn std::error::Error>>;
const SOURCE: &[u8] = br#"<CDXML CreationProgram='Original independent test'><fonttable><font id='42' name='Roboto'/></fonttable><colortable><color r='1' g='1' b='1'/><color r='0' g='0' b='0'/><color r='0.2' g='0.4' b='0.8'/></colortable><page id='90' DrawingSpace='Poster' Width='256' Height='256'><fragment id='51'><n id='73' Element='8' NumHydrogens='1' p='72 96'><t id='21' p='71.5 96.25' Justification='Left' LabelJustification='Auto' LabelAlignment='Right'><s font='42' size='12.5' face='96' color='4'>OH</s></t></n></fragment></page></CDXML>"#;

#[test]
fn explicit_node_child_text_is_owned_independently_of_numeric_element_and_h() -> TestResult {
    let graph = ElementalGraph::parse(SOURCE)?;
    assert_eq!(
        graph.to_bytes(),
        SOURCE,
        "unchanged public source remains authoritative"
    );
    let value = serde_json::to_value(graph.fragments())?;
    assert_eq!(value[0]["atoms"][0]["element"], 8);
    assert_eq!(value[0]["atoms"][0]["explicit_hydrogens"], 1);
    assert_eq!(
        value[0]["atoms"][0]["authored_label"],
        json!({
            "position":{"x_pt":71.5,"y_pt":96.25},
            "justification":"left","label_justification":"auto","label_alignment":"right",
            "runs":[{"text":"OH","style":{"font_family":"Roboto","size_pt":12.5,"face":96,"rgb":[0.2,0.4,0.8]}}]
        })
    );
    Ok(())
}
// Append after the first independently declared existing-API runtime RED.
use bondline_io::{
    StoredDocument,
    cdxml::write_document,
    mol::{self, MolTransform, Version, v3000},
    smiles::{RecordSelection, SmilesFileDocument},
};
use bondline_model::{Atom, Document, Fragment, FragmentId, PageId, PageSize, Point2};
use bondline_render::{
    Cancellation, Error as RenderError, PageView, graph_page_scene, graph_page_scene_with_cancel,
    graph_page_scene_with_display_hydrogens,
};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    io::Write,
    process::{Command, Stdio},
    sync::Arc,
};

type Out<T> = Result<T, Box<dyn std::error::Error>>;
fn label_value() -> Value {
    json!({"position":{"x_pt":71.5,"y_pt":96.25},"justification":"left","label_justification":"auto","label_alignment":"right",
        "runs":[{"text":"OH","style":{"font_family":"Roboto","size_pt":12.5,"face":96,"rgb":[0.2,0.4,0.8]}}]})
}
fn model_with_label(label: Value) -> Out<Document> {
    let atom: Atom = serde_json::from_value(
        json!({"id":1,"element":8,"position":{"x_pt":72.0,"y_pt":96.0},
        "isotope":null,"charge":0,"radical":"none","explicit_hydrogens":1,"authored_label":label}),
    )?;
    let mut doc = Document::default();
    doc.append_fragments(
        PageId::new(1).ok_or("page")?,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            vec![atom],
            vec![],
        )?],
    )?;
    Ok(doc)
}
fn source_label(raw: &[u8]) -> Out<Value> {
    let stored = StoredDocument::from_cdxml(raw.into(), PageSize::A4)?;
    assert_eq!(stored.to_cdxml()?.as_ref(), raw);
    assert!(!stored.is_fully_mapped());
    let value = serde_json::to_value(stored.model())?;
    Ok(value["pages"][0]["fragments"][0]["atoms"][0]["authored_label"].clone())
}
fn independently_read(raw: &[u8]) -> Out<Value> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(python)
        .args([
            "-X",
            "utf8",
            "-B",
            "-c",
            include_str!("atom_label_xml_oracle.py"),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("oracle stdin")?.write_all(raw)?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(format!(
            "independent XML consumer: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}
fn expected_facts() -> Value {
    json!([{"parent":"n","position":[71.5,96.25],"justification":"Left","label_justification":"Auto","label_alignment":"Right",
        "runs":[{"text":"OH","font_family":"Roboto","size_pt":12.5,"face":96,"rgb":[0.2,0.4,0.8]}]}])
}

#[test]
fn independently_checked_native_data_encoding_preserves_child_label_and_refuses_authoring()
-> TestResult {
    assert_eq!(independently_read(SOURCE)?, expected_facts());
    let doc = model_with_label(label_value())?;
    let mut own = StoredDocument::default();
    *own.model_mut() = doc.clone();
    assert!(
        !own.is_fully_mapped(),
        "source-free data does not grant unsupported label authority"
    );
    assert!(!own.has_display_projection());
    let encoded = write_document(&doc)?;
    assert_eq!(independently_read(&encoded)?, expected_facts());
    let reopened = StoredDocument::from_cdxml(encoded.clone().into(), PageSize::A4)?;
    assert_eq!(reopened.model(), &doc);
    assert!(
        !reopened.is_fully_mapped(),
        "exact native bytes still lack label layout certification"
    );
    assert!(!reopened.has_display_projection());
    assert_eq!(reopened.to_cdxml()?.as_ref(), encoded);
    assert_eq!(own.to_cdxml()?.as_ref(), encoded);
    let good = String::from_utf8(encoded)?;
    let bad = good.replacen("font=\"0\"", "font=\"65535\"", 1);
    assert_ne!(bad, good, "qualify exact native font-reference mutation");
    assert!(independently_read(bad.as_bytes()).is_err());
    Ok(())
}

#[test]
fn optional_placement_and_every_distinct_named_enum_are_data_without_inference() -> TestResult {
    let original = std::str::from_utf8(SOURCE)?;
    let all = original
        .replace(" p='71.5 96.25'", "")
        .replace(" Justification='Left'", "")
        .replace(" LabelJustification='Auto'", "")
        .replace(" LabelAlignment='Right'", "");
    let mut expected = label_value();
    for key in [
        "position",
        "justification",
        "label_justification",
        "label_alignment",
    ] {
        expected[key] = Value::Null;
    }
    assert_eq!(source_label(all.as_bytes())?, expected);
    for (property, key, names) in [
        (
            "Justification",
            "justification",
            vec![
                "Right", "Left", "Center", "Full", "Above", "Below", "Auto", "Best",
            ],
        ),
        (
            "LabelJustification",
            "label_justification",
            vec![
                "Right", "Left", "Center", "Full", "Above", "Below", "Auto", "Best",
            ],
        ),
        (
            "LabelAlignment",
            "label_alignment",
            vec!["Auto", "Left", "Center", "Right", "Above", "Below", "Best"],
        ),
    ] {
        for name in names {
            let raw = all.replacen("<t id='21'", &format!("<t id='21' {property}='{name}'"), 1);
            let mut want = expected.clone();
            want[key] = json!(name.to_ascii_lowercase());
            assert_eq!(source_label(raw.as_bytes())?, want, "{property}={name}");
            let doc = model_with_label(want)?;
            let native = write_document(&doc)?;
            let reread = StoredDocument::from_cdxml(native.clone().into(), PageSize::A4)?;
            assert_eq!(reread.model(), &doc);
            let independent = independently_read(&native)?;
            assert_eq!(independent[0][key], name);
            for other in ["justification", "label_justification", "label_alignment"] {
                if other != key {
                    assert_eq!(independent[0][other], Value::Null);
                }
            }
            assert_eq!(independent[0]["position"], Value::Null);
        }
    }
    Ok(())
}

#[test]
fn unresolved_or_ambiguous_child_text_stays_opaque_and_preserves_every_source_byte() -> TestResult {
    let original = std::str::from_utf8(SOURCE)?;
    let mut variants = vec![
        original.replace("font='42'", "font='43'"),
        original.replace("font='42' ", ""),
        original.replace("size='12.5' ", ""),
        original.replace("face='96' ", ""),
        original.replace("color='4'", ""),
        original.replace("color='4'", "color='65000'"),
        original.replace("Justification='Left'", "Justification='Unknown'"),
        original.replace("LabelJustification='Auto'", "LabelJustification='Unknown'"),
        original.replace("LabelAlignment='Right'", "LabelAlignment='Full'"),
        original.replace("p='71.5 96.25'", "p='71.5 96.25 8'"),
        original.replace("p='71.5 96.25'", "p='NaN 96.25'"),
        original.replace(
            "</t>",
            "</t><t id='22'><s font='42' size='12.5' face='96' color='4'>other</s></t>",
        ),
        original
            .replace("<t id='21'", "<x:t xmlns:x='urn:foreign' id='21'")
            .replace("</t>", "</x:t>"),
    ];
    for raw in variants.drain(..) {
        assert_eq!(
            source_label(raw.as_bytes())?,
            Value::Null,
            "unsupported child text must not become an owned label: {raw}"
        );
    }
    // Known projection may coexist with unknown raw data, but never supplies complete-source authority.
    let unknown = original.replace("<t id='21'", "<t id='21' Future='keep &amp; opaque'");
    assert_eq!(source_label(unknown.as_bytes())?, label_value());
    let empty = original.replace(">OH</s>", "></s>");
    let mut expected = label_value();
    expected["runs"][0]["text"] = json!("");
    assert_eq!(
        source_label(empty.as_bytes())?,
        expected,
        "present empty label is not absent"
    );
    Ok(())
}

#[test]
fn direct_fragment_writers_and_every_graph_scene_entry_refuse_even_empty_authored_labels()
-> TestResult {
    for body in ["OH", ""] {
        let mut label = label_value();
        label["runs"][0]["text"] = json!(body);
        let doc = model_with_label(label)?;
        let page = doc.pages().first().ok_or("page")?;
        let fragment = page.fragments().first().ok_or("fragment")?;
        let frame = MolTransform::new(1.0, Point2::new(0.0, 0.0)?)?;
        assert_eq!(
            mol::write_v2000(fragment, "owned label", frame)
                .err()
                .ok_or("V2000 dropped label")?
                .code(),
            "unsupported_content"
        );
        assert_eq!(
            v3000::write_v3000(fragment, "owned label", frame)
                .err()
                .ok_or("V3000 dropped label")?
                .code(),
            "unsupported_content"
        );
        let mut own = StoredDocument::default();
        *own.model_mut() = doc.clone();
        for version in [Version::V2000, Version::V3000] {
            assert_eq!(
                own.to_mol_as(version, frame)
                    .err()
                    .ok_or("whole MOL dropped label")?
                    .code(),
                "unsupported_content"
            );
        }
        let cancel = Cancellation::default();
        let counts = BTreeMap::from([(fragment.atoms()[0].id, 1)]);
        let view = PageView::fit(1024, 1024, page.size())?;
        assert!(matches!(
            graph_page_scene(1024, 1024, page),
            Err(RenderError::UnsupportedChemicalContent)
        ));
        assert!(matches!(
            graph_page_scene_with_cancel(1024, 1024, page, &cancel),
            Err(RenderError::UnsupportedChemicalContent)
        ));
        assert!(matches!(
            graph_page_scene_with_display_hydrogens(1024, 1024, page, &counts, &cancel),
            Err(RenderError::UnsupportedChemicalContent)
        ));
        assert!(matches!(
            view.graph_scene(page, &cancel),
            Err(RenderError::UnsupportedChemicalContent)
        ));
        assert!(matches!(
            view.graph_scene_with_display_hydrogens(page, &counts, &cancel),
            Err(RenderError::UnsupportedChemicalContent)
        ));
        cancel.cancel();
        assert!(matches!(
            graph_page_scene_with_cancel(0, 0, page, &cancel),
            Err(RenderError::Cancelled)
        ));
        assert!(matches!(
            graph_page_scene(0, 0, page),
            Err(RenderError::InvalidViewport)
        ));
        assert_eq!(
            doc,
            model_with_label({
                let mut value = label_value();
                value["runs"][0]["text"] = json!(body);
                value
            })?
        );
    }
    Ok(())
}

#[test]
fn synchronized_smiles_replacement_cannot_acquire_and_silently_drop_authored_label_metadata()
-> TestResult {
    let raw = SmilesFileDocument::parse(Arc::from(b"CO original\r\n".as_slice()))?;
    let stored = StoredDocument::from_smiles_records(
        raw,
        RecordSelection::All,
        48.0,
        Point2::new(72.0, 96.0)?,
        PageSize::A4,
        &Cancellation::default(),
    )?;
    let page = stored.model().pages().first().ok_or("page")?;
    let fragment = page.fragments().first().ok_or("fragment")?;
    let snapshot = stored.clone();
    let mut atoms = fragment.atoms().to_vec();
    let mut value = serde_json::to_value(atoms.get(1).ok_or("oxygen")?)?;
    value["authored_label"] = label_value();
    *atoms.get_mut(1).ok_or("oxygen")? = serde_json::from_value(value)?;
    let replacement = Fragment::new(fragment.id(), atoms, fragment.bonds().to_vec())?;
    assert_eq!(
        stored
            .with_smiles_fragment(page.id(), replacement, &Cancellation::default())
            .err()
            .ok_or("SMILES replacement dropped new label")?
            .code(),
        "unsupported_content"
    );
    assert_eq!(stored, snapshot);
    assert_eq!(stored.smiles_file()?.as_bytes(), b"CO original\r\n");
    Ok(())
}

#[test]
fn native_resource_enumeration_covers_node_only_mixed_runs_and_page_captions_together() -> TestResult
{
    use bondline_model::{StyledText, TextId, TextRun, TextStyle};
    let source = std::str::from_utf8(SOURCE)?
        .replace("<font id='42' name='Roboto'/>", "<font id='42' name='Roboto'/><font id='43' name='Independent Sans'/>")
        .replace("</s></t></n>", "</s><s font='43' size='8.75' face='32' color='3'> &amp; μ </s></t></n>")
        .replace("</fragment></page>", "</fragment><t id='88' p='72 144' Justification='Left'><s font='42' size='18' face='0' color='3'>caption</s></t></page>");
    let stored = StoredDocument::from_cdxml(source.as_bytes().into(), PageSize::A4)?;
    assert_eq!(stored.to_cdxml()?.as_ref(), source.as_bytes());
    assert!(!stored.is_fully_mapped());
    assert!(
        !stored.has_caption_projection(),
        "mixed child-label documents do not inherit caption-only authority"
    );
    let mut label = label_value();
    label["runs"].as_array_mut().ok_or("runs")?.push(json!({"text":" & μ ","style":{"font_family":"Independent Sans","size_pt":8.75,"face":32,"rgb":[0.0,0.0,0.0]}}));
    let mut doc = model_with_label(label.clone())?;
    let page = PageId::new(1).ok_or("page")?;
    doc.set_page_size(page, PageSize::new(256.0, 256.0)?)?;
    doc.append_texts(
        page,
        vec![StyledText::new(
            TextId::new(1).ok_or("caption id")?,
            Point2::new(72.0, 144.0)?,
            vec![TextRun::new(
                "caption".into(),
                TextStyle::new("Roboto".into(), 18.0, 0, [0.0; 3])?,
            )?],
        )?],
    )?;
    assert_eq!(
        stored.model(),
        &doc,
        "numeric source Text IDs21 and88 are not application caption IDs"
    );
    let expected = json!([
        {"parent":"n","position":[71.5,96.25],"justification":"Left","label_justification":"Auto","label_alignment":"Right",
         "runs":[{"text":"OH","font_family":"Roboto","size_pt":12.5,"face":96,"rgb":[0.2,0.4,0.8]},
                 {"text":" & μ ","font_family":"Independent Sans","size_pt":8.75,"face":32,"rgb":[0.0,0.0,0.0]}]},
        {"parent":"page","position":[72.0,144.0],"justification":"Left","label_justification":null,"label_alignment":null,
         "runs":[{"text":"caption","font_family":"Roboto","size_pt":18.0,"face":0,"rgb":[0.0,0.0,0.0]}]}
    ]);
    assert_eq!(independently_read(source.as_bytes())?, expected);
    let encoded = write_document(&doc)?;
    assert_eq!(independently_read(&encoded)?, expected);
    let reopened = StoredDocument::from_cdxml(encoded.into(), PageSize::A4)?;
    assert_eq!(reopened.model(), &doc);
    assert!(!reopened.is_fully_mapped());
    assert!(!reopened.has_caption_projection());
    let value = serde_json::to_value(reopened.model())?;
    assert_eq!(value["pages"][0]["texts"][0]["id"], 1);
    assert_eq!(
        value["pages"][0]["fragments"][0]["atoms"][0]["authored_label"],
        label
    );
    Ok(())
}

#[test]
fn raw_native_label_encoding_does_not_grant_foreign_source_or_editing_authority() -> TestResult {
    let document = model_with_label(label_value())?;
    let mut native = StoredDocument::from_cdxml(write_document(&document)?.into(), PageSize::A4)?;
    let page = native.model().pages()[0].id();
    native
        .model_mut()
        .set_page_size(page, PageSize::new(512.0, 512.0)?)?;
    let bytes = native.to_cdxml()?.into_owned();
    let reopened = StoredDocument::from_cdxml(bytes.clone().into(), PageSize::A4)?;
    assert_eq!(reopened.model(), native.model());
    assert_eq!(independently_read(&bytes)?, expected_facts());
    assert!(!reopened.is_fully_mapped());
    assert!(!reopened.has_display_projection());

    let mut opaque = StoredDocument::from_cdxml(SOURCE.into(), PageSize::A4)?;
    opaque
        .model_mut()
        .set_page_size(page, PageSize::new(512.0, 512.0)?)?;
    assert!(matches!(
        opaque.to_cdxml(),
        Err(bondline_io::DocumentError::UnsupportedEdit)
    ));

    let mut no_label = document;
    let fragment = &no_label.pages()[0].fragments()[0];
    let mut atoms = fragment.atoms().to_vec();
    atoms[0].authored_label = None;
    no_label.replace_fragment(page, Fragment::new(fragment.id(), atoms, vec![])?)?;
    let foreign = String::from_utf8(write_document(&no_label)?)?.replacen(
        "<page id=\"1\"",
        "<page id='91'",
        1,
    );
    let mut preserved = StoredDocument::from_cdxml(foreign.as_bytes().into(), PageSize::A4)?;
    assert!(
        preserved.is_fully_mapped(),
        "qualify existing closed source before injection"
    );
    preserved.model_mut().replace_fragment(
        page,
        model_with_label(label_value())?.pages()[0].fragments()[0]
            .as_ref()
            .clone(),
    )?;
    assert!(!preserved.is_fully_mapped());
    assert!(matches!(
        preserved.to_cdxml(),
        Err(bondline_io::DocumentError::UnsupportedEdit)
    ));
    Ok(())
}
// Append to crates/io/tests/atom_label_data.rs, reusing SOURCE and label_value().
#[test]
fn explicit_empty_namespace_is_ordinary_child_text_without_granting_label_only_chemistry()
-> TestResult {
    let reset = std::str::from_utf8(SOURCE)?.replacen("<t id='21'", "<t xmlns='' id='21'", 1);
    let graph = ElementalGraph::parse(reset.as_bytes())?;
    assert_eq!(graph.to_bytes(), reset.as_bytes());
    let value = serde_json::to_value(graph.fragments())?;
    assert_eq!(value[0]["atoms"][0]["authored_label"], label_value());
    let absent = reset.replacen(" Element='8'", "", 1);
    assert!(matches!(
        ElementalGraph::parse(absent.as_bytes()),
        Err(bondline_io::cdxml::MappingError::UnsupportedLabelOnlyAtom)
    ));
    Ok(())
}
