use bondline_io::smiles::SmilesDocument;
use bondline_model::{BondOrder, PageSize, Point2, Radical};
use bondline_render::Cancellation;
use std::sync::Arc;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn degree_three_branch_keeps_literal_graph_hydrogens_and_source() -> TestResult {
    let source = b"CC(C)C\r\n";
    let owner = SmilesDocument::parse(Arc::from(source.as_slice()))?;
    let before = owner.clone();
    let document = owner.layout_neutral_acyclic(
        48.0,
        Point2::new(200.0, 200.0)?,
        PageSize::A4,
        &Cancellation::default(),
    )?;
    assert_eq!(document.pages().len(), 1);
    let page = document.pages().first().ok_or("page")?;
    assert_eq!(page.size(), PageSize::A4);
    assert_eq!(page.fragments().len(), 1);
    let fragment = page.fragments().first().ok_or("fragment")?;
    assert_eq!(fragment.id().get(), 1);
    assert_eq!(fragment.atoms().len(), 4);
    for (atom, (id, h)) in fragment
        .atoms()
        .iter()
        .zip([(1, 3), (2, 1), (3, 3), (4, 3)])
    {
        assert_eq!(atom.id.get(), id);
        assert_eq!(atom.element.get(), 6);
        assert_eq!(atom.explicit_hydrogens, Some(h));
        assert_eq!(atom.isotope, None);
        assert_eq!(atom.charge, 0);
        assert_eq!(atom.radical, Radical::None);
        assert!(atom.position.is_some());
    }
    assert_eq!(
        fragment
            .bonds()
            .iter()
            .map(|bond| (bond.id.get(), bond.begin.get(), bond.end.get(), bond.order))
            .collect::<Vec<_>>(),
        [
            (1, 1, 2, BondOrder::Single),
            (2, 2, 3, BondOrder::Single),
            (3, 2, 4, BondOrder::Single),
        ]
    );
    assert_eq!(owner, before);
    assert_eq!(owner.as_bytes(), source);
    assert_eq!(owner.as_bytes().as_ptr(), before.as_bytes().as_ptr());
    assert_eq!(owner.atoms().as_ptr(), before.atoms().as_ptr());
    assert_eq!(owner.bonds().as_ptr(), before.bonds().as_ptr());
    Ok(())
}

type ValueResult<T> = Result<T, Box<dyn std::error::Error>>;

use bondline_io::{StoredDocument, cdxml::write_document, smiles::LayoutError};
use bondline_model::{BondOrder::Double as D, BondOrder::Single as S, BondOrder::Triple as T};
use bondline_model::{Document, Fragment, PageId};
use bondline_render::{PageViewport, cpu_scene, graph_page_scene, page_scene};
use serde_json::{Value, json};
use std::{
    io::Write as _,
    process::{Command, Output, Stdio},
};

struct Example {
    source: &'static str,
    atoms: Vec<(u8, u16)>,
    bonds: Vec<(u32, u32, BondOrder)>,
    formula: &'static str,
}

