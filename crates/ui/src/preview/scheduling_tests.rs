use super::*;
use bondline_commands::Engine;
use eframe::egui;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn physical_size_uses_uniform_scale_and_full_logical_dpi_identity() -> TestResult {
    let engine = Engine::default();
    let rect = egui::Rect::from_min_size(egui::pos2(10.0, 20.0), egui::vec2(1000.0, 800.0));
    let a = PreviewKey::new(&engine, rect, 1.0, 4096)?;
    let b = PreviewKey::new(&engine, rect, 2.0, 4096)?;
    let capped = PreviewKey::new(&engine, rect, 4.0, 4096)?;
    assert_eq!(a.pixels, [1000, 800]);
    assert_eq!(b.pixels, [2000, 1600]);
    assert_eq!(capped.pixels, [2048, 1638]);
    assert_ne!(a, b);
    let other = PreviewKey::new(
        &engine,
        egui::Rect::from_min_size(rect.min, egui::vec2(500.0, 400.0)),
        4.0,
        4096,
    )?;
    assert_eq!(other.pixels, b.pixels);
    assert_ne!(
        other, b,
        "same device size does not mean same logical text size"
    );
    assert!(b.labels_readable(Some(16.0)));
    assert!(!b.labels_readable(Some(15.0)));
    assert!(!other.labels_readable(Some(16.0)));
    for em in [None, Some(f32::NAN), Some(f32::INFINITY), Some(-1.0)] {
        assert!(!b.labels_readable(em));
    }
    Ok(())
}

#[test]
fn close_cancel_epoch_changes_identity_without_authored_edits() -> TestResult {
    let mut engine = Engine::default();
    let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1000.0, 800.0));
    let first = PreviewKey::new(&engine, rect, 1.0, 2048)?;
    engine.dispatch("document.close", serde_json::json!({"expected_revision":0}))?;
    engine.dispatch(
        "document.close",
        serde_json::json!({"expected_revision":0,"action":"cancel"}),
    )?;
    let next = PreviewKey::new(&engine, rect, 1.0, 2048)?;
    assert_eq!(first.stamp, next.stamp);
    assert_ne!(first, next);
    assert_eq!(engine.state().revision, 0);
    Ok(())
}

#[test]
fn invalid_view_inputs_fail_before_any_job_or_texture() {
    let engine = Engine::default();
    let good = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1024.0, 1024.0));
    for dpi in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        assert!(PreviewKey::new(&engine, good, dpi, 2048).is_err());
    }
    for size in [
        egui::vec2(0.0, 10.0),
        egui::vec2(10.0, f32::INFINITY),
        egui::vec2(100_001.0, 100.0),
    ] {
        assert!(
            PreviewKey::new(
                &engine,
                egui::Rect::from_min_size(egui::Pos2::ZERO, size),
                1.0,
                2048
            )
            .is_err()
        );
    }
    assert!(PreviewKey::new(&engine, good, 1.0, 0).is_err());
}
