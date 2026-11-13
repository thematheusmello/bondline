use bondline_io::cdxml::{ElementalGraph, MappingError};
use bondline_model::{BondOrder, Point2, Radical};

fn document(body: &str) -> String {
    format!("<CDXML><page><fragment>{body}</fragment></page></CDXML>")
}

#[test]
fn original_fixture_maps_chemistry_and_retains_all_unmodelled_source_bytes()
-> Result<(), Box<dyn std::error::Error>> {
    let source = include_bytes!("../../cdxml/tests/fixtures/unknown.cdxml");
    let graph = ElementalGraph::parse(source)?;
    assert_eq!(graph.as_bytes(), source);
    assert_eq!(graph.to_bytes(), source);
    let fragment = graph.fragments().first().ok_or("missing fragment")?;
    assert_eq!(fragment.id().get(), 1);
    assert_eq!(fragment.atoms().len(), 2);
    let carbon = fragment.atoms().first().ok_or("missing carbon")?;
    assert_eq!(carbon.id.get(), 1);
    assert_eq!(carbon.element.get(), 6);
    assert_eq!(carbon.isotope.map(|mass| mass.get()), Some(13));
    assert_eq!(carbon.position, Some(Point2::new(12.5, 25.0)?));
    assert_eq!(
        fragment
            .atoms()
            .get(1)
            .ok_or("missing oxygen")?
            .element
            .get(),
        8
    );
    let bond = fragment.bonds().first().ok_or("missing bond")?;
    assert_eq!(
        (bond.id.get(), bond.begin.get(), bond.end.get(), bond.order),
        (1, 1, 2, BondOrder::Double)
    );
    assert_eq!(
        ElementalGraph::parse(source)?.fragments(),
        graph.fragments()
    );
    Ok(())
}

#[test]
fn defaults_explicit_zeroes_and_authored_atom_properties_follow_the_public_spec()
-> Result<(), Box<dyn std::error::Error>> {
    let source = document(
        "<n id='4294967295.00' Element='6.00' p='72 144' Isotope='13.0' Charge='-1.00' Radical='Doublet' NumHydrogens='0'><t><s>untrusted label N</s></t></n><n Isotope='0'/>",
    );
    let graph = ElementalGraph::parse(source.as_bytes())?;
    let atoms = graph.fragments().first().ok_or("missing fragment")?.atoms();
    let first = atoms.first().ok_or("missing first atom")?;
    assert_eq!(first.id.get(), 1);
    assert_eq!(first.element.get(), 6);
    assert_eq!(first.charge, -1);
    assert_eq!(first.radical, Radical::Doublet);
    assert_eq!(first.explicit_hydrogens, Some(0));
    assert_eq!(first.isotope.map(|mass| mass.get()), Some(13));
    assert_eq!(first.position, Some(Point2::new(72.0, 144.0)?));
    let second = atoms.get(1).ok_or("missing second atom")?;
    assert_eq!(second.element.get(), 6);
    assert_eq!(second.charge, 0);
    assert_eq!(second.radical, Radical::None);
    assert_eq!(second.position, None);
    assert_eq!(second.isotope, None);
    assert_eq!(second.explicit_hydrogens, None);
    assert_eq!(graph.as_bytes(), source.as_bytes());
    Ok(())
}

#[test]
fn fragment_scopes_and_unassigned_source_ids_receive_distinct_application_ids()
-> Result<(), Box<dyn std::error::Error>> {
    let source=b"<CDXML><fonttable><font id='21' name='metadata only'/></fonttable><page><fragment id='1'><n id='0021.00'/><n id='22'/><b id='23' B='21' E='22'/></fragment><group><group><fragment id='1'><n id='21'/><n id='22'/><b id='23' B='21' E='22'/><n id='0'/><n id='0'/><n/></fragment></group></group></page></CDXML>";
    let graph = ElementalGraph::parse(source)?;
    assert_eq!(graph.fragments().len(), 2);
    let second = graph.fragments().get(1).ok_or("missing second fragment")?;
    assert_eq!(second.id().get(), 2);
    assert_eq!(
        second
            .atoms()
            .iter()
            .map(|atom| atom.id.get())
            .collect::<Vec<_>>(),
        [3, 4, 5, 6, 7]
    );
    let bond = second.bonds().first().ok_or("missing second bond")?;
    assert_eq!((bond.id.get(), bond.begin.get(), bond.end.get()), (2, 3, 4));
    assert_eq!(graph.as_bytes(), source);
    Ok(())
}

