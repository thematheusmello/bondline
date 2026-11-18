//! Runtime declarations for the source-locked explicit physical margin path.
#[path = "support/boxed_atom.rs"]
mod support;
use bondline_model::{
    AtomId, Bond, BondId, BondOrder, Document, DoublePosition, DrawingStyle, Fragment, FragmentId,
    Page, PageSize, Point2,
};
use bondline_render::{Cancellation, GraphScene, HitTarget, PageView, Primitive};
use std::sync::Arc;
use support::*;

fn pair() -> TestResult<Page> {
    let v = cases()?
        .into_iter()
        .find(|v| v["name"] == "O-12pt-80-100")
        .ok_or("O vector")?;
    let a = atom(&v)?;
    let mut b = a.clone();
    b.id = AtomId::new(2).ok_or("atom")?;
    let node = a.position.ok_or("position")?;
    b.position = Some(Point2::new(node.x_pt() + 80.0, node.y_pt())?);
    let label = b.authored_label.as_ref().ok_or("label")?;
    let mut bounds = label.bounding_box().ok_or("bounds")?;
    bounds[0] += 80.0;
    bounds[2] += 80.0;
    b.authored_label = Some(Arc::new(label.with_bounding_box(Some(bounds))?));
    let mut document = Document::default();
    let p = document.pages()[0].id();
    document.set_page_size(p, PageSize::new(512.0, 512.0)?)?;
    document.append_fragments(
        p,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            vec![a, b],
            vec![Bond {
                id: BondId::new(1).ok_or("bond")?,
                begin: AtomId::new(1).ok_or("atom")?,
                end: AtomId::new(2).ok_or("atom")?,
                order: BondOrder::Single,
                double_position: DoublePosition::Unspecified,
            }],
        )?],
    )?;
    Ok(document.pages()[0].clone())
}
fn view(page: &Page, margin: Option<f64>) -> TestResult<PageView> {
    Ok(PageView::fit(1088, 1088, page.size())?
        .with_drawing_style(DrawingStyle::new(Some(0.6))?.with_margin_width(margin)?))
}
fn glyphs(g: &GraphScene) -> Vec<[u32; 4]> {
    g.scene()
        .primitives
        .iter()
        .filter_map(|p| match p {
            Primitive::Rectangle(r) if r.color == [51, 102, 204, 255] => {
                Some([r.x, r.y, r.width, r.height])
            }
            _ => None,
        })
        .collect()
}
fn stroke(g: &GraphScene) -> TestResult<&bondline_render::Stroke> {
    g.bond_strokes(
        FragmentId::new(1).ok_or("fragment")?,
        BondId::new(1).ok_or("bond")?,
    )
    .next()
    .ok_or("stroke".into())
}
#[test]
fn explicit_margin_shortens_actual_ink_without_moving_glyphs_source_or_atom_hits() -> TestResult {
    let p = pair()?;
    let before = p.clone();
    let cancel = Cancellation::default();
    let legacy = view(&p, None)?
        .graph_scene(&p, &cancel)
        .map_err(|e| format!("legacy: {e:?}"))?;
    let zero = view(&p, Some(0.0))?
        .graph_scene(&p, &cancel)
        .map_err(|e| format!("zero: {e:?}"))?;
    let two = view(&p, Some(2.0))?
        .graph_scene(&p, &cancel)
        .map_err(|e| format!("two: {e:?}"))?;
    assert_eq!(glyphs(&legacy), glyphs(&zero));
    assert_eq!(glyphs(&zero), glyphs(&two));
    assert_eq!(p, before);
    assert!(stroke(&zero)?.begin[0] > stroke(&legacy)?.begin[0]);
    assert!(stroke(&two)?.begin[0] > stroke(&zero)?.begin[0] + 3.999);
    assert!(stroke(&two)?.end[0] < stroke(&zero)?.end[0] - 3.999);
    let white_count = |g: &GraphScene| {
        g.scene()
            .primitives
            .iter()
            .filter(|p| matches!(p,Primitive::Rectangle(r) if r.color == [255;4]))
            .count()
    };
    assert_eq!(
        white_count(&zero),
        1,
        "explicit zero has paper but no old white erasers"
    );
    assert_eq!(white_count(&two), 1);
    assert_eq!(white_count(&legacy), 3);
    for x in 176..210 {
        for y in 214..252 {
            let point = [x as f32 + 0.5, y as f32 + 0.5];
            let a = legacy.hit_test(point, 0.0)?;
            let b = two.hit_test(point, 0.0)?;
            assert_eq!(
                a.filter(|t| matches!(t, HitTarget::Atom { .. })),
                b.filter(|t| matches!(t, HitTarget::Atom { .. }))
            );
        }
    }
    let component = stroke(&two)?;
    assert_eq!(
        two.hit_test(
            [
                (component.begin[0] + component.end[0]) / 2.0,
                component.begin[1]
            ],
            0.0
        )?,
        Some(HitTarget::Bond {
            fragment_id: FragmentId::new(1).ok_or("fragment")?,
            bond_id: BondId::new(1).ok_or("bond")?,
        })
    );
    Ok(())
}

