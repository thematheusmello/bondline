//! Source-preserving created labels against independently frozen source/font facts.
use bondline_io::{
    StoredDocument,
    display::{drawing_page_scene, drawing_page_scene_with_view},
};
use bondline_model::{AtomId, BondId};
use bondline_model::{
    AtomLabel, Element, Fragment, PageSize, Point2, TextJustification, TextRun, TextStyle,
};
use bondline_render::{
    Cancellation, GraphScene, HitTarget, PageView, Primitive, Scene, cpu_scene, gpu_scene,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const LAYOUT: &str =
    include_str!("../../../xtask/oracles/created-layout/created-layout-vectors.json");
const PAINTED: &str =
    include_str!("../../../xtask/oracles/created-layout/created-painted-spans.json");

fn source(name: &str) -> TestResult<&'static str> {
    Ok(match name {
        "base" => include_str!("../../io/tests/fixtures/created-labels/base.cdxml"),
        "created_3_black" => {
            include_str!("../../io/tests/fixtures/created-labels/created_3_black.cdxml")
        }
        "created_3_blue14" => {
            include_str!("../../io/tests/fixtures/created-labels/created_3_blue14.cdxml")
        }
        "created_3_nh" => include_str!("../../io/tests/fixtures/created-labels/created_3_nh.cdxml"),
        "created_all_black" => {
            include_str!("../../io/tests/fixtures/created-labels/created_all_black.cdxml")
        }
        "created_3_selected_p12_5_m7_25" => include_str!(
            "../../io/tests/fixtures/created-labels/created_3_selected_p12_5_m7_25.cdxml"
        ),
        _ => return Err("unknown frozen creation source".into()),
    })
}
fn facts() -> TestResult<Value> {
    Ok(serde_json::from_str(include_str!(
        "../../io/tests/fixtures/created-labels/creation-facts.json"
    ))?)
}
fn current(name: &str) -> TestResult<StoredDocument> {
    let mut stored = StoredDocument::from_cdxml(
        Arc::from(source("base")?.as_bytes()),
        PageSize::new(512.0, 512.0)?,
    )?;
    let page = stored.model().pages()[0].id();
    let fragment = &stored.model().pages()[0].fragments()[0];
    let mut atoms = fragment.atoms().to_vec();
    let facts = facts()?;
    for node in facts[name]["nodes"].as_array().ok_or("nodes")? {
        let atom = atoms
            .iter_mut()
            .find(|a| node["model_id"] == a.id.get())
            .ok_or("atom")?;
        atom.element =
            Element::try_from(u8::try_from(node["element"].as_u64().ok_or("element")?)?)?;
        atom.explicit_hydrogens = Some(u16::try_from(node["hydrogens"].as_u64().ok_or("H")?)?);
        let point: [f64; 2] = serde_json::from_value(node["point"].clone())?;
        atom.position = Some(Point2::new(point[0], point[1])?);
        if !node["label"].is_null() {
            let label = &node["label"];
            let rgb: [f64; 3] = serde_json::from_value(label["rgb"].clone())?;
            let style = TextStyle::new(
                label["font"].as_str().ok_or("font")?.into(),
                label["size"].as_f64().ok_or("size")?,
                u16::try_from(label["face"].as_u64().ok_or("face")?)?,
                rgb,
            )?;
            let expected = AtomLabel::new(
                None,
                Some(TextJustification::Left),
                Some(TextJustification::Left),
                None,
                vec![TextRun::new(
                    label["text"].as_str().ok_or("body")?.into(),
                    style.clone(),
                )?],
            )?
            .with_bounding_box(Some(serde_json::from_value(label["bounding_box"].clone())?))?;
            if atom.authored_label.is_none() {
                let actual = bondline_render::create_atom_label(atom, style)?;
                assert_eq!(actual, expected, "independent newly-created child data");
            }
            atom.authored_label = Some(Arc::new(expected));
        } else {
            assert!(atom.authored_label.is_none());
        }
    }
    let replacement = Fragment::new(fragment.id(), atoms, fragment.bonds().to_vec())?;
    stored.model_mut().replace_fragment(page, replacement)?;
    Ok(stored)
}

#[test]
fn current_created_source_renders_before_encoding_and_matches_frozen_bytes() -> TestResult {
    let stored = current("created_3_black")?;
    let page = stored.model().pages()[0].id();
    drawing_page_scene(1088, 1088, &stored, page, &Cancellation::default())?;
    assert_eq!(
        stored.to_cdxml()?.as_ref(),
        source("created_3_black")?.as_bytes()
    );
    Ok(())
}

fn graph(case: &Value) -> TestResult<(StoredDocument, GraphScene)> {
    let name = case["source_case"].as_str().ok_or("source case")?;
    let stored = current(name)?;
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
    assert_eq!(stored.to_cdxml()?.as_ref(), source(name)?.as_bytes());
    Ok((stored, scene))
}

