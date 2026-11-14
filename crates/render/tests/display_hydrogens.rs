use bondline_model::{
    Atom, AtomId, Document, Element, Fragment, FragmentId, Page, PageId, PageSize, Point2,
};
use bondline_render::{Cancellation, Error, graph_page_scene_with_display_hydrogens};
use std::collections::BTreeMap;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn isolated(h: Option<u16>, position: [f64; 2]) -> TestResult<Page> {
    let mut atom = Atom::new(
        AtomId::new(7).ok_or("atom")?,
        Element::try_from(8)?,
        Some(Point2::new(position[0], position[1])?),
    );
    atom.explicit_hydrogens = h;
    let mut document = Document::default();
    let id = PageId::new(1).ok_or("page")?;
    document.set_page_size(id, PageSize::new(256.0, 256.0)?)?;
    document.append_fragments(
        id,
        vec![Fragment::new(
            FragmentId::new(9).ok_or("fragment")?,
            vec![atom],
            vec![],
        )?],
    )?;
    Ok(document.pages().first().ok_or("page")?.clone())
}

#[test]
fn display_h_requires_exact_coverage_and_agreement_without_authoring_unknown_h() -> TestResult {
    let page = isolated(None, [128.0, 128.0])?;
    let before = page.clone();
    let id = AtomId::new(7).ok_or("atom")?;
    let counts = BTreeMap::from([(id, 1)]);
    let cancel = Cancellation::default();
    assert!(graph_page_scene_with_display_hydrogens(576, 576, &page, &counts, &cancel).is_ok());
    assert_eq!(page, before);
    assert_eq!(page.fragments()[0].atoms()[0].explicit_hydrogens, None);
    for bad in [
        BTreeMap::new(),
        BTreeMap::from([(AtomId::new(8).ok_or("extra")?, 1)]),
        BTreeMap::from([(id, 1), (AtomId::new(8).ok_or("extra")?, 0)]),
    ] {
        assert!(matches!(
            graph_page_scene_with_display_hydrogens(576, 576, &page, &bad, &cancel),
            Err(Error::UnsupportedChemicalContent)
        ));
    }
    for authored in [0, 1, 12, u16::MAX] {
        let page = isolated(Some(authored), [128.0, 128.0])?;
        assert!(
            graph_page_scene_with_display_hydrogens(
                576,
                576,
                &page,
                &BTreeMap::from([(id, authored)]),
                &cancel
            )
            .is_ok()
        );
        assert!(matches!(
            graph_page_scene_with_display_hydrogens(
                576,
                576,
                &page,
                &BTreeMap::from([(id, authored.wrapping_add(1))]),
                &cancel
            ),
            Err(Error::UnsupportedChemicalContent)
        ));
    }
    cancel.cancel();
    assert!(matches!(
        graph_page_scene_with_display_hydrogens(576, 576, &page, &counts, &cancel),
        Err(Error::Cancelled)
    ));
    Ok(())
}

#[test]
fn display_ink_must_fit_the_paper_and_not_merely_the_viewport() -> TestResult {
    let id = AtomId::new(7).ok_or("atom")?;
    for position in [[0.0, 128.0], [256.0, 128.0], [128.0, 0.0], [128.0, 256.0]] {
        let page = isolated(None, position)?;
        assert!(matches!(
            graph_page_scene_with_display_hydrogens(
                576,
                576,
                &page,
                &BTreeMap::from([(id, 1)]),
                &Cancellation::default()
            ),
            Err(Error::LabelPlacement)
        ));
    }
    Ok(())
}

#[test]
fn display_mode_retains_physical_h_fragment_ids_and_existing_resource_guards() -> TestResult {
    let cancel = Cancellation::default();
    let mut document = Document::default();
    let page = PageId::new(1).ok_or("page")?;
    document.set_page_size(page, PageSize::new(256.0, 256.0)?)?;
    let mut counts = BTreeMap::new();
    for (fragment, id, z, x, h) in [
        (3, 7, 1, 64.0, 0),
        (9, 11, 7, 128.0, 2),
        (4, 19, 17, 192.0, 0),
    ] {
        let id = AtomId::new(id).ok_or("id")?;
        let atom = Atom::new(id, Element::try_from(z)?, Some(Point2::new(x, 128.0)?));
        counts.insert(id, h);
        document.append_fragments(
            page,
            vec![Fragment::new(
                FragmentId::new(fragment).ok_or("fragment")?,
                vec![atom],
                vec![],
            )?],
        )?;
    }
    let page = document.pages().first().ok_or("page")?;
    let graph = graph_page_scene_with_display_hydrogens(576, 576, page, &counts, &cancel)?;
    for (fragment, id, x) in [(3, 7, 160.0), (9, 11, 288.0), (4, 19, 416.0)] {
        assert_eq!(
            graph.hit_test([x, 288.0], 0.0)?,
            Some(bondline_render::HitTarget::Atom {
                fragment_id: FragmentId::new(fragment).ok_or("fragment")?,
                atom_id: AtomId::new(id).ok_or("atom")?
            })
        );
    }
    counts.insert(AtomId::new(7).ok_or("id")?, 1);
    assert!(matches!(
        graph_page_scene_with_display_hydrogens(576, 576, page, &counts, &cancel),
        Err(Error::UnsupportedChemicalContent)
    ));
    // These valid model items cross the existing hit-region preflight, before
    // attempting glyphs, overlap work or a large canvas allocation.
    let mut document = Document::default();
    let id = PageId::new(1).ok_or("page")?;
    let mut counts = BTreeMap::new();
    let atoms = (1..=4097)
        .map(|n| {
            let id = AtomId::new(n).ok_or("id")?;
            counts.insert(id, 0);
            Ok(Atom::new(
                id,
                Element::try_from(7)?,
                Some(Point2::new(f64::from(n), 128.0)?),
            ))
        })
        .collect::<TestResult<Vec<_>>>()?;
    document.append_fragments(
        id,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            atoms,
            vec![],
        )?],
    )?;
    assert!(matches!(
        graph_page_scene_with_display_hydrogens(
            576,
            576,
            document.pages().first().ok_or("page")?,
            &counts,
            &cancel
        ),
        Err(Error::SceneTooComplex)
    ));
    let mut document = Document::default();
    document.set_page_size(id, PageSize::new(1024.0, 1024.0)?)?;
    let mut counts = BTreeMap::new();
    let atoms = (1..=300)
        .map(|n| {
            let id = AtomId::new(n).ok_or("id")?;
            counts.insert(id, 0);
            Ok(Atom::new(
                id,
                Element::try_from(7)?,
                Some(Point2::new(
                    32.0 + f64::from((n - 1) % 15) * 48.0,
                    32.0 + f64::from((n - 1) / 15) * 48.0,
                )?),
            ))
        })
        .collect::<TestResult<Vec<_>>>()?;
    document.append_fragments(
        id,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            atoms,
            vec![],
        )?],
    )?;
    assert!(
        matches!(
            graph_page_scene_with_display_hydrogens(
                2048,
                2048,
                document.pages().first().ok_or("page")?,
                &counts,
                &cancel
            ),
            Err(Error::SceneTooComplex)
        ),
        "300 separated N glyphs must still respect the primitive budget"
    );
    let empty = Document::default();
    assert_eq!(
        graph_page_scene_with_display_hydrogens(
            576,
            576,
            empty.pages().first().ok_or("page")?,
            &BTreeMap::new(),
            &cancel
        )?
        .scene(),
        &bondline_render::page_scene(576, 576, empty.pages().first().ok_or("page")?.size())?
    );
    Ok(())
}
