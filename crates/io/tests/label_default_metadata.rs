//! Explicit run styles keep their ink while unused Label* metadata stays in source.
use bondline_io::StoredDocument;
use bondline_model::PageSize;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

// Original independently checked creation vectors; metadata-only additions are literal.
fn declared(name: &str) -> &'static str {
    match name {
        "base" => include_str!("fixtures/created-labels/base.cdxml"),
        "created" => include_str!("fixtures/created-labels/created_3_black.cdxml"),
        _ => "",
    }
}
fn with_metadata(raw: &str) -> String {
    raw.replacen(
        "<CDXML ",
        "<CDXML LabelFont='42' LabelSize='10.00' LabelFace='96' ",
        1,
    )
}
fn stored(raw: &str) -> Result<StoredDocument> {
    Ok(StoredDocument::from_cdxml(
        raw.as_bytes().into(),
        PageSize::A4,
    )?)
}
#[test]
fn explicit_runs_beside_unused_default_metadata_have_display_coverage() -> Result {
    let raw = with_metadata(declared("base"));
    let own = stored(&raw)?;
    assert_eq!(own.model(), stored(declared("base"))?.model());
    assert_eq!(own.to_cdxml()?.as_ref(), raw.as_bytes());
    assert!(!own.is_fully_mapped());
    assert!(own.has_display_projection());
    Ok(())
}
#[test]
fn source_qualified_explicit_run_capture_ignores_unused_default_size_and_face() -> Result {
    let raw = with_metadata(declared("base"));
    let own = stored(&raw)?;
    let page = own.model().pages().first().ok_or("page")?;
    let scene =
        bondline_io::display::drawing_page_scene(1024, 1024, &own, page.id(), &Default::default())?;
    bondline_render::cpu_scene(scene.scene())?.validate()?;
    Ok(())
}
#[test]
fn source_writer_creates_explicit_child_without_materializing_default_metadata() -> Result {
    let mut own = stored(&with_metadata(declared("base")))?;
    *own.model_mut() = stored(declared("created"))?.model().clone();
    assert_eq!(
        own.to_cdxml()?.as_ref(),
        with_metadata(declared("created")).as_bytes()
    );
    Ok(())
}
#[test]
fn preserved_default_metadata_has_its_separate_capability() -> Result {
    assert!(stored(&with_metadata(declared("base")))?.has_label_defaults_projection());
    Ok(())
}

use bondline_io::DocumentError;
use bondline_model::{Atom, AtomLabel, Fragment, Point2, TextRun, TextStyle};
use std::{borrow::Cow, sync::Arc};

#[path = "../../../xtask/oracles/label_metadata_support.rs"]
#[allow(dead_code)]
mod oracle;

