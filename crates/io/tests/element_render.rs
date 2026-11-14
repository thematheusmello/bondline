#![cfg(not(target_arch = "wasm32"))]

use bondline_io::{StoredDocument, files, mol::MolTransform};
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Element, Fragment, FragmentId, PageId, PageSize, Point2,
};
use bondline_render::{Cancellation, cpu_scene, graph_page_scene};
use std::{
    fs,
    io::Write,
    process::{Command, Stdio},
    sync::Arc,
};

#[path = "../../../xtask/oracles/files_test_support.rs"]
pub mod file_support;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn consumer(script: &str, bytes: &[u8]) -> TestResult<serde_json::Value> {
    let interpreter = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut process = Command::new(interpreter)
        .arg("-B")
        .arg(script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    process.stdin.take().ok_or("stdin")?.write_all(bytes)?;
    let result = process.wait_with_output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(serde_json::from_slice(&result.stdout)?)
}

#[test]
fn actual_native_cdxml_and_mol_saves_retain_neutral_labels_and_source_authority() -> TestResult {
    let directory = file_support::FileTestDirectory::new()?;
    let mut stored = StoredDocument::default();
    let mut atoms = Vec::new();
    for (id, number, x, hydrogens) in [(1, 7, 144.0, 2), (2, 8, 216.0, 1)] {
        let mut atom = Atom::new(
            AtomId::new(id).ok_or("atom")?,
            Element::try_from(number)?,
            Some(Point2::new(x, 288.0)?),
        );
        atom.explicit_hydrogens = Some(hydrogens);
        atoms.push(atom);
    }
    stored.model_mut().append_fragments(
        PageId::new(1).ok_or("page")?,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            atoms,
            vec![Bond {
                id: BondId::new(1).ok_or("bond")?,
                begin: AtomId::new(1).ok_or("begin")?,
                end: AtomId::new(2).ok_or("end")?,
                order: BondOrder::Single,
            }],
        )?],
    )?;
    let before = stored.clone();
    let reference = cpu_scene(
        graph_page_scene(1024, 1024, stored.model().pages().first().ok_or("page")?)?.scene(),
    )?;
    let xml = directory.path("neutral-λ.cdxml");
    let cancel = Cancellation::default();
    let receipt = files::save_cdxml_file(&xml, &stored, false, &cancel)?;
    let xml_bytes = fs::read(&xml)?;
    assert_eq!(receipt.byte_length, xml_bytes.len());
    let independent = consumer(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../xtask/oracles/cdxml_graph.py"
        ),
        &xml_bytes,
    )?;
    assert_eq!(
        independent,
        serde_json::to_value(
            stored.model().pages()[0]
                .fragments()
                .iter()
                .map(Arc::as_ref)
                .collect::<Vec<_>>()
        )?
    );
    let reopened = files::read_cdxml_file(&xml, &cancel)?.document;
    assert!(reopened.is_fully_mapped());
    assert_eq!(reopened.to_cdxml()?.as_ref(), xml_bytes);
    assert_eq!(reopened.model(), stored.model());
    assert_eq!(
        cpu_scene(
            graph_page_scene(1024, 1024, reopened.model().pages().first().ok_or("page")?)?.scene()
        )?,
        reference
    );

    let mol = directory.path("neutral-λ.mol");
    let frame = MolTransform::new(24.0, Point2::new(72.0, 72.0)?)?;
    let receipt = files::save_mol_file(&mol, &stored, false, frame, &cancel)?;
    let mol_bytes = fs::read(&mol)?;
    assert_eq!(receipt.byte_length, mol_bytes.len());
    let independent = consumer(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/mol_oracle.py"),
        &mol_bytes,
    )?;
    assert_eq!(independent["smiles"], "NO");
    assert_eq!(independent["formula"], "H3NO");
    assert_eq!(independent["atoms"][0]["element"], 7);
    assert_eq!(independent["atoms"][1]["element"], 8);
    assert_eq!(independent["atoms"][0]["hydrogens"], 2);
    assert_eq!(independent["atoms"][1]["hydrogens"], 1);
    assert_eq!(
        independent["atoms"][0]["position"],
        serde_json::json!([3.0, -9.0, 0.0])
    );
    assert_eq!(
        independent["atoms"][1]["position"],
        serde_json::json!([6.0, -9.0, 0.0])
    );
    let reopened = files::read_mol_file(&mol, frame, &cancel)?.document;
    assert!(reopened.is_fully_mapped());
    assert_eq!(reopened.to_mol(frame)?.as_ref(), mol_bytes);
    assert_eq!(reopened.model(), stored.model());
    assert_eq!(
        cpu_scene(
            graph_page_scene(1024, 1024, reopened.model().pages().first().ok_or("page")?)?.scene()
        )?,
        reference
    );
    assert!(
        reopened.to_cdxml().is_err(),
        "renderability does not authorize dropping MOL source"
    );
    assert_eq!(stored, before);
    Ok(())
}

#[test]
fn new_element_ink_does_not_promote_unmapped_sources_to_editable_native_profiles() -> TestResult {
    let bytes=b"<CDXML Future='preserved'><page><fragment id='1'><n id='2' Element='8' NumHydrogens='2' p='144 288'/></fragment></page></CDXML>";
    let mut source = StoredDocument::from_cdxml(Arc::from(bytes.as_slice()), PageSize::A4)?;
    assert!(!source.is_fully_mapped());
    assert_eq!(source.export_cdxml()?, bytes);
    source
        .model_mut()
        .set_page_size(PageId::new(1).ok_or("page")?, PageSize::new(612.0, 792.0)?)?;
    assert!(source.to_cdxml().is_err());
    Ok(())
}
