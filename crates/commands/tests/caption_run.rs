use bondline_commands::Engine;
use bondline_io::StoredDocument;
use bondline_model::PageSize;
use proptest::prelude::*;
use serde_json::json;
type TestResult = Result<(), Box<dyn std::error::Error>>;
const CAPTION: &[u8] = br#"<CDXML CreationProgram="Independent fixture"><fonttable><font id="42" name="Roboto"/></fonttable><colortable><color r="1" g="1" b="1"/><color r="0" g="0" b="0"/><color r="0.2" g="0.4" b="0.8"/></colortable><page id="90" DrawingSpace="Poster" Width="256" Height="256"><t id="21" p="72 96" Justification="Left"><s font="42" size="12" face="0" color="4">Phase A</s></t></page></CDXML>"#;

proptest! {
    #![proptest_config(ProptestConfig { cases:24, rng_seed:proptest::test_runner::RngSeed::Fixed(671308429), failure_persistence:None, ..ProptestConfig::default() })]
    #[test]
    fn generated_caption_edits_restore_complete_snapshots_and_exact_source(
        edits in proptest::collection::vec((0usize..2,0usize..5),1..12)
    ) {
        let mut engine=caption_engine(MULTI).map_err(|error|TestCaseError::fail(error.to_string()))?;
        let mut snapshots=vec![engine.content().clone()];
        let mut expected=serde_json::to_value(engine.document()?)?;
        let mut bodies=["Phase A"," q"];
        let literals=[("",""),("Phase A","Phase A"),("β & q","β &amp; q"),(" μmol"," μmol"),("T & V","T &amp; V")];
        for (run,index) in edits {
            let (text,escaped)=literals[index];
            let changed=expected["pages"][0]["texts"][0]["runs"][run]["text"]!=text;
            let result=engine.dispatch("text.set_run",json!({"expected_revision":engine.state().revision,
                "page_id":1,"text_id":1,"run_index":run,"text":text}))?;
            expected["pages"][0]["texts"][0]["runs"][run]["text"]=json!(text);
            prop_assert_eq!(result.changed,changed);
            prop_assert_eq!(serde_json::to_value(engine.document()?)?,expected.clone());
            bodies[run]=escaped;
            let xml=std::str::from_utf8(MULTI)?.replacen(">Phase A</s>",&format!(">{}</s>",bodies[0]),1)
                .replacen("> q</s>",&format!(">{}</s>",bodies[1]),1);
            let encoded=engine.stored_document()?.to_cdxml()?;
            prop_assert_eq!(encoded.as_ref(),xml.as_bytes());
            if changed {snapshots.push(engine.content().clone());}
        }
        for expected in snapshots.iter().rev().skip(1) {
            engine.dispatch("history.undo",json!({}))?;
            prop_assert_eq!(engine.content(),expected);
        }
        prop_assert_eq!(engine.stored_document()?.export_cdxml()?,MULTI);
        for expected in snapshots.iter().skip(1) {
            engine.dispatch("history.redo",json!({}))?;
            prop_assert_eq!(engine.content(),expected);
        }
    }
}

#[test]
fn a_foreign_caption_run_edit_is_one_source_preserving_history_entry() -> TestResult {
    let mut e = Engine::from_document(StoredDocument::from_cdxml(CAPTION.into(), PageSize::A4)?);
    let result = e.dispatch(
        "text.set_run",
        json!({"expected_revision":0,"page_id":1,
        "text_id":1,"run_index":0,"text":"Phase B"}),
    )?;
    assert!(result.changed && result.file.dirty);
    assert_eq!(
        (result.revision, result.undo_depth, result.redo_depth),
        (1, 1, 0)
    );
    assert_eq!(
        serde_json::to_value(e.document()?)?["pages"][0]["texts"][0]["runs"][0]["text"],
        "Phase B"
    );
    assert_eq!(
        e.stored_document()?.to_cdxml()?.as_ref(),
        String::from_utf8(CAPTION.to_vec())?
            .replace("Phase A", "Phase B")
            .as_bytes()
    );
    Ok(())
}

