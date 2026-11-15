//! Independent authoring layout expectations; old converter policies stay separate.

use bondline_io::smiles::SmilesDocument;
use bondline_model::{PageSize, Point2};
use bondline_render::Cancellation;
use std::sync::Arc;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn benzene_authoring_layout_keeps_all_six_literal_ch_atoms_and_source() -> TestResult {
    let source = b"c1ccccc1\r\n";
    let owner = SmilesDocument::parse(Arc::from(source.as_slice()))?;
    let before = owner.clone();
    let document = owner.layout_authoring(
        64.0,
        Point2::new(96.0, 96.0)?,
        PageSize::A4,
        &Cancellation::default(),
    )?;
    assert_eq!(document.pages().len(), 1);
    let page = document.pages().first().ok_or("page")?;
    assert_eq!(page.size(), PageSize::A4);
    assert_eq!(page.fragments().len(), 1);
    let fragment = page.fragments().first().ok_or("fragment")?;
    assert_eq!((fragment.atoms().len(), fragment.bonds().len()), (6, 6));
    assert!(fragment.atoms().iter().all(|atom| atom.element.get() == 6
        && atom.explicit_hydrogens == Some(1)
        && atom.position.is_some()));
    assert_eq!(owner, before);
    assert_eq!(owner.as_bytes(), source);
    assert_eq!(owner.as_bytes().as_ptr(), before.as_bytes().as_ptr());
    Ok(())
}

use bondline_model::{BondOrder, Document, Fragment, Radical};
use bondline_render::{
    cpu_scene, gpu_scene, graph_page_scene, graph_page_scene_with_display_hydrogens,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    future::Future,
    io::Write as _,
    process::{Command, Stdio},
    task::{Context, Poll, Wake, Waker},
    thread,
};

const BENZENE: &[(u32, u32, u8)] = &[
    (1, 2, 2),
    (2, 3, 1),
    (3, 4, 2),
    (4, 5, 1),
    (5, 6, 2),
    (1, 6, 1),
];
const PYRROLE: &[(u32, u32, u8)] = &[(1, 2, 1), (2, 3, 2), (3, 4, 1), (4, 5, 1), (1, 5, 2)];
const LENGTH: f64 = 64.0;
const ORIGIN: f64 = 96.0;

fn parse(source: &str) -> TestResult<SmilesDocument> {
    Ok(SmilesDocument::parse(Arc::from(source.as_bytes()))?)
}

fn layout(owner: &SmilesDocument) -> TestResult<Document> {
    Ok(owner.layout_authoring(
        LENGTH,
        Point2::new(ORIGIN, ORIGIN)?,
        PageSize::A4,
        &Cancellation::default(),
    )?)
}

fn retained(owner: &SmilesDocument, before: &SmilesDocument) {
    assert_eq!(owner, before);
    assert_eq!(owner.as_bytes().as_ptr(), before.as_bytes().as_ptr());
    assert_eq!(owner.atoms().as_ptr(), before.atoms().as_ptr());
    assert_eq!(owner.bonds().as_ptr(), before.bonds().as_ptr());
}

fn fragment(document: &Document) -> TestResult<&Fragment> {
    assert_eq!(document.pages().len(), 1);
    let page = document.pages().first().ok_or("page")?;
    assert_eq!((page.id().get(), page.size()), (1, PageSize::A4));
    assert_eq!(page.fragments().len(), 1);
    Ok(page.fragments().first().ok_or("fragment")?)
}

fn point(fragment: &Fragment, id: u32) -> TestResult<[f64; 2]> {
    let p = fragment
        .atoms()
        .iter()
        .find(|a| a.id.get() == id)
        .ok_or("atom")?
        .position
        .ok_or("position")?;
    Ok([p.x_pt(), p.y_pt()])
}

fn near(actual: f64, expected: f64) {
    assert!((actual - expected).abs() <= 1e-8, "{actual} != {expected}");
}

