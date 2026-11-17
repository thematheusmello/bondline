#![no_main]

use libfuzzer_sys::fuzz_target;

fn integer_property(value: &str) {
    let id = value.parse::<bondline_cdxml::ObjectId>();
    if let Ok(integer) = bondline_cdxml::parse_integer(value) {
        assert_eq!(
            bondline_cdxml::parse_integer(&integer.to_string()).unwrap(),
            integer
        );
        assert_eq!(id.ok().map(|id| id.get()), u32::try_from(integer).ok());
    } else {
        assert!(id.is_err());
    }
}

fn writer_property(value: &str) {
    let mut writer = bondline_cdxml::Writer::new(&[]).unwrap();
    match writer.empty_element("page", &[("Comment", value)]) {
        Ok(()) => {
            let bytes = writer.finish().unwrap();
            let parsed = bondline_cdxml::Document::parse(&bytes).unwrap();
            let page = parsed.xml().root_element().first_element_child().unwrap();
            assert_eq!(page.attribute("Comment"), Some(value));
        }
        Err(error) => {
            assert!(matches!(
                error,
                bondline_cdxml::Error::InvalidAttributeValue | bondline_cdxml::Error::InputTooLarge
            ));
            let bytes = writer.finish().unwrap();
            let parsed = bondline_cdxml::Document::parse(&bytes).unwrap();
            assert!(parsed.xml().root_element().first_element_child().is_none());
        }
    }
}

fn resource_value_property(value: &str) {
    use bondline_cdxml::{Document, Edit, ResourceError, Writer};
    let mut writer = Writer::new(&[]).unwrap();
    writer.start_element("fonttable", &[]).unwrap();
    if let Err(error) = writer.empty_element("font", &[("id", "0"), ("name", value)]) {
        assert!(matches!(
            error,
            bondline_cdxml::Error::InvalidAttributeValue | bondline_cdxml::Error::InputTooLarge
        ));
        return;
    }
    writer.end_element().unwrap();
    writer.start_element("colortable", &[]).unwrap();
    writer
        .empty_element("color", &[("r", ".125"), ("g", ".5"), ("b", ".875")])
        .unwrap();
    writer.end_element().unwrap();
    let bytes = writer.finish().unwrap();
    let document = Document::parse(&bytes).unwrap();
    let resources = document.resources().unwrap();
    assert_eq!(resources.font(0).unwrap().name(), value);
    assert_eq!(resources.color(0).unwrap(), [0.0; 3]);
    assert_eq!(resources.color(1).unwrap(), [1.0; 3]);
    assert_eq!(resources.color(2).unwrap(), [0.125, 0.5, 0.875]);
    assert_eq!(resources.color(3), Err(ResourceError::MissingColor(3)));
    let font_bytes = document
        .apply_edits(&[Edit::SetAttribute {
            element: resources.font(0).unwrap().element(),
            name: "id",
            value,
        }])
        .unwrap();
    let font_document = Document::parse(&font_bytes).unwrap();
    match bondline_cdxml::parse_integer(value)
        .ok()
        .and_then(|id| u16::try_from(id).ok())
    {
        Some(id) => assert_eq!(
            font_document.resources().unwrap().font(id).unwrap().name(),
            value
        ),
        None => assert_eq!(
            font_document.resources().err(),
            Some(ResourceError::InvalidFontId)
        ),
    }
    let color_bytes = document
        .apply_edits(&[Edit::SetAttribute {
            element: resources.colors().first().unwrap().element(),
            name: "r",
            value,
        }])
        .unwrap();
    let color_document = Document::parse(&color_bytes).unwrap();
    match value
        .trim_matches([' ', '\t', '\r', '\n'])
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite() && (0.0..=1.0).contains(v))
    {
        Some(r) => assert_eq!(
            color_document.resources().unwrap().color(2).unwrap(),
            [r, 0.5, 0.875]
        ),
        None => assert_eq!(
            color_document.resources().err(),
            Some(ResourceError::InvalidColorComponent("r"))
        ),
    }
    assert_eq!(document.as_bytes(), bytes);
}