use bondline_commands::{CloseState, Command};
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, DrawingStyle, Element, Fragment, FragmentId, PageId,
    Point2, StyledText, TextId, TextRun, TextStyle,
};
use serde_json::Value;
use std::{
    fs,
    io::Write,
    process::{Command as Process, Stdio},
    sync::Arc,
};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(
    dead_code,
    reason = "reuse the unchanged isolated directory helper only"
)]
mod support;

const MULTI: &[u8] = br#"<CDXML CreationProgram='Original caption test'>
<fonttable><font id='42' name='Roboto'/></fonttable><colortable><color r='1' g='1' b='1'/><color r='0' g='0' b='0'/><color r='0.2' g='0.4' b='0.8'/></colortable>
<page id='90' DrawingSpace='Poster' Width='256' Height='256'><t id='21' p='72 96' Justification='Left'><s font='42' size='12' face='0' color='4'>Phase A</s><s font='42' size='18' face='0' color='3'> q</s></t><t id='22' p='72 144' Justification='Left'><s font='42' size='12' face='0' color='3'>Other</s></t></page></CDXML>"#;

fn caption_engine(source: &[u8]) -> Result<Engine, Box<dyn std::error::Error>> {
    Ok(Engine::from_document(StoredDocument::from_cdxml(
        source.into(),
        PageSize::A4,
    )?))
}

fn run_params(e: &Engine, text: &str) -> Value {
    json!({"expected_revision":e.state().revision,"page_id":1,"text_id":1,"run_index":0,"text":text})
}

fn refused(e: &mut Engine, params: Value, code: &str) -> TestResult {
    let state = serde_json::to_value(e.state())?;
    let content = e.content().clone();
    let stamp = e.capture_stamp();
    let file = e.ui_file_context_stamp();
    let model = e.stored_document().ok().map(StoredDocument::model_snapshot);
    let error = e
        .dispatch("text.set_run", params)
        .err()
        .ok_or("invalid caption edit accepted")?;
    assert_eq!(error.info().code, code, "{error}");
    assert_eq!(serde_json::to_value(e.state())?, state);
    assert_eq!(e.content(), &content);
    assert_eq!(e.capture_stamp(), stamp);
    assert!(e.ui_file_context_stamp_matches(&file));
    if let Some(model) = model {
        assert!(Arc::ptr_eq(&model, &e.stored_document()?.model_snapshot()));
    }
    Ok(())
}

#[test]
fn unicode_edit_changes_only_one_run_and_preserves_all_styles_ids_positions_and_source_lexemes()
-> TestResult {
    let mut e = caption_engine(MULTI)?;
    assert!(e.stored_document()?.has_caption_projection());
    assert!(
        !e.stored_document()?.is_fully_mapped(),
        "foreign caption permission is operation-specific"
    );
    let before = e.stored_document()?.clone();
    let untouched = Arc::clone(&e.document()?.pages()[0].texts()[1]);
    let mut expected = serde_json::to_value(e.document()?)?;
    expected["pages"][0]["texts"][0]["runs"][0]["text"] = json!("Phase β & q");
    let r = e.dispatch("text.set_run", run_params(&e, "Phase β & q"))?;
    assert_eq!(
        (r.changed, r.revision, r.undo_depth, r.redo_depth),
        (true, 1, 1, 0)
    );
    assert!(r.file.dirty && r.source.ok_or("source")?.modified);
    assert_eq!(serde_json::to_value(e.document()?)?, expected);
    assert!(Arc::ptr_eq(
        &untouched,
        &e.document()?.pages()[0].texts()[1]
    ));
    let raw = String::from_utf8(MULTI.to_vec())?.replace("Phase A", "Phase β &amp; q");
    assert_eq!(e.stored_document()?.to_cdxml()?.as_ref(), raw.as_bytes());
    assert_eq!(before.export_cdxml()?, MULTI);
    assert!(!e.stored_document()?.is_fully_mapped());
    let edited = e.stored_document()?.clone();
    e.dispatch("history.undo", json!({}))?;
    assert_eq!(e.stored_document()?, &before);
    e.dispatch("history.redo", json!({}))?;
    assert_eq!(e.stored_document()?, &edited);
    // Caption permission must never enable chemical mutation on a foreign caption page.
    let full = serde_json::to_value(e.state())?;
    let error = e
        .dispatch(
            "bond.create",
            json!({"page_id":1,"begin_x_pt":20,"begin_y_pt":20,"end_x_pt":68,"end_y_pt":20}),
        )
        .err()
        .ok_or("caption authorised chemical editing")?;
    assert_eq!(error.info().code, "unsupported_content");
    assert_eq!(serde_json::to_value(e.state())?, full);
    let error = e
        .dispatch(
            "page.set_size",
            json!({"page_id":1,"width_pt":512,"height_pt":256}),
        )
        .err()
        .ok_or("caption authorised page edits")?;
    assert_eq!(error.info().code, "unsupported_content");
    assert_eq!(serde_json::to_value(e.state())?, full);
    for control in e.controls().iter().filter(|control| control.menu == "Page") {
        assert!(!control.enabled);
        assert_eq!(control.disabled_reason, Some("unsupported_content"));
    }
    Ok(())
}