fn examples() -> Vec<Example> {
    vec![
        Example {
            source: "CC(C)C",
            atoms: vec![(6, 3), (6, 1), (6, 3), (6, 3)],
            bonds: vec![(1, 2, S), (2, 3, S), (2, 4, S)],
            formula: "C4H10",
        },
        Example {
            source: "CCO",
            atoms: vec![(6, 3), (6, 2), (8, 1)],
            bonds: vec![(1, 2, S), (2, 3, S)],
            formula: "C2H6O",
        },
        Example {
            source: "CN",
            atoms: vec![(6, 3), (7, 2)],
            bonds: vec![(1, 2, S)],
            formula: "CH5N",
        },
        Example {
            source: "C(F)(Cl)Br",
            atoms: vec![(6, 1), (9, 0), (17, 0), (35, 0)],
            bonds: vec![(1, 2, S), (1, 3, S), (1, 4, S)],
            formula: "CHBrClF",
        },
        Example {
            source: "C[H]",
            atoms: vec![(6, 3), (1, 0)],
            bonds: vec![(1, 2, S)],
            formula: "CH4",
        },
        Example {
            source: "[H]O[H]",
            atoms: vec![(1, 0), (8, 0), (1, 0)],
            bonds: vec![(1, 2, S), (2, 3, S)],
            formula: "H2O",
        },
        Example {
            source: "C=C",
            atoms: vec![(6, 2), (6, 2)],
            bonds: vec![(1, 2, D)],
            formula: "C2H4",
        },
        Example {
            source: "CC=O",
            atoms: vec![(6, 3), (6, 1), (8, 0)],
            bonds: vec![(1, 2, S), (2, 3, D)],
            formula: "C2H4O",
        },
        Example {
            source: "CC#N",
            atoms: vec![(6, 3), (6, 0), (7, 0)],
            bonds: vec![(1, 2, S), (2, 3, T)],
            formula: "C2H3N",
        },
        Example {
            source: "C=C=C",
            atoms: vec![(6, 2), (6, 0), (6, 2)],
            bonds: vec![(1, 2, D), (2, 3, D)],
            formula: "C3H4",
        },
        Example {
            source: "CC(C)=C",
            atoms: vec![(6, 3), (6, 0), (6, 3), (6, 2)],
            bonds: vec![(1, 2, S), (2, 3, S), (2, 4, D)],
            formula: "C4H8",
        },
        Example {
            source: "CC=CC",
            atoms: vec![(6, 3), (6, 1), (6, 1), (6, 3)],
            bonds: vec![(1, 2, S), (2, 3, D), (3, 4, S)],
            formula: "C4H8",
        },
        Example {
            source: "C(C)(C)(C)C",
            atoms: vec![(6, 0), (6, 3), (6, 3), (6, 3), (6, 3)],
            bonds: vec![(1, 2, S), (1, 3, S), (1, 4, S), (1, 5, S)],
            formula: "C5H12",
        },
        Example {
            source: "CCC(C)(C)CC",
            atoms: vec![(6, 3), (6, 2), (6, 0), (6, 3), (6, 3), (6, 2), (6, 3)],
            bonds: vec![
                (1, 2, S),
                (2, 3, S),
                (3, 4, S),
                (3, 5, S),
                (3, 6, S),
                (6, 7, S),
            ],
            formula: "C7H16",
        },
        Example {
            source: "C(F)(Cl)(Br)I",
            atoms: vec![(6, 0), (9, 0), (17, 0), (35, 0), (53, 0)],
            bonds: vec![(1, 2, S), (1, 3, S), (1, 4, S), (1, 5, S)],
            formula: "CBrClFI",
        },
    ]
}

fn parse(source: &str) -> ValueResult<SmilesDocument> {
    Ok(SmilesDocument::parse(Arc::from(source.as_bytes()))?)
}

fn fragment(document: &Document) -> ValueResult<&Fragment> {
    assert_eq!(document.pages().len(), 1);
    let page = document.pages().first().ok_or("page")?;
    assert_eq!(page.id().get(), 1);
    assert_eq!(page.fragments().len(), 1);
    Ok(page.fragments().first().ok_or("fragment")?)
}

fn xy(document: &Document, id: u32) -> ValueResult<[f64; 2]> {
    let point = fragment(document)?
        .atoms()
        .iter()
        .find(|a| a.id.get() == id)
        .ok_or("atom id")?
        .position
        .ok_or("position")?;
    Ok([point.x_pt(), point.y_pt()])
}

fn near(actual: f64, expected: f64, epsilon: f64) {
    assert!(
        (actual - expected).abs() <= epsilon,
        "{actual} != {expected} within {epsilon}"
    );
}

fn same_owner(owner: &SmilesDocument, before: &SmilesDocument) {
    assert_eq!(owner, before);
    assert_eq!(owner.as_bytes(), before.as_bytes());
    assert_eq!(owner.notation_range(), before.notation_range());
    assert_eq!(owner.chain_starts(), before.chain_starts());
    assert_eq!(owner.atoms(), before.atoms());
    assert_eq!(owner.bonds(), before.bonds());
    assert_eq!(owner.as_bytes().as_ptr(), before.as_bytes().as_ptr());
    assert_eq!(owner.atoms().as_ptr(), before.atoms().as_ptr());
    assert_eq!(owner.bonds().as_ptr(), before.bonds().as_ptr());
}

