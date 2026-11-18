//! Exact source and model acceptance for existing explicit CDX atom labels.
use bondline_io::{
    DocumentError, SourceInfo, StoredDocument,
    display::{DisplayError, drawing_page_scene},
    files::{read_cdx_file, save_cdx_file},
};
use bondline_model::{AtomId, Document, Element, FragmentId, PageId, PageSize};
use bondline_render::Cancellation;
use serde::Deserialize;
use serde_json::Value;
use std::{borrow::Cow, collections::BTreeMap, fs, mem::size_of, sync::Arc};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse the existing isolated file directory")]
mod files;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const CASES: &str = include_str!("fixtures/cdx-label-edit/cases.json");
const WATER: &[u8] = include_bytes!("fixtures/cdx-label-preview/water.cdx");
const BOXED: &[u8] = include_bytes!("fixtures/cdx-label-preview/boxed_reordered.cdx");
const CYANIDE: &[u8] = include_bytes!("fixtures/cdx-label-preview/cyanide.cdx");
const WATER_NH: &[u8] = include_bytes!("fixtures/cdx-label-edit/water_to_nh.cdx");
const BOXED_OH: &[u8] = include_bytes!("fixtures/cdx-label-edit/boxed_nh2_to_oh.cdx");
const BOXED_BR: &[u8] = include_bytes!("fixtures/cdx-label-edit/boxed_cl_to_br.cdx");

#[derive(Clone, Copy, Deserialize)]
struct Edit {
    page_id: PageId,
    fragment_id: FragmentId,
    atom_id: AtomId,
    element: Element,
    explicit_hydrogens: u16,
}

impl Edit {
    fn apply(self, stored: &StoredDocument) -> std::result::Result<StoredDocument, DocumentError> {
        stored.with_cdx_atom_label(
            self.page_id,
            self.fragment_id,
            self.atom_id,
            self.element,
            self.explicit_hydrogens,
        )
    }
}

#[derive(Deserialize)]
struct Case {
    params: Edit,
    before_model: Value,
    after_model: Value,
}

#[derive(Deserialize)]
struct Manifest {
    cases: BTreeMap<String, Case>,
}

fn witnesses() -> Result<BTreeMap<String, Case>> {
    Ok(serde_json::from_str::<Manifest>(CASES)?.cases)
}

fn read(bytes: &[u8]) -> Result<StoredDocument> {
    Ok(StoredDocument::from_cdx(Arc::from(bytes), PageSize::A4)?)
}

fn source(bytes: &[u8], modified: bool) -> Option<SourceInfo> {
    Some(SourceInfo {
        format: "cdx",
        byte_length: bytes.len(),
        modified,
    })
}

#[test]
fn edits_match_independent_bytes_and_whole_models_through_save_and_reopen() -> Result {
    let cases = witnesses()?;
    let directory = files::FileTestDirectory::new()?;
    let cancel = Cancellation::default();
    assert_eq!(cases.len(), 3);
    for (name, before, after) in [
        ("water_to_nh", WATER, WATER_NH),
        ("boxed_nh2_to_oh", BOXED, BOXED_OH),
        ("boxed_cl_to_br", BOXED, BOXED_BR),
    ] {
        let case = cases.get(name).ok_or("missing edit witness")?;
        let raw: Arc<[u8]> = Arc::from(before);
        let original = StoredDocument::from_cdx(Arc::clone(&raw), PageSize::A4)?;
        assert_eq!(
            serde_json::to_value(original.model())?,
            case.before_model,
            "{name}"
        );
        assert_eq!(original.source_info(), source(before, false), "{name}");
        assert!(
            original.can_edit_cdx_atom_label(case.params.atom_id),
            "{name}"
        );

        let edited = case.params.apply(&original)?;
        assert_eq!(
            StoredDocument::retained_bytes([&original, &edited, &edited]),
            2 * size_of::<StoredDocument>()
                + before.len()
                + after.len()
                + Document::retained_bytes([original.model(), edited.model()]),
            "each original and edited source allocation is counted once for {name}"
        );
        assert_eq!(
            serde_json::to_value(edited.model())?,
            case.after_model,
            "{name}"
        );
        assert_eq!(edited.source_info(), source(before, true), "{name}");
        assert!(edited.has_cdx_atom_label_preview(), "{name}");
        assert!(
            edited.can_edit_cdx_atom_label(case.params.atom_id),
            "{name}"
        );
        assert!(!edited.is_fully_mapped(), "{name}");
        assert!(
            matches!(edited.to_cdx()?, Cow::Borrowed(bytes) if bytes == after),
            "{name}"
        );
        assert!(
            matches!(
                edited.to_cdxml(),
                Err(DocumentError::CrossFormatSource {
                    from: "cdx",
                    to: "cdxml"
                })
            ),
            "{name}"
        );

        let path = directory.path(&format!("{name}-再開-λ.cdx"));
        let receipt = save_cdx_file(&path, &edited, false, &cancel)?;
        assert_eq!(receipt.byte_length, after.len(), "{name}");
        assert_eq!(fs::read(&path)?, after, "{name}");
        let reopened = read_cdx_file(&path, &cancel)?.document;
        assert_eq!(reopened.model(), edited.model(), "{name}");
        assert_eq!(reopened.export_cdx()?, after, "{name}");
        assert_eq!(reopened.source_info(), source(after, false), "{name}");
        assert!(
            reopened.can_edit_cdx_atom_label(case.params.atom_id),
            "{name}"
        );

        assert_eq!(original.export_cdx()?.as_ptr(), raw.as_ptr(), "{name}");
        assert_eq!(original.export_cdx()?, before, "{name}");
        assert_eq!(
            serde_json::to_value(original.model())?,
            case.before_model,
            "{name}"
        );
    }
    Ok(())
}

