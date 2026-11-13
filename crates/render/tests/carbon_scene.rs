use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Document, Element, Fragment, FragmentId, PageId,
    PageSize, Point2, Radical,
};
use bondline_render::{Error, PageViewport, carbon_page_scene, cpu_scene, gpu_scene, page_scene};

fn document(
    mutator: impl FnOnce(&mut Atom, &mut Bond),
) -> Result<Document, Box<dyn std::error::Error>> {
    let a = AtomId::new(1).ok_or("atom id")?;
    let b = AtomId::new(2).ok_or("atom id")?;
    let carbon = Element::new(6).ok_or("carbon")?;
    let mut first = Atom::new(a, carbon, Some(Point2::new(10.0, 20.5)?));
    let mut second = Atom::new(b, carbon, Some(Point2::new(50.0, 20.5)?));
    let mut bond = Bond {
        id: BondId::new(1).ok_or("bond id")?,
        begin: a,
        end: b,
        order: BondOrder::Single,
    };
    first.explicit_hydrogens = Some(3);
    second.explicit_hydrogens = Some(3);
    mutator(&mut first, &mut bond);
    let fragment = Fragment::new(
        FragmentId::new(1).ok_or("fragment id")?,
        vec![first, second],
        vec![bond],
    )?;
    let mut document = Document::default();
    let page = PageId::new(1).ok_or("page id")?;
    document.set_page_size(page, PageSize::new(60.0, 60.0)?)?;
    document.append_fragments(page, vec![fragment])?;
    Ok(document)
}

#[test]
fn carbon_single_bond_positions_map_from_points_to_exact_cpu_gpu_pixels()
-> Result<(), Box<dyn std::error::Error>> {
    let document = document(|_, _| {})?;
    let page = document.pages().first().ok_or("page")?;
    let before = document.clone();
    let scene = carbon_page_scene(80, 80, page)?;
    let baseline = cpu_scene(&page_scene(80, 80, page.size())?)?;
    let cpu = cpu_scene(&scene)?;
    for y in 0..80 {
        for x in 0..80 {
            let offset = ((y * 80 + x) * 4) as usize;
            // FontTools bounds for 12-em C/H plus 8-em subscript 3, with one-pixel masks.
            if (25..39).contains(&y) && ((2..25).contains(&x) || (55..78).contains(&x)) {
                assert!(
                    [&[0, 0, 0, 255][..], &[255; 4][..]].contains(&&cpu.rgba[offset..offset + 4])
                );
                continue;
            }
            let expected = if y == 30 && (25..55).contains(&x) {
                &[0, 0, 0, 255]
            } else {
                &baseline.rgba[offset..offset + 4]
            };
            assert_eq!(&cpu.rgba[offset..offset + 4], expected, "pixel {x},{y}");
        }
    }
    for region in [2..25, 55..78] {
        let ink = (25..39)
            .flat_map(|y| region.clone().map(move |x| (y * 80 + x) * 4))
            .filter(|offset| cpu.rgba.get(*offset..*offset + 4) == Some(&[0, 0, 0, 255]))
            .count();
        assert!(ink > 25, "missing terminal label: {ink} ink pixels");
    }
    assert_eq!(pollster::block_on(gpu_scene(&scene))?.0, cpu);
    assert_eq!(document, before);
    let viewport = PageViewport::new(80, 80, page.size())?;
    let point = Point2::new(10.0, 20.5)?;
    assert_eq!(viewport.canvas_position(point)?, [20.0, 30.5]);
    assert_eq!(viewport.document_position([20.0, 30.5])?, point);
    assert!(viewport.contains([20.0, 30.5]));
    assert!(!viewport.contains([0.0, 0.0]));
    assert!(viewport.document_position([f32::NAN, 0.0]).is_err());
    Ok(())
}

#[test]
fn unsupported_atom_metadata_and_bond_kinds_fail_instead_of_being_hidden()
-> Result<(), Box<dyn std::error::Error>> {
    for kind in 0..7 {
        let document = document(|atom, bond| match kind {
            0 => atom.element = Element::new(7).unwrap_or(atom.element),
            1 => atom.position = None,
            2 => atom.charge = 1,
            3 => atom.isotope = std::num::NonZeroU16::new(13),
            4 => atom.radical = Radical::Doublet,
            5 => atom.explicit_hydrogens = Some(2),
            _ => bond.order = BondOrder::Double,
        })?;
        assert!(matches!(
            carbon_page_scene(80, 80, document.pages().first().ok_or("page")?),
            Err(Error::UnsupportedChemicalContent)
        ));
    }
    let mut isolated = Document::default();
    let page = PageId::new(1).ok_or("page")?;
    let atom = Atom::new(
        AtomId::new(1).ok_or("atom")?,
        Element::new(6).ok_or("carbon")?,
        Some(Point2::new(1.0, 1.0)?),
    );
    isolated.append_fragments(
        page,
        vec![Fragment::new(
            FragmentId::new(1).ok_or("fragment")?,
            vec![atom],
            vec![],
        )?],
    )?;
    assert!(matches!(
        carbon_page_scene(80, 80, isolated.pages().first().ok_or("page")?),
        Err(Error::UnsupportedChemicalContent)
    ));
    Ok(())
}