fn vector_page(v: &serde_json::Value, length: f64, second_label: bool) -> TestResult<Page> {
    use bondline_model::{Atom, AtomLabel, Element};
    let anchor: [f64; 2] = serde_json::from_value(v["anchor_pt"].clone())?;
    let direction: [f64; 2] = serde_json::from_value(v["direction"].clone())?;
    let mut a = Atom::new(
        AtomId::new(1).ok_or("atom")?,
        Element::try_from(v["element"].as_u64().ok_or("element")? as u8)?,
        Some(Point2::new(anchor[0], anchor[1])?),
    );
    a.explicit_hydrogens = Some(v["explicit_hydrogens"].as_u64().ok_or("H")? as u16);
    let label: AtomLabel = serde_json::from_value(v["authored_label"].clone())?;
    a.authored_label = Some(Arc::new(label));
    let mut b = if second_label {
        a.clone()
    } else {
        Atom::new(AtomId::new(2).ok_or("atom")?, Element::try_from(6)?, None)
    };
    b.id = AtomId::new(2).ok_or("atom")?;
    b.position = Some(Point2::new(
        anchor[0] + direction[0] * length,
        anchor[1] + direction[1] * length,
    )?);
    if second_label {
        let label = b.authored_label.as_ref().ok_or("label")?;
        let mut bounds = label.bounding_box().ok_or("bounds")?;
        bounds[0] += direction[0] * length;
        bounds[2] += direction[0] * length;
        bounds[1] += direction[1] * length;
        bounds[3] += direction[1] * length;
        b.authored_label = Some(Arc::new(label.with_bounding_box(Some(bounds))?));
    }
    let n = v["rail_offsets_pt"].as_array().ok_or("rails")?.len();
    let order = match n {
        1 => BondOrder::Single,
        2 => BondOrder::Double,
        3 => BondOrder::Triple,
        _ => return Err("rails".into()),
    };
    let mut d = Document::default();
    let p = d.pages()[0].id();
    d.set_page_size(p, PageSize::new(512.0, 512.0)?)?;
    d.append_fragments(
        p,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            vec![a, b],
            vec![Bond {
                id: BondId::new(1).ok_or("bond")?,
                begin: AtomId::new(1).ok_or("atom")?,
                end: AtomId::new(2).ok_or("atom")?,
                order,
                double_position: if n == 2 {
                    DoublePosition::Center
                } else {
                    DoublePosition::Unspecified
                },
            }],
        )?],
    )?;
    Ok(d.pages()[0].clone())
}
fn vector_view(p: &Page, v: &serde_json::Value) -> TestResult<PageView> {
    use bondline_model::DrawingSettings;
    let number = |key: &str| v[key].as_f64().ok_or("number");
    Ok(PageView::fit(576, 576, p.size())?
        .with_drawing_style(
            DrawingStyle::new(Some(number("line_width_pt")?))?
                .with_margin_width(Some(number("margin_pt")?))?
                .with_bond_spacing_percent(Some(number("bond_spacing_percent")?))?,
        )
        .with_drawing_settings(DrawingSettings::new(Some(number("bond_length_pt")?), None)?)
        .at_display_scale(number("canvas_scale")?, [0.25, 0.75])?)
}
#[test]
fn public_authored_scene_placements_match_all_independent_oblique_multiple_and_physical_vectors()
-> TestResult {
    let all: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/connected-label-clearance.json"))?;
    let mut checked = 0;
    for v in all["cases"]
        .as_array()
        .ok_or("cases")?
        .iter()
        .filter(|v| v["authored_source_case"] == true)
    {
        let p = vector_page(v, 80.0, false)?;
        let original = p.clone();
        let view = vector_view(&p, v)?;
        let graph = view
            .graph_scene(&p, &Cancellation::default())
            .map_err(|e| format!("{}: {e:?}", v["id"]))?;
        let anchor =
            view.canvas_position(p.fragments()[0].atoms()[0].position.ok_or("position")?)?;
        let u: [f64; 2] = serde_json::from_value(v["direction"].clone())?;
        let expected: [f64; 2] = serde_json::from_value(v["common_cut_canvas_interval"].clone())?;
        let mut actual = Vec::new();
        for s in graph.bond_strokes(p.fragments()[0].id(), p.fragments()[0].bonds()[0].id) {
            let cut = (f64::from(s.begin[0]) - f64::from(anchor[0])) * u[0]
                + (f64::from(s.begin[1]) - f64::from(anchor[1])) * u[1];
            let rounding = 8.0
                * f64::from(f32::EPSILON)
                * s.begin
                    .iter()
                    .chain(&s.end)
                    .map(|v| f64::from(v.abs()))
                    .fold(1.0, f64::max);
            assert!(
                cut + rounding >= expected[0] && cut <= expected[1] + 1e-4 + rounding,
                "{} cut {cut} expected{expected:?}",
                v["id"]
            );
            actual.push(cut);
        }
        assert_eq!(
            actual.len(),
            v["rail_offsets_pt"].as_array().ok_or("rails")?.len()
        );
        assert_eq!(p, original);
        checked += 1;
    }
    assert!(checked >= 100);
    Ok(())
}
#[test]
fn actual_short_oblique_oxygen_connector_survives_but_consumed_ink_refuses() -> TestResult {
    use bondline_render::{Error, cpu_scene, gpu_scene};
    let all: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/connected-label-clearance.json"))?;
    for item in all["short_oo_cases"].as_array().ok_or("short cases")? {
        let v = all["cases"]
            .as_array()
            .ok_or("cases")?
            .iter()
            .find(|v| v["id"] == item["begin_case"])
            .ok_or("case")?;
        let p = vector_page(v, item["visible_length_pt"].as_f64().ok_or("length")?, true)?;
        let view = vector_view(&p, v)?;
        let graph = view.graph_scene(&p, &Cancellation::default())?;
        let s = stroke(&graph)?;
        assert!((s.end[0] - s.begin[0]).hypot(s.end[1] - s.begin[1]) > 1.9);
        let center = [(s.begin[0] + s.end[0]) / 2.0, (s.begin[1] + s.end[1]) / 2.0];
        // Old rectangular atom ownership has priority even above newly visible short ink.
        let mut labels_only = Document::default();
        let page_id = labels_only.pages()[0].id();
        labels_only.set_page_size(page_id, p.size())?;
        labels_only.append_fragments(
            page_id,
            vec![Fragment::new(
                p.fragments()[0].id(),
                p.fragments()[0].atoms().to_vec(),
                vec![],
            )?],
        )?;
        let old_hit = vector_view(&labels_only.pages()[0], v)?
            .graph_scene(&labels_only.pages()[0], &Cancellation::default())?
            .hit_test(center, 0.0)?;
        let expected = old_hit.or(Some(HitTarget::Bond {
            fragment_id: p.fragments()[0].id(),
            bond_id: p.fragments()[0].bonds()[0].id,
        }));
        assert_eq!(graph.hit_test(center, 0.0)?, expected);
        let cpu = cpu_scene(graph.scene())?;
        assert!(
            cpu.rgba.chunks_exact(4).any(|p| p == [0, 0, 0, 255]),
            "actual black connector ink must remain"
        );
        let (gpu, adapter) = pollster::block_on(gpu_scene(graph.scene()))?;
        assert!(!adapter.name.is_empty());
        assert_eq!(cpu, gpu);
        let p = vector_page(
            v,
            item["consumed_length_pt"].as_f64().ok_or("length")?,
            true,
        )?;
        assert!(matches!(
            vector_view(&p, v)?.graph_scene(&p, &Cancellation::default()),
            Err(Error::LabelOverlap)
        ));
    }
    Ok(())
}