fn literal_graph(document: &Document, expected: &Example) -> TestResult {
    let f = fragment(document)?;
    assert_eq!(f.id().get(), 1);
    assert_eq!(f.atoms().len(), expected.atoms.len(), "{}", expected.source);
    for (index, (atom, &(element, h))) in f.atoms().iter().zip(&expected.atoms).enumerate() {
        assert_eq!(atom.id.get(), u32::try_from(index + 1)?);
        assert_eq!(atom.element.get(), element);
        assert_eq!(atom.explicit_hydrogens, Some(h));
        assert_eq!(atom.isotope, None);
        assert_eq!(atom.charge, 0);
        assert_eq!(atom.radical, Radical::None);
        let p = atom.position.ok_or("position")?;
        assert!(p.x_pt().is_finite() && p.y_pt().is_finite());
    }
    assert_eq!(
        f.bonds()
            .iter()
            .map(|b| (b.begin.get(), b.end.get(), b.order))
            .collect::<Vec<_>>(),
        expected.bonds
    );
    assert_eq!(
        f.bonds().iter().map(|b| b.id.get()).collect::<Vec<_>>(),
        (1..=u32::try_from(expected.bonds.len())?).collect::<Vec<_>>()
    );
    Ok(())
}

fn geometry_and_real_paper_ink(document: &Document, length: f64, origin: [f64; 2]) -> TestResult {
    let f = fragment(document)?;
    let points = f
        .atoms()
        .iter()
        .map(|a| a.position.ok_or("position"))
        .collect::<Result<Vec<_>, _>>()?;
    near(
        points
            .iter()
            .map(|p| p.x_pt())
            .fold(f64::INFINITY, f64::min),
        origin[0],
        1e-9,
    );
    near(
        points
            .iter()
            .map(|p| p.y_pt())
            .fold(f64::INFINITY, f64::min),
        origin[1],
        1e-9,
    );
    for bond in f.bonds() {
        let a = xy(document, bond.begin.get())?;
        let b = xy(document, bond.end.get())?;
        near(
            (a[0] - b[0]).hypot(a[1] - b[1]),
            length,
            length.max(1.0) * 1e-9,
        );
    }
    let page = document.pages().first().ok_or("page")?;
    let scene = graph_page_scene(1024, 1024, page)?;
    let actual = cpu_scene(scene.scene())?;
    let background = cpu_scene(&page_scene(1024, 1024, page.size())?)?;
    let viewport = PageViewport::new(1024, 1024, page.size())?;
    let paper = viewport.paper();
    let mut ink = 0;
    for (index, (actual, base)) in actual
        .rgba
        .chunks_exact(4)
        .zip(background.rgba.chunks_exact(4))
        .enumerate()
    {
        let x = u32::try_from(index % 1024)?;
        let y = u32::try_from(index / 1024)?;
        if x <= paper.x
            || x >= paper.x + paper.width - 1
            || y <= paper.y
            || y >= paper.y + paper.height - 1
        {
            assert_eq!(
                actual, base,
                "complete glyph/stroke/mask must stay on physical paper at {x},{y}"
            );
        } else if actual == [0, 0, 0, 255] {
            ink += 1;
        }
    }
    assert!(ink > 0);
    Ok(())
}

fn angle_cos(document: &Document, a: u32, center: u32, b: u32) -> ValueResult<f64> {
    let a = xy(document, a)?;
    let c = xy(document, center)?;
    let b = xy(document, b)?;
    let u = [a[0] - c[0], a[1] - c[1]];
    let v = [b[0] - c[0], b[1] - c[1]];
    Ok((u[0] * v[0] + u[1] * v[1]) / (u[0].hypot(u[1]) * v[0].hypot(v[1])))
}

