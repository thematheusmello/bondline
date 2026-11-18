//! Literal CDX Node metadata; independent byte declarations precede producer support.
use bondline_io::cdx::{ElementalGraph, MappingError};
use bondline_model::Radical;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const HEADER: &[u8] = b"VjCD0100\x04\x03\x02\x01\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
const ISOTOPE: u16 = 0x0420;
const CHARGE: u16 = 0x0421;
const RADICAL: u16 = 0x0422;
const HYDROGENS: u16 = 0x042b;

fn property(tag: u16, payload: &[u8]) -> Vec<u8> {
    let mut bytes = tag.to_le_bytes().to_vec();
    assert!(payload.len() < usize::from(u16::MAX));
    bytes.extend_from_slice(&(payload.len() as u16).to_le_bytes());
    bytes.extend_from_slice(payload);
    bytes
}
fn object(tag: u16, id: u32, children: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = tag.to_le_bytes().to_vec();
    bytes.extend_from_slice(&id.to_le_bytes());
    for child in children {
        bytes.extend_from_slice(child);
    }
    bytes.extend_from_slice(&[0, 0]);
    bytes
}
fn root(children: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = HEADER.to_vec();
    bytes.extend_from_slice(&object(0x8000, 0, children));
    bytes
}
fn fragment(nodes: &[Vec<u8>]) -> Vec<u8> {
    object(0x8003, 100, nodes)
}
fn node(fields: &[Vec<u8>]) -> Vec<u8> {
    object(0x8004, 101, fields)
}
fn original(fields: &[Vec<u8>]) -> Vec<u8> {
    root(&[object(0x8001, 42, &[fragment(&[node(fields)])])])
}

#[test]
fn canonical_literal_node_fields_are_projected_and_complete_source_stays_unchanged() -> Result {
    let bytes = original(&[
        property(ISOTOPE, &[13, 0]),
        property(CHARGE, &[255]),
        property(RADICAL, &[2]),
        property(HYDROGENS, &[0, 0]),
    ]);
    let graph = ElementalGraph::parse(&bytes)?;
    let atom = graph
        .fragments()
        .first()
        .ok_or("fragment")?
        .atoms()
        .first()
        .ok_or("atom")?;
    assert_eq!(atom.isotope.map(|value| value.get()), Some(13));
    assert_eq!(atom.charge, -1);
    assert_eq!(atom.radical, Radical::Doublet);
    assert_eq!(atom.explicit_hydrogens, Some(0));
    assert_eq!(graph.as_bytes(), bytes);
    assert_eq!(graph.to_bytes(), bytes);
    Ok(())
}

#[test]
fn explicit_zeroes_are_literal_and_missing_hydrogens_remain_unresolved() -> Result {
    let bytes = root(&[object(
        0x8001,
        42,
        &[fragment(&[
            node(&[
                property(ISOTOPE, &[0, 0]),
                property(CHARGE, &[0]),
                property(RADICAL, &[0]),
                property(HYDROGENS, &[0, 0]),
            ]),
            object(0x8004, 102, &[]),
        ])],
    )]);
    let graph = ElementalGraph::parse(&bytes)?;
    let atoms = graph.fragments().first().ok_or("fragment")?.atoms();
    let present = atoms.first().ok_or("present atom")?;
    let missing = atoms.get(1).ok_or("missing atom")?;
    assert_eq!(
        (present.isotope, present.charge, present.radical),
        (None, 0, Radical::None)
    );
    assert_eq!(
        (missing.isotope, missing.charge, missing.radical),
        (None, 0, Radical::None)
    );
    assert_eq!(present.explicit_hydrogens, Some(0));
    assert_eq!(missing.explicit_hydrogens, None);
    Ok(())
}

#[test]
fn documented_four_byte_charge_uses_signed_high_byte_and_retains_low_bytes() -> Result {
    for (raw, expected) in [
        ([1, 0, 0, 0], 0_i8),
        ([0, 0, 0, 1], 1),
        ([0, 0, 0, 255], -1),
        ([170, 187, 204, 127], 127),
        ([0, 0, 0, 128], -128),
    ] {
        let bytes = original(&[property(CHARGE, &raw)]);
        let graph = ElementalGraph::parse(&bytes)?;
        assert_eq!(graph.fragments()[0].atoms()[0].charge, expected);
        assert_eq!(graph.to_bytes(), bytes);
    }
    Ok(())
}

