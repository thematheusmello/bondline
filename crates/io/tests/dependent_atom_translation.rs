//! Original independently frozen Node.p/BoundingBox source deltas, not relayout tests.
use bondline_io::{DocumentError, StoredDocument};
use bondline_model::{Atom, Element, Fragment, PageSize, Point2};
use serde_json::Value;
use std::{
    borrow::Cow,
    io::Write,
    process::{Command, Stdio},
    sync::Arc,
};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn source(name: &str) -> Result<Vec<u8>> {
    Ok(std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/dependent-labels")
            .join(format!("{name}.cdxml")),
    )?)
}
fn stored(raw: &[u8]) -> Result<StoredDocument> {
    Ok(StoredDocument::from_cdxml(raw.into(), PageSize::A4)?)
}
fn case(name: &str) -> Result<StoredDocument> {
    stored(&source(name)?)
}
fn facts(name: &str) -> Result<Value> {
    let all: Value = serde_json::from_str(include_str!(
        "fixtures/dependent-labels/dependent-label-facts.json"
    ))?;
    Ok(all.get(name).ok_or("declared independent facts")?.clone())
}
fn independent(raw: &[u8]) -> Result<Value> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(python)
        .args([
            "-X",
            "utf8",
            "-B",
            "-c",
            include_str!("fixtures/dependent-labels/dependent_label_oracle.py"),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("independent stdin")?
        .write_all(raw)?;
    let out = child.wait_with_output()?;
    if !out.status.success() {
        return Err(format!(
            "independent Expat/Decimal/RDKit: {}",
            String::from_utf8_lossy(&out.stderr)
        )
        .into());
    }
    Ok(serde_json::from_slice(&out.stdout)?)
}
fn with_model_from(before: &str, after: &str) -> Result<StoredDocument> {
    let mut value = case(before)?;
    *value.model_mut() = case(after)?.model().clone();
    Ok(value)
}
fn assert_authority(value: &StoredDocument, connected: bool) {
    assert_eq!(value.has_boxed_atom_labels(), !connected);
    assert_eq!(value.has_connected_atom_labels(), connected);
    assert!(value.has_display_projection());
    assert!(!value.is_fully_mapped());
}
fn move_case(before: &str, after: &str, connected: bool) -> Result {
    let base = case(before)?;
    assert_authority(&base, connected);
    let value = with_model_from(before, after)?;
    assert_authority(&value, connected);
    let encoded = value.to_cdxml()?;
    assert_eq!(encoded.as_ref(), source(after)?);
    assert_eq!(independent(&encoded)?, facts(after)?);
    let reopened = stored(&encoded)?;
    assert_authority(&reopened, connected);
    assert_eq!(reopened.model(), value.model());
    assert_eq!(base.to_cdxml()?.as_ref(), source(before)?);
    Ok(())
}

#[test]
fn boxed_subset_has_current_delta_authority() -> Result {
    let value = with_model_from("boxed", "boxed_sub_p12_5_m7_25")?;
    assert_authority(&value, false);
    Ok(())
}
#[test]
fn boxed_subset_writer_splices_node_point_and_literal_box() -> Result {
    let value = with_model_from("boxed", "boxed_sub_p12_5_m7_25")?;
    assert_eq!(value.to_cdxml()?.as_ref(), source("boxed_sub_p12_5_m7_25")?);
    Ok(())
}
#[test]
fn boxed_subset_matches_independent_actual_file() -> Result {
    move_case("boxed", "boxed_sub_p12_5_m7_25", false)
}
#[test]
fn tolerated_imported_box_moves_literally_without_regeneration() -> Result {
    assert!(
        String::from_utf8(source("boxed_tolerated_sub_p12_5_m7_25")?)?
            .contains("BoundingBox=\"93.19141006469727 84.1015625")
    );
    move_case(
        "boxed_tolerated_raw",
        "boxed_tolerated_sub_p12_5_m7_25",
        false,
    )
}
#[test]
fn boxed_all_atoms_keep_ids_resources_and_lexemes() -> Result {
    move_case("boxed_pair", "boxed_pair_all_p24_m12", false)
}
#[test]
fn connected_subset_has_separate_current_delta_authority() -> Result {
    let value = with_model_from("connected", "connected_sub_p12_5_m7_25")?;
    assert_authority(&value, true);
    Ok(())
}
#[test]
fn connected_subset_writer_splices_node_point_and_literal_box() -> Result {
    let value = with_model_from("connected", "connected_sub_p12_5_m7_25")?;
    assert_eq!(
        value.to_cdxml()?.as_ref(),
        source("connected_sub_p12_5_m7_25")?
    );
    Ok(())
}
#[test]
fn connected_subset_all_and_component_moves_match_independent_files() -> Result {
    for (before, after) in [
        ("connected", "connected_sub_p12_5_m7_25"),
        ("connected", "connected_all_p24_m12"),
        ("connected_component", "connected_component_p24_m12"),
    ] {
        move_case(before, after, true)?;
    }
    Ok(())
}
#[test]
fn prior_label_edit_then_move_uses_current_box_not_original_box_plus_net_point() -> Result {
    let mut value = with_model_from("connected", "connected_nh")?;
    assert_authority(&value, true);
    assert_eq!(value.to_cdxml()?.as_ref(), source("connected_nh")?);
    *value.model_mut() = case("connected_nh_sub_p12_5_m7_25")?.model().clone();
    assert_authority(&value, true);
    let encoded = value.to_cdxml()?;
    assert_eq!(encoded.as_ref(), source("connected_nh_sub_p12_5_m7_25")?);
    assert_eq!(
        independent(&encoded)?,
        facts("connected_nh_sub_p12_5_m7_25")?
    );
    Ok(())
}

fn changed_atom(
    value: &StoredDocument,
    id: u32,
    edit: impl FnOnce(&mut Atom) -> Result,
) -> Result<StoredDocument> {
    let page = value.model().pages().first().ok_or("page")?;
    let fragment = page
        .fragments()
        .iter()
        .find(|fragment| fragment.atoms().iter().any(|a| a.id.get() == id))
        .ok_or("fragment")?;
    let mut atoms = fragment.atoms().to_vec();
    edit(
        atoms
            .iter_mut()
            .find(|atom| atom.id.get() == id)
            .ok_or("atom")?,
    )?;
    let mut result = value.clone();
    result.model_mut().replace_fragment(
        page.id(),
        Fragment::new(fragment.id(), atoms, fragment.bonds().to_vec())?,
    )?;
    Ok(result)
}
fn translated(value: &StoredDocument, id: u32, dx: f64, dy: f64) -> Result<StoredDocument> {
    changed_atom(value, id, |atom| {
        let before = atom.position.ok_or("point")?;
        let after = Point2::new(before.x_pt() + dx, before.y_pt() + dy)?;
        if after != before {
            atom.position = Some(after);
            if let Some(label) = atom.authored_label.as_ref() {
                let [l, t, r, b] = label.bounding_box().ok_or("box")?;
                atom.authored_label = Some(Arc::new(label.with_bounding_box(Some([
                    l + dx,
                    t + dy,
                    r + dx,
                    b + dy,
                ]))?));
            }
        }
        Ok(())
    })
}
fn refused(value: &StoredDocument) {
    assert!(!value.has_boxed_atom_labels());
    assert!(!value.has_connected_atom_labels());
    assert!(!value.has_display_projection());
    assert!(!value.is_fully_mapped());
    assert!(matches!(
        value.to_cdxml(),
        Err(DocumentError::UnsupportedEdit)
    ));
}

#[test]
fn current_connected_geometry_rejects_new_crossing_coincident_and_collinear_points() -> Result {
    // The new O-H crosses the independent horizontal H-H segment y=250 at x=220.
    let base = case("connected_crossing_base")?;
    assert_authority(&base, true);
    let crossing = translated(&base, 3, 0.0, 60.0)?;
    for atom in crossing.model().pages()[0].fragments()[0].atoms() {
        bondline_render::validate_atom_label(atom)?;
    }
    refused(&crossing);
    // Existing H at80,220 -> O at160,160; second H either coincides, overlaps the
    // same incident ray, or leaves an old H lying on the longer incident edge.
    for (dx, dy) in [(-160.0, 0.0), (-120.0, -30.0), (-240.0, 60.0)] {
        refused(&translated(&case("connected")?, 3, dx, dy)?);
    }
    assert_eq!(
        base.to_cdxml()?.as_ref(),
        source("connected_crossing_base")?
    );
    Ok(())
}

#[test]
fn point_only_or_box_only_large_changes_do_not_qualify_label_agreement() -> Result {
    let base = case("boxed")?;
    refused(&changed_atom(&base, 1, |atom| {
        let p = atom.position.ok_or("point")?;
        atom.position = Some(Point2::new(p.x_pt() + 12.5, p.y_pt() - 7.25)?);
        Ok(())
    })?);
    refused(&changed_atom(&base, 1, |atom| {
        let label = atom.authored_label.as_ref().ok_or("label")?;
        let [l, t, r, b] = label.bounding_box().ok_or("box")?;
        atom.authored_label = Some(Arc::new(label.with_bounding_box(Some([
            l + 12.5,
            t - 7.25,
            r + 12.5,
            b - 7.25,
        ]))?));
        Ok(())
    })?);
    Ok(())
}

#[test]
fn rounded_point_identical_keeps_literal_box_arc_and_borrowed_source() -> Result {
    let base = case("boxed_rounding_boundary")?;
    let label = base.model().pages()[0].fragments()[0].atoms()[0]
        .authored_label
        .as_ref()
        .ok_or("label")?;
    let left = label.bounding_box().ok_or("box")?[0];
    assert_eq!(128.0 + 1e-14, 128.0);
    assert_ne!(left + 1e-14, left);
    let value = translated(&base, 1, 1e-14, 0.0)?;
    let after = value.model().pages()[0].fragments()[0].atoms()[0]
        .authored_label
        .as_ref()
        .ok_or("label")?;
    assert!(Arc::ptr_eq(label, after));
    assert_eq!(base.model(), value.model());
    assert_authority(&value, false);
    assert!(matches!(value.to_cdxml()?, Cow::Borrowed(_)));
    assert_eq!(
        value.to_cdxml()?.as_ref(),
        source("boxed_rounding_boundary")?
    );
    Ok(())
}

#[test]
fn native_and_source_free_richer_labels_allow_only_point_changes_on_unlabelled_atoms() -> Result {
    let raw = String::from_utf8(source("boxed_pair")?)?.replace("</fragment>",
        "<n id='99' NodeType='Element' Element='6' Charge='0' Radical='None' NumHydrogens='4' p='300 300'/><b id='900' B='73' E='97' Order='1'/></fragment>");
    let foreign = stored(raw.as_bytes())?;
    assert!(!foreign.has_boxed_atom_labels());
    assert!(!foreign.has_connected_atom_labels());
    let mut pure = StoredDocument::default();
    *pure.model_mut() = foreign.model().clone();
    let native_bytes = pure.to_cdxml()?.into_owned();
    let native = stored(&native_bytes)?;
    for own in [&pure, &native] {
        assert_authority(own, false);
        let value = translated(own, 3, 24.0, -12.0)?;
        assert_authority(&value, false);
        let encoded = value.to_cdxml()?;
        let reopened = stored(&encoded)?;
        assert_eq!(reopened.model(), value.model());
        let forbidden = changed_atom(&value, 3, |atom| {
            atom.element = Element::try_from(7)?;
            atom.explicit_hydrogens = Some(3);
            Ok(())
        })?;
        if own.source_info().is_some() {
            assert!(!forbidden.has_boxed_atom_labels());
            assert!(!forbidden.has_connected_atom_labels());
            assert!(!forbidden.has_display_projection());
            assert!(!forbidden.is_fully_mapped());
            // Existing native serialization remains broader than the narrow label-delta grant.
            assert_eq!(stored(&forbidden.to_cdxml()?)?.model(), forbidden.model());
        }
        assert_eq!(
            own.model().pages()[0].fragments()[0].atoms()[2].position,
            Some(Point2::new(300.0, 300.0)?)
        );
    }
    Ok(())
}

#[test]
fn prior_source_proof_and_immutable_fields_cannot_be_repaired_by_movement() -> Result {
    let raw = String::from_utf8(source("connected")?)?
        .replace("</fragment>", "<opaque note='retained'/></fragment>");
    let opaque = stored(raw.as_bytes())?;
    assert_eq!(opaque.to_cdxml()?.as_ref(), raw.as_bytes());
    let mut repair = opaque.clone();
    *repair.model_mut() = case("connected_sub_p12_5_m7_25")?.model().clone();
    refused(&repair);
    for connected in [false, true] {
        let base = if connected {
            with_model_from("connected", "connected_sub_p12_5_m7_25")?
        } else {
            with_model_from("boxed", "boxed_sub_p12_5_m7_25")?
        };
        for field in 0..6 {
            let value = changed_atom(&base, 1, |atom| {
                match field {
                    0 => atom.charge = 1,
                    1 => atom.isotope = std::num::NonZeroU16::new(18),
                    2 => atom.authored_label = None,
                    3 => atom.show_terminal_carbon_labels = Some(true),
                    4 => atom.show_non_terminal_carbon_labels = Some(true),
                    _ => atom.position = None,
                }
                Ok(())
            })?;
            refused(&value);
        }
    }
    Ok(())
}

#[test]
fn unchanged_cloned_qualified_sources_keep_original_borrowed_bytes() -> Result {
    for name in [
        "boxed",
        "boxed_tolerated_raw",
        "boxed_pair",
        "connected",
        "connected_nh",
        "connected_component",
    ] {
        let base = case(name)?;
        let mut value = base.clone();
        *value.model_mut() = base.model().clone();
        assert_authority(&value, name.starts_with("connected"));
        assert!(matches!(value.to_cdxml()?, Cow::Borrowed(_)));
        assert_eq!(value.to_cdxml()?.as_ref(), source(name)?);
    }
    Ok(())
}

#[test]
fn all_frozen_sources_face_independent_expat_decimal_and_direct_rdkit() -> Result {
    let manifest: Value =
        serde_json::from_str(include_str!("fixtures/dependent-labels/move-vectors.json"))?;
    assert_eq!(manifest["base"], "9cf7103");
    let cases = manifest["cases"].as_array().ok_or("cases")?;
    assert_eq!(cases.len(), 19);
    for item in cases {
        let name = item["name"].as_str().ok_or("name")?;
        let raw = source(name)?;
        assert_eq!(
            u64::try_from(raw.len())?,
            item["bytes"].as_u64().ok_or("bytes")?
        );
        assert_eq!(independent(&raw)?, facts(name)?);
    }
    Ok(())
}
