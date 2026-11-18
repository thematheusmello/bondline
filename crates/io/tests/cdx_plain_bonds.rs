//! Frozen original plain-bond sources and literal negative vectors before CDX IO support.
use bondline_io::{
    DocumentError, StoredDocument,
    cdx::{ElementalGraph, MappingError},
};
use bondline_model::{BondOrder, DoublePosition, PageSize};

#[path = "../../../xtask/oracles/cdx_bond_support.rs"]
#[allow(
    dead_code,
    reason = "shared frozen source, record and complete-model expectations"
)]
mod f;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn document(name: &str) -> Result<StoredDocument> {
    Ok(StoredDocument::from_cdx(
        f::source(name).into(),
        PageSize::A4,
    )?)
}
#[test]
fn double_and_triple_are_binary_masks_two_and_four_and_keep_source_direction() -> Result {
    for (name, order) in [("ethene", BondOrder::Double), ("ethyne", BondOrder::Triple)] {
        let raw = f::source(name);
        let graph = ElementalGraph::parse(raw)?;
        let bond = graph
            .fragments()
            .first()
            .ok_or("fragment")?
            .bonds()
            .first()
            .ok_or("bond")?;
        assert_eq!(bond.order, order);
        assert_eq!((bond.id.get(), bond.begin.get(), bond.end.get()), (1, 1, 2));
        assert_eq!(bond.double_position, DoublePosition::Unspecified);
        assert_eq!(graph.to_bytes(), raw);
    }
    Ok(())
}
#[test]
fn all_six_explicit_double_positions_preserve_geometric_side_without_inventing_mode() -> Result {
    for (name, side) in [
        ("auto_center", DoublePosition::Center),
        ("auto_right", DoublePosition::Right),
        ("auto_left", DoublePosition::Left),
        ("manual_center", DoublePosition::Center),
        ("manual_right", DoublePosition::Right),
        ("manual_left", DoublePosition::Left),
    ] {
        let graph = ElementalGraph::parse(f::source(name))?;
        assert_eq!(graph.fragments()[0].bonds()[0].double_position, side);
        assert_eq!(graph.to_bytes(), f::source(name));
    }
    Ok(())
}
#[test]
fn explicit_solid_is_known_data_and_never_changes_source_or_gives_display_authority() -> Result {
    let plain = document("single_absent")?;
    let solid = document("single_solid")?;
    assert_eq!(plain.model(), solid.model());
    assert_ne!(plain, solid);
    assert_eq!(solid.to_cdx()?.as_ref(), f::source("single_solid"));
    assert!(!solid.is_fully_mapped());
    assert!(!solid.has_display_projection());
    Ok(())
}

const HEADER: &[u8] = b"VjCD0100\x04\x03\x02\x01\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
const ORDER: u16 = 0x0600;
const DISPLAY: u16 = 0x0601;
const DISPLAY2: u16 = 0x0602;
const POSITION: u16 = 0x0603;
fn property(tag: u16, payload: &[u8]) -> Vec<u8> {
    assert!(payload.len() < usize::from(u16::MAX));
    let mut raw = tag.to_le_bytes().to_vec();
    raw.extend_from_slice(&(payload.len() as u16).to_le_bytes());
    raw.extend_from_slice(payload);
    raw
}
fn object(tag: u16, id: u32, children: &[Vec<u8>]) -> Vec<u8> {
    let mut raw = tag.to_le_bytes().to_vec();
    raw.extend_from_slice(&id.to_le_bytes());
    for child in children {
        raw.extend_from_slice(child);
    }
    raw.extend_from_slice(&[0, 0]);
    raw
}
fn root(children: &[Vec<u8>]) -> Vec<u8> {
    let mut raw = HEADER.to_vec();
    raw.extend_from_slice(&object(0x8000, 0, children));
    raw
}
fn atom(id: u32, h: u16, extra: &[Vec<u8>]) -> Vec<u8> {
    let mut fields = vec![
        property(0x0402, &6_i16.to_le_bytes()),
        property(0x042b, &h.to_le_bytes()),
    ];
    fields.extend_from_slice(extra);
    object(0x8004, id, &fields)
}
fn bond(fields: &[Vec<u8>]) -> Vec<u8> {
    let mut values = vec![
        property(0x0604, &73_u32.to_le_bytes()),
        property(0x0605, &149_u32.to_le_bytes()),
    ];
    values.extend_from_slice(fields);
    object(0x8005, 900, &values)
}
fn fragment(fields: &[Vec<u8>]) -> Vec<u8> {
    object(
        0x8003,
        51,
        &[atom(73, 3, &[]), atom(149, 3, &[]), bond(fields)],
    )
}
fn original(fields: &[Vec<u8>]) -> Vec<u8> {
    root(&[object(0x8001, 90, &[fragment(fields)])])
}