fn cyclic_angle_gaps(document: &Document, center: u32, neighbors: &[u32]) -> ValueResult<Vec<f64>> {
    let c = xy(document, center)?;
    let mut angles = neighbors
        .iter()
        .map(|id| {
            let p = xy(document, *id)?;
            Ok((p[1] - c[1])
                .atan2(p[0] - c[0])
                .to_degrees()
                .rem_euclid(360.0))
        })
        .collect::<ValueResult<Vec<_>>>()?;
    angles.sort_by(f64::total_cmp);
    let mut gaps = angles.windows(2).map(|w| w[1] - w[0]).collect::<Vec<_>>();
    gaps.push(360.0 + angles.first().ok_or("angle")? - angles.last().ok_or("angle")?);
    gaps.sort_by(f64::total_cmp);
    Ok(gaps)
}

fn matches_angles(actual: &[f64], expected: &[f64]) -> bool {
    actual.len() == expected.len()
        && actual
            .iter()
            .zip(expected)
            .all(|(a, b)| (a - b).abs() < 1e-8)
}

#[test]
fn heteroatoms_physical_h_and_multiple_bonds_have_independent_literal_graphs() -> TestResult {
    for expected in examples() {
        let owner = parse(expected.source)?;
        let before = owner.clone();
        let page = PageSize::new(612.0, 792.0)?;
        let document = owner.layout_neutral_acyclic(
            64.0,
            Point2::new(144.0, 144.0)?,
            page,
            &Cancellation::default(),
        )?;
        assert_eq!(document.pages()[0].size(), page);
        literal_graph(&document, &expected)?;
        geometry_and_real_paper_ink(&document, 64.0, [144.0, 144.0])?;
        same_owner(&owner, &before);
    }
    Ok(())
}

#[test]
fn branch_angles_and_contextual_degree_four_patterns_are_analytic() -> TestResult {
    let branch = parse("CC(C)C")?.layout_neutral_acyclic(
        64.0,
        Point2::new(144.0, 144.0)?,
        PageSize::A4,
        &Cancellation::default(),
    )?;
    for (a, b) in [(1, 3), (1, 4), (3, 4)] {
        near(angle_cos(&branch, a, 2, b)?, -0.5, 1e-10);
    }
    let simple = parse("C(C)(C)(C)C")?.layout_neutral_acyclic(
        64.0,
        Point2::new(144.0, 144.0)?,
        PageSize::A4,
        &Cancellation::default(),
    )?;
    assert!(matches_angles(
        &cyclic_angle_gaps(&simple, 1, &[2, 3, 4, 5])?,
        &[90.0, 90.0, 90.0, 90.0]
    ));
    for (source, center, neighbors) in [
        ("CCC(C)(C)CC", 3, [2, 4, 5, 6]),
        ("C(F)(Cl)(Br)I", 1, [2, 3, 4, 5]),
    ] {
        let document = parse(source)?.layout_neutral_acyclic(
            64.0,
            Point2::new(144.0, 144.0)?,
            PageSize::A4,
            &Cancellation::default(),
        )?;
        let gaps = cyclic_angle_gaps(&document, center, &neighbors)?;
        assert!(
            matches_angles(&gaps, &[60.0, 90.0, 90.0, 120.0])
                || matches_angles(&gaps, &[60.0, 60.0, 120.0, 120.0]),
            "{source}: {gaps:?}"
        );
    }
    Ok(())
}

