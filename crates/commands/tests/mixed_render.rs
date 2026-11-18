//! Actual source-aware scenes against frozen independent FontTools/FreeType evidence.
use bondline_io::{
    StoredDocument,
    display::{drawing_page_scene, drawing_page_scene_with_view},
};
use bondline_model::{AtomId, BondId, PageSize};
use bondline_render::{Cancellation, GraphScene, HitTarget, PageView, cpu_scene, gpu_scene};
use serde_json::Value;
use std::{collections::BTreeSet, sync::Arc};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const LAYOUT: &str = include_str!("../../../xtask/oracles/mixed-layout/mixed-layout-vectors.json");
const PAINTED: &str = include_str!("../../../xtask/oracles/mixed-layout/mixed-painted-spans.json");

fn source(name: &str) -> TestResult<&'static str> {
    Ok(match name {
        "amino_alcohol" => include_str!("../../io/tests/fixtures/mixed-labels/amino_alcohol.cdxml"),
        "amino_diamine" => include_str!("../../io/tests/fixtures/mixed-labels/amino_diamine.cdxml"),
        "hidden_middle_p12_5_m7_25" => {
            include_str!("../../io/tests/fixtures/mixed-labels/hidden_middle_p12_5_m7_25.cdxml")
        }
        "all_p24_m12" => include_str!("../../io/tests/fixtures/mixed-labels/all_p24_m12.cdxml"),
        "subset_1_3_5_p12_5_m7_25" => {
            include_str!("../../io/tests/fixtures/mixed-labels/subset_1_3_5_p12_5_m7_25.cdxml")
        }
        "edited_all_p24_m12" => {
            include_str!("../../io/tests/fixtures/mixed-labels/edited_all_p24_m12.cdxml")
        }
        "methanol" => include_str!("../../io/tests/fixtures/mixed-labels/methanol.cdxml"),
        _ => return Err("unknown frozen mixed source".into()),
    })
}

fn graph(case: &Value) -> TestResult<(StoredDocument, GraphScene)> {
    let raw = source(case["source_case"].as_str().ok_or("source case")?)?;
    let stored =
        StoredDocument::from_cdxml(Arc::from(raw.as_bytes()), PageSize::new(512.0, 512.0)?)?;
    let page = stored.model().pages().first().ok_or("page")?;
    let cancel = Cancellation::default();
    let view = &case["view"];
    let scene = if view["id"] == "physical2x" {
        let phase: [f64; 2] = serde_json::from_value(view["phase"].clone())?;
        let logical = PageView::fit(576, 576, page.size())?;
        let physical = logical.at_display_scale(2.0, phase)?;
        drawing_page_scene_with_view(&physical, &stored, page.id(), &cancel)?
    } else {
        drawing_page_scene(
            serde_json::from_value(view["width_px"].clone())?,
            serde_json::from_value(view["height_px"].clone())?,
            &stored,
            page.id(),
            &cancel,
        )?
    };
    assert_eq!(stored.to_cdxml()?.as_ref(), raw.as_bytes());
    Ok((stored, scene))
}

