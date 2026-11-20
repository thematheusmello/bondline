#![no_main]

use libfuzzer_sys::fuzz_target;

const INPUT_LIMIT: usize = 16 * 1024;

fn label_default_role(name: &str, namespace: Option<&str>, actual_root: bool) -> bool {
    namespace.is_none_or(str::is_empty)
        && (actual_root
            || matches!(
                name,
                "n" | "b" | "t" | "table" | "spectrum" | "constraint" | "tlcplate"
            ))
}

fn label_default_value_valid(name: &str, value: &str) -> bool {
    if name == "LabelSize" {
        value
            .trim_matches([' ', '\t', '\r', '\n'])
            .parse::<f64>()
            .ok()
            .is_some_and(|value| value.is_finite() && value > 0.0)
    } else {
        bondline_cdxml::parse_integer(value)
            .ok()
            .is_some_and(|value| i16::try_from(value).is_ok())
    }
}

fn label_default_views(document: &bondline_cdxml::Document<'_>, source: &[u8]) {
    use bondline_cdxml::{Document, LabelDefaultsError};
    let foreign = Document::parse(source).unwrap();
    assert_eq!(
        document
            .label_defaults(foreign.xml().root_element())
            .unwrap_err(),
        LabelDefaultsError::InvalidElement
    );
    let root = document.xml().root_element();
    for element in document.xml().descendants() {
        let admitted = element.is_element()
            && label_default_role(
                element.tag_name().name(),
                element.tag_name().namespace(),
                element == root,
            );
        let result = document.label_defaults(element);
        if !admitted {
            assert_eq!(result.unwrap_err(), LabelDefaultsError::InvalidElement);
            continue;
        }
        match result {
            Ok(defaults) => {
                for name in ["LabelFont", "LabelSize", "LabelFace"] {
                    for owner in element.ancestors().filter(|node| node.is_element()) {
                        if let Some(attribute) = owner
                            .attributes()
                            .find(|a| a.namespace().is_none() && a.name() == name)
                        {
                            assert!(label_default_role(
                                owner.tag_name().name(),
                                owner.tag_name().namespace(),
                                owner == root
                            ));
                            assert!(label_default_value_valid(name, attribute.value()));
                        }
                    }
                    let nearest = element
                        .ancestors()
                        .filter(|node| node.is_element())
                        .find_map(|owner| {
                            owner
                                .attributes()
                                .find(|a| a.namespace().is_none() && a.name() == name)
                                .map(|attribute| (owner, attribute.value()))
                        });
                    if name == "LabelSize" {
                        let expected = nearest.map(|(owner, raw)| {
                            (
                                owner,
                                raw,
                                raw.trim_matches([' ', '\t', '\r', '\n'])
                                    .parse::<f64>()
                                    .unwrap(),
                            )
                        });
                        assert_eq!(
                            defaults
                                .size_pt()
                                .map(|p| (p.owner(), p.attribute_value(), p.value())),
                            expected
                        );
                    } else {
                        let expected = nearest.map(|(owner, raw)| {
                            (
                                owner,
                                raw,
                                i16::try_from(bondline_cdxml::parse_integer(raw).unwrap()).unwrap(),
                            )
                        });
                        let actual = if name == "LabelFont" {
                            defaults.font()
                        } else {
                            defaults.face()
                        };
                        assert_eq!(
                            actual.map(|p| (p.owner(), p.attribute_value(), p.value())),
                            expected
                        );
                    }
                }
            }
            Err(LabelDefaultsError::InvalidElement) => {
                panic!("ordinary owned admitted target was refused")
            }
            Err(LabelDefaultsError::UnsupportedOwner(name)) => assert!(
                element.ancestors().filter(|node| node.is_element()).any(
                    |owner| !label_default_role(
                        owner.tag_name().name(),
                        owner.tag_name().namespace(),
                        owner == root
                    ) && owner
                        .attributes()
                        .any(|a| a.namespace().is_none() && a.name() == name)
                )
            ),
            Err(LabelDefaultsError::InvalidProperty(name)) => assert!(
                element.ancestors().filter(|node| node.is_element()).any(
                    |owner| label_default_role(
                        owner.tag_name().name(),
                        owner.tag_name().namespace(),
                        owner == root
                    ) && owner.attributes().any(|a| a.namespace().is_none()
                        && a.name() == name
                        && !label_default_value_valid(name, a.value()))
                )
            ),
        }
        for attribute in element
            .attributes()
            .filter(|attribute| attribute.namespace().is_some())
        {
            if let Some(namespace) = attribute.namespace() {
                assert_eq!(
                    element.attribute((namespace, attribute.name())),
                    Some(attribute.value())
                );
            }
        }
        assert_eq!(document.as_bytes(), source);
    }
    assert_eq!(document.as_bytes(), source);
}

