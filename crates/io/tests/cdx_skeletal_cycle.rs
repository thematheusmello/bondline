#![cfg(not(target_arch = "wasm32"))]
//! An independently constructed cycle exercises omitted chemistry and glyphless paper.
use bondline_io::{
    StoredDocument,
    display::drawing_page_scene,
    files::{read_cdx_file, save_cdx_file},
};
use bondline_model::{BondOrder, PageSize};
use bondline_render::{Cancellation, Frame, cpu_scene};
use std::{fs, path::Path, process::Command, sync::Arc};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "reuse the existing isolated file directory")]
mod files;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn check_model(stored: &StoredDocument, dormant: bool) -> Result {
    assert!(stored.has_cdx_skeletal_preview() && stored.has_display_projection());
    assert!(!stored.is_fully_mapped() && !stored.can_create_caption());
    let [page] = stored.model().pages() else {
        return Err("one cycle page".into());
    };
    assert_eq!(page.size(), PageSize::new(512.0, 512.0)?);
    assert!(page.texts().is_empty());
    let [fragment] = page.fragments() else {
        return Err("one cycle fragment".into());
    };
    let positions = [
        [128.0, 128.0],
        [256.0, 128.0],
        [256.0, 256.0],
        [128.0, 256.0],
    ];
    assert_eq!(fragment.atoms().len(), positions.len());
    for (i, (atom, [x, y])) in fragment.atoms().iter().zip(positions).enumerate() {
        assert_eq!(atom.id.get() as usize, i + 1);
        assert_eq!(atom.element.get(), 6);
        assert_eq!(atom.position.map(|p| [p.x_pt(), p.y_pt()]), Some([x, y]));
        assert_eq!(atom.explicit_hydrogens, None);
        assert!(atom.authored_label.is_none());
        assert_eq!(
            atom.show_terminal_carbon_labels,
            (dormant && i == 0).then_some(true)
        );
        assert_eq!(atom.show_non_terminal_carbon_labels, None);
    }
    assert_eq!(fragment.bonds().len(), 4);
    for (bond, (begin, end)) in fragment
        .bonds()
        .iter()
        .zip([(1, 2), (2, 3), (3, 4), (4, 1)])
    {
        assert_eq!(
            (bond.begin.get(), bond.end.get(), bond.order),
            (begin, end, BondOrder::Single)
        );
    }
    Ok(())
}

fn check_optics(frame: &Frame) {
    assert_eq!((frame.width, frame.height), (512, 512));
    // Fit leaves32px margins: scale7/8 places the square at144..256 with3.5px ink.
    let mut ink = 0;
    for y in 34..478 {
        for x in 34..478 {
            let px = x as f64 + 0.5;
            let py = y as f64 + 0.5;
            let expected = (144.0..=256.0).contains(&px)
                && ((py - 144.0).abs() <= 1.75 || (py - 256.0).abs() <= 1.75)
                || (144.0..=256.0).contains(&py)
                    && ((px - 144.0).abs() <= 1.75 || (px - 256.0).abs() <= 1.75);
            let start = (512 * y + x) * 4;
            assert_eq!(
                &frame.rgba[start..start + 4],
                if expected { &[0, 0, 0, 255] } else { &[255; 4] }
            );
            ink += usize::from(expected);
        }
    }
    assert_eq!(ink, 1776);
}

#[test]
fn unlabelled_cycles_preserve_omitted_h_and_exact_optics_through_unicode_save() -> Result {
    let cancel = Cancellation::default();
    let directory = files::FileTestDirectory::new()?;
    for (name, raw, dormant) in [
        (
            "cycle",
            include_bytes!("fixtures/cdx-skeletal-cycle/cycle.cdx").as_slice(),
            false,
        ),
        (
            "dormant",
            include_bytes!("fixtures/cdx-skeletal-cycle/dormant.cdx").as_slice(),
            true,
        ),
    ] {
        let input = directory.path(&format!("{name}-原.cdx"));
        let output = directory.path(&format!("{name}-保存-β.cdx"));
        fs::write(&input, raw)?;
        let loaded = read_cdx_file(&input, &cancel)?.document;
        check_model(&loaded, dormant)?;
        let page = &loaded.model().pages()[0];
        let graph = drawing_page_scene(512, 512, &loaded, page.id(), &cancel)?;
        assert!(graph.label_em_px().is_none() && graph.text_em_range_px().is_none());
        let frame = cpu_scene(graph.scene())?;
        check_optics(&frame);
        let saved = save_cdx_file(&output, &loaded, false, &cancel)?;
        assert_eq!(saved.byte_length, raw.len());
        assert_eq!(fs::read(&input)?, raw);
        assert_eq!(fs::read(&output)?, raw);
        let reopened = read_cdx_file(&output, &cancel)?.document;
        check_model(&reopened, dormant)?;
        assert_eq!(reopened.model(), loaded.model());
        assert_eq!(reopened.export_cdx()?, raw);
        let page = &reopened.model().pages()[0];
        let graph = drawing_page_scene(512, 512, &reopened, page.id(), &cancel)?;
        assert_eq!(cpu_scene(graph.scene())?, frame);
        let result = Command::new(std::env::var_os("BONDLINE_PYTHON").ok_or("pinned Python")?)
            .arg("-B")
            .arg(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("tests/fixtures/cdx-skeletal-cycle/check_rdkit.py"),
            )
            .arg(&output)
            .output()?;
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    Ok(())
}

#[test]
fn active_true_and_impossible_literal_h_do_not_become_hidden_cycle_ink() -> Result {
    let active = include_bytes!("fixtures/cdx-skeletal-cycle/active.cdx").as_slice();
    assert_eq!(
        StoredDocument::from_cdx(Arc::from(active), PageSize::A4)
            .err()
            .ok_or("active flag accepted")?
            .code(),
        "unsupported_content"
    );
    let impossible = include_bytes!("fixtures/cdx-skeletal-cycle/h-inconsistent.cdx").as_slice();
    let stored = StoredDocument::from_cdx(Arc::from(impossible), PageSize::A4)?;
    assert!(!stored.has_cdx_skeletal_preview() && !stored.has_display_projection());
    assert_eq!(stored.export_cdx()?, impossible);
    let page = &stored.model().pages()[0];
    assert_eq!(
        drawing_page_scene(512, 512, &stored, page.id(), &Cancellation::default())
            .err()
            .ok_or("invalid H was painted")?
            .code(),
        "unsupported_content"
    );
    Ok(())
}