fn case(name: &str) -> Result<StoredDocument> {
    stored(oracle::source(name))
}
fn precise(own: &StoredDocument) {
    assert!(own.has_label_defaults_projection());
    assert!(own.has_display_projection());
    assert!(!own.is_fully_mapped());
    assert!(!own.has_boxed_atom_labels());
    assert!(!own.has_connected_atom_labels());
    assert!(!own.has_mixed_atom_labels());
    assert!(!own.has_flagged_atom_labels());
    assert!(!own.has_caption_projection());
}
fn replace_once(raw: &str, before: &str, after: &str) -> String {
    assert!(raw.contains(before), "declared lexical mutation: {before}");
    raw.replacen(before, after, 1)
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
fn refused_change(own: &StoredDocument) {
    assert!(!own.has_label_defaults_projection());
    assert!(!own.has_display_projection());
    assert!(!own.is_fully_mapped());
    assert!(matches!(
        own.to_cdxml(),
        Err(DocumentError::UnsupportedEdit)
    ));
}
fn opaque(raw: &str) -> Result {
    let own = stored(raw)?;
    assert!(!own.has_label_defaults_projection(), "source: {raw}");
    assert!(!own.has_display_projection(), "source: {raw}");
    assert!(!own.is_fully_mapped());
    assert!(matches!(own.to_cdxml()?, Cow::Borrowed(_)));
    assert_eq!(own.to_cdxml()?.as_ref(), raw.as_bytes());
    let mut changed = own.clone();
    let page = changed.model().pages().first().ok_or("page")?.id();
    changed
        .model_mut()
        .set_page_size(page, PageSize::new(513.0, 512.0)?)?;
    refused_change(&changed);
    assert_eq!(own.to_cdxml()?.as_ref(), raw.as_bytes());
    Ok(())
}

#[test]
fn all_frozen_originals_have_exact_separate_coverage_and_independent_facts() -> Result {
    let vectors: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/label-metadata/metadata-vectors.json"
    ))?;
    let cases = vectors["cases"].as_array().ok_or("cases")?;
    assert_eq!(cases.len(), 12);
    for row in cases {
        let name = row["name"].as_str().ok_or("name")?;
        let own = case(name)?;
        precise(&own);
        let raw = oracle::source(name).as_bytes();
        assert_eq!(raw.len() as u64, row["bytes"].as_u64().ok_or("bytes")?);
        assert!(matches!(own.to_cdxml()?, Cow::Borrowed(_)));
        assert_eq!(own.to_cdxml()?.as_ref(), raw);
        assert_eq!(
            oracle::read(raw)?,
            oracle::facts(name),
            "independent {name}"
        );
    }
    Ok(())
}

#[test]
fn source_writer_composes_original_edits_creations_moves_and_reopened_continuations() -> Result {
    for (before, afters) in [
        (
            "mixed_a",
            vec![
                "edited_endpoint",
                "created_3",
                "all_printed",
                "mixed_moved",
                "compound",
                "all_moved",
            ],
        ),
        ("created_3", vec!["all_printed", "compound", "all_moved"]),
        ("all_printed", vec!["all_moved"]),
    ] {
        let base = case(before)?;
        for after in afters {
            let mut own = base.clone();
            *own.model_mut() = case(after)?.model().clone();
            precise(&own);
            let raw = own.to_cdxml()?;
            assert_eq!(
                raw.as_ref(),
                oracle::source(after).as_bytes(),
                "{before}→{after}"
            );
            assert_eq!(oracle::read(&raw)?, oracle::facts(after));
            let reopened = stored(std::str::from_utf8(&raw)?)?;
            precise(&reopened);
            assert_eq!(reopened.model(), own.model());
            assert_eq!(base.to_cdxml()?.as_ref(), oracle::source(before).as_bytes());
        }
    }
    Ok(())
}

#[test]
fn model_equal_metadata_sources_stay_distinct_and_noops_keep_borrowed_lexemes() -> Result {
    let a = case("mixed_a")?;
    let b = case("mixed_b")?;
    assert_eq!(a.model(), b.model());
    assert_ne!(a, b);
    assert_ne!(a.to_cdxml()?.as_ref(), b.to_cdxml()?.as_ref());
    let mut copy = a.clone();
    *copy.model_mut() = a.model().clone();
    precise(&copy);
    assert!(matches!(copy.to_cdxml()?, Cow::Borrowed(_)));
    assert_eq!(
        copy.to_cdxml()?.as_ref(),
        oracle::source("mixed_a").as_bytes()
    );
    assert!(!copy.source_info().ok_or("source")?.modified);
    let mut owned_history = a.clone();
    for name in [
        "created_3",
        "all_printed",
        "created_3",
        "mixed_a",
        "edited_endpoint",
        "mixed_a",
    ] {
        *owned_history.model_mut() = case(name)?.model().clone();
        precise(&owned_history);
        assert_eq!(
            owned_history.to_cdxml()?.as_ref(),
            oracle::source(name).as_bytes()
        );
    }
    Ok(())
}

