//! Certified coordinate edits retain source identity across sub-grid and off-paper requests.
use bondline_io::StoredDocument;
use bondline_model::{PageId, PageSize};
use std::sync::Arc;

#[test]
fn apply_one_quantized_displacement_to_every_selected_atom()
-> Result<(), Box<dyn std::error::Error>> {
    let raw = include_bytes!("fixtures/cdx-skeletal-move/aliased_extended.cdx");
    let stored = StoredDocument::from_cdx(Arc::from(raw.as_slice()), PageSize::A4)?;
    let page = PageId::new(1).ok_or("page id")?;
    let expected = include_bytes!("fixtures/cdx-skeletal-move/aliased_moved.cdx");
    let first = stored.with_cdx_skeletal_translation(page, &[2, 4], -5.125, 3.25)?;
    assert_eq!(first.to_cdx()?.as_ref(), expected);
    let shifted =
        first.with_cdx_skeletal_translation(page, &[2, 4], 1.5 / 65536., -1.5 / 65536.)?;
    let a = first.model().pages()[0].fragments();
    let b = shifted.model().pages()[0].fragments();
    for (original, moved) in a.iter().zip(b) {
        for (p, q) in original.atoms().iter().zip(moved.atoms()) {
            let selected = [2, 4].contains(&p.id.get());
            let (p, q) = (
                p.position.ok_or("old point")?,
                q.position.ok_or("new point")?,
            );
            assert_eq!(
                q.x_pt() - p.x_pt(),
                if selected { 2.0 / 65536. } else { 0. }
            );
            assert_eq!(
                q.y_pt() - p.y_pt(),
                if selected { -2.0 / 65536. } else { 0. }
            );
        }
    }
    let back = shifted
        .with_cdx_skeletal_translation(page, &[2, 4], -1.5 / 65536., 1.5 / 65536.)?
        .with_cdx_skeletal_translation(page, &[2, 4], 5.125, -3.25)?;
    assert_eq!(back.to_cdx()?.as_ref(), raw);
    assert!(Arc::ptr_eq(
        &back.model_snapshot(),
        &stored.model_snapshot()
    ));
    for ids in [&[][..], &[0], &[2, 2], &[4, 2], &[88]] {
        assert!(
            stored
                .with_cdx_skeletal_translation(page, ids, 0., 0.)
                .is_err()
        );
    }
    let odd = include_bytes!("fixtures/cdx-skeletal-move/odd_position.cdx");
    let odd_moved = include_bytes!("fixtures/cdx-skeletal-move/odd_position_halves.cdx");
    let odd_source = StoredDocument::from_cdx(Arc::from(odd.as_slice()), PageSize::A4)?;
    let no_op =
        odd_source.with_cdx_skeletal_translation(page, &[2, 4], 0.5 / 65536., -0.5 / 65536.)?;
    assert!(Arc::ptr_eq(
        &no_op.model_snapshot(),
        &odd_source.model_snapshot()
    ));
    let rigid =
        odd_source.with_cdx_skeletal_translation(page, &[2, 4], 1.5 / 65536., -1.5 / 65536.)?;
    assert_eq!(rigid.to_cdx()?.as_ref(), odd_moved);
    let mut detached = stored.clone();
    detached.model_mut().set_page_size(page, PageSize::A4)?;
    assert!(
        detached
            .with_cdx_skeletal_translation(page, &[2], 1., 0.)
            .is_err()
    );
    Ok(())
}
