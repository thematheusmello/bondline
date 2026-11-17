#![no_main]

use bondline_cdxml::{Document, Edit, Error};
use libfuzzer_sys::fuzz_target;

const MAX_CASE_BYTES: usize = 16 * 1024;
const EMPTY: &[u8] = b"<CDXML/>";
const A: &str = "batch_fuzz_value";
const B: &str = "batch_fuzz_other";
const VALUE: &str = "μ & < > \" ' \t\r\n";
const CHILD: &str =
    "<bf:batch_child xmlns:bf=\"urn:bondline:fuzz:batch\" xmlns=\"\" marker=\"added\"/>";

fn shallow_resource(error: &Error) -> bool {
    match error {
        Error::InputTooLarge | Error::AttributeLimit | Error::NodeLimit => true,
        // No direct roxmltree dependency: pinned0.21.2 parse.rs:264–265 has this exact
        // Display only for NodesLimitReached. Other XML errors must remain failures.
        Error::Xml(inner) => inner.to_string() == "nodes limit reached",
        _ => false,
    }
}

fn fragment_resource(error: &Error) -> bool {
    matches!(error, Error::DepthLimit) || shallow_resource(error)
}

fn fixed_edits(document: &Document<'_>, input: &[u8]) {
    let root = document.xml().root_element();
    assert_eq!(document.apply_edits(&[]).unwrap(), input);
    let edits = [
        Edit::SetAttribute {
            element: root,
            name: A,
            value: VALUE,
        },
        Edit::SetAttribute {
            element: root,
            name: B,
            value: "checked",
        },
        Edit::AppendChildren {
            element: root,
            xml: CHILD,
        },
    ];
    match document.apply_edits(&edits) {
        Ok(output) => {
            let written = Document::parse(&output).unwrap();
            let after = written.xml().root_element();
            assert_eq!(after.attribute(A), Some(VALUE));
            assert_eq!(after.attribute(B), Some("checked"));
            assert_eq!(after.children().count(), root.children().count() + 1);
            for (before, after) in root.children().zip(after.children()) {
                assert_eq!(
                    input.get(before.range()).unwrap(),
                    output.get(after.range()).unwrap(),
                    "root-only edits must retain every existing child subtree byte"
                );
            }
            let child = after.last_element_child().unwrap();
            assert_eq!(
                child.tag_name().namespace(),
                Some("urn:bondline:fuzz:batch")
            );
            assert_eq!(child.tag_name().name(), "batch_child");
            assert_eq!(child.attribute("marker"), Some("added"));
        }
        // This adds a depth-one child; an existing accepted deep subtree is untouched.
        Err(error) => assert!(
            shallow_resource(&error),
            "fixed valid edit failed: {error:?}"
        ),
    }
    assert_eq!(document.as_bytes(), input);

    let duplicate = [
        Edit::SetAttribute {
            element: root,
            name: A,
            value: VALUE,
        },
        Edit::SetAttribute {
            element: root,
            name: A,
            value: VALUE,
        },
    ];
    assert!(matches!(
        document.apply_edits(&duplicate),
        Err(Error::ConflictingEdits)
    ));
    let duplicate_empty_append = [
        Edit::AppendChildren {
            element: root,
            xml: "",
        },
        Edit::AppendChildren {
            element: root,
            xml: "",
        },
    ];
    assert!(matches!(
        document.apply_edits(&duplicate_empty_append),
        Err(Error::ConflictingEdits)
    ));
    let foreign = Document::parse(EMPTY).unwrap();
    assert!(matches!(
        document.apply_edits(&[Edit::SetAttribute {
            element: foreign.xml().root_element(),
            name: A,
            value: VALUE,
        }]),
        Err(Error::InvalidElement)
    ));
    assert_eq!(foreign.as_bytes(), EMPTY);
    assert_eq!(document.as_bytes(), input);
}

fn nested_appends(document: &Document<'_>, input: &[u8]) {
    let root = document.xml().root_element();
    let Some(child) = root.first_element_child() else {
        return;
    };
    let edits = [
        Edit::AppendChildren {
            element: root,
            xml: "<bf:root_added xmlns:bf='urn:bondline:fuzz:nested' xmlns='' marker='root'/>",
        },
        Edit::AppendChildren {
            element: child,
            xml: "<bf:child_added xmlns:bf='urn:bondline:fuzz:nested' xmlns='' marker='child'/>",
        },
    ];
    let output = match (
        document.apply_edits(&edits),
        document.apply_edits(&[edits[1], edits[0]]),
    ) {
        (Ok(first), Ok(second)) => {
            assert_eq!(
                first, second,
                "owner nesting must not depend on caller order"
            );
            first
        }
        (Err(first), Err(second)) => {
            // The only additions are at depth one/two, never inside the deepest subtree.
            assert!(
                shallow_resource(&first) && shallow_resource(&second),
                "fixed nested composition failed: {first:?}, {second:?}"
            );
            assert_eq!(document.as_bytes(), input);
            return;
        }
        (first, second) => panic!("caller order changed success: {first:?}, {second:?}"),
    };
    let written = Document::parse(&output).unwrap();
    let after_root = written.xml().root_element();
    let after_child = after_root.first_element_child().unwrap();
    assert_eq!(after_child.tag_name(), child.tag_name());
    assert_eq!(after_root.children().count(), root.children().count() + 1);
    assert_eq!(after_child.children().count(), child.children().count() + 1);
    let root_added = after_root.last_element_child().unwrap();
    let child_added = after_child.last_element_child().unwrap();
    assert_eq!(root_added.parent(), Some(after_root));
    assert_eq!(child_added.parent(), Some(after_child));
    assert_eq!(
        root_added.tag_name().namespace(),
        Some("urn:bondline:fuzz:nested")
    );
    assert_eq!(
        child_added.tag_name().namespace(),
        Some("urn:bondline:fuzz:nested")
    );
    assert_eq!(root_added.tag_name().name(), "root_added");
    assert_eq!(child_added.tag_name().name(), "child_added");
    assert_eq!(root_added.attribute("marker"), Some("root"));
    assert_eq!(child_added.attribute("marker"), Some("child"));
    // The edited child changes as a whole; all other old root subtrees stay byte-exact.
    for (before, after) in root.children().zip(after_root.children()) {
        if before.id() != child.id() {
            assert_eq!(
                input.get(before.range()).unwrap(),
                output.get(after.range()).unwrap()
            );
        }
    }
    // Appending to the selected child leaves each of its existing subtrees untouched.
    for (before, after) in child.children().zip(after_child.children()) {
        assert_eq!(
            input.get(before.range()).unwrap(),
            output.get(after.range()).unwrap()
        );
    }
    assert_eq!(document.as_bytes(), input);
}