#[test]
fn unused_signed_metadata_and_unsigned_actual_font_have_separate_domains() -> Result {
    precise(&case("mixed_a")?); // Unused serif7/face96 and Bond face32767/size1e308.
    precise(&case("unsigned_actual_font")?); // Actual style/resource65535 remains UINT16.
    opaque(&replace_once(
        oracle::source("unsigned_actual_font"),
        "LabelFont=' +&#55;.000 '",
        "LabelFont='65535'",
    ))?;
    precise(&case("partial")?); // Metadata absence supplies no additional value.
    let original = with_metadata(declared("base"));
    for (before, after) in [
        ("LabelSize='10.00'", "LabelSize='1e-308'"),
        ("LabelSize='10.00'", "LabelSize='1e308'"),
        ("LabelFace='96'", "LabelFace='32767'"),
    ] {
        precise(&stored(&replace_once(&original, before, after))?);
    }
    let zero_font = replace_once(&original, "LabelFont='42'", "LabelFont='0'");
    let zero_font = replace_once(
        &zero_font,
        "</fonttable>",
        "<font id='0' name='Original unused zero family'/></fonttable>",
    );
    precise(&stored(&zero_font)?);
    let high_font = replace_once(&original, "LabelFont='42'", "LabelFont='32767'");
    let high_font = replace_once(
        &high_font,
        "</fonttable>",
        "<font id='32767' name='Original unused family'/></fonttable>",
    );
    let high = stored(&high_font)?;
    precise(&high);
    assert_eq!(high.model(), stored(&original)?.model());
    for (before, after) in [
        ("LabelFont='42'", "LabelFont='-1'"),
        ("LabelFont='42'", "LabelFont='32768'"),
        ("LabelFont='42'", "LabelFont='65535'"),
        ("LabelFont='42'", "LabelFont='99'"),
        ("LabelFont='42'", "LabelFont='42.5'"),
        ("LabelFace='96'", "LabelFace='-1'"),
        ("LabelFace='96'", "LabelFace='32768'"),
        ("LabelFace='96'", "LabelFace='0.5'"),
        ("LabelSize='10.00'", "LabelSize='0'"),
        ("LabelSize='10.00'", "LabelSize='-1'"),
        ("LabelSize='10.00'", "LabelSize='NaN'"),
        ("LabelSize='10.00'", "LabelSize='inf'"),
        ("LabelSize='10.00'", "LabelSize='1e309'"),
        ("LabelSize='10.00'", "LabelSize='1e-9999'"),
    ] {
        opaque(&replace_once(&original, before, after))?;
    }
    Ok(())
}

#[test]
fn every_scalar_owner_is_checked_even_if_shadowed_by_supported_values() -> Result {
    let base = oracle::source("mixed_a");
    for (before, after) in [
        ("LabelFont=' +&#55;.000 '", "LabelFont='999'"),
        ("LabelFace='96'", "LabelFace='-1'"),
        ("LabelSize='10.00'", "LabelSize='NaN'"),
        ("LabelSize='14.00'", "LabelSize='0'"),
        ("LabelSize='16.0'", "LabelSize='-1'"),
        ("LabelFace='0'><s", "LabelFace='32768'><s"),
        ("LabelSize='1e308'", "LabelSize='inf'"),
        ("LabelFace='32767'", "LabelFace='-1'"),
    ] {
        opaque(&replace_once(base, before, after))?;
    }
    let caption = oracle::source("caption_owners");
    opaque(&replace_once(
        caption,
        "LabelSize='20.000'",
        "LabelSize='0'",
    ))?;
    opaque(&replace_once(
        caption,
        "LabelFace='96'><s",
        "LabelFace='-1'><s",
    ))?;
    let shadowed = replace_once(
        base,
        "<n id='73' ",
        "<n id='73' LabelFont='42' LabelFace='0' ",
    );
    opaque(&replace_once(
        &shadowed,
        "LabelFont=' +&#55;.000 '",
        "LabelFont='999'",
    ))?;
    Ok(())
}

