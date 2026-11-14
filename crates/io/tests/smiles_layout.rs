use bondline_io::{
    StoredDocument,
    cdxml::write_document,
    smiles::{LayoutError, SmilesDocument},
};
use bondline_model::{Document, Fragment, PageSize, Point2};
use bondline_render::{PageViewport, cpu_scene, graph_page_scene};
use serde_json::Value;
use std::{
    io::Write as _,
    process::{Command, Stdio},
    sync::Arc,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn parse(source: &str) -> Result<SmilesDocument, Box<dyn std::error::Error>> {
    Ok(SmilesDocument::parse(Arc::from(source.as_bytes()))?)
}
fn fragment(document: &Document) -> Result<&Fragment, Box<dyn std::error::Error>> {
    Ok(document
        .pages()
        .first()
        .ok_or("page")?
        .fragments()
        .first()
        .ok_or("fragment")?)
}
fn xy(document: &Document, index: usize) -> Result<[f64; 2], Box<dyn std::error::Error>> {
    let p = fragment(document)?
        .atoms()
        .get(index)
        .ok_or("atom")?
        .position
        .ok_or("position")?;
    Ok([p.x_pt(), p.y_pt()])
}
fn near(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} != {expected} within {tolerance}"
    );
}

#[test]
fn two_and_three_atom_layouts_have_independent_length40_coordinates() -> TestResult {
    let origin = Point2::new(72.0, 72.0)?;
    let two = parse("CC")?.layout_carbon_path(40.0, origin, PageSize::A4)?;
    assert_eq!(xy(&two, 0)?, [72.0, 72.0]);
    near(xy(&two, 1)?[0], 112.0, 1e-10);
    near(xy(&two, 1)?[1], 72.0, 1e-10);
    let three = parse("CCC")?.layout_carbon_path(40.0, origin, PageSize::A4)?;
    assert_eq!(xy(&three, 0)?, [72.0, 72.0]);
    near(xy(&three, 1)?[0], 72.0 + 20.0 * 3.0_f64.sqrt(), 1e-10);
    near(xy(&three, 1)?[1], 92.0, 1e-10);
    near(xy(&three, 2)?[0], 72.0 + 40.0 * 3.0_f64.sqrt(), 1e-10);
    near(xy(&three, 2)?[1], 72.0, 1e-10);
    assert_eq!(three.pages()[0].size(), PageSize::A4);
    let letter = PageSize::new(612.0, 792.0)?;
    let custom = parse("CC")?.layout_carbon_path(40.0, Point2::new(100.0, 200.0)?, letter)?;
    assert_eq!(custom.pages()[0].size(), letter);
    assert_eq!(xy(&custom, 0)?, [100.0, 200.0]);
    assert_eq!(xy(&custom, 1)?, [140.0, 200.0]);
    external(
        "../../xtask/oracles/files_rdkit.py",
        &write_document(&custom)?,
        &["1".to_owned()],
    )?;
    Ok(())
}