#[test]
fn known_bond_fields_cannot_hide_on_root_page_group_fragment_node_or_custom_owner() {
    let base = fragment(&[]);
    for tag in [DISPLAY, DISPLAY2, POSITION] {
        let field = property(tag, &[0, 0]);
        let mut implicit = HEADER.to_vec();
        implicit.extend_from_slice(&field);
        implicit.extend_from_slice(&object(0x8001, 90, std::slice::from_ref(&base)));
        implicit.extend_from_slice(&[0, 0]);
        for raw in [
            implicit,
            root(&[
                field.clone(),
                object(0x8001, 90, std::slice::from_ref(&base)),
            ]),
            root(&[object(0x8001, 90, &[field.clone(), base.clone()])]),
            root(&[object(
                0x8001,
                90,
                &[object(0x8002, 7, &[field.clone(), base.clone()])],
            )]),
            root(&[object(
                0x8001,
                90,
                &[object(
                    0x8003,
                    51,
                    &[
                        field.clone(),
                        atom(73, 3, &[]),
                        atom(149, 3, &[]),
                        bond(&[]),
                    ],
                )],
            )]),
            root(&[object(
                0x8001,
                90,
                &[object(
                    0x8003,
                    51,
                    &[
                        atom(73, 3, std::slice::from_ref(&field)),
                        atom(149, 3, &[]),
                        bond(&[]),
                    ],
                )],
            )]),
            root(&[object(
                0x8001,
                90,
                &[base.clone(), object(0xc123, 77, &[field])],
            )]),
        ] {
            assert!(
                matches!(ElementalGraph::parse(&raw),Err(MappingError::UnsupportedProperty {tag:actual,..}) if actual==tag)
            );
        }
    }
}

use bondline_io::{
    files::{read_cdx_file, save_cdx_file},
    mol::{MolTransform, Version},
};
use bondline_model::{Bond, Fragment, Point2};
use bondline_render::Cancellation;
use serde_json::Value;
use std::{borrow::Cow, fs, process::Command, sync::Arc};
#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(dead_code, reason = "existing isolated actual-file directory")]
mod files;

