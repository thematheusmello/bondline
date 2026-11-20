use bondline_cdx::{Document, RecordKind};
use bondline_io::StoredDocument;
use bondline_model::{AtomId, Element, PageSize, TextStyle};
use std::sync::Arc;

// The closed skeletal reader assigns model atoms in Node encounter order.
fn surroundings<'a>(source: &Document<'a>, id: AtomId, created: bool) -> Vec<&'a [u8]> {
    let records = source.records();
    let node = records
        .iter()
        .enumerate()
        .filter(|(_, r)| matches!(r.kind(), RecordKind::ObjectStart { tag: 0x8004, .. }))
        .nth(id.get() as usize - 1)
        .unwrap()
        .0;
    let mut children = records.iter().enumerate().filter(|(_, r)| {
        r.parent() == Some(node) && matches!(r.kind(), RecordKind::ObjectStart { tag: 0x8006, .. })
    });
    let child = children.next().map(|(i, _)| i);
    assert_eq!(child.is_some(), created);
    assert!(children.next().is_none());
    records
        .iter()
        .enumerate()
        .filter(|(i, r)| {
            let scalar = r.parent() == Some(node)
                && matches!(
                    r.kind(),
                    RecordKind::Property {
                        tag: 0x0402 | 0x042b,
                        ..
                    }
                );
            let label = child.is_some_and(|index| *i == index || r.parent() == Some(index)
            || matches!(r.kind(), RecordKind::ObjectEnd {start: Some(owner)} if *owner == index));
            !scalar && !label
        })
        .map(|(_, r)| &source.as_bytes()[r.range()])
        .collect()
}

pub(super) fn check(stored: &StoredDocument, source: &Document<'_>) {
    let page = stored.model().pages().first().unwrap();
    let Some((fragment, atom)) = page
        .fragments()
        .iter()
        .flat_map(|fragment| fragment.atoms().iter().map(move |atom| (fragment, atom)))
        .find(|(_, atom)| {
            atom.authored_label.is_none()
                && atom.show_terminal_carbon_labels.is_none()
                && atom.show_non_terminal_carbon_labels.is_none()
        })
    else {
        return;
    };
    let label = stored.with_new_cdx_atom_label(
        page.id(),
        fragment.id(),
        atom.id,
        Element::new(7).unwrap(),
        2,
        TextStyle::new("Roboto".into(), 12.0, 0, [0.0; 3]).unwrap(),
    );
    assert_eq!(stored.export_cdx().unwrap(), source.as_bytes());
    if source.as_bytes()
        == include_bytes!("../../../crates/io/tests/fixtures/cdx-mixed-skeleton/ethanol.cdx")
    {
        assert_eq!(
            label.as_ref().unwrap().export_cdx().unwrap(),
            include_bytes!(
                "../../../crates/io/tests/fixtures/cdx-skeletal-label-creation/ethanol.cdx"
            )
        );
    }
    if let Ok(edited) = label {
        let bytes = edited.export_cdx().unwrap();
        let parsed = Document::parse(bytes).unwrap();
        assert_eq!(
            surroundings(source, atom.id, false),
            surroundings(&parsed, atom.id, true)
        );
        let reopened = StoredDocument::from_cdx(Arc::from(bytes), PageSize::A4).unwrap();
        assert_eq!(edited.model(), reopened.model());
        assert!(reopened.can_edit_cdx_atom_label(atom.id));
        assert!(!reopened.can_create_cdx_atom_label(atom.id));
        assert!(!reopened.is_fully_mapped());
    }
}
