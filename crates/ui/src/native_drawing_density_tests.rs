//! Actual App-frame vector/picking acceptance; separate from native OS execution.
use super::*;
use bondline_model::{Atom, AtomId, Bond, BondId, BondOrder, Element, Fragment, FragmentId};
use bondline_render::{Error as RenderError, HitTarget};

fn source_drawing(fixture: &mut Fixture) -> TestResult {
    let mut stored = Engine::default().stored_document()?.clone();
    let a = AtomId::new(1).ok_or("atom")?;
    let b = AtomId::new(2).ok_or("atom")?;
    stored.model_mut().append_fragments(
        PageId::new(1).ok_or("page")?,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            vec![
                Atom::new(a, Element::try_from(6)?, Some(Point2::new(72.0, 72.0)?)),
                Atom::new(b, Element::try_from(6)?, Some(Point2::new(120.0, 72.0)?)),
            ],
            vec![Bond {
                id: BondId::new(1).ok_or("bond")?,
                begin: a,
                end: b,
                order: BondOrder::Double,
            }],
        )?],
    )?;
    let mut engine = Engine::from_document(stored);
    engine.set_ui_file_host(true);
    *fixture.app.engine.lock().map_err(|_| "engine")? = engine;
    Ok(())
}

fn guard_text(output: &egui::FullOutput) -> bool {
    let expected = RenderError::BondScale.to_string();
    output
        .shapes
        .iter()
        .any(|s| matches!(&s.shape,egui::Shape::Text(t) if t.galley.text().contains(&expected)))
}

fn black_quads(output: &egui::FullOutput, area: egui::Rect) -> Vec<Vec<Pos2>> {
    output
        .shapes
        .iter()
        .filter_map(|s| match &s.shape {
            egui::Shape::Path(p)
                if p.closed
                    && p.points.len() == 4
                    && p.fill == egui::Color32::BLACK
                    && p.points.iter().all(|point| area.contains(*point)) =>
            {
                Some(p.points.clone())
            }
            _ => None,
        })
        .collect()
}

fn covers(points: &[Pos2], point: Pos2) -> bool {
    let mut positive = false;
    let mut negative = false;
    for index in 0..points.len() {
        let a = points[index];
        let b = points[(index + 1) % points.len()];
        let cross = (b.x - a.x) * (point.y - a.y) - (b.y - a.y) * (point.x - a.x);
        positive |= cross > 1e-5;
        negative |= cross < -1e-5;
    }
    !(positive && negative)
}

#[test]
fn actual_app_one_x_keeps_the_truthful_fixed_fit_visibility_refusal() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 1.0)?;
    let area = canvas(&f.warm())?;
    source_drawing(&mut f)?;
    let before = f.state()?;
    let view = f.view()?;
    let output = f.warm();
    println!("one-x actual canvas {:?}", area);
    assert_eq!(output.pixels_per_point, 1.0);
    assert!(
        guard_text(&output),
        "one-x hidden strip must remain an honest BondScale refusal"
    );
    assert!(black_quads(&output, area).is_empty());
    assert_eq!(f.state()?, before);
    assert_eq!(f.view()?, view);
    Ok(())
}

