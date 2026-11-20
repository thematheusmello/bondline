//! Offline browser IO regression. Run `python3 xtask/oracles/cdx_caption_browser.py`.
//! Sparse IDs are supplied directly to the codec. Command history has its own tests.
use std::{error::Error, sync::Arc};

use bondline_io::{DocumentError, StoredDocument};
use bondline_model::{
    AtomId, Element, FragmentId, PageId, PageSize, Point2, StyledText, TextId, TextRun, TextStyle,
};

type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

const CHEMICAL: &[u8] = include_bytes!("../tests/fixtures/cdx-label-preview/boxed.cdx");
const MIXED: &[u8] = include_bytes!("../tests/fixtures/cdx-mixed-caption-edit/before.cdx");
const EXHAUSTED: &[u8] = include_bytes!("../tests/fixtures/cdx-mixed-caption/boxed_after.cdx");
const BODY: &str = "Phase β & q";

fn append(source: &StoredDocument, id: u32) -> Result<StoredDocument> {
    let style = TextStyle::new("Roboto".into(), 12.0, 0, [0.0; 3])?;
    let caption = StyledText::new(
        TextId::new(id).ok_or("expected a nonzero caption ID")?,
        Point2::new(144.25, 216.75)?,
        vec![TextRun::new(BODY.into(), style)?],
    )?;
    Ok(source.with_cdx_caption(PageId::new(1).ok_or("missing page ID")?, caption)?)
}

fn verify(before: &[u8], expected: &[u8], old_ids: &[u32], id: u32) -> Result<StoredDocument> {
    let source = StoredDocument::from_cdx(Arc::from(before), PageSize::A4)?;
    let [original] = source.model().pages() else {
        return Err("expected a single original page".into());
    };
    let created = append(&source, id)?;
    if source.export_cdx()? != before || created.export_cdx()? != expected {
        return Err("created bytes differ from the independent fixture or source changed".into());
    }
    let reopened = StoredDocument::from_cdx(Arc::from(created.export_cdx()?), PageSize::A4)?;
    if reopened.model() != created.model()
        || !reopened.has_cdx_mixed_preview()
        || reopened.export_cdx()? != expected
    {
        return Err("mixed page failed preserving reopen".into());
    }
    let [page] = reopened.model().pages() else {
        return Err("expected one reopened page".into());
    };
    let (caption, old) = page.texts().split_last().ok_or("missing new caption")?;
    if original.fragments().len() != 2
        || page.fragments() != original.fragments()
        || old != original.texts()
        || !old.iter().map(|t| t.id().get()).eq(old_ids.iter().copied())
        || caption.id().get() != id
        || !matches!(caption.runs(), [run] if run.text() == BODY)
    {
        return Err("reopened graph, source captions, new ID or text differs".into());
    }
    Ok(reopened)
}

fn verify_label(
    source: &StoredDocument,
    owner: (u32, u32),
    label: (u8, u16, &str),
    expected: &[u8],
) -> Result<StoredDocument> {
    let (atomic_number, hydrogens, body) = label;
    let element = Element::new(atomic_number).ok_or("invalid element")?;
    let fragment_id = FragmentId::new(owner.0).ok_or("invalid fragment ID")?;
    let atom_id = AtomId::new(owner.1).ok_or("invalid atom ID")?;
    let [original] = source.model().pages() else {
        return Err("expected one input page".into());
    };
    let change =
        source.with_cdx_atom_label(original.id(), fragment_id, atom_id, element, hydrogens)?;
    let bytes = change.export_cdx()?;
    if bytes != expected {
        return Err("label edit differs from independent CDX".into());
    }
    let reopened = StoredDocument::from_cdx(Arc::from(bytes), PageSize::A4)?;
    let [page] = reopened.model().pages() else {
        return Err("expected one reopened page".into());
    };
    let target = page
        .fragments()
        .iter()
        .find(|f| f.id() == fragment_id)
        .and_then(|f| f.atoms().iter().find(|a| a.id == atom_id))
        .ok_or("missing target atom")?;
    let unaffected = |f: &&Arc<bondline_model::Fragment>| f.id() != fragment_id;
    if reopened.model() != change.model()
        || !reopened.has_cdx_mixed_preview()
        || reopened.export_cdx()? != expected
        || page.texts() != original.texts()
        || page
            .fragments()
            .iter()
            .filter(unaffected)
            .ne(original.fragments().iter().filter(unaffected))
        || target.element != element
        || target.explicit_hydrogens != Some(hydrogens)
        || !matches!(target.authored_label.as_ref().map(|l| l.runs()), Some([run]) if run.text() == body)
    {
        return Err("reopen changed the selected label, peer fragment or page caption".into());
    }
    Ok(reopened)
}

fn main() -> Result {
    let chemical = verify(
        CHEMICAL,
        include_bytes!("../tests/fixtures/cdx-chemical-caption-create/chemical_first.cdx"),
        &[],
        1,
    )?;
    verify(
        CHEMICAL,
        include_bytes!("../tests/fixtures/cdx-chemical-caption-create/chemical_after_undo.cdx"),
        &[],
        2,
    )?;
    let mixed = verify(
        MIXED,
        include_bytes!("../tests/fixtures/cdx-chemical-caption-create/mixed_first.cdx"),
        &[1, 19],
        20,
    )?;
    verify(
        MIXED,
        include_bytes!("../tests/fixtures/cdx-chemical-caption-create/mixed_after_undo.cdx"),
        &[1, 19],
        21,
    )?;
    verify_label(
        &chemical,
        (1, 1),
        (8, 1, "OH"),
        include_bytes!("../tests/fixtures/cdx-mixed-label-edit/chemical_first_oh.cdx"),
    )?;
    let oh = verify_label(
        &mixed,
        (1, 1),
        (8, 1, "OH"),
        include_bytes!("../tests/fixtures/cdx-mixed-label-edit/mixed_first_oh.cdx"),
    )?;
    verify_label(
        &oh,
        (2, 2),
        (35, 0, "Br"),
        include_bytes!("../tests/fixtures/cdx-mixed-label-edit/mixed_first_both.cdx"),
    )?;
    let exhausted = StoredDocument::from_cdx(Arc::from(EXHAUSTED), PageSize::A4)?;
    if !exhausted.has_cdx_mixed_preview() {
        return Err("raw-ID refusal requires a complete mixed source".into());
    }
    let error = append(&exhausted, 2)
        .err()
        .ok_or("raw ID exhaustion accepted")?;
    if !matches!(
        error.downcast_ref::<DocumentError>(),
        Some(DocumentError::UnsupportedCdxCaption(
            "an available raw object ID"
        ))
    ) || exhausted.export_cdx()? != EXHAUSTED
    {
        return Err("raw ID exhaustion changed the source or failed for another reason".into());
    }
    Ok(())
}