fn label_default_value_property(value: &str) {
    use bondline_cdxml::{Document, Error, LabelDefaultsError, Writer};
    for name in ["LabelFont", "LabelSize", "LabelFace"] {
        let mut writer = Writer::new(&[]).unwrap();
        if let Err(error) = writer.empty_element("t", &[(name, value)]) {
            assert!(matches!(error, Error::InvalidAttributeValue));
            continue;
        }
        let bytes = writer.finish().unwrap();
        let document = Document::parse(&bytes).unwrap();
        let text = document.xml().root_element().first_element_child().unwrap();
        match document.label_defaults(text) {
            Ok(defaults) => {
                assert!(label_default_value_valid(name, value));
                assert_eq!(defaults.font().is_some(), name == "LabelFont");
                assert_eq!(defaults.size_pt().is_some(), name == "LabelSize");
                assert_eq!(defaults.face().is_some(), name == "LabelFace");
                if name == "LabelSize" {
                    let p = defaults.size_pt().unwrap();
                    assert_eq!(
                        p.value(),
                        value
                            .trim_matches([' ', '\t', '\r', '\n'])
                            .parse::<f64>()
                            .unwrap()
                    );
                    assert_eq!(p.owner(), text);
                    assert_eq!(p.attribute_value(), value);
                } else {
                    let p = if name == "LabelFont" {
                        defaults.font()
                    } else {
                        defaults.face()
                    }
                    .unwrap();
                    assert_eq!(
                        p.value(),
                        i16::try_from(bondline_cdxml::parse_integer(value).unwrap()).unwrap()
                    );
                    assert_eq!(p.owner(), text);
                    assert_eq!(p.attribute_value(), value);
                }
            }
            Err(error) => {
                assert!(!label_default_value_valid(name, value));
                assert_eq!(error, LabelDefaultsError::InvalidProperty(name));
            }
        }
        assert_eq!(document.as_bytes(), bytes);
    }
}