#[test]
fn true_noop_retains_redo_close_model_source_and_every_transient_context() -> TestResult {
    let mut e = caption_engine(CAPTION)?;
    let original = e.stored_document()?.model_snapshot();
    e.dispatch("text.set_run", run_params(&e, "Phase B"))?;
    e.dispatch("history.undo", json!({}))?;
    assert_eq!(e.state().redo_depth, 1);
    e.dispatch(
        "document.close",
        json!({"expected_revision":e.state().revision,"action":"request"}),
    )?;
    assert_eq!(e.state().close, CloseState::Approved);
    let state = serde_json::to_value(e.state())?;
    let stamp = e.capture_stamp();
    let file = e.ui_file_context_stamp();
    let bytes_ptr = e.stored_document()?.export_cdxml()?.as_ptr();
    assert!(Arc::ptr_eq(
        &original,
        &e.stored_document()?.model_snapshot()
    ));
    let r = e.dispatch("text.set_run", run_params(&e, "Phase A"))?;
    assert!(!r.changed);
    assert_eq!(serde_json::to_value(e.state())?, state);
    assert_eq!(e.capture_stamp(), stamp);
    assert!(e.ui_file_context_stamp_matches(&file));
    assert_eq!(e.stored_document()?.export_cdxml()?.as_ptr(), bytes_ptr);
    assert_eq!(e.stored_document()?.export_cdxml()?, CAPTION);
    assert!(Arc::ptr_eq(
        &original,
        &e.stored_document()?.model_snapshot()
    ));
    let revision = e.state().revision;
    let changed = e.dispatch("text.set_run", run_params(&e, "Phase β"))?;
    assert_eq!(
        (
            changed.changed,
            changed.revision,
            changed.undo_depth,
            changed.redo_depth
        ),
        (true, revision + 1, 1, 0)
    );
    assert_eq!(changed.close, CloseState::NeedsDecision);
    assert!(
        !e.complete_close(),
        "old approval cannot approve the changed caption"
    );
    Ok(())
}

