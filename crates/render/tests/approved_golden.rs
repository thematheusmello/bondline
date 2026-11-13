use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Document, Element, Fragment, FragmentId, PageId,
    PageSize, Point2,
};
use bondline_render::{carbon_page_scene, cpu_scene, gpu_scene};
use std::{
    io::Write,
    process::{Command, Stdio},
};

fn compare(rgba: &[u8], hash: &str) -> Result<std::process::Output, Box<dyn std::error::Error>> {
    let interpreter = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(interpreter)
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/golden_oracle.py"
        ))
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/goldens/single-bond.png"
        ))
        .arg(hash)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut input = child.stdin.take().ok_or("oracle stdin")?;
    input.write_all(rgba)?;
    drop(input);
    Ok(child.wait_with_output()?)
}

#[test]
fn cpu_and_gpu_match_the_human_approved_native_bond_capture()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("goldens/single-bond.json"))?;
    let number = |name| {
        fixture
            .get(name)
            .and_then(serde_json::Value::as_f64)
            .ok_or("fixture coordinate")
    };
    let first = AtomId::new(1).ok_or("atom")?;
    let second = AtomId::new(2).ok_or("atom")?;
    let mut a = Atom::new(
        first,
        Element::try_from(6)?,
        Some(Point2::new(number("begin_x_pt")?, number("begin_y_pt")?)?),
    );
    let mut b = Atom::new(
        second,
        Element::try_from(6)?,
        Some(Point2::new(number("end_x_pt")?, number("end_y_pt")?)?),
    );
    a.explicit_hydrogens = Some(3);
    b.explicit_hydrogens = Some(3);
    let mut document = Document::default();
    let page = PageId::new(1).ok_or("page")?;
    document.set_page_size(
        page,
        PageSize::new(number("page_width_pt")?, number("page_height_pt")?)?,
    )?;
    document.append_fragments(
        page,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            vec![a, b],
            vec![Bond {
                id: BondId::new(1).ok_or("bond")?,
                begin: first,
                end: second,
                order: BondOrder::Single,
            }],
        )?],
    )?;
    assert_eq!(fixture["width_px"], 1024);
    assert_eq!(fixture["height_px"], 1024);
    let scene = carbon_page_scene(1024, 1024, document.pages().first().ok_or("page")?)?;
    let hash = fixture["png_sha256"].as_str().ok_or("hash")?;
    let cpu = cpu_scene(&scene)?;
    let output = compare(&cpu.rgba, hash)?;
    assert!(
        output.status.success(),
        "CPU golden: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let (gpu, _) = pollster::block_on(gpu_scene(&scene))?;
    let output = compare(&gpu.rgba, hash)?;
    assert!(
        output.status.success(),
        "GPU golden: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut corrupted = cpu.rgba;
    *corrupted.first_mut().ok_or("empty frame")? ^= 1;
    let output = compare(&corrupted, hash)?;
    assert!(!output.status.success(), "oracle accepted corrupted pixels");
    assert!(String::from_utf8_lossy(&output.stderr).contains("rendered pixels differ"));
    Ok(())
}