fn literal_graph(
    document: &Document,
    elements: &[u8],
    h: &[u16],
    edges: &[(u32, u32, u8)],
) -> TestResult {
    let f = fragment(document)?;
    assert_eq!(f.id().get(), 1);
    assert_eq!(
        (f.atoms().len(), f.bonds().len()),
        (elements.len(), edges.len())
    );
    assert_eq!(elements.len(), h.len());
    for (index, ((atom, z), h)) in f.atoms().iter().zip(elements).zip(h).enumerate() {
        assert_eq!(
            (atom.id.get(), atom.element.get(), atom.explicit_hydrogens),
            (u32::try_from(index + 1)?, *z, Some(*h))
        );
        assert_eq!(
            (atom.isotope, atom.charge, atom.radical),
            (None, 0, Radical::None)
        );
        assert!(atom.position.is_some());
    }
    for (index, (bond, &(begin, end, order))) in f.bonds().iter().zip(edges).enumerate() {
        assert_eq!(
            (bond.id.get(), bond.begin.get(), bond.end.get()),
            (u32::try_from(index + 1)?, begin, end)
        );
        assert_eq!(
            bond.order,
            match order {
                1 => BondOrder::Single,
                2 => BondOrder::Double,
                _ => return Err("literal edge order".into()),
            }
        );
        let (a, b) = (point(f, begin)?, point(f, end)?);
        near((a[0] - b[0]).hypot(a[1] - b[1]), LENGTH);
    }
    for axis in [0, 1] {
        let minimum = f
            .atoms()
            .iter()
            .map(|a| point(f, a.id.get()).map(|p| p[axis]))
            .collect::<TestResult<Vec<_>>>()?
            .into_iter()
            .fold(f64::INFINITY, f64::min);
        near(minimum, ORIGIN);
    }
    Ok(())
}

fn regular_cycle(f: &Fragment, ids: &[u32]) -> TestResult {
    let points = ids
        .iter()
        .map(|&id| point(f, id))
        .collect::<TestResult<Vec<_>>>()?;
    let count = f64::from(u32::try_from(points.len())?);
    let center = points.iter().fold([0.0, 0.0], |s, p| {
        [s[0] + p[0] / count, s[1] + p[1] / count]
    });
    let radius = LENGTH / (2.0 * (std::f64::consts::PI / count).sin());
    for (index, a) in points.iter().enumerate() {
        near((a[0] - center[0]).hypot(a[1] - center[1]), radius);
        let b = points
            .get((index + 1) % points.len())
            .ok_or("next vertex")?;
        near((a[0] - b[0]).hypot(a[1] - b[1]), LENGTH);
    }
    Ok(())
}

fn consume(
    f: &Fragment,
    source: &str,
    elements: &[u8],
    h: &[u16],
    edges: &[(u32, u32, u8)],
    formula: &str,
    components: &[&[u32]],
) -> TestResult {
    let options = json!({"atoms":elements.iter().zip(h).map(|(&z,&h)| json!([z,null,h])).collect::<Vec<_>>(),
        "bonds":edges,"formula":formula,"components":components});
    let python = std::env::var_os("BONDLINE_PYTHON").ok_or("pinned external interpreter")?;
    let mut child = Command::new(python)
        .arg("-B")
        .arg(format!(
            "{}/tests/smiles_authoring_oracle.py",
            env!("CARGO_MANIFEST_DIR")
        ))
        .arg(source)
        .arg(options.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("stdin")?
        .write_all(&serde_json::to_vec(f)?)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(result["formula"], formula);
    assert_eq!(result["no_assigned_stereo"], true);
    assert_eq!(result["no_radicals"], true);
    Ok(())
}

struct Unpark(thread::Thread);
impl Wake for Unpark {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
}
fn block_on<F: Future>(future: F) -> F::Output {
    let waker = Waker::from(Arc::new(Unpark(thread::current())));
    let mut context = Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(result) => return result,
            Poll::Pending => thread::park(),
        }
    }
}