#[test]
fn closed_required_params_target_errors_and_unrenderable_text_are_atomic() -> TestResult {
    let entry = bondline_commands::registry()
        .into_iter()
        .find(|entry| entry.name == "text.set_run")
        .ok_or("caption registry missing")?;
    let schema = serde_json::to_value(entry.input_schema)?;
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(
        schema["properties"].as_object().ok_or("properties")?.len(),
        5
    );
    let mut required = schema["required"]
        .as_array()
        .ok_or("required")?
        .iter()
        .map(|v| v.as_str().ok_or("name"))
        .collect::<Result<Vec<_>, _>>()?;
    required.sort_unstable();
    assert_eq!(
        required,
        [
            "expected_revision",
            "page_id",
            "run_index",
            "text",
            "text_id"
        ]
    );
    let mut e = caption_engine(CAPTION)?;
    let valid = run_params(&e, "Phase A");
    for field in [
        "expected_revision",
        "page_id",
        "text_id",
        "run_index",
        "text",
    ] {
        let mut params = valid.clone();
        params.as_object_mut().ok_or("params")?.remove(field);
        refused(&mut e, params, "invalid_params")?;
    }
    for (field, value, code) in [
        ("extra", json!(0), "invalid_params"),
        ("page_id", json!(0), "invalid_params"),
        ("text_id", json!(0), "invalid_params"),
        ("page_id", json!(2), "not_found"),
        ("text_id", json!(2), "not_found"),
        ("run_index", json!(1), "not_found"),
        ("run_index", json!(usize::MAX), "not_found"),
        ("run_index", json!(-1), "invalid_params"),
        ("run_index", json!(0.5), "invalid_params"),
        ("run_index", json!("0"), "invalid_params"),
        ("page_id", Value::Null, "invalid_params"),
        ("text", Value::Null, "invalid_params"),
        ("text", json!(12), "invalid_params"),
        ("text", json!("bad\u{0}value"), "invalid_params"),
        ("text", json!("line\nbreak"), "unsupported_content"),
        ("text", json!("missing \u{10ffff}"), "unsupported_content"),
        ("text", json!("a".repeat(1_048_577)), "invalid_params"),
        ("text", json!("a".repeat(4097)), "unsupported_content"),
        ("text", json!(" ".repeat(4097)), "render_failed"),
        ("expected_revision", json!(u64::MAX), "stale_revision"),
    ] {
        let mut params = valid.clone();
        params[field] = value;
        refused(&mut e, params, code)?;
    }
    for params in [json!(null), json!([]), json!([0, 1, 1, 0, "Phase A"])] {
        refused(&mut e, params, "invalid_params")?;
    }
    let mut stale = valid;
    stale["expected_revision"] = json!(u64::MAX);
    stale["page_id"] = json!(0);
    refused(&mut e, stale, "stale_revision")?;
    e.dispatch(
        "document.close",
        json!({"expected_revision":0,"action":"request"}),
    )?;
    assert!(e.complete_close());
    let params = run_params(&e, "Phase A");
    refused(&mut e, params, "close_committed")?;
    Ok(())
}

