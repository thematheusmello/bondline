//! Independent original caption positions/metrics, plus external OpenType/FreeType.
use bondline_model::{Document, Page, PageSize, Point2, StyledText, TextId, TextRun, TextStyle};
use bondline_render::{Cancellation, Error, GraphScene, PageView, Primitive, cpu_scene, gpu_scene};
use serde_json::{Value, json};
use std::{
    io::Write,
    process::{Command, Stdio},
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn caption(id: u32, position: [f64; 2], runs: &[(&str, f64, [f64; 3])]) -> TestResult<StyledText> {
    Ok(StyledText::new(
        TextId::new(id).ok_or("text")?,
        Point2::new(position[0], position[1])?,
        runs.iter()
            .map(|(text, size, rgb)| {
                Ok(TextRun::new(
                    (*text).into(),
                    TextStyle::new("Roboto".into(), *size, 0, *rgb)?,
                )?)
            })
            .collect::<TestResult<Vec<_>>>()?,
    )?)
}
fn page(texts: Vec<StyledText>, extent: f64) -> TestResult<Page> {
    let mut d = Document::default();
    let id = d.pages()[0].id();
    d.set_page_size(id, PageSize::new(extent, extent)?)?;
    d.append_texts(id, texts)?;
    Ok(d.pages()[0].clone())
}
fn graph(page: &Page) -> TestResult<GraphScene> {
    Ok(PageView::fit(576, 576, page.size())?.graph_scene(page, &Cancellation::default())?)
}
fn boxes(graph: &GraphScene) -> Value {
    json!(
        graph
            .scene()
            .primitives
            .iter()
            .filter_map(|p| match p {
                Primitive::Rectangle(r)
                    if r.color != [196, 196, 196, 255]
                        && r.color != [210, 210, 210, 255]
                        && r.color != [255, 255, 255, 255] =>
                    Some(json!([r.x, r.y, r.width, r.height, r.color])),
                _ => None,
            })
            .collect::<Vec<_>>()
    )
}
fn consume(
    graph: &GraphScene,
    baseline: [f64; 2],
    runs: Value,
) -> TestResult<std::process::Output> {
    let interpreter = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| "python3".into());
    let mut child = Command::new(interpreter)
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
            "/tests/fixtures/caption-font-vectors.json"
        ))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut stdin = child.stdin.take().ok_or("stdin")?;
    serde_json::to_writer(
        &mut stdin,
        &json!({"dimensions":[graph.scene().width,graph.scene().height],
        "baseline":baseline,"runs":runs,"boxes":boxes(graph)}),
    )?;
    stdin.flush()?;
    drop(stdin);
    Ok(child.wait_with_output()?)
}
fn external(graph: &GraphScene, baseline: [f64; 2], runs: Value) -> TestResult<Value> {
    let output = consume(graph, baseline, runs)?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout)?;
    println!("caption external {result}");
    Ok(result)
}
fn gpu_equal(graph: &GraphScene) -> TestResult {
    let cpu = cpu_scene(graph.scene())?;
    let (gpu, adapter) = pollster::block_on(gpu_scene(graph.scene()))?;
    println!("caption GPU {:?} {}", adapter.backend, adapter.name);
    assert!(!adapter.name.trim().is_empty());
    assert_eq!(cpu, gpu);
    Ok(())
}

#[test]
fn multiple_runs_sizes_rgb_unicode_and_actual_baseline_match_independent_font_tools_and_pillow()
-> TestResult {
    let p = page(
        vec![caption(
            7,
            [32.0, 128.0],
            &[
                ("AV To", 48.0, [1.0, 0.0, 0.0]),
                ("λΩé", 32.0, [0.0, 0.0, 1.0]),
            ],
        )?],
        256.0,
    )?;
    let before = p.clone();
    let g = graph(&p)?;
    assert_eq!(g.text_em_range_px(), Some([64.0, 96.0]));
    assert_eq!(g.label_em_px(), None);
    let runs = json!([{"text":"AV To","em":96.0,"rgb":[255,0,0]},{"text":"λΩé","em":64.0,"rgb":[0,0,255]}]);
    let result = external(&g, [96.0, 288.0], runs.clone())?;
    assert_eq!(result["fonttools"], "4.61.1");
    gpu_equal(&g)?;
    // Actual source expectation controls: wrong baseline, glyph and exact colour fail.
    assert!(!consume(&g, [96.0, 296.0], runs.clone())?.status.success());
    let mut wrong = runs.clone();
    wrong[0]["rgb"] = json!([0, 255, 0]);
    assert!(!consume(&g, [96.0, 288.0], wrong)?.status.success());
    let mut wrong = runs;
    wrong[1]["text"] = json!("Ωé");
    assert!(!consume(&g, [96.0, 288.0], wrong)?.status.success());
    assert_eq!(
        g.hit_test([110.0, 250.0], 5.0)?,
        None,
        "Text has no fabricated chemical hit target"
    );
    assert_eq!(p, before);
    Ok(())
}

