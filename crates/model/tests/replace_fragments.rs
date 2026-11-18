//! Independent atomic multi-fragment replacement for the selection move candidate.
use bondline_model::{Atom, AtomId, Document, Element, Fragment, FragmentId, PageId, Point2};
use std::sync::Arc;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn fragment(id: u32, atoms: &[(u32, f64, f64)]) -> TestResult<Fragment> {
    Ok(Fragment::new(
        FragmentId::new(id).ok_or("fragment identity")?,
        atoms
            .iter()
            .map(|&(id, x, y)| {
                Ok(Atom::new(
                    AtomId::new(id).ok_or("atom identity")?,
                    Element::new(6).ok_or("carbon")?,
                    Some(Point2::new(x, y)?),
                ))
            })
            .collect::<TestResult<Vec<_>>>()?,
        vec![],
    )?)
}

#[test]
fn batch_replacement_keeps_page_order_and_unrelated_arc_and_rejects_every_partial_candidate()
-> TestResult {
    let page = PageId::new(1).ok_or("page")?;
    let mut original = Document::default();
    original.append_fragments(
        page,
        vec![
            fragment(17, &[(29, 72.0, 72.0), (11, 120.0, 72.0)])?,
            fragment(7, &[(91, 500.0, 500.0)])?,
            fragment(31, &[(401, 220.0, 72.0), (407, 268.0, 72.0)])?,
        ],
    )?;
    let first = fragment(17, &[(29, 96.0, 60.0), (11, 144.0, 60.0)])?;
    let last = fragment(31, &[(401, 244.0, 60.0), (407, 292.0, 60.0)])?;
    let unrelated = Arc::clone(&original.pages()[0].fragments()[1]);
    let mut replaced = original.clone();
    replaced.replace_fragments(page, vec![last.clone(), first.clone()])?;
    let result = replaced.pages()[0].fragments();
    assert_eq!(
        result.iter().map(|f| f.id().get()).collect::<Vec<_>>(),
        [17, 7, 31]
    );
    assert_eq!(result[0].as_ref(), &first);
    assert_eq!(result[2].as_ref(), &last);
    assert!(Arc::ptr_eq(&result[1], &unrelated));
    assert_eq!(
        original.pages()[0].fragments()[0].atoms()[0].position,
        Some(Point2::new(72.0, 72.0)?),
        "the original candidate owner stays immutable"
    );
    let missing = fragment(99, &[(900, 300.0, 400.0)])?;
    let collision_between_replacements = fragment(31, &[(29, 244.0, 60.0)])?;
    let collision_with_retained = fragment(17, &[(91, 96.0, 60.0)])?;
    for invalid in [
        vec![first.clone(), missing],
        vec![first.clone(), first.clone()],
        vec![first.clone(), collision_between_replacements],
        vec![collision_with_retained, last],
    ] {
        let mut document = original.clone();
        let arcs = document.pages()[0].fragments().to_vec();
        assert!(document.replace_fragments(page, invalid).is_err());
        assert_eq!(
            document, original,
            "a rejected batch must not apply an earlier replacement"
        );
        for (before, after) in arcs.iter().zip(document.pages()[0].fragments()) {
            assert!(Arc::ptr_eq(before, after));
        }
    }
    Ok(())
}