#[test]
fn unsupported_owners_namespaces_unknown_data_and_omitted_actual_runs_stay_opaque() -> Result {
    let raw = oracle::source("mixed_a");
    for (before, after) in [
        ("<page ", "<page LabelSize='12' "),
        ("<fragment ", "<fragment LabelFont='42' "),
        ("<fonttable>", "<fonttable LabelFace='0'>"),
        ("<colortable>", "<colortable LabelSize='12'>"),
        ("<font id='42'", "<font LabelSize='12' id='42'"),
        ("<colortable>", "<colortable LabelFont='42'>"),
        ("<s font='42'", "<s LabelSize='12' font='42'"),
        ("<CDXML ", "<CDXML xmlns:q='urn:original' "),
        ("<CDXML ", "<CDXML FutureOwnedValue='retain' "),
        (
            "<CDXML ",
            "<CDXML CaptionFont='42' CaptionSize='12' CaptionFace='0' ",
        ),
        ("<n id='109' ", "<n id='109' FutureValue='retain' "),
        (
            "<s font='42' size='12' face='0' color='4'>OH",
            "<s size='12' face='0' color='4'>OH",
        ),
        (
            "<s font='42' size='12' face='0' color='4'>OH",
            "<s font='42' face='0' color='4'>OH",
        ),
        (
            "<s font='42' size='12' face='0' color='4'>OH",
            "<s font='42' size='12' color='4'>OH",
        ),
        (
            "<s font='42' size='12' face='0' color='4'>OH",
            "<s font='42' size='12' face='0'>OH",
        ),
        (">OH</s>", ">NH2</s>"),
        ("BoundingBox='92.5693359375", "BoundingBox='90.5693359375"),
    ] {
        opaque(&replace_once(raw, before, after))?;
    }
    for attr in ["font='42' ", "size='12' ", "face='0' ", "color='4'"] {
        let caption = oracle::source("caption_owners");
        let start = caption.rfind("<t id='1'").ok_or("caption")?;
        let (head, tail) = caption.split_at(start);
        opaque(&format!("{head}{}", replace_once(tail, attr, "")))?;
    }
    Ok(())
}

#[test]
fn current_hidden_and_immutable_authoring_context_are_reproved_before_getter_and_write() -> Result {
    let base = case("mixed_a")?;
    for change in [
        changed_atom(&base, 3, |a| {
            a.position = Some(Point2::new(240.0, 208.0)?);
            Ok(())
        })?,
        changed_atom(&base, 3, |a| {
            a.charge = 1;
            Ok(())
        })?,
        changed_atom(&base, 3, |a| {
            a.explicit_hydrogens = Some(1);
            Ok(())
        })?,
        changed_atom(&base, 3, |a| {
            a.show_non_terminal_carbon_labels = Some(true);
            Ok(())
        })?,
        changed_atom(&base, 1, |a| {
            a.authored_label = None;
            Ok(())
        })?,
        changed_atom(&base, 1, |a| {
            let old = a.authored_label.as_ref().ok_or("label")?;
            a.authored_label = Some(Arc::new(old.with_run_text(0, "NH2".into())?));
            Ok(())
        })?,
    ] {
        refused_change(&change);
    }
    let mut page = base.clone();
    page.model_mut()
        .set_page_size(base.model().pages()[0].id(), PageSize::new(513.0, 512.0)?)?;
    refused_change(&page);
    let mut style = base.clone();
    style
        .model_mut()
        .set_drawing_style(base.model().drawing_style().with_margin_width(Some(3.0))?);
    refused_change(&style);
    assert_eq!(
        base.to_cdxml()?.as_ref(),
        oracle::source("mixed_a").as_bytes()
    );
    Ok(())
}