fn render_certificate(document: &Document) -> TestResult {
    let before = document.clone();
    let page = document.pages().first().ok_or("page")?;
    let counts = fragment(document)?
        .atoms()
        .iter()
        .map(|a| Ok((a.id, a.explicit_hydrogens.ok_or("literal H")?)))
        .collect::<TestResult<BTreeMap<_, _>>>()?;
    let literal = graph_page_scene(1024, 1024, page)?;
    let resolved = graph_page_scene_with_display_hydrogens(
        1024,
        1024,
        page,
        &counts,
        &Cancellation::default(),
    )?;
    assert_eq!(
        literal.scene(),
        resolved.scene(),
        "actual literal picture equals validated H/paper picture"
    );
    let cpu = cpu_scene(literal.scene())?;
    assert!(cpu.rgba.chunks_exact(4).any(|p| p == [0, 0, 0, 255]));
    assert_eq!(block_on(gpu_scene(literal.scene()))?.0, cpu);
    assert_eq!(document, &before);
    Ok(())
}

#[test]
fn aromatic_and_fixed_kekule_rings_have_literal_graphs_regular_geometry_and_actual_consumers()
-> TestResult {
    for (source, z, h, edges, formula) in [
        (
            "c1ccccc1",
            &[6, 6, 6, 6, 6, 6][..],
            &[1, 1, 1, 1, 1, 1][..],
            BENZENE,
            "C6H6",
        ),
        (
            "C1=CC=CC=C1",
            &[6, 6, 6, 6, 6, 6][..],
            &[1, 1, 1, 1, 1, 1][..],
            BENZENE,
            "C6H6",
        ),
        (
            "n1ccccc1",
            &[7, 6, 6, 6, 6, 6][..],
            &[0, 1, 1, 1, 1, 1][..],
            BENZENE,
            "C5H5N",
        ),
        (
            "c1cc[nH]c1",
            &[6, 6, 6, 7, 6][..],
            &[1, 1, 1, 1, 1][..],
            PYRROLE,
            "C4H5N",
        ),
    ] {
        let owner = parse(source)?;
        let before = owner.clone();
        let document = layout(&owner)?;
        literal_graph(&document, z, h, edges)?;
        let ids = (1..=u32::try_from(z.len())?).collect::<Vec<_>>();
        regular_cycle(fragment(&document)?, &ids)?;
        consume(fragment(&document)?, source, z, h, edges, formula, &[&ids])?;
        render_certificate(&document)?;
        assert_eq!(document, layout(&owner)?);
        retained(&owner, &before);
    }
    assert_eq!(
        layout(&parse("c1ccccc1")?)?,
        layout(&parse("C1=CC=CC=C1")?)?
    );
    Ok(())
}

#[test]
fn disconnected_benzene_and_ethane_keep_the_complete_arrays_and_separated_component_ink()
-> TestResult {
    let owner = parse("c1ccccc1.CC\r\n")?;
    let before = owner.clone();
    let document = layout(&owner)?;
    let mut edges = BENZENE.to_vec();
    edges.push((7, 8, 1));
    literal_graph(&document, &[6; 8], &[1, 1, 1, 1, 1, 1, 3, 3], &edges)?;
    let f = fragment(&document)?;
    regular_cycle(f, &[1, 2, 3, 4, 5, 6])?;
    let mut bounds = Vec::new();
    for ids in [&[1, 2, 3, 4, 5, 6][..], &[7, 8][..]] {
        let points = ids
            .iter()
            .map(|&id| point(f, id))
            .collect::<TestResult<Vec<_>>>()?;
        bounds.push([
            points.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min),
            points.iter().map(|p| p[1]).fold(f64::INFINITY, f64::min),
            points
                .iter()
                .map(|p| p[0])
                .fold(f64::NEG_INFINITY, f64::max),
            points
                .iter()
                .map(|p| p[1])
                .fold(f64::NEG_INFINITY, f64::max),
        ]);
    }
    let (a, b) = (
        bounds.first().ok_or("ring bounds")?,
        bounds.get(1).ok_or("ethane bounds")?,
    );
    assert!(
        b[0] - a[2] >= LENGTH - 1e-8 || b[1] - a[3] >= LENGTH - 1e-8,
        "whole component boxes need a bond-length gap"
    );
    consume(
        f,
        "c1ccccc1.CC",
        &[6; 8],
        &[1, 1, 1, 1, 1, 1, 3, 3],
        &edges,
        "C8H12",
        &[&[1, 2, 3, 4, 5, 6], &[7, 8]],
    )?;
    render_certificate(&document)?;
    retained(&owner, &before);
    Ok(())
}

