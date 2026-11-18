use bondline_io::smiles::SmilesDocument;
use bondline_model::{PageSize, Point2, Radical};
use bondline_render::Cancellation;
use std::sync::Arc;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn isolated_components_share_one_fragment_and_exact_center_gap() -> TestResult {
    let source = b"C.O\r\n";
    let owner = SmilesDocument::parse(Arc::from(source.as_slice()))?;
    let before = owner.clone();
    let page_size = PageSize::new(1000.0, 1000.0)?;
    let document = owner.layout_neutral_components(
        64.0,
        Point2::new(144.0, 144.0)?,
        page_size,
        &Cancellation::default(),
    )?;
    assert_eq!(document.pages().len(), 1);
    let page = document.pages().first().ok_or("page")?;
    assert_eq!(page.id().get(), 1);
    assert_eq!(page.size(), page_size);
    assert_eq!(page.fragments().len(), 1);
    let fragment = page.fragments().first().ok_or("complete fragment")?;
    assert_eq!(fragment.id().get(), 1);
    assert_eq!(fragment.atoms().len(), 2);
    assert!(fragment.bonds().is_empty());
    for (atom, (id, element, h, position)) in fragment
        .atoms()
        .iter()
        .zip([(1, 6, 4, [144.0, 144.0]), (2, 8, 2, [208.0, 144.0])])
    {
        assert_eq!(atom.id.get(), id);
        assert_eq!(atom.element.get(), element);
        assert_eq!(atom.explicit_hydrogens, Some(h));
        assert_eq!(atom.position, Some(Point2::new(position[0], position[1])?));
        assert_eq!(atom.charge, 0);
        assert_eq!(atom.isotope, None);
        assert_eq!(atom.radical, Radical::None);
    }
    assert_eq!(owner, before);
    assert_eq!(owner.as_bytes(), source);
    assert_eq!(owner.as_bytes().as_ptr(), before.as_bytes().as_ptr());
    assert_eq!(owner.atoms().as_ptr(), before.atoms().as_ptr());
    assert_eq!(owner.bonds().as_ptr(), before.bonds().as_ptr());
    Ok(())
}

use bondline_io::{StoredDocument, cdxml::write_document, smiles::LayoutError};
use bondline_model::BondOrder::{Double as D, Single as S, Triple as T};
use bondline_model::{BondOrder, Document, Fragment};
use bondline_render::{cpu_scene, graph_page_scene, graph_page_scene_with_display_hydrogens};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::Write as _,
    process::{Command, Output, Stdio},
};

struct Example {
    source: &'static str,
    atoms: Vec<(u8, u16)>,
    bonds: Vec<(u32, u32, BondOrder)>,
    components: Vec<Vec<u32>>,
    formula: &'static str,
}

fn examples() -> Vec<Example> {
    vec![
        Example {
            source: "C1.O.C1",
            atoms: vec![(6, 3), (8, 2), (6, 3)],
            bonds: vec![(1, 3, S)],
            components: vec![vec![1, 3], vec![2]],
            formula: "C2H8O",
        },
        Example {
            source: "CC.O.C1CCCCC1",
            atoms: vec![
                (6, 3),
                (6, 3),
                (8, 2),
                (6, 2),
                (6, 2),
                (6, 2),
                (6, 2),
                (6, 2),
                (6, 2),
            ],
            bonds: vec![
                (1, 2, S),
                (4, 5, S),
                (5, 6, S),
                (6, 7, S),
                (7, 8, S),
                (8, 9, S),
                (4, 9, S),
            ],
            components: vec![vec![1, 2], vec![3], vec![4, 5, 6, 7, 8, 9]],
            formula: "C8H20O",
        },
        Example {
            source: "C[H].C=O.C#N",
            atoms: vec![(6, 3), (1, 0), (6, 2), (8, 0), (6, 1), (7, 0)],
            bonds: vec![(1, 2, S), (3, 4, D), (5, 6, T)],
            components: vec![vec![1, 2], vec![3, 4], vec![5, 6]],
            formula: "C3H7NO",
        },
        Example {
            source: "CC(C)O.F",
            atoms: vec![(6, 3), (6, 1), (6, 3), (8, 1), (9, 1)],
            bonds: vec![(1, 2, S), (2, 3, S), (2, 4, S)],
            components: vec![vec![1, 2, 3, 4], vec![5]],
            formula: "C3H9FO",
        },
        Example {
            source: "CC=CC.O",
            atoms: vec![(6, 3), (6, 1), (6, 1), (6, 3), (8, 2)],
            bonds: vec![(1, 2, S), (2, 3, D), (3, 4, S)],
            components: vec![vec![1, 2, 3, 4], vec![5]],
            formula: "C4H10O",
        },
        Example {
            source: "C1CC1.N1CC1",
            atoms: vec![(6, 2), (6, 2), (6, 2), (7, 1), (6, 2), (6, 2)],
            bonds: vec![
                (1, 2, S),
                (2, 3, S),
                (1, 3, S),
                (4, 5, S),
                (5, 6, S),
                (4, 6, S),
            ],
            components: vec![vec![1, 2, 3], vec![4, 5, 6]],
            formula: "C5H11N",
        },
    ]
}