#[test]
fn pair_kerning_is_nonzero_within_one_run_and_resets_at_an_authored_boundary() -> TestResult {
    let joined = page(
        vec![caption(1, [32.0, 128.0], &[("AV", 48.0, [1.0, 0.0, 0.0])])?],
        256.0,
    )?;
    let split = page(
        vec![caption(
            1,
            [32.0, 128.0],
            &[("A", 48.0, [1.0, 0.0, 0.0]), ("V", 48.0, [1.0, 0.0, 0.0])],
        )?],
        256.0,
    )?;
    let a = graph(&joined)?;
    let b = graph(&split)?;
    assert_ne!(a.scene(), b.scene());
    let whole = external(
        &a,
        [96.0, 288.0],
        json!([{"text":"AV","em":96.0,"rgb":[255,0,0]}]),
    )?;
    let separate = external(
        &b,
        [96.0, 288.0],
        json!([{"text":"A","em":96.0,"rgb":[255,0,0]},{"text":"V","em":96.0,"rgb":[255,0,0]}]),
    )?;
    // FontTools actual GPOS: A1336,V1303,kernAV=-87,units2048.
    assert!((whole["end_x"].as_f64().ok_or("end")? - (96.0 + 2552.0 * 96.0 / 2048.0)).abs() < 1e-9);
    assert!(
        (separate["end_x"].as_f64().ok_or("end")? - (96.0 + 2639.0 * 96.0 / 2048.0)).abs() < 1e-9
    );
    assert!(
        !consume(
            &a,
            [96.0, 288.0],
            json!([{"text":"A","em":96.0,"rgb":[255,0,0]},{"text":"V","em":96.0,"rgb":[255,0,0]}])
        )?
        .status
        .success()
    );
    Ok(())
}