fn structured_label_defaults(data: &[u8]) {
    use bondline_cdxml::{Document, LabelDefaultsError};
    let font = i16::from_le_bytes([
        data.first().copied().unwrap_or(0),
        data.get(1).copied().unwrap_or(0),
    ]);
    let sizes = ["0.25", "14.5", "100001", "1e300"];
    let size = sizes[usize::from(data.get(2).copied().unwrap_or(0)) % sizes.len()];
    let source = format!(
        "<CDXML LabelFont='{font}' LabelSize='10' LabelFace='96'><page><fragment><n LabelSize='{size}'><t LabelFace='0'><s>OH</s></t></n></fragment></page></CDXML>"
    );
    let document = Document::parse(source.as_bytes()).unwrap();
    let root = document.xml().root_element();
    let node = document
        .xml()
        .descendants()
        .find(|n| n.is_element() && n.tag_name().name() == "n")
        .unwrap();
    let text = node.first_element_child().unwrap();
    let defaults = document.label_defaults(text).unwrap();
    let own_font = defaults.font().unwrap();
    assert_eq!(own_font.value(), font);
    assert_eq!(own_font.owner(), root);
    assert_eq!(own_font.attribute_value(), font.to_string());
    let own_size = defaults.size_pt().unwrap();
    assert_eq!(own_size.value(), size.parse::<f64>().unwrap());
    assert_eq!(own_size.owner(), node);
    assert_eq!(own_size.attribute_value(), size);
    let own_face = defaults.face().unwrap();
    assert_eq!(own_face.value(), 0);
    assert_eq!(own_face.owner(), text);
    let run = document.styled_text(text).unwrap().runs()[0];
    assert_eq!((run.font(), run.size_pt(), run.face()), (None, None, None));
    assert_eq!(document.as_bytes(), source.as_bytes());

    let absent = Document::parse(b"<CDXML><n><t><s/></t></n></CDXML>").unwrap();
    let text = absent
        .xml()
        .root_element()
        .first_element_child()
        .unwrap()
        .first_element_child()
        .unwrap();
    let defaults = absent.label_defaults(text).unwrap();
    assert_eq!(
        (defaults.font(), defaults.size_pt(), defaults.face()),
        (None, None, None)
    );

    for (name, valid, invalid) in [
        ("LabelFont", "-32768.00", "65535"),
        ("LabelSize", "0.25", "-1e-9999"),
        ("LabelFace", "32767", "32768"),
    ] {
        let bad_shadow =
            format!("<CDXML {name}='{invalid}'><n {name}='{valid}'><t><s/></t></n></CDXML>");
        let document = Document::parse(bad_shadow.as_bytes()).unwrap();
        let text = document
            .xml()
            .root_element()
            .first_element_child()
            .unwrap()
            .first_element_child()
            .unwrap();
        assert_eq!(
            document.label_defaults(text).unwrap_err(),
            LabelDefaultsError::InvalidProperty(name)
        );
        assert_eq!(document.as_bytes(), bad_shadow.as_bytes());
        let bad_owner = format!(
            "<CDXML {name}='{valid}'><group {name}='{valid}'><n {name}='{valid}'><t><s/></t></n></group></CDXML>"
        );
        let document = Document::parse(bad_owner.as_bytes()).unwrap();
        let text = document
            .xml()
            .descendants()
            .find(|node| node.is_element() && node.tag_name().name() == "t")
            .unwrap();
        assert_eq!(
            document.label_defaults(text).unwrap_err(),
            LabelDefaultsError::UnsupportedOwner(name)
        );
        assert_eq!(document.as_bytes(), bad_owner.as_bytes());
    }
    for (source, error) in [
        (
            "<CDXML LabelFont='bad'><n><t LabelSize='bad'><s/></t></n></CDXML>",
            LabelDefaultsError::InvalidProperty("LabelSize"),
        ),
        (
            "<CDXML><n><t LabelFont='bad' LabelSize='0' LabelFace='bad'><s/></t></n></CDXML>",
            LabelDefaultsError::InvalidProperty("LabelFont"),
        ),
        (
            "<CDXML LabelFont='bad'><group LabelSize='bad'><n><t><s/></t></n></group></CDXML>",
            LabelDefaultsError::UnsupportedOwner("LabelSize"),
        ),
        (
            "<CDXML xmlns:x='urn:label'><x:n LabelSize='12'><t><s/></t></x:n></CDXML>",
            LabelDefaultsError::UnsupportedOwner("LabelSize"),
        ),
    ] {
        let document = Document::parse(source.as_bytes()).unwrap();
        let text = document
            .xml()
            .descendants()
            .find(|node| node.is_element() && node.tag_name().name() == "t")
            .unwrap();
        assert_eq!(document.label_defaults(text).unwrap_err(), error);
        assert_eq!(document.as_bytes(), source.as_bytes());
    }
    const QUALIFIED: &[u8] = b"<CDXML xmlns:x='urn:label' LabelFont='-1' x:LabelFont='bad'><x:n xmlns='urn:other'><t xmlns='' LabelSize='2.5' x:LabelSize='bad'><s/></t></x:n></CDXML>";
    let document = Document::parse(QUALIFIED).unwrap();
    let text = document
        .xml()
        .descendants()
        .find(|node| {
            node.is_element()
                && node.tag_name().namespace().is_none_or(str::is_empty)
                && node.tag_name().name() == "t"
        })
        .unwrap();
    let defaults = document.label_defaults(text).unwrap();
    assert_eq!(defaults.font().unwrap().value(), -1);
    assert_eq!(
        defaults.font().unwrap().owner(),
        document.xml().root_element()
    );
    assert_eq!(defaults.size_pt().unwrap().value(), 2.5);
    assert_eq!(text.attribute(("urn:label", "LabelSize")), Some("bad"));
    assert_eq!(
        document
            .xml()
            .root_element()
            .attribute(("urn:label", "LabelFont")),
        Some("bad")
    );
    assert_eq!(document.as_bytes(), QUALIFIED);
}

