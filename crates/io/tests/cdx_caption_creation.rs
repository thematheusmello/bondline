//! Independently authored source witnesses for the bounded CDX caption writer.
use bondline_io::{DocumentError, SourceInfo, StoredDocument};
use bondline_model::{PageId, PageSize, Point2, StyledText, TextId, TextRun, TextStyle};
use std::sync::Arc;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const BASELINE: &[u8] = include_bytes!("fixtures/cdx-caption-edit/single_before.cdx");
const LEGACY_EDIT: &[u8] = include_bytes!("fixtures/cdx-caption-edit/single_after.cdx");
const FIRST: &[u8] = include_bytes!("fixtures/cdx-caption-create/first.cdx");
const SPARSE: &[u8] = include_bytes!("fixtures/cdx-caption-create/after_undo.cdx");
const BODY: &str = "Phase β & q";
const X_PT: f64 = 144.25;
const Y_PT: f64 = 216.75;

fn read(raw: &[u8]) -> Result<StoredDocument> {
    Ok(StoredDocument::from_cdx(Arc::from(raw), PageSize::A4)?)
}

fn page_id() -> Result<PageId> {
    PageId::new(1).ok_or_else(|| "positive Page ID".into())
}

fn text_id(id: u32) -> Result<TextId> {
    TextId::new(id).ok_or_else(|| "positive Text ID".into())
}

fn plain_run(body: &str) -> Result<TextRun> {
    let style = TextStyle::new("Roboto".into(), 12.0, 0, [0.0; 3])?;
    Ok(TextRun::new(body.into(), style)?)
}

fn plain_caption(id: u32, x: f64, y: f64) -> Result<StyledText> {
    Ok(StyledText::new(
        text_id(id)?,
        Point2::new(x, y)?,
        vec![plain_run(BODY)?],
    )?)
}

fn edited_baseline() -> Option<SourceInfo> {
    Some(SourceInfo {
        format: "cdx",
        byte_length: BASELINE.len(),
        modified: true,
    })
}

#[test]
fn creation_matches_independent_bytes_for_first_and_after_undo_ids() -> Result {
    let source = read(BASELINE)?;
    let old_model = source.model_snapshot();
    let old_bytes = source.export_cdx()?;
    let page = page_id()?;
    let half_tick = 1.0 / 131_072.0;

    for (id, frozen, offset) in [(2, FIRST, 0.0), (3, SPARSE, half_tick)] {
        // The positive half-ties round away from zero, to the exact frozen baseline.
        let requested = plain_caption(id, X_PT - offset, Y_PT - offset)?;
        let created = source.with_cdx_caption(page, requested)?;
        assert_eq!(created.export_cdx()?, frozen, "Text ID {id}");
        assert_eq!(created.source_info(), edited_baseline());
        assert!(created.has_cdx_caption_preview());
        assert!(!created.is_fully_mapped());
        let page = created.model().pages().first().ok_or("created page")?;
        let created_text = page.texts().last().ok_or("appended caption")?;
        assert_eq!(created_text.as_ref(), &plain_caption(id, X_PT, Y_PT)?);
        assert_eq!(read(frozen)?.model(), created.model());
        assert!(matches!(
            created.to_cdxml(),
            Err(DocumentError::CrossFormatSource {
                from: "cdx",
                to: "cdxml"
            })
        ));
    }
    assert_eq!(source.export_cdx()?, BASELINE);
    assert_eq!(source.export_cdx()?.as_ptr(), old_bytes.as_ptr());
    assert!(Arc::ptr_eq(&source.model_snapshot(), &old_model));
    Ok(())
}

#[test]
fn creation_preserves_all_five_admitted_framings() -> Result {
    use bondline_cdx::{Document, Framing};

    let root = b"\x00\x80\x00\x00\x00\x00";
    let file_end = b"\x00\x00";
    let cases: [(Framing, usize, &[u8], &[u8]); 5] = [
        (Framing::ImplicitDocument, 28, b"", b""),
        (Framing::ExplicitDocument, 28, root, b""),
        (Framing::ImplicitDocumentFileEnd, 28, b"", file_end),
        (Framing::ExplicitDocumentFileEnd, 28, root, file_end),
        (Framing::HeaderDocument, 22, root, file_end),
    ];
    for (framing, header, root, suffix) in cases {
        let frame = |wire: &[u8]| [&wire[..header], root, &wire[28..], suffix].concat();
        let original = frame(BASELINE);
        let expected = frame(FIRST);
        assert_eq!(Document::parse(&original)?.framing(), framing);
        let created =
            read(&original)?.with_cdx_caption(page_id()?, plain_caption(2, X_PT, Y_PT)?)?;
        assert_eq!(created.export_cdx()?, expected);
        assert_eq!(Document::parse(created.export_cdx()?)?.framing(), framing);
        assert_eq!(read(&expected)?.model(), created.model());
    }
    Ok(())
}

