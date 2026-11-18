#[path = "support/boxed_atom.rs"]
mod support;
use bondline_render::{Primitive, graph_page_scene};
use support::*;
#[test]
fn explicit_boxed_literal_label_paints_authored_color_instead_of_refusing_or_synthesizing()
-> TestResult {
    let cases = cases()?;
    let v = cases
        .iter()
        .find(|v| v["name"] == "OH2-24pt-80-100")
        .ok_or("vector")?;
    let p = page(atom(v)?)?;
    let before = p.clone();
    let g = graph_page_scene(1088, 1088, &p)?;
    assert!(
        g.scene()
            .primitives
            .iter()
            .any(|p| matches!(p,Primitive::Rectangle(r) if r.color==[51,102,204,255]))
    );
    assert_eq!(g.label_em_px(), Some(48.0));
    assert_eq!(g.text_em_range_px(), Some([48.0, 48.0]));
    assert_eq!(p, before);
    Ok(())
}

fn colored_boxes(graph: &bondline_render::GraphScene) -> serde_json::Value {
    serde_json::json!(
        graph
            .scene()
            .primitives
            .iter()
            .filter_map(|p| match p {
                Primitive::Rectangle(r) if r.color == [51, 102, 204, 255] =>
                    Some(serde_json::json!([r.x, r.y, r.width, r.height, r.color])),
                _ => None,
            })
            .collect::<Vec<_>>()
    )
}
fn external(
    graph: &bondline_render::GraphScene,
    text: &str,
    baseline: [f64; 2],
    em: f64,
) -> TestResult<std::process::Output> {
    use std::process::{Command, Stdio};
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| "python3".into());
    let mut child = Command::new(python)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/caption_oracle.py"
        ))
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/fonts/Roboto-Regular.ttf"
        ))
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/boxed-atom-font-vectors.json"
        ))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    serde_json::to_writer(
        child.stdin.take().ok_or("stdin")?,
        &serde_json::json!({"dimensions":[graph.scene().width,graph.scene().height],"baseline":baseline,"runs":[{"text":text,"em":em,"rgb":[51,102,204]}],"boxes":colored_boxes(graph)}),
    )?;
    Ok(child.wait_with_output()?)
}
#[test]
fn literal_digits_and_whole_element_anchor_match_independent_pillow_and_gpu() -> TestResult {
    use bondline_render::{Cancellation, HitTarget, PageView, cpu_scene, gpu_scene};
    for text in ["OH2", "Cl", "CH65535"] {
        let v = cases()?
            .into_iter()
            .find(|v| v["name"] == format!("{text}-24pt-80-100"))
            .ok_or("vector")?;
        let p = page(atom(&v)?)?;
        let view = PageView::fit(1088, 1088, p.size())?;
        let g = view.graph_scene(&p, &Cancellation::default())?;
        let result = external(&g, text, [192.0, 232.0], 48.0)?;
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        println!("boxed {text}: {}", String::from_utf8_lossy(&result.stdout));
        assert!(
            !external(&g, text, [192.0, 242.0], 48.0)?.status.success(),
            "wrong baseline must fail"
        );
        if text == "OH2" {
            assert!(
                !external(&g, "OH", [192.0, 232.0], 48.0)?.status.success(),
                "missing literal2 must fail"
            );
        }
        let node = view.canvas_position(p.fragments()[0].atoms()[0].position.ok_or("position")?)?;
        assert_eq!(
            g.hit_test(node, 0.0)?,
            Some(HitTarget::Atom {
                fragment_id: p.fragments()[0].id(),
                atom_id: p.fragments()[0].atoms()[0].id
            })
        );
        let cpu = cpu_scene(g.scene())?;
        let (gpu, adapter) = pollster::block_on(gpu_scene(g.scene()))?;
        println!("boxed GPU {:?}: {}", adapter.backend, adapter.name);
        assert!(!adapter.name.is_empty());
        assert_eq!(cpu, gpu);
    }
    Ok(())
}
#[test]
fn all_scene_entries_keep_refusal_order_physical_paper_and_authoritative_h_guards() -> TestResult {
    use bondline_render::{
        Cancellation, Error, PageView, graph_page_scene_with_cancel,
        graph_page_scene_with_display_hydrogens,
    };
    use std::collections::BTreeMap;
    let v = cases()?
        .into_iter()
        .find(|v| v["name"] == "O-12pt-80-100")
        .ok_or("vector")?;
    let p = page(atom(&v)?)?;
    let cancel = Cancellation::default();
    let view = PageView::fit(1088, 1088, p.size())?;
    let counts = BTreeMap::from([(p.fragments()[0].atoms()[0].id, 0)]);
    assert!(graph_page_scene_with_display_hydrogens(1088, 1088, &p, &counts, &cancel).is_ok());
    assert!(
        view.graph_scene_with_display_hydrogens(&p, &counts, &cancel)
            .is_ok()
    );
    let bad_counts = BTreeMap::from([(p.fragments()[0].atoms()[0].id, 1)]);
    assert!(matches!(
        view.graph_scene_with_display_hydrogens(&p, &bad_counts, &cancel),
        Err(Error::UnsupportedChemicalContent)
    ));
    let physical = view.at_display_scale(2.0, [0.25, 0.5])?;
    let g = physical.graph_scene(&p, &cancel)?;
    assert_eq!(g.label_em_px(), Some(48.0));
    assert_eq!(g.text_em_range_px(), Some([48.0, 48.0]));
    assert!(matches!(
        graph_page_scene(0, 0, &p),
        Err(Error::InvalidViewport)
    ));
    cancel.cancel();
    assert!(matches!(
        graph_page_scene_with_cancel(0, 0, &p, &cancel),
        Err(Error::Cancelled)
    ));
    for v in cases()?.iter().filter(|v| {
        v["name"]
            .as_str()
            .is_some_and(|s| s.ends_with("-80-40") || s.ends_with("0-0"))
    }) {
        let p = page(atom(v)?)?;
        assert!(
            matches!(graph_page_scene(1088, 1088, &p), Err(Error::LabelPlacement)),
            "{}",
            v["name"]
        );
    }
    Ok(())
}
#[test]
fn authored_em_minimum_and_maximum_join_caption_range_without_changing_legacy_labels() -> TestResult
{
    use bondline_model::{
        Atom, AtomId, Element, Fragment, Point2, StyledText, TextId, TextRun, TextStyle,
    };
    use bondline_render::{Cancellation, PageView};
    let v = cases()?
        .into_iter()
        .find(|v| v["name"] == "OH-10pt-80-100")
        .ok_or("vector")?;
    let mut d = bondline_model::Document::default();
    let id = d.pages()[0].id();
    d.set_page_size(id, bondline_model::PageSize::new(512.0, 512.0)?)?;
    let mut legacy = Atom::new(
        AtomId::new(2).ok_or("id")?,
        Element::try_from(6)?,
        Some(Point2::new(220.0, 220.0)?),
    );
    legacy.explicit_hydrogens = Some(4);
    d.append_fragments(
        id,
        vec![Fragment::new(
            bondline_model::FragmentId::new(1).ok_or("fragment")?,
            vec![atom(&v)?, legacy],
            vec![],
        )?],
    )?;
    d.append_texts(
        id,
        vec![StyledText::new(
            TextId::new(1).ok_or("text")?,
            Point2::new(60.0, 320.0)?,
            vec![TextRun::new(
                "A".into(),
                TextStyle::new("Roboto".into(), 24.0, 0, [0.0; 3])?,
            )?],
        )?],
    )?;
    let p = &d.pages()[0];
    let view = PageView::fit(1088, 1088, p.size())?;
    let g = view.graph_scene(p, &Cancellation::default())?;
    assert_eq!(g.label_em_px(), Some(20.0));
    assert_eq!(g.text_em_range_px(), Some([20.0, 48.0]));
    let dense = view
        .at_display_scale(2.0, [0.0; 2])?
        .graph_scene(p, &Cancellation::default())?;
    assert_eq!(dense.label_em_px(), Some(40.0));
    assert_eq!(dense.text_em_range_px(), Some([40.0, 96.0]));
    Ok(())
}

