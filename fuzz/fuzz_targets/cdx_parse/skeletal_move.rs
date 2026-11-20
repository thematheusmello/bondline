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
                nodes.insert(index, None);
            }
            RecordKind::Property {
                tag: 0x0200,
                payload,
                ..
            } => {
                let owned = nodes.get_mut(&record.parent().unwrap()).unwrap();
                assert!(owned.replace(payload.clone()).is_none());
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
    for (atom, slot) in atoms.iter().zip(nodes.into_values()).step_by(2) {
        let span = slot.unwrap();
        assert_eq!(span.len(), 8);
        let (y, x) = expected[span].split_at_mut(4);
        let old_y = i32::from_le_bytes(y.as_ref().try_into().unwrap());
        let old_x = i32::from_le_bytes(x.as_ref().try_into().unwrap());
        selected.push(atom.id.get());
        match (old_y.checked_add(8192), old_x.checked_sub(24576)) {
            (Some(new_y), Some(new_x)) => {
                y.copy_from_slice(&new_y.to_le_bytes());
                x.copy_from_slice(&new_x.to_le_bytes());
            }
            _ => overflow = true,
        }
    }
    assert!(!selected.is_empty());
    let snapshot = stored.model_snapshot();
    let control = (!overflow)
        .then(|| StoredDocument::from_cdx(Arc::from(expected.as_slice()), page.size()).ok())
        .flatten()
        .filter(StoredDocument::has_cdx_skeletal_preview);
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
