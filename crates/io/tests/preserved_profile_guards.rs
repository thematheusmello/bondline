//! Independent closed-profile, unsupported-delta and whole-format loss guards.
use bondline_io::{
    DocumentError, StoredDocument,
    cdxml::MappingError,
    mol::{MolTransform, Version},
};
use bondline_model::{
    Atom, AtomId, BondId, Document as ModelDocument, Element, Fragment, FragmentId, PageSize,
    Point2,
};
use std::{borrow::Cow, sync::Arc};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const SOURCE: &str = "<CDXML CreationProgram='Bondline 0.1.0'><page id='91' DrawingSpace='Poster' Width='612.00' Height='792.000'><fragment id='400'><n id='101' NodeType='Element' Element='6' Charge='0' Radical='None' p='100 200' NumHydrogens='3'/><n id='207' NodeType='Element' Element='6' Charge='0' Radical='None' p='140 200' NumHydrogens='3'/><b id='900' B='101' E='207' Order='1'/></fragment><fragment id='500'><n id='301' NodeType='Element' Element='8' Charge='0' Radical='None' p='240 240' NumHydrogens='2'/></fragment></page></CDXML>";

#[test]
fn opaque_syntax_metadata_and_default_key_mismatches_keep_readonly_source_exact() -> TestResult {
    let mut sources = vec![
        SOURCE.replacen(
            "<CDXML ",
            "<CDXML xmlns:xml='http://www.w3.org/XML/1998/namespace' ",
            1,
        ),
        SOURCE.replacen("<CDXML ", "<CDXML xmlns:f='urn:unused' ", 1),
        SOURCE.replacen("<page ", "<page xmlns='' ", 1),
        SOURCE.replacen("<fragment ", "<fragment xmlns='' ", 1),
        format!("<!DOCTYPE CDXML>{SOURCE}"),
        format!("<?future opaque?>{SOURCE}"),
        SOURCE.replacen("</fragment>", "<![CDATA[ \t]]></fragment>", 1),
        SOURCE.replacen("</fragment>", "not whitespace</fragment>", 1),
        SOURCE.replacen("<CDXML ", "<CDXML Future='opaque' ", 1),
        SOURCE.replacen("<CDXML ", "<CDXML LineWidth='9' ", 1),
        SOURCE.replacen("<page ", "<page Future='opaque' ", 1),
        SOURCE.replacen("<fragment ", "<fragment Future='opaque' ", 1),
        SOURCE.replacen("<n ", "<n Future='opaque' ", 1),
        SOURCE.replacen("<b ", "<b Future='opaque' ", 1),
        SOURCE.replacen("</page>", "<unknown/></page>", 1),
        SOURCE.replacen("</page>", "<t><s>caption</s></t></page>", 1),
        SOURCE
            .replacen(
                "<fragment id='400'>",
                "<group id='77'><fragment id='400'>",
                1,
            )
            .replacen("</fragment>", "</fragment></group>", 1),
        SOURCE.replacen("Bondline 0.1.0", "Bondline 999.0.0", 1),
        SOURCE.replacen(" CreationProgram='Bondline 0.1.0'", "", 1),
        SOURCE.replacen(" Charge='0'", "", 1),
        SOURCE.replacen(" Radical='None'", "", 1),
        SOURCE.replacen(" Element='6'", "", 1),
        SOURCE.replacen(" NodeType='Element'", "", 1),
        SOURCE.replacen(" Order='1'", "", 1),
        SOURCE.replacen("<n ", "<n Isotope='0' ", 1),
        SOURCE.replacen("DrawingSpace='Poster'", "DrawingSpace='Pages'", 1),
        SOURCE.replacen("<CDXML ", "<CDXML BondLength='0' ", 1),
        SOURCE.replacen("id='91'", "id='0'", 1),
        SOURCE.replacen("id='400'", "id='0'", 1),
        SOURCE.replacen(" id='91'", "", 1),
    ];
    // A nested reset on an atom is a valid readonly control with no dangling reference.
    sources.push(SOURCE.replacen("<n ", "<n xmlns='' ", 1).replacen(
        "<b id='900' B='101' E='207' Order='1'/>",
        "",
        1,
    ));
    for source in sources {
        let mut stored = StoredDocument::from_cdxml(source.as_bytes().into(), PageSize::A4)?;
        assert!(!stored.is_fully_mapped(), "must stay opaque: {source}");
        assert!(matches!(stored.to_cdxml()?, Cow::Borrowed(_)));
        assert_eq!(stored.to_cdxml()?.as_ref(), source.as_bytes());
        let before = stored.clone();
        let page = stored.model().pages()[0].id();
        stored
            .model_mut()
            .set_page_size(page, PageSize::new(640.0, 800.0)?)?;
        assert!(!stored.is_fully_mapped());
        assert!(
            matches!(stored.to_cdxml(), Err(DocumentError::UnsupportedEdit)),
            "{source}"
        );
        assert_eq!(before.to_cdxml()?.as_ref(), source.as_bytes());
        *stored.model_mut() = before.model().clone();
        assert!(!stored.source_info().ok_or("source")?.modified);
        assert_eq!(stored.to_cdxml()?.as_ref(), source.as_bytes());
    }
    Ok(())
}