#[test]
fn creation_uses_the_current_source_after_an_existing_run_changes() -> Result {
    let original = read(BASELINE)?;
    let page = page_id()?;
    let edited = original.with_cdx_caption_run(page, text_id(1)?, 0, "Bondline λ notes".into())?;
    assert_eq!(edited.export_cdx()?, LEGACY_EDIT);

    let after = edited.with_cdx_caption(page, plain_caption(3, X_PT, Y_PT)?)?;
    // The old edit's independently verified Page ending starts at 157. The complete
    // new child spans 152..264 in the independent marked-caption source.
    let exact = [&LEGACY_EDIT[..157], &SPARSE[152..264], &LEGACY_EDIT[157..]].concat();
    assert_eq!(after.export_cdx()?, exact);
    assert_eq!(after.source_info(), edited_baseline());
    assert_eq!(read(&exact)?.model(), after.model());
    assert_eq!(edited.export_cdx()?, LEGACY_EDIT);
    assert_eq!(original.export_cdx()?, BASELINE);
    Ok(())
}

fn check_refusal(source: &StoredDocument, name: &str, candidate: StyledText) -> Result {
    assert!(
        source.has_cdx_caption_preview(),
        "invalid premise for {name}"
    );
    let old_model = source.model_snapshot();
    let old_bytes = source.export_cdx()?;
    let old_info = source.source_info();
    assert!(
        source.with_cdx_caption(page_id()?, candidate).is_err(),
        "{name} was accepted"
    );
    assert!(Arc::ptr_eq(&source.model_snapshot(), &old_model), "{name}");
    assert_eq!(source.export_cdx()?.as_ptr(), old_bytes.as_ptr(), "{name}");
    assert_eq!(source.source_info(), old_info, "{name}");
    assert!(source.has_cdx_caption_preview(), "{name}");
    Ok(())
}

#[test]
fn creation_refuses_duplicate_ids_invalid_styles_and_out_of_paper_baselines_atomically() -> Result {
    let source = read(BASELINE)?;
    check_refusal(&source, "duplicate live ID", plain_caption(1, X_PT, Y_PT)?)?;
    let quarter_tick = 1.0 / 262_144.0;
    check_refusal(
        &source,
        "baseline rounded onto the left paper edge",
        plain_caption(2, quarter_tick, Y_PT)?,
    )?;
    check_refusal(
        &source,
        "caption ink outside paper",
        plain_caption(2, 510.0, Y_PT)?,
    )?;

    let point = Point2::new(X_PT, Y_PT)?;
    for (name, font, size, face, rgb) in [
        ("font", "Other", 12.0, 0, [0.0; 3]),
        ("size", "Roboto", 11.0, 0, [0.0; 3]),
        ("face", "Roboto", 12.0, 1, [0.0; 3]),
        ("colour", "Roboto", 12.0, 0, [1.0, 0.0, 0.0]),
    ] {
        let style = TextStyle::new(font.into(), size, face, rgb)?;
        let run = TextRun::new(BODY.into(), style)?;
        check_refusal(
            &source,
            name,
            StyledText::new(text_id(2)?, point, vec![run])?,
        )?;
    }
    let two_runs = StyledText::new(text_id(2)?, point, vec![plain_run("H")?, plain_run("g")?])?;
    check_refusal(&source, "two runs", two_runs)?;
    let blank = StyledText::new(text_id(2)?, point, vec![plain_run("   ")?])?;
    check_refusal(&source, "blank caption", blank)?;
    Ok(())
}

#[test]
fn creation_refuses_raw_object_id_and_signed_depth_exhaustion_atomically() -> Result {
    let mut max_object_id = BASELINE.to_vec();
    // The literal Page opening is at 52, with its document-wide maximum ID at 54.
    assert_eq!(&max_object_id[52..58], b"\x01\x80\x5a\0\0\0");
    max_object_id[54..58].copy_from_slice(&u32::MAX.to_le_bytes());
    let source = read(&max_object_id)?;
    assert_eq!(source.export_cdx()?, max_object_id);
    check_refusal(
        &source,
        "raw object ID exhaustion",
        plain_caption(2, X_PT, Y_PT)?,
    )?;

    let mut max_depth = SPARSE.to_vec();
    // The new Text's explicit ZOrder property is at 175, with its signed value at 179.
    assert_eq!(&max_depth[175..181], b"\x0a\0\x02\0\x01\0");
    max_depth[179..181].copy_from_slice(&i16::MAX.to_le_bytes());
    let source = read(&max_depth)?;
    assert_eq!(source.export_cdx()?, max_depth);
    check_refusal(&source, "ZOrder exhaustion", plain_caption(4, X_PT, 300.0)?)?;
    Ok(())
}