#[test]
fn known_atom_fields_are_never_silently_ignored_on_other_owners() -> Result {
    let base = fragment(&[node(&[])]);
    for field in [
        property(ISOTOPE, &[13, 0]),
        property(CHARGE, &[255]),
        property(RADICAL, &[2]),
        property(HYDROGENS, &[0, 0]),
    ] {
        let mut implicit = HEADER.to_vec();
        implicit.extend_from_slice(&field);
        implicit.extend_from_slice(&object(0x8001, 42, std::slice::from_ref(&base)));
        implicit.extend_from_slice(&[0, 0]);
        let documents = [
            implicit,
            root(&[
                field.clone(),
                object(0x8001, 42, std::slice::from_ref(&base)),
            ]),
            root(&[object(0x8001, 42, &[field.clone(), base.clone()])]),
            root(&[object(
                0x8001,
                42,
                &[object(0x8002, 99, &[field.clone(), base.clone()])],
            )]),
            root(&[object(0x8001, 42, &[fragment(&[field.clone(), node(&[])])])]),
            root(&[object(
                0x8001,
                42,
                &[base.clone(), object(0xc123, 77, &[field])],
            )]),
        ];
        for bytes in documents {
            assert!(matches!(
                ElementalGraph::parse(&bytes),
                Err(MappingError::UnsupportedProperty { .. })
            ));
        }
    }
    Ok(())
}

use bondline_io::{
    DocumentError, StoredDocument,
    mol::{MolTransform, Version},
};
use bondline_model::{Atom, Fragment, PageSize, Point2};
use serde_json::{Value, json};
use std::{borrow::Cow, sync::Arc};

#[path = "../../../xtask/oracles/cdx_literal_support.rs"]
#[allow(
    dead_code,
    reason = "shared frozen independent binary sources and actual-file facts"
)]
mod oracle;

fn literal_atom(fields: &[Vec<u8>]) -> Result<Atom> {
    let bytes = original(fields);
    Ok(ElementalGraph::parse(&bytes)?.fragments()[0].atoms()[0].clone())
}

#[test]
fn literal_scalar_domains_preserve_signedness_and_all_radical_states() -> Result {
    for value in [0_i16, 1, 2, 13, i16::MAX] {
        assert_eq!(
            literal_atom(&[property(ISOTOPE, &value.to_le_bytes())])?
                .isotope
                .map(|v| v.get()),
            u16::try_from(value).ok().filter(|v| *v != 0)
        );
    }
    for value in [i8::MIN, -1, 0, 1, i8::MAX] {
        assert_eq!(
            literal_atom(&[property(CHARGE, &value.to_le_bytes())])?.charge,
            value
        );
    }
    for (value, state) in [
        (0, Radical::None),
        (1, Radical::Singlet),
        (2, Radical::Doublet),
        (3, Radical::Triplet),
    ] {
        assert_eq!(literal_atom(&[property(RADICAL, &[value])])?.radical, state);
    }
    for value in [0_u16, 1, 2, 256, u16::MAX] {
        assert_eq!(
            literal_atom(&[property(HYDROGENS, &value.to_le_bytes())])?.explicit_hydrogens,
            Some(value)
        );
    }
    for (tag, raw) in [
        (ISOTOPE, vec![255, 255]),
        (ISOTOPE, vec![0, 128]),
        (RADICAL, vec![4]),
        (RADICAL, vec![255]),
    ] {
        let bytes = original(&[property(tag, &raw)]);
        assert!(
            matches!(ElementalGraph::parse(&bytes), Err(MappingError::InvalidProperty { tag: actual, .. }) if actual == tag)
        );
    }
    Ok(())
}

#[test]
fn unsupported_payload_widths_are_distinct_from_accepted_width_domain_errors() -> Result {
    for (tag, allowed) in [
        (ISOTOPE, &[2_usize][..]),
        (CHARGE, &[1_usize, 4][..]),
        (RADICAL, &[1_usize][..]),
        (HYDROGENS, &[2_usize][..]),
    ] {
        for size in 0..=8 {
            if allowed.contains(&size) {
                continue;
            }
            let bytes = original(&[property(tag, &vec![0; size])]);
            let err = ElementalGraph::parse(&bytes)
                .err()
                .ok_or("unsupported width accepted")?;
            assert!(
                matches!(err, MappingError::UnsupportedProperty { tag: actual, .. } if actual == tag)
            );
            assert_eq!(err.code(), "unsupported_content");
        }
    }
    Ok(())
}

#[test]
fn duplicate_mapped_properties_are_rejected_on_the_second_record_even_for_equal_zeroes() {
    for (tag, zero, other) in [
        (ISOTOPE, vec![0, 0], vec![13, 0]),
        (CHARGE, vec![0], vec![255]),
        (RADICAL, vec![0], vec![2]),
        (HYDROGENS, vec![0, 0], vec![1, 0]),
    ] {
        for second in [zero.clone(), other] {
            let bytes = original(&[property(tag, &zero), property(tag, &second)]);
            assert!(
                matches!(ElementalGraph::parse(&bytes), Err(MappingError::DuplicateProperty { tag: actual, record:5 }) if actual == tag)
            );
        }
    }
    let bytes = original(&[property(CHARGE, &[0]), property(CHARGE, &[1, 0, 0, 0])]);
    assert!(matches!(
        ElementalGraph::parse(&bytes),
        Err(MappingError::DuplicateProperty {
            tag: CHARGE,
            record: 5
        })
    ));
}