#[test]
fn current_created_layout_hidden_anchors_and_entire_frames_match_independent_vectors() -> TestResult
{
    let vectors: Value = serde_json::from_str(LAYOUT)?;
    let cases = vectors["cases"].as_array().ok_or("cases")?;
    assert_eq!(cases.len(), 15);
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
    assert_eq!(bond_count, 60);
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
fn current_created_strokes_and_coloured_glyphs_match_frozen_independent_spans() -> TestResult {
    let layout: Value = serde_json::from_str(LAYOUT)?;
    let painted: Value = serde_json::from_str(PAINTED)?;
    let cases = painted["cases"].as_array().ok_or("painted cases")?;
    assert_eq!(cases.len(), 5);
    let mut uncertain_total = 0;
    for case in cases {
        let layout_case = layout["cases"]
            .as_array()
            .ok_or("layout")?
            .iter()
            .find(|v| v["source_case"] == case["source_case"] && v["view"]["id"] == case["view"])
            .ok_or("layout case")?;
        let (_, graph) = graph(layout_case)?;
        let actual = graph.scene();
        let [w, h]: [usize; 2] = serde_json::from_value(case["dimensions"].clone())?;
        assert_eq!([actual.width as usize, actual.height as usize], [w, h]);
        // Black child glyphs and black bonds have distinct public primitive types.
        // Inspect both actual layers without treating a glyph pixel as bond ink.
        let strokes = cpu_scene(&Scene {
            width: actual.width,
            height: actual.height,
            background: [255; 4],
            primitives: actual
                .primitives
                .iter()
                .filter(|p| matches!(p, Primitive::Stroke(_)))
                .cloned()
                .collect(),
        })?;
        let certain = mask(&case["certain_black_butt_spans"], w, h)?;
        let possible = mask(&case["possible_black_butt_spans"], w, h)?;
        let mut uncertain = 0;
        for (i, pixel) in strokes.rgba.chunks_exact(4).enumerate() {
            let black = pixel == [0, 0, 0, 255];
            assert!(
                !certain[i] || black,
                "certain bond ink {}/{} at{},{}",
                case["source_case"],
                case["view"],
                i % w,
                i / w
            );
            assert!(
                !black || possible[i],
                "unexpected bond ink {}/{} at{},{}",
                case["source_case"],
                case["view"],
                i % w,
                i / w
            );
            uncertain += usize::from(possible[i] && !certain[i]);
        }
        assert_eq!(
            uncertain,
            case["uncertain_black_pixels"]
                .as_u64()
                .ok_or("uncertainty")? as usize
        );
        uncertain_total += uncertain;
        let full = cpu_scene(actual)?;
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
                let offset = (y * w + x) * 4;
                assert_eq!(&strokes.rgba[offset..offset + 4], &[0, 0, 0, 255]);
                assert_eq!(
                    &full.rgba[offset..offset + 4],
                    &[0, 0, 0, 255],
                    "hidden anchor has no white eraser or glyph overlay"
                );
            }
        }
        let mut glyph_groups = BTreeMap::<[u8; 4], Vec<bool>>::new();
        for layer in case["glyph_layers"].as_array().ok_or("glyph layers")? {
            let [r, g, b]: [u8; 3] = serde_json::from_value(layer["rgb"].clone())?;
            let target = glyph_groups
                .entry([r, g, b, 255])
                .or_insert_with(|| vec![false; w * h]);
            for (target, value) in
                target
                    .iter_mut()
                    .zip(mask(&layer["freetype_glyph_spans"], w, h)?)
            {
                *target |= value;
            }
        }
        for (color, expected) in glyph_groups {
            let glyphs = cpu_scene(&Scene {
                width: actual.width,
                height: actual.height,
                background: [255; 4],
                primitives: actual
                    .primitives
                    .iter()
                    .filter(|p| matches!(p,Primitive::Rectangle(r) if r.color==color))
                    .cloned()
                    .collect(),
            })?;
            let (mut intersection, mut union) = (0_usize, 0_usize);
            for (pixel, expected) in glyphs.rgba.chunks_exact(4).zip(expected) {
                let actual = pixel == color;
                intersection += usize::from(actual && expected);
                union += usize::from(actual || expected);
            }
            assert!(union > 0);
            assert!(
                intersection as f64 / union as f64 >= 0.90,
                "predeclared FreeType glyph IoU {}/{} {color:?}: {intersection}/{union}",
                case["source_case"],
                case["view"]
            );
        }
    }
    assert_eq!(
        uncertain_total, 3,
        "independently frozen uncertainty count before actual tests"
    );
    Ok(())
}