fn expected_chemical_caption(source: &[u8], page_end: usize, depth: i16, id: u8) -> Vec<u8> {
    // The independent child already encodes these coordinates, UTF-8 body, font and marker.
    let mut child = FIRST[152..264].to_vec();
    child[2..6].copy_from_slice(&903_u32.to_le_bytes()); // highest existing Fragment is 902
    child[27..29].copy_from_slice(&depth.to_le_bytes());
    assert_eq!(&child[105..108], b"1:2");
    child[107] = b'0' + id;
    [&source[..page_end], &child, &source[page_end..]].concat()
}

#[test]
fn creation_above_chemical_ink_preserves_scoped_ids_and_all_original_records() -> Result {
    let boxed = include_bytes!("fixtures/cdx-label-preview/boxed.cdx").as_slice();
    let overlap = include_bytes!("fixtures/cdx-mixed-caption/overlap_before.cdx").as_slice();
    for (original, end, depth, id) in [(boxed, 316, 1_i16, 1_u8), (overlap, 364, 8, 2)] {
        let source = read(original)?;
        let page = page_id()?;
        let mut expected_model = source.model().clone();
        let caption = plain_caption(u32::from(id), X_PT, Y_PT)?;
        expected_model.append_texts(page, vec![caption.clone()])?;
        let created = source.with_cdx_caption(page, caption)?;

        let expected = expected_chemical_caption(original, end, depth, id);
        assert_eq!(created.export_cdx()?, expected);
        assert_eq!(created.model(), &expected_model);
        assert!(created.has_cdx_mixed_preview() && created.can_edit_cdx_caption_run());
        assert!(!created.has_cdx_atom_label_preview() && !created.is_fully_mapped());
        assert_eq!(read(&expected)?.model(), &expected_model);
        assert_eq!(
            created.source_info().ok_or("source")?.byte_length,
            original.len()
        );
        // A transitioned history entry owns the mixed proof for subsequent edits.
        let edited =
            created.with_cdx_caption_run(page, text_id(u32::from(id))?, 0, "Note".into())?;
        assert!(edited.has_cdx_mixed_preview());
        assert_eq!(read(edited.export_cdx()?)?.model(), edited.model());
        assert_eq!(created.export_cdx()?, expected);
        assert_eq!(source.export_cdx()?, original);
        assert_eq!(source.has_cdx_atom_label_preview(), id == 1);
    }
    Ok(())
}

#[test]
fn first_caption_preserves_a_prior_owned_atom_label_edit_and_its_original_source() -> Result {
    use bondline_model::{AtomId, Element, FragmentId};
    let raw = include_bytes!("fixtures/cdx-label-preview/boxed_reordered.cdx");
    let labelled = include_bytes!("fixtures/cdx-label-edit/boxed_nh2_to_oh.cdx");
    let original = read(raw)?;
    let page = page_id()?;
    let atom = AtomId::new(1).ok_or("atom")?;
    let edited = original.with_cdx_atom_label(
        page,
        FragmentId::new(1).ok_or("fragment")?,
        atom,
        Element::new(8).ok_or("oxygen")?,
        1,
    )?;
    assert_eq!(edited.export_cdx()?, labelled);
    let with_text = edited.with_cdx_caption(page, plain_caption(1, X_PT, Y_PT)?)?;
    assert_eq!(
        with_text.export_cdx()?,
        expected_chemical_caption(labelled, 277, 1, 1)
    );
    assert!(with_text.has_cdx_mixed_preview());
    assert!(with_text.can_edit_cdx_atom_label(atom));
    assert!(edited.can_edit_cdx_atom_label(atom));
    assert_eq!(
        with_text.source_info().ok_or("provenance")?.byte_length,
        raw.len()
    );
    assert_eq!(edited.export_cdx()?, labelled);
    assert_eq!(original.export_cdx()?, raw);
    Ok(())
}

#[test]
fn mixed_creation_refuses_raw_id_and_front_depth_exhaustion_without_modifying_the_source() -> Result
{
    let max_object = include_bytes!("fixtures/cdx-mixed-caption/boxed_before.cdx").as_slice();
    let mut max_depth = include_bytes!("fixtures/cdx-mixed-caption/overlap_before.cdx").to_vec();
    assert_eq!(&max_depth[155..157], &7_i16.to_le_bytes());
    max_depth[155..157].copy_from_slice(&i16::MAX.to_le_bytes());
    let edited_at_max = include_bytes!("fixtures/cdx-mixed-label-edit/depth_max_oh.cdx");
    for (raw, reason, next_id) in [
        (max_object, "an available raw object ID", 2),
        (max_depth.as_slice(), "an available caption depth", 2),
        (edited_at_max.as_slice(), "an available caption depth", 21),
    ] {
        let source = read(raw)?;
        assert!(source.has_cdx_mixed_preview());
        let result = source.with_cdx_caption(page_id()?, plain_caption(next_id, X_PT, Y_PT)?);
        assert!(
            matches!(result, Err(DocumentError::UnsupportedCdxCaption(message)) if message == reason)
        );
        assert_eq!(source.export_cdx()?, raw);
        assert!(!source.source_info().ok_or("source")?.modified);
    }
    Ok(())
}
