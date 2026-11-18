use bondline_model::{
    Atom, AtomId, AtomLabel, Document, Element, Fragment, FragmentId, Page, PageSize, Point2,
    TextJustification, TextRun, TextStyle,
};
use serde_json::Value;
use std::sync::Arc;
pub type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
pub fn cases() -> TestResult<Vec<Value>> {
    let all: Value =
        serde_json::from_str(include_str!("../fixtures/boxed-atom-font-vectors.json"))?;
    Ok(all["cases"].as_array().ok_or("cases")?.clone())
}
pub fn atom(v: &Value) -> TestResult<Atom> {
    let mut atom = Atom::new(
        AtomId::new(1).ok_or("atom")?,
        Element::try_from(v["element"].as_u64().ok_or("element")? as u8)?,
        Some(Point2::new(
            v["node_point"][0].as_f64().ok_or("x")?,
            v["node_point"][1].as_f64().ok_or("y")?,
        )?),
    );
    atom.explicit_hydrogens = Some(v["explicit_hydrogens"].as_u64().ok_or("H")? as u16);
    let bounds: [f64; 4] = serde_json::from_value(v["bounding_box"].clone())?;
    let label = AtomLabel::new(
        None,
        Some(TextJustification::Left),
        Some(TextJustification::Left),
        None,
        vec![TextRun::new(
            v["text"].as_str().ok_or("text")?.into(),
            TextStyle::new(
                "Roboto".into(),
                v["size_pt"].as_f64().ok_or("size")?,
                0,
                [0.2, 0.4, 0.8],
            )?,
        )?],
    )?
    .with_bounding_box(Some(bounds))?;
    atom.authored_label = Some(Arc::new(label));
    Ok(atom)
}
#[allow(
    dead_code,
    reason = "shared with scene tests; geometry tests use literal atoms only"
)]
pub fn page(atom: Atom) -> TestResult<Page> {
    let mut d = Document::default();
    let p = d.pages()[0].id();
    d.set_page_size(p, PageSize::new(512.0, 512.0)?)?;
    d.append_fragments(
        p,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            vec![atom],
            vec![],
        )?],
    )?;
    Ok(d.pages()[0].clone())
}