#[test]
fn all_published_single_term_bond_orders_and_direction_map_without_float_rounding()
-> Result<(), Box<dyn std::error::Error>> {
    for (value, expected) in [
        ("1.00", BondOrder::Single),
        ("2", BondOrder::Double),
        ("3", BondOrder::Triple),
        ("4", BondOrder::Quadruple),
        ("5", BondOrder::Quintuple),
        ("6", BondOrder::Sextuple),
        ("0.50", BondOrder::Half),
        ("1.500", BondOrder::Aromatic),
        ("2.5", BondOrder::TwoAndHalf),
        ("3.5", BondOrder::ThreeAndHalf),
        ("4.5", BondOrder::FourAndHalf),
        ("5.5", BondOrder::FiveAndHalf),
        ("dative", BondOrder::Dative),
        ("ionic", BondOrder::Ionic),
        ("hydrogen", BondOrder::Hydrogen),
        ("threecenter", BondOrder::ThreeCenter),
    ] {
        let source = document(&format!(
            "<b B='22' E='21' Order='{value}'/><n id='21' Element='8'/><n id='22' Element='7'/>"
        ));
        let graph = ElementalGraph::parse(source.as_bytes())?;
        let bond = graph
            .fragments()
            .first()
            .and_then(|fragment| fragment.bonds().first())
            .ok_or("missing bond")?;
        assert_eq!(bond.order, expected);
        assert_eq!((bond.begin.get(), bond.end.get()), (2, 1));
    }
    for value in [
        "0",
        "7",
        "1 2",
        "any",
        "Dative",
        "1.50000000000000000001",
        "NaN",
        "1e0",
    ] {
        let source = document(&format!(
            "<n id='1'/><n id='2'/><b B='1' E='2' Order='{value}'/>"
        ));
        assert!(matches!(
            ElementalGraph::parse(source.as_bytes()),
            Err(MappingError::UnsupportedBondOrder)
        ));
    }
    Ok(())
}

#[test]
fn invalid_properties_and_ambiguous_source_ids_fail_instead_of_changing_meaning()
-> Result<(), Box<dyn std::error::Error>> {
    for property in [
        "Element='0'",
        "Element='119'",
        "Element='6.00000000000000001'",
        "Charge='128'",
        "Charge='-129'",
        "Charge='0.00000000000001'",
        "Isotope='-1'",
        "Isotope='32768'",
        "NumHydrogens='65536'",
        "NumHydrogens='-1'",
        "Radical='unknown'",
        "p='NaN 0'",
        "id='4294967296'",
    ] {
        let source = document(&format!("<n {property}/>"));
        assert!(
            ElementalGraph::parse(source.as_bytes()).is_err(),
            "accepted {property}"
        );
    }
    for body in [
        "<n id='1'/><n id='1.0'/>",
        "<n id='1'/><b id='1' B='1' E='2'/>",
        "<n id='1'/><graphic id='1'/>",
    ] {
        assert!(matches!(
            ElementalGraph::parse(document(body).as_bytes()),
            Err(MappingError::DuplicateSourceId)
        ));
    }
    Ok(())
}

#[test]
fn unresolved_or_zero_references_and_non_element_nodes_are_explicitly_unsupported()
-> Result<(), Box<dyn std::error::Error>> {
    for (body, zero) in [
        ("<n id='1'/><b B='1' E='2'/>", false),
        ("<n id='0'/><b B='0' E='0'/>", true),
    ] {
        let source = document(body);
        let error = ElementalGraph::parse(source.as_bytes())
            .err()
            .ok_or("accepted invalid reference")?;
        if zero {
            assert!(matches!(error, MappingError::UnassignedReference));
        } else {
            assert!(matches!(error, MappingError::UnresolvedReference));
        }
    }
    for node_type in ["Unspecified", "ElementList", "Nickname", "future"] {
        let source = document(&format!("<n NodeType='{node_type}'/>"));
        assert!(matches!(
            ElementalGraph::parse(source.as_bytes()),
            Err(MappingError::UnsupportedNodeType)
        ));
    }
    let outside = b"<CDXML><page><n id='1'/></page></CDXML>";
    assert!(matches!(
        ElementalGraph::parse(outside),
        Err(MappingError::UnsupportedLayout)
    ));
    let cross=b"<CDXML><page><fragment><n id='1'/><b B='1' E='2'/></fragment><fragment><n id='2'/></fragment></page></CDXML>";
    assert!(matches!(
        ElementalGraph::parse(cross),
        Err(MappingError::UnresolvedReference)
    ));
    let extension =
        b"<CDXML xmlns:f='urn:test'><page><f:n id='not a number' p='opaque'/></page></CDXML>";
    assert!(ElementalGraph::parse(extension)?.fragments().is_empty());
    Ok(())
}
