//! Independent frozen mixed-source existing-API declarations before profile authority.
use bondline_io::{DocumentError, StoredDocument};
use bondline_model::{Atom, Fragment, PageSize, Point2, Radical};
use serde_json::{Value, json};
use std::{
    borrow::Cow,
    io::Write,
    process::{Command, Stdio},
};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn source(name: &str) -> Result<Vec<u8>> {
    Ok(std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/mixed-labels")
            .join(format!("{name}.cdxml")),
    )?)
}
fn stored(raw: &[u8]) -> Result<StoredDocument> {
    Ok(StoredDocument::from_cdxml(raw.into(), PageSize::A4)?)
}
fn case(name: &str) -> Result<StoredDocument> {
    stored(&source(name)?)
}

#[test]
fn mixed_exact_source_gets_display_without_borrowing_font_defaults() -> Result {
    let own = case("amino_alcohol")?;
    let data = serde_json::to_value(own.model())?;
    assert_eq!(
        data["pages"][0]["fragments"][0]["atoms"][1]["show_non_terminal_carbon_labels"],
        json!(false)
    );
    assert!(
        data["pages"][0]["fragments"][0]["atoms"][1]
            .get("authored_label")
            .is_none()
    );
    assert!(!own.is_fully_mapped());
    assert!(!own.has_boxed_atom_labels());
    assert!(!own.has_connected_atom_labels());
    assert_eq!(own.to_cdxml()?.as_ref(), source("amino_alcohol")?);
    assert!(
        own.has_display_projection(),
        "complete explicit/hidden mixed source needs its separate display grant"
    );
    Ok(())
}
#[test]
fn mixed_source_captures_the_actual_current_page() -> Result {
    let own = case("amino_alcohol")?;
    let page = own.model().pages().first().ok_or("page")?;
    let scene =
        bondline_io::display::drawing_page_scene(1024, 1024, &own, page.id(), &Default::default())?;
    let frame = bondline_render::cpu_scene(scene.scene())?;
    assert_eq!([frame.width, frame.height], [1024, 1024]);
    frame.validate()?;
    Ok(())
}
#[test]
fn mixed_source_writer_synchronizes_an_existing_label_without_rewriting_hidden_carbons() -> Result {
    let mut own = case("amino_alcohol")?;
    *own.model_mut() = case("amino_diamine")?.model().clone();
    assert_eq!(own.to_cdxml()?.as_ref(), source("amino_diamine")?);
    Ok(())
}

fn manifest() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "fixtures/mixed-labels/mixed-vectors.json"
    ))?)
}
fn facts(name: &str) -> Result<Value> {
    let all: Value =
        serde_json::from_str(include_str!("fixtures/mixed-labels/mixed-label-facts.json"))?;
    Ok(all.get(name).ok_or("declared mixed facts")?.clone())
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
            include_str!("fixtures/mixed-labels/mixed_label_oracle.py"),
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
            "independent mixed Expat/Decimal/RDKit: {}",
            String::from_utf8_lossy(&out.stderr)
        )
        .into());
    }
    Ok(serde_json::from_slice(&out.stdout)?)
}
fn assert_mixed(value: &StoredDocument) {
    assert!(value.has_mixed_atom_labels());
    assert!(!value.has_boxed_atom_labels());
    assert!(!value.has_connected_atom_labels());
    assert!(value.has_display_projection());
    assert!(!value.is_fully_mapped());
}
fn with_model_from(before: &str, after: &str) -> Result<StoredDocument> {
    let mut value = case(before)?;
    *value.model_mut() = case(after)?.model().clone();
    Ok(value)
}
fn refused(value: &StoredDocument) {
    assert!(!value.has_mixed_atom_labels());
    assert!(!value.has_boxed_atom_labels());
    assert!(!value.has_connected_atom_labels());
    assert!(!value.has_display_projection());
    assert!(!value.is_fully_mapped());
    assert!(matches!(
        value.to_cdxml(),
        Err(DocumentError::UnsupportedEdit)
    ));
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
        .find(|f| f.atoms().iter().any(|a| a.id.get() == id))
        .ok_or("fragment")?;
    let mut atoms = fragment.atoms().to_vec();
    edit(atoms.iter_mut().find(|a| a.id.get() == id).ok_or("atom")?)?;
    let mut result = value.clone();
    result.model_mut().replace_fragment(
        page.id(),
        Fragment::new(fragment.id(), atoms, fragment.bonds().to_vec())?,
    )?;
    Ok(result)
}
fn attribute(raw: &str, id: u32, key: &str, value: Option<&str>) -> Result<String> {
    let start = raw.find(&format!("<n id='{id}' ")).ok_or("source node")?;
    let end = start
        + raw
            .get(start..)
            .ok_or("node suffix")?
            .find('>')
            .ok_or("node end")?;
    let part = raw.get(start..end).ok_or("node attributes")?;
    let prefix = format!(" {key}='");
    let begin = start + part.find(&prefix).ok_or("attribute")?;
    let value_start = begin + prefix.len();
    let value_end = value_start
        + raw
            .get(value_start..end)
            .ok_or("value suffix")?
            .find('\'')
            .ok_or("value end")?;
    let mut result = raw.to_owned();
    match value {
        Some(value) => result.replace_range(value_start..value_end, value),
        None => result.replace_range(begin..value_end + 1, ""),
    }
    assert_ne!(result, raw);
    Ok(result)
}
fn hidden_child(raw: &str, child: &str) -> Result<String> {
    let start = raw.find("<n id='97' ").ok_or("hidden node")?;
    let end = start
        + raw
            .get(start..)
            .ok_or("node suffix")?
            .find("/>")
            .ok_or("node end")?;
    let mut result = raw.to_owned();
    result.replace_range(end..end + 2, &format!(">{child}</n>"));
    Ok(result)
}
fn opaque(raw: &str) -> Result {
    let base = stored(raw.as_bytes())?;
    assert!(!base.has_mixed_atom_labels(), "unsupported source:{raw}");
    assert!(!base.has_boxed_atom_labels());
    assert!(!base.has_connected_atom_labels());
    assert!(!base.has_display_projection());
    assert!(!base.is_fully_mapped());
    assert!(matches!(base.to_cdxml()?, Cow::Borrowed(_)));
    assert_eq!(base.to_cdxml()?.as_ref(), raw.as_bytes());
    let mut same = base.clone();
    *same.model_mut() = base.model().clone();
    assert!(!same.has_mixed_atom_labels());
    assert!(matches!(same.to_cdxml()?, Cow::Borrowed(_)));
    let mut repaired = base.clone();
    *repaired.model_mut() = case("amino_diamine")?.model().clone();
    refused(&repaired);
    assert_eq!(base.to_cdxml()?.as_ref(), raw.as_bytes());
    Ok(())
}

