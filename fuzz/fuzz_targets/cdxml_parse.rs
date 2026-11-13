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

fuzz_target!(|data: &[u8]| {
    if let Ok(value) = std::str::from_utf8(data) {
        integer_property(value);
    }
    if let Ok(document) = bondline_cdxml::Document::parse(data) {
        assert_eq!(document.as_bytes(), data);
        assert_eq!(document.to_bytes(), data);
        assert_eq!(document.xml().root_element().tag_name().name(), "CDXML");
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
                assert_eq!(target.attribute(attribute.name()), Some(value));
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
});
