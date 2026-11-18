//! Independent immutable existing-API creation declarations before Flagged source authority.
use bondline_io::{DocumentError, StoredDocument};
use bondline_model::{Atom, AtomLabel, Fragment, PageSize, TextRun, TextStyle};
use std::{borrow::Cow, sync::Arc};
#[path = "../../../xtask/oracles/created_label_support.rs"]
#[allow(dead_code)]
mod oracle;
use serde_json::json;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn source(name: &str) -> Result<Vec<u8>> {
    Ok(std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/created-labels")
            .join(format!("{name}.cdxml")),
    )?)
}
fn stored(raw: &[u8]) -> Result<StoredDocument> {
    Ok(StoredDocument::from_cdxml(raw.into(), PageSize::A4)?)
}
fn case(name: &str) -> Result<StoredDocument> {
    stored(&source(name)?)
}
#[test]
fn explicit_child_with_retained_flags_has_separate_display_coverage() -> Result {
    let own = case("created_3_black")?;
    let model = serde_json::to_value(own.model())?;
    let atom = &model["pages"][0]["fragments"][0]["atoms"][2];
    assert_eq!(atom["show_terminal_carbon_labels"], json!(true));
    assert_eq!(atom["show_non_terminal_carbon_labels"], json!(false));
    assert_eq!(atom["authored_label"]["runs"][0]["text"], "CH2");
    assert_eq!(own.to_cdxml()?.as_ref(), source("created_3_black")?);
    assert!(!own.is_fully_mapped());
    assert!(own.has_display_projection());
    Ok(())
}
#[test]
fn retained_flag_child_captures_the_actual_page() -> Result {
    let own = case("created_3_black")?;
    let page = own.model().pages().first().ok_or("page")?;
    let scene =
        bondline_io::display::drawing_page_scene(1024, 1024, &own, page.id(), &Default::default())?;
    let frame = bondline_render::cpu_scene(scene.scene())?;
    frame.validate()?;
    assert_eq!([frame.width, frame.height], [1024, 1024]);
    Ok(())
}
#[test]
fn mixed_original_writer_appends_one_child_and_preserves_flag_lexemes() -> Result {
    let mut own = case("base")?;
    assert!(own.has_mixed_atom_labels());
    *own.model_mut() = case("created_3_black")?.model().clone();
    assert_eq!(own.to_cdxml()?.as_ref(), source("created_3_black")?);
    Ok(())
}

#[test]
fn retained_flagged_source_has_its_own_capability() -> Result {
    assert!(case("created_3_black")?.has_flagged_atom_labels());
    Ok(())
}