#[test]
fn unknown_foreign_sources_and_raw_model_drift_never_gain_caption_edit_authority() -> TestResult {
    for source in [
        String::from_utf8(CAPTION.to_vec())?.replace("<CDXML ", "<CDXML Private='keep' "),
        String::from_utf8(CAPTION.to_vec())?
            .replace("name=\"Roboto\"", "name=\"Uninstalled family\""),
        String::from_utf8(CAPTION.to_vec())?.replace("</page>", "<oval id='99'/></page>"),
    ] {
        let mut e = caption_engine(source.as_bytes())?;
        assert!(!e.stored_document()?.has_caption_projection());
        let params = run_params(&e, "Phase A");
        refused(&mut e, params, "unsupported_content")?;
        assert_eq!(e.stored_document()?.export_cdxml()?, source.as_bytes());
    }
    for kind in 0..7 {
        let mut stored = StoredDocument::from_cdxml(CAPTION.into(), PageSize::A4)?;
        let original = stored.clone();
        let old = stored.model().pages()[0].texts()[0].as_ref().clone();
        let page = PageId::new(1).ok_or("page")?;
        match kind {
            0 => stored
                .model_mut()
                .replace_text(page, old.with_run_text(0, "Phase raw".into())?)?,
            1 => stored
                .model_mut()
                .replace_text(page, old.with_position(Point2::new(73.0, 96.0)?)?)?,
            2 => {
                let style = TextStyle::new("Roboto".into(), 18.0, 0, [0.2, 0.4, 0.8])?;
                stored.model_mut().replace_text(
                    page,
                    StyledText::new(
                        old.id(),
                        old.position(),
                        vec![TextRun::new("Phase A".into(), style)?],
                    )?,
                )?;
            }
            3 => {
                let style = TextStyle::new("Roboto".into(), 12.0, 0, [0.8, 0.2, 0.4])?;
                stored.model_mut().replace_text(
                    page,
                    StyledText::new(
                        old.id(),
                        old.position(),
                        vec![TextRun::new("Phase A".into(), style)?],
                    )?,
                )?;
            }
            4 => stored
                .model_mut()
                .set_page_size(page, PageSize::new(300.0, 256.0)?)?,
            5 => stored
                .model_mut()
                .set_drawing_style(DrawingStyle::new(Some(2.0))?),
            _ => {
                let mut a = Atom::new(
                    AtomId::new(91).ok_or("atom")?,
                    Element::new(6).ok_or("C")?,
                    Some(Point2::new(160.0, 160.0)?),
                );
                a.explicit_hydrogens = Some(3);
                let mut b = Atom::new(
                    AtomId::new(92).ok_or("atom")?,
                    Element::new(6).ok_or("C")?,
                    Some(Point2::new(208.0, 160.0)?),
                );
                b.explicit_hydrogens = Some(3);
                stored.model_mut().append_fragments(
                    page,
                    vec![Fragment::new(
                        FragmentId::new(17).ok_or("fragment")?,
                        vec![a, b],
                        vec![Bond {
                            id: BondId::new(41).ok_or("bond")?,
                            begin: AtomId::new(91).ok_or("begin")?,
                            end: AtomId::new(92).ok_or("end")?,
                            order: BondOrder::Single,
                            double_position: bondline_model::DoublePosition::Unspecified,
                        }],
                    )?],
                )?;
            }
        }
        if kind == 0 {
            // Same-style text is a supported low-level format delta, not a separate authorisation.
            assert!(stored.has_caption_projection());
            assert!(!stored.is_fully_mapped());
            assert!(stored.source_info().ok_or("source")?.modified);
            assert!(
                stored.export_cdxml().is_err(),
                "never borrow stale original text"
            );
            let expected = String::from_utf8(CAPTION.to_vec())?.replace("Phase A", "Phase raw");
            let encoded = stored.to_cdxml()?.into_owned();
            assert_eq!(encoded, expected.as_bytes());
            assert_eq!(
                inspect_caption_xml(&encoded)?,
                literal_caption_readback("Phase raw")
            );
            assert_eq!(original.export_cdxml()?, CAPTION);
            assert!(!original.source_info().ok_or("original source")?.modified);
            continue;
        }
        assert!(!stored.has_caption_projection(), "raw drift {kind}");
        assert!(
            stored.to_cdxml().is_err(),
            "drift cannot save stale text/source"
        );
        let mut e = Engine::from_document(stored);
        let params = run_params(&e, if kind == 0 { "Phase raw" } else { "Phase A" });
        refused(&mut e, params, "unsupported_content")?;
    }
    let directory = support::FileTestDirectory::new()?;
    for (name, bytes, command) in [
        (
            "foreign.mol",
            include_bytes!("../../io/tests/fixtures/mol/ethane-v2000.mol").as_slice(),
            "document.open_mol",
        ),
        (
            "foreign.cdx",
            include_bytes!("../../io/tests/fixtures/cdx/metadata-explicit.cdx").as_slice(),
            "document.open_cdx",
        ),
        (
            "foreign.sdf",
            include_bytes!("../../io/tests/fixtures/sdf/mixed-v2000-v3000.sdf").as_slice(),
            "document.open_sdf",
        ),
        (
            "foreign.smi",
            b"CC original\n".as_slice(),
            "document.open_smiles",
        ),
    ] {
        let path = directory.path(name);
        fs::write(&path, bytes)?;
        let mut e = Engine::default();
        e.dispatch(command, json!({"path":path}))?;
        let params = run_params(&e, "Phase A");
        refused(&mut e, params, "unsupported_content")?;
        assert_eq!(fs::read(&path)?, bytes);
    }
    Ok(())
}