#[test]
fn triple_allene_and_unspecified_internal_double_have_collinear_centers() -> TestResult {
    for (source, angles) in [
        ("CC#N", vec![(1, 2, 3)]),
        ("C=C=C", vec![(1, 2, 3)]),
        ("CC=CC", vec![(1, 2, 3), (2, 3, 4)]),
    ] {
        let document = parse(source)?.layout_neutral_acyclic(
            64.0,
            Point2::new(144.0, 144.0)?,
            PageSize::A4,
            &Cancellation::default(),
        )?;
        for (a, c, b) in angles {
            near(angle_cos(&document, a, c, b)?, -1.0, 1e-10);
        }
    }
    let ordinary = parse("CC=O")?.layout_neutral_acyclic(
        64.0,
        Point2::new(144.0, 144.0)?,
        PageSize::A4,
        &Cancellation::default(),
    )?;
    near(angle_cos(&ordinary, 1, 2, 3)?, -0.5, 1e-10);
    for source in ["CCC(C)=CC", "FC(Cl)=C(F)Cl", "CC=C(C)CC"] {
        let owner = parse(source)?;
        let before = owner.clone();
        assert_eq!(
            owner
                .layout_neutral_acyclic(
                    64.0,
                    Point2::new(144.0, 144.0)?,
                    PageSize::A4,
                    &Cancellation::default()
                )
                .err()
                .ok_or("invented stereogenic Double")?
                .code(),
            "unsupported_content"
        );
        same_owner(&owner, &before);
    }
    Ok(())
}

#[test]
fn origin_is_center_bounds_translation_is_rigid_and_topology_not_tokens_is_authoritative()
-> TestResult {
    let a = Point2::new(100.0, 200.0)?;
    let b = Point2::new(150.0, 300.0)?;
    for source in ["C", "CCO", "CC(C)C", "C(C)(C)(C)C"] {
        let owner = parse(source)?;
        let before = owner.clone();
        let one = owner.layout_neutral_acyclic(48.0, a, PageSize::A4, &Cancellation::default())?;
        let two = owner.layout_neutral_acyclic(48.0, b, PageSize::A4, &Cancellation::default())?;
        assert_eq!(
            one,
            owner.layout_neutral_acyclic(48.0, a, PageSize::A4, &Cancellation::default())?
        );
        for atom in fragment(&one)?.atoms() {
            let p = xy(&one, atom.id.get())?;
            let q = xy(&two, atom.id.get())?;
            near(q[0] - p[0], 50.0, 1e-10);
            near(q[1] - p[1], 100.0, 1e-10);
        }
        geometry_and_real_paper_ink(&one, 48.0, [100.0, 200.0])?;
        geometry_and_real_paper_ink(&two, 48.0, [150.0, 300.0])?;
        same_owner(&owner, &before);
    }
    let layout = |s: &str| -> ValueResult<Document> {
        Ok(parse(s)?.layout_neutral_acyclic(48.0, a, PageSize::A4, &Cancellation::default())?)
    };
    assert_eq!(layout("CC")?, layout("C1.C1")?);
    assert_eq!(layout("CCO")?, layout("[CH3][CH2][OH]\r\n")?);
    assert_eq!(layout("CC(C)C")?, layout("[CH3][CH]([CH3])[CH3]")?);
    assert_eq!(xy(&layout("C")?, 1)?, [100.0, 200.0]);
    Ok(())
}

#[test]
fn cycles_components_aromatic_and_unsupported_metadata_refuse_without_source_loss() -> TestResult {
    for source in [
        "C1CC1",
        "C1CCCCC1",
        "C1CCC2(CC1)CCCC2",
        "C1CC2CCC1C2",
        "C.O",
        "CC.CN",
        "c1ccccc1",
        "C:C",
        "[13CH3]C",
        "[0CH3]C",
        "[NH4+]",
        "[Cl-]",
        "[CH3]",
        "[O]",
        "B",
        "P",
        "S",
        "[Fe]",
    ] {
        let owner = parse(source)?;
        let before = owner.clone();
        assert_eq!(
            owner
                .layout_neutral_acyclic(
                    48.0,
                    Point2::new(144.0, 144.0)?,
                    PageSize::A4,
                    &Cancellation::default()
                )
                .err()
                .ok_or_else(|| format!("accepted {source}"))?
                .code(),
            "unsupported_content",
            "{source}"
        );
        same_owner(&owner, &before);
    }
    for source in [
        "[CH3:4]",
        "[C@H](F)(Cl)Br",
        "C/C=C/C",
        "C~C",
        "CC name",
        "CC\nCO\n",
    ] {
        assert!(
            SmilesDocument::parse(Arc::from(source.as_bytes())).is_err(),
            "unchanged parser accepted {source}"
        );
    }
    Ok(())
}

