#![cfg(not(target_arch = "wasm32"))]

use bondline_io::{
    StoredDocument,
    files::{read_cdx_file, save_cdx_file},
};
use bondline_model::{PageSize, Point2};
use bondline_render::Cancellation;
use serde_json::{Value, json};
use std::{fs, process::Command};
#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(
    dead_code,
    reason = "reuse the existing atomic isolated file directory"
)]
mod support;
const SOURCE: &[u8] = include_bytes!("../../cdx/tests/fixtures/external-ethane-indigo-1.47.0.cdx");
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn original_external_file_end_owned_io_keeps_styles_spans_and_literal_projection() -> TestResult {
    let directory = support::FileTestDirectory::new()?;
    let input = directory.path("input-λ.cdx");
    let output = directory.path("output-化学.data");
    let refused = directory.path("edited-refused.cdx");
    fs::write(&input, SOURCE)?;
    let cancel = Cancellation::default();
    let loaded = read_cdx_file(&input, &cancel)?;
    assert_eq!(loaded.path, fs::canonicalize(&input)?);
    let mut stored = loaded.document;
    assert_eq!(stored.export_cdx()?, SOURCE);
    assert!(!stored.is_fully_mapped());
    assert_eq!(stored.source_info().ok_or("source")?.byte_length, 258);
    let page = stored.model().pages().first().ok_or("page")?;
    assert_eq!(page.size(), PageSize::A4);
    assert_eq!(page.fragments().len(), 1);
    let fragment = page.fragments().first().ok_or("fragment")?;
    assert_eq!(fragment.id().get(), 1);
    assert_eq!(fragment.atoms().len(), 2);
    for (atom, (id, x)) in fragment.atoms().iter().zip([(1, 0.0), (2, 45.0)]) {
        assert_eq!(atom.id.get(), id);
        assert_eq!(atom.element.get(), 6);
        assert_eq!(atom.position, Some(Point2::new(x, 0.0)?));
        assert_eq!(atom.explicit_hydrogens, None);
        assert_eq!(atom.isotope, None);
        assert_eq!(atom.charge, 0);
        assert_eq!(atom.radical, bondline_model::Radical::None);
    }
    assert_eq!(fragment.bonds().len(), 1);
    let bond = fragment.bonds().first().ok_or("bond")?;
    assert_eq!(
        (bond.id.get(), bond.begin.get(), bond.end.get(), bond.order),
        (1, 1, 2, bondline_model::BondOrder::Single)
    );
    let before = stored.clone();
    let page_id = page.id();
    let receipt = save_cdx_file(&output, &stored, false, &cancel)?;
    assert_eq!(receipt.byte_length, 258);
    assert_eq!(fs::read(&input)?, SOURCE);
    assert_eq!(fs::read(&output)?, SOURCE);
    let consumed = Command::new(std::env::var_os("BONDLINE_PYTHON").ok_or("BONDLINE_PYTHON")?)
        .arg("-B")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../cdx/tests/external_ethane_oracle.py"
        ))
        .stdin(fs::File::open(&output)?)
        .output()?;
    assert!(
        consumed.status.success(),
        "{}",
        String::from_utf8_lossy(&consumed.stderr)
    );
    let independent: Value = serde_json::from_slice(&consumed.stdout)?;
    assert_eq!(
        independent,
        json!({"atoms":2,"bonds":1,"formula":"C2H6","smiles":"CC","hydrogens":[3,3],"byte_length":258,"record_count":24,"header_bytes":28,"document_end":[254,256],"file_end":[256,258],"raw_y_x16_16":[[0,0],[0,2949120]]})
    );
    let reopened = read_cdx_file(&output, &cancel)?;
    assert_eq!(reopened.document, before);
    assert_eq!(reopened.document.to_cdx()?.as_ref(), SOURCE);
    stored
        .model_mut()
        .set_page_size(page_id, PageSize::new(612.0, 792.0)?)?;
    assert!(stored.source_info().ok_or("source")?.modified);
    assert!(stored.to_cdx().is_err());
    assert_eq!(
        save_cdx_file(&refused, &stored, false, &cancel)
            .err()
            .ok_or("changed source saved")?
            .code(),
        "unsupported_content"
    );
    assert!(!refused.try_exists()?);
    assert!(before.to_cdxml().is_err());
    assert_eq!(before.export_cdx()?, SOURCE);
    assert_eq!(fs::read(&output)?, SOURCE);
    let restored: StoredDocument = before;
    assert!(!restored.source_info().ok_or("source")?.modified);
    assert_eq!(restored.to_cdx()?.as_ref(), SOURCE);
    Ok(())
}