#[test]
fn spaces_and_empty_runs_advance_without_ink_and_every_caption_is_drawn() -> TestResult {
    let p = page(
        vec![
            caption(1, [32.0, 96.0], &[(" H", 48.0, [1.0, 0.0, 0.0])])?,
            caption(
                2,
                [32.0, 192.0],
                &[("", 12.0, [0.0, 1.0, 0.0]), ("V", 32.0, [0.0, 0.0, 1.0])],
            )?,
        ],
        256.0,
    )?;
    let before = p.clone();
    let g = graph(&p)?;
    assert_eq!(g.text_em_range_px(), Some([24.0, 96.0]));
    let red = page(
        vec![caption(1, [32.0, 96.0], &[(" H", 48.0, [1.0, 0.0, 0.0])])?],
        256.0,
    )?;
    external(
        &graph(&red)?,
        [96.0, 224.0],
        json!([{"text":" H","em":96.0,"rgb":[255,0,0]}]),
    )?;
    let colors = g
        .scene()
        .primitives
        .iter()
        .filter_map(|p| match p {
            Primitive::Rectangle(r) => Some(r.color),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(colors.contains(&[255, 0, 0, 255]) && colors.contains(&[0, 0, 255, 255]));
    assert!(!colors.contains(&[0, 255, 0, 255]));
    gpu_equal(&g)?;
    let blank = page(
        vec![caption(
            1,
            [32.0, 96.0],
            &[(" \u{a0}", 12.0, [1.0, 0.0, 0.0])],
        )?],
        256.0,
    )?;
    let blank = graph(&blank)?;
    assert_eq!(blank.scene().primitives.len(), 3);
    assert_eq!(blank.text_em_range_px(), Some([24.0, 24.0]));
    assert_eq!(blank.label_em_px(), None);
    assert_eq!(p, before);
    Ok(())
}

#[test]
fn fractional_physical_geometry_has_fresh_glyphs_and_min_max_including_blank_runs() -> TestResult {
    let p = page(
        vec![caption(
            1,
            [32.0, 128.0],
            &[("AV To", 24.0, [1.0, 0.0, 0.0])],
        )?],
        256.0,
    )?;
    let before = p.clone();
    let view = PageView::fit(576, 576, p.size())?;
    let g = view
        .at_display_scale(2.0, [0.25, 0.75])?
        .graph_scene(&p, &Cancellation::default())?;
    assert_eq!(g.text_em_range_px(), Some([96.0, 96.0]));
    assert_eq!(g.label_em_px(), None);
    external(
        &g,
        [192.25, 576.75],
        json!([{"text":"AV To","em":96.0,"rgb":[255,0,0]}]),
    )?;
    gpu_equal(&g)?;
    let mixed = page(
        vec![caption(
            1,
            [32.0, 128.0],
            &[("", 4.0, [0.0, 1.0, 0.0]), ("H", 96.0, [1.0, 0.0, 0.0])],
        )?],
        256.0,
    )?;
    let logical = PageView::zoom(576, 576, mixed.size(), 100.0, Point2::new(128.0, 128.0)?)?;
    assert!(matches!(
        logical.graph_scene(&mixed, &Cancellation::default()),
        Err(Error::LabelScale)
    ));
    let physical = logical
        .at_display_scale(2.0, [0.0, 0.0])?
        .graph_scene(&mixed, &Cancellation::default())?;
    assert_eq!(physical.text_em_range_px(), Some([8.0, 192.0]));
    assert_eq!(p, before);
    Ok(())
}

#[test]
fn unsupported_font_face_missing_cmap_and_line_controls_refuse_without_tofu_or_partial_page()
-> TestResult {
    for (family, face, text) in [
        ("Arial", 0, "H"),
        ("Roboto Regular", 0, "H"),
        ("Roboto", 1, "H"),
        ("Roboto", 32, "H"),
        ("Roboto", 96, "H"),
        ("Roboto", 0, "\u{10ffff}"),
        ("Roboto", 0, "H\nH"),
        ("Roboto", 0, "H\tH"),
        ("Roboto", 0, "H\rH"),
    ] {
        let bad = StyledText::new(
            TextId::new(2).ok_or("text")?,
            Point2::new(32.0, 192.0)?,
            vec![TextRun::new(
                text.into(),
                TextStyle::new(family.into(), 12.0, face, [0.0; 3])?,
            )?],
        )?;
        let p = page(
            vec![
                caption(1, [32.0, 96.0], &[("H", 12.0, [1.0, 0.0, 0.0])])?,
                bad,
            ],
            256.0,
        )?;
        let before = p.clone();
        assert!(
            matches!(graph(&p),Err(error) if error.to_string().contains("caption")),
            "{family}/{face}/{text:?}"
        );
        let cancel = Cancellation::default();
        cancel.cancel();
        assert!(matches!(
            PageView::fit(576, 576, p.size())?.graph_scene(&p, &cancel),
            Err(Error::Cancelled)
        ));
        assert_eq!(p, before);
    }
    Ok(())
}

#[test]
fn actual_paper_scale_glyph_run_pixel_and_primitive_budgets_are_atomic() -> TestResult {
    for position in [[32.0, 1.0], [254.0, 96.0]] {
        let p = page(vec![caption(1, position, &[("H", 12.0, [0.0; 3])])?], 256.0)?;
        let before = p.clone();
        assert!(
            matches!(graph(&p),Err(error) if error.to_string().contains("viewport")),
            "paper {position:?}"
        );
        assert_eq!(p, before);
    }
    for size in [3.9, 128.1] {
        let p = page(
            vec![caption(1, [32.0, 192.0], &[("H", size, [0.0; 3])])?],
            256.0,
        )?;
        assert!(matches!(graph(&p),Err(error) if error.to_string().contains("em size")));
    }
    let huge = page(
        vec![caption(
            1,
            [72.0, 1000.0],
            &[(&"W".repeat(100), 256.0, [0.0; 3])],
        )?],
        100000.0,
    )?;
    let many = page(
        vec![caption(
            1,
            [72.0, 1000.0],
            &[(&"H".repeat(100), 64.0, [0.0; 3])],
        )?],
        10000.0,
    )?;
    for (p, dimension) in [(huge, 100000), (many, 10000)] {
        let before = p.clone();
        assert!(matches!(
            PageView::fit(dimension, dimension, p.size())?
                .graph_scene(&p, &Cancellation::default()),
            Err(Error::SceneTooComplex)
        ));
        assert_eq!(p, before);
    }
    let spaces = page(
        vec![caption(
            1,
            [32.0, 96.0],
            &[(&" ".repeat(4097), 12.0, [0.0; 3])],
        )?],
        256.0,
    )?;
    assert!(matches!(graph(&spaces),Err(error) if error.to_string().contains("4096")));
    let style = TextStyle::new("Roboto".into(), 12.0, 0, [0.0; 3])?;
    let a = StyledText::new(
        TextId::new(1).ok_or("id")?,
        Point2::new(32.0, 96.0)?,
        (0..4096)
            .map(|_| TextRun::new(String::new(), style.clone()))
            .collect::<Result<Vec<_>, _>>()?,
    )?;
    let runs = page(
        vec![a, caption(2, [32.0, 128.0], &[("", 12.0, [0.0; 3])])?],
        256.0,
    )?;
    assert!(matches!(graph(&runs),Err(error) if error.to_string().contains("4096")));
    Ok(())
}

#[test]
fn combining_contextual_and_bidirectional_sequences_are_explicitly_unsupported() -> TestResult {
    for text in [
        "A\u{0301}",
        "\u{0627}",
        "A\u{200d}V",
        "\u{202e}AV",
        "\u{2067}AV",
    ] {
        let p = page(
            vec![caption(1, [32.0, 96.0], &[(text, 12.0, [0.0; 3])])?],
            256.0,
        )?;
        let before = p.clone();
        assert!(graph(&p).is_err(), "unsupported shaping {text:?}");
        assert_eq!(p, before);
    }
    Ok(())
}
