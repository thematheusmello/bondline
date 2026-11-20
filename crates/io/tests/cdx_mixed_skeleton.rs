//! Binary paper and exact-source ownership against independently authored CDXML controls.
use bondline_io::{StoredDocument, display::drawing_page_scene_with_view};
use bondline_model::{PageSize, Point2};
use bondline_render::{Cancellation, HitTarget, PageView, cpu_scene};
use std::{path::Path, sync::Arc};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn fixture(name: &str) -> Result<Vec<u8>> {
    Ok(std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name),
    )?)
}

#[test]
fn hidden_vertices_and_printed_labels_share_a_source_certified_binary_drawing() -> Result {
    let cuts: std::collections::BTreeMap<String, Vec<[[f64; 2]; 2]>> =
        serde_json::from_slice(include_bytes!("fixtures/cdx-mixed-skeleton/clearance.json"))?;
    for (case, reference) in [
        ("methanol", "mixed-labels/methanol.cdxml"),
        ("literal_h", "mixed-labels/methanol.cdxml"),
        ("ethanol", "cdx-mixed-skeleton/ethanol.cdxml"),
        ("amino_alcohol", "mixed-labels/amino_alcohol.cdxml"),
    ] {
        let bytes = fixture(&format!("cdx-mixed-skeleton/{case}.cdx"))?;
        let stored = StoredDocument::from_cdx(Arc::from(bytes.as_slice()), PageSize::A4)?;
        assert!(
            stored.has_cdx_skeletal_label_preview() && stored.has_display_projection(),
            "{case}"
        );
        assert!(!stored.has_cdx_skeletal_preview() && !stored.has_cdx_label_edit_profile());
        assert!(
            !stored.can_create_caption() && !stored.is_fully_mapped() && stored.to_cdxml().is_err()
        );
        let page = &stored.model().pages()[0];
        assert_eq!(page.size(), PageSize::new(512.0, 512.0)?);
        let fragment = &page.fragments()[0];
        assert_eq!(stored.export_cdx()?, bytes);
        assert!(stored.can_move_cdx_skeletal_atoms());
        let control = StoredDocument::from_cdxml(Arc::from(fixture(reference)?), PageSize::A4)?;
        let cf = &control.model().pages()[0].fragments()[0];
        assert_eq!(fragment.bonds(), cf.bonds());
        assert_eq!(fragment.atoms().len(), cf.atoms().len());
        for (atom, witness) in fragment.atoms().iter().zip(cf.atoms()) {
            assert_eq!(
                (atom.element, atom.position, &atom.authored_label),
                (witness.element, witness.position, &witness.authored_label)
            );
            if atom.authored_label.is_some() || case == "literal_h" {
                assert_eq!(atom.explicit_hydrogens, witness.explicit_hydrogens);
            } else {
                assert_eq!(atom.explicit_hydrogens, None);
            }
            let expected_flag = |v| (case != "literal_h" && v == Some(true)).then_some(true);
            assert_eq!(
                atom.show_terminal_carbon_labels,
                expected_flag(witness.show_terminal_carbon_labels)
            );
            assert_eq!(
                atom.show_non_terminal_carbon_labels,
                expected_flag(witness.show_non_terminal_carbon_labels)
            );
            assert!(!stored.can_edit_cdx_atom_label(atom.id));
        }
        let cancel = Cancellation::default();
        for scale in [1, 2] {
            let view = PageView::zoom(
                512 * scale,
                512 * scale,
                page.size(),
                f64::from(100 * scale),
                Point2::new(256.0, 256.0)?,
            )?;
            let graph = drawing_page_scene_with_view(&view, &stored, page.id(), &cancel)?;
            let other = drawing_page_scene_with_view(
                &view,
                &control,
                control.model().pages()[0].id(),
                &cancel,
            )?;
            let frame = cpu_scene(graph.scene())?;
            assert_eq!(
                frame.rgba,
                cpu_scene(other.scene())?.rgba,
                "{case}, {scale}x"
            );
            for atom in fragment.atoms() {
                let xy = atom.position.ok_or("missing position")?;
                assert_eq!(
                    graph.hit_test(
                        [
                            xy.x_pt() as f32 * scale as f32,
                            xy.y_pt() as f32 * scale as f32
                        ],
                        0.0
                    )?,
                    Some(HitTarget::Atom {
                        fragment_id: fragment.id(),
                        atom_id: atom.id
                    })
                );
            }
            let bounds = &cuts[if case == "literal_h" {
                "methanol"
            } else {
                case
            }];
            assert_eq!(fragment.bonds().len(), bounds.len());
            for (bond, expected) in fragment.bonds().iter().zip(bounds) {
                let pos = |id| {
                    fragment
                        .atoms()
                        .iter()
                        .find(|a| a.id == id)
                        .and_then(|a| a.position)
                        .ok_or("no endpoint")
                };
                let a = pos(bond.begin)?;
                let b = pos(bond.end)?;
                let rail = graph
                    .bond_strokes(fragment.id(), bond.id)
                    .next()
                    .ok_or("missing rail")?;
                for ((at, endpoint), [lo, hi]) in
                    [(a, rail.begin), (b, rail.end)].into_iter().zip(expected)
                {
                    let x = f64::from(endpoint[0]) / f64::from(scale) - at.x_pt();
                    let y = f64::from(endpoint[1]) / f64::from(scale) - at.y_pt();
                    let cut = x.hypot(y);
                    // Independent outline envelope and the renderer's published f32/tessellation allowance.
                    let tolerance = 0.0001 + 8.0 * f64::from(f32::EPSILON) * 512.0;
                    assert!(
                        cut >= lo - tolerance && cut <= hi + tolerance,
                        "{case} {cut} outside {lo}..{hi}"
                    );
                }
                let point = [a.x_pt() + b.x_pt(), a.y_pt() + b.y_pt()]
                    .map(|v| (v * f64::from(scale) / 2.0) as f32);
                assert_eq!(
                    graph.hit_test(point, 0.0)?,
                    Some(HitTarget::Bond {
                        fragment_id: fragment.id(),
                        bond_id: bond.id
                    })
                );
            }
            if scale == 1 {
                let pixels =
                    |x: usize, y: usize| &frame.rgba[(y * 512 + x) * 4..(y * 512 + x + 1) * 4];
                let (ink, white, rail) = if case == "amino_alcohol" {
                    ((93, 160), (96, 160), (132, 184))
                } else {
                    ((205, 144), (208, 144), (167, 168))
                };
                assert_eq!(pixels(ink.0, ink.1), &[51, 102, 204, 255]);
                assert_eq!(pixels(white.0, white.1), &[255; 4]);
                assert_eq!(pixels(rail.0, rail.1), &[0, 0, 0, 255]);
            }
        }
        let mut detached = stored.clone();
        detached
            .model_mut()
            .set_page_size(page.id(), PageSize::A4)?;
        assert!(!detached.has_display_projection() && detached.export_cdx().is_err());
    }
    Ok(())
}

#[test]
fn conflicting_geometry_opaque_flags_and_unmatched_unicode_keep_source_behavior() -> Result {
    for case in [
        "hidden_oxygen",
        "missing_label_h",
        "overlap",
        "bad_box",
        "coincident",
        "literal_utf8",
    ] {
        let bytes = fixture(&format!("cdx-mixed-skeleton/{case}.cdx"))?;
        let stored = StoredDocument::from_cdx(Arc::from(bytes.as_slice()), PageSize::A4)?;
        assert!(!stored.has_display_projection(), "{case}");
        assert_eq!(stored.export_cdx()?, bytes);
        if case == "literal_utf8" {
            let atoms = stored.model().pages()[0].fragments()[0].atoms();
            assert_eq!(
                atoms[1].authored_label.as_ref().ok_or("owned text")?.runs()[0].text(),
                "Oη"
            );
        }
    }
    for case in ["malformed_flag", "active_flag"] {
        let bytes = fixture(&format!("cdx-mixed-skeleton/{case}.cdx"))?;
        assert!(
            StoredDocument::from_cdx(Arc::from(bytes), PageSize::A4).is_err(),
            "{case}"
        );
    }
    Ok(())
}