fuzz_target!(|data: &[u8]| {
    if let Ok(value) = std::str::from_utf8(data) {
        integer_property(value);
        writer_property(value);
        resource_value_property(value);
    }
    if let Ok(document) = bondline_cdxml::Document::parse(data) {
        assert_eq!(document.as_bytes(), data);
        assert_eq!(document.to_bytes(), data);
        assert_eq!(document.xml().root_element().tag_name().name(), "CDXML");
        if let Ok(resources) = document.resources() {
            assert_eq!(resources.color(0).unwrap(), [0.0; 3]);
            assert_eq!(resources.color(1).unwrap(), [1.0; 3]);
            if resources.color_table().is_none() {
                assert_eq!(resources.color(2).unwrap(), [1.0; 3]);
                assert_eq!(resources.color(3).unwrap(), [0.0; 3]);
            }
            for font in resources.fonts() {
                assert_eq!(resources.font(font.id()).unwrap().element(), font.element());
            }
            for (ordinal, color) in resources.colors().iter().enumerate() {
                assert_eq!(usize::from(color.index()), ordinal + 2);
                assert_eq!(resources.color(color.index()).unwrap(), color.rgb());
                assert!(
                    color
                        .rgb()
                        .into_iter()
                        .all(|v| v.is_finite() && (0.0..=1.0).contains(&v))
                );
            }
        }
        for attribute in document
            .xml()
            .descendants()
            .flat_map(|node| node.attributes())
        {
            integer_property(attribute.value());
        }
        for element in document
            .xml()
            .descendants()
            .filter(|node| node.is_element())
        {
            if let Some(attribute) = element.attributes().find(|attr| attr.namespace().is_none()) {
                let value = "μ & < > \" ' \t\r\n";
                let output = match document.replace_attribute(element, attribute.name(), value) {
                    Ok(output) => output,
                    Err(error) => {
                        assert!(matches!(error, bondline_cdxml::Error::InputTooLarge));
                        break;
                    }
                };
                let edited = bondline_cdxml::Document::parse(&output).unwrap();
                let target = edited.xml().get_node(element.id()).unwrap();
                assert_eq!(
                    target
                        .attributes()
                        .find(|attr| attr.namespace().is_none() && attr.name() == attribute.name())
                        .map(|attr| attr.value()),
                    Some(value)
                );
                for original in element
                    .attributes()
                    .filter(|attr| attr.namespace().is_some())
                {
                    assert_eq!(
                        target
                            .attributes()
                            .find(|attr| attr.namespace() == original.namespace()
                                && attr.name() == original.name())
                            .map(|attr| attr.value()),
                        Some(original.value())
                    );
                }
                assert_eq!(document.as_bytes(), data);
                break;
            }
        }
    }
    if let Ok(geometry) = bondline_io::cdxml::Geometry::parse(data) {
        assert_eq!(geometry.as_bytes(), data);
        if let Some(node) = geometry
            .nodes()
            .iter()
            .find(|node| node.position().is_some())
        {
            let target = bondline_model::Point2::new(72.0, 144.0).unwrap();
            match geometry.with_node_position(node.id(), target) {
                Ok(output) => {
                    let edited = bondline_io::cdxml::Geometry::parse(&output).unwrap();
                    let moved = edited
                        .nodes()
                        .iter()
                        .find(|edited| edited.id() == node.id())
                        .unwrap();
                    assert_eq!(moved.position(), Some(target));
                    assert_eq!(geometry.as_bytes(), data);
                }
                Err(error) => assert!(matches!(
                    error,
                    bondline_io::cdxml::Error::Syntax(bondline_cdxml::Error::InputTooLarge)
                )),
            }
        }
    }
    if let Ok(graph) = bondline_io::cdxml::ElementalGraph::parse(data) {
        assert_eq!(graph.as_bytes(), data);
        assert_eq!(graph.to_bytes(), data);
        let mut fragments = std::collections::BTreeSet::new();
        let mut atoms = std::collections::BTreeSet::new();
        let mut bonds = std::collections::BTreeSet::new();
        for fragment in graph.fragments() {
            assert!(fragments.insert(fragment.id()));
            let local: std::collections::BTreeSet<_> =
                fragment.atoms().iter().map(|atom| atom.id).collect();
            for atom in fragment.atoms() {
                assert!(atoms.insert(atom.id));
            }
            for bond in fragment.bonds() {
                assert!(bonds.insert(bond.id));
                assert!(local.contains(&bond.begin) && local.contains(&bond.end));
                assert_ne!(bond.begin, bond.end);
            }
        }
        match bondline_io::StoredDocument::from_cdxml(
            std::sync::Arc::from(data),
            bondline_model::PageSize::A4,
        ) {
            Ok(stored) => {
                assert_eq!(stored.export_cdxml().unwrap(), data);
                let mut edited = stored.clone();
                let page = edited.model().pages().first().unwrap().id();
                let target = if stored.model().pages().first().unwrap().size()
                    == bondline_model::PageSize::new(72.0, 144.0).unwrap()
                {
                    bondline_model::PageSize::new(144.0, 72.0).unwrap()
                } else {
                    bondline_model::PageSize::new(72.0, 144.0).unwrap()
                };
                edited.model_mut().set_page_size(page, target).unwrap();
                assert!(matches!(
                    edited.export_cdxml(),
                    Err(bondline_io::DocumentError::UnsupportedEdit)
                ));
                if stored.is_fully_mapped() {
                    let bytes = edited.to_cdxml().unwrap().into_owned();
                    let reopened = bondline_io::StoredDocument::from_cdxml(
                        bytes.into(),
                        bondline_model::PageSize::A4,
                    )
                    .unwrap();
                    assert!(reopened.is_fully_mapped());
                    assert_eq!(reopened.model(), edited.model());
                } else {
                    assert!(matches!(
                        edited.to_cdxml(),
                        Err(bondline_io::DocumentError::UnsupportedEdit)
                    ));
                }
                *edited.model_mut() = stored.model().clone();
                assert_eq!(edited.export_cdxml().unwrap(), data);
            }
            Err(error) => assert!(matches!(
                error,
                bondline_io::DocumentError::Mapping(
                    bondline_io::cdxml::MappingError::UnsupportedPageLayout
                        | bondline_io::cdxml::MappingError::InvalidProperty("Width" | "Height")
                        | bondline_io::cdxml::MappingError::PreviewModel(
                            bondline_model::Error::InvalidPageSize
                        )
                )
            )),
        }
    }
});