#[test]
fn supported_old_single_component_documents_remain_exact() -> TestResult {
    for source in ["CCO", "CC(C)O", "C#N", "C=C", "C1CCCCC1"] {
        let owner = parse(source)?;
        let before = owner.clone();
        let old = owner.layout_neutral_components(
            LENGTH,
            Point2::new(ORIGIN, ORIGIN)?,
            PageSize::A4,
            &Cancellation::default(),
        )?;
        assert_eq!(
            layout(&owner)?,
            old,
            "old complete geometry/H/IDs for {source}"
        );
        retained(&owner, &before);
    }
    // The new integer-ring route must not broaden any existing converter's refusal.
    let fixed = parse("C1=CC=CC=C1")?;
    let origin = Point2::new(ORIGIN, ORIGIN)?;
    let cancel = Cancellation::default();
    for result in [
        fixed.layout_carbon_path(LENGTH, origin, PageSize::A4),
        fixed.layout_neutral_acyclic(LENGTH, origin, PageSize::A4, &cancel),
        fixed.layout_neutral_simple_ring(LENGTH, origin, PageSize::A4, &cancel),
        fixed.layout_neutral_components(LENGTH, origin, PageSize::A4, &cancel),
    ] {
        assert!(
            result.is_err(),
            "old converter unexpectedly accepts a Double ring"
        );
    }
    Ok(())
}

#[test]
fn valid_fused_chemistry_and_positive_isotopes_refuse_the_whole_layout_without_partial_output()
-> TestResult {
    for source in ["CC.c1ccc2ccccc2c1", "CC.[13CH4]"] {
        let owner = parse(source)?;
        let before = owner.clone();
        assert!(
            owner
                .normalize_authoring_fragment(&Cancellation::default())
                .is_ok(),
            "chemistry prerequisite for {source}"
        );
        let error = owner
            .layout_authoring(
                LENGTH,
                Point2::new(ORIGIN, ORIGIN)?,
                PageSize::A4,
                &Cancellation::default(),
            )
            .err()
            .ok_or("unsupported later component must refuse whole layout")?;
        assert_eq!(error.code(), "unsupported_content");
        retained(&owner, &before);
    }
    Ok(())
}

#[test]
fn authoring_layout_keeps_cancel_atom_length_and_paper_failures_distinct() -> TestResult {
    let owner = parse("c1ccccc1")?;
    let before = owner.clone();
    let cancelled = Cancellation::default();
    cancelled.cancel();
    assert_eq!(
        owner
            .layout_authoring(
                LENGTH,
                Point2::new(ORIGIN, ORIGIN)?,
                PageSize::A4,
                &cancelled
            )
            .err()
            .ok_or("cancelled layout")?
            .code(),
        "cancelled"
    );
    for length in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(
            owner
                .layout_authoring(
                    length,
                    Point2::new(ORIGIN, ORIGIN)?,
                    PageSize::A4,
                    &Cancellation::default()
                )
                .err()
                .ok_or("invalid length")?
                .code(),
            "invalid_params"
        );
    }
    assert_eq!(
        owner
            .layout_authoring(
                LENGTH,
                Point2::new(10.0, 10.0)?,
                PageSize::new(80.0, 80.0)?,
                &Cancellation::default()
            )
            .err()
            .ok_or("ring cannot fit without scaling")?
            .code(),
        "unsupported_content"
    );
    retained(&owner, &before);
    let large = parse(&vec!["C"; 65].join("."))?;
    let before = large.clone();
    assert_eq!(
        large
            .layout_authoring(
                LENGTH,
                Point2::new(ORIGIN, ORIGIN)?,
                PageSize::A4,
                &Cancellation::default()
            )
            .err()
            .ok_or("whole 65-atom input")?
            .code(),
        "resource_limit"
    );
    retained(&large, &before);
    Ok(())
}
