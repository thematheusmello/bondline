use bondline_io::{StoredDocument, mol::MolTransform};
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Element, Fragment, FragmentId, PageId, PageSize, Point2,
};
use bondline_render::{Cancellation, PageView, cpu_scene};
use std::{
    io::Write,
    process::{Command, Stdio},
    sync::Arc,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn zoomed_mol_uses_independent_literal_cco_h_counts_without_changing_source_or_none() -> TestResult
{
    let raw = include_bytes!("fixtures/sdf-preview/valence-absent-mixed.sdf")
        .get(0..371)
        .ok_or("original ethanol MOL")?;
    let mut child = Command::new(std::env::var_os("BONDLINE_PYTHON").ok_or("pinned Python")?)
        .args([
            "-B",
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/mol_oracle.py"),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("oracle stdin")?.write_all(raw)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let external: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(external["smiles"], "CCO");
    assert_eq!(external["formula"], "C2H6O");
    for (atom, h) in external["atoms"]
        .as_array()
        .ok_or("atoms")?
        .iter()
        .zip([3, 2, 1])
    {
        assert_eq!(atom["hydrogens"], h);
    }
    let stored = StoredDocument::from_mol(
        Arc::from(raw),
        PageSize::A4,
        MolTransform::new(24.0, Point2::new(72.0, 72.0)?)?,
    )?;
    let before = stored.clone();
    let model = stored.model_snapshot();
    assert!(
        model
            .pages()
            .iter()
            .flat_map(|p| p.fragments())
            .flat_map(|f| f.atoms())
            .all(|atom| atom.explicit_hydrogens.is_none())
    );
    let mut reference = StoredDocument::default();
    let atoms = [
        (1, 6, 42.0, 60.0, 3),
        (2, 6, 78.0, 60.0, 2),
        (3, 8, 78.0, 24.0, 1),
    ]
    .into_iter()
    .map(|(id, z, x, y, h)| {
        let mut atom = Atom::new(
            AtomId::new(id).ok_or("atom")?,
            Element::new(z).ok_or("element")?,
            Some(Point2::new(x, y)?),
        );
        atom.explicit_hydrogens = Some(h);
        Ok(atom)
    })
    .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    let bonds = [(1, 1, 2), (2, 2, 3)]
        .into_iter()
        .map(|(id, a, b)| {
            Ok(Bond {
                double_position: bondline_model::DoublePosition::Unspecified,
                id: BondId::new(id).ok_or("bond")?,
                begin: AtomId::new(a).ok_or("begin")?,
                end: AtomId::new(b).ok_or("end")?,
                order: BondOrder::Single,
            })
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    let page_id = PageId::new(1).ok_or("page")?;
    reference.model_mut().append_fragments(
        page_id,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            atoms,
            bonds,
        )?],
    )?;
    let camera = PageView::zoom(360, 240, PageSize::A4, 200.0, Point2::new(72.0, 72.0)?)?;
    let cancel = Cancellation::default();
    let actual =
        bondline_io::display::drawing_page_scene_with_view(&camera, &stored, page_id, &cancel)?;
    let expected = camera.graph_scene(
        reference.model().pages().first().ok_or("reference page")?,
        &cancel,
    )?;
    let literal = camera.graph_scene(model.pages().first().ok_or("source page")?, &cancel)?;
    assert_eq!(
        cpu_scene(actual.scene())?,
        cpu_scene(expected.scene())?,
        "actual OH/H3/H2 must agree with independently declared counts/coordinates"
    );
    assert_ne!(
        cpu_scene(actual.scene())?,
        cpu_scene(literal.scene())?,
        "unresolved source None is a meaningful missing-H control"
    );
    assert_eq!(stored, before);
    assert!(Arc::ptr_eq(&stored.model_snapshot(), &model));
    assert_eq!(stored.export_mol()?, raw);
    Ok(())
}