#[test]
fn mixed_capability_is_separate_and_complete_for_all_frozen_sources() -> Result {
    let data = manifest()?;
    let rows = data["cases"].as_array().ok_or("cases")?;
    assert_eq!(rows.len(), 9);
    for row in rows {
        let name = row["name"].as_str().ok_or("name")?;
        let raw = source(name)?;
        let own = stored(&raw)?;
        assert_eq!(
            own.has_mixed_atom_labels(),
            row["mixed_expected"].as_bool().ok_or("admission")?,
            "case{name}"
        );
        assert!(!own.has_boxed_atom_labels());
        assert!(!own.has_connected_atom_labels());
        assert!(!own.is_fully_mapped());
        assert_eq!(own.has_display_projection(), own.has_mixed_atom_labels());
        assert!(matches!(own.to_cdxml()?, Cow::Borrowed(_)));
        assert_eq!(own.to_cdxml()?.as_ref(), raw);
        assert_eq!(independent(&raw)?, facts(name)?);
    }
    Ok(())
}
#[test]
fn frozen_hidden_only_subset_all_and_compound_moves_patch_existing_source_leaves() -> Result {
    for (before, after) in [
        ("amino_alcohol", "hidden_middle_p12_5_m7_25"),
        ("amino_alcohol", "subset_1_3_5_p12_5_m7_25"),
        ("amino_alcohol", "all_p24_m12"),
        ("amino_alcohol", "amino_diamine"),
        ("amino_alcohol", "edited_all_p24_m12"),
        ("amino_diamine", "edited_all_p24_m12"),
    ] {
        let base = case(before)?;
        assert_mixed(&base);
        let value = with_model_from(before, after)?;
        assert_mixed(&value);
        let encoded = value.to_cdxml()?;
        assert_eq!(encoded.as_ref(), source(after)?);
        assert_eq!(independent(&encoded)?, facts(after)?);
        let reopened = stored(&encoded)?;
        assert_mixed(&reopened);
        assert_eq!(reopened.model(), value.model());
        assert_eq!(base.to_cdxml()?.as_ref(), source(before)?);
    }
    Ok(())
}
#[test]
fn current_hidden_status_is_reproved_after_collinear_moves_and_adjacent_element_edits() -> Result {
    for (before, after) in [
        ("amino_alcohol", "mandatory_collinear"),
        ("methanol", "methanol_bad_c2"),
    ] {
        let base = case(before)?;
        assert_mixed(&base);
        let value = with_model_from(before, after)?;
        refused(&value);
        let page = value.model().pages().first().ok_or("page")?;
        assert!(matches!(
            bondline_io::display::drawing_page_scene(
                1024,
                1024,
                &value,
                page.id(),
                &Default::default()
            ),
            Err(bondline_io::display::DisplayError::IncompleteSource)
        ));
        let frame = bondline_render::graph_page_scene(1024, 1024, page)?;
        assert!(
            bondline_render::cpu_scene(frame.scene()).is_ok(),
            "renderability alone must not grant unknown foreign fonts"
        );
        assert_eq!(base.to_cdxml()?.as_ref(), source(before)?);
    }
    Ok(())
}
#[test]
fn hidden_metadata_is_strict_even_where_the_pure_classifier_still_returns_hidden() -> Result {
    let raw = String::from_utf8(source("amino_alcohol")?)?;
    for (name, modified) in [
        ("charge", attribute(&raw, 97, "Charge", Some("1"))?),
        ("radical", attribute(&raw, 97, "Radical", Some("Doublet"))?),
        (
            "isotope",
            raw.replace("<n id='97' ", "<n id='97' Isotope='13' "),
        ),
    ] {
        let value = stored(modified.as_bytes())?;
        let fragment = value.model().pages()[0]
            .fragments()
            .first()
            .ok_or("fragment")?;
        let hidden = bondline_render::hidden_carbon_atoms(fragment, &Default::default())?;
        assert!(
            hidden.iter().any(|id| id.get() == 2),
            "pure classifier {name} must not be mistaken for metadata authority"
        );
        assert!(independent(modified.as_bytes()).is_err());
        opaque(&modified)?;
    }
    Ok(())
}
#[test]
fn explicit_node_flags_hydrogens_and_supported_children_are_required() -> Result {
    let raw = String::from_utf8(source("amino_alcohol")?)?;
    let controls = [
        attribute(&raw, 97, "ShowNonTerminalCarbonLabels", Some("yes"))?,
        attribute(&raw, 97, "ShowNonTerminalCarbonLabels", None)?,
        attribute(&raw, 97, "NumHydrogens", Some("1"))?,
        attribute(&raw, 97, "NumHydrogens", None)?,
        attribute(&raw, 97, "Element", Some("7"))?,
        hidden_child(&raw, "<t unsupported='retain'/>")?,
        raw.replace(">OH</s>", ">N</s>"),
        raw.replace(" LineWidth='1.2500'", ""),
    ];
    for changed in controls {
        assert!(independent(changed.as_bytes()).is_err());
        opaque(&changed)?;
    }
    for changed in [
        hidden_child(&raw, "<t/> ")?,
        attribute(&raw, 97, "NodeType", None)?,
        attribute(&raw, 97, "Element", None)?,
        attribute(&raw, 97, "ShowNonTerminalCarbonLabels", None)?
            .replace("<CDXML ", "<CDXML ShowNonTerminalCarbonLabels='no' "),
    ] {
        opaque(&changed)?;
    }
    Ok(())
}
#[test]
fn raw_unsupported_child_is_not_replaced_by_projected_missing_label() -> Result {
    let raw = hidden_child(
        &String::from_utf8(source("amino_alcohol")?)?,
        "<t unsupported='retain'/>",
    )?;
    let value = stored(raw.as_bytes())?;
    assert!(
        value.model().pages()[0].fragments()[0].atoms()[1]
            .authored_label
            .is_none()
    );
    assert!(
        bondline_render::hidden_carbon_atoms(
            &value.model().pages()[0].fragments()[0],
            &Default::default()
        )?
        .iter()
        .any(|id| id.get() == 2)
    );
    opaque(&raw)
}
#[test]
fn complete_mixed_binding_keeps_unknown_styles_namespaces_and_lexical_features_opaque() -> Result {
    let raw = String::from_utf8(source("amino_alcohol")?)?;
    let dtd = format!("<!DOCTYPE CDXML []>{raw}");
    assert!(matches!(
        StoredDocument::from_cdxml(dtd.as_bytes().into(), PageSize::A4),
        Err(DocumentError::Mapping(
            bondline_io::cdxml::MappingError::Syntax(bondline_cdxml::Error::UnsupportedDtd)
        ))
    ));
    for changed in [
        raw.replace("<CDXML ", "<CDXML LabelFont='42' "),
        raw.replace("<n id='97' ", "<n id='97' LabelSize='12' "),
        raw.replace("<b id='900' ", "<b id='900' LineWidth='1.25' "),
        raw.replace("<CDXML ", "<CDXML xmlns:q='urn:unowned' "),
        raw.replace("<n id='97' ", "<n id='97' FutureValue='retain' "),
        format!("<?bondline-test retain?>{raw}"),
        raw.replace(">OH</s>", "><![CDATA[OH]]></s>"),
        raw.replace(" Display='Solid'", " Display='Dash'"),
        raw.replace("color='3'", "color='4'"),
        raw.replace("<color r='0' g='0' b='0'/>", "<color r='0' g='0' b='1'/>"),
    ] {
        opaque(&changed)?;
    }
    Ok(())
}
#[test]
fn dormant_flags_are_source_data_but_not_new_edit_permissions() -> Result {
    let base = case("amino_alcohol")?;
    let raw = String::from_utf8(source("amino_alcohol")?)?.replace(
        "ShowTerminalCarbonLabels='yes'",
        "ShowTerminalCarbonLabels='no'",
    );
    let reloaded = stored(raw.as_bytes())?;
    assert_mixed(&reloaded);
    assert_eq!(
        independent(raw.as_bytes())?["hidden_model_atoms"],
        json!([2, 3, 4])
    );
    let changed = changed_atom(&base, 3, |atom| {
        atom.show_terminal_carbon_labels = Some(false);
        Ok(())
    })?;
    assert_eq!(changed.model(), reloaded.model());
    refused(&changed);
    assert_eq!(base.to_cdxml()?.as_ref(), source("amino_alcohol")?);
    Ok(())
}
#[test]
fn source_static_geometry_and_hidden_numeric_fields_cannot_drift() -> Result {
    let base = case("amino_alcohol")?;
    for kind in 0..7 {
        let mut value = base.clone();
        let page = base.model().pages().first().ok_or("page")?;
        let fragment = page.fragments().first().ok_or("fragment")?;
        match kind {
            0 => value
                .model_mut()
                .set_page_size(page.id(), PageSize::new(513.0, 512.0)?)?,
            1 => value
                .model_mut()
                .set_drawing_style(base.model().drawing_style().with_margin_width(Some(3.0))?),
            2 => {
                value = changed_atom(&value, 2, |atom| {
                    atom.explicit_hydrogens = Some(1);
                    Ok(())
                })?;
            }
            3 => {
                value = changed_atom(&value, 2, |atom| {
                    atom.radical = Radical::Doublet;
                    Ok(())
                })?;
            }
            4 => {
                value = changed_atom(&value, 1, |atom| {
                    atom.authored_label = None;
                    Ok(())
                })?;
            }
            _ => {
                let mut atoms = fragment.atoms().to_vec();
                let mut bonds = fragment.bonds().to_vec();
                if kind == 5 {
                    atoms.swap(1, 2);
                } else {
                    let b = bonds.first_mut().ok_or("bond")?;
                    std::mem::swap(&mut b.begin, &mut b.end);
                }
                value
                    .model_mut()
                    .replace_fragment(page.id(), Fragment::new(fragment.id(), atoms, bonds)?)?;
            }
        }
        refused(&value);
    }
    Ok(())
}
#[test]
fn old_native_boxed_connected_and_loss_domains_remain_separate() -> Result {
    let boxed = stored(include_bytes!("fixtures/dependent-labels/boxed.cdxml"))?;
    assert!(boxed.has_boxed_atom_labels());
    assert!(!boxed.has_mixed_atom_labels());
    let connected = stored(include_bytes!("fixtures/connected-labels/water.cdxml"))?;
    assert!(connected.has_connected_atom_labels());
    assert!(!connected.has_mixed_atom_labels());
    let base = case("amino_alcohol")?;
    let mut pure = StoredDocument::default();
    *pure.model_mut() = base.model().clone();
    assert!(pure.has_boxed_atom_labels());
    assert!(!pure.has_mixed_atom_labels());
    let encoded = pure.to_cdxml()?;
    let native = stored(&encoded)?;
    assert_eq!(native.model(), pure.model());
    assert!(native.has_boxed_atom_labels());
    assert!(!native.has_mixed_atom_labels());
    assert!(!native.is_fully_mapped());
    let transform = bondline_io::mol::MolTransform::new(1.0, Point2::new(0.0, 0.0)?)?;
    assert!(base.to_mol(transform).is_err());
    assert!(base.to_cdx().is_err());
    Ok(())
}
#[test]
fn cloned_noops_keep_raw_source_and_complete_source_baseline() -> Result {
    for name in ["amino_alcohol", "amino_diamine", "methanol"] {
        let base = case(name)?;
        let mut same = base.clone();
        *same.model_mut() = base.model().clone();
        assert_mixed(&same);
        assert!(matches!(same.to_cdxml()?, Cow::Borrowed(_)));
        assert_eq!(same.to_cdxml()?.as_ref(), source(name)?);
        assert_eq!(base.to_cdxml()?.as_ref(), source(name)?);
    }
    Ok(())
}