#[test]
fn mapped_atom_fields_on_bonds_and_custom_owners_cannot_be_mistaken_for_other_metadata() {
    for field in [
        property(ISOTOPE, &[13, 0]),
        property(CHARGE, &[255]),
        property(RADICAL, &[2]),
        property(HYDROGENS, &[0, 0]),
    ] {
        let bytes = root(&[object(
            0x8001,
            42,
            &[fragment(&[
                node(&[]),
                object(0x8004, 102, &[]),
                object(
                    0x8005,
                    103,
                    &[
                        property(0x0604, &101_u32.to_le_bytes()),
                        property(0x0605, &102_u32.to_le_bytes()),
                        field,
                    ],
                ),
            ])],
        )]);
        assert!(matches!(
            ElementalGraph::parse(&bytes),
            Err(MappingError::UnsupportedProperty { .. })
        ));
    }
}

#[test]
fn truncated_record_framing_is_a_syntax_error_instead_of_a_scalar_width_guess() {
    let bytes = original(&[property(HYDROGENS, &[2, 0])]);
    for remove in 1..=12 {
        let end = bytes.len() - remove;
        assert!(matches!(
            ElementalGraph::parse(&bytes[..end]),
            Err(MappingError::Syntax(_))
        ));
    }
    for tail in [
        vec![0x20],
        vec![0x20, 0x04, 2],
        vec![0x20, 0x04, 0xff, 0xff, 2],
        vec![0x20, 0x04, 0xff, 0xff, 2, 0, 0, 0, 13],
    ] {
        let mut raw = HEADER.to_vec();
        raw.extend_from_slice(&tail);
        assert!(matches!(
            ElementalGraph::parse(&raw),
            Err(MappingError::Syntax(_))
        ));
    }
}

