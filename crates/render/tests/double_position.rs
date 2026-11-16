//! Data/format prerequisite: no inset geometry or aromaticity is claimed here.
use bondline_model::{
    Atom, AtomId, Bond, BondId, Document, Element, Fragment, FragmentId, Page, PageId, PageSize,
    Point2,
};
use bondline_render::{
    Cancellation, Error, GraphScene, HitTarget, PageView, Primitive, Stroke, cpu_scene, gpu_scene,
    graph_page_scene, graph_page_scene_with_cancel, graph_page_scene_with_display_hydrogens,
};
use serde_json::json;
use std::collections::BTreeMap;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn page(order: &str, position: Option<&str>, reversed: bool) -> TestResult<Page> {
    let h = match order {
        "single" => 3,
        "double" => 2,
        "triple" => 1,
        _ => return Err("order".into()),
    };
    let atoms = [(29, 32.0), (11, 96.0)]
        .into_iter()
        .map(|(id, x)| {
            let mut atom = Atom::new(
                AtomId::new(id).ok_or("atom")?,
                Element::try_from(6)?,
                Some(Point2::new(x, 64.0)?),
            );
            atom.explicit_hydrogens = Some(h);
            Ok(atom)
        })
        .collect::<TestResult<Vec<_>>>()?;
    let (begin, end) = if reversed { (11, 29) } else { (29, 11) };
    let mut literal = json!({"id":41,"begin":begin,"end":end,"order":order});
    if let Some(position) = position {
        literal["double_position"] = json!(position);
    }
    // No new Rust API is needed: after model support, this reaches the renderer guard.
    let bond: Bond = serde_json::from_value(literal)?;
    let mut document = Document::default();
    let id = PageId::new(1).ok_or("page")?;
    document.set_page_size(id, PageSize::new(256.0, 256.0)?)?;
    document.append_fragments(
        id,
        vec![Fragment::new(
            FragmentId::new(17).ok_or("fragment")?,
            atoms,
            vec![bond],
        )?],
    )?;
    Ok(document.pages().first().ok_or("page")?.clone())
}

fn unsupported(result: Result<GraphScene, Error>) -> TestResult {
    let error = result
        .err()
        .ok_or("unsupported placement silently rendered a scene")?;
    assert!(
        matches!(error, Error::UnsupportedChemicalContent),
        "{error}"
    );
    Ok(())
}

#[test]
fn left_and_right_double_metadata_refuse_all_shared_scene_entries_without_mutation() -> TestResult {
    for position in ["left", "right"] {
        for reversed in [false, true] {
            let page = page("double", Some(position), reversed)?;
            let before = page.clone();
            let serialized = serde_json::to_value(&page)?;
            unsupported(graph_page_scene(576, 576, &page))?;
            let counts = BTreeMap::from([
                (AtomId::new(29).ok_or("atom")?, 2),
                (AtomId::new(11).ok_or("atom")?, 2),
            ]);
            unsupported(graph_page_scene_with_display_hydrogens(
                576,
                576,
                &page,
                &counts,
                &Cancellation::default(),
            ))?;
            let view = PageView::fit(576, 576, page.size())?;
            unsupported(view.graph_scene(&page, &Cancellation::default()))?;
            assert_eq!(page, before);
            assert_eq!(serde_json::to_value(&page)?, serialized);
        }
    }
    Ok(())
}

#[test]
fn every_specified_position_on_single_or_triple_refuses_instead_of_losing_dormant_data()
-> TestResult {
    for order in ["single", "triple"] {
        let old = page(order, None, false)?;
        graph_page_scene(576, 576, &old)?;
        for position in ["center", "left", "right"] {
            let page = page(order, Some(position), false)?;
            let before = serde_json::to_value(&page)?;
            unsupported(graph_page_scene(576, 576, &page))?;
            assert_eq!(serde_json::to_value(&page)?, before);
            assert_eq!(
                before["fragments"][0]["bonds"][0]["double_position"],
                position
            );
        }
    }
    Ok(())
}

#[test]
fn explicit_center_double_keeps_the_sealed_centered_components_pixels_and_hits() -> TestResult {
    let missing = page("double", None, false)?;
    let centered = page("double", Some("center"), false)?;
    let missing_before = serde_json::to_value(&missing)?;
    let centered_before = serde_json::to_value(&centered)?;
    let base = graph_page_scene(576, 576, &missing)?;
    let actual = graph_page_scene(576, 576, &centered)?;
    assert_eq!(actual.scene(), base.scene());
    let explicit_unspecified = page("double", Some("unspecified"), false)?;
    assert_eq!(serde_json::to_value(&explicit_unspecified)?, missing_before);
    assert_eq!(
        graph_page_scene(576, 576, &explicit_unspecified)?.scene(),
        base.scene()
    );
    let expected = [157.0, 163.0].map(|y| Stroke {
        begin: [96.0, y],
        end: [224.0, y],
        width: 2.0,
        color: [0, 0, 0, 255],
    });
    let strokes = actual
        .scene()
        .primitives
        .iter()
        .filter_map(|p| match p {
            Primitive::Stroke(s) => Some(s.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(strokes, expected);
    let fid = FragmentId::new(17).ok_or("fragment")?;
    let bid = BondId::new(41).ok_or("bond")?;
    assert_eq!(
        actual.bond_strokes(fid, bid).cloned().collect::<Vec<_>>(),
        expected
    );
    for y in [157.0, 163.0] {
        assert_eq!(
            actual.hit_test([160.0, y], 0.0)?,
            Some(HitTarget::Bond {
                fragment_id: fid,
                bond_id: bid
            })
        );
    }
    assert_eq!(actual.hit_test([160.0, 160.0], 0.0)?, None);
    let cpu = cpu_scene(actual.scene())?;
    assert_eq!(cpu, cpu_scene(base.scene())?);
    // Analytic middle strips are far from the authored endpoint glyphs.
    for y in 150_u32..171 {
        for x in 128_u32..192 {
            let offset = usize::try_from((u64::from(y) * 576 + u64::from(x)) * 4)?;
            let expected = if [157.0, 163.0]
                .iter()
                .any(|row| (f64::from(y) + 0.5 - row).abs() <= 1.0)
            {
                &[0, 0, 0, 255][..]
            } else {
                &[255, 255, 255, 255][..]
            };
            assert_eq!(
                cpu.rgba.get(offset..offset + 4).ok_or("RGBA")?,
                expected,
                "pixel{x},{y}"
            );
        }
    }
    assert_eq!(pollster::block_on(gpu_scene(actual.scene()))?.0, cpu);
    let view = PageView::fit(576, 576, centered.size())?;
    assert_eq!(
        view.graph_scene(&centered, &Cancellation::default())?
            .scene(),
        actual.scene()
    );
    assert_eq!(serde_json::to_value(&missing)?, missing_before);
    assert_eq!(serde_json::to_value(&centered)?, centered_before);
    Ok(())
}

#[test]
fn cancellation_still_precedes_unsupported_placement() -> TestResult {
    let page = page("double", Some("right"), false)?;
    let before = serde_json::to_value(&page)?;
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(matches!(
        graph_page_scene_with_cancel(576, 576, &page, &cancel),
        Err(Error::Cancelled)
    ));
    assert_eq!(serde_json::to_value(&page)?, before);
    Ok(())
}