#[test]
fn numeric_noop_retains_model_arc_and_noncanonical_source_bytes() -> Result {
    let cases = witnesses()?;
    let case = cases
        .get("boxed_nh2_to_oh")
        .ok_or("missing boxed witness")?;
    let original = read(BOXED)?;
    let no_change = Edit {
        element: Element::try_from(7_u8)?,
        explicit_hydrogens: 2,
        ..case.params
    };
    let same = no_change.apply(&original)?;
    assert_eq!(same, original);
    assert!(Arc::ptr_eq(
        &same.model_snapshot(),
        &original.model_snapshot()
    ));
    assert_eq!(same.export_cdx()?.as_ptr(), original.export_cdx()?.as_ptr());
    assert_eq!(same.export_cdx()?, BOXED);
    assert_eq!(same.source_info(), source(BOXED, false));
    let restored = no_change.apply(&case.params.apply(&original)?)?;
    assert_eq!(restored.model(), original.model());
    assert_eq!(restored.export_cdx()?.len(), BOXED.len() - 4);
    assert_eq!(restored.source_info(), source(BOXED, true));

    let water = read(WATER)?;
    let edit = cases
        .get("water_to_nh")
        .ok_or("missing water witness")?
        .params;
    let reverse = Edit {
        element: Element::try_from(8_u8)?,
        explicit_hydrogens: 0,
        ..edit
    };
    let restored = reverse.apply(&edit.apply(&water)?)?;
    assert_eq!(restored.export_cdx()?, WATER);
    assert!(Arc::ptr_eq(
        &restored.model_snapshot(),
        &water.model_snapshot()
    ));
    assert_eq!(restored.source_info(), source(WATER, false));
    Ok(())
}

#[test]
fn successive_edits_rebase_scoped_records_on_the_current_snapshot() -> Result {
    let cases = witnesses()?;
    let first = cases
        .get("boxed_nh2_to_oh")
        .ok_or("missing first fragment witness")?;
    let second = cases
        .get("boxed_cl_to_br")
        .ok_or("missing second fragment witness")?;
    let original = read(BOXED)?;
    let first_only = first.params.apply(&original)?;
    let second_only = second.params.apply(&original)?;

    let forward = second.params.apply(&first_only)?;
    let reverse = first.params.apply(&second_only)?;
    // The independent record oracle marks the second Fragment start at 176
    // in BOXED and BOXED_BR. The first edit shifts that start by five bytes.
    let combined = [
        BOXED_OH.get(..171).ok_or("missing first fragment prefix")?,
        BOXED_BR
            .get(176..)
            .ok_or("missing second fragment suffix")?,
    ]
    .concat();
    assert_eq!(combined.len(), 339);
    let mut model = first.after_model.clone();
    model["pages"][0]["fragments"][1] = second.after_model["pages"][0]["fragments"][1].clone();
    assert_eq!(serde_json::to_value(forward.model())?, model);
    assert_eq!(forward.export_cdx()?, combined);
    assert_eq!(forward, reverse);
    assert_eq!(forward.source_info(), source(BOXED, true));
    assert_eq!(forward.model(), read(&combined)?.model());
    assert_eq!(first_only.export_cdx()?, BOXED_OH);
    assert_eq!(second_only.export_cdx()?, BOXED_BR);
    assert_eq!(original.export_cdx()?, BOXED);

    let wrong_fragment = Edit {
        fragment_id: first.params.fragment_id,
        ..second.params
    };
    let error = wrong_fragment
        .apply(&forward)
        .err()
        .ok_or("wrong fragment edited")?;
    assert_eq!(error.code(), "unsupported_content");
    assert_eq!(forward.export_cdx()?, combined);
    Ok(())
}