#[test]
fn ordinary_comments_and_quotes_qualify_but_old_root_namespace_parser_errors_stay_errors()
-> TestResult {
    for source in [
        SOURCE.to_owned(),
        SOURCE.replacen(
            "<page ",
            "<!--xmlns='' <?ghost data?> <![CDATA[ ]]>--><page ",
            1,
        ),
    ] {
        let mut stored = StoredDocument::from_cdxml(source.as_bytes().into(), PageSize::A4)?;
        assert!(stored.is_fully_mapped());
        let page = stored.model().pages()[0].id();
        stored
            .model_mut()
            .set_page_size(page, PageSize::new(640.0, 792.0)?)?;
        assert_eq!(
            stored.to_cdxml()?.as_ref(),
            source
                .replacen("Width='612.00'", "Width='640'", 1)
                .as_bytes()
        );
    }
    for source in ["<CDXML xmlns=''/>", "<CDXML xmlns='urn:foreign'/>"] {
        assert!(matches!(
            StoredDocument::from_cdxml(source.as_bytes().into(), PageSize::A4),
            Err(DocumentError::Mapping(MappingError::Syntax(
                bondline_cdxml::Error::InvalidRoot
            )))
        ));
    }
    Ok(())
}

fn rebuilt(fragments: Vec<Fragment>, size: PageSize) -> TestResult<ModelDocument> {
    let mut d = ModelDocument::default();
    let page = d.pages()[0].id();
    d.set_page_size(page, size)?;
    d.append_fragments(page, fragments)?;
    Ok(d)
}