#[test]
fn finite_geometry_cap_before_normalization_and_typed_cancellation_are_bounded() -> TestResult {
    let owner = parse("CC")?;
    let before = owner.clone();
    let origin = Point2::new(144.0, 144.0)?;
    for length in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let error = owner
            .layout_neutral_acyclic(length, origin, PageSize::A4, &Cancellation::default())
            .err()
            .ok_or("invalid length")?;
        assert!(matches!(error, LayoutError::InvalidBondLength));
        assert_eq!(error.code(), "invalid_params");
    }
    for source in ["C".repeat(65), format!("P{}", "C".repeat(64))] {
        let large = parse(&source)?;
        let old = large.clone();
        let error = large
            .layout_neutral_acyclic(48.0, origin, PageSize::A4, &Cancellation::default())
            .err()
            .ok_or("over cap")?;
        assert!(matches!(error, LayoutError::AcyclicAtomLimit));
        assert_eq!(error.code(), "resource_limit");
        same_owner(&large, &old);
    }
    let atcap = parse(&"C".repeat(64))?;
    let old = atcap.clone();
    match atcap.layout_neutral_acyclic(48.0, origin, PageSize::A4, &Cancellation::default()) {
        Ok(document) => {
            assert_eq!(fragment(&document)?.atoms().len(), 64);
            geometry_and_real_paper_ink(&document, 48.0, [144.0, 144.0])?;
        }
        Err(LayoutError::Geometry(_) | LayoutError::Render(_)) => {}
        Err(error) => {
            return Err(
                format!("valid at-cap chain had a non-geometry/render refusal: {error}").into(),
            );
        }
    }
    same_owner(&atcap, &old);
    let cancel = Cancellation::default();
    cancel.cancel();
    let error = owner
        .layout_neutral_acyclic(48.0, origin, PageSize::A4, &cancel)
        .err()
        .ok_or("cancelled succeeded")?;
    assert!(matches!(error, LayoutError::Cancelled));
    assert_eq!(error.code(), "cancelled");
    for (length, origin, page) in [
        (1e-300, origin, PageSize::A4),
        (f64::MAX, origin, PageSize::A4),
        (48.0, Point2::new(-1.0, 144.0)?, PageSize::A4),
        (48.0, Point2::new(100_000.0, 144.0)?, PageSize::A4),
        (48.0, origin, PageSize::new(1.0, 100_000.0)?),
    ] {
        assert!(
            owner
                .layout_neutral_acyclic(length, origin, page, &Cancellation::default())
                .is_err()
        );
    }
    for point in [(1.0, 200.0), (590.0, 200.0), (200.0, 1.0), (200.0, 839.0)] {
        assert!(
            parse("C")?
                .layout_neutral_acyclic(
                    48.0,
                    Point2::new(point.0, point.1)?,
                    PageSize::A4,
                    &Cancellation::default()
                )
                .is_err(),
            "atom center inside paper must not permit clipped label"
        );
    }
    same_owner(&owner, &before);
    Ok(())
}

fn external(script: &str, raw: &[u8], arguments: &[String]) -> ValueResult<Output> {
    let python = std::env::var_os("BONDLINE_PYTHON")
        .ok_or("BONDLINE_PYTHON must select the pinned external interpreter")?;
    let mut child = Command::new(python)
        .arg("-B")
        .arg(format!("{}/{}", env!("CARGO_MANIFEST_DIR"), script))
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("stdin")?.write_all(raw)?;
    Ok(child.wait_with_output()?)
}