fn writer_text_property(value: &str) {
    use bondline_cdxml::{Document, Error, Writer};
    let mut writer = Writer::new(&[]).unwrap();
    writer.start_element("t", &[]).unwrap();
    writer
        .start_element(
            "s",
            &[
                ("font", "65535"),
                ("size", "1.25"),
                ("face", "65535"),
                ("color", "65535"),
            ],
        )
        .unwrap();
    writer.text("prefix:").unwrap();
    writer.text("").unwrap();
    let expected = match writer.text(value) {
        Ok(()) => format!("prefix:{value}:suffix"),
        Err(error) => {
            // At most16KiB data cannot exhaust this fixed writer's size/node budgets.
            assert!(matches!(error, Error::InvalidTextValue));
            "prefix::suffix".to_owned()
        }
    };
    writer.text(":suffix").unwrap();
    writer.end_element().unwrap();
    writer.end_element().unwrap();
    let bytes = writer.finish().unwrap();
    let document = Document::parse(&bytes).unwrap();
    let text = document.xml().root_element().first_element_child().unwrap();
    let view = document.styled_text(text).unwrap();
    assert_eq!(view.runs().len(), 1);
    let run = view.runs().first().unwrap();
    assert_eq!(run.text(), expected);
    assert_eq!(run.font(), Some(u16::MAX));
    assert_eq!(run.size_pt(), Some(1.25));
    assert_eq!(run.face(), Some(u16::MAX));
    assert_eq!(run.color(), Some(u16::MAX));
    assert_eq!(document.as_bytes(), bytes);
}

fn text_selector_property(value: &str) {
    use bondline_cdxml::{Document, Error, TextError, Writer};
    for index in 0..4 {
        let mut attributes = [("font", "0"), ("size", "12"), ("face", "0"), ("color", "1")];
        attributes[index].1 = value;
        let property = attributes[index].0;
        let mut writer = Writer::new(&[]).unwrap();
        writer.start_element("t", &[]).unwrap();
        if let Err(error) = writer.empty_element("s", &attributes) {
            assert!(matches!(error, Error::InvalidAttributeValue));
            continue;
        }
        writer.end_element().unwrap();
        let bytes = writer.finish().unwrap();
        let document = Document::parse(&bytes).unwrap();
        let text = document.xml().root_element().first_element_child().unwrap();
        let integer = bondline_cdxml::parse_integer(value)
            .ok()
            .and_then(|value| u16::try_from(value).ok());
        let size = value
            .trim_matches([' ', '\t', '\r', '\n'])
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite() && *value > 0.0);
        match document.styled_text(text) {
            Ok(view) => {
                assert!(if index == 1 {
                    size.is_some()
                } else {
                    integer.is_some()
                });
                let run = view.runs().first().unwrap();
                assert_eq!(run.font(), if index == 0 { integer } else { Some(0) });
                assert_eq!(run.size_pt(), if index == 1 { size } else { Some(12.0) });
                assert_eq!(run.face(), if index == 2 { integer } else { Some(0) });
                assert_eq!(run.color(), if index == 3 { integer } else { Some(1) });
                assert_eq!(run.text(), "");
            }
            Err(error) => {
                assert!(if index == 1 {
                    size.is_none()
                } else {
                    integer.is_none()
                });
                assert_eq!(error, TextError::InvalidProperty(property));
            }
        }
        assert_eq!(document.as_bytes(), bytes);
    }
}

