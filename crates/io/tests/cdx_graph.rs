use bondline_io::cdx::{ElementalGraph, MappingError};
use bondline_model::{BondOrder, Point2};
use std::{
    io::Write,
    process::{Command, Stdio},
};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const ORIGINAL: &[u8] = include_bytes!("../../cdx/tests/fixtures/original-ethane-explicit.cdx");

#[test]
fn independent_original_scene_maps_dense_identities_and_exact_fixed_point_geometry()
-> Result<(), Box<dyn std::error::Error>> {
    let bytes = ORIGINAL;
    assert_eq!(bytes.len(), 65_750);
    let graph = ElementalGraph::parse(bytes)?;
    assert_eq!(graph.as_bytes(), bytes);
    assert_eq!(graph.to_bytes(), bytes);
    assert!(std::ptr::eq(graph.as_bytes().as_ptr(), bytes.as_ptr()));
    assert_eq!(graph.source().records().len(), 29);
    assert_eq!(graph.fragments().len(), 1);
    let fragment = graph.fragments().first().ok_or("missing fragment")?;
    assert_eq!(fragment.id().get(), 1);
    assert_eq!(fragment.atoms().len(), 2);
    let first = fragment.atoms().first().ok_or("missing first carbon")?;
    let second = fragment.atoms().get(1).ok_or("missing second carbon")?;
    assert_eq!((first.id.get(), first.element.get()), (1, 6));
    assert_eq!((second.id.get(), second.element.get()), (2, 6));
    assert_eq!(first.position, Some(Point2::new(10.25, -3.5)?));
    assert_eq!(second.position, Some(Point2::new(26.25, 8.5)?));
    assert_eq!(first.explicit_hydrogens, None);
    assert_eq!(second.explicit_hydrogens, None);
    assert_eq!(fragment.bonds().len(), 1);
    let bond = fragment.bonds().first().ok_or("missing bond")?;
    assert_eq!(
        (bond.id.get(), bond.begin.get(), bond.end.get(), bond.order),
        (1, 1, 2, BondOrder::Single)
    );
    Ok(())
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

fn property(tag: u16, payload: &[u8]) -> Vec<u8> {
    assert!(payload.len() < usize::from(u16::MAX));
    let mut bytes = tag.to_le_bytes().to_vec();
    bytes.extend_from_slice(&(payload.len() as u16).to_le_bytes());
    bytes.extend_from_slice(payload);
    bytes
}

fn root(children: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = b"VjCD0100\x04\x03\x02\x01".to_vec();
    bytes.extend_from_slice(&[0; 16]);
    bytes.extend_from_slice(&object(0x8000, 0, children));
    bytes
}

fn document(children: &[Vec<u8>]) -> Vec<u8> {
    root(&[object(0x8001, 42, children)])
}

fn bond(id: u32, begin: u32, end: u32, extra: &[Vec<u8>]) -> Vec<u8> {
    let mut fields = vec![
        property(0x0604, &begin.to_le_bytes()),
        property(0x0605, &end.to_le_bytes()),
    ];
    fields.extend_from_slice(extra);
    object(0x8005, id, &fields)
}

fn bare_fragment() -> Vec<u8> {
    object(
        0x8003,
        100,
        &[
            object(0x8004, 101, &[]),
            object(0x8004, 102, &[]),
            bond(103, 101, 102, &[]),
        ],
    )
}

#[test]
fn chemical_properties_cannot_be_ignored_on_wrong_container_types() {
    for field in [
        property(0x0400, &1_i16.to_le_bytes()),
        property(0x0402, &8_i16.to_le_bytes()),
        property(0x0200, &[0; 8]),
        property(0x0600, &1_u16.to_le_bytes()),
        property(0x0604, &101_u32.to_le_bytes()),
        property(0x0605, &102_u32.to_le_bytes()),
    ] {
        let root_field = root(&[field.clone(), object(0x8001, 42, &[bare_fragment()])]);
        let page_field = document(&[field.clone(), bare_fragment()]);
        let group_field = document(&[object(0x8002, 99, &[field, bare_fragment()])]);
        for source in [root_field, page_field, group_field] {
            assert!(matches!(
                ElementalGraph::parse(&source),
                Err(MappingError::UnsupportedProperty { .. })
            ));
        }
    }
}

#[test]
fn absent_defaults_and_explicit_element_types_do_not_invent_hydrogens() -> TestResult {
    let source = document(&[object(
        0x8003,
        100,
        &[
            bond(103, 102, 101, &[]),
            object(0x8004, 101, &[]),
            object(0x8004, 102, &[property(0x0400, &1_i16.to_le_bytes())]),
        ],
    )]);
    let graph = ElementalGraph::parse(&source)?;
    let fragment = graph.fragments().first().ok_or("fragment")?;
    for atom in fragment.atoms() {
        assert_eq!(atom.element.get(), 6);
        assert_eq!(atom.position, None);
        assert_eq!(atom.explicit_hydrogens, None);
        assert_eq!(atom.isotope, None);
        assert_eq!(atom.charge, 0);
        assert_eq!(atom.radical, bondline_model::Radical::None);
    }
    let bond = fragment.bonds().first().ok_or("bond")?;
    assert_eq!(
        (bond.begin.get(), bond.end.get(), bond.order),
        (2, 1, BondOrder::Single)
    );
    for element in [1_i16, 8, 118] {
        let source = document(&[object(
            0x8003,
            100,
            &[object(
                0x8004,
                101,
                &[property(0x0402, &element.to_le_bytes())],
            )],
        )]);
        assert_eq!(
            ElementalGraph::parse(&source)?
                .fragments()
                .first()
                .ok_or("fragment")?
                .atoms()
                .first()
                .ok_or("atom")?
                .element
                .get(),
            element as u8
        );
    }
    Ok(())
}

#[test]
fn signed_fixed_point_extremes_and_smallest_quantum_are_exact() -> TestResult {
    for (raw_y, raw_x, x, y) in [
        (i32::MIN, i32::MAX, 32767.99998474121, -32768.0),
        (-1, 1, 1.0 / 65536.0, -1.0 / 65536.0),
    ] {
        let mut point = raw_y.to_le_bytes().to_vec();
        point.extend_from_slice(&raw_x.to_le_bytes());
        let source = document(&[object(
            0x8003,
            100,
            &[object(0x8004, 101, &[property(0x0200, &point)])],
        )]);
        let graph = ElementalGraph::parse(&source)?;
        assert_eq!(
            graph
                .fragments()
                .first()
                .ok_or("fragment")?
                .atoms()
                .first()
                .ok_or("atom")?
                .position,
            Some(Point2::new(x, y)?)
        );
        assert_eq!(graph.to_bytes(), source);
    }
    Ok(())
}

#[test]
fn source_identifiers_are_scoped_and_application_ids_are_dense() -> TestResult {
    let source = document(&[
        bare_fragment(),
        object(0x8002, 99, &[object(0x8002, 98, &[bare_fragment()])]),
    ]);
    let graph = ElementalGraph::parse(&source)?;
    assert_eq!(
        graph
            .fragments()
            .iter()
            .map(|f| f.id().get())
            .collect::<Vec<_>>(),
        [1, 2]
    );
    let second = graph.fragments().get(1).ok_or("second fragment")?;
    assert_eq!(
        second
            .atoms()
            .iter()
            .map(|a| a.id.get())
            .collect::<Vec<_>>(),
        [3, 4]
    );
    let second_bond = second.bonds().first().ok_or("second bond")?;
    assert_eq!(
        (
            second_bond.id.get(),
            second_bond.begin.get(),
            second_bond.end.get()
        ),
        (2, 3, 4)
    );
    assert_eq!(
        ElementalGraph::parse(&source)?.fragments(),
        graph.fragments()
    );
    assert_eq!(graph.source().as_bytes(), source);

    let source = document(&[object(
        0x8003,
        0,
        &[
            object(0x8004, 0, &[]),
            object(0x8004, 0, &[]),
            object(0x8004, u32::MAX, &[]),
            object(0x8004, 2, &[]),
            bond(0, u32::MAX, 2, &[]),
        ],
    )]);
    let graph = ElementalGraph::parse(&source)?;
    let fragment = graph.fragments().first().ok_or("fragment")?;
    assert_eq!(
        fragment
            .atoms()
            .iter()
            .map(|a| a.id.get())
            .collect::<Vec<_>>(),
        [1, 2, 3, 4]
    );
    let bond = fragment.bonds().first().ok_or("bond")?;
    assert_eq!((bond.id.get(), bond.begin.get(), bond.end.get()), (1, 3, 4));
    Ok(())
}

#[test]
fn ambiguous_missing_and_cross_fragment_endpoints_are_refused() {
    let duplicate = document(&[object(
        0x8003,
        100,
        &[object(0x8004, 101, &[]), object(0x8004, 101, &[])],
    )]);
    assert!(matches!(
        ElementalGraph::parse(&duplicate),
        Err(MappingError::DuplicateSourceId { id: 101, .. })
    ));
    let custom_collision = document(&[object(
        0x8003,
        100,
        &[object(0x8004, 101, &[]), object(0xC123, 101, &[])],
    )]);
    assert!(matches!(
        ElementalGraph::parse(&custom_collision),
        Err(MappingError::DuplicateSourceId { id: 101, .. })
    ));
    for endpoint in [0, 999] {
        let source = document(&[object(
            0x8003,
            100,
            &[object(0x8004, 101, &[]), bond(103, 101, endpoint, &[])],
        )]);
        match endpoint {
            0 => assert!(matches!(
                ElementalGraph::parse(&source),
                Err(MappingError::UnassignedReference { .. })
            )),
            _ => assert!(matches!(
                ElementalGraph::parse(&source),
                Err(MappingError::UnresolvedReference { id: 999, .. })
            )),
        }
    }
    let missing = document(&[object(
        0x8003,
        100,
        &[
            object(0x8004, 101, &[]),
            object(0x8005, 103, &[property(0x0604, &101_u32.to_le_bytes())]),
        ],
    )]);
    assert!(matches!(
        ElementalGraph::parse(&missing),
        Err(MappingError::MissingProperty { tag: 0x0605, .. })
    ));
    let cross = document(&[
        bare_fragment(),
        object(
            0x8003,
            200,
            &[object(0x8004, 201, &[]), bond(203, 101, 201, &[])],
        ),
    ]);
    assert!(matches!(
        ElementalGraph::parse(&cross),
        Err(MappingError::UnresolvedReference { id: 101, .. })
    ));
}

#[test]
fn unsupported_labels_queries_and_chemical_metadata_are_explicit_errors() {
    for node_type in [0_i16, 2, 4, 13, -1] {
        let source = document(&[object(
            0x8003,
            100,
            &[object(
                0x8004,
                101,
                &[property(0x0400, &node_type.to_le_bytes())],
            )],
        )]);
        assert!(matches!(
            ElementalGraph::parse(&source),
            Err(MappingError::UnsupportedNodeType { .. })
        ));
    }
    for field in [
        property(0x0420, &13_i16.to_le_bytes()),
        property(0x0421, &[0xff]),
        property(0x0422, &[2]),
        property(0x042B, &0_u16.to_le_bytes()),
        property(0x000A, &1_i16.to_le_bytes()),
    ] {
        let source = document(&[object(0x8003, 100, &[object(0x8004, 101, &[field])])]);
        assert!(matches!(
            ElementalGraph::parse(&source),
            Err(MappingError::UnsupportedProperty { .. })
        ));
    }
    let labeled = document(&[object(
        0x8003,
        100,
        &[object(0x8004, 101, &[object(0x8006, 999, &[])])],
    )]);
    assert!(matches!(
        ElementalGraph::parse(&labeled),
        Err(MappingError::UnsupportedObject { tag: 0x8006, .. })
    ));
    let fragment_metadata = document(&[object(
        0x8003,
        100,
        &[property(0x0500, &[1]), object(0x8004, 101, &[])],
    )]);
    assert!(matches!(
        ElementalGraph::parse(&fragment_metadata),
        Err(MappingError::UnsupportedProperty { tag: 0x0500, .. })
    ));
    for order in [0_u16, 2, 3, 0xffff] {
        let source = document(&[object(
            0x8003,
            100,
            &[
                object(0x8004, 101, &[]),
                object(0x8004, 102, &[]),
                bond(103, 101, 102, &[property(0x0600, &order.to_le_bytes())]),
            ],
        )]);
        assert!(matches!(
            ElementalGraph::parse(&source),
            Err(MappingError::UnsupportedBondOrder { .. })
        ));
    }
}

#[test]
fn mapped_fields_validate_exact_widths_values_and_duplicate_properties() -> TestResult {
    for element in [-1_i16, 0, 119] {
        let source = document(&[object(
            0x8003,
            100,
            &[object(
                0x8004,
                101,
                &[property(0x0402, &element.to_le_bytes())],
            )],
        )]);
        assert!(matches!(
            ElementalGraph::parse(&source),
            Err(MappingError::InvalidProperty {
                tag: 0x0402,
                record: 4
            })
        ));
    }
    for field in [
        property(0x0400, &[1]),
        property(0x0402, &[6]),
        property(0x0200, &[0; 7]),
        property(0x0200, &[0; 9]),
    ] {
        let source = document(&[object(0x8003, 100, &[object(0x8004, 101, &[field])])]);
        assert!(matches!(
            ElementalGraph::parse(&source),
            Err(MappingError::InvalidProperty { record: 4, .. })
        ));
    }
    for tag in [0x0200, 0x0400, 0x0402] {
        let field = property(
            tag,
            match tag {
                0x0200 => &[0; 8],
                _ => &[1, 0],
            },
        );
        let source = document(&[object(
            0x8003,
            100,
            &[object(0x8004, 101, &[field.clone(), field])],
        )]);
        assert!(matches!(
            ElementalGraph::parse(&source),
            Err(MappingError::DuplicateProperty { .. })
        ));
    }
    for (tag, field) in [
        (0x0604, property(0x0604, &[1, 0])),
        (0x0605, property(0x0605, &[2, 0])),
        (0x0600, property(0x0600, &[1])),
    ] {
        let begin = property(0x0604, &101_u32.to_le_bytes());
        let end = property(0x0605, &102_u32.to_le_bytes());
        let fields = match tag {
            0x0604 => vec![field, end],
            0x0605 => vec![begin, field],
            _ => vec![begin, end, field],
        };
        let source = document(&[object(
            0x8003,
            100,
            &[
                object(0x8004, 101, &[]),
                object(0x8004, 102, &[]),
                object(0x8005, 103, &fields),
            ],
        )]);
        assert!(matches!(
            ElementalGraph::parse(&source),
            Err(MappingError::InvalidProperty { tag: actual, .. }) if actual == tag
        ));
    }
    for duplicate in [
        property(0x0604, &101_u32.to_le_bytes()),
        property(0x0605, &102_u32.to_le_bytes()),
        property(0x0600, &1_u16.to_le_bytes()),
    ] {
        let source = document(&[object(
            0x8003,
            100,
            &[
                object(0x8004, 101, &[]),
                object(0x8004, 102, &[]),
                bond(
                    103,
                    101,
                    102,
                    &[property(0x0600, &1_u16.to_le_bytes()), duplicate],
                ),
            ],
        )]);
        assert!(matches!(
            ElementalGraph::parse(&source),
            Err(MappingError::DuplicateProperty { .. })
        ));
    }
    let source = document(&[object(
        0x8003,
        100,
        &[object(0x8004, 101, &[property(0x0402, &[0, 0])])],
    )]);
    let syntax = bondline_cdx::Document::parse(&source)?;
    assert_eq!(syntax.records().get(4).ok_or("record")?.range(), 52..58);
    Ok(())
}

#[test]
fn unsupported_containers_cannot_hide_chemical_objects_or_extra_pages() {
    for child in [
        object(0x8004, 101, &[]),
        object(0x8003, 100, &[]),
        object(0x8002, 99, &[]),
    ] {
        let source = root(&[child]);
        assert!(matches!(
            ElementalGraph::parse(&source),
            Err(MappingError::UnsupportedLayout { .. })
        ));
    }
    let hidden = document(&[object(0xC123, 99, &[bare_fragment()])]);
    assert!(matches!(
        ElementalGraph::parse(&hidden),
        Err(MappingError::UnsupportedLayout { .. })
    ));
    let pages = root(&[object(0x8001, 42, &[]), object(0x8001, 43, &[])]);
    assert!(matches!(
        ElementalGraph::parse(&pages),
        Err(MappingError::UnsupportedPageLayout { .. })
    ));
    let table = root(&[object(0x8016, 99, &[object(0x8001, 42, &[])])]);
    assert!(matches!(
        ElementalGraph::parse(&table),
        Err(MappingError::UnsupportedObject { tag: 0x8016, .. })
    ));
    let nested = root(&[object(0x8000, 99, &[])]);
    assert!(matches!(
        ElementalGraph::parse(&nested),
        Err(MappingError::UnsupportedLayout { .. })
    ));
}

#[test]
fn model_self_bond_and_parallel_connection_policies_are_retained() {
    let self_bond = document(&[object(
        0x8003,
        100,
        &[object(0x8004, 101, &[]), bond(103, 101, 101, &[])],
    )]);
    assert!(matches!(
        ElementalGraph::parse(&self_bond),
        Err(MappingError::Graph(bondline_model::GraphError::SelfBond(1)))
    ));
    let parallel = document(&[object(
        0x8003,
        100,
        &[
            object(0x8004, 101, &[]),
            object(0x8004, 102, &[]),
            bond(103, 101, 102, &[]),
            bond(104, 102, 101, &[]),
        ],
    )]);
    assert!(matches!(
        ElementalGraph::parse(&parallel),
        Err(MappingError::Graph(
            bondline_model::GraphError::DuplicateConnection
        ))
    ));
}

#[test]
fn independent_struct_and_rdkit_verify_original_and_absent_defaults() -> TestResult {
    let first = [(-229376_i32).to_le_bytes(), 671744_i32.to_le_bytes()].concat();
    let second = [557056_i32.to_le_bytes(), 1720320_i32.to_le_bytes()].concat();
    let defaults = document(&[object(
        0x8003,
        100,
        &[
            object(0x8004, 101, &[property(0x0200, &first)]),
            object(0x8004, 102, &[property(0x0200, &second)]),
            bond(103, 101, 102, &[]),
        ],
    )]);
    for input in [ORIGINAL, defaults.as_slice()] {
        let graph = ElementalGraph::parse(input)?;
        let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
            if cfg!(windows) {
                "python".into()
            } else {
                "python3".into()
            }
        });
        let mut child = Command::new(python)
            .arg("-B")
            .arg(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/cdx_projection_oracle.py"
            ))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        child.stdin.take().ok_or("oracle stdin")?.write_all(input)?;
        let output = child.wait_with_output()?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let expected: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        let fragment = graph.fragments().first().ok_or("fragment")?;
        let positions = fragment
            .atoms()
            .iter()
            .map(|a| a.position.map(|p| [p.x_pt(), p.y_pt()]))
            .collect::<Vec<_>>();
        assert_eq!(
            serde_json::to_value(positions)?,
            expected.get("positions_pt").ok_or("positions")?.clone()
        );
        assert!(
            fragment
                .atoms()
                .iter()
                .all(|a| a.element.get() == 6 && a.explicit_hydrogens.is_none())
        );
        assert_eq!(
            expected.get("external_smiles").and_then(|v| v.as_str()),
            Some("CC")
        );
        assert_eq!(graph.to_bytes(), input);
    }
    Ok(())
}

