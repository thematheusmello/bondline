//! Retained Label* scalars never replace the fully explicit run typography.
#[path = "../../../xtask/oracles/label_metadata_support.rs"]
#[allow(
    dead_code,
    reason = "shared frozen source and independent-reader helpers"
)]
mod metadata;
use bondline_io::{
    StoredDocument,
    display::{DisplayError, drawing_page_scene},
};
use bondline_model::{AtomId, BondId, PageSize};
use bondline_render::{Cancellation, GraphScene, HitTarget, cpu_scene, gpu_scene};
use serde_json::Value;
use std::sync::Arc;
type TestResult<T = ()> = metadata::Result<T>;

fn stored(raw: &str) -> TestResult<StoredDocument> {
    Ok(StoredDocument::from_cdxml(
        Arc::from(raw.as_bytes()),
        PageSize::new(512.0, 512.0)?,
    )?)
}
fn controls(name: &str) -> TestResult<(&'static str, &'static str, &'static str)> {
    Ok(match name {
        "mixed_a" => (
            include_str!("../../io/tests/fixtures/created-labels/base.cdxml"),
            include_str!("../../../xtask/oracles/mixed-layout/mixed-layout-vectors.json"),
            "amino_alcohol",
        ),
        "created_3" => (
            include_str!("../../io/tests/fixtures/created-labels/created_3_black.cdxml"),
            include_str!("../../../xtask/oracles/created-layout/created-layout-vectors.json"),
            "created_3_black",
        ),
        "caption_owners" => (
            include_str!("../../io/tests/fixtures/created-labels/caption_base.cdxml"),
            include_str!("../../../xtask/oracles/mixed-layout/mixed-layout-vectors.json"),
            "amino_alcohol",
        ),
        _ => return Err("undeclared bounded render case".into()),
    })
}
fn check_known_geometry(
    graph: &GraphScene,
    doc: &StoredDocument,
    name: &str,
    size: u32,
) -> TestResult {
    let facts = metadata::facts(name);
    let scale = if size == 576 { 1.0 } else { 2.0 };
    let fragment = &doc.model().pages()[0].fragments()[0];
    for node in facts["nodes"].as_array().ok_or("frozen nodes")? {
        let id = u32::try_from(node["model_id"].as_u64().ok_or("model id")?)?;
        let atom = fragment
            .atoms()
            .iter()
            .find(|a| a.id.get() == id)
            .ok_or("atom")?;
        let point: [f64; 2] = serde_json::from_value(node["point"].clone())?;
        assert_eq!(
            graph.hit_test(
                [
                    (32.0 + scale * point[0]) as f32,
                    (32.0 + scale * point[1]) as f32
                ],
                0.0
            )?,
            Some(HitTarget::Atom {
                fragment_id: fragment.id(),
                atom_id: AtomId::new(id).ok_or("atom id")?
            })
        );
        if node["label"].is_null() {
            assert!(atom.authored_label.is_none());
            continue;
        }
        let label = atom.authored_label.as_deref().ok_or("literal label")?;
        let expected: [f64; 4] = serde_json::from_value(node["label"]["bounding_box"].clone())?;
        assert_eq!(
            label.bounding_box(),
            Some(expected),
            "existing independent12pt FontTools box"
        );
        assert_eq!(label.runs().len(), 1);
        assert_eq!(
            label.runs()[0].text(),
            node["label"]["text"].as_str().ok_or("literal body")?
        );
        assert_eq!(label.runs()[0].style().font_family(), "Roboto");
        assert_eq!(label.runs()[0].style().size_pt(), 12.0);
        assert_eq!(label.runs()[0].style().face(), 0);
    }
    for text in doc.model().pages()[0].texts() {
        for run in text.runs() {
            assert_eq!(run.style().font_family(), "Roboto");
            assert_eq!(run.style().size_pt(), 12.0);
            assert_eq!(run.style().face(), 0);
        }
    }
    assert_eq!(graph.label_em_px(), Some((12.0 * scale) as f32));
    assert_eq!(graph.text_em_range_px(), Some([(12.0 * scale) as f32; 2]));
    let (_, vectors, source_case) = controls(name)?;
    let vectors: Value = serde_json::from_str(vectors)?;
    let view_id = if size == 576 { "fit1x" } else { "fit2x" };
    let case = vectors["cases"]
        .as_array()
        .ok_or("frozen cases")?
        .iter()
        .find(|v| v["source_case"] == source_case && v["view"]["id"] == view_id)
        .ok_or("existing independent layout")?;
    for bond in case["bonds"].as_array().ok_or("frozen bonds")? {
        let id = u32::try_from(bond["model_id"].as_u64().ok_or("bond id")?)?;
        let components = graph
            .bond_strokes(fragment.id(), BondId::new(id).ok_or("bond")?)
            .collect::<Vec<_>>();
        assert_eq!(components.len(), 1);
        let stroke = components[0];
        assert_eq!(
            f64::from(stroke.width),
            bond["width_canvas"].as_f64().ok_or("width")?
        );
        for (actual, key) in [
            (stroke.begin, "begin_canvas_xy_intervals"),
            (stroke.end, "end_canvas_xy_intervals"),
        ] {
            let expected: [[f64; 2]; 2] = serde_json::from_value(bond[key].clone())?;
            for (coordinate, interval) in actual.into_iter().zip(expected) {
                let actual = f64::from(coordinate);
                let rounding = 8.0 * f64::from(f32::EPSILON) * actual.abs().max(1.0);
                assert!(
                    actual >= interval[0] - 1e-4 - rounding
                        && actual <= interval[1] + 1e-4 + rounding,
                    "unchanged independent endpoint {name}/{size} bond{id} {key}: {actual} outside{interval:?}"
                );
            }
        }
    }
    Ok(())
}

