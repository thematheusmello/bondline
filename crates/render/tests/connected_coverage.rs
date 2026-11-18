//! Exact saved-source numeric data and an independent rational pixel-centre oracle.
use bondline_model::{
    Atom, AtomId, AtomLabel, Bond, BondId, BondOrder, Document, DoublePosition, DrawingSettings,
    DrawingStyle, Element, Fragment, FragmentId, PageSize, Point2, TextJustification, TextRun,
    TextStyle,
};
use bondline_render::{
    Cancellation, PageView, Primitive, Rectangle, Scene, Stroke, cpu_scene, gpu_scene,
};
use serde_json::{Value, json};
use std::{
    io::Write,
    process::{Command, Stdio},
    sync::Arc,
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn saved_scene() -> TestResult<bondline_render::GraphScene> {
    let v: Value = serde_json::from_str(include_str!("fixtures/moved-connected-coverage.json"))?;
    let mut atoms = Vec::new();
    for (i, a) in v["atoms"].as_array().ok_or("atoms")?.iter().enumerate() {
        let point: [f64; 2] = serde_json::from_value(a["point"].clone())?;
        let mut atom = Atom::new(
            AtomId::new(i as u32 + 1).ok_or("atom")?,
            Element::try_from(a["element"].as_u64().ok_or("element")? as u8)?,
            Some(Point2::new(point[0], point[1])?),
        );
        atom.explicit_hydrogens = Some(a["hydrogens"].as_u64().ok_or("H")? as u16);
        let bounds: [f64; 4] = serde_json::from_value(a["bounding_box"].clone())?;
        atom.authored_label = Some(Arc::new(
            AtomLabel::new(
                None,
                Some(TextJustification::Left),
                Some(TextJustification::Left),
                None,
                vec![TextRun::new(
                    a["text"].as_str().ok_or("text")?.into(),
                    TextStyle::new(
                        "Roboto".into(),
                        a["size"].as_f64().ok_or("size")?,
                        0,
                        [0.2, 0.4, 0.8],
                    )?,
                )?],
            )?
            .with_bounding_box(Some(bounds))?,
        ));
        atoms.push(atom);
    }
    let mut d = Document::default();
    let p = d.pages()[0].id();
    d.set_page_size(p, PageSize::new(512., 512.)?)?;
    let bonds = [(1, 2), (1, 3)]
        .into_iter()
        .enumerate()
        .map(|(i, (a, b))| {
            Ok(Bond {
                id: BondId::new(i as u32 + 1).ok_or("bond")?,
                begin: AtomId::new(a).ok_or("atom")?,
                end: AtomId::new(b).ok_or("atom")?,
                order: BondOrder::Single,
                double_position: DoublePosition::Unspecified,
            })
        })
        .collect::<TestResult<Vec<_>>>()?;
    d.append_fragments(
        p,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            atoms,
            bonds,
        )?],
    )?;
    let s = &v["style"];
    let view = PageView::fit(1088, 1088, d.pages()[0].size())?
        .with_drawing_style(
            DrawingStyle::new(Some(s["line_width"].as_f64().ok_or("width")?))?
                .with_margin_width(s["margin"].as_f64())?
                .with_bond_spacing_percent(s["spacing"].as_f64())?,
        )
        .with_drawing_settings(DrawingSettings::new(s["length"].as_f64(), None)?);
    Ok(view.graph_scene(&d.pages()[0], &Cancellation::default())?)
}

fn rational_pixels(scene: &bondline_render::Scene, points: &[[u32; 2]]) -> TestResult<Value> {
    let primitives = scene
        .primitives
        .iter()
        .map(|primitive| match primitive {
            Primitive::Rectangle(r) => {
                json!({"rectangle":[r.x,r.y,r.width,r.height],"color":r.color})
            }
            Primitive::Stroke(s) => json!({
                "stroke":[s.begin[0].to_bits(),s.begin[1].to_bits(),s.end[0].to_bits(),s.end[1].to_bits(),s.width.to_bits()],
                "color":s.color,
            }),
        })
        .collect::<Vec<_>>();
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| "python3".into());
    let mut child = Command::new(python)
        .args([
            "-B",
            "-c",
            r#"
from fractions import Fraction as F
import json,sys,struct
v=json.load(sys.stdin)
def scalar(bits): return F.from_float(struct.unpack('<f',int(bits).to_bytes(4,'little'))[0])
result=[]
for x,y in v['points']:
 p=[F(2*x+1,2),F(2*y+1,2)]; color=v['background']
 for primitive in v['primitives']:
  if 'rectangle' in primitive:
   a,b,w,h=primitive['rectangle']; inside=a<=x<a+w and b<=y<b+h
  else:
   ax,ay,bx,by,w=map(scalar,primitive['stroke']);dx,dy=bx-ax,by-ay;px,py=p[0]-ax,p[1]-ay
   length2=dx*dx+dy*dy; along=px*dx+py*dy; cross=px*dy-py*dx
   inside=0<=along<=length2 and 4*cross*cross<=w*w*length2
  if inside: color=primitive['color']
 result.append(color)
json.dump(result,sys.stdout)
"#,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("stdin")?.write_all(
        serde_json::to_string(
            &json!({"background":scene.background,"primitives":primitives,"points":points}),
        )?
        .as_bytes(),
    )?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(serde_json::from_slice(&result.stdout)?)
}