#[test]
fn sourcefree_and_complete_native_captions_edit_without_a_foreign_permission_shortcut() -> TestResult
{
    let page = PageId::new(1).ok_or("page")?;
    let mut stored = StoredDocument::default();
    stored.model_mut().append_texts(
        page,
        vec![StyledText::new(
            TextId::new(1).ok_or("text")?,
            Point2::new(72.0, 96.0)?,
            vec![TextRun::new(
                "Native A".into(),
                TextStyle::new("Roboto".into(), 12.0, 0, [0.0; 3])?,
            )?],
        )?],
    )?;
    let native = stored.to_cdxml()?.into_owned();
    let imported = StoredDocument::from_cdxml(native.into(), PageSize::A4)?;
    assert!(
        imported.is_fully_mapped(),
        "new native captions need an exact complete encoder proof"
    );
    for stored in [stored, imported] {
        let mut e = Engine::from_document(stored);
        let before = serde_json::to_value(e.document()?)?;
        e.dispatch("text.set_run", run_params(&e, "Native β"))?;
        let mut expected = before;
        expected["pages"][0]["texts"][0]["runs"][0]["text"] = json!("Native β");
        assert_eq!(serde_json::to_value(e.document()?)?, expected);
        assert_eq!((e.state().revision, e.state().undo_depth), (1, 1));
        let result = e.dispatch("text.set_run", run_params(&e, ""))?;
        assert!(result.changed);
        assert_eq!(e.document()?.pages()[0].texts()[0].runs()[0].text(), "");
        assert!(
            e.stored_document()?
                .to_cdxml()?
                .as_ref()
                .windows(2)
                .any(|w| w == b"<t")
        );
    }
    Ok(())
}

fn inspect_caption_xml(bytes: &[u8]) -> Result<Value, Box<dyn std::error::Error>> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child=Process::new(python).args(["-B","-c",r#"
import json,sys,xml.parsers.expat
from decimal import Decimal
p=xml.parsers.expat.ParserCreate(); texts=[]; fonts=[]; colors=[]; current=None; run=None

def start(name,a):
 global current,run
 if name=='font': fonts.append(dict(a))
 elif name=='color': colors.append({k:str(Decimal(a[k])) for k in ('r','g','b')})
 elif name=='t':
  current={'id':int(a['id']),'p':[str(Decimal(v)) for v in a['p'].split()], 'justification':a['Justification'],'runs':[]};texts.append(current)
 elif name=='s':
  assert current is not None
  run={'font':int(a['font']),'size':str(Decimal(a['size'])),'face':int(a['face']),'color':int(a['color']),'text':''};current['runs'].append(run)

def data(value):
 if run is not None: run['text']+=value

def end(name):
 global current,run
 if name=='s':run=None
 elif name=='t':current=None
p.StartElementHandler=start;p.CharacterDataHandler=data;p.EndElementHandler=end
p.Parse(sys.stdin.buffer.read(),True)
print(json.dumps({'fonts':fonts,'colors':colors,'texts':texts},ensure_ascii=False,separators=(',',':')))
"#]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    child.stdin.take().ok_or("Expat input")?.write_all(bytes)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn literal_caption_readback(value: &str) -> Value {
    json!({"fonts":[{"id":"42","name":"Roboto"}],"colors":[{"r":"1","g":"1","b":"1"},{"r":"0","g":"0","b":"0"},{"r":"0.2","g":"0.4","b":"0.8"}],
        "texts":[{"id":21,"p":["72","96"],"justification":"Left","runs":[{"font":42,"size":"12","face":0,"color":4,"text":value}]}]})
}

#[test]
fn unicode_save_and_reopen_keep_original_current_saved_baselines_and_literal_xml_distinct()
-> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("original-caption-λ.cdxml");
    let output = directory.path("saved-caption-β.cdxml");
    fs::write(&input, CAPTION)?;
    let mut e = Engine::default();
    e.dispatch("document.open_cdxml", json!({"path":input}))?;
    let original = e.stored_document()?.model_snapshot();
    e.dispatch("text.set_run", run_params(&e, "Phase β & q"))?;
    let changed = e.stored_document()?.model_snapshot();
    let saved = e.dispatch("document.save_cdxml", json!({"path":output}))?;
    assert!(!saved.file.dirty && saved.source.ok_or("source")?.modified);
    let bytes = fs::read(&output)?;
    let expected = String::from_utf8(CAPTION.to_vec())?.replace("Phase A", "Phase β &amp; q");
    assert_eq!(bytes, expected.as_bytes());
    assert_eq!(
        inspect_caption_xml(&bytes)?,
        literal_caption_readback("Phase β & q")
    );
    e.dispatch("history.undo", json!({}))?;
    assert!(e.state().file.dirty);
    assert!(!e.state().source.ok_or("source")?.modified);
    assert!(Arc::ptr_eq(
        &original,
        &e.stored_document()?.model_snapshot()
    ));
    assert_eq!(e.stored_document()?.export_cdxml()?, CAPTION);
    e.dispatch("history.redo", json!({}))?;
    assert!(!e.state().file.dirty);
    assert!(Arc::ptr_eq(
        &changed,
        &e.stored_document()?.model_snapshot()
    ));
    let mut reopened = Engine::default();
    reopened.dispatch("document.open_cdxml", json!({"path":output}))?;
    assert!(!reopened.state().file.dirty);
    assert!(!reopened.state().source.ok_or("new original")?.modified);
    assert_eq!(reopened.document()?, changed.as_ref());
    assert!(reopened.stored_document()?.has_caption_projection());
    assert!(!reopened.stored_document()?.is_fully_mapped());
    assert_eq!(reopened.stored_document()?.export_cdxml()?, bytes);
    assert_eq!(fs::read(&input)?, CAPTION);
    assert_eq!(fs::read(&output)?, bytes);
    Ok(())
}