fn native_stacked(below: bool) -> TestResult<Page> {
    use bondline_model::{Atom, Element};
    let mut center = Atom::new(
        AtomId::new(1).ok_or("atom")?,
        Element::try_from(7)?,
        Some(Point2::new(160., 160.)?),
    );
    center.explicit_hydrogens = Some(3);
    let mut atoms = vec![center];
    let mut edges = vec![];
    let mut endpoints = vec![[80., 160.], [240., 160.]];
    if below {
        endpoints.push([160., 80.]);
    }
    for (i, p) in endpoints.into_iter().enumerate() {
        let id = AtomId::new(i as u32 + 2).ok_or("atom")?;
        atoms.push(Atom::new(
            id,
            Element::try_from(6)?,
            Some(Point2::new(p[0], p[1])?),
        ));
        edges.push(Bond {
            id: BondId::new(i as u32 + 1).ok_or("bond")?,
            begin: AtomId::new(1).ok_or("atom")?,
            end: id,
            order: BondOrder::Single,
            double_position: DoublePosition::Unspecified,
        });
    }
    let mut d = Document::default();
    let p = d.pages()[0].id();
    d.set_page_size(p, PageSize::new(512., 512.)?)?;
    d.append_fragments(
        p,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            atoms,
            edges,
        )?],
    )?;
    Ok(d.pages()[0].clone())
}
#[test]
fn native_above_below_count_layouts_keep_all_glyphs_and_masks_while_clipping_each_incident_bond()
-> TestResult {
    for below in [false, true] {
        let p = native_stacked(below)?;
        let make = |margin| -> TestResult<PageView> {
            Ok(PageView::fit(576, 576, p.size())?
                .with_drawing_style(DrawingStyle::default().with_margin_width(margin)?))
        };
        let old = make(None)?.graph_scene(&p, &Cancellation::default())?;
        let new = make(Some(1.0))?.graph_scene(&p, &Cancellation::default())?;
        let black = |g: &GraphScene| {
            g.scene()
                .primitives
                .iter()
                .filter_map(|p| match p {
                    Primitive::Rectangle(r) if r.color == [0, 0, 0, 255] => {
                        Some((r.x, r.y, r.width, r.height))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(black(&old), black(&new));
        for bond in p.fragments()[0].bonds() {
            let before = old
                .bond_strokes(p.fragments()[0].id(), bond.id)
                .next()
                .ok_or("stroke")?;
            let after = new
                .bond_strokes(p.fragments()[0].id(), bond.id)
                .next()
                .ok_or("stroke")?;
            assert_ne!(before.begin, after.begin);
            assert_eq!(before.end, after.end);
        }
    }
    Ok(())
}

#[test]
fn explicit_margin_keeps_single_crossing_ink_refused_even_on_a_mixed_multiple_page() -> TestResult {
    use bondline_model::{Atom, Element};
    use bondline_render::Error;
    for mixed in [false, true] {
        let mut points = vec![[100., 100.], [200., 200.], [100., 200.], [200., 100.]];
        if mixed {
            points.extend([[100., 300.], [200., 300.]]);
        }
        let atoms = points
            .into_iter()
            .enumerate()
            .map(|(i, p)| -> TestResult<_> {
                Ok(Atom::new(
                    AtomId::new(i as u32 + 1).ok_or("atom")?,
                    Element::try_from(6)?,
                    Some(Point2::new(p[0], p[1])?),
                ))
            })
            .collect::<TestResult<Vec<_>>>()?;
        let mut bonds = vec![];
        for (i, (a, b)) in [(1, 2), (3, 4), (5, 6)]
            .into_iter()
            .take(if mixed { 3 } else { 2 })
            .enumerate()
        {
            bonds.push(Bond {
                id: BondId::new(i as u32 + 1).ok_or("bond")?,
                begin: AtomId::new(a).ok_or("atom")?,
                end: AtomId::new(b).ok_or("atom")?,
                order: if i == 2 {
                    BondOrder::Double
                } else {
                    BondOrder::Single
                },
                double_position: DoublePosition::Unspecified,
            });
        }
        let mut d = Document::default();
        let p = d.pages()[0].id();
        d.set_page_size(p, PageSize::new(512., 512.)?)?;
        d.append_fragments(
            p,
            vec![Fragment::new(
                FragmentId::new(1).ok_or("fragment")?,
                atoms,
                bonds,
            )?],
        )?;
        let page = &d.pages()[0];
        let ink_view = |margin| -> TestResult<PageView> {
            Ok(PageView::fit(1088, 1088, page.size())?
                .with_drawing_style(DrawingStyle::new(Some(1.5))?.with_margin_width(margin)?))
        };
        assert!(
            ink_view(None)?
                .graph_scene(page, &Cancellation::default())
                .is_ok(),
            "legacy None path remains unchanged"
        );
        assert!(matches!(
            ink_view(Some(0.0))?.graph_scene(page, &Cancellation::default()),
            Err(Error::LabelOverlap)
        ));
    }
    Ok(())
}