fn text_views(document: &bondline_cdxml::Document<'_>, source: &[u8]) {
    use bondline_cdxml::{Document, TextError};
    let foreign = Document::parse(b"<CDXML><t><s/></t></CDXML>").unwrap();
    let foreign_text = foreign.xml().root_element().first_element_child().unwrap();
    assert_eq!(
        document.styled_text(foreign_text).err(),
        Some(TextError::InvalidElement)
    );
    assert_eq!(
        document.styled_text(document.xml().root_element()).err(),
        Some(TextError::InvalidElement)
    );
    for element in document.xml().descendants().filter(|node| {
        node.is_element() && node.tag_name().namespace().is_none() && node.tag_name().name() == "t"
    }) {
        if let Ok(view) = document.styled_text(element) {
            assert_eq!(view.element(), element);
            assert!(!view.runs().is_empty());
            assert!(view.runs().len() <= bondline_cdxml::MAX_NODES as usize);
            for run in view.runs() {
                let node = run.element();
                assert_eq!(node.parent(), Some(element));
                assert_eq!(node.tag_name().name(), "s");
                assert!(node.tag_name().namespace().is_none());
                assert!(node.children().all(|child| child.is_text()));
                let decoded: String = node.children().filter_map(|child| child.text()).collect();
                assert_eq!(run.text(), decoded);
                assert!(run.text().len() <= source.len());
                let ordinary = |name| {
                    node.attributes()
                        .find(|attribute| {
                            attribute.namespace().is_none() && attribute.name() == name
                        })
                        .map(|attribute| attribute.value())
                };
                for (name, actual) in [
                    ("font", run.font()),
                    ("face", run.face()),
                    ("color", run.color()),
                ] {
                    let expected = ordinary(name).map(|raw| {
                        u16::try_from(bondline_cdxml::parse_integer(raw).unwrap()).unwrap()
                    });
                    assert_eq!(actual, expected);
                }
                let expected_size = ordinary("size").map(|raw| {
                    raw.trim_matches([' ', '\t', '\r', '\n'])
                        .parse::<f64>()
                        .unwrap()
                });
                assert_eq!(run.size_pt(), expected_size);
                assert!(
                    run.size_pt()
                        .is_none_or(|value| value.is_finite() && value > 0.0)
                );
                for attribute in node
                    .attributes()
                    .filter(|attribute| attribute.namespace().is_some())
                {
                    if let Some(namespace) = attribute.namespace() {
                        assert_eq!(
                            run.element().attribute((namespace, attribute.name())),
                            Some(attribute.value())
                        );
                    }
                }
            }
        }
        assert_eq!(document.as_bytes(), source);
    }
}

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

fn carbon_defaults(stored: &bondline_io::StoredDocument) {
    let counts = stored
        .analysis_hydrogen_counts(&Default::default())
        .unwrap();
    if counts.is_empty() {
        return;
    }
    assert!(stored.has_analysis_coverage());
    assert!(!stored.has_display_projection() && !stored.is_fully_mapped());
    let mut h_total = 0_usize;
    let mut omitted = 0_usize;
    let mut carbons = 0_usize;
    let mut bond_order = 0_usize;
    for fragment in stored.model().pages()[0].fragments() {
        for atom in fragment.atoms() {
            assert_eq!(atom.element.get(), 6);
            assert_eq!(atom.charge, 0);
            assert_eq!(atom.radical, bondline_model::Radical::None);
            assert!(atom.isotope.is_none());
            let h = match atom.explicit_hydrogens {
                Some(written) => {
                    assert!(!counts.contains_key(&atom.id));
                    written
                }
                None => {
                    omitted += 1;
                    counts[&atom.id]
                }
            };
            h_total += usize::from(h);
            carbons += 1;
        }
        for bond in fragment.bonds() {
            bond_order += match bond.order {
                bondline_model::BondOrder::Single => 1,
                bondline_model::BondOrder::Double => 2,
                bondline_model::BondOrder::Triple => 3,
                _ => panic!("an unresolved bond cannot qualify for carbon defaults"),
            };
        }
    }
    assert_eq!(counts.len(), omitted);
    // Valence conservation for neutral, uncharged carbons in any ring or chain.
    assert_eq!(4 * carbons, 2 * bond_order + h_total);
}

fuzz_target!(|data: &[u8]| {
    if data.len() > INPUT_LIMIT {
        return;
    }
    if let Ok(value) = std::str::from_utf8(data) {
        integer_property(value);
        writer_property(value);
        resource_value_property(value);
        writer_text_property(value);
        text_selector_property(value);
        label_default_value_property(value);
    }
    structured_label_defaults(data);
    if let Ok(document) = bondline_cdxml::Document::parse(data) {
        assert_eq!(document.as_bytes(), data);
        assert_eq!(document.to_bytes(), data);
        assert_eq!(document.xml().root_element().tag_name().name(), "CDXML");
        text_views(&document, data);
        label_default_views(&document, data);
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
                carbon_defaults(&stored);
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
                assert!(
                    edited
                        .analysis_hydrogen_counts(&Default::default())
                        .unwrap()
                        .is_empty()
                );
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