#[test]
fn captured_caption_save_holds_one_shared_lease_and_never_cleans_a_later_edit() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("original-lease-λ.cdxml");
    let output = directory.path("captured-lease-β.cdxml");
    let unused = directory.path("unused.cdxml");
    fs::write(&input, CAPTION)?;
    let mut e = Engine::default();
    e.dispatch("document.open_cdxml", json!({"path":input}))?;
    let pending = e.prepare(Command::parse(
        "document.save_cdxml",
        json!({"path":output}),
    )?)?;
    e.dispatch("text.set_run", run_params(&e, "Phase β"))?;
    let current = e.stored_document()?.model_snapshot();
    let request = || Command::parse("document.save_cdxml", json!({"path":unused}));
    assert_eq!(
        e.prepare(request()?)
            .err()
            .ok_or("save lease lost")?
            .info()
            .code,
        "busy"
    );
    let completed = pending.complete()?;
    assert_eq!(fs::read(&output)?, CAPTION);
    assert_eq!(
        inspect_caption_xml(&fs::read(&output)?)?,
        literal_caption_readback("Phase A")
    );
    assert_eq!(
        e.prepare(request()?)
            .err()
            .ok_or("lease ended before finish")?
            .info()
            .code,
        "busy"
    );
    let result = e.finish(completed)?;
    assert!(!result.changed && result.file.dirty && result.source.ok_or("source")?.modified);
    let receipt = result.file_operation.ok_or("save receipt")?;
    assert_eq!(
        (receipt.input_revision, receipt.byte_length),
        (1, CAPTION.len())
    );
    assert!(receipt.ui_request_id.is_none() && receipt.ui_request_current.is_none());
    assert!(Arc::ptr_eq(
        &current,
        &e.stored_document()?.model_snapshot()
    ));
    e.dispatch("text.set_run", run_params(&e, "Phase A"))?;
    assert!(
        !e.state().file.dirty,
        "captured original is the saved baseline"
    );
    drop(e.prepare(request()?)?);
    assert!(!unused.exists());
    assert_eq!(fs::read(&input)?, CAPTION);
    Ok(())
}
