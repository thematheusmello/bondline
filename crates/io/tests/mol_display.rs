use bondline_io::{
    StoredDocument,
    display::{DisplayError, drawing_page_scene},
    mol::MolTransform,
};
use bondline_model::{
    Atom, AtomId, Bond, BondId, Element, Fragment, FragmentId, PageId, PageSize, Point2,
};
use bondline_render::{
    Cancellation, PageViewport, cpu_scene, graph_page_scene,
    graph_page_scene_with_display_hydrogens,
};
use std::{
    collections::BTreeMap,
    io::Write as _,
    process::{Command, Stdio},
    sync::Arc,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const ORDINARY: &[u8] = include_bytes!("fixtures/sdf-preview/valence-absent-mixed.sdf");

fn v3(atoms: &[&str], bonds: &[&str]) -> Vec<u8> {
    format!("Original drawing display test\n  Bondline          2D\nIndependent fields\n  0  0  0  0  0  0            999 V3000\nM  V30 BEGIN CTAB\nM  V30 COUNTS {} {} 0 0 0\nM  V30 BEGIN ATOM\n{}M  V30 END ATOM\nM  V30 BEGIN BOND\n{}M  V30 END BOND\nM  V30 END CTAB\nM  END\n", atoms.len(), bonds.len(), atoms.iter().map(|line|format!("M  V30 {line}\n")).collect::<String>(), bonds.iter().map(|line|format!("M  V30 {line}\n")).collect::<String>()).into_bytes()
}

fn from_mol(source: &[u8]) -> Result<StoredDocument, Box<dyn std::error::Error>> {
    Ok(StoredDocument::from_mol(
        Arc::from(source),
        PageSize::A4,
        MolTransform::new(24.0, Point2::new(72.0, 72.0)?)?,
    )?)
}

fn display(stored: &StoredDocument) -> Result<bondline_render::GraphScene, DisplayError> {
    drawing_page_scene(
        1024,
        1024,
        stored,
        stored.model().pages()[0].id(),
        &Cancellation::default(),
    )
}

fn external(source: &[u8]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let interpreter = std::env::var_os("BONDLINE_PYTHON").ok_or("BONDLINE_PYTHON is required")?;
    let mut child = Command::new(interpreter)
        .arg("-B")
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/mol_oracle.py"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("stdin")?.write_all(source)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

#[test]
fn actual_raw_mol_spans_face_the_unchanged_independent_consumer() -> TestResult {
    for (range, smiles, formula, h, positions) in [
        (
            0..371,
            "CCO",
            "C2H6O",
            vec![3, 2, 1],
            vec![[-1.25, 0.5, 0.0], [0.25, 0.5, 0.0], [0.25, 2.0, 0.0]],
        ),
        (
            410..713,
            "CN",
            "CH5N",
            vec![3, 2],
            vec![[1.5, -1.5, 0.0], [0.0, 0.0, 0.0]],
        ),
    ] {
        let source = ORDINARY.get(range).ok_or("raw MOL span")?;
        let report = external(source)?;
        assert_eq!(report["smiles"], smiles);
        assert_eq!(report["formula"], formula);
        for (index, (count, point)) in h.into_iter().zip(positions).enumerate() {
            assert_eq!(report["atoms"][index]["hydrogens"], count);
            assert_eq!(report["atoms"][index]["position"], serde_json::json!(point));
        }
        let stored = from_mol(source)?;
        let before = stored.clone();
        let _scene = display(&stored)?;
        assert_eq!(stored, before);
        assert_eq!(stored.export_mol()?, source);
        assert_eq!(
            stored
                .to_mol(stored.mol_transform().ok_or("frame")?)?
                .as_ref(),
            source
        );
        assert!(
            stored.to_cdxml().is_err(),
            "display grants no cross-format authority"
        );
    }
    Ok(())
}

#[test]
fn isolated_literal_mol_h0_remains_supported_while_sdf_keeps_its_strict_policy() -> TestResult {
    let source = v3(&["1 H 0 0 0 0 VAL=-1"], &[]);
    let stored = from_mol(&source)?;
    assert!(stored.is_fully_mapped());
    let page = stored.model().pages().first().ok_or("page")?;
    let old_literal = graph_page_scene(1024, 1024, page)?;
    let new = drawing_page_scene(1024, 1024, &stored, page.id(), &Cancellation::default())?;
    assert_eq!(cpu_scene(new.scene())?, cpu_scene(old_literal.scene())?);
    let mut collection = source.clone();
    collection.extend_from_slice(b"$$$$\n");
    let sdf = bondline_io::sdf::SdfDocument::parse(Arc::from(collection))?;
    let error = bondline_io::sdf::preview::prepare_sdf_preview(&sdf, &Cancellation::default())
        .err()
        .ok_or("SDF physical H must retain its original topology refusal")?;
    assert_eq!(error.code(), "unsupported_content");
    assert_eq!(error.record_id().map(|id| id.get()), Some(1));
    assert!(matches!(
        error,
        bondline_io::sdf::preview::PreviewError::Unsupported {
            feature: "physical H requires one Single bond and no attached H",
            ..
        }
    ));
    assert_eq!(stored.export_mol()?, source);
    Ok(())
}

#[test]
fn ordinary_mol_uses_actual_coordinates_and_independent_display_counts() -> TestResult {
    for (range, hydrogens, coordinates) in [
        (
            0..371,
            vec![3_u16, 2, 1],
            vec![[42.0, 60.0], [78.0, 60.0], [78.0, 24.0]],
        ),
        (410..713, vec![3, 2], vec![[108.0, 108.0], [72.0, 72.0]]),
    ] {
        let source: Arc<[u8]> = Arc::from(ORDINARY.get(range).ok_or("raw MOL span")?);
        let stored = StoredDocument::from_mol(
            Arc::clone(&source),
            PageSize::A4,
            MolTransform::new(24.0, Point2::new(72.0, 72.0)?)?,
        )?;
        let before = stored.clone();
        assert!(stored.is_fully_mapped());
        let page = stored.model().pages().first().ok_or("page")?;
        let fragment = page.fragments().first().ok_or("fragment")?;
        for (atom, expected) in fragment.atoms().iter().zip(coordinates) {
            let point = atom.position.ok_or("point")?;
            assert_eq!([point.x_pt(), point.y_pt()], expected);
            assert_eq!(atom.explicit_hydrogens, None);
        }
        let expected = hydrogens
            .into_iter()
            .enumerate()
            .map(|(index, h)| Ok((AtomId::new(u32::try_from(index + 1)?).ok_or("atom")?, h)))
            .collect::<Result<BTreeMap<_, _>, Box<dyn std::error::Error>>>()?;
        let actual = drawing_page_scene(
            1024,
            1024,
            &stored,
            PageId::new(1).ok_or("page ID")?,
            &Cancellation::default(),
        )?;
        let literal_expected = graph_page_scene_with_display_hydrogens(
            1024,
            1024,
            page,
            &expected,
            &Cancellation::default(),
        )?;
        assert_eq!(actual.scene(), literal_expected.scene());
        assert_eq!(
            cpu_scene(actual.scene())?,
            cpu_scene(literal_expected.scene())?
        );
        assert_ne!(actual.scene(), graph_page_scene(1024, 1024, page)?.scene());
        let viewport = PageViewport::new(1024, 1024, page.size())?;
        for atom in fragment.atoms() {
            assert_eq!(
                actual.hit_test(
                    viewport.canvas_position(atom.position.ok_or("position")?)?,
                    0.0
                )?,
                Some(bondline_render::HitTarget::Atom {
                    fragment_id: fragment.id(),
                    atom_id: atom.id,
                })
            );
        }
        assert_eq!(stored, before);
        assert_eq!(stored.export_mol()?, source.as_ref());
    }
    Ok(())
}

#[test]
fn literal_zero_counts_cover_all_118_elements_and_unusual_counts_stay_literal() -> TestResult {
    for number in 1..=118 {
        let symbol = Element::try_from(number)?.symbol().ok_or("symbol")?;
        for count in [0_u16, 14] {
            if number == 1 && count != 0 {
                continue;
            }
            let valence = if count == 0 { -1 } else { i32::from(count) };
            let line = format!("1 {symbol} 0 0 0 0 VAL={valence}");
            let source = v3(&[&line], &[]);
            let stored = from_mol(&source)?;
            let before = stored.clone();
            assert!(stored.is_fully_mapped(), "{number}/{count}");
            let page = &stored.model().pages()[0];
            assert_eq!(
                page.fragments()[0].atoms()[0].explicit_hydrogens,
                Some(count)
            );
            let expected = BTreeMap::from([(AtomId::new(1).ok_or("atom")?, count)]);
            let scene = display(&stored)?;
            assert_eq!(
                scene.scene(),
                graph_page_scene_with_display_hydrogens(
                    1024,
                    1024,
                    page,
                    &expected,
                    &Cancellation::default()
                )?
                .scene()
            );
            assert_eq!(stored, before);
            assert_eq!(stored.export_mol()?, source);
        }
    }
    // A current authored literal can exceed MOL's VAL field without becoming an inferred default.
    let mut stored = from_mol(&v3(&["1 Si 0 0 0 0 VAL=4"], &[]))?;
    let page = stored.model().pages()[0].clone();
    let fragment = &page.fragments()[0];
    let mut atoms = fragment.atoms().to_vec();
    atoms[0].explicit_hydrogens = Some(u16::MAX);
    stored
        .model_mut()
        .replace_fragment(page.id(), Fragment::new(fragment.id(), atoms, vec![])?)?;
    let before = stored.clone();
    let expected = BTreeMap::from([(AtomId::new(1).ok_or("atom")?, u16::MAX)]);
    assert_eq!(
        display(&stored)?.scene(),
        graph_page_scene_with_display_hydrogens(
            1024,
            1024,
            &stored.model().pages()[0],
            &expected,
            &Cancellation::default()
        )?
        .scene()
    );
    assert_eq!(stored, before);
    assert!(stored.export_mol().is_err());
    assert!(
        stored
            .to_mol(stored.mol_transform().ok_or("frame")?)
            .is_err()
    );
    Ok(())
}

#[test]
fn source_free_and_native_cdxml_keep_literal_none_and_exact_legacy_errors() -> TestResult {
    let mut native = StoredDocument::default();
    let atoms = [(7, 100.0), (8, 172.0)]
        .into_iter()
        .enumerate()
        .map(|(i, (z, x))| {
            Ok(Atom::new(
                AtomId::new(u32::try_from(i + 1)?).ok_or("atom")?,
                Element::try_from(z)?,
                Some(Point2::new(x, 200.0)?),
            ))
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    native.model_mut().append_fragments(
        PageId::new(1).ok_or("page")?,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            atoms,
            vec![],
        )?],
    )?;
    let xml = bondline_io::cdxml::write_document(native.model())?;
    let imported = StoredDocument::from_cdxml(Arc::from(xml), PageSize::A4)?;
    for stored in [&native, &imported] {
        assert!(stored.is_fully_mapped());
        assert!(stored.mol_version().is_none());
        let before = stored.clone();
        let page = &stored.model().pages()[0];
        let legacy = graph_page_scene(1024, 1024, page)?;
        assert_eq!(display(stored)?.scene(), legacy.scene());
        assert!(
            page.fragments()[0]
                .atoms()
                .iter()
                .all(|atom| atom.explicit_hydrogens.is_none())
        );
        assert_eq!(*stored, before);
    }
    let mut isolated = StoredDocument::default();
    isolated.model_mut().append_fragments(
        PageId::new(1).ok_or("page")?,
        vec![Fragment::new(
            FragmentId::new(7).ok_or("fragment")?,
            vec![Atom::new(
                AtomId::new(70).ok_or("atom")?,
                Element::try_from(6)?,
                Some(Point2::new(144.0, 144.0)?),
            )],
            vec![],
        )?],
    )?;
    let old_error = graph_page_scene(1024, 1024, &isolated.model().pages()[0])
        .err()
        .ok_or("old refusal")?;
    let new_error = display(&isolated).err().ok_or("new refusal")?;
    assert_eq!(new_error.code(), "unsupported_content");
    assert_eq!(new_error.to_string(), old_error.to_string());
    assert!(matches!(
        new_error,
        DisplayError::Render(bondline_render::Error::UnsupportedChemicalContent)
    ));
    Ok(())
}

#[test]
fn actual_current_page_ids_layout_and_frame_are_used_without_repacking() -> TestResult {
    let source = ORDINARY.get(0..371).ok_or("raw MOL")?;
    let mut stored = StoredDocument::from_mol(
        Arc::from(source),
        PageSize::new(612.0, 792.0)?,
        MolTransform::new(40.0, Point2::new(140.0, 300.0)?)?,
    )?;
    let page = stored.model().pages()[0].clone();
    let original = &page.fragments()[0];
    let expected_positions = [[90.0, 280.0], [150.0, 280.0], [150.0, 220.0]];
    for (atom, xy) in original.atoms().iter().zip(expected_positions) {
        let p = atom.position.ok_or("position")?;
        assert_eq!([p.x_pt(), p.y_pt()], xy);
    }
    let mut atoms = original.atoms().to_vec();
    for (atom, id) in atoms.iter_mut().zip([10, 25, 40]) {
        atom.id = AtomId::new(id).ok_or("id")?;
    }
    let bonds = vec![
        Bond {
            id: BondId::new(70).ok_or("bond")?,
            begin: atoms[0].id,
            end: atoms[1].id,
            order: bondline_model::BondOrder::Single,
        },
        Bond {
            id: BondId::new(90).ok_or("bond")?,
            begin: atoms[1].id,
            end: atoms[2].id,
            order: bondline_model::BondOrder::Single,
        },
    ];
    stored
        .model_mut()
        .replace_fragment(page.id(), Fragment::new(original.id(), atoms, bonds)?)?;
    let before = stored.clone();
    let scene = display(&stored)?;
    let current = &stored.model().pages()[0];
    assert_eq!(current.size(), PageSize::new(612.0, 792.0)?);
    let counts = [(10, 3_u16), (25, 2), (40, 1)]
        .into_iter()
        .map(|(id, h)| Ok((AtomId::new(id).ok_or("atom")?, h)))
        .collect::<Result<BTreeMap<_, _>, Box<dyn std::error::Error>>>()?;
    assert_eq!(
        scene.scene(),
        graph_page_scene_with_display_hydrogens(
            1024,
            1024,
            current,
            &counts,
            &Cancellation::default()
        )?
        .scene()
    );
    let viewport = PageViewport::new(1024, 1024, current.size())?;
    for (atom, xy) in current.fragments()[0]
        .atoms()
        .iter()
        .zip(expected_positions)
    {
        assert_eq!(
            scene.hit_test(viewport.canvas_position(Point2::new(xy[0], xy[1])?)?, 0.0)?,
            Some(bondline_render::HitTarget::Atom {
                fragment_id: original.id(),
                atom_id: atom.id
            })
        );
        assert_eq!(atom.explicit_hydrogens, None);
    }
    assert_eq!(stored, before);
    Ok(())
}

#[test]
fn incomplete_sources_refuse_before_any_partial_drawing() -> TestResult {
    let opaque: &[u8] = include_bytes!("../../cdxml/tests/fixtures/unknown.cdxml");
    let cdxml = StoredDocument::from_cdxml(Arc::from(opaque), PageSize::A4)?;
    let base: &[u8] = include_bytes!("fixtures/mol/rdkit-CH3.mol");
    let (_, rest) = std::str::from_utf8(base)?
        .split_once('\n')
        .ok_or("header")?;
    let long_header = format!("{}\n{rest}", "x".repeat(81));
    let mol = from_mol(long_header.as_bytes())?;
    for stored in [&cdxml, &mol] {
        let before = stored.clone();
        assert!(!stored.is_fully_mapped());
        let error = display(stored)
            .err()
            .ok_or("incomplete source must refuse")?;
        assert!(matches!(error, DisplayError::IncompleteSource));
        assert_eq!(error.code(), "unsupported_content");
        assert_eq!(*stored, before);
    }
    assert_eq!(mol.export_mol()?, long_header.as_bytes());
    assert_eq!(cdxml.export_cdxml()?, opaque);
    Ok(())
}

#[test]
fn unknown_or_unsupported_chemistry_refuses_without_mutating_source() -> TestResult {
    for (atoms, bonds, feature) in [
        (
            vec!["1 Si 0 0 0 0"],
            vec![],
            "unresolved H outside neutral C/N/O/halogen profile",
        ),
        (
            vec!["1 C 0 0 0 0 CHG=1"],
            vec![],
            "charge, isotope or radical annotations",
        ),
        (
            vec!["1 C 0 0 0 0 MASS=13"],
            vec![],
            "charge, isotope or radical annotations",
        ),
        (
            vec!["1 C 0 0 0 0 RAD=2"],
            vec![],
            "charge, isotope or radical annotations",
        ),
        (
            vec!["1 H 0 0 0 0"],
            vec![],
            "physical H requires one Single bond and no attached H",
        ),
        (
            vec!["1 C 0 0 0 0", "2 C 0 0 0 0"],
            vec!["1 1 1 2"],
            "coincident bond endpoints",
        ),
        (
            vec![
                "1 N 0 0 0 0",
                "2 C 2 0 0 0",
                "3 C -2 0 0 0",
                "4 C 0 2 0 0",
                "5 C 0 -2 0 0",
            ],
            vec!["1 1 1 2", "2 1 1 3", "3 1 1 4", "4 1 1 5"],
            "bond sum exceeds neutral default valence",
        ),
    ] {
        let source = v3(&atoms, &bonds);
        let stored = from_mol(&source)?;
        let before = stored.clone();
        assert!(stored.is_fully_mapped());
        let error = display(&stored).err().ok_or("unsupported chemistry")?;
        assert_eq!(error.code(), "unsupported_content");
        assert!(
            matches!(error,DisplayError::Unsupported{feature:actual} if actual==feature),
            "{error}"
        );
        assert_eq!(stored, before);
        assert_eq!(stored.export_mol()?, source);
    }
    Ok(())
}

#[test]
fn original_multiple_order_vectors_have_separate_components_without_authoring_h() -> TestResult {
    for order in [2, 3] {
        let line = format!("1 {order} 1 2");
        let source = v3(&["1 C 0 0 0 0", "2 C 2 0 0 0"], &[&line]);
        let stored = from_mol(&source)?;
        let before = stored.clone();
        let molecule = external(&source)?;
        assert_eq!(molecule["smiles"], if order == 2 { "C=C" } else { "C#C" });
        assert_eq!(
            molecule["formula"],
            if order == 2 { "C2H4" } else { "C2H2" }
        );
        for atom in molecule["atoms"].as_array().ok_or("external atoms")? {
            assert_eq!(atom["hydrogens"], 4 - order);
        }
        let page = &stored.model().pages()[0];
        let fragment = &page.fragments()[0];
        assert!(
            fragment
                .atoms()
                .iter()
                .all(|a| a.explicit_hydrogens.is_none())
        );
        let graph = display(&stored)?;
        let strokes = graph
            .bond_strokes(fragment.id(), fragment.bonds()[0].id)
            .collect::<Vec<_>>();
        let offsets: &[f32] = if order == 2 {
            &[-1.5, 1.5]
        } else {
            &[-3.0, 0.0, 3.0]
        };
        assert_eq!(strokes.len(), offsets.len());
        let viewport = PageViewport::new(1024, 1024, page.size())?;
        let a = viewport.canvas_position(Point2::new(72.0, 72.0)?)?;
        let b = viewport.canvas_position(Point2::new(120.0, 72.0)?)?;
        let width = viewport.stroke_width();
        let frame = cpu_scene(graph.scene())?;
        let pixel = |y: f32| {
            let x = ((a[0] + b[0]) / 2.0).floor() as u32;
            let at = ((y.floor() as u32 * frame.width + x) * 4) as usize;
            frame
                .rgba
                .get(at..at + 4)
                .ok_or("probe outside actual frame")
        };
        for (stroke, offset) in strokes.iter().zip(offsets) {
            let y = a[1] + offset * width;
            assert_eq!(stroke.begin[0], a[0]);
            assert_eq!(stroke.end[0], b[0]);
            assert!((stroke.begin[1] - y).abs() <= 1e-5);
            assert!((stroke.end[1] - y).abs() <= 1e-5);
            assert_eq!(stroke.width, width);
            assert_eq!(pixel(y)?, &[0, 0, 0, 255]);
        }
        assert_eq!(
            pixel(a[1] + if order == 2 { 0.0 } else { 1.5 * width })?,
            &[255; 4]
        );
        assert_eq!(stored, before);
        assert_eq!(stored.export_mol()?, source);
    }
    Ok(())
}

#[test]
fn physical_unknown_h_derives_only_when_bonded_and_literal_h_is_not_duplicated() -> TestResult {
    let source = v3(&["1 H 0 0 0 0", "2 O 2 0 0 0"], &["1 1 1 2"]);
    let stored = from_mol(&source)?;
    let page = &stored.model().pages()[0];
    assert!(
        page.fragments()[0]
            .atoms()
            .iter()
            .all(|a| a.explicit_hydrogens.is_none())
    );
    let expected = BTreeMap::from([
        (AtomId::new(1).ok_or("H")?, 0),
        (AtomId::new(2).ok_or("O")?, 1),
    ]);
    assert_eq!(
        display(&stored)?.scene(),
        graph_page_scene_with_display_hydrogens(
            1024,
            1024,
            page,
            &expected,
            &Cancellation::default()
        )?
        .scene()
    );
    assert_eq!(stored.export_mol()?, source);
    Ok(())
}

#[test]
fn cancellation_missing_page_empty_mol_and_render_boundaries_stay_typed() -> TestResult {
    let stored = from_mol(ORDINARY.get(0..371).ok_or("raw MOL")?)?;
    let before = stored.clone();
    let missing = PageId::new(27).ok_or("page")?;
    let error = drawing_page_scene(1024, 1024, &stored, missing, &Cancellation::default())
        .err()
        .ok_or("missing page")?;
    assert_eq!(error.code(), "not_found");
    assert!(matches!(error,DisplayError::MissingPage{page_id} if page_id==missing));
    let cancel = Cancellation::default();
    cancel.cancel();
    let error = drawing_page_scene(1024, 1024, &stored, missing, &cancel)
        .err()
        .ok_or("cancel")?;
    assert_eq!(error.code(), "cancelled");
    assert!(matches!(error, DisplayError::Cancelled));
    for (width, height, code) in [(0, 1024, "render_failed"), (32, 32, "unsupported_scale")] {
        let error = drawing_page_scene(
            width,
            height,
            &stored,
            PageId::new(1).ok_or("page")?,
            &Cancellation::default(),
        )
        .err()
        .ok_or("render boundary")?;
        assert_eq!(error.code(), code, "{error}");
    }
    let empty = from_mol(&v3(&[], &[]))?;
    assert_eq!(
        display(&empty)?.scene(),
        graph_page_scene(1024, 1024, &empty.model().pages()[0])?.scene()
    );
    let outside = from_mol(&v3(&["1 O -4 0 0 0"], &[]))?;
    let error = display(&outside)
        .err()
        .ok_or("paper ink must not be translated into view")?;
    assert_eq!(error.code(), "unsupported_content");
    assert!(matches!(
        error,
        DisplayError::Render(bondline_render::Error::LabelPlacement)
    ));
    assert_eq!(stored, before);
    Ok(())
}

#[test]
fn existing_hit_budget_precedes_unresolved_h_and_counts_atoms_plus_bonds() -> TestResult {
    for (atom_count, bond_count) in [(4097, 0), (2049, 2048)] {
        let atoms = (1..=atom_count)
            .map(|id| format!("{id} {} 0 0 0 0", if id == 1 { "P" } else { "C" }))
            .collect::<Vec<_>>();
        // Coincident connected endpoints are also deliberately invalid after the budget.
        let bonds = (1..=bond_count)
            .map(|id| format!("{id} 1 {id} {}", id + 1))
            .collect::<Vec<_>>();
        let source = v3(
            &atoms.iter().map(String::as_str).collect::<Vec<_>>(),
            &bonds.iter().map(String::as_str).collect::<Vec<_>>(),
        );
        let stored = from_mol(&source)?;
        let before = stored.clone();
        assert!(stored.is_fully_mapped());
        let page = &stored.model().pages()[0];
        assert!(matches!(
            graph_page_scene(1024, 1024, page),
            Err(bondline_render::Error::SceneTooComplex)
        ));
        let error = display(&stored).err().ok_or("existing hit budget")?;
        assert_eq!(error.code(), "render_failed");
        assert!(matches!(
            error,
            DisplayError::Render(bondline_render::Error::SceneTooComplex)
        ));
        assert_eq!(stored, before);
        assert_eq!(stored.export_mol()?, source);
    }
    Ok(())
}