#[test]
fn actual_app_two_x_uses_device_pixels_for_strips_gap_and_logical_pointer_hits() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 2.0)?;
    let area = canvas(&f.warm())?;
    source_drawing(&mut f)?;
    let before = f.state()?;
    let view = f.view()?;
    let output = f.warm();
    assert_eq!(output.pixels_per_point, 2.0);
    println!("two-x actual canvas {:?}", area);
    assert!(
        !guard_text(&output),
        "the physical two-x pixel lattice can display both strips and their gap"
    );
    let quads = black_quads(&output, area);
    assert_eq!(
        quads.len(),
        2,
        "both actual App vector components must be painted"
    );
    // Derive Fit independently from the actual App canvas, never a widget rectangle guess.
    let [width, height] = [
        f64::from(area.width().floor()),
        f64::from(area.height().floor()),
    ];
    let page = [210.0 * 72.0 / 25.4, 297.0 * 72.0 / 25.4];
    let fit = ((width - 64.0) / page[0]).min((height - 64.0) / page[1]);
    let paper = [(page[0] * fit).round(), (page[1] * fit).round()];
    let origin = [
        ((width - paper[0]) / 2.0).floor(),
        ((height - paper[1]) / 2.0).floor(),
    ];
    let sx = paper[0] / page[0];
    let sy = paper[1] / page[1];
    let logical_axis = [
        f64::from(area.left()) + origin[0] + 96.0 * sx,
        f64::from(area.top()) + origin[1] + 72.0 * sy,
    ];
    let device_axis = logical_axis.map(|v| v * 2.0);
    let centers = [device_axis[1] - 3.0 * sx, device_axis[1] + 3.0 * sx];
    let half_width = sx;
    let column = device_axis[0].floor() as i64;
    let mut rows = [Vec::new(), Vec::new()];
    let mut gap = Vec::new();
    for y in (centers[0] - half_width).floor() as i64..=(centers[1] + half_width).ceil() as i64 {
        let center = y as f64 + 0.5;
        for (index, axis) in centers.iter().enumerate() {
            if (center - axis).abs() <= half_width {
                rows[index].push(y);
            }
        }
        if center > centers[0] + half_width && center < centers[1] - half_width {
            gap.push(y);
        }
    }
    assert!(
        rows.iter().all(|row| !row.is_empty()) && !gap.is_empty(),
        "independent global device-pixel corridor must be representable"
    );
    for (index, row) in rows.iter().enumerate() {
        for &y in row {
            let logical = egui::pos2((column as f32 + 0.5) / 2.0, (y as f32 + 0.5) / 2.0);
            assert!(
                quads.iter().any(|quad| covers(quad, logical)),
                "missing strip{index} at devicepixel{column},{y}"
            );
        }
    }
    for y in gap {
        let logical = egui::pos2((column as f32 + 0.5) / 2.0, (y as f32 + 0.5) / 2.0);
        assert!(
            quads.iter().all(|quad| !covers(quad, logical)),
            "device-pixel gap must remain clear"
        );
        assert!(output.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Rect(r) if r.fill==egui::Color32::WHITE&&r.rect.contains(logical)&&s.clip_rect.contains(logical))),"gap must have actual white paper beneath it");
    }
    let pointer = egui::pos2(logical_axis[0] as f32, logical_axis[1] as f32);
    f.frame(vec![Event::PointerMoved(pointer)]);
    assert_eq!(
        f.app.drawing.hover,
        Some(HitTarget::Bond {
            fragment_id: FragmentId::new(1).ok_or("fragment")?,
            bond_id: BondId::new(1).ok_or("bond")?
        }),
        "logical pointer remains aligned with the displayed physical bundle"
    );
    assert_eq!(f.state()?, before);
    assert_eq!(f.view()?, view);
    assert!(f.app.error.is_none(), "{:?}", f.app.error);
    Ok(())
}

fn literal_water(fixture: &mut Fixture) -> TestResult {
    let mut stored = Engine::default().stored_document()?.clone();
    let size = stored.model().pages().first().ok_or("page")?.size();
    let mut oxygen = Atom::new(
        AtomId::new(1).ok_or("atom")?,
        Element::try_from(8)?,
        Some(Point2::new(size.width_pt() / 2.0, size.height_pt() / 2.0)?),
    );
    oxygen.explicit_hydrogens = Some(2);
    stored.model_mut().append_fragments(
        PageId::new(1).ok_or("page")?,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            vec![oxygen],
            vec![],
        )?],
    )?;
    let mut engine = Engine::from_document(stored);
    engine.set_ui_file_host(true);
    *fixture.app.engine.lock().map_err(|_| "engine")? = engine;
    Ok(())
}

