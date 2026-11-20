//! Differential coordinate bytes. The parser supplies spans; no production patcher supplies expectations.
use bondline_cdx::{Document, RecordKind};
use bondline_io::StoredDocument;
use std::{collections::BTreeMap, sync::Arc};

pub(super) fn check(stored: &StoredDocument, source: &Document<'_>) {
    let before = source.as_bytes();
    let page = &stored.model().pages()[0];
    let mut nodes = BTreeMap::new();
    for (index, record) in source.records().iter().enumerate() {
        match record.kind() {
            RecordKind::ObjectStart { tag: 0x8004, .. } => {
                nodes.insert(index, [None, None]);
            }
            RecordKind::Property {
                tag: 0x0200,
                payload,
                ..
            } => {
                let owned = nodes.get_mut(&record.parent().unwrap()).unwrap();
                assert!(owned[0].replace(payload.clone()).is_none());
            }
            RecordKind::Property {
                tag: 0x0204,
                payload,
                ..
            } => {
                let text = &source.records()[record.parent().unwrap()];
                assert!(matches!(
                    text.kind(),
                    RecordKind::ObjectStart { tag: 0x8006, .. }
                ));
                let node = text.parent().unwrap();
                assert!(
                    nodes.get_mut(&node).unwrap()[1]
                        .replace(payload.clone())
                        .is_none()
                );
            }
            _ => {}
        }
    }
    let atoms: Vec<_> = page.fragments().iter().flat_map(|f| f.atoms()).collect();
    assert_eq!(atoms.len(), nodes.len());
    let mut expected = before.to_vec();
    let mut selected = Vec::new();
    let mut overflow = false;
    // Alternating owners cover a subset and repeated raw IDs across fragments.
    for (atom, [point, bounds]) in atoms.iter().zip(nodes.into_values()).step_by(2) {
        assert_eq!(bounds.is_some(), atom.authored_label.is_some());
        let point = point.unwrap();
        assert_eq!(point.len(), 8);
        if let Some(b) = &bounds {
            assert_eq!(b.len(), 16);
        }
        selected.push(atom.id.get());
        for span in std::iter::once(point).chain(bounds) {
            for (i, word) in expected[span].chunks_exact_mut(4).enumerate() {
                let original = i32::from_le_bytes(word.as_ref().try_into().unwrap());
                let delta = if i % 2 == 0 { 8192 } else { -24576 };
                if let Some(new) = original.checked_add(delta) {
                    word.copy_from_slice(&new.to_le_bytes());
                } else {
                    overflow = true;
                }
            }
        }
    }
    assert!(!selected.is_empty());
    let snapshot = stored.model_snapshot();
    let control = (!overflow)
        .then(|| StoredDocument::from_cdx(Arc::from(expected.as_slice()), page.size()).ok())
        .flatten()
        .filter(|doc| {
            doc.can_move_cdx_skeletal_atoms()
                && doc.has_cdx_skeletal_label_preview() == stored.has_cdx_skeletal_label_preview()
        });
    let moved = stored.with_cdx_skeletal_translation(page.id(), &selected, -0.375, 0.125);
    assert_eq!(moved.is_ok(), control.is_some());
    if let Some(control) = control {
        let moved = moved.unwrap();
        assert_eq!(moved.export_cdx().unwrap(), expected);
        assert_eq!(moved.model(), control.model());
        let noop = moved
            .with_cdx_skeletal_translation(page.id(), &selected, 0.5 / 65536., -0.5 / 65536.)
            .unwrap();
        assert!(Arc::ptr_eq(&moved.model_snapshot(), &noop.model_snapshot()));
        let restored = moved
            .with_cdx_skeletal_translation(page.id(), &selected, 0.375, -0.125)
            .unwrap();
        assert_eq!(restored.export_cdx().unwrap(), before);
        assert!(Arc::ptr_eq(&snapshot, &restored.model_snapshot()));
    }
    assert_eq!(stored.export_cdx().unwrap(), before);
    assert!(Arc::ptr_eq(&snapshot, &stored.model_snapshot()));
}
