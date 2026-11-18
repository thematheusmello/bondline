//! New model API qualification; separate from the existing-API XML runtime RED.
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Document, DoublePosition, DrawingSettings, Element,
    Fragment, FragmentId, PageId, PageSize, Point2,
};
use serde_json::json;
use std::sync::Arc;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn copy<T: Copy + Default + PartialEq>(value: T) -> T {
    value
}

#[test]
fn absence_single_and_both_values_are_copy_data_without_injected_defaults_or_old_json_changes()
-> TestResult {
    let empty = DrawingSettings::default();
    assert!(empty.is_empty());
    assert_eq!(empty.bond_length_pt(), None);
    assert_eq!(empty.chain_angle_degrees(), None);
    assert_eq!(serde_json::to_value(empty)?, json!({}));
    let mut document = Document::default();
    document.set_page_size(PageId::new(1).ok_or("page")?, PageSize::new(612.0, 792.0)?)?;
    let old =
        json!({"pages":[{"id":1,"size":{"width_pt":612.0,"height_pt":792.0},"fragments":[]}]});
    assert_eq!(serde_json::to_value(&document)?, old);
    assert_eq!(document.drawing_settings(), empty);
    for (length, angle) in [
        (None, None),
        (Some(24.5), None),
        (None, Some(120.5)),
        (Some(f64::from_bits(1)), Some(1.0)),
        (Some(100000.0), Some(179.0)),
        (Some(24.5), Some(120.5)),
    ] {
        let value = copy(DrawingSettings::new(length, angle)?);
        assert_eq!(value.bond_length_pt(), length);
        assert_eq!(value.chain_angle_degrees(), angle);
        assert_eq!(value.is_empty(), length.is_none() && angle.is_none());
        let mut expected = json!({});
        if let Some(v) = length {
            expected["bond_length_pt"] = json!(v);
        }
        if let Some(v) = angle {
            expected["chain_angle_degrees"] = json!(v);
        }
        assert_eq!(serde_json::to_value(value)?, expected);
        assert_eq!(
            serde_json::from_value::<DrawingSettings>(expected.clone())?,
            value
        );
        document.set_drawing_settings(value);
        assert_eq!(document.drawing_settings(), value);
        let mut expected_document = old.clone();
        if !value.is_empty() {
            expected_document["drawing_settings"] = expected;
        }
        assert_eq!(serde_json::to_value(&document)?, expected_document);
    }
    for value in [
        json!({}),
        json!({"bond_length_pt":null}),
        json!({"chain_angle_degrees":null}),
        json!({"bond_length_pt":null,"chain_angle_degrees":null}),
    ] {
        let empty = serde_json::from_value::<DrawingSettings>(value)?;
        assert_eq!(empty, DrawingSettings::default());
        assert_eq!(serde_json::to_value(empty)?, json!({}));
    }
    Ok(())
}

#[test]
fn typed_and_closed_map_validation_rejects_bad_domains_spellings_types_and_positional_arrays()
-> TestResult {
    for value in [
        0.0,
        -1.0,
        100001.0,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ] {
        assert!(DrawingSettings::new(Some(value), Some(120.5)).is_err());
    }
    for value in [0.0, 180.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(DrawingSettings::new(Some(24.5), Some(value)).is_err());
    }
    for value in [
        json!([]),
        json!([24.5, 120.5]),
        serde_json::Value::Null,
        json!({"BondLength":24.5}),
        json!({"chain_angle":120.5}),
        json!({"bond_length_pt":24.5,"unknown":0}),
        json!({"bond_length_pt":"24.5"}),
        json!({"chain_angle_degrees":true}),
        json!({"bond_length_pt":0}),
        json!({"bond_length_pt":100001}),
        json!({"chain_angle_degrees":180}),
        json!({"bond_length_pt":[24.5]}),
        json!({"chain_angle_degrees":{"value":120.5}}),
    ] {
        assert!(serde_json::from_value::<DrawingSettings>(value).is_err());
    }
    Ok(())
}

#[test]
fn style_only_changes_are_real_model_equality_and_keep_geometry_arcs_and_fixed_size_accounting()
-> TestResult {
    let mut first = Atom::new(
        AtomId::new(29).ok_or("atom")?,
        Element::try_from(6)?,
        Some(Point2::new(144.0, 144.0)?),
    );
    first.explicit_hydrogens = Some(3);
    first.show_terminal_carbon_labels = Some(false);
    let mut second = Atom::new(
        AtomId::new(7).ok_or("atom")?,
        Element::try_from(6)?,
        Some(Point2::new(208.0, 144.0)?),
    );
    second.explicit_hydrogens = Some(3);
    second.show_non_terminal_carbon_labels = Some(true);
    let mut document = Document::default();
    document.append_fragments(
        PageId::new(1).ok_or("page")?,
        vec![Fragment::new(
            FragmentId::new(17).ok_or("fragment")?,
            vec![first, second],
            vec![Bond {
                id: BondId::new(41).ok_or("bond")?,
                begin: AtomId::new(7).ok_or("begin")?,
                end: AtomId::new(29).ok_or("end")?,
                order: BondOrder::Single,
                double_position: DoublePosition::Unspecified,
            }],
        )?],
    )?;
    let before = document.clone();
    let graph = Arc::clone(&document.pages()[0].fragments()[0]);
    let bytes = Document::retained_bytes([&document]);
    let length = DrawingSettings::new(Some(24.5), None)?;
    document.set_drawing_settings(length);
    assert_ne!(document, before);
    assert_eq!(document.pages(), before.pages());
    assert!(Arc::ptr_eq(&graph, &document.pages()[0].fragments()[0]));
    assert_eq!(Document::retained_bytes([&document]), bytes);
    assert_eq!(Document::retained_bytes([&document, &document]), bytes);
    let length_only = document.clone();
    document.set_drawing_settings(DrawingSettings::new(Some(24.5), Some(120.5))?);
    assert_ne!(document, length_only);
    assert_eq!(document.pages(), before.pages());
    assert_eq!(document.clone(), document);
    document.set_drawing_settings(DrawingSettings::default());
    assert_eq!(document, before);
    assert_eq!(
        serde_json::to_value(&document)?,
        serde_json::to_value(before)?
    );
    Ok(())
}
