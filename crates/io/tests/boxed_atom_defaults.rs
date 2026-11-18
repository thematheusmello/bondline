//! Foreign explicit text must not grant Bondline's unstated glyph or bond defaults.
use bondline_io::{DocumentError, StoredDocument};
use bondline_model::{Element, Fragment, PageSize};
use std::{borrow::Cow, sync::Arc};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

// Original independently measured Roboto12pt OH vector, shared with boxed_atom_label.
const EXPLICIT: &str = r#"<CDXML CreationProgram="Original box-label oracle"><fonttable><font id="42" name="Roboto"/></fonttable><colortable><color r="1" g="1" b="1"/><color r="0" g="0" b="0"/><color r="0.2" g="0.4" b="0.8"/></colortable><page id="90" DrawingSpace="Poster" Width="512" Height="512"><fragment id="51"><n id="73" NodeType="Element" Element="8" Charge="0" Radical="None" NumHydrogens="1" p="84.1220703125 95.734375"><t id="21" BoundingBox="80.69140625 91.3515625 95.796875 100.1171875" Justification="Left" LabelJustification="Left"><s font="42" size="12" face="0" color="4">OH</s></t></n></fragment></page></CDXML>"#;
const SECOND: &str = r#"<n id='97' NodeType='Element' Element='8' Charge='0' Radical='None' NumHydrogens='1' p='184.1220703125 95.734375'><t id='21' BoundingBox='180.69140625 91.3515625 195.796875 100.1171875' Justification='Left' LabelJustification='Left'><s font='42' size='12' face='0' color='4'>OH</s></t></n>"#;
const UNLABELED: &str = r#"<n id='99' NodeType='Element' Element='6' Charge='0' Radical='None' NumHydrogens='4' p='200 240'/>"#;
const BOND: &str = "<b id='900' B='97' E='73' Order='1'/>";
const CAPTION: &str = "<t id='101' p='72 320' Justification='Left'><s font='42' size='12' face='0' color='3'>explicit caption</s></t>";

fn stored(raw: &str) -> Result<StoredDocument> {
    Ok(StoredDocument::from_cdxml(
        raw.as_bytes().into(),
        PageSize::A4,
    )?)
}

fn append(raw: &str, objects: &str) -> String {
    raw.replace("</fragment>", &format!("{objects}</fragment>"))
}

fn edited(base: &StoredDocument) -> Result<StoredDocument> {
    let page = base.model().pages().first().ok_or("page")?;
    let fragment = page.fragments().first().ok_or("fragment")?;
    let mut atoms = fragment.atoms().to_vec();
    let atom = atoms.first_mut().ok_or("atom")?;
    atom.element = Element::try_from(7)?;
    atom.explicit_hydrogens = Some(2);
    atom.authored_label = Some(Arc::new(bondline_render::position_atom_label(atom)?));
    let mut result = base.clone();
    result.model_mut().replace_fragment(
        page.id(),
        Fragment::new(fragment.id(), atoms, fragment.bonds().to_vec())?,
    )?;
    Ok(result)
}

fn opaque_without_default_authority(raw: &str) -> Result {
    let base = stored(raw)?;
    assert!(!base.is_fully_mapped());
    assert!(
        !base.has_boxed_atom_labels(),
        "foreign implicit glyph/bond defaults must not acquire boxed coverage"
    );
    assert!(!base.has_display_projection());
    assert!(matches!(base.to_cdxml()?, Cow::Borrowed(_)));
    assert_eq!(base.to_cdxml()?.as_ref(), raw.as_bytes());
    let mut noop = base.clone();
    *noop.model_mut() = base.model().clone();
    assert!(!noop.has_boxed_atom_labels());
    assert!(matches!(noop.to_cdxml()?, Cow::Borrowed(_)));
    assert_eq!(noop.to_cdxml()?.as_ref(), raw.as_bytes());
    let changed = edited(&base)?;
    assert!(!changed.has_boxed_atom_labels());
    assert!(!changed.has_display_projection());
    assert!(matches!(
        changed.to_cdxml(),
        Err(DocumentError::UnsupportedEdit)
    ));
    assert_eq!(base.to_cdxml()?.as_ref(), raw.as_bytes());
    Ok(())
}