fn assert_flagged(own: &StoredDocument) {
    assert!(own.has_flagged_atom_labels());
    assert!(!own.has_boxed_atom_labels());
    assert!(!own.has_connected_atom_labels());
    assert!(!own.has_mixed_atom_labels());
    assert!(own.has_display_projection());
    assert!(!own.is_fully_mapped());
}
fn replaced(base: &StoredDocument, name: &str) -> Result<StoredDocument> {
    let mut own = base.clone();
    *own.model_mut() = case(name)?.model().clone();
    Ok(own)
}
fn changed_atom(
    base: &StoredDocument,
    id: u32,
    change: impl FnOnce(&mut Atom) -> Result,
) -> Result<StoredDocument> {
    let page = base.model().pages().first().ok_or("page")?;
    let fragment = page.fragments().first().ok_or("fragment")?;
    let mut atoms = fragment.atoms().to_vec();
    change(atoms.iter_mut().find(|a| a.id.get() == id).ok_or("atom")?)?;
    let mut own = base.clone();
    own.model_mut().replace_fragment(
        page.id(),
        Fragment::new(fragment.id(), atoms, fragment.bonds().to_vec())?,
    )?;
    Ok(own)
}
fn refused(own: &StoredDocument) {
    assert!(!own.has_flagged_atom_labels());
    assert!(!own.has_boxed_atom_labels());
    assert!(!own.has_connected_atom_labels());
    assert!(!own.has_mixed_atom_labels());
    assert!(!own.has_display_projection());
    assert!(!own.is_fully_mapped());
    assert!(matches!(
        own.to_cdxml(),
        Err(DocumentError::UnsupportedEdit)
    ));
}
#[test]
fn all_twenty_original_sources_have_precise_caps_and_independent_facts() -> Result {
    let data: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/created-labels/creation-vectors.json"
    ))?;
    let rows = data["cases"].as_array().ok_or("cases")?;
    assert_eq!(rows.len(), 20);
    for row in rows {
        let name = row["name"].as_str().ok_or("name")?;
        let raw = source(name)?;
        let own = stored(&raw)?;
        let flagged = row["flagged_expected"].as_bool().ok_or("flagged")?;
        assert_eq!(own.has_flagged_atom_labels(), flagged, "case{name}");
        if flagged {
            assert_flagged(&own);
        } else {
            assert!(own.has_mixed_atom_labels());
        }
        assert!(matches!(own.to_cdxml()?, Cow::Borrowed(_)));
        assert_eq!(own.to_cdxml()?.as_ref(), raw);
        assert_eq!(oracle::read(&raw)?, oracle::facts(name));
    }
    Ok(())
}
#[test]
fn all_original_snapshot_insertions_edits_moves_and_caption_ids_are_exact() -> Result {
    for (base, names) in [
        (
            "base",
            vec![
                "created_3_black",
                "created_3_2_black",
                "created_all_black",
                "created_2_black",
                "created_3_blue14",
                "created_3_nh",
                "endpoint_edited_created_3",
                "created_3_all_p24_m12",
                "created_3_selected_p12_5_m7_25",
                "created_all_p24_m12",
            ],
        ),
        ("paired_base", vec!["paired_created_3"]),
        (
            "dense_base",
            vec!["dense_created_3", "dense_created_3_2", "dense_created_all"],
        ),
        ("caption_base", vec!["caption_created_3"]),
    ] {
        let original = case(base)?;
        for name in names {
            let own = replaced(&original, name)?;
            assert_flagged(&own);
            let encoded = own.to_cdxml()?;
            assert_eq!(encoded.as_ref(), source(name)?, "case{name}");
            assert_eq!(oracle::read(&encoded)?, oracle::facts(name));
            let opened = stored(&encoded)?;
            assert_flagged(&opened);
            assert_eq!(opened.model(), own.model());
            assert_eq!(original.to_cdxml()?.as_ref(), source(base)?);
        }
    }
    Ok(())
}
#[test]
fn reverse_order_history_and_reopened_continuations_keep_source_ids() -> Result {
    for (base, sequence) in [
        (
            "base",
            vec![
                "created_3_black",
                "created_3_2_black",
                "created_all_black",
                "created_3_2_black",
                "created_3_black",
                "base",
                "created_2_black",
                "created_3_2_black",
                "created_all_black",
            ],
        ),
        (
            "dense_base",
            vec!["dense_created_3", "dense_created_3_2", "dense_created_all"],
        ),
    ] {
        let original = case(base)?;
        let mut own = original.clone();
        for name in sequence {
            *own.model_mut() = case(name)?.model().clone();
            assert_eq!(own.to_cdxml()?.as_ref(), source(name)?);
            if name == base {
                assert!(matches!(own.to_cdxml()?, Cow::Borrowed(_)));
            } else {
                assert_flagged(&own);
            }
        }
        assert_eq!(original.to_cdxml()?.as_ref(), source(base)?);
    }
    for (before, after) in [
        ("created_3_black", "created_3_2_black"),
        ("created_2_black", "created_all_black"),
        ("created_3_2_black", "created_all_black"),
        ("created_3_black", "created_3_old48_nh"),
        ("created_3_black", "created_3_all_p24_m12"),
        ("dense_created_3", "dense_created_all"),
    ] {
        let own = replaced(&case(before)?, after)?;
        assert_flagged(&own);
        assert_eq!(own.to_cdxml()?.as_ref(), source(after)?);
    }
    Ok(())
}
fn recolored(base: &StoredDocument, rgb: [f64; 3]) -> Result<StoredDocument> {
    changed_atom(base, 3, |atom| {
        let old = atom.authored_label.as_ref().ok_or("label")?;
        let run = old.runs().first().ok_or("run")?;
        let style = TextStyle::new("Roboto".into(), run.style().size_pt(), 0, rgb)?;
        atom.authored_label = Some(Arc::new(
            AtomLabel::new(
                None,
                old.justification(),
                old.label_justification(),
                None,
                vec![TextRun::new(run.text().into(), style)?],
            )?
            .with_bounding_box(old.bounding_box())?,
        ));
        bondline_render::validate_atom_label(atom)?;
        Ok(())
    })
}
#[test]
fn complete_getter_checks_new_style_against_original_resources() -> Result {
    let original = case("base")?;
    let created = replaced(&original, "created_3_black")?;
    let red = recolored(&created, [1.0, 0.0, 0.0])?;
    refused(&red);
    assert_eq!(original.to_cdxml()?.as_ref(), source("base")?);
    let mut source_free = StoredDocument::default();
    *source_free.model_mut() = case("created_3_black")?.model().clone();
    let native = stored(&source_free.to_cdxml()?)?;
    assert_flagged(&native);
    let mut native_new = changed_atom(&native, 2, |atom| {
        atom.authored_label = Some(Arc::new(bondline_render::create_atom_label(
            atom,
            TextStyle::new("Roboto".into(), 12.0, 0, [1.0, 0.0, 0.0])?,
        )?));
        Ok(())
    })?;
    assert!(!native_new.has_flagged_atom_labels());
    assert!(!native_new.has_display_projection());
    // Exact-native data encoding remains separate from a source/edit/display grant.
    assert_eq!(stored(&native_new.to_cdxml()?)?.model(), native_new.model());
    *native_new.model_mut() = native.model().clone();
    assert_flagged(&native_new);
    Ok(())
}
#[test]
fn existing_labels_keep_style_flags_presence_and_context_while_new_labels_initialize_style()
-> Result {
    let original = case("created_3_black")?;
    refused(&replaced(&original, "created_3_blue14")?);
    refused(&replaced(&original, "base")?);
    refused(&changed_atom(&original, 3, |atom| {
        atom.show_terminal_carbon_labels = Some(false);
        Ok(())
    })?);
    let mut page = original.clone();
    let id = page.model().pages()[0].id();
    page.model_mut()
        .set_page_size(id, PageSize::new(613.0, 799.0)?)?;
    refused(&page);
    let mut style = original.clone();
    style.model_mut().set_drawing_style(
        original
            .model()
            .drawing_style()
            .with_margin_width(Some(3.0))?,
    );
    refused(&style);
    assert_flagged(&replaced(&case("base")?, "created_3_blue14")?);
    Ok(())
}
#[test]
fn exact_original_native_data_codec_keeps_broad_page_serialization_without_edit_authority() -> Result
{
    let foreign = case("created_3_black")?;
    let mut pure = StoredDocument::default();
    *pure.model_mut() = foreign.model().clone();
    assert!(!pure.has_boxed_atom_labels());
    assert!(!pure.has_flagged_atom_labels());
    let encoded = pure.to_cdxml()?.into_owned();
    let native = stored(&encoded)?;
    assert_flagged(&native);
    let mut changed = native.clone();
    let id = changed.model().pages()[0].id();
    let size = PageSize::new(613.0, 799.0)?;
    changed.model_mut().set_page_size(id, size)?;
    assert!(!changed.has_flagged_atom_labels());
    assert!(!changed.has_display_projection());
    assert!(!changed.is_fully_mapped());
    let output = changed.to_cdxml()?;
    assert!(matches!(output, Cow::Owned(_)));
    assert_eq!(stored(&output)?.model(), changed.model());
    assert_eq!(native.to_cdxml()?.as_ref(), encoded);
    let mut foreign_page = foreign.clone();
    foreign_page.model_mut().set_page_size(id, size)?;
    refused(&foreign_page);
    Ok(())
}
#[test]
fn remaining_hidden_atoms_are_currently_proved_and_raw_children_never_disappear() -> Result {
    let own = case("created_2_black")?;
    let bad = changed_atom(&own, 3, |atom| {
        atom.position = Some(bondline_model::Point2::new(240.0, 208.0)?);
        Ok(())
    })?;
    refused(&bad);
    let text = String::from_utf8(source("base")?)?;
    let raw = text.replace("<n id='109' ", "<n id='109' FutureValue='retain' ");
    let opaque = stored(raw.as_bytes())?;
    let attempted = replaced(&opaque, "created_3_black")?;
    refused(&attempted);
    assert_eq!(opaque.to_cdxml()?.as_ref(), raw.as_bytes());
    let unsupported = text.replace(
        " ShowNonTerminalCarbonLabels='no'/>",
        " ShowNonTerminalCarbonLabels='no'><t unknown='retain'/></n>",
    );
    let opaque = stored(unsupported.as_bytes())?;
    refused(&replaced(&opaque, "created_3_black")?);
    assert_eq!(opaque.to_cdxml()?.as_ref(), unsupported.as_bytes());
    Ok(())
}
#[test]
fn old_profiles_remain_literal_and_generic_loss_guards_stay_closed() -> Result {
    let boxed = stored(include_bytes!("fixtures/dependent-labels/boxed.cdxml"))?;
    assert!(boxed.has_boxed_atom_labels());
    assert!(!boxed.has_flagged_atom_labels());
    let connected = stored(include_bytes!("fixtures/connected-labels/water.cdxml"))?;
    assert!(connected.has_connected_atom_labels());
    assert!(!connected.has_flagged_atom_labels());
    let mixed = case("base")?;
    assert!(mixed.has_mixed_atom_labels());
    assert!(!mixed.has_flagged_atom_labels());
    let flagged = case("created_3_black")?;
    assert_flagged(&flagged);
    let transform =
        bondline_io::mol::MolTransform::new(1.0, bondline_model::Point2::new(0.0, 0.0)?)?;
    assert!(flagged.to_mol(transform).is_err());
    assert!(flagged.to_cdx().is_err());
    Ok(())
}