#[test]
fn original_resources_are_required_for_new_labels_even_with_unused_metadata_present() -> Result {
    let base = case("mixed_a")?;
    let mut created = base.clone();
    *created.model_mut() = case("created_3")?.model().clone();
    precise(&created);
    let missing_rgb = changed_atom(&created, 3, |a| {
        let old = a.authored_label.as_ref().ok_or("label")?;
        let run = old.runs().first().ok_or("run")?;
        let style = TextStyle::new("Roboto".into(), 12.0, 0, [1.0, 0.0, 0.0])?;
        a.authored_label = Some(Arc::new(
            AtomLabel::new(
                None,
                old.justification(),
                old.label_justification(),
                None,
                vec![TextRun::new(run.text().into(), style)?],
            )?
            .with_bounding_box(old.bounding_box())?,
        ));
        Ok(())
    })?;
    refused_change(&missing_rgb);
    let changed_existing = changed_atom(&case("created_3")?, 3, |a| {
        let old = a.authored_label.as_ref().ok_or("label")?;
        let run = old.runs().first().ok_or("run")?;
        let style = TextStyle::new("Roboto".into(), 12.0, 0, [0.2, 0.4, 0.8])?;
        a.authored_label = Some(Arc::new(
            AtomLabel::new(
                None,
                old.justification(),
                old.label_justification(),
                None,
                vec![TextRun::new(run.text().into(), style)?],
            )?
            .with_bounding_box(old.bounding_box())?,
        ));
        Ok(())
    })?;
    refused_change(&changed_existing);
    Ok(())
}

#[test]
fn old_profiles_and_data_codecs_do_not_gain_metadata_capability_or_loss_authority() -> Result {
    for raw in [
        declared("base"),
        declared("created"),
        include_str!("fixtures/connected-labels/water.cdxml"),
        include_str!("fixtures/dependent-labels/boxed.cdxml"),
    ] {
        assert!(!stored(raw)?.has_label_defaults_projection());
    }
    let boxed = case("boxed")?;
    precise(&boxed);
    let mut own = StoredDocument::default();
    *own.model_mut() = boxed.model().clone();
    assert!(!own.has_label_defaults_projection());
    assert!(own.has_boxed_atom_labels());
    let native = stored(std::str::from_utf8(&own.to_cdxml()?)?)?;
    assert!(!native.has_label_defaults_projection());
    let transform = bondline_io::mol::MolTransform::new(1.0, Point2::new(0.0, 0.0)?)?;
    assert!(boxed.to_mol(transform).is_err());
    assert!(boxed.to_cdx().is_err());
    assert!(!boxed.has_smiles_projection());
    let raw = replace_once(
        oracle::source("mixed_a"),
        "ShowNonTerminalCarbonLabels='no'/>",
        "ShowNonTerminalCarbonLabels='no'><t FutureValue='retain'/></n>",
    );
    opaque(&raw)?;
    Ok(())
}

#[test]
fn no_bond_explicit_flagged_labels_remain_a_distinct_metadata_mode() -> Result {
    fn without_bonds(raw: &str) -> Result<String> {
        let mut result = raw.to_owned();
        while let Some(start) = result.find("<b ") {
            let end = start
                + result
                    .get(start..)
                    .and_then(|tail| tail.find("/>"))
                    .ok_or("bond close")?
                + 2;
            result.replace_range(start..end, "");
        }
        Ok(result)
    }
    let base_raw = without_bonds(oracle::source("all_printed"))?;
    let moved_raw = without_bonds(oracle::source("all_moved"))?;
    let base = stored(&base_raw)?;
    precise(&base);
    assert!(base.model().pages()[0].fragments()[0].bonds().is_empty());
    let mut moved = base.clone();
    *moved.model_mut() = stored(&moved_raw)?.model().clone();
    precise(&moved);
    assert_eq!(moved.to_cdxml()?.as_ref(), moved_raw.as_bytes());
    assert_eq!(base.to_cdxml()?.as_ref(), base_raw.as_bytes());
    Ok(())
}