#[test]
fn foreign_unlabeled_node_cannot_borrow_the_explicit_childs_font_defaults() -> Result {
    opaque_without_default_authority(&append(EXPLICIT, UNLABELED))
}

#[test]
fn fully_labeled_foreign_single_bond_with_absent_width_cannot_borrow_native_ink() -> Result {
    opaque_without_default_authority(&append(EXPLICIT, &format!("{SECOND}{BOND}")))
}

#[test]
fn fully_labeled_foreign_bond_even_with_explicit_root_style_stays_outside_this_closed_proof()
-> Result {
    let raw = append(EXPLICIT, &format!("{SECOND}{BOND}")).replace(
        "<CDXML ",
        "<CDXML LineWidth='1.00' BondLength='48' BondSpacing='12' ChainAngle='120' ",
    );
    opaque_without_default_authority(&raw)
}

#[test]
fn matching_producer_string_is_not_an_exception_for_foreign_implicit_objects() -> Result {
    let producer = concat!("Bondline ", env!("CARGO_PKG_VERSION"));
    for objects in [UNLABELED.to_owned(), format!("{SECOND}{BOND}")] {
        let raw = append(EXPLICIT, &objects).replace("Original box-label oracle", producer);
        opaque_without_default_authority(&raw)?;
    }
    Ok(())
}

#[test]
fn source_free_and_exact_native_richer_models_keep_their_declared_own_defaults() -> Result {
    let raw = append(EXPLICIT, &format!("{SECOND}{UNLABELED}{BOND}"))
        .replace("</page>", &format!("{CAPTION}</page>"));
    let foreign = stored(&raw)?;
    let mut pure = StoredDocument::default();
    *pure.model_mut() = foreign.model().clone();
    assert!(pure.has_boxed_atom_labels());
    assert!(pure.has_display_projection());
    assert!(!pure.is_fully_mapped());
    assert!(pure.model().drawing_style().is_empty());
    assert!(pure.model().drawing_settings().is_empty());
    let native_bytes = pure.to_cdxml()?.into_owned();
    let native = StoredDocument::from_cdxml(native_bytes.as_slice().into(), PageSize::A4)?;
    assert_eq!(native.model(), pure.model());
    assert!(native.has_boxed_atom_labels());
    assert!(native.has_display_projection());
    assert!(!native.is_fully_mapped());
    assert_eq!(native.to_cdxml()?.as_ref(), native_bytes);
    for own in [&pure, &native] {
        let page = own.model().pages().first().ok_or("page")?;
        let scene = bondline_io::display::drawing_page_scene(
            1024,
            1024,
            own,
            page.id(),
            &Default::default(),
        )?;
        assert!(scene.label_em_px().is_some());
        let changed = edited(own)?;
        assert!(changed.has_boxed_atom_labels());
        assert!(changed.has_display_projection());
        let saved = changed.to_cdxml()?.into_owned();
        let reopened = StoredDocument::from_cdxml(saved.into(), PageSize::A4)?;
        assert_eq!(reopened.model(), changed.model());
        assert!(reopened.has_boxed_atom_labels());
    }
    Ok(())
}

#[test]
fn foreign_all_explicit_labels_and_captions_remain_admitted_without_irrelevant_bond_defaults()
-> Result {
    let raw = append(EXPLICIT, SECOND).replace("</page>", &format!("{CAPTION}</page>"));
    for raw in [
        raw.clone(),
        raw.replace(" CreationProgram=\"Original box-label oracle\"", ""),
    ] {
        let base = stored(&raw)?;
        assert!(base.has_boxed_atom_labels());
        assert!(base.has_display_projection());
        assert!(!base.is_fully_mapped());
        assert_eq!(base.to_cdxml()?.as_ref(), raw.as_bytes());
        let changed = edited(&base)?;
        let saved = changed.to_cdxml()?.into_owned();
        let reopened = StoredDocument::from_cdxml(saved.as_slice().into(), PageSize::A4)?;
        assert_eq!(reopened.model(), changed.model());
        assert!(reopened.has_boxed_atom_labels());
        assert!(std::str::from_utf8(&saved)?.contains(CAPTION));
    }
    Ok(())
}
