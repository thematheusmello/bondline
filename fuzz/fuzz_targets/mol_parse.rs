#![no_main]

use bondline_commands::Cancellation;
use bondline_io::mol::{
    MAX_MOL_BYTES, MAX_MOL_ITEMS, MAX_MOL_LINE_BYTES, MAX_MOL_LINES, MolError, MolFile,
    MolTransform, write_v2000,
};
use bondline_io::{StoredDocument, display::drawing_page_scene};
use bondline_model::{Fragment, PageSize, Point2};
use libfuzzer_sys::fuzz_target;
use std::sync::Arc;

fn assert_graph(expected: &Fragment, recovered: &Fragment) {
    assert_eq!(recovered.id(), expected.id());
    assert_eq!(recovered.bonds(), expected.bonds());
    assert_eq!(recovered.atoms().len(), expected.atoms().len());
    for (original, actual) in expected.atoms().iter().zip(recovered.atoms()) {
        assert_eq!(actual.id, original.id);
        assert_eq!(actual.element, original.element);
        assert_eq!(actual.explicit_hydrogens, original.explicit_hydrogens);
        assert_eq!(actual.isotope, original.isotope);
        assert_eq!(actual.charge, original.charge);
        assert_eq!(actual.radical, original.radical);
        let original = original.position.expect("strict MOL mapping has positions");
        let actual = actual
            .position
            .expect("canonical MOL mapping has positions");
        for (before, after) in [
            (original.x_pt(), actual.x_pt()),
            (original.y_pt(), actual.y_pt()),
        ] {
            // The identity transform makes these raw units: half a 4dp step plus f64 error.
            let tolerance = 0.00005 + 4.0 * f64::EPSILON * before.abs().max(after.abs()).max(1.0);
            assert!((after - before).abs() <= tolerance);
        }
    }
}

fuzz_target!(|data: &[u8]| {
    let Ok(document) = MolFile::parse(data) else {
        return;
    };
    assert_eq!(document.as_bytes(), data);
    assert!(data.len() <= MAX_MOL_BYTES);
    assert!(document.atoms().len() <= MAX_MOL_ITEMS);
    assert!(document.bonds().len() <= MAX_MOL_ITEMS);
    let text = std::str::from_utf8(data).expect("accepted MOL is ASCII");
    let line_count = text.split_terminator('\n').count();
    assert!(line_count <= MAX_MOL_LINES);
    for line in text.split_terminator('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        assert!(line.len() <= MAX_MOL_LINE_BYTES);
        assert!(!line.contains('\r'));
    }
    assert!(
        document
            .unsupported()
            .iter()
            .all(|feature| (1..=line_count).contains(&feature.line))
    );

    let transform = MolTransform::new(1.0, Point2::new(0.0, 0.0).unwrap()).unwrap();
    let Ok(fragment) = document.to_fragment(transform) else {
        assert_eq!(document.as_bytes(), data);
        return;
    };
    if let Ok(stored) = StoredDocument::from_mol(Arc::from(data), PageSize::A4, transform) {
        let before = stored.clone();
        let page = stored
            .model()
            .pages()
            .first()
            .expect("owned MOL has a page");
        let _scene = drawing_page_scene(1024, 1024, &stored, page.id(), &Cancellation::default());
        assert_eq!(
            stored, before,
            "scene success or refusal must not author data"
        );
        assert_eq!(stored.export_mol().unwrap(), data);
        let cancel = Cancellation::default();
        cancel.cancel();
        let error = drawing_page_scene(1024, 1024, &stored, page.id(), &cancel)
            .err()
            .expect("pre-cancelled drawing must refuse");
        assert_eq!(error.code(), "cancelled");
        assert_eq!(stored, before);
        assert_eq!(stored.export_mol().unwrap(), data);
    }
    let output = match write_v2000(&fragment, "bondline fuzz", transform) {
        Ok(output) => output,
        Err(MolError::Unsupported { .. } | MolError::CoordinateRange) => {
            assert_eq!(document.as_bytes(), data);
            return;
        }
        Err(error) => panic!("strictly mapped graph unexpectedly failed export: {error}"),
    };
    let canonical = MolFile::parse(&output).expect("successful writer output must parse");
    let recovered = canonical
        .to_fragment(transform)
        .expect("successful writer output must retain the strict graph");
    assert_graph(&fragment, &recovered);
    // Canonical writing may quantize coordinates and normalize properties; raw source is separate.
    assert_eq!(document.as_bytes(), data);
});
