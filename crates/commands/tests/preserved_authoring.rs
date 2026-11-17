//! Post-producer acceptance of existing public commands, independent of IO implementation.
use bondline_commands::{CommandResult, Engine};
use bondline_io::StoredDocument;
use bondline_model::PageSize;
use serde_json::{Value, json};
use std::sync::Arc;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const CANONICAL: &[u8] = include_bytes!("../../io/tests/fixtures/bondline-v0.cdxml");
const SOURCE: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<CDXML CreationProgram="Bondline 0.1.0">
 <page id='91' Width='612.00' Height="792.000" DrawingSpace='Poster'>
  <fragment id="400">
   <b id='900' E='207' Order='1' B='101' />
   <n Radical='None' id='101' p='100.00 200.000' NumHydrogens='3' Charge='0' Element='6' NodeType='Element'/>
   <n Element="6" NumHydrogens="3" NodeType="Element" p="140 200" id="207" Radical="None" Charge="0"></n>
  </fragment>
 </page>
</CDXML>
"#;

struct Pair {
    canonical: Engine,
    preserved: Engine,
}
impl Pair {
    fn new() -> TestResult<Self> {
        assert_eq!(SOURCE.len(), 464);
        let load = |bytes: &[u8]| -> TestResult<Engine> {
            Ok(Engine::from_document(StoredDocument::from_cdxml(
                Arc::from(bytes),
                PageSize::A4,
            )?))
        };
        let pair = Self {
            canonical: load(CANONICAL)?,
            preserved: load(SOURCE)?,
        };
        pair.check()?;
        assert_eq!(
            pair.preserved.stored_document()?.to_cdxml()?.as_ref(),
            SOURCE
        );
        Ok(pair)
    }
    fn check(&self) -> TestResult {
        assert_eq!(
            self.preserved.document()?,
            self.canonical.document()?,
            "FULL typed document"
        );
        let a = serde_json::to_value(self.preserved.state())?;
        let b = serde_json::to_value(self.canonical.state())?;
        for key in [
            "revision",
            "undo_depth",
            "redo_depth",
            "file",
            "close",
            "close_epoch",
            "ui_file",
            "ui_selection",
        ] {
            assert_eq!(a.get(key), b.get(key), "paired state {key}");
        }
        assert_eq!(a["source"]["format"], "cdxml");
        assert_eq!(a["source"]["modified"], b["source"]["modified"]);
        for engine in [&self.canonical, &self.preserved] {
            assert!(engine.stored_document()?.is_fully_mapped());
            let bytes = engine.stored_document()?.to_cdxml()?;
            let reopened = StoredDocument::from_cdxml(Arc::from(bytes.as_ref()), PageSize::A4)?;
            assert!(reopened.is_fully_mapped());
            assert_eq!(
                reopened.model(),
                engine.document()?,
                "no typed field lost on this export"
            );
        }
        let bytes = self.preserved.stored_document()?.to_cdxml()?;
        let text = std::str::from_utf8(&bytes)?;
        assert!(text.starts_with(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<CDXML CreationProgram=\"Bondline 0.1.0\""
        ));
        // Appended fragments are inserted after the old pre-closing-tag whitespace.
        assert!(text.ends_with("</page>\n</CDXML>\n"));
        for token in [
            "id='91'",
            "<fragment id=\"400\">",
            "<b id='900' E='207'",
            "B='101'",
            "Radical='None' id='101'",
            "id=\"207\" Radical=\"None\" Charge=\"0\"",
        ] {
            assert!(text.contains(token), "original lexical token lost: {token}");
        }
        let bond = text.find("<b id='900'").ok_or("original source bond")?;
        let first = text.find("id='101'").ok_or("original source atom1")?;
        let second = text.find("id=\"207\"").ok_or("original source atom2")?;
        assert!(
            bond < first && first < second,
            "original source object order changed"
        );
        let atoms = self.preserved.document()?.pages()[0].fragments()[0].atoms();
        if atoms[0].position.map(|p| (p.x_pt(), p.y_pt())) == Some((100.0, 200.0)) {
            assert!(
                text.contains("p='100.00 200.000'"),
                "unchanged typed position was normalized"
            );
        }
        if self.preserved.document()?.pages()[0].size().width_pt() == 612.0 {
            assert!(text.contains("Width='612.00'"));
        }
        Ok(())
    }
    fn step(&mut self, name: &str, mut params: Value, revision: bool, changed: bool) -> TestResult {
        self.check()?;
        let before = self.canonical.state();
        if revision {
            params["expected_revision"] = json!(before.revision);
        }
        // Canonical FIRST: a bad positive setup is distinguishable from a profile refusal.
        let control = self
            .canonical
            .dispatch(name, params.clone())
            .map_err(|e| format!("{name}: canonical positive control failed: {e}"))?;
        let actual = self
            .preserved
            .dispatch(name, params)
            .map_err(|e| format!("{name}: preserved source failed after canonical success: {e}"))?;
        for result in [&control, &actual] {
            assert_eq!(result.changed, changed, "{name}");
            assert_eq!(
                result.revision,
                before.revision + u64::from(changed),
                "{name}"
            );
            assert_eq!(
                result.undo_depth,
                before.undo_depth + usize::from(changed),
                "{name}"
            );
            assert_eq!(
                result.redo_depth,
                if changed { 0 } else { before.redo_depth },
                "{name}"
            );
            assert!(result.file_operation.is_none() && result.ui_file_operation.is_none());
        }
        self.check()
    }
    fn expand(&mut self) -> TestResult {
        self.step(
            "page.set_size",
            json!({"page_id":1,"width_pt":1200,"height_pt":1000}),
            false,
            true,
        )
    }
    fn history(&mut self) -> TestResult {
        let after = self.preserved.document()?.clone();
        let revision = self.canonical.state().revision;
        let depth = self.canonical.state().undo_depth;
        for name in ["history.undo", "history.redo"] {
            let a = self.canonical.dispatch(name, json!({}))?;
            let b = self.preserved.dispatch(name, json!({}))?;
            assert!(a.changed && b.changed);
            self.check()?;
        }
        assert_eq!(self.preserved.document()?, &after);
        assert_eq!(self.preserved.state().revision, revision + 2);
        assert_eq!(self.preserved.state().undo_depth, depth);
        assert_eq!(self.preserved.state().redo_depth, 0);
        Ok(())
    }
}