#[test]
fn all_twelve_path_sizes_preserve_owner_and_fixed_model_except_positions() -> TestResult {
    for n in 1..=12 {
        let source = format!("{}\r\n", "C".repeat(n));
        let owner = parse(&source)?;
        let before = owner.clone();
        let normalized = owner.normalize_explicit_h_fragment()?;
        let document = owner.layout_carbon_path(48.0, Point2::new(72.0, 72.0)?, PageSize::A4)?;
        let result = fragment(&document)?;
        assert_eq!(document.pages().len(), 1);
        assert_eq!(document.pages()[0].fragments().len(), 1);
        assert_eq!(result.id(), normalized.id());
        assert_eq!(result.bonds(), normalized.bonds());
        assert_eq!(result.atoms().len(), n);
        for (actual, original) in result.atoms().iter().zip(normalized.atoms()) {
            let mut without_position = actual.clone();
            assert!(without_position.position.take().is_some());
            assert_eq!(&without_position, original);
        }
        for (i, atom) in result.atoms().iter().enumerate() {
            assert_eq!(
                atom.explicit_hydrogens,
                Some(if n == 1 {
                    4
                } else if i == 0 || i + 1 == n {
                    3
                } else {
                    2
                })
            );
            if i > 0 {
                let a = xy(&document, i - 1)?;
                let b = xy(&document, i)?;
                near((b[0] - a[0]).hypot(b[1] - a[1]), 48.0, 48e-9);
                assert!(b[0] > a[0], "path must not fold back or cross");
            }
            if i > 0 && i + 1 < n {
                let a = xy(&document, i - 1)?;
                let b = xy(&document, i)?;
                let c = xy(&document, i + 1)?;
                let u = [a[0] - b[0], a[1] - b[1]];
                let v = [c[0] - b[0], c[1] - b[1]];
                near((u[0] * v[0] + u[1] * v[1]) / (48.0 * 48.0), -0.5, 1e-12);
            }
            for j in (i + 2)..n {
                let a = xy(&document, i)?;
                let b = xy(&document, j)?;
                assert!((b[0] - a[0]).hypot(b[1] - a[1]) >= 1.7 * 48.0);
            }
        }
        near(xy(&document, 0)?[1], xy(&document, n - 1)?[1], 1e-10);
        let page = &document.pages()[0];
        let graph = graph_page_scene(1024, 1024, page)?;
        let frame = cpu_scene(graph.scene())?;
        let viewport = PageViewport::new(1024, 1024, page.size())?;
        let paper = viewport.paper();
        let mut black = 0;
        for (i, pixel) in frame.rgba.chunks_exact(4).enumerate() {
            if pixel == [0, 0, 0, 255] {
                black += 1;
                let x = u32::try_from(i % 1024)?;
                let y = u32::try_from(i / 1024)?;
                assert!(x > paper.x && x < paper.x + paper.width - 1);
                assert!(y > paper.y && y < paper.y + paper.height - 1);
            }
        }
        assert!(black > 0);
        assert_eq!(owner, before);
        assert_eq!(owner.as_bytes(), source.as_bytes());
        assert_eq!(owner.as_bytes().as_ptr(), before.as_bytes().as_ptr());
        assert_eq!(owner.atoms().as_ptr(), before.atoms().as_ptr());
        assert_eq!(owner.bonds().as_ptr(), before.bonds().as_ptr());
        assert!(normalized.atoms().iter().all(|a| a.position.is_none()));
    }
    Ok(())
}

#[test]
fn connectivity_not_token_order_selects_the_lowest_id_endpoint() -> TestResult {
    let origin = Point2::new(72.0, 72.0)?;
    let branch = parse("C(C)C")?.layout_carbon_path(40.0, origin, PageSize::A4)?;
    assert_eq!(xy(&branch, 1)?, [72.0, 72.0]); // Source atom2 is the lowest endpoint.
    near(xy(&branch, 0)?[0], 72.0 + 20.0 * 3.0_f64.sqrt(), 1e-10);
    near(xy(&branch, 2)?[0], 72.0 + 40.0 * 3.0_f64.sqrt(), 1e-10);
    let bracketed =
        parse("[CH2]([CH3])[CH3]\r\n")?.layout_carbon_path(40.0, origin, PageSize::A4)?;
    assert_eq!(branch, bracketed);
    let cross_dot = parse("C1.C1")?.layout_carbon_path(40.0, origin, PageSize::A4)?;
    let ordinary = parse("CC")?.layout_carbon_path(40.0, origin, PageSize::A4)?;
    assert_eq!(cross_dot, ordinary);
    Ok(())
}

#[test]
fn unsupported_topology_and_metadata_refuse_without_source_changes() -> TestResult {
    for source in [
        "C1CC1",
        "CC(C)C",
        "C(C)(C)(C)C",
        "CO",
        "C[H]",
        "[13CH3]C",
        "[0CH3]C",
        "[CH3+]",
        "CC.CC",
        "C=C",
        "C#C",
        "[CH3]",
    ] {
        let owner = parse(source)?;
        let before = owner.clone();
        let error = owner
            .layout_carbon_path(48.0, Point2::new(72.0, 72.0)?, PageSize::A4)
            .err()
            .ok_or_else(|| format!("accepted {source}"))?;
        assert_eq!(error.code(), "unsupported_content", "{source}: {error}");
        assert_eq!(owner, before);
        assert_eq!(owner.as_bytes().as_ptr(), before.as_bytes().as_ptr());
        assert_eq!(owner.atoms().as_ptr(), before.atoms().as_ptr());
        assert_eq!(owner.bonds().as_ptr(), before.bonds().as_ptr());
    }
    let large = parse(&"C".repeat(13))?;
    assert!(matches!(
        large.layout_carbon_path(48.0, Point2::new(72.0, 72.0)?, PageSize::A4),
        Err(LayoutError::AtomLimit)
    ));
    for source in ["c1ccccc1", "[cH]1ccccc1", "[CH3:4]", "[C@H](F)(Cl)Br"] {
        match SmilesDocument::parse(Arc::from(source.as_bytes())) {
            Ok(owner) => assert_eq!(
                owner
                    .layout_carbon_path(48.0, Point2::new(72.0, 72.0)?, PageSize::A4)
                    .err()
                    .ok_or("accepted aromatic/stereo/class")?
                    .code(),
                "unsupported_content"
            ),
            Err(error) => assert_eq!(error.code(), "unsupported_content"),
        }
    }
    Ok(())
}