fn parse(source: &str) -> TestResult<SmilesDocument> {
    Ok(SmilesDocument::parse(Arc::from(source.as_bytes()))?)
}

fn layout(owner: &SmilesDocument, page: PageSize) -> TestResult<Document> {
    Ok(owner.layout_neutral_components(
        64.0,
        Point2::new(144.0, 144.0)?,
        page,
        &Cancellation::default(),
    )?)
}

fn fragment(document: &Document) -> TestResult<&Fragment> {
    assert_eq!(document.pages().len(), 1);
    let page = document.pages().first().ok_or("page")?;
    assert_eq!(page.id().get(), 1);
    assert_eq!(page.fragments().len(), 1);
    Ok(page
        .fragments()
        .first()
        .ok_or("complete fragment")?
        .as_ref())
}

fn xy(document: &Document, id: u32) -> TestResult<[f64; 2]> {
    let point = fragment(document)?
        .atoms()
        .iter()
        .find(|a| a.id.get() == id)
        .ok_or("atom")?
        .position
        .ok_or("position")?;
    Ok([point.x_pt(), point.y_pt()])
}

fn near(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 64.0e-9,
        "{actual} != {expected}"
    );
}

fn bounds(document: &Document, ids: &[u32]) -> TestResult<[f64; 4]> {
    let mut result = [
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    ];
    for &id in ids {
        let p = xy(document, id)?;
        result[0] = result[0].min(p[0]);
        result[1] = result[1].min(p[1]);
        result[2] = result[2].max(p[0]);
        result[3] = result[3].max(p[1]);
    }
    Ok(result)
}

fn same_source(owner: &SmilesDocument, before: &SmilesDocument) {
    assert_eq!(owner, before);
    assert_eq!(owner.as_bytes().as_ptr(), before.as_bytes().as_ptr());
    assert_eq!(owner.atoms().as_ptr(), before.atoms().as_ptr());
    assert_eq!(owner.bonds().as_ptr(), before.bonds().as_ptr());
    assert_eq!(owner.chain_starts(), before.chain_starts());
}

fn literal_graph(document: &Document, owner: &SmilesDocument, e: &Example) -> TestResult {
    let f = fragment(document)?;
    let normalized = owner.normalize_explicit_h_fragment()?;
    assert_eq!(f.id().get(), 1);
    assert_eq!(f.atoms().len(), e.atoms.len());
    for (index, ((actual, original), &(z, h))) in f
        .atoms()
        .iter()
        .zip(normalized.atoms())
        .zip(&e.atoms)
        .enumerate()
    {
        assert_eq!(actual.id.get(), u32::try_from(index + 1)?);
        assert_eq!(actual.element.get(), z);
        assert_eq!(actual.explicit_hydrogens, Some(h));
        assert_eq!(actual.isotope, None);
        assert_eq!(actual.charge, 0);
        assert_eq!(actual.radical, Radical::None);
        assert!(actual.position.is_some());
        let mut without_position = actual.clone();
        without_position.position = None;
        assert_eq!(
            &without_position, original,
            "positions are the only changed normalized field"
        );
    }
    assert_eq!(f.bonds(), normalized.bonds());
    assert_eq!(
        f.bonds()
            .iter()
            .map(|b| (b.begin.get(), b.end.get(), b.order))
            .collect::<Vec<_>>(),
        e.bonds
    );
    assert_eq!(
        f.bonds().iter().map(|b| b.id.get()).collect::<Vec<_>>(),
        (1..=u32::try_from(e.bonds.len())?).collect::<Vec<_>>()
    );
    for bond in f.bonds() {
        let a = xy(document, bond.begin.get())?;
        let b = xy(document, bond.end.get())?;
        near((a[0] - b[0]).hypot(a[1] - b[1]), 64.0);
    }
    Ok(())
}

