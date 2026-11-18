//! Snapshot/source assertions are independent of the new profile's private representation.
use bondline_io::StoredDocument;
use bondline_model::{BondOrder, Fragment, PageSize};
use std::{borrow::Cow, sync::Arc};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn edited_output_never_replaces_original_baseline_and_logical_restore_borrows_the_same_input()
-> TestResult {
    let source=b"<CDXML CreationProgram='Bondline 0.1.0'><!--keep--><page Height='792.00' Width='612.000' DrawingSpace='Poster' id='91'><fragment id='400'><b id='900' E='207' Order='1' B='101'/><n id='101' NodeType='Element' Element='6' Charge='0' Radical='None' p='100.00 200.000' NumHydrogens='&#51;'/><n id='207' NodeType='Element' Element='6' Charge='0' Radical='None' p='140.00 200.000' NumHydrogens='&#51;'/></fragment></page></CDXML>";
    let input: Arc<[u8]> = source.as_slice().into();
    let mut stored = StoredDocument::from_cdxml(Arc::clone(&input), PageSize::A4)?;
    assert!(stored.is_fully_mapped());
    let baseline = stored.model_snapshot();
    let before = stored.clone();
    assert!(Arc::ptr_eq(&baseline, &before.model_snapshot()));
    assert_eq!(stored.export_cdxml()?.as_ptr(), input.as_ptr());
    let original_json = serde_json::json!({"pages":[{"id":1,"size":{"width_pt":612.0,"height_pt":792.0},"fragments":[{"id":1,"atoms":[{"id":1,"element":6,"position":{"x_pt":100.0,"y_pt":200.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":3},{"id":2,"element":6,"position":{"x_pt":140.0,"y_pt":200.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":3}],"bonds":[{"id":1,"begin":1,"end":2,"order":"single"}]}]}]});
    assert_eq!(serde_json::to_value(stored.model())?, original_json);
    assert_eq!(
        serde_json::to_value(stored.source_info())?,
        serde_json::json!({"format":"cdxml","byte_length":source.len(),"modified":false})
    );
    let page = stored.model().pages()[0].id();
    let f = stored.model().pages()[0].fragments()[0].as_ref();
    let mut atoms = f.atoms().to_vec();
    let mut bonds = f.bonds().to_vec();
    atoms[0].explicit_hydrogens = Some(2);
    atoms[1].explicit_hydrogens = Some(2);
    bonds[0].order = BondOrder::Double;
    let replacement = Fragment::new(f.id(), atoms, bonds)?;
    stored.model_mut().replace_fragment(page, replacement)?;
    let edited_snapshot = stored.model_snapshot();
    assert!(!Arc::ptr_eq(&baseline, &edited_snapshot));
    let edited_json = serde_json::to_value(stored.model())?;
    let info = stored.source_info().ok_or("source info")?;
    assert_eq!(info.format, "cdxml");
    assert_eq!(info.byte_length, source.len());
    assert!(info.modified);
    let expected = std::str::from_utf8(source)?
        .replacen("Order='1'", "Order='2'", 1)
        .replace("NumHydrogens='&#51;'", "NumHydrogens='2'");
    for _ in 0..2 {
        let output = stored.to_cdxml()?;
        assert!(matches!(output, Cow::Owned(_)));
        assert_eq!(output.as_ref(), expected.as_bytes());
        assert_eq!(stored.source_info(), Some(info.clone()));
        assert!(Arc::ptr_eq(&edited_snapshot, &stored.model_snapshot()));
        assert_eq!(serde_json::to_value(stored.model())?, edited_json);
    }
    assert_eq!(input.as_ref(), source);
    assert_eq!(before.to_cdxml()?.as_ref(), source);
    assert!(Arc::ptr_eq(&baseline, &before.model_snapshot()));
    assert!(stored.export_cdxml().is_err());
    *stored.model_mut() = baseline.as_ref().clone();
    assert!(
        !Arc::ptr_eq(&baseline, &stored.model_snapshot()),
        "restoration need not recreate the original allocation"
    );
    let restored = stored.to_cdxml()?;
    assert!(matches!(restored, Cow::Borrowed(_)));
    assert_eq!(restored.as_ref().as_ptr(), input.as_ptr());
    assert_eq!(restored.as_ref(), source);
    assert_eq!(stored, before);
    assert_eq!(serde_json::to_value(stored.model())?, original_json);
    assert!(!stored.source_info().ok_or("source")?.modified);
    Ok(())
}