#[test]
fn actual_native_cdxml_has_independent_raw_chemical_and_editable_reopen_evidence() -> TestResult {
    for expected in examples() {
        let owner = parse(expected.source)?;
        let before = owner.clone();
        let document = owner.layout_neutral_acyclic(
            64.0,
            Point2::new(144.0, 144.0)?,
            PageSize::new(612.0, 792.0)?,
            &Cancellation::default(),
        )?;
        literal_graph(&document, &expected)?;
        let raw = write_document(&document)?;
        let options = json!({"atoms":expected.atoms.iter().map(|(z,h)|json!([z,h])).collect::<Vec<_>>(),"bonds":expected.bonds.iter().map(|(a,b,o)|json!([a,b,match o {S=>1,D=>2,T=>3,_=>0}])).collect::<Vec<_>>(),"formula":expected.formula,"length":"64","origin":["144","144"],"page":["612","792"]});
        let output = external(
            "tests/smiles_acyclic_oracle.py",
            &raw,
            &[expected.source.into(), options.to_string()],
        )?;
        assert!(
            output.status.success(),
            "{}: {}",
            expected.source,
            String::from_utf8_lossy(&output.stderr)
        );
        let metadata: Value = serde_json::from_slice(&output.stdout)?;
        assert_eq!(metadata["formula"], expected.formula);
        assert_eq!(metadata["no_assigned_stereo"], true);
        if expected.source == "CC=CC" {
            assert_eq!(metadata["stereo_none"], false);
            assert_eq!(
                metadata["bond_stereo"],
                json!(["STEREONONE", "STEREOANY", "STEREONONE"])
            );
        } else {
            assert_eq!(metadata["stereo_none"], true);
            assert_eq!(
                metadata["bond_stereo"],
                json!(vec!["STEREONONE"; expected.bonds.len()])
            );
        }
        let projection = external("../../xtask/oracles/cdxml_graph.py", &raw, &[])?;
        assert!(projection.status.success());
        let projected: Value = serde_json::from_slice(&projection.stdout)?;
        let atoms = projected[0]["atoms"]
            .as_array()
            .ok_or("independent atoms")?;
        assert_eq!(atoms.len(), expected.atoms.len());
        for (atom, (z, h)) in atoms.iter().zip(&expected.atoms) {
            assert_eq!(atom["element"], *z);
            assert_eq!(atom["explicit_hydrogens"], *h);
        }
        let mut reopened = StoredDocument::from_cdxml(Arc::from(raw.as_slice()), PageSize::A4)?;
        assert!(reopened.is_fully_mapped());
        assert_eq!(reopened.model(), &document);
        assert_eq!(reopened.to_cdxml()?.as_ref(), raw.as_slice());
        let old = reopened.clone();
        let f = fragment(reopened.model())?;
        let mut atoms = f.atoms().to_vec();
        let atom = atoms.first_mut().ok_or("editable atom")?;
        let p = atom.position.ok_or("editable position")?;
        atom.position = Some(Point2::new(p.x_pt() + 1.0, p.y_pt())?);
        let replacement = Fragment::new(f.id(), atoms, f.bonds().to_vec())?;
        reopened
            .model_mut()
            .replace_fragment(PageId::new(1).ok_or("page id")?, replacement)?;
        let changed = reopened.to_cdxml()?.into_owned();
        assert_ne!(changed, raw);
        assert_eq!(old.to_cdxml()?.as_ref(), raw.as_slice());
        assert!(reopened.source_info().ok_or("source")?.modified);
        let changed_reopened = StoredDocument::from_cdxml(changed.into(), PageSize::A4)?;
        assert!(changed_reopened.is_fully_mapped());
        assert_eq!(changed_reopened.model(), reopened.model());
        if expected.source == "CCO" {
            let mut wrong = options.clone();
            wrong["atoms"][0][1] = json!(4);
            assert!(
                !external(
                    "tests/smiles_acyclic_oracle.py",
                    &raw,
                    &[expected.source.into(), wrong.to_string()]
                )?
                .status
                .success(),
                "negative literal-H control must fail"
            );
            let mut wrong = options.clone();
            wrong["length"] = json!("63");
            assert!(
                !external(
                    "tests/smiles_acyclic_oracle.py",
                    &raw,
                    &[expected.source.into(), wrong.to_string()]
                )?
                .status
                .success(),
                "negative raw-coordinate control must fail"
            );
        }
        same_owner(&owner, &before);
    }
    Ok(())
}