#[test]
fn borrowed_source_retains_opaque_headers_and_noncanonical_lengths() -> TestResult {
    let opaque = include_bytes!("../../cdx/tests/fixtures/unknown-implicit.cdx");
    let graph = ElementalGraph::parse(opaque)?;
    assert!(graph.fragments().is_empty());
    assert_eq!(graph.to_bytes(), opaque);
    assert_eq!(graph.source().header(), opaque.get(..28).ok_or("header")?);
    let extended_element = vec![0x02, 0x04, 0xff, 0xff, 2, 0, 0, 0, 6, 0];
    let source = document(&[object(
        0x8003,
        100,
        &[object(0x8004, 101, &[extended_element])],
    )]);
    let graph = ElementalGraph::parse(&source)?;
    assert_eq!(
        graph
            .fragments()
            .first()
            .ok_or("fragment")?
            .atoms()
            .first()
            .ok_or("atom")?
            .element
            .get(),
        6
    );
    assert_eq!(graph.to_bytes(), source);
    Ok(())
}

#[test]
fn malformed_syntax_is_propagated_and_debug_does_not_include_opaque_payloads() -> TestResult {
    assert!(matches!(
        ElementalGraph::parse(b"not a CDX"),
        Err(MappingError::Syntax(_))
    ));
    assert!(matches!(
        ElementalGraph::parse(ORIGINAL.get(..ORIGINAL.len() - 1).ok_or("prefix")?),
        Err(MappingError::Syntax(_))
    ));
    let graph = ElementalGraph::parse(ORIGINAL)?;
    let debug = format!("{graph:?}");
    assert!(!debug.contains("user object"));
    assert!(debug.len() < 2_000);
    Ok(())
}