fn closed(document: &StoredDocument) {
    assert!(!document.is_fully_mapped());
    assert!(!document.has_display_projection());
    assert!(!document.has_boxed_atom_labels());
    assert!(!document.has_connected_atom_labels());
    assert!(!document.has_mixed_atom_labels());
    assert!(!document.has_flagged_atom_labels());
    assert!(!document.has_label_defaults_projection());
    assert!(!document.has_caption_projection());
    assert!(!document.has_smiles_projection());
}
#[test]
fn all_sixteen_actual_sources_match_independent_records_and_complete_model_without_authority()
-> Result {
    let vectors: Value =
        serde_json::from_str(include_str!("fixtures/cdx-plain-bonds/bond-vectors.json"))?;
    let cases = vectors["cases"].as_array().ok_or("cases")?;
    assert_eq!(cases.len(), 16);
    for case in cases {
        let name = case["name"].as_str().ok_or("case name")?;
        let raw = f::source(name);
        assert_eq!(f::read(raw)?, f::facts(name), "independent {name}");
        let own = document(name)?;
        assert_eq!(
            serde_json::to_value(own.model())?,
            f::model(name)?,
            "complete model {name}"
        );
        closed(&own);
        assert!(matches!(own.to_cdx()?, Cow::Borrowed(_)));
        assert_eq!(own.to_cdx()?.as_ref(), raw);
        let graph = ElementalGraph::parse(raw)?;
        assert_eq!(graph.as_bytes(), raw);
        assert_eq!(graph.to_bytes(), raw);
        let page = own.model().pages()[0].id();
        assert!(
            bondline_io::display::drawing_page_scene(
                1024,
                1024,
                &own,
                page,
                &Cancellation::default()
            )
            .is_err()
        );
    }
    Ok(())
}
#[test]
fn raw_modes_absence_and_directed_endpoints_remain_source_facts_not_normalized_geometry() -> Result
{
    for side in ["center", "right", "left"] {
        let auto = document(&format!("auto_{side}"))?;
        let manual = document(&format!("manual_{side}"))?;
        assert_eq!(auto.model(), manual.model());
        assert_ne!(auto, manual);
        assert_ne!(auto.to_cdx()?.as_ref(), manual.to_cdx()?.as_ref());
        assert_eq!(
            f::facts(&format!("auto_{side}"))["bonds"][0]["position_manual"],
            false
        );
        assert_eq!(
            f::facts(&format!("manual_{side}"))["bonds"][0]["position_manual"],
            true
        );
        closed(&auto);
        closed(&manual);
    }
    assert_ne!(
        document("ethene")?.model(),
        document("auto_center")?.model()
    );
    let unknown = document("ethene_unknown_h")?;
    assert!(
        unknown.model().pages()[0].fragments()[0]
            .atoms()
            .iter()
            .all(|a| a.explicit_hydrogens.is_none())
    );
    let reverse = document("reversed_right")?;
    let bond = &reverse.model().pages()[0].fragments()[0].bonds()[0];
    assert_eq!(
        (bond.begin.get(), bond.end.get(), bond.double_position),
        (2, 1, DoublePosition::Right)
    );
    assert_eq!(f::facts("reversed_right")["bonds"][0]["begin_source"], 149);
    assert_eq!(f::facts("reversed_right")["bonds"][0]["end_source"], 73);
    for name in [
        "reordered_right",
        "extended_right",
        "embedded_right",
        "implied_right",
    ] {
        assert_eq!(document(name)?.model(), document("manual_right")?.model());
    }
    Ok(())
}
fn kind(error: &MappingError) -> &'static str {
    match error {
        MappingError::Syntax(_) => "Syntax",
        MappingError::UnsupportedProperty { .. } => "UnsupportedProperty",
        MappingError::InvalidProperty { .. } => "InvalidProperty",
        MappingError::UnsupportedBondOrder { .. } => "UnsupportedBondOrder",
        MappingError::DuplicateProperty { .. } => "DuplicateProperty",
        _ => "unexpected error",
    }
}
#[test]
fn all_149_frozen_independent_refusal_sources_keep_exact_error_classes() -> Result {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let folder =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cdx-plain-bonds");
    let out=Command::new(python).args(["-X","utf8","-B","-c",r#"
import importlib.util,json,sys
from pathlib import Path
p=Path(sys.argv[1]);facts=json.loads((p/'bond-facts.json').read_text());sources={n:(p/(n+'.cdx')).read_bytes() for n in facts}
s=importlib.util.spec_from_file_location('frozen_bond_negatives',p/'negative_bonds.py');m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
print(json.dumps([{'name':n,'raw':list(raw),'error':e} for n,(raw,e) in m.negative_sources(sources,facts).items()]))
"#]).arg(folder).output()?;
    assert!(
        out.status.success(),
        "independent negatives: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let rows: Value = serde_json::from_slice(&out.stdout)?;
    let rows = rows.as_array().ok_or("negative cases")?;
    assert_eq!(rows.len(), 149);
    for case in rows {
        let raw: Vec<u8> = case["raw"]
            .as_array()
            .ok_or("negative bytes")?
            .iter()
            .map(|v| u8::try_from(v.as_u64().ok_or("byte value")?).map_err(Into::into))
            .collect::<Result<_>>()?;
        let error = ElementalGraph::parse(&raw)
            .err()
            .ok_or("independent invalid source accepted")?;
        assert_eq!(
            kind(&error),
            case["error"].as_str().ok_or("error class")?,
            "{}",
            case["name"]
        );
    }
    Ok(())
}
#[test]
fn every_published_unsupported_order_bit_and_pair_stays_a_query_or_other_unsupported_kind() {
    for bit in 0..16 {
        let value = 1_u16 << bit;
        if ![1, 2, 4].contains(&value) {
            let raw = original(&[property(ORDER, &value.to_le_bytes())]);
            assert!(matches!(
                ElementalGraph::parse(&raw),
                Err(MappingError::UnsupportedBondOrder { .. })
            ));
        }
        for other in bit + 1..16 {
            let raw = original(&[property(ORDER, &(value | (1_u16 << other)).to_le_bytes())]);
            assert!(matches!(
                ElementalGraph::parse(&raw),
                Err(MappingError::UnsupportedBondOrder { .. })
            ));
        }
    }
    for value in [0_u16, u16::MAX] {
        let raw = original(&[property(ORDER, &value.to_le_bytes())]);
        assert!(matches!(
            ElementalGraph::parse(&raw),
            Err(MappingError::UnsupportedBondOrder { .. })
        ));
    }
}
#[test]
fn non_double_explicit_secondary_style_or_position_is_refused_even_for_zero() {
    for order in [1_u16, 4] {
        for tag in [DISPLAY2, POSITION] {
            for value in [0_i16, 1, 2, 256, 257, 258, -1, 512] {
                let raw = original(&[
                    property(ORDER, &order.to_le_bytes()),
                    property(tag, &value.to_le_bytes()),
                ]);
                assert!(
                    matches!(ElementalGraph::parse(&raw),Err(MappingError::UnsupportedProperty{tag:actual,..}) if actual==tag)
                );
            }
        }
    }
    let raw = original(&[
        property(ORDER, &2_u16.to_le_bytes()),
        property(POSITION, &512_i16.to_le_bytes()),
    ]);
    assert!(matches!(
        ElementalGraph::parse(&raw),
        Err(MappingError::UnsupportedProperty { tag: POSITION, .. })
    ));
}
#[test]
fn duplicates_precede_scalar_role_or_width_fallbacks_and_keep_the_second_record() {
    for tag in [ORDER, DISPLAY, DISPLAY2, POSITION] {
        for payload in [[0_u8, 0], [1, 0]] {
            let mut fields = vec![property(ORDER, &2_u16.to_le_bytes())];
            if tag == ORDER {
                fields.clear();
                fields.push(property(ORDER, &2_u16.to_le_bytes()));
            } else {
                fields.push(property(tag, &[0, 0]));
            }
            fields.push(property(tag, &payload));
            let raw = original(&fields);
            let second_record = if tag == ORDER { 15 } else { 16 };
            assert!(
                matches!(ElementalGraph::parse(&raw),Err(MappingError::DuplicateProperty{tag:actual,record}) if actual==tag && record==second_record)
            );
        }
    }
    for tag in [DISPLAY2, POSITION] {
        let raw = original(&[property(tag, &[0, 0]), property(tag, &[0, 0])]);
        assert!(
            matches!(ElementalGraph::parse(&raw),Err(MappingError::DuplicateProperty{tag:actual,record:15}) if actual==tag)
        );
    }
}
fn changed_bond(base: &StoredDocument, change: impl FnOnce(&mut Bond)) -> Result<StoredDocument> {
    let page = &base.model().pages()[0];
    let fragment = &page.fragments()[0];
    let mut bonds = fragment.bonds().to_vec();
    change(bonds.first_mut().ok_or("bond")?);
    let mut own = base.clone();
    own.model_mut().replace_fragment(
        page.id(),
        Fragment::new(fragment.id(), fragment.atoms().to_vec(), bonds)?,
    )?;
    Ok(own)
}
#[test]
fn model_noops_keep_borrowed_source_and_bond_or_page_drift_requires_restoration() -> Result {
    let base = document("manual_right")?;
    let snapshot = base.model_snapshot();
    let mut same = base.clone();
    assert!(Arc::ptr_eq(&snapshot, &same.model_snapshot()));
    *same.model_mut() = base.model().clone();
    assert_eq!(same, base);
    assert!(matches!(same.to_cdx()?, Cow::Borrowed(_)));
    closed(&same);
    let mut changes = vec![
        changed_bond(&base, |b| b.order = BondOrder::Single)?,
        changed_bond(&base, |b| b.double_position = DoublePosition::Left)?,
        changed_bond(&base, |b| std::mem::swap(&mut b.begin, &mut b.end))?,
    ];
    let mut page = base.clone();
    page.model_mut()
        .set_page_size(base.model().pages()[0].id(), PageSize::new(420., 594.)?)?;
    changes.push(page);
    for mut own in changes {
        assert!(matches!(
            own.to_cdx(),
            Err(DocumentError::UnsupportedCdxEdit)
        ));
        closed(&own);
        *own.model_mut() = base.model().clone();
        assert_eq!(own.to_cdx()?.as_ref(), f::source("manual_right"));
    }
    let frame = MolTransform::new(1., Point2::new(0., 0.)?)?;
    for version in [Version::V2000, Version::V3000] {
        assert!(matches!(
            base.to_mol_as(version, frame),
            Err(DocumentError::CrossFormatSource {
                from: "cdx",
                to: "mol"
            })
        ));
    }
    assert!(matches!(
        base.to_cdxml(),
        Err(DocumentError::CrossFormatSource {
            from: "cdx",
            to: "cdxml"
        })
    ));
    Ok(())
}
#[test]
fn actual_unicode_files_preserve_source_and_cancellation_or_failed_save_never_publishes_drift()
-> Result {
    let dir = files::FileTestDirectory::new()?;
    let input = dir.path("plain-原-λ.cdx");
    let output = dir.path("plain-保存-λ.cdx");
    fs::write(&input, f::source("manual_right"))?;
    let loaded = read_cdx_file(&input, &Cancellation::default())?;
    assert_eq!(
        serde_json::to_value(loaded.document.model())?,
        f::model("manual_right")?
    );
    closed(&loaded.document);
    let receipt = save_cdx_file(&output, &loaded.document, false, &Cancellation::default())?;
    assert_eq!(receipt.byte_length, f::source("manual_right").len());
    let saved = fs::read(&output)?;
    assert_eq!(saved, f::source("manual_right"));
    assert_eq!(f::read(&saved)?, f::facts("manual_right"));
    assert_eq!(
        read_cdx_file(&output, &Cancellation::default())?.document,
        loaded.document
    );
    let cancel = Cancellation::default();
    cancel.cancel();
    let canceled = dir.path("cancel.cdx");
    assert!(read_cdx_file(&dir.path("absent.cdx"), &cancel).is_err());
    assert!(save_cdx_file(&canceled, &loaded.document, false, &cancel).is_err());
    assert!(!canceled.exists());
    assert!(save_cdx_file(&output, &loaded.document, false, &Cancellation::default()).is_err());
    assert_eq!(fs::read(&output)?, saved);
    let changed = changed_bond(&loaded.document, |b| {
        b.double_position = DoublePosition::Left
    })?;
    assert!(save_cdx_file(&output, &changed, true, &Cancellation::default()).is_err());
    assert_eq!(fs::read(&output)?, saved);
    assert_eq!(fs::read(input)?, f::source("manual_right"));
    assert_eq!(fs::read_dir(dir.path(".")).map(|x| x.count())?, 2);
    Ok(())
}