fn no_source_authority(document: &StoredDocument) {
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
fn literal_data_does_not_enable_source_encoding_display_or_cross_format_loss() -> Result {
    let bytes = original(&[
        property(ISOTOPE, &[13, 0]),
        property(CHARGE, &[255]),
        property(RADICAL, &[2]),
        property(HYDROGENS, &[0, 0]),
    ]);
    let source: Arc<[u8]> = bytes.clone().into();
    let pointer = source.as_ptr();
    let document = StoredDocument::from_cdx(source, PageSize::A4)?;
    no_source_authority(&document);
    assert_eq!(document.export_cdx()?.as_ptr(), pointer);
    assert!(matches!(document.to_cdx()?, Cow::Borrowed(_)));
    assert_eq!(document.to_cdx()?.as_ref(), bytes);
    assert!(matches!(
        document.to_cdxml(),
        Err(DocumentError::CrossFormatSource {
            from: "cdx",
            to: "cdxml"
        })
    ));
    let frame = MolTransform::new(1.0, Point2::new(0.0, 0.0)?)?;
    for version in [Version::V2000, Version::V3000] {
        assert!(matches!(
            document.to_mol_as(version, frame),
            Err(DocumentError::CrossFormatSource {
                from: "cdx",
                to: "mol"
            })
        ));
    }
    let mut changed = document.clone();
    // Existing detached preview-page edits stay allowed; source publication still refuses.
    changed.model_mut().set_page_size(
        document.model().pages()[0].id(),
        PageSize::new(420.0, 594.0)?,
    )?;
    assert!(matches!(
        changed.to_cdx(),
        Err(DocumentError::UnsupportedCdxEdit)
    ));
    assert!(changed.source_info().ok_or("source")?.modified);
    no_source_authority(&changed);
    *changed.model_mut() = document.model().clone();
    assert_eq!(changed.to_cdx()?.as_ref(), bytes);
    assert!(!changed.source_info().ok_or("source")?.modified);
    no_source_authority(&changed);
    Ok(())
}

#[test]
fn same_modeled_defaults_do_not_erase_raw_property_presence_or_charge_low_bytes() -> Result {
    let absent = original(&[]);
    let zeroes = original(&[
        property(ISOTOPE, &[0, 0]),
        property(CHARGE, &[0]),
        property(RADICAL, &[0]),
    ]);
    let low = original(&[property(CHARGE, &[1, 0, 0, 0])]);
    let a = StoredDocument::from_cdx(absent.clone().into(), PageSize::A4)?;
    for raw in [zeroes, low] {
        let b = StoredDocument::from_cdx(raw.clone().into(), PageSize::A4)?;
        assert_eq!(a.model(), b.model());
        assert_ne!(a, b);
        assert_eq!(a.to_cdx()?.as_ref(), absent);
        assert_eq!(b.to_cdx()?.as_ref(), raw);
        no_source_authority(&b);
    }
    Ok(())
}

fn compare_model(graph: &ElementalGraph<'_>, expected: &Value) -> Result {
    let atoms = expected["atoms"].as_array().ok_or("expected atoms")?;
    let bonds = expected["bonds"].as_array().ok_or("expected bonds")?;
    let fragments = expected["fragments"]
        .as_array()
        .ok_or("expected fragments")?;
    let mut wanted = Vec::new();
    for fragment in fragments {
        let ids = fragment["atom_ids"].as_array().ok_or("fragment atoms")?;
        let bond_ids = fragment["bond_ids"].as_array().ok_or("fragment bonds")?;
        let mut fragment_atoms = Vec::new();
        let mut fragment_bonds = Vec::new();
        for id in ids {
            let atom = atoms
                .iter()
                .find(|a| a["model_id"] == *id)
                .ok_or("expected atom id")?;
            fragment_atoms.push(json!({"id":atom["model_id"],"element":atom["element"],"position":atom["position"],"isotope":atom["isotope"],"charge":atom["charge"],"radical":atom["radical"],"explicit_hydrogens":atom["explicit_hydrogens"]}));
        }
        for id in bond_ids {
            let bond = bonds
                .iter()
                .find(|b| b["model_id"] == *id)
                .ok_or("expected bond id")?;
            fragment_bonds.push(json!({"id":bond["model_id"],"begin":bond["begin"],"end":bond["end"],"order":bond["order"]}));
        }
        wanted
            .push(json!({"id":fragment["model_id"],"atoms":fragment_atoms,"bonds":fragment_bonds}));
    }
    assert_eq!(serde_json::to_value(graph.fragments())?, json!(wanted));
    Ok(())
}

#[test]
fn changing_any_projected_scalar_keeps_the_original_source_and_refuses_stale_publication() -> Result
{
    let raw = original(&[
        property(ISOTOPE, &[13, 0]),
        property(CHARGE, &[255]),
        property(RADICAL, &[2]),
        property(HYDROGENS, &[0, 0]),
    ]);
    let document = StoredDocument::from_cdx(raw.clone().into(), PageSize::A4)?;
    let page = document.model().pages()[0].id();
    let fragment = &document.model().pages()[0].fragments()[0];
    for change in 0..4 {
        let mut atoms = fragment.atoms().to_vec();
        let target = atoms.first_mut().ok_or("atom")?;
        match change {
            0 => target.isotope = std::num::NonZeroU16::new(14),
            1 => target.charge = 1,
            2 => target.radical = Radical::Triplet,
            _ => target.explicit_hydrogens = Some(1),
        }
        let mut candidate = document.clone();
        candidate.model_mut().replace_fragment(
            page,
            Fragment::new(fragment.id(), atoms, fragment.bonds().to_vec())?,
        )?;
        no_source_authority(&candidate);
        assert!(matches!(
            candidate.export_cdx(),
            Err(DocumentError::UnsupportedCdxEdit)
        ));
        assert!(matches!(
            candidate.to_cdx(),
            Err(DocumentError::UnsupportedCdxEdit)
        ));
        assert_eq!(document.to_cdx()?.as_ref(), raw);
        *candidate.model_mut() = document.model().clone();
        assert!(matches!(candidate.to_cdx()?, Cow::Borrowed(_)));
        assert_eq!(candidate.to_cdx()?.as_ref(), raw);
    }
    Ok(())
}

#[test]
fn all_frozen_original_binary_sources_match_independent_records_and_literal_model_facts() -> Result
{
    let vectors: Value = serde_json::from_str(include_str!(
        "fixtures/cdx-literal-metadata/literal-vectors.json"
    ))?;
    let cases = vectors["cases"].as_array().ok_or("cases")?;
    assert_eq!(cases.len(), 18);
    for case in cases {
        let name = case["name"].as_str().ok_or("name")?;
        let raw = oracle::source(name);
        let expected = oracle::facts(name);
        assert_eq!(
            oracle::read(raw)?,
            expected,
            "independent actual source: {name}"
        );
        let graph = ElementalGraph::parse(raw)?;
        compare_model(&graph, &expected)?;
        assert_eq!(graph.as_bytes(), raw);
        assert_eq!(graph.to_bytes(), raw);
        let document = StoredDocument::from_cdx(raw.into(), PageSize::A4)?;
        no_source_authority(&document);
        assert!(matches!(document.to_cdx()?, Cow::Borrowed(_)));
        assert_eq!(document.to_cdx()?.as_ref(), raw);
        let mut detached = document.clone();
        *detached.model_mut() = document.model().clone();
        assert_eq!(detached, document);
        assert_eq!(detached.to_cdx()?.as_ref(), raw);
        no_source_authority(&detached);
    }
    Ok(())
}