#[test]
fn detached_style_drift_revokes_preview_export_and_even_noop_edit_until_restored() -> Result {
    let cases = witnesses()?;
    let edit = cases
        .get("water_to_nh")
        .ok_or("missing water witness")?
        .params;
    let edited = edit.apply(&read(WATER)?)?;
    let mut detached = edited.clone();
    let style = detached.model().drawing_style();
    detached
        .model_mut()
        .set_drawing_style(style.with_margin_width(Some(11.0))?);
    assert_eq!(detached.source_info(), source(WATER, true));
    assert!(!detached.has_display_projection());
    assert!(matches!(
        drawing_page_scene(512, 512, &detached, edit.page_id, &Cancellation::default()),
        Err(DisplayError::IncompleteSource)
    ));
    assert!(!detached.can_edit_cdx_atom_label(edit.atom_id));
    assert!(matches!(
        detached.to_cdx(),
        Err(DocumentError::UnsupportedCdxEdit)
    ));
    assert!(matches!(
        detached.export_cdx(),
        Err(DocumentError::UnsupportedCdxEdit)
    ));
    let error = edit
        .apply(&detached)
        .err()
        .ok_or("detached no-op accepted")?;
    assert_eq!(error.code(), "unsupported_content");
    assert_eq!(edited.export_cdx()?, WATER_NH);

    detached.model_mut().set_drawing_style(style);
    assert_eq!(detached, edited);
    assert!(detached.has_display_projection());
    assert!(detached.can_edit_cdx_atom_label(edit.atom_id));
    assert_eq!(detached.export_cdx()?, WATER_NH);
    Ok(())
}

#[test]
fn implicit_carbon_keeps_preview_and_exact_bytes_with_edit_refusal() -> Result {
    // The independent struct oracle locates this Element on cyanide source Node 7.
    // The following NumHydrogens record remains explicit, and the SDK defaults C.
    assert_eq!(CYANIDE.get(272..278), Some(&[2, 4, 2, 0, 6, 0][..]));
    assert_eq!(CYANIDE.get(278..284), Some(&[43, 4, 2, 0, 0, 0][..]));
    let missing_element = [
        CYANIDE.get(..272).ok_or("missing original prefix")?,
        CYANIDE.get(278..).ok_or("missing original suffix")?,
    ]
    .concat();
    let explicit = read(CYANIDE)?;
    let implicit = read(&missing_element)?;
    let edit = Edit {
        page_id: PageId::new(1).ok_or("invalid page")?,
        fragment_id: FragmentId::new(1).ok_or("invalid fragment")?,
        atom_id: AtomId::new(2).ok_or("invalid atom")?,
        element: Element::try_from(6_u8)?,
        explicit_hydrogens: 0,
    };
    assert_eq!(implicit.model(), explicit.model());
    assert!(explicit.has_cdx_atom_label_preview());
    assert!(implicit.has_cdx_atom_label_preview());
    assert!(explicit.can_edit_cdx_atom_label(edit.atom_id));
    assert!(!implicit.can_edit_cdx_atom_label(edit.atom_id));
    for element in [6_u8, 8] {
        let change = Edit {
            element: Element::try_from(element)?,
            ..edit
        };
        let error = change
            .apply(&implicit)
            .err()
            .ok_or("missing Element was rewritten")?;
        assert_eq!(error.code(), "unsupported_content");
    }
    assert_eq!(implicit.source_info(), source(&missing_element, false));
    assert_eq!(implicit.export_cdx()?, missing_element);
    Ok(())
}
