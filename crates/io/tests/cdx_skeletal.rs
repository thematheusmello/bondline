//! A closed, read-only CDX skeletal page keeps actual atom fields and unchanged source bytes.
use std::sync::Arc;

use bondline_io::{
    StoredDocument,
    cdx::{ElementalGraph, MappingError},
};
use bondline_model::{AtomId, PageSize};
use serde_json::{Value, json};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn read(bytes: &[u8]) -> Result<StoredDocument> {
    Ok(StoredDocument::from_cdx(Arc::from(bytes), PageSize::A4)?)
}

#[test]
fn independent_binary_inputs_and_rdkit_facts_reproduce() -> Result {
    let directory =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cdx-skeletal");
    let python = std::env::var_os("BONDLINE_PYTHON")
        .unwrap_or_else(|| if cfg!(windows) { "python" } else { "python3" }.into());
    for script in ["generate.py", "check_rdkit.py"] {
        let output = std::process::Command::new(&python)
            .arg("-B")
            .arg(directory.join(script))
            .output()?;
        assert!(
            output.status.success(),
            "{script}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

fn expected(h: [Option<u16>; 3], dormant: Option<(usize, &str)>) -> Value {
    let positions = [(160.0, 240.0), (224.0, 192.0), (288.0, 240.0)];
    let mut atoms = Vec::new();
    for (i, ((x, y), hydrogens)) in positions.into_iter().zip(h).enumerate() {
        let mut atom = json!({
            "id": i+1, "element": 6, "position": {"x_pt": x, "y_pt": y},
            "isotope": null, "charge": 0, "radical": "none", "explicit_hydrogens": hydrogens
        });
        if let Some((index, flag)) = dormant
            && i == index
        {
            atom[flag] = json!(true);
        }
        atoms.push(atom);
    }
    json!({"pages":[{"id":1,"size":{"width_pt":512.0,"height_pt":512.0},
        "fragments":[{"id":1,"atoms":atoms,"bonds":[
            {"id":1,"begin":1,"end":2,"order":"single"},{"id":2,"begin":2,"end":3,"order":"single"}
        ]}]}],"drawing_style":{"line_width_pt":4.0,"margin_width_pt":2.0}})
}

#[test]
fn skeletal_preview_retains_literal_or_absent_h_and_scoped_dormant_flag() -> Result {
    let h = [Some(3), Some(2), Some(3)];
    for (raw, hydrogens, dormant) in [
        (
            include_bytes!("fixtures/cdx-skeletal/propane.cdx").as_slice(),
            h,
            None,
        ),
        (
            include_bytes!("fixtures/cdx-skeletal/dormant_true.cdx").as_slice(),
            h,
            Some((1, "show_terminal_carbon_labels")),
        ),
        (
            include_bytes!("fixtures/cdx-skeletal/dormant_nonterminal.cdx").as_slice(),
            h,
            Some((0, "show_non_terminal_carbon_labels")),
        ),
        (
            include_bytes!("fixtures/cdx-skeletal/missing_h.cdx").as_slice(),
            [Some(3), None, Some(3)],
            None,
        ),
        (
            include_bytes!("fixtures/cdx-skeletal/all_h_missing.cdx").as_slice(),
            [None; 3],
            None,
        ),
    ] {
        let stored = read(raw)?;
        assert!(stored.has_cdx_skeletal_preview() && stored.has_display_projection());
        assert_eq!(
            serde_json::to_value(stored.model())?,
            expected(hydrogens, dormant)
        );
        assert_eq!(stored.export_cdx()?, raw);
        assert!(!stored.is_fully_mapped());
        assert!(!stored.has_cdx_caption_preview() && !stored.has_cdx_mixed_preview());
        assert!(!stored.has_cdx_label_edit_profile() && !stored.can_create_caption());
        assert!(!stored.can_edit_cdx_caption_run());
        assert!(!stored.can_edit_cdx_atom_label(AtomId::new(2).ok_or("atom")?));
        assert!(stored.to_cdxml().is_err());
        let mut changed = stored.clone();
        let page = changed.model().pages()[0].id();
        changed.model_mut().set_page_size(page, PageSize::A4)?;
        assert!(!changed.has_display_projection());
        assert!(changed.export_cdx().is_err());
    }
    Ok(())
}

#[test]
fn unsuitable_carbon_diagrams_retain_their_original_data_only_fallback() -> Result {
    for raw in [
        include_bytes!("fixtures/cdx-skeletal/inconsistent_h.cdx").as_slice(),
        include_bytes!("fixtures/cdx-skeletal/heteroatom.cdx").as_slice(),
        include_bytes!("fixtures/cdx-skeletal/charge.cdx").as_slice(),
        include_bytes!("fixtures/cdx-skeletal/isotope.cdx").as_slice(),
        include_bytes!("fixtures/cdx-skeletal/radical.cdx").as_slice(),
        include_bytes!("fixtures/cdx-skeletal/custom_root.cdx").as_slice(),
        include_bytes!("fixtures/cdx-skeletal/custom_node.cdx").as_slice(),
        include_bytes!("fixtures/cdx-skeletal/child_text.cdx").as_slice(),
        include_bytes!("fixtures/cdx-skeletal/collinear.cdx").as_slice(),
        include_bytes!("fixtures/cdx-skeletal/ethane.cdx").as_slice(),
        include_bytes!("fixtures/cdx-skeletal/off_paper.cdx").as_slice(),
        include_bytes!("fixtures/cdx-skeletal/missing_width.cdx").as_slice(),
        include_bytes!("fixtures/cdx-skeletal/root_flag.cdx").as_slice(),
    ] {
        let stored = read(raw)?;
        assert!(!stored.has_display_projection());
        assert_eq!(stored.model().pages()[0].size(), PageSize::A4);
        assert_eq!(stored.export_cdx()?, raw);
    }
    Ok(())
}

#[test]
fn active_visibility_and_bad_encodings_keep_the_existing_elemental_diagnostic() -> Result {
    for (raw, tag) in [
        (
            include_bytes!("fixtures/cdx-skeletal/terminal_visible.cdx").as_slice(),
            0x0442,
        ),
        (
            include_bytes!("fixtures/cdx-skeletal/interior_visible.cdx").as_slice(),
            0x0443,
        ),
        (
            include_bytes!("fixtures/cdx-skeletal/malformed_false.cdx").as_slice(),
            0x0443,
        ),
        (
            include_bytes!("fixtures/cdx-skeletal/duplicate_flag.cdx").as_slice(),
            0x0443,
        ),
        (
            include_bytes!("fixtures/cdx-skeletal/malformed_dormant.cdx").as_slice(),
            0x0442,
        ),
        (
            include_bytes!("fixtures/cdx-skeletal/duplicate_dormant.cdx").as_slice(),
            0x0442,
        ),
    ] {
        assert!(
            matches!(ElementalGraph::parse(raw), Err(MappingError::UnsupportedProperty {tag: actual,..}) if actual==tag)
        );
        assert!(read(raw).is_err());
    }
    Ok(())
}