#[test]
fn preserved_page_settings_label_and_translation_keep_the_complete_native_state() -> TestResult {
    let mut p = Pair::new()?;
    p.expand()?;
    let settings = json!({"bond_length_pt":64,"chain_angle_degrees":120});
    p.step(
        "document.set_drawing_settings",
        settings.clone(),
        true,
        true,
    )?;
    p.step("document.set_drawing_settings", settings, true, false)?;
    p.step(
        "document.set_drawing_settings",
        json!({"bond_length_pt":null,"chain_angle_degrees":null}),
        true,
        true,
    )?;
    p.step(
        "atom.set_label",
        json!({"page_id":1,"fragment_id":1,"atom_id":2,
        "element":8,"explicit_hydrogens":1}),
        true,
        true,
    )?;
    p.step(
        "ui.selection",
        json!({"action":"set","page_id":1,"atom_ids":[1,2]}),
        true,
        false,
    )?;
    let epoch = p.canonical.selection().epoch;
    p.step(
        "selection.translate",
        json!({"expected_selection_epoch":epoch,
        "delta_x_pt":24,"delta_y_pt":-12}),
        true,
        true,
    )?;
    let doc = serde_json::to_value(p.preserved.document()?)?;
    assert_eq!(
        doc["pages"][0]["fragments"][0]["atoms"][0]["position"],
        json!({"x_pt":124.0,"y_pt":188.0})
    );
    assert_eq!(
        doc["pages"][0]["fragments"][0]["atoms"][1]["position"],
        json!({"x_pt":164.0,"y_pt":188.0})
    );
    assert_eq!(doc["pages"][0]["fragments"][0]["atoms"][1]["element"], 8);
    assert_eq!(
        doc["pages"][0]["fragments"][0]["atoms"][1]["explicit_hydrogens"],
        1
    );
    p.history()
}

