use bondline_io::{
    mol::{MolFile, MolTransform, Version, v3000},
    sdf::{
        SdfDocument,
        preview::{HydrogenOrigin, prepare_sdf_preview},
    },
};
use bondline_model::{AtomId, BondId, FragmentId, PageSize};
use bondline_render::Cancellation;
use std::{
    collections::BTreeMap,
    io::Write as _,
    process::{Command, Stdio},
    sync::Arc,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const ORDINARY: &[u8] = include_bytes!("fixtures/sdf-preview/valence-absent-mixed.sdf");

#[test]
fn valence_absent_records_resolve_display_h_without_authoring_it() -> TestResult {
    let source: Arc<[u8]> = Arc::from(ORDINARY);
    let pointer = source.as_ptr();
    let weak = Arc::downgrade(&source);
    let owner = SdfDocument::parse(source)?;
    let before = owner.clone();
    let preview = prepare_sdf_preview(&owner, &Cancellation::default())?;
    assert_eq!(owner, before);
    assert_eq!(owner.as_bytes(), ORDINARY);
    assert_eq!(owner.as_bytes().as_ptr(), pointer);
    assert_eq!(preview.page().size(), PageSize::A4);
    assert_eq!(preview.records().len(), 2);
    assert_eq!(preview.page().fragments().len(), 2);
    let expected_positions = [
        vec![[72.0, 108.0], [108.0, 108.0], [108.0, 72.0]],
        vec![[108.0, 216.0], [72.0, 180.0]],
    ];
    for (index, (record, fragment)) in preview
        .records()
        .iter()
        .zip(preview.page().fragments())
        .enumerate()
    {
        assert_eq!(record.record_id().get(), u32::try_from(index + 1)?);
        assert_eq!(
            record.fragment_id(),
            FragmentId::new(u32::try_from(index + 1)?).ok_or("fragment")?
        );
        assert_eq!(record.fragment_id(), fragment.id());
        assert_eq!(fragment.atoms().len(), expected_positions[index].len());
        for (atom, xy) in fragment.atoms().iter().zip(&expected_positions[index]) {
            let position = atom.position.ok_or("position")?;
            assert_eq!([position.x_pt(), position.y_pt()], *xy);
            assert_eq!(atom.explicit_hydrogens, None);
        }
    }
    let expected_h = (1..=5)
        .zip([3, 2, 1, 3, 2])
        .map(|(id, h)| Ok((AtomId::new(id).ok_or("atom")?, h)))
        .collect::<Result<BTreeMap<_, _>, Box<dyn std::error::Error>>>()?;
    assert_eq!(preview.display_hydrogens(), &expected_h);
    assert_eq!(preview.hydrogen_origins().len(), 5);
    assert!(
        preview
            .hydrogen_origins()
            .values()
            .all(|origin| *origin == HydrogenOrigin::NeutralDefaultV1)
    );
    let second = preview.records().get(1).ok_or("second")?;
    assert_eq!(
        second
            .atom_mapping()
            .iter()
            .map(|(preview, local)| (preview.get(), local.get()))
            .collect::<Vec<_>>(),
        [(4, 1), (5, 2)]
    );
    assert_eq!(
        second
            .bond_mapping()
            .iter()
            .map(|(preview, local)| (preview.get(), local.get()))
            .collect::<Vec<_>>(),
        [(3, 1)]
    );
    let bond = preview.page().fragments()[1].bonds()[0];
    assert_eq!(bond.id, BondId::new(3).ok_or("bond")?);
    assert_eq!([bond.begin.get(), bond.end.get()], [4, 5]);
    drop(before);
    drop(owner);
    assert!(
        weak.upgrade().is_none(),
        "preview must not lease source bytes"
    );
    assert_eq!(preview.page().fragments().len(), 2);
    Ok(())
}

fn v3(atoms: &[&str], bonds: &[&str]) -> Vec<u8> {
    format!("Original preview test\n  Bondline          2D\nIndependent CTfile fields\n  0  0  0  0  0  0            999 V3000\nM  V30 BEGIN CTAB\nM  V30 COUNTS {} {} 0 0 0\nM  V30 BEGIN ATOM\n{}M  V30 END ATOM\nM  V30 BEGIN BOND\n{}M  V30 END BOND\nM  V30 END CTAB\nM  END\n", atoms.len(), bonds.len(), atoms.iter().map(|line|format!("M  V30 {line}\n")).collect::<String>(), bonds.iter().map(|line|format!("M  V30 {line}\n")).collect::<String>()).into_bytes()
}

fn collection(molecules: &[Vec<u8>]) -> Vec<u8> {
    let mut source = Vec::new();
    for molecule in molecules {
        source.extend_from_slice(molecule);
        source.extend_from_slice(b"$$$$\n");
    }
    source
}

fn parse(source: &[u8]) -> Result<SdfDocument, Box<dyn std::error::Error>> {
    Ok(SdfDocument::parse(Arc::from(source))?)
}

fn external(
    script: &str,
    source: &[u8],
    mode: Option<&str>,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut command = Command::new(python);
    command
        .arg("-B")
        .arg(format!("{}/tests/{script}", env!("CARGO_MANIFEST_DIR")));
    if let Some(mode) = mode {
        command.arg(mode);
    }
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("oracle stdin")?
        .write_all(source)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

#[test]
fn actual_fixture_bytes_face_independent_raw_ranges_and_unchanged_molecular_consumers() -> TestResult
{
    assert_eq!(ORDINARY.len(), 740);
    assert_eq!(ORDINARY.get(405..410), Some(b"$$$$\n".as_slice()));
    assert_eq!(ORDINARY.get(735..740), Some(b"$$$$\n".as_slice()));
    let owner = parse(ORDINARY)?;
    let preview = prepare_sdf_preview(&owner, &Cancellation::default())?;
    let whole = external("sdf_mixed_oracle.py", ORDINARY, Some("describe"))?;
    assert_eq!(
        whole["source_sha256"],
        "e5cbdc14329c59bad55148c5de37058de90f651af10966c30cf8b89b3b5efd39"
    );
    assert_eq!(whole["records"].as_array().ok_or("records")?.len(), 2);
    for (index, (range, smiles, formula, hydrogens, positions)) in [
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
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(owner.records()[index].mol_range(), range.clone());
        let molecule = external(
            "mol_oracle.py",
            ORDINARY.get(range).ok_or("raw MOL range")?,
            None,
        )?;
        assert_eq!(molecule["smiles"], smiles);
        assert_eq!(molecule["formula"], formula);
        let fragment = &preview.page().fragments()[index];
        for (at, ((expected_h, expected_position), atom)) in hydrogens
            .into_iter()
            .zip(positions)
            .zip(fragment.atoms())
            .enumerate()
        {
            assert_eq!(molecule["atoms"][at]["hydrogens"], expected_h);
            assert_eq!(
                molecule["atoms"][at]["position"],
                serde_json::json!(expected_position)
            );
            assert_eq!(
                preview.display_hydrogens().get(&atom.id),
                Some(&u16::try_from(expected_h)?)
            );
            assert_eq!(
                whole["records"][index]["atoms"][at]["position"],
                serde_json::json!(expected_position)
            );
        }
    }
    assert_eq!(
        whole["records"][0]["properties"]["NOTE"],
        "ethanol\nvertical O bond"
    );
    assert_eq!(whole["records"][1]["properties"]["NOTE"], "methylamine");
    assert_eq!(owner.as_bytes(), ORDINARY);
    Ok(())
}

#[test]
fn both_existing_two_record_sources_keep_local_graphs_except_presentation_ids_and_translation()
-> TestResult {
    for source in [
        include_bytes!("fixtures/sdf/ordered-fields-v2000.sdf").as_slice(),
        include_bytes!("fixtures/sdf/mixed-v2000-v3000.sdf").as_slice(),
    ] {
        let owner = parse(source)?;
        let before = owner.clone();
        let preview = prepare_sdf_preview(&owner, &Cancellation::default())?;
        let frame = MolTransform::new(24.0, bondline_model::Point2::new(72.0, 72.0)?)?;
        assert_eq!(preview.records().len(), 2);
        for (index, ((metadata, mapping), fragment)) in owner
            .records()
            .iter()
            .zip(preview.records())
            .zip(preview.page().fragments())
            .enumerate()
        {
            let raw = source.get(metadata.mol_range()).ok_or("MOL")?;
            let original = match metadata.mol_version() {
                Version::V2000 => MolFile::parse(raw)?.to_fragment(frame)?,
                Version::V3000 => v3000::MolFile::parse(raw)?.to_fragment(frame)?,
            };
            assert_eq!(fragment.atoms().len(), original.atoms().len());
            assert_eq!(fragment.bonds().len(), original.bonds().len());
            for (at, (atom, old)) in fragment.atoms().iter().zip(original.atoms()).enumerate() {
                assert_eq!(mapping.atom_mapping().get(&atom.id), Some(&old.id));
                let mut reset = atom.clone();
                reset.id = old.id;
                reset.position = old.position;
                assert_eq!(&reset, old);
                assert_eq!(
                    preview.hydrogen_origins().get(&atom.id),
                    Some(&HydrogenOrigin::Literal)
                );
                assert_eq!(preview.display_hydrogens().get(&atom.id), Some(&3));
                let position = atom.position.ok_or("position")?;
                assert_eq!(
                    [position.x_pt(), position.y_pt()],
                    [72.0 + 36.0 * at as f64, 72.0 + 72.0 * index as f64]
                );
            }
            for (bond, old) in fragment.bonds().iter().zip(original.bonds()) {
                assert_eq!(mapping.bond_mapping().get(&bond.id), Some(&old.id));
                assert_eq!(mapping.atom_mapping().get(&bond.begin), Some(&old.begin));
                assert_eq!(mapping.atom_mapping().get(&bond.end), Some(&old.end));
                assert_eq!(bond.order, old.order);
            }
        }
        assert_eq!(owner, before);
    }
    Ok(())
}

#[test]
fn literal_counts_are_not_narrowed_to_the_derived_elements_or_rectified() -> TestResult {
    let source = collection(&[v3(
        &[
            "1 Si 0 0 0 0 VAL=4",
            "2 S 2 0 0 0 VAL=6",
            "3 Ne 4 0 0 0 VAL=-1",
            "4 C 6 0 0 0 VAL=3",
        ],
        &[],
    )]);
    let owner = parse(&source)?;
    let preview = prepare_sdf_preview(&owner, &Cancellation::default())?;
    for (atom, count) in preview.page().fragments()[0]
        .atoms()
        .iter()
        .zip([4, 6, 0, 3])
    {
        assert_eq!(atom.explicit_hydrogens, Some(count));
        assert_eq!(preview.display_hydrogens().get(&atom.id), Some(&count));
        assert_eq!(
            preview.hydrogen_origins().get(&atom.id),
            Some(&HydrogenOrigin::Literal)
        );
    }
    assert_eq!(owner.as_bytes(), source);
    Ok(())
}

#[test]
fn neutral_defaults_cover_all_closed_elements_and_integer_bond_orders() -> TestResult {
    let source = collection(&[
        v3(
            &[
                "1 C 0 0 0 0",
                "2 N 2 0 0 0",
                "3 O 4 0 0 0",
                "4 F 6 0 0 0",
                "5 Cl 8 0 0 0",
                "6 Br 10 0 0 0",
                "7 I 12 0 0 0",
            ],
            &[],
        ),
        v3(
            &[
                "10 C 0 0 0 0",
                "40 C 2 0 0 0",
                "90 C 4 0 0 0",
                "100 N 6 0 0 0",
            ],
            &["3 2 10 40", "8 3 90 100"],
        ),
    ]);
    let owner = parse(&source)?;
    let preview = prepare_sdf_preview(&owner, &Cancellation::default())?;
    assert_eq!(
        preview
            .display_hydrogens()
            .values()
            .copied()
            .collect::<Vec<_>>(),
        [4, 3, 2, 1, 1, 1, 1, 2, 2, 1, 0]
    );
    assert!(
        preview
            .page()
            .fragments()
            .iter()
            .flat_map(|f| f.atoms())
            .all(|atom| atom.explicit_hydrogens.is_none())
    );
    assert_eq!(
        preview.page().fragments()[1]
            .bonds()
            .iter()
            .map(|bond| bond.order)
            .collect::<Vec<_>>(),
        [
            bondline_model::BondOrder::Double,
            bondline_model::BondOrder::Triple
        ]
    );
    Ok(())
}

#[test]
fn physical_h_is_a_node_and_does_not_get_counted_twice() -> TestResult {
    for (physical, expected_origin) in [
        ("1 H 0 0 0 0", HydrogenOrigin::PhysicalHydrogenV1),
        ("1 H 0 0 0 0 VAL=1", HydrogenOrigin::Literal),
    ] {
        let source = collection(&[v3(&[physical, "2 O 1.5 0 0 0"], &["1 1 1 2"])]);
        let owner = parse(&source)?;
        let preview = prepare_sdf_preview(&owner, &Cancellation::default())?;
        assert_eq!(
            preview
                .display_hydrogens()
                .values()
                .copied()
                .collect::<Vec<_>>(),
            [0, 1]
        );
        assert_eq!(
            preview.hydrogen_origins().get(&AtomId::new(1).ok_or("H")?),
            Some(&expected_origin)
        );
        assert_eq!(
            preview.page().fragments()[0].atoms()[1].explicit_hydrogens,
            None
        );
    }
    for (atoms, bonds) in [
        (vec!["1 H 0 0 0 0"], vec![]),
        (vec!["1 H 0 0 0 0 VAL=2", "2 O 2 0 0 0"], vec!["1 1 1 2"]),
        (vec!["1 H 0 0 0 0", "2 C 2 0 0 0"], vec!["1 2 1 2"]),
        (
            vec!["1 H 0 0 0 0", "2 C 2 0 0 0", "3 C -2 0 0 0"],
            vec!["1 1 1 2", "2 1 1 3"],
        ),
    ] {
        let owner = parse(&collection(&[v3(&atoms, &bonds)]))?;
        let error = prepare_sdf_preview(&owner, &Cancellation::default())
            .err()
            .ok_or("unsupported physical H")?;
        assert_eq!(error.code(), "unsupported_content");
        assert_eq!(error.record_id().map(|id| id.get()), Some(1));
    }
    Ok(())
}

#[test]
fn branches_cycles_and_opposed_nitrogen_keep_actual_source_directions() -> TestResult {
    let source = collection(&[
        v3(
            &["1 N 0 0 0 0", "2 C -1.5 0 0 0", "3 C 1.5 0 0 0"],
            &["1 1 1 2", "2 1 1 3"],
        ),
        v3(
            &["1 C 0 0 0 0", "2 C 1 0 0 0", "3 C 0 1 0 0"],
            &["1 1 1 2", "2 1 2 3", "3 1 3 1"],
        ),
        v3(
            &["1 C 0 0 0 0", "2 C -1 0 0 0", "3 C 1 0 0 0", "4 C 0 1 0 0"],
            &["1 1 1 2", "2 1 1 3", "3 1 1 4"],
        ),
    ]);
    let owner = parse(&source)?;
    let preview = prepare_sdf_preview(&owner, &Cancellation::default())?;
    assert_eq!(
        preview
            .display_hydrogens()
            .values()
            .copied()
            .collect::<Vec<_>>(),
        [1, 3, 3, 2, 2, 2, 1, 3, 3, 3]
    );
    let opposed = preview.page().fragments().first().ok_or("opposed")?;
    let xy = opposed
        .atoms()
        .iter()
        .map(|atom| {
            atom.position
                .ok_or("position")
                .map(|p| [p.x_pt(), p.y_pt()])
        })
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(xy, [[108.0, 72.0], [72.0, 72.0], [144.0, 72.0]]);
    assert!(
        opposed
            .atoms()
            .iter()
            .all(|atom| atom.explicit_hydrogens.is_none())
    );
    assert_eq!(owner.as_bytes(), source);
    // This verifies data, not the deferred renderer's opposed-H label attachment.
    Ok(())
}

#[test]
fn later_unprojectable_records_refuse_the_whole_preview_without_touching_source() -> TestResult {
    for atoms in [
        vec!["1 Si 0 0 0 0"],
        vec!["1 C 0 0 0 0 CHG=1"],
        vec!["1 C 0 0 0 0 RAD=2"],
        vec!["1 C 0 0 0 0 MASS=13"],
        vec!["1 C 0 0 1 0"],
        vec!["1 C 0 0 0 0 CFG=1"],
        vec!["1 C 0 0 0 0 FUTURE=7"],
        vec!["1 Q 0 0 0 0"],
        vec![],
    ] {
        let source = collection(&[v3(&["1 C 0 0 0 0"], &[]), v3(&atoms, &[])]);
        let owner = parse(&source)?;
        let before = owner.clone();
        let error = prepare_sdf_preview(&owner, &Cancellation::default())
            .err()
            .ok_or("later record must refuse")?;
        assert_eq!(error.code(), "unsupported_content", "{error}");
        assert_eq!(error.record_id().map(|id| id.get()), Some(2));
        assert_eq!(owner, before);
        assert_eq!(owner.as_bytes(), source);
        assert_eq!(owner.records().len(), 2);
    }
    Ok(())
}

#[test]
fn overvalence_and_coincident_bond_geometry_are_explicit_refusals() -> TestResult {
    for molecule in [
        v3(
            &[
                "1 N 0 0 0 0",
                "2 C 1 0 0 0",
                "3 C -1 0 0 0",
                "4 C 0 1 0 0",
                "5 C 0 -1 0 0",
            ],
            &["1 1 1 2", "2 1 1 3", "3 1 1 4", "4 1 1 5"],
        ),
        v3(&["1 C 0 0 0 0", "2 C 0 0 0 0"], &["1 1 1 2"]),
        v3(&["1 C 0 0 0 0", "2 C 1 0 0 0"], &["1 4 1 2"]),
    ] {
        let owner = parse(&collection(&[molecule]))?;
        let error = prepare_sdf_preview(&owner, &Cancellation::default())
            .err()
            .ok_or("explicit refusal")?;
        assert_eq!(error.code(), "unsupported_content");
        assert_eq!(error.record_id().map(|id| id.get()), Some(1));
    }
    Ok(())
}

#[test]
fn page_grows_from_a4_but_preserves_relative_geometry_and_bounds() -> TestResult {
    let owner = parse(&collection(&[v3(
        &["1 C 0 0 0 0", "2 C 4000 0 0 0"],
        &["1 1 1 2"],
    )]))?;
    let preview = prepare_sdf_preview(&owner, &Cancellation::default())?;
    assert_eq!(preview.page().size().width_pt(), 96_144.0);
    assert_eq!(preview.page().size().height_pt(), PageSize::A4.height_pt());
    let a = preview.page().fragments()[0].atoms()[0]
        .position
        .ok_or("a")?;
    let b = preview.page().fragments()[0].atoms()[1]
        .position
        .ok_or("b")?;
    assert_eq!([b.x_pt() - a.x_pt(), b.y_pt() - a.y_pt()], [96_000.0, 0.0]);
    for (molecules, record) in [
        (
            vec![v3(&["1 C -2082 0 0 0", "2 C 2082 0 0 0"], &["1 1 1 2"])],
            1,
        ),
        (
            vec![v3(&["1 C 0 0 0 0", "2 C 0 -2100 0 0"], &["1 1 1 2"]); 2],
            2,
        ),
    ] {
        let owner = parse(&collection(&molecules))?;
        let error = prepare_sdf_preview(&owner, &Cancellation::default())
            .err()
            .ok_or("page bound")?;
        assert_eq!(error.code(), "resource_limit");
        assert_eq!(error.record_id().map(|id| id.get()), Some(record));
    }
    let owner = parse(&collection(&[v3(&["1 C 10000 0 0 0"], &[])]))?;
    let error = prepare_sdf_preview(&owner, &Cancellation::default())
        .err()
        .ok_or("strict mapper coordinate bound")?;
    assert_eq!(error.record_id().map(|id| id.get()), Some(1));
    assert_eq!(error.code(), "invalid_mol");
    Ok(())
}

#[test]
fn aggregate_graph_budgets_are_checked_across_records() -> TestResult {
    let atoms = (1..=50_001)
        .map(|id| format!("{id} C 0 0 0 0"))
        .collect::<Vec<_>>();
    let rows = atoms.iter().map(String::as_str).collect::<Vec<_>>();
    let molecule = v3(&rows, &[]);
    let owner = parse(&collection(&[molecule.clone(), molecule]))?;
    let error = prepare_sdf_preview(&owner, &Cancellation::default())
        .err()
        .ok_or("aggregate atom budget")?;
    assert_eq!(error.code(), "resource_limit");
    assert_eq!(error.record_id().map(|id| id.get()), Some(2));
    assert_eq!(owner.records().len(), 2);
    Ok(())
}

#[test]
fn cancelled_work_retains_source_and_debug_omits_source_data() -> TestResult {
    let owner = parse(ORDINARY)?;
    let before = owner.clone();
    let cancel = Cancellation::default();
    cancel.cancel();
    let error = prepare_sdf_preview(&owner, &cancel)
        .err()
        .ok_or("cancelled")?;
    assert_eq!(error.code(), "cancelled");
    assert_eq!(owner, before);
    let preview = prepare_sdf_preview(&owner, &Cancellation::default())?;
    let debug = format!("{preview:?}");
    assert!(debug.contains("record_count"));
    assert!(!debug.contains("ethanol"));
    assert!(!debug.contains("vertical O bond"));
    assert!(!debug.contains("M  V30"));
    Ok(())
}
