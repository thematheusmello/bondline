use bondline_io::{StoredDocument, cdxml::write_document};
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Document, Element, Fragment, FragmentId, PageId,
    PageSize, Point2, Radical,
};
use std::{
    borrow::Cow,
    io::Write,
    num::NonZeroU16,
    process::{Command, Stdio},
};

fn external_graph(bytes: &[u8]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(python)
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../xtask/oracles/cdxml_graph.py"
        ))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("oracle stdin")?.write_all(bytes)?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(serde_json::from_slice(&result.stdout)?)
}

#[test]
fn native_elemental_metadata_and_all_orders_match_an_independent_projection()
-> Result<(), Box<dyn std::error::Error>> {
    let orders = [
        BondOrder::Single,
        BondOrder::Double,
        BondOrder::Triple,
        BondOrder::Quadruple,
        BondOrder::Quintuple,
        BondOrder::Sextuple,
        BondOrder::Half,
        BondOrder::Aromatic,
        BondOrder::TwoAndHalf,
        BondOrder::ThreeAndHalf,
        BondOrder::FourAndHalf,
        BondOrder::FiveAndHalf,
        BondOrder::Dative,
        BondOrder::Ionic,
        BondOrder::Hydrogen,
        BondOrder::ThreeCenter,
    ];
    let mut document = Document::default();
    let page = PageId::new(1).ok_or("page")?;
    document.set_page_size(page, PageSize::new(612.125, 792.5)?)?;
    let mut fragments = Vec::new();
    for (index, order) in orders.into_iter().enumerate() {
        let index = u32::try_from(index)?;
        let a = AtomId::new(index * 2 + 1).ok_or("atom")?;
        let b = AtomId::new(index * 2 + 2).ok_or("atom")?;
        let mut first = Atom::new(
            a,
            Element::try_from(6)?,
            Some(Point2::new(-100000.0 + f64::from(index), 1e-100)?),
        );
        first.isotope = NonZeroU16::new(13);
        first.charge = -128;
        first.radical = match index % 4 {
            0 => Radical::None,
            1 => Radical::Singlet,
            2 => Radical::Doublet,
            _ => Radical::Triplet,
        };
        first.explicit_hydrogens = Some(if index % 2 == 0 { 0 } else { u16::MAX });
        let mut second = Atom::new(b, Element::try_from(118)?, None);
        second.charge = 127;
        second.isotope = NonZeroU16::new(i16::MAX as u16);
        fragments.push(Fragment::new(
            FragmentId::new(index + 1).ok_or("fragment")?,
            vec![first, second],
            vec![Bond {
                double_position: bondline_model::DoublePosition::Unspecified,
                id: BondId::new(index + 1).ok_or("bond")?,
                begin: b,
                end: a,
                order,
            }],
        )?);
    }
    document.append_fragments(page, fragments)?;
    let before = document.clone();
    let bytes = write_document(&document)?;
    let expected = serde_json::to_value(document.pages().first().ok_or("page")?.fragments())?;
    assert_eq!(external_graph(&bytes)?, expected);
    let loaded = StoredDocument::from_cdxml(bytes.into(), PageSize::new(1.0, 2.0)?)?;
    assert_eq!(
        loaded.model(),
        &document,
        "explicit Poster extents override only the fallback preview size"
    );
    assert_eq!(document, before);
    Ok(())
}

#[test]
fn native_export_handles_sparse_maximal_ids_with_global_source_id_uniqueness()
-> Result<(), Box<dyn std::error::Error>> {
    let mut native = StoredDocument::default();
    let a = AtomId::new(u32::MAX).ok_or("atom")?;
    let b = AtomId::new(u32::MAX - 1).ok_or("atom")?;
    native.model_mut().append_fragments(
        PageId::new(1).ok_or("page")?,
        vec![Fragment::new(
            FragmentId::new(u32::MAX).ok_or("fragment")?,
            vec![
                Atom::new(a, Element::try_from(7)?, None),
                Atom::new(b, Element::try_from(8)?, Some(Point2::new(-0.0, 100000.0)?)),
            ],
            vec![Bond {
                double_position: bondline_model::DoublePosition::Unspecified,
                id: BondId::new(u32::MAX).ok_or("bond")?,
                begin: b,
                end: a,
                order: BondOrder::Dative,
            }],
        )?],
    )?;
    let before = native.clone();
    let bytes = native.to_cdxml()?;
    assert!(matches!(bytes, Cow::Owned(_)));
    let xml = roxmltree::Document::parse(std::str::from_utf8(&bytes)?)?;
    let ids: Vec<_> = xml
        .descendants()
        .filter_map(|node| node.attribute("id"))
        .collect();
    let unique: std::collections::BTreeSet<_> = ids.iter().copied().collect();
    assert_eq!(ids.len(), 5);
    assert_eq!(ids.len(), unique.len());
    let expected = serde_json::json!([{"id":1,"atoms":[{"id":1,"element":7,"position":null,"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":null},{"id":2,"element":8,"position":{"x_pt":-0.0,"y_pt":100000.0},"isotope":null,"charge":0,"radical":"none","explicit_hydrogens":null}],"bonds":[{"id":1,"begin":2,"end":1,"order":"dative"}]}]);
    assert_eq!(external_graph(&bytes)?, expected);
    assert_eq!(native, before);
    assert!(native.source_info().is_none());
    Ok(())
}

#[test]
fn source_backed_export_remains_borrowed_exact_and_refuses_unsynchronized_edits()
-> Result<(), Box<dyn std::error::Error>> {
    const SOURCE: &[u8] = include_bytes!("../../cdxml/tests/fixtures/unknown.cdxml");
    let original = StoredDocument::from_cdxml(SOURCE.into(), PageSize::A4)?;
    let bytes = original.to_cdxml()?;
    assert!(matches!(bytes, Cow::Borrowed(_)));
    assert_eq!(bytes.as_ref(), SOURCE);
    let mut modified = original.clone();
    modified
        .model_mut()
        .set_page_size(PageId::new(1).ok_or("page")?, PageSize::new(72.0, 144.0)?)?;
    assert!(matches!(
        modified.to_cdxml(),
        Err(bondline_io::DocumentError::UnsupportedEdit)
    ));
    assert_eq!(original.to_cdxml()?.as_ref(), SOURCE);
    Ok(())
}

#[test]
fn unrepresentable_isotopes_and_oversized_graphs_fail_without_partial_export()
-> Result<(), Box<dyn std::error::Error>> {
    let mut document = Document::default();
    let mut atom = Atom::new(AtomId::new(1).ok_or("atom")?, Element::try_from(6)?, None);
    atom.isotope = NonZeroU16::new(32768);
    document.append_fragments(
        PageId::new(1).ok_or("page")?,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            vec![atom],
            vec![],
        )?],
    )?;
    assert!(matches!(
        write_document(&document),
        Err(bondline_io::cdxml::MappingError::InvalidProperty("Isotope"))
    ));
    let mut large = Document::default();
    let fragments: Result<Vec<_>, _> = (1..=bondline_model::MAX_GRAPH_ITEMS)
        .map(|id| {
            Fragment::new(
                FragmentId::new(u32::try_from(id).map_err(|_| "id")?).ok_or("fragment")?,
                vec![],
                vec![],
            )
            .map_err(|_| "graph")
        })
        .collect();
    large.append_fragments(PageId::new(1).ok_or("page")?, fragments?)?;
    assert!(matches!(
        write_document(&large),
        Err(bondline_io::cdxml::MappingError::Syntax(
            bondline_cdxml::Error::AttributeLimit
        ))
    ));
    Ok(())
}