#[test]
fn stable_error_codes_separate_unsupported_profiles_invalid_data_and_budgets() {
    use bondline_cdx::Error as Syntax;
    use bondline_model::GraphError;
    for error in [
        MappingError::Syntax(Syntax::InputTooLarge),
        MappingError::Syntax(Syntax::RecordLimit),
        MappingError::Syntax(Syntax::DepthLimit),
        MappingError::Syntax(Syntax::Allocation),
        MappingError::Graph(GraphError::Limit("atoms")),
        MappingError::IdentityLimit,
    ] {
        assert_eq!(error.code(), "resource_limit");
    }
    for error in [
        MappingError::UnsupportedObject {
            tag: 0x8006,
            record: 3,
        },
        MappingError::UnsupportedLayout { record: 3 },
        MappingError::UnsupportedPageLayout { record: 3 },
        MappingError::UnsupportedProperty {
            tag: 0x0421,
            record: 3,
        },
        MappingError::UnsupportedNodeType { record: 3 },
        MappingError::UnsupportedBondOrder { record: 3 },
    ] {
        assert_eq!(error.code(), "unsupported_content");
    }
    for error in [
        MappingError::Syntax(Syntax::InvalidHeader),
        MappingError::Syntax(Syntax::Truncated { offset: 28 }),
        MappingError::InvalidProperty {
            tag: 0x0402,
            record: 3,
        },
        MappingError::DuplicateProperty {
            tag: 0x0402,
            record: 3,
        },
        MappingError::MissingProperty {
            tag: 0x0604,
            record: 3,
        },
        MappingError::DuplicateSourceId { id: 101, record: 3 },
        MappingError::UnassignedReference { record: 3 },
        MappingError::UnresolvedReference { id: 101, record: 3 },
        MappingError::Graph(GraphError::SelfBond(1)),
    ] {
        assert_eq!(error.code(), "invalid_document");
    }
}