fn original_mask(spans: &Value, width: usize, height: usize) -> TestResult<Vec<bool>> {
    let mut mask = vec![false; width * height];
    for span in spans.as_array().ok_or("frozen spans")? {
        let [x, y, w, h]: [usize; 4] = serde_json::from_value(span.clone())?;
        assert_eq!(h, 1);
        assert!(x + w <= width && y < height);
        mask[y * width + x..y * width + x + w].fill(true);
    }
    Ok(mask)
}

fn check_original_mixed_pixels(frame: &bondline_render::Frame, size: u32) -> TestResult {
    let reference: Value = serde_json::from_str(include_str!(
        "../../../xtask/oracles/mixed-layout/mixed-painted-spans.json"
    ))?;
    let view = if size == 576 { "fit1x" } else { "fit2x" };
    let case = reference["cases"]
        .as_array()
        .ok_or("frozen pixel cases")?
        .iter()
        .find(|v| v["source_case"] == "amino_alcohol" && v["view"] == view)
        .ok_or("frozen pixel case")?;
    let [w, h]: [usize; 2] = serde_json::from_value(case["dimensions"].clone())?;
    assert_eq!([frame.width as usize, frame.height as usize], [w, h]);
    let certain = original_mask(&case["certain_black_butt_spans"], w, h)?;
    let possible = original_mask(&case["possible_black_butt_spans"], w, h)?;
    let glyphs = original_mask(&case["freetype_glyph_spans"], w, h)?;
    let (mut intersection, mut union, mut uncertain) = (0_usize, 0_usize, 0_usize);
    for (i, pixel) in frame.rgba.chunks_exact(4).enumerate() {
        let black = pixel == [0, 0, 0, 255];
        let blue = pixel == [51, 102, 204, 255];
        assert!(
            !certain[i] || black,
            "unchanged certain stroke pixel {view} at{},{}",
            i % w,
            i / w
        );
        assert!(
            !black || possible[i],
            "unchanged possible stroke pixel {view} at{},{}",
            i % w,
            i / w
        );
        intersection += usize::from(blue && glyphs[i]);
        union += usize::from(blue || glyphs[i]);
        uncertain += usize::from(possible[i] && !certain[i]);
    }
    assert!(
        union > 0 && intersection as f64 / union as f64 >= 0.90,
        "original FreeType glyph IoU {view}: {intersection}/{union}"
    );
    assert_eq!(
        uncertain,
        case["uncertain_black_pixels"]
            .as_u64()
            .ok_or("frozen uncertainty")? as usize
    );
    Ok(())
}