#[test]
fn prefix_deletion_reorder_reparent_identity_and_directed_endpoint_drift_refuse_without_fallback()
-> TestResult {
    let base = StoredDocument::from_cdxml(SOURCE.as_bytes().into(), PageSize::A4)?;
    assert!(base.is_fully_mapped());
    let page = base.model().pages()[0].id();
    let size = base.model().pages()[0].size();
    let f = base.model().pages()[0].fragments()[0].as_ref();
    let other = base.model().pages()[0].fragments()[1].as_ref();
    assert_eq!(
        f.atoms().iter().map(|a| a.id.get()).collect::<Vec<_>>(),
        [1, 2]
    );
    assert_eq!(other.atoms()[0].id.get(), 3);
    let mut cases = Vec::new();
    let mut atoms = f.atoms().to_vec();
    atoms.reverse();
    let mut d = base.model().clone();
    d.replace_fragment(page, Fragment::new(f.id(), atoms, f.bonds().to_vec())?)?;
    cases.push(("atom reorder", d));
    let mut bonds = f.bonds().to_vec();
    let (begin, end) = (bonds[0].begin, bonds[0].end);
    bonds[0].begin = end;
    bonds[0].end = begin;
    let mut d = base.model().clone();
    d.replace_fragment(page, Fragment::new(f.id(), f.atoms().to_vec(), bonds)?)?;
    cases.push(("directed endpoint reversal", d));
    let mut atoms = f.atoms().to_vec();
    atoms[0].id = AtomId::new(1000).ok_or("atom")?;
    let mut bonds = f.bonds().to_vec();
    bonds[0].begin = atoms[0].id;
    let mut d = base.model().clone();
    d.replace_fragment(page, Fragment::new(f.id(), atoms, bonds)?)?;
    cases.push(("old atom identity replacement", d));
    let mut bonds = f.bonds().to_vec();
    bonds[0].id = BondId::new(1000).ok_or("bond")?;
    let mut d = base.model().clone();
    d.replace_fragment(page, Fragment::new(f.id(), f.atoms().to_vec(), bonds)?)?;
    cases.push(("old bond identity replacement", d));
    let mut d = base.model().clone();
    d.replace_fragment(
        page,
        Fragment::new(f.id(), vec![f.atoms()[0].clone()], vec![])?,
    )?;
    cases.push(("atom and bond deletion", d));
    cases.push(("fragment deletion", rebuilt(vec![f.clone()], size)?));
    cases.push((
        "fragment reorder",
        rebuilt(vec![other.clone(), f.clone()], size)?,
    ));
    cases.push((
        "old fragment identity replacement",
        rebuilt(
            vec![
                Fragment::new(
                    FragmentId::new(1000).ok_or("fragment")?,
                    f.atoms().to_vec(),
                    f.bonds().to_vec(),
                )?,
                other.clone(),
            ],
            size,
        )?,
    ));
    cases.push((
        "membership reparenting",
        rebuilt(
            vec![
                Fragment::new(f.id(), other.atoms().to_vec(), other.bonds().to_vec())?,
                Fragment::new(other.id(), f.atoms().to_vec(), f.bonds().to_vec())?,
            ],
            size,
        )?,
    ));
    cases.push((
        "new fragment before old prefix",
        rebuilt(
            vec![
                Fragment::new(FragmentId::new(1000).ok_or("fragment")?, vec![], vec![])?,
                f.clone(),
                other.clone(),
            ],
            size,
        )?,
    ));
    let added = Atom::new(
        AtomId::new(1000).ok_or("atom")?,
        Element::new(6).ok_or("carbon")?,
        Some(Point2::new(180.0, 200.0)?),
    );
    let mut atoms = f.atoms().to_vec();
    atoms.insert(0, added.clone());
    let mut d = base.model().clone();
    d.replace_fragment(page, Fragment::new(f.id(), atoms, f.bonds().to_vec())?)?;
    cases.push(("new atom before old prefix", d));
    let mut atoms = f.atoms().to_vec();
    atoms.push(added);
    let mut bonds = f.bonds().to_vec();
    let mut added_bond = bonds[0];
    added_bond.id = BondId::new(1000).ok_or("bond")?;
    added_bond.begin = atoms[1].id;
    added_bond.end = atoms[2].id;
    bonds.insert(0, added_bond);
    let mut d = base.model().clone();
    d.replace_fragment(page, Fragment::new(f.id(), atoms, bonds)?)?;
    cases.push(("new bond before old prefix", d));
    for (label, model) in cases {
        let mut stored = base.clone();
        *stored.model_mut() = model;
        let snapshot = stored.model_snapshot();
        let json = serde_json::to_vec(stored.model())?;
        assert!(!stored.is_fully_mapped(), "{label}");
        assert!(
            matches!(stored.to_cdxml(), Err(DocumentError::UnsupportedEdit)),
            "{label}"
        );
        assert!(Arc::ptr_eq(&snapshot, &stored.model_snapshot()));
        assert_eq!(serde_json::to_vec(stored.model())?, json);
        assert_eq!(base.to_cdxml()?.as_ref(), SOURCE.as_bytes());
    }
    Ok(())
}

#[test]
fn preserved_edit_authority_does_not_authorize_whole_mol_metadata_loss() -> TestResult {
    const ORIGINAL: &[u8] = include_bytes!("fixtures/bondline-v0.cdxml");
    let mut canonical = StoredDocument::from_cdxml(ORIGINAL.into(), PageSize::A4)?;
    assert!(canonical.is_fully_mapped());
    let page = canonical.model().pages()[0].id();
    canonical.model_mut().set_page_size(page, PageSize::A4)?;
    let preserved_source = std::str::from_utf8(ORIGINAL)?.replace('"', "'");
    let mut preserved =
        StoredDocument::from_cdxml(preserved_source.as_bytes().into(), PageSize::A4)?;
    assert!(preserved.is_fully_mapped());
    preserved.model_mut().set_page_size(page, PageSize::A4)?;
    let frame = MolTransform::new(24.0, Point2::new(72.0, 72.0)?)?;
    for version in [Version::V2000, Version::V3000] {
        assert!(
            canonical.to_mol_as(version, frame).is_ok(),
            "canonical control has one A4 page, no settings/flags and supported C2 chemistry"
        );
        assert!(matches!(
            preserved.to_mol_as(version, frame),
            Err(DocumentError::CrossFormatSource {
                from: "cdxml",
                to: "mol"
            })
        ));
    }
    assert!(preserved.to_cdxml().is_ok());
    Ok(())
}
