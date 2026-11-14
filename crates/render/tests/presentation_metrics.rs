use bondline_model::{
    Atom, AtomId, Document, Element, Fragment, FragmentId, PageId, PageSize, Point2,
};
use bondline_render::{Cancellation, graph_page_scene, graph_page_scene_with_display_hydrogens};
use std::collections::BTreeMap;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn actual_scene_reports_label_em_for_both_paths_and_none_for_empty_pages() -> TestResult {
    let mut document = Document::default();
    let page_id = PageId::new(1).ok_or("page")?;
    document.set_page_size(page_id, PageSize::new(256.0, 256.0)?)?;
    let empty = graph_page_scene(576, 576, document.pages().first().ok_or("page")?)?;
    assert_eq!(empty.label_em_px(), None);
    let atom_id = AtomId::new(7).ok_or("atom")?;
    let mut atom = Atom::new(
        atom_id,
        Element::new(8).ok_or("oxygen")?,
        Some(Point2::new(128.0, 128.0)?),
    );
    atom.explicit_hydrogens = Some(1);
    document.append_fragments(
        page_id,
        vec![Fragment::new(
            FragmentId::new(3).ok_or("fragment")?,
            vec![atom],
            vec![],
        )?],
    )?;
    let page = document.pages().first().ok_or("page")?;
    //512 pixels of paper for256 authored points, with the original12pt label style.
    assert_eq!(graph_page_scene(576, 576, page)?.label_em_px(), Some(24.0));
    let resolved = graph_page_scene_with_display_hydrogens(
        576,
        576,
        page,
        &BTreeMap::from([(atom_id, 1)]),
        &Cancellation::default(),
    )?;
    assert_eq!(resolved.label_em_px(), Some(24.0));
    //The fact follows the actual viewport, not a hard-coded A4 size or display DPI.
    assert_eq!(
        graph_page_scene(1088, 1088, page)?.label_em_px(),
        Some(48.0)
    );
    Ok(())
}