#[test]
fn unused_metadata_preserves_literal_glyphs_bonds_hits_and_six_entire_cpu_gpu_frames() -> TestResult
{
    let mut failures = Vec::new();
    let mut frames = 0;
    for name in ["mixed_a", "created_3", "caption_owners"] {
        let raw = metadata::source(name);
        let doc = stored(raw)?;
        let (raw_control, _, _) = controls(name)?;
        let control = stored(raw_control)?;
        assert_eq!(
            doc.model(),
            control.model(),
            "unused metadata changes no authored style/data"
        );
        assert!(!doc.is_fully_mapped());
        assert!(
            !doc.has_boxed_atom_labels()
                && !doc.has_connected_atom_labels()
                && !doc.has_mixed_atom_labels()
                && !doc.has_flagged_atom_labels(),
            "old literal profile truth stays closed"
        );
        assert_eq!(doc.to_cdxml()?.as_ref(), raw.as_bytes());
        let page = doc.model().pages()[0].id();
        for size in [576, 1088] {
            let graph = match drawing_page_scene(size, size, &doc, page, &Cancellation::default()) {
                Ok(graph) => graph,
                Err(error) => {
                    failures.push(format!("{name}/{size}: {error:?}"));
                    continue;
                }
            };
            let raw_graph =
                drawing_page_scene(size, size, &control, page, &Cancellation::default())?;
            check_known_geometry(&graph, &doc, name, size)?;
            assert_eq!(
                graph.scene(),
                raw_graph.scene(),
                "all actual glyph/paper/bond primitives stay identical"
            );
            let cpu = cpu_scene(graph.scene())?;
            if name == "mixed_a" {
                check_original_mixed_pixels(&cpu, size)?;
            }
            assert_eq!(
                cpu,
                cpu_scene(raw_graph.scene())?,
                "raw-control pixels {name}/{size}"
            );
            let (gpu, adapter) = pollster::block_on(gpu_scene(graph.scene()))?;
            if frames == 0 {
                println!(
                    "metadata render adapter {:?} {}",
                    adapter.backend, adapter.name
                );
            }
            assert!(cpu == gpu, "entire actual CPU/GPU frame {name}/{size}");
            frames += 1;
        }
    }
    assert!(
        failures.is_empty(),
        "actual source-aware metadata failures: {failures:?}"
    );
    assert_eq!(frames, 6);
    Ok(())
}

#[test]
fn metadata_size_sixteen_never_certifies_a_wrong_box_for_the_explicit_twelve_point_run()
-> TestResult {
    let original = metadata::source("mixed_a");
    let from = "BoundingBox='92.5693359375 155.6171875 107.6748046875 164.3828125'";
    assert!(original.contains(from));
    // Independently calculated/pinned in label-metadata/SOURCES.md before producers.
    let raw = original.replacen(
        from,
        "BoundingBox='91.42578125 154.15625 111.56640625 165.84375'",
        1,
    );
    let doc = stored(&raw)?;
    let label = doc.model().pages()[0].fragments()[0].atoms()[0]
        .authored_label
        .as_deref()
        .ok_or("literal label")?;
    assert_eq!(label.runs()[0].style().size_pt(), 12.0);
    assert_eq!(
        doc.to_cdxml()?.as_ref(),
        raw.as_bytes(),
        "unsupported source still round-trips unchanged"
    );
    for size in [576, 1088] {
        assert!(matches!(
            drawing_page_scene(
                size,
                size,
                &doc,
                doc.model().pages()[0].id(),
                &Cancellation::default()
            ),
            Err(DisplayError::IncompleteSource)
        ));
    }
    Ok(())
}