#[test]
fn source_aware_mixed_layout_hidden_anchors_and_all_frames_match_independent_vectors() -> TestResult
{
    let vectors: Value = serde_json::from_str(LAYOUT)?;
    let cases = vectors["cases"].as_array().ok_or("cases")?;
    assert_eq!(cases.len(), 21);
    let mut failures = Vec::new();
    let mut bond_count = 0;
    for case in cases {
        let (stored, graph) = match graph(case) {
            Ok(scene) => scene,
            Err(error) => {
                failures.push(format!(
                    "{}/{}: {error:?}",
                    case["source_case"], case["view"]["id"]
                ));
                continue;
            }
        };
        let fragment = stored.model().pages()[0]
            .fragments()
            .first()
            .ok_or("fragment")?;
        let expected_hidden: BTreeSet<u32> =
            serde_json::from_value(case["hidden_model_atoms"].clone())?;
        let actual_hidden =
            bondline_render::hidden_carbon_atoms(fragment, &Cancellation::default())?
                .into_iter()
                .map(AtomId::get)
                .collect::<BTreeSet<_>>();
        assert_eq!(actual_hidden, expected_hidden);
        for (id, point) in case["hidden_canvas_anchors"].as_object().ok_or("anchors")? {
            let point: [f32; 2] = serde_json::from_value(point.clone())?;
            assert_eq!(
                graph.hit_test(point, 0.0)?,
                Some(HitTarget::Atom {
                    fragment_id: fragment.id(),
                    atom_id: AtomId::new(id.parse()?).ok_or("atom")?,
                })
            );
        }
        for (id, glyphs) in case["authored_glyphs"].as_object().ok_or("glyph tuples")? {
            let atom = fragment
                .atoms()
                .iter()
                .find(|a| a.id.get().to_string() == *id)
                .ok_or("authored atom")?;
            let label = atom.authored_label.as_deref().ok_or("authored label")?;
            let glyphs = glyphs.as_array().ok_or("glyphs")?;
            let body = glyphs
                .iter()
                .map(|g| g["character"].as_str().ok_or("character"))
                .collect::<Result<Vec<_>, _>>()?
                .concat();
            assert_eq!(
                label.runs().iter().map(|r| r.text()).collect::<String>(),
                body
            );
            assert_eq!(label.runs().len(), 1);
            for glyph in glyphs {
                assert_eq!(
                    label.runs()[0].style().size_pt(),
                    glyph["size_pt"].as_f64().ok_or("size")?
                );
            }
            bondline_render::validate_atom_label(atom)?;
        }
        let scale = case["view"]["scale"].as_f64().ok_or("scale")?;
        for bond in case["bonds"].as_array().ok_or("bonds")? {
            let id: u32 = serde_json::from_value(bond["model_id"].clone())?;
            let strokes = graph
                .bond_strokes(fragment.id(), BondId::new(id).ok_or("bond")?)
                .collect::<Vec<_>>();
            assert_eq!(strokes.len(), 1);
            let stroke = strokes[0];
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
                        "{}/{} bond{id} {key}: {actual} outside{interval:?}",
                        case["source_case"],
                        case["view"]["id"]
                    );
                }
            }
            let origin: [f64; 2] = serde_json::from_value(case["view"]["origin_px"].clone())?;
            let points: [[f64; 2]; 2] =
                serde_json::from_value(bond["untrimmed_points_pt"].clone())?;
            for (actual, point, key, atom_key) in [
                (
                    stroke.begin,
                    points[0],
                    "begin_cut_pt_interval",
                    "begin_model_atom",
                ),
                (
                    stroke.end,
                    points[1],
                    "end_cut_pt_interval",
                    "end_model_atom",
                ),
            ] {
                let atom: u32 = serde_json::from_value(bond[atom_key].clone())?;
                if expected_hidden.contains(&atom) {
                    assert_eq!(bond[key], serde_json::json!([0.0, 0.0]));
                    assert_eq!(
                        actual,
                        [
                            (point[0] * scale + origin[0]) as f32,
                            (point[1] * scale + origin[1]) as f32
                        ],
                        "hidden endpoint must retain its exact anchor without a synthesized label stub"
                    );
                }
                let direction: [f64; 2] = serde_json::from_value(bond["direction"].clone())?;
                let sign = if atom_key == "begin_model_atom" {
                    1.0
                } else {
                    -1.0
                };
                let anchor = [point[0] * scale + origin[0], point[1] * scale + origin[1]];
                let cut = sign
                    * ((f64::from(actual[0]) - anchor[0]) * direction[0]
                        + (f64::from(actual[1]) - anchor[1]) * direction[1]);
                let interval: [f64; 2] = serde_json::from_value(bond[key].clone())?;
                let rounding = 8.0
                    * f64::from(f32::EPSILON)
                    * stroke
                        .begin
                        .iter()
                        .chain(&stroke.end)
                        .map(|v| f64::from(v.abs()))
                        .fold(1.0, f64::max);
                assert!(
                    cut + rounding >= interval[0] * scale
                        && cut <= interval[1] * scale + 1e-4 + rounding,
                    "{}/{} bond{id} {key} axial{cut} outside{interval:?}",
                    case["source_case"],
                    case["view"]["id"]
                );
            }
            bond_count += 1;
        }
        let cpu = cpu_scene(graph.scene())?;
        let (gpu, adapter) = pollster::block_on(gpu_scene(graph.scene()))?;
        assert!(!adapter.name.is_empty());
        assert_eq!(
            cpu, gpu,
            "entire actual frame {}/{}",
            case["source_case"], case["view"]["id"]
        );
        if case["view"]["id"] == "physical2x" {
            assert_eq!(
                [cpu.width, cpu.height],
                [1153, 1153],
                "existing ceil(logical*density+phase) viewport"
            );
        }
    }
    assert!(
        failures.is_empty(),
        "actual source-aware display failures: {failures:?}"
    );
    assert_eq!(bond_count, 75);
    Ok(())
}