fn raw_text(value: &str) {
    let document = Document::parse(EMPTY).unwrap();
    let root = document.xml().root_element();
    let allowed = value.chars().all(|ch| {
        matches!(ch,
        '\t' | '\n' | '\r' | ' '..='\u{d7ff}' | '\u{e000}'..='\u{fffd}'
        | '\u{10000}'..='\u{10ffff}')
    });
    let result = document.apply_edits(&[Edit::SetAttribute {
        element: root,
        name: A,
        value,
    }]);
    if allowed {
        // <=16KiB can escape to <=96KiB, far below all syntax resource limits.
        let output = result.unwrap();
        let written = Document::parse(&output).unwrap();
        assert_eq!(written.xml().root_element().attribute(A), Some(value));
    } else {
        assert!(matches!(result, Err(Error::InvalidAttributeValue)));
    }
    assert_eq!(document.as_bytes(), EMPTY);
    match document.apply_edits(&[Edit::AppendChildren {
        element: root,
        xml: value,
    }]) {
        Ok(output) => {
            Document::parse(&output).unwrap();
            if value.is_empty() {
                assert_eq!(output, EMPTY);
            } else {
                assert_eq!(output, format!("<CDXML>{value}</CDXML>").as_bytes());
            }
        }
        Err(error) => assert!(
            matches!(error, Error::InvalidFragment) || fragment_resource(&error),
            "fragment error must be classified, not arbitrary malformed output: {error:?}"
        ),
    }
    assert_eq!(document.as_bytes(), EMPTY);
}

fn invalid_fragments() {
    let document = Document::parse(b"<CDXML xmlns:f='urn:inherited-only'/>").unwrap();
    let root = document.xml().root_element();
    for xml in [
        "<x>",
        "<?xml version='1.0'?><x/>",
        "<!DOCTYPE x><x/>",
        "</CDXML><outside/><CDXML>",
        "<f:child/>",
    ] {
        assert!(
            matches!(
                document.apply_edits(&[Edit::AppendChildren { element: root, xml }]),
                Err(Error::InvalidFragment)
            ),
            "accepted invalid/non-self-contained fragment: {xml}"
        );
        assert_eq!(
            document.as_bytes(),
            b"<CDXML xmlns:f='urn:inherited-only'/>"
        );
    }
}

fn default_namespace_controls() {
    let input = b"<CDXML><scope xmlns='urn:destination'><old/></scope></CDXML>";
    let document = Document::parse(input).unwrap();
    let scope = document.xml().root_element().first_element_child().unwrap();
    assert_eq!(
        document
            .apply_edits(&[Edit::AppendChildren {
                element: scope,
                xml: "",
            }])
            .unwrap(),
        input,
        "empty append never inherits or changes a namespace"
    );
    assert!(
        matches!(
            document.apply_edits(&[Edit::AppendChildren {
                element: scope,
                xml: "<!--prefix--><plain/>",
            }]),
            Err(Error::InvalidFragment)
        ),
        "a new unqualified element must not inherit destination scope"
    );
    for (xml, expected) in [
        ("<plain xmlns=''/>", Some("")),
        ("<plain xmlns='urn:explicit'/>", Some("urn:explicit")),
    ] {
        let output = document
            .apply_edits(&[Edit::AppendChildren {
                element: scope,
                xml,
            }])
            .unwrap();
        let written = Document::parse(&output).unwrap();
        let after_scope = written.xml().root_element().first_element_child().unwrap();
        let added = after_scope.last_element_child().unwrap();
        assert_eq!(after_scope.tag_name().namespace(), Some("urn:destination"));
        assert_eq!(added.tag_name().name(), "plain");
        assert_eq!(added.tag_name().namespace(), expected);
        assert_eq!(after_scope.children().count(), scope.children().count() + 1);
    }
    assert_eq!(document.as_bytes(), input);
}

fuzz_target!(|data: &[u8]| {
    if data.len() > MAX_CASE_BYTES {
        return;
    }
    invalid_fragments();
    default_namespace_controls();
    if let Ok(document) = Document::parse(data) {
        fixed_edits(&document, data);
        nested_appends(&document, data);
    }
    if let Ok(value) = std::str::from_utf8(data) {
        raw_text(value);
    }
});
