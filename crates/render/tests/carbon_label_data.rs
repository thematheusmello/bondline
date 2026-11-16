//! Temporary data-prerequisite guard; no skeletal suppression or H inference.
use bondline_model::{Atom, AtomId, Bond, Document, Fragment, FragmentId, Page, PageId, PageSize};
use bondline_render::{
    Cancellation, Error, GraphScene, HitTarget, PageView, Primitive, Stroke, cpu_scene, gpu_scene,
    graph_page_scene, graph_page_scene_with_cancel, graph_page_scene_with_display_hydrogens,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn atom(id: u32, element: u8, x: f64, h: u16) -> Value {
    json!({"id":id,"element":element,"position":{"x_pt":x,"y_pt":64.0},"isotope":null,
        "charge":0,"radical":"none","explicit_hydrogens":h})
}
fn make(mut atoms: Vec<Value>, bonds: Value, flag: Option<(u32, Value)>) -> TestResult<Page> {
    if let Some((id, value)) = flag {
        let target = atoms
            .iter_mut()
            .find(|atom| atom["id"] == id)
            .ok_or("target")?;
        target["show_non_terminal_carbon_labels"] = value;
    }
    // Existing serde APIs let the old-model stage fail unknown_field, not a future API.
    let atoms = serde_json::from_value::<Vec<Atom>>(json!(atoms))?;
    let bonds = serde_json::from_value::<Vec<Bond>>(bonds)?;
    let mut doc = Document::default();
    let page = PageId::new(1).ok_or("page")?;
    doc.set_page_size(page, PageSize::new(256.0, 256.0)?)?;
    doc.append_fragments(
        page,
        vec![Fragment::new(
            FragmentId::new(17).ok_or("fragment")?,
            atoms,
            bonds,
        )?],
    )?;
    Ok(doc.pages().first().ok_or("page")?.clone())
}
fn chain(flag: Option<(u32, Value)>) -> TestResult<Page> {
    make(
        vec![
            atom(29, 6, 32.0, 3),
            atom(11, 6, 96.0, 2),
            atom(7, 8, 160.0, 1),
        ],
        json!([{"id":41,"begin":29,"end":11,"order":"single"},
            {"id":42,"begin":11,"end":7,"order":"single"}]),
        flag,
    )
}
fn centered(flag: Option<Value>) -> TestResult<Page> {
    make(
        vec![atom(29, 6, 32.0, 2), atom(11, 6, 96.0, 2)],
        json!([{"id":41,"begin":29,"end":11,"order":"double"}]),
        flag.map(|v| (29, v)),
    )
}
fn refused(result: Result<GraphScene, Error>) -> TestResult {
    let error = result
        .err()
        .ok_or("specified carbon flag silently rendered the legacy scene")?;
    assert!(
        matches!(error, Error::UnsupportedChemicalContent),
        "{error}"
    );
    Ok(())
}
fn all_entries_refuse(page: &Page, counts: &BTreeMap<AtomId, u16>) -> TestResult {
    let before = (*page).clone();
    let json = serde_json::to_value(page)?;
    refused(graph_page_scene(576, 576, page))?;
    refused(graph_page_scene_with_display_hydrogens(
        576,
        576,
        page,
        counts,
        &Cancellation::default(),
    ))?;
    let view = PageView::fit(576, 576, page.size())?;
    refused(view.graph_scene(page, &Cancellation::default()))?;
    refused(view.graph_scene_with_display_hydrogens(page, counts, &Cancellation::default()))?;
    assert_eq!(page, &before);
    assert_eq!(serde_json::to_value(page)?, json);
    Ok(())
}

#[test]
fn any_specified_flag_refuses_even_on_terminal_carbon_and_noncarbon_without_losing_h() -> TestResult
{
    let baseline = chain(None)?;
    graph_page_scene(576, 576, &baseline)?;
    let counts = BTreeMap::from([
        (AtomId::new(29).ok_or("atom")?, 3),
        (AtomId::new(11).ok_or("atom")?, 2),
        (AtomId::new(7).ok_or("atom")?, 1),
    ]);
    for flag in [false, true] {
        //29 terminal C,11 nonterminal C,7 terminal O: datum retained regardless of future role.
        for id in [29, 11, 7] {
            all_entries_refuse(&chain(Some((id, json!(flag))))?, &counts)?;
        }
        let double_counts = BTreeMap::from([
            (AtomId::new(29).ok_or("atom")?, 2),
            (AtomId::new(11).ok_or("atom")?, 2),
        ]);
        all_entries_refuse(&centered(Some(json!(flag)))?, &double_counts)?;
    }
    assert_eq!(
        baseline.fragments()[0]
            .atoms()
            .iter()
            .map(|atom| atom.explicit_hydrogens)
            .collect::<Vec<_>>(),
        vec![Some(3), Some(2), Some(1)]
    );
    Ok(())
}

#[test]
fn absent_and_explicit_null_keep_every_legacy_centered_pixel_hit_and_model_value() -> TestResult {
    let absent = centered(None)?;
    let null = centered(Some(Value::Null))?;
    let before = serde_json::to_value(&absent)?;
    assert_eq!(serde_json::to_value(&null)?, before);
    for atom in absent.fragments()[0].atoms() {
        assert!(
            serde_json::to_value(atom)?
                .get("show_non_terminal_carbon_labels")
                .is_none()
        );
    }
    let actual = graph_page_scene(576, 576, &absent)?;
    assert_eq!(graph_page_scene(576, 576, &null)?.scene(), actual.scene());
    let counts = BTreeMap::from([
        (AtomId::new(29).ok_or("atom")?, 2),
        (AtomId::new(11).ok_or("atom")?, 2),
    ]);
    assert_eq!(
        graph_page_scene_with_display_hydrogens(
            576,
            576,
            &absent,
            &counts,
            &Cancellation::default()
        )?
        .scene(),
        actual.scene()
    );
    let view = PageView::fit(576, 576, absent.size())?;
    assert_eq!(
        view.graph_scene(&absent, &Cancellation::default())?.scene(),
        actual.scene()
    );
    let expected = [157.0, 163.0].map(|y| Stroke {
        begin: [96.0, y],
        end: [224.0, y],
        width: 2.0,
        color: [0, 0, 0, 255],
    });
    assert_eq!(
        actual
            .scene()
            .primitives
            .iter()
            .filter_map(|p| match p {
                Primitive::Stroke(s) => Some(s.clone()),
                _ => None,
            })
            .collect::<Vec<_>>(),
        expected
    );
    let fragment = FragmentId::new(17).ok_or("fragment")?;
    let bond = bondline_model::BondId::new(41).ok_or("bond")?;
    assert_eq!(
        actual
            .bond_strokes(fragment, bond)
            .cloned()
            .collect::<Vec<_>>(),
        expected
    );
    for y in [157.0, 163.0] {
        assert_eq!(
            actual.hit_test([160.0, y], 0.0)?,
            Some(HitTarget::Bond {
                fragment_id: fragment,
                bond_id: bond
            })
        );
    }
    assert_eq!(actual.hit_test([160.0, 160.0], 0.0)?, None);
    let cpu = cpu_scene(actual.scene())?;
    for y in 150_u32..171 {
        for x in 128_u32..192 {
            let offset = usize::try_from((u64::from(y) * 576 + u64::from(x)) * 4)?;
            let ink = [157.0, 163.0]
                .iter()
                .any(|row| (f64::from(y) + 0.5 - row).abs() <= 1.0);
            assert_eq!(
                cpu.rgba.get(offset..offset + 4).ok_or("RGBA")?,
                if ink {
                    &[0, 0, 0, 255]
                } else {
                    &[255, 255, 255, 255]
                },
                "pixel{x},{y}"
            );
        }
    }
    assert_eq!(cpu_scene(graph_page_scene(576, 576, &null)?.scene())?, cpu);
    assert_eq!(pollster::block_on(gpu_scene(actual.scene()))?.0, cpu);
    assert_eq!(serde_json::to_value(&absent)?, before);
    assert_eq!(serde_json::to_value(&null)?, before);
    Ok(())
}

#[test]
fn entry_cancellation_precedes_any_specified_flag_in_all_shared_routes() -> TestResult {
    let counts = BTreeMap::from([
        (AtomId::new(29).ok_or("atom")?, 3),
        (AtomId::new(11).ok_or("atom")?, 2),
        (AtomId::new(7).ok_or("atom")?, 1),
    ]);
    for flag in [false, true] {
        let page = chain(Some((11, json!(flag))))?;
        let before = page.clone();
        let cancel = Cancellation::default();
        cancel.cancel();
        assert!(matches!(
            graph_page_scene_with_cancel(576, 576, &page, &cancel),
            Err(Error::Cancelled)
        ));
        assert!(matches!(
            graph_page_scene_with_display_hydrogens(576, 576, &page, &counts, &cancel),
            Err(Error::Cancelled)
        ));
        let view = PageView::fit(576, 576, page.size())?;
        assert!(matches!(
            view.graph_scene(&page, &cancel),
            Err(Error::Cancelled)
        ));
        assert!(matches!(
            view.graph_scene_with_display_hydrogens(&page, &counts, &cancel),
            Err(Error::Cancelled)
        ));
        assert_eq!(page, before);
    }
    Ok(())
}