#[test]
fn boxed_labels_keep_single_and_multiple_bond_real_ink_and_collision_guards() -> TestResult {
    use bondline_model::{
        AtomId, Bond, BondId, BondOrder, Document, DoublePosition, Fragment, FragmentId, PageSize,
        Point2,
    };
    use bondline_render::{Error, position_atom_label};
    use std::sync::Arc;
    let value = cases()?
        .into_iter()
        .find(|v| v["name"] == "O-24pt-80-100")
        .ok_or("vector")?;
    let first = atom(&value)?;
    for (dx, expected) in [(12.0, false), (96.0, true)] {
        for order in [BondOrder::Single, BondOrder::Double, BondOrder::Triple] {
            let mut second = first.clone();
            second.id = AtomId::new(2).ok_or("id")?;
            let p = second.position.ok_or("position")?;
            second.position = Some(Point2::new(p.x_pt() + dx, p.y_pt())?);
            second.authored_label = Some(Arc::new(position_atom_label(&second)?));
            let mut d = Document::default();
            let id = d.pages()[0].id();
            d.set_page_size(id, PageSize::new(512.0, 512.0)?)?;
            d.append_fragments(
                id,
                vec![Fragment::new(
                    FragmentId::new(1).ok_or("fragment")?,
                    vec![first.clone(), second],
                    vec![Bond {
                        id: BondId::new(1).ok_or("bond")?,
                        begin: first.id,
                        end: AtomId::new(2).ok_or("id")?,
                        order,
                        double_position: DoublePosition::Unspecified,
                    }],
                )?],
            )?;
            let result = graph_page_scene(1088, 1088, &d.pages()[0]);
            if expected {
                assert!(result.is_ok(), "{order:?}");
            } else {
                assert!(
                    matches!(&result, Err(Error::LabelOverlap)),
                    "{order:?}: {:?}",
                    result.err()
                );
            }
        }
    }
    Ok(())
}
#[test]
fn large_authored_style_cannot_hide_behind_a_small_legacy_label_em() -> TestResult {
    use bondline_model::{
        Atom, AtomId, AtomLabel, Document, Element, Fragment, FragmentId, PageSize, Point2,
        TextJustification, TextRun, TextStyle,
    };
    use bondline_render::{Cancellation, Error, PageView, position_atom_label};
    use std::sync::Arc;
    let v = cases()?
        .into_iter()
        .find(|v| v["name"] == "O-12pt-80-100")
        .ok_or("vector")?;
    let mut authored = atom(&v)?;
    authored.authored_label = Some(Arc::new(AtomLabel::new(
        None,
        Some(TextJustification::Left),
        Some(TextJustification::Left),
        None,
        vec![TextRun::new(
            "O".into(),
            TextStyle::new("Roboto".into(), 128.0, 0, [0.2, 0.4, 0.8])?,
        )?],
    )?));
    authored.authored_label = Some(Arc::new(position_atom_label(&authored)?));
    let mut legacy = Atom::new(
        AtomId::new(2).ok_or("id")?,
        Element::try_from(6)?,
        Some(Point2::new(300.0, 100.0)?),
    );
    legacy.explicit_hydrogens = Some(4);
    let mut d = Document::default();
    let id = d.pages()[0].id();
    d.set_page_size(id, PageSize::new(512.0, 512.0)?)?;
    d.append_fragments(
        id,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("id")?,
            vec![authored, legacy],
            vec![],
        )?],
    )?;
    let p = &d.pages()[0];
    let view = PageView::fit(1088, 1088, p.size())?;
    let g = view.graph_scene(p, &Cancellation::default())?;
    assert_eq!(g.label_em_px(), Some(24.0));
    assert_eq!(g.text_em_range_px(), Some([256.0, 256.0]));
    assert!(matches!(
        view.at_display_scale(2.0, [0.0; 2])?
            .graph_scene(p, &Cancellation::default()),
        Err(Error::LabelScale)
    ));
    Ok(())
}