#[test]
fn actual_app_large_two_x_canvas_keeps_fresh_device_pixel_glyph_runs_without_readback_cap()
-> TestResult {
    let mut f = Fixture::new(1600.0, 1000.0, 2.0)?;
    let area = canvas(&f.warm())?;
    assert!(
        area.width() * 2.0 > 2048.0,
        "this actual vector surface must exceed the capture/readback bound"
    );
    literal_water(&mut f)?;
    let before = f.state()?;
    let view = f.view()?;
    let output = f.warm();
    assert_eq!(output.pixels_per_point, 2.0);
    let ink = output
        .shapes
        .iter()
        .filter_map(|s| match &s.shape {
            egui::Shape::Rect(r)
                if r.fill == egui::Color32::BLACK && area.contains_rect(r.rect) =>
            {
                Some(r.rect)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(
        !ink.is_empty(),
        "large physical vector display must not blank at a2048 readback cap"
    );
    assert!(
        ink.iter().any(|r| {
            let width = (r.width() * 2.0).round() as u32;
            let height = (r.height() * 2.0).round() as u32;
            width % 2 == 1 || height % 2 == 1
        }),
        "fresh device-pixel glyph runs must not be logical-pixel rectangles upscaled2x"
    );
    for rect in ink {
        for coordinate in [rect.left(), rect.top(), rect.right(), rect.bottom()] {
            let physical = coordinate * 2.0;
            assert!(
                (physical - physical.round()).abs() <= 0.001,
                "actual glyph runs align to global device pixel centers"
            );
        }
    }
    assert_eq!(f.state()?, before);
    assert_eq!(f.view()?, view);
    assert!(f.app.error.is_none(), "{:?}", f.app.error);
    Ok(())
}

#[test]
fn actual_app_physical_font_legibility_does_not_allow_logically_tiny_labels() -> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 2.0)?;
    f.warm();
    literal_water(&mut f)?;
    f.command("ui.view", json!({"action":"set_zoom","percent":50}))?;
    let before = f.state()?;
    let view = f.view()?;
    let output = f.warm();
    assert_eq!(output.pixels_per_point, 2.0);
    let expected = RenderError::LabelScale.to_string();
    assert!(
        output.shapes.iter().any(
            |s| matches!(&s.shape,egui::Shape::Text(t) if t.galley.text().contains(&expected))
        ),
        "logical6-em label must refuse even when physical2x yields12-em"
    );
    assert_eq!(f.state()?, before);
    assert_eq!(f.view()?, view);
    Ok(())
}

fn fractional_frame(fixture: &mut Fixture, events: Vec<Event>) -> egui::FullOutput {
    let mut input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::pos2(0.125, 0.375),
            egui::vec2(fixture.width, fixture.height),
        )),
        events,
        focused: true,
        ..Default::default()
    };
    input
        .viewports
        .entry(egui::ViewportId::ROOT)
        .or_default()
        .native_pixels_per_point = Some(2.0);
    let context = fixture.context.clone();
    let output = context.run(input, |ctx| fixture.app.update_workspace(ctx));
    if let Some(update) = &output.platform_output.accesskit_update {
        fixture.nodes.extend(update.nodes.iter().cloned());
    }
    output
}