#[test]
fn actual_moved_connected_scene_gpu_matches_exact_closed_butt_coverage() -> TestResult {
    let graph = saved_scene()?;
    let scene = graph.scene();
    let cpu = cpu_scene(scene)?;
    let (gpu, adapter) = pollster::block_on(gpu_scene(scene))?;
    println!("coverage adapter {:?} {}", adapter.backend, adapter.name);
    let points = [
        [517, 436],
        [522, 436],
        [521, 439],
        [526, 439],
        [538, 448],
        [537, 451],
        [542, 451],
        [541, 454],
        [546, 454],
    ];
    let expected = rational_pixels(scene, &points)?;
    for (i, [x, y]) in points.into_iter().enumerate() {
        let offset = ((y * scene.width + x) * 4) as usize;
        let expected: [u8; 4] = serde_json::from_value(expected[i].clone())?;
        assert_eq!(
            &cpu.rgba[offset..offset + 4],
            &expected,
            "independent exact CPU {x},{y}"
        );
        println!(
            "exact sample {x},{y}: expected{expected:?} GPU{:?}",
            &gpu.rgba[offset..offset + 4]
        );
    }
    let differences = cpu
        .rgba
        .chunks_exact(4)
        .zip(gpu.rgba.chunks_exact(4))
        .enumerate()
        .filter_map(|(i, (a, b))| {
            (a != b).then_some([i as u32 % scene.width, i as u32 / scene.width])
        })
        .collect::<Vec<_>>();
    assert!(
        differences.is_empty(),
        "every pixel of the actual moved scene must agree; differences:{differences:?}"
    );
    for primitive in &scene.primitives {
        if let Primitive::Stroke(s) = primitive {
            println!("actual primitive {:?}", s);
        }
    }
    Ok(())
}

#[test]
fn row_geometry_preserves_exact_closed_edges_clipping_empty_rows_and_order() -> TestResult {
    let cases = [
        // Pixel centres exactly on both caps and both side edges are included.
        ([0.5, 3.5], [31.5, 3.5], 2.0),
        ([-10.0, 3.0], [40.0, 20.0], 1.25),
        ([31.5, -5.5], [4.25, 31.75], 0.25),
        // Visible conservative bounds contain no covered pixel centres.
        ([10.125, 10.125], [10.375, 10.375], 0.125),
        // Fully clipped geometry must leave only the ordered rectangle.
        ([-20.0, -30.0], [-3.0, -1.0], 1.0),
    ];
    for (a, b, width) in cases {
        for (begin, end) in [(a, b), (b, a)] {
            let scene = Scene {
                width: 32,
                height: 24,
                background: [255; 4],
                primitives: vec![
                    Primitive::Stroke(Stroke {
                        begin,
                        end,
                        width,
                        color: [12, 34, 56, 78],
                    }),
                    Primitive::Rectangle(Rectangle {
                        x: 4,
                        y: 2,
                        width: 5,
                        height: 4,
                        color: [90, 80, 70, 60],
                    }),
                ],
            };
            let points = (0..scene.height)
                .flat_map(|y| (0..scene.width).map(move |x| [x, y]))
                .collect::<Vec<_>>();
            let expected: Vec<[u8; 4]> = serde_json::from_value(rational_pixels(&scene, &points)?)?;
            let expected = expected.into_iter().flatten().collect::<Vec<_>>();
            assert_eq!(cpu_scene(&scene)?.rgba, expected, "CPU {begin:?}->{end:?}");
            assert_eq!(
                pollster::block_on(gpu_scene(&scene))?.0.rgba,
                expected,
                "GPU {begin:?}->{end:?}"
            );
        }
    }
    Ok(())
}