fn mask(spans: &Value, width: usize, height: usize) -> TestResult<Vec<bool>> {
    let mut mask = vec![false; width * height];
    for span in spans.as_array().ok_or("spans")? {
        let [x, y, w, h]: [usize; 4] = serde_json::from_value(span.clone())?;
        assert_eq!(h, 1);
        assert!(x + w <= width && y < height);
        mask[y * width + x..y * width + x + w].fill(true);
    }
    Ok(mask)
}

#[test]
fn actual_mixed_source_pixels_obey_frozen_black_bands_and_freetype_glyphs() -> TestResult {
    let layout: Value = serde_json::from_str(LAYOUT)?;
    let painted: Value = serde_json::from_str(PAINTED)?;
    let cases = painted["cases"].as_array().ok_or("painted cases")?;
    assert_eq!(cases.len(), 5);
    let mut failures = Vec::new();
    let mut uncertain_total = 0;
    for case in cases {
        let layout_case = layout["cases"]
            .as_array()
            .ok_or("layout")?
            .iter()
            .find(|v| v["source_case"] == case["source_case"] && v["view"]["id"] == case["view"])
            .ok_or("layout case")?;
        let (_, graph) = match graph(layout_case) {
            Ok(scene) => scene,
            Err(error) => {
                failures.push(format!(
                    "{}/{}: {error:?}",
                    case["source_case"], case["view"]
                ));
                continue;
            }
        };
        let frame = cpu_scene(graph.scene())?;
        let [w, h]: [usize; 2] = serde_json::from_value(case["dimensions"].clone())?;
        assert!(w <= frame.width as usize && h <= frame.height as usize);
        let certain = mask(&case["certain_black_butt_spans"], w, h)?;
        let possible = mask(&case["possible_black_butt_spans"], w, h)?;
        let glyphs = mask(&case["freetype_glyph_spans"], w, h)?;
        let mut intersection = 0;
        let mut union = 0;
        let mut uncertain = 0;
        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                let offset = (y * frame.width as usize + x) * 4;
                let pixel = &frame.rgba[offset..offset + 4];
                let black = pixel == [0, 0, 0, 255];
                let blue = pixel == [51, 102, 204, 255];
                assert!(
                    !certain[i] || black,
                    "certain black {}/{} at{x},{y}",
                    case["source_case"],
                    case["view"]
                );
                assert!(
                    !black || possible[i],
                    "unexpected black {}/{} at{x},{y}",
                    case["source_case"],
                    case["view"]
                );
                intersection += usize::from(blue && glyphs[i]);
                union += usize::from(blue || glyphs[i]);
                uncertain += usize::from(possible[i] && !certain[i]);
            }
        }
        assert!(union > 0);
        assert!(
            intersection as f64 / union as f64 >= 0.90,
            "predeclared FreeType IoU {}/{}: {intersection}/{union}",
            case["source_case"],
            case["view"]
        );
        assert_eq!(
            uncertain,
            case["uncertain_black_pixels"].as_u64().ok_or("uncertain")? as usize
        );
        uncertain_total += uncertain;
        for probe in case["hidden_anchor_probes"]
            .as_object()
            .ok_or("probes")?
            .values()
        {
            for point in probe["certain_black_near_anchor"]
                .as_array()
                .ok_or("probe pixels")?
            {
                let [x, y]: [usize; 2] = serde_json::from_value(point.clone())?;
                let offset = (y * frame.width as usize + x) * 4;
                assert_eq!(
                    &frame.rgba[offset..offset + 4],
                    &[0, 0, 0, 255],
                    "hidden anchor retains actual bond ink"
                );
            }
        }
    }
    assert!(
        failures.is_empty(),
        "actual source-aware pixel failures: {failures:?}"
    );
    assert_eq!(
        uncertain_total, 1,
        "frozen uncertainty band remains unchanged"
    );
    Ok(())
}