#[test]
fn invalid_lengths_collapsed_geometry_labels_and_paper_ink_refuse() -> TestResult {
    let owner = parse("CC")?;
    for length in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(matches!(
            owner.layout_carbon_path(length, Point2::new(72.0, 72.0)?, PageSize::A4),
            Err(LayoutError::InvalidBondLength)
        ));
        assert_eq!(
            owner
                .layout_carbon_path(length, Point2::new(72.0, 72.0)?, PageSize::A4)
                .err()
                .ok_or("accepted invalid length")?
                .code(),
            "invalid_params"
        );
    }
    for (length, origin, page) in [
        (1e-300, Point2::new(72.0, 72.0)?, PageSize::A4),
        (f64::MAX, Point2::new(72.0, 72.0)?, PageSize::A4),
        (48.0, Point2::new(-1.0, 72.0)?, PageSize::A4),
        (48.0, Point2::new(100_000.0, 72.0)?, PageSize::A4),
        (
            48.0,
            Point2::new(72.0, 72.0)?,
            PageSize::new(1.0, 100_000.0)?,
        ),
    ] {
        assert!(owner.layout_carbon_path(length, origin, page).is_err());
    }
    assert!(matches!(
        owner.layout_carbon_path(1.0, Point2::new(72.0, 72.0)?, PageSize::A4),
        Err(LayoutError::Render(bondline_render::Error::LabelOverlap))
    ));
    for origin in [
        Point2::new(2.0, 72.0)?,
        Point2::new(548.0, 72.0)?,
        Point2::new(72.0, 2.0)?,
        Point2::new(72.0, 838.0)?,
    ] {
        // Every center is on the page; the full glyph/mask extent must also fit.
        assert!(matches!(
            owner.layout_carbon_path(40.0, origin, PageSize::A4),
            Err(LayoutError::Geometry(_))
        ));
    }
    Ok(())
}

fn external(
    script: &str,
    source: &[u8],
    arguments: &[String],
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(python)
        .arg("-B")
        .arg(format!("{}/{}", env!("CARGO_MANIFEST_DIR"), script))
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("stdin")?.write_all(source)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "external generated-CDXML consumer: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(output.stdout)
}

#[test]
fn actual_generated_native_cdxml_faces_unchanged_consumers_and_reopens_completely() -> TestResult {
    for n in 1..=12 {
        let owner = parse(&"C".repeat(n))?;
        let document = owner.layout_carbon_path(48.0, Point2::new(72.0, 72.0)?, PageSize::A4)?;
        let bytes = write_document(&document)?;
        let independent: Value = serde_json::from_slice(&external(
            "../../xtask/oracles/cdxml_graph.py",
            &bytes,
            &[],
        )?)?;
        // Both sides cross the same JSON-number boundary; raw Decimal coordinates below
        // and strict model/reopen equality separately detect any actual CDXML loss.
        let expected: Value =
            serde_json::from_slice(&serde_json::to_vec(document.pages()[0].fragments())?)?;
        assert_eq!(independent, expected);
        if n == 2 {
            external("../../xtask/oracles/native_bond_rdkit.py", &bytes, &[])?;
        }
        let coordinates = fragment(&document)?
            .atoms()
            .iter()
            .map(|atom| {
                let point = atom.position.ok_or("position")?;
                Ok([point.x_pt().to_string(), point.y_pt().to_string()])
            })
            .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
        let chemistry: Value = serde_json::from_slice(&external(
            "tests/smiles_layout_oracle.py",
            &bytes,
            &[serde_json::to_string(&coordinates)?],
        )?)?;
        assert_eq!(chemistry["smiles"], "C".repeat(n));
        assert_eq!(
            chemistry["formula"],
            if n == 1 {
                "CH4".to_owned()
            } else {
                format!("C{n}H{}", 2 * n + 2)
            }
        );
        assert_eq!(chemistry["atoms"], n);
        assert_eq!(chemistry["bonds"], n - 1);
        let loaded = StoredDocument::from_cdxml(bytes.clone().into(), PageSize::new(1.0, 2.0)?)?;
        assert!(loaded.is_fully_mapped());
        assert_eq!(loaded.model(), &document);
        assert_eq!(loaded.to_cdxml()?.as_ref(), bytes);
        assert!(!loaded.source_info().ok_or("source")?.modified);
    }
    Ok(())
}
