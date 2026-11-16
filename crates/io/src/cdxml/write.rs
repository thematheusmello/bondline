use super::MappingError;
use bondline_cdxml::Writer;
use bondline_model::{Atom, AtomId, BondOrder, Document, DoublePosition, Radical};
use std::collections::BTreeMap;

/// New elemental CDXML only. Never regenerate a source-backed projection with this API.
/// Canvas dimensions are explicit Poster extents, not native printer configuration.
pub fn write_document(document: &Document) -> Result<Vec<u8>, MappingError> {
    let mut writer = Writer::new(&[(
        "CreationProgram",
        concat!("Bondline ", env!("CARGO_PKG_VERSION")),
    )])?;
    let mut next = 0_u32;
    for page in document.pages() {
        let id = allocate(&mut next)?.to_string();
        let width = page.size().width_pt().to_string();
        let height = page.size().height_pt().to_string();
        writer.start_element(
            "page",
            &[
                ("id", &id),
                ("DrawingSpace", "Poster"),
                ("Width", &width),
                ("Height", &height),
            ],
        )?;
        for fragment in page.fragments() {
            writer.start_element("fragment", &[("id", &allocate(&mut next)?.to_string())])?;
            let mut ids = BTreeMap::<AtomId, u32>::new();
            for atom in fragment.atoms() {
                let id = allocate(&mut next)?;
                write_atom(&mut writer, atom, id)?;
                ids.insert(atom.id, id);
            }
            for bond in fragment.bonds() {
                let begin = ids
                    .get(&bond.begin)
                    .ok_or(MappingError::UnresolvedReference)?
                    .to_string();
                let end = ids
                    .get(&bond.end)
                    .ok_or(MappingError::UnresolvedReference)?
                    .to_string();
                let id = allocate(&mut next)?.to_string();
                let mut attributes = vec![
                    ("id", id.as_str()),
                    ("B", begin.as_str()),
                    ("E", end.as_str()),
                    ("Order", order(bond.order)),
                ];
                let position = match bond.double_position {
                    DoublePosition::Unspecified => None,
                    DoublePosition::Center => Some("Center"),
                    DoublePosition::Right => Some("Right"),
                    DoublePosition::Left => Some("Left"),
                };
                if let Some(position) = position {
                    attributes.push(("DoublePosition", position));
                }
                writer.empty_element("b", &attributes)?;
            }
            writer.end_element()?;
        }
        writer.end_element()?;
    }
    Ok(writer.finish()?)
}

fn allocate(counter: &mut u32) -> Result<u32, MappingError> {
    *counter = counter.checked_add(1).ok_or(MappingError::IdentityLimit)?;
    Ok(*counter)
}

fn write_atom(writer: &mut Writer, atom: &Atom, id: u32) -> Result<(), MappingError> {
    let mut attributes = vec![
        ("id", id.to_string()),
        ("NodeType", "Element".to_owned()),
        ("Element", atom.element.get().to_string()),
        ("Charge", atom.charge.to_string()),
        (
            "Radical",
            match atom.radical {
                Radical::None => "None",
                Radical::Singlet => "Singlet",
                Radical::Doublet => "Doublet",
                Radical::Triplet => "Triplet",
            }
            .to_owned(),
        ),
    ];
    if let Some(position) = atom.position {
        attributes.push(("p", format!("{} {}", position.x_pt(), position.y_pt())));
    }
    if let Some(isotope) = atom.isotope {
        if isotope.get() > i16::MAX as u16 {
            return Err(MappingError::InvalidProperty("Isotope"));
        }
        attributes.push(("Isotope", isotope.to_string()));
    }
    if let Some(hydrogens) = atom.explicit_hydrogens {
        attributes.push(("NumHydrogens", hydrogens.to_string()));
    }
    let borrowed: Vec<_> = attributes
        .iter()
        .map(|(key, value)| (*key, value.as_str()))
        .collect();
    Ok(writer.empty_element("n", &borrowed)?)
}

fn order(order: BondOrder) -> &'static str {
    match order {
        BondOrder::Single => "1",
        BondOrder::Double => "2",
        BondOrder::Triple => "3",
        BondOrder::Quadruple => "4",
        BondOrder::Quintuple => "5",
        BondOrder::Sextuple => "6",
        BondOrder::Half => "0.5",
        BondOrder::Aromatic => "1.5",
        BondOrder::TwoAndHalf => "2.5",
        BondOrder::ThreeAndHalf => "3.5",
        BondOrder::FourAndHalf => "4.5",
        BondOrder::FiveAndHalf => "5.5",
        BondOrder::Dative => "dative",
        BondOrder::Ionic => "ionic",
        BondOrder::Hydrogen => "hydrogen",
        BondOrder::ThreeCenter => "threecenter",
    }
}