#[test]
fn preserved_free_extended_and_ordered_bonds_each_have_a_canonical_positive_control() -> TestResult
{
    for (name, params, revision) in [
        (
            "bond.create",
            json!({"page_id":1,"begin_x_pt":300,"begin_y_pt":400,
            "end_x_pt":364,"end_y_pt":400,"order":"single"}),
            false,
        ),
        (
            "bond.extend",
            json!({"page_id":1,"fragment_id":1,"begin_atom_id":2,
            "end_x_pt":180,"end_y_pt":200,"hydrogen_policy":"rectify_terminal_carbon"}),
            true,
        ),
        (
            "bond.set_order",
            json!({"page_id":1,"fragment_id":1,"bond_id":1,
            "order":"double","hydrogen_policy":"rectify_neutral_carbon"}),
            true,
        ),
    ] {
        let mut p = Pair::new()?;
        p.expand()?;
        p.step(name, params, revision, true)?;
        p.history()?;
    }
    Ok(())
}

#[test]
fn preserved_sized_named_chair_and_sprout_rings_retain_the_original_source() -> TestResult {
    for selector in [
        json!({"size":6}),
        json!({"kind":"benzene"}),
        json!({"kind":"cyclohexane_chair_1"}),
    ] {
        let mut p = Pair::new()?;
        p.expand()?;
        let mut params = json!({"page_id":1,"hydrogen_policy":"rectify_neutral_carbon",
            "placement":{"mode":"free","begin_x_pt":300,"begin_y_pt":400,
                "end_x_pt":300,"end_y_pt":352,"side":"positive"}});
        params
            .as_object_mut()
            .ok_or("params")?
            .extend(selector.as_object().ok_or("selector")?.clone());
        p.step("ring.create", params, true, true)?;
        p.history()?;
    }
    let mut p = Pair::new()?;
    p.expand()?;
    p.step(
        "ring.sprout",
        json!({"page_id":1,"fragment_id":1,"atom_id":2,
        "hydrogen_policy":"rectify_neutral_carbon","ring":{"kind":"polygon","size":6},
        "ring_begin_x_pt":188,"ring_begin_y_pt":200,"ring_end_x_pt":188,
        "ring_end_y_pt":152,"side":"positive"}),
        true,
        true,
    )?;
    p.history()
}

#[test]
fn preserved_fixed_path_chains_and_smiles_paste_are_complete_writable_candidates() -> TestResult {
    for (name, params) in [
        (
            "chain.create",
            json!({"page_id":1,"count":4,"angle_degrees":120,
            "hydrogen_policy":"rectify_neutral_carbon","placement":{"mode":"free",
            "begin_x_pt":300,"begin_y_pt":400,"end_x_pt":348,"end_y_pt":400,"side":"positive"}}),
        ),
        (
            "chain.create_path",
            json!({"page_id":1,"hydrogen_policy":"rectify_neutral_carbon",
            "placement":{"mode":"atom","fragment_id":1,"atom_id":2,
            "new_points":[{"x_pt":180,"y_pt":200},{"x_pt":180,"y_pt":248}]}}),
        ),
        (
            "structure.paste_smiles",
            json!({"page_id":1,"notation":"CO",
            "bond_length_pt":48,"origin_x_pt":320,"origin_y_pt":400}),
        ),
    ] {
        let mut p = Pair::new()?;
        p.expand()?;
        p.step(name, params, true, true)?;
        p.history()?;
    }
    Ok(())
}

#[test]
fn public_cpu_capture_of_equal_preserved_and_canonical_models_is_equal_and_non_authoring()
-> TestResult {
    let mut p = Pair::new()?;
    // Give the two H3 labels a64pt edge using the already-delivered translation command.
    p.step(
        "ui.selection",
        json!({"action":"set","page_id":1,"atom_ids":[2]}),
        true,
        false,
    )?;
    p.step(
        "selection.translate",
        json!({"expected_selection_epoch":p.canonical.selection().epoch,
        "delta_x_pt":24,"delta_y_pt":0}),
        true,
        true,
    )?;
    let before_a = serde_json::to_value(p.canonical.state())?;
    let before_b = serde_json::to_value(p.preserved.state())?;
    let params = json!({"page_id":1,"width_px":1024,"height_px":1024,"backend":"cpu"});
    let a: CommandResult = p.canonical.dispatch("render.capture", params.clone())?;
    let b: CommandResult = p.preserved.dispatch("render.capture", params)?;
    assert!(!a.changed && !b.changed);
    assert_eq!(
        a.capture.as_ref().ok_or("canonical image")?.png_base64,
        b.capture.as_ref().ok_or("preserved image")?.png_base64
    );
    assert_eq!(serde_json::to_value(p.canonical.state())?, before_a);
    assert_eq!(serde_json::to_value(p.preserved.state())?, before_b);
    p.check()
}