#[test]
fn actual_app_fractional_canvas_origin_keeps_global_device_geometry_glyph_phase_and_hits()
-> TestResult {
    let mut f = Fixture::new(1160.0, 800.0, 2.0)?;
    fractional_frame(&mut f, vec![]);
    let area = canvas(&fractional_frame(&mut f, vec![]))?;
    let device_origin = [f64::from(area.left()) * 2.0, f64::from(area.top()) * 2.0];
    let phase = device_origin.map(|v| v - v.floor());
    println!("fractional actual canvas {area:?}, device phase={phase:?}");
    assert!(
        phase.iter().any(|v| (0.2..=0.8).contains(v)),
        "declared raw screen origin must actually yield a substantial device phase; snapping would be a setup failure"
    );
    source_drawing(&mut f)?;
    let mut stored = f
        .app
        .engine
        .lock()
        .map_err(|_| "engine")?
        .stored_document()?
        .clone();
    let size = stored.model().pages().first().ok_or("page")?.size();
    let mut oxygen = Atom::new(
        AtomId::new(3).ok_or("atom")?,
        Element::try_from(8)?,
        Some(Point2::new(size.width_pt() / 2.0, size.height_pt() / 2.0)?),
    );
    oxygen.explicit_hydrogens = Some(2);
    stored.model_mut().append_fragments(
        PageId::new(1).ok_or("page")?,
        vec![Fragment::new(
            FragmentId::new(2).ok_or("fragment")?,
            vec![oxygen],
            vec![],
        )?],
    )?;
    let mut engine = Engine::from_document(stored);
    engine.set_ui_file_host(true);
    *f.app.engine.lock().map_err(|_| "engine")? = engine;
    let before = f.state()?;
    let view = f.view()?;
    fractional_frame(&mut f, vec![]);
    let output = fractional_frame(&mut f, vec![]);
    assert_eq!(output.pixels_per_point, 2.0);
    assert!(
        !guard_text(&output),
        "valid physical strips must not use an unrelated logical lattice"
    );
    let mut quads = black_quads(&output, area);
    assert_eq!(quads.len(), 2);
    quads.sort_by(|a, b| {
        a.iter()
            .map(|p| p.y)
            .sum::<f32>()
            .total_cmp(&b.iter().map(|p| p.y).sum::<f32>())
    });
    let [width, height] = [
        f64::from(area.width().floor()),
        f64::from(area.height().floor()),
    ];
    let page = [210.0 * 72.0 / 25.4, 297.0 * 72.0 / 25.4];
    let fit = ((width - 64.0) / page[0]).min((height - 64.0) / page[1]);
    let paper = [(page[0] * fit).round(), (page[1] * fit).round()];
    let origin = [
        ((width - paper[0]) / 2.0).floor(),
        ((height - paper[1]) / 2.0).floor(),
    ];
    let sx = paper[0] / page[0];
    let sy = paper[1] / page[1];
    let begin = (f64::from(area.left()) + origin[0] + 72.0 * sx) * 2.0;
    let end = (f64::from(area.left()) + origin[0] + 120.0 * sx) * 2.0;
    let axis = (f64::from(area.top()) + origin[1] + 72.0 * sy) * 2.0;
    let centers = [axis - 3.0 * sx, axis + 3.0 * sx];
    for (quad, center) in quads.iter().zip(centers) {
        let mut actual = quad
            .iter()
            .map(|p| [f64::from(p.x) * 2.0, f64::from(p.y) * 2.0])
            .collect::<Vec<_>>();
        let mut expected = vec![
            [begin, center - sx],
            [begin, center + sx],
            [end, center - sx],
            [end, center + sx],
        ];
        let order = |a: &[f64; 2], b: &[f64; 2]| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1]));
        actual.sort_by(order);
        expected.sort_by(order);
        for (point, target) in actual.iter().zip(expected) {
            for axis in 0..2 {
                assert!(
                    (point[axis] - target[axis]).abs() <= 1.0 / 256.0,
                    "global device corner{point:?} != independent{target:?}"
                );
            }
        }
        let actual_center = quad.iter().map(|p| f64::from(p.y) * 2.0).sum::<f64>() / 4.0;
        assert!((actual_center - center).abs() <= 1.0 / 256.0);
    }
    let column = ((begin + end) / 2.0).floor() as i64;
    let mut strips = [0, 0];
    let mut gaps = 0;
    for y in (centers[0] - sx).floor() as i64..=(centers[1] + sx).ceil() as i64 {
        let py = y as f64 + 0.5;
        let logical = egui::pos2((column as f32 + 0.5) / 2.0, (y as f32 + 0.5) / 2.0);
        let in_ink = centers.iter().position(|center| (py - center).abs() <= sx);
        if let Some(index) = in_ink {
            strips[index] += 1;
            assert!(quads.iter().any(|q| covers(q, logical)));
        } else if py > centers[0] + sx && py < centers[1] - sx {
            gaps += 1;
            assert!(quads.iter().all(|q| !covers(q, logical)));
            assert!(output.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Rect(r) if r.fill==egui::Color32::WHITE&&r.rect.contains(logical)&&s.clip_rect.contains(logical))));
        }
    }
    assert!(strips.iter().all(|n| *n > 0) && gaps > 0);
    let glyphs = output
        .shapes
        .iter()
        .filter_map(|s| match &s.shape {
            egui::Shape::Rect(r)
                if r.fill == egui::Color32::BLACK && area.contains_rect(r.rect) =>
            {
                Some(r.rect)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(
        !glyphs.is_empty(),
        "the independently authored O/H2 supplies actual glyph-phase evidence"
    );
    for rect in glyphs {
        for coordinate in [rect.left(), rect.top(), rect.right(), rect.bottom()] {
            let physical = f64::from(coordinate) * 2.0;
            assert!(
                (physical - physical.round()).abs() <= 1.0 / 256.0,
                "glyph pixel must align to the GLOBAL device lattice despite fractional canvas origin"
            );
        }
    }
    fractional_frame(
        &mut f,
        vec![Event::PointerMoved(egui::pos2(
            ((begin + end) / 4.0) as f32,
            (axis / 2.0) as f32,
        ))],
    );
    assert_eq!(
        f.app.drawing.hover,
        Some(HitTarget::Bond {
            fragment_id: FragmentId::new(1).ok_or("fragment")?,
            bond_id: BondId::new(1).ok_or("bond")?
        })
    );
    assert_eq!(f.state()?, before);
    assert_eq!(f.view()?, view);
    assert!(f.app.error.is_none(), "{:?}", f.app.error);
    Ok(())
}