fn scene_certificate(document: &Document) -> TestResult {
    let page = document.pages().first().ok_or("page")?;
    let before = document.clone();
    let counts = fragment(document)?
        .atoms()
        .iter()
        .map(|a| Ok((a.id, a.explicit_hydrogens.ok_or("fixed H")?)))
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
        "ordinary drawing equals full attachment-safe/paper-checked scene"
    );
    let image = cpu_scene(literal.scene())?;
    assert_eq!(image.rgba.len(), 1024 * 1024 * 4);
    assert!(image.rgba.chunks_exact(4).any(|p| p == [0, 0, 0, 255]));
    assert_eq!(*document, before);
    Ok(())
}

fn external(raw: &[u8], source: &str, options: &Value) -> TestResult<Output> {
    let python = std::env::var_os("BONDLINE_PYTHON").ok_or("pinned external interpreter")?;
    let mut child = Command::new(python)
        .arg("-B")
        .arg(format!(
            "{}/tests/smiles_components_oracle.py",
            env!("CARGO_MANIFEST_DIR")
        ))
        .arg(source)
        .arg(options.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("stdin")?.write_all(raw)?;
    Ok(child.wait_with_output()?)
}

fn options(e: &Example, page: PageSize) -> Value {
    json!({"atoms":e.atoms.iter().map(|&(z,h)|json!([z,h])).collect::<Vec<_>>(),
        "bonds":e.bonds.iter().map(|&(a,b,o)|json!([a,b,match o {S=>1,D=>2,T=>3,_=>0}])).collect::<Vec<_>>(),
        "components":e.components,"formula":e.formula,"length":"64","origin":["144","144"],
        "page":[page.width_pt().to_string(),page.height_pt().to_string()]})
}

#[test]
fn interleaved_ids_are_components_by_actual_edges_not_chain_starts() -> TestResult {
    let e = examples().remove(0);
    let owner = parse(e.source)?;
    let before = owner.clone();
    assert_eq!(
        owner.chain_starts(),
        3,
        "source has three textual chain starts"
    );
    let document = layout(&owner, PageSize::new(1000.0, 1000.0)?)?;
    literal_graph(&document, &owner, &e)?;
    let pair = bounds(&document, &[1, 3])?;
    let water = bounds(&document, &[2])?;
    near(pair[0], 144.0);
    near(pair[1], 144.0);
    near(pair[2] - pair[0], 64.0);
    near(water[0], pair[2] + 64.0);
    near(water[1], 144.0);
    scene_certificate(&document)?;
    same_source(&owner, &before);
    Ok(())
}

#[test]
fn one_actual_component_is_exactly_the_existing_tree_or_ring_document() -> TestResult {
    for source in ["C1.C1", "CC(C)O", "CC#N", "C1CCCCC1", "CC1CCCCC1"] {
        let owner = parse(source)?;
        let before = owner.clone();
        let page = PageSize::new(1000.0, 1000.0)?;
        let origin = Point2::new(144.0, 144.0)?;
        let expected = if source.ends_with("CCCCC1") {
            owner.layout_neutral_simple_ring(64.0, origin, page, &Cancellation::default())?
        } else {
            owner.layout_neutral_acyclic(64.0, origin, page, &Cancellation::default())?
        };
        let actual = layout(&owner, page)?;
        assert_eq!(
            actual, expected,
            "{source}: existing pose/IDs/page must stay exact"
        );
        assert_eq!(write_document(&actual)?, write_document(&expected)?);
        same_source(&owner, &before);
    }
    Ok(())
}

#[test]
fn literal_mixed_graphs_retain_all_arrays_hydrogens_and_rigid_shelf_gaps() -> TestResult {
    for e in examples() {
        let owner = parse(e.source)?;
        let before = owner.clone();
        let document = layout(&owner, PageSize::new(1000.0, 1000.0)?)?;
        literal_graph(&document, &owner, &e)?;
        if e.source == "CC(C)O.F" {
            let center = xy(&document, 2)?;
            let mut rays = Vec::new();
            for id in [1, 3, 4] {
                let point = xy(&document, id)?;
                rays.push([point[0] - center[0], point[1] - center[1]]);
            }
            for (index, a) in rays.iter().enumerate() {
                for b in rays.iter().skip(index + 1) {
                    let cosine = (a[0] * b[0] + a[1] * b[1]) / (64.0 * 64.0);
                    assert!(
                        (cosine + 0.5).abs() <= 1e-9,
                        "three source-template 120-degree arms"
                    );
                }
            }
        }
        let mut next_x = 144.0;
        for component in &e.components {
            let b = bounds(&document, component)?;
            near(b[0], next_x);
            near(b[1], 144.0);
            next_x = b[2] + 64.0;
        }
        assert_eq!(document, layout(&owner, PageSize::new(1000.0, 1000.0)?)?);
        scene_certificate(&document)?;
        same_source(&owner, &before);
    }
    Ok(())
}

#[test]
fn shelves_wrap_after_actual_tall_component_and_keep_later_zero_width_component() -> TestResult {
    let owner = parse("C1CCCCC1.CC.O")?;
    let before = owner.clone();
    let page = PageSize::new(360.0, 650.0)?;
    let document = layout(&owner, page)?;
    let ring = bounds(&document, &[1, 2, 3, 4, 5, 6])?;
    let pair = bounds(&document, &[7, 8])?;
    let water = bounds(&document, &[9])?;
    near(ring[0], 144.0);
    near(ring[1], 144.0);
    near(ring[2] - ring[0], 64.0 * 3.0_f64.sqrt());
    near(ring[3] - ring[1], 128.0);
    assert!(ring[2] + 64.0 + 64.0 > page.width_pt());
    near(pair[0], 144.0);
    near(pair[1], ring[3] + 64.0);
    near(water[0], pair[2] + 64.0);
    near(water[1], pair[1]);
    scene_certificate(&document)?;
    let e = Example {
        source: "C1CCCCC1.CC.O",
        atoms: vec![
            (6, 2),
            (6, 2),
            (6, 2),
            (6, 2),
            (6, 2),
            (6, 2),
            (6, 3),
            (6, 3),
            (8, 2),
        ],
        bonds: vec![
            (1, 2, S),
            (2, 3, S),
            (3, 4, S),
            (4, 5, S),
            (5, 6, S),
            (1, 6, S),
            (7, 8, S),
        ],
        components: vec![vec![1, 2, 3, 4, 5, 6], vec![7, 8], vec![9]],
        formula: "C8H20O",
    };
    literal_graph(&document, &owner, &e)?;
    let result = external(&write_document(&document)?, e.source, &options(&e, page))?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    same_source(&owner, &before);
    Ok(())
}

#[test]
fn actual_cdxml_faces_independent_raw_geometry_and_chemistry_then_reopens_complete() -> TestResult {
    for e in examples() {
        let owner = parse(e.source)?;
        let before = owner.clone();
        let page = PageSize::new(1000.0, 1000.0)?;
        let document = layout(&owner, page)?;
        let raw = write_document(&document)?;
        let expected = options(&e, page);
        let result = external(&raw, e.source, &expected)?;
        assert!(
            result.status.success(),
            "{}: {}",
            e.source,
            String::from_utf8_lossy(&result.stderr)
        );
        let actual: Value = serde_json::from_slice(&result.stdout)?;
        assert_eq!(actual["atoms"], e.atoms.len());
        assert_eq!(actual["bonds"], e.bonds.len());
        assert_eq!(actual["components"], e.components.len());
        assert_eq!(actual["formula"], e.formula);
        assert_eq!(actual["no_assigned_stereo"], true);
        assert_eq!(
            actual["raw_graph"],
            json!({"atoms":expected["atoms"],"bonds":expected["bonds"],"components":expected["components"]})
        );
        let stereo = if e.source == "CC=CC.O" {
            vec!["STEREONONE", "STEREOANY", "STEREONONE"]
        } else {
            vec!["STEREONONE"; e.bonds.len()]
        };
        assert_eq!(actual["bond_stereo"], json!(stereo));
        assert_eq!(actual["stereo_none"], e.source != "CC=CC.O");
        assert_eq!(
            actual["physical_h"],
            e.atoms.iter().filter(|&&(z, _)| z == 1).count()
        );
        let reopened = StoredDocument::from_cdxml(Arc::from(raw.as_slice()), PageSize::A4)?;
        assert!(reopened.is_fully_mapped());
        assert_eq!(reopened.model(), &document);
        assert_eq!(reopened.to_cdxml()?.as_ref(), raw.as_slice());
        same_source(&owner, &before);
        if e.source == "CC.O.C1CCCCC1" {
            for key in ["atoms", "bonds", "components", "length"] {
                let mut wrong = expected.clone();
                match key {
                    "atoms" => wrong[key][0][1] = json!(2),
                    "bonds" => wrong[key][0] = json!([2, 1, 1]),
                    "components" => wrong[key] = json!([[1, 2, 3], [4, 5, 6, 7, 8, 9]]),
                    _ => wrong[key] = json!(63),
                }
                assert!(
                    !external(&raw, e.source, &wrong)?.status.success(),
                    "new oracle must reject wrong {key}"
                );
            }
        }
    }
    Ok(())
}

#[test]
fn sixty_four_duplicate_isolated_carbons_fit_original_shelf_without_deduplication() -> TestResult {
    let source = vec!["C"; 64].join(".");
    let owner = parse(&source)?;
    let before = owner.clone();
    let document = owner.layout_neutral_components(
        48.0,
        Point2::new(72.0, 72.0)?,
        PageSize::A4,
        &Cancellation::default(),
    )?;
    let f = fragment(&document)?;
    assert_eq!(f.atoms().len(), 64);
    assert!(f.bonds().is_empty());
    for (index, atom) in f.atoms().iter().enumerate() {
        assert_eq!(atom.id.get(), u32::try_from(index + 1)?);
        assert_eq!(atom.element.get(), 6);
        assert_eq!(atom.explicit_hydrogens, Some(4));
        let point = atom.position.ok_or("position")?;
        near(point.x_pt(), 72.0 + 48.0 * (index % 11) as f64);
        near(point.y_pt(), 72.0 + 48.0 * (index / 11) as f64);
    }
    scene_certificate(&document)?;
    let literal = json!({"atoms":vec![[6,4];64],"bonds":[],"components":(1..=64).map(|id|vec![id]).collect::<Vec<_>>(),"formula":"C64H256"});
    let result = external(&write_document(&document)?, &source, &literal)?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    same_source(&owner, &before);
    Ok(())
}

#[test]
fn total_cap_invalid_scalars_cancellation_late_unsupported_and_whole_paper_refuse_immutably()
-> TestResult {
    for source in [
        vec!["C"; 65].join("."),
        format!("[13C].{}", vec!["C"; 64].join(".")),
    ] {
        let owner = parse(&source)?;
        let before = owner.clone();
        let error = owner
            .layout_neutral_components(
                64.0,
                Point2::new(144.0, 144.0)?,
                PageSize::A4,
                &Cancellation::default(),
            )
            .err()
            .ok_or("65 atoms succeeded")?;
        assert_eq!(
            error.code(),
            "resource_limit",
            "total cap must precede unsupported isotope normalization"
        );
        same_source(&owner, &before);
    }
    for source in [
        "CC.[13CH4]",
        "CC.[NH4+]",
        "CC.c1ccccc1",
        "CC.C1=CCCCC1",
        "CC.C1CCCCCCCC1",
        "CC.C1CC2CCC1C2",
        "CC.CC(=CC)C",
        "CC.[H]",
    ] {
        let owner = parse(source)?;
        let before = owner.clone();
        let error = owner
            .layout_neutral_components(
                64.0,
                Point2::new(144.0, 144.0)?,
                PageSize::A4,
                &Cancellation::default(),
            )
            .err()
            .ok_or("later unsupported component succeeded")?;
        assert_eq!(error.code(), "unsupported_content", "{source}: {error}");
        same_source(&owner, &before);
    }
    let owner = parse("CC.O")?;
    let before = owner.clone();
    for length in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let error = owner
            .layout_neutral_components(
                length,
                Point2::new(144.0, 144.0)?,
                PageSize::A4,
                &Cancellation::default(),
            )
            .err()
            .ok_or("invalid length succeeded")?;
        assert!(matches!(error, LayoutError::InvalidBondLength));
        assert_eq!(error.code(), "invalid_params");
    }
    let cancelled = Cancellation::default();
    cancelled.cancel();
    let error = owner
        .layout_neutral_components(64.0, Point2::new(144.0, 144.0)?, PageSize::A4, &cancelled)
        .err()
        .ok_or("cancelled succeeded")?;
    assert!(matches!(error, LayoutError::Cancelled));
    assert_eq!(error.code(), "cancelled");
    for (length, origin, page) in [
        (64.0, Point2::new(-1.0, 72.0)?, PageSize::A4),
        (64.0, Point2::new(1000.0, 72.0)?, PageSize::A4),
        (64.0, Point2::new(0.0, 0.0)?, PageSize::A4),
        (64.0, Point2::new(72.0, 72.0)?, PageSize::new(100.0, 100.0)?),
        (f64::MIN_POSITIVE, Point2::new(72.0, 72.0)?, PageSize::A4),
        (f64::MAX, Point2::new(72.0, 72.0)?, PageSize::A4),
    ] {
        assert!(
            owner
                .layout_neutral_components(length, origin, page, &Cancellation::default())
                .is_err(),
            "paper/ink/finite geometry must refuse"
        );
    }
    same_source(&owner, &before);
    Ok(())
}
