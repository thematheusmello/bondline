#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(document) = bondline_cdxml::Document::parse(data) {
        assert_eq!(document.as_bytes(), data);
        assert_eq!(document.to_bytes(), data);
        assert_eq!(document.xml().root_element().tag_name().name(), "CDXML");
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
});
