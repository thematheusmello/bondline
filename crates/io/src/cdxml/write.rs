use super::MappingError;
use bondline_cdxml::Writer;
use bondline_model::{Atom, AtomId, Bond, BondOrder, Document, DoublePosition, Page, Radical};
use std::collections::BTreeMap;

pub(super) type Attributes = Vec<(&'static str, String)>;

pub(super) fn root_attributes(document: &Document) -> Attributes {
    let settings = document.drawing_settings();
    let mut attributes = vec![(
        "CreationProgram",
        concat!("Bondline ", env!("CARGO_PKG_VERSION")).to_owned(),
    )];
    if let Some(value) = settings.bond_length_pt() {
        attributes.push(("BondLength", value.to_string()));
    }
    if let Some(value) = settings.chain_angle_degrees() {
        attributes.push(("ChainAngle", value.to_string()));
    }
    if let Some(value) = document.drawing_style().line_width_pt() {
        attributes.push(("LineWidth", value.to_string()));
    }
    if let Some(value) = document.drawing_style().bond_spacing_percent() {
        attributes.push(("BondSpacing", value.to_string()));
    }
    if let Some(value) = document.drawing_style().margin_width_pt() {
        attributes.push(("MarginWidth", value.to_string()));
    }
    attributes
}

pub(super) fn page_attributes(page: &Page, id: u32) -> Attributes {
    vec![
        ("id", id.to_string()),
        ("DrawingSpace", "Poster".to_owned()),
        ("Width", page.size().width_pt().to_string()),
        ("Height", page.size().height_pt().to_string()),
    ]
}

pub(super) fn borrowed(attributes: &Attributes) -> Vec<(&'static str, &str)> {
    attributes
        .iter()
        .map(|(name, value)| (*name, value.as_str()))
        .collect()
}

/// New elemental CDXML only. Never regenerate a source-backed projection with this API.
/// Canvas dimensions are explicit Poster extents, not native printer configuration.
pub fn write_document(document: &Document) -> Result<Vec<u8>, MappingError> {
    let attributes = root_attributes(document);
    let mut writer = Writer::new(&borrowed(&attributes))?;
    let resources = super::caption::NativeResources::new(document)?;
    resources.write(&mut writer)?;
    let mut next = 0_u32;
    for page in document.pages() {
        let attributes = page_attributes(page, allocate(&mut next)?);
        writer.start_element("page", &borrowed(&attributes))?;
        for fragment in page.fragments() {
            writer.start_element("fragment", &[("id", &allocate(&mut next)?.to_string())])?;
            let mut ids = BTreeMap::<AtomId, u32>::new();
            for atom in fragment.atoms() {
                let id = allocate(&mut next)?;
                if let Some(label) = &atom.authored_label {
                    writer.start_element("n", &borrowed(&atom_attributes(atom, id)?))?;
                    super::atom_text::write(&mut writer, &resources, label, allocate(&mut next)?)?;
                    writer.end_element()?;
                } else {
                    write_atom(&mut writer, atom, id)?;
                }
                ids.insert(atom.id, id);
            }
            for bond in fragment.bonds() {
                let begin = *ids
                    .get(&bond.begin)
                    .ok_or(MappingError::UnresolvedReference)?;
                let end = *ids
                    .get(&bond.end)
                    .ok_or(MappingError::UnresolvedReference)?;
                write_bond(&mut writer, bond, allocate(&mut next)?, begin, end)?;
            }
            writer.end_element()?;
        }
        for text in page.texts() {
            resources.write_text(&mut writer, text, allocate(&mut next)?)?;
        }
        writer.end_element()?;
    }
    Ok(writer.finish()?)
}

fn allocate(counter: &mut u32) -> Result<u32, MappingError> {
    *counter = counter.checked_add(1).ok_or(MappingError::IdentityLimit)?;
    Ok(*counter)
}

pub(super) fn atom_attributes(atom: &Atom, id: u32) -> Result<Attributes, MappingError> {
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
    if let Some(show) = atom.show_terminal_carbon_labels {
        attributes.push((
            "ShowTerminalCarbonLabels",
            if show { "yes" } else { "no" }.to_owned(),
        ));
    }
    if let Some(show) = atom.show_non_terminal_carbon_labels {
        attributes.push((
            "ShowNonTerminalCarbonLabels",
            if show { "yes" } else { "no" }.to_owned(),
        ));
    }
    Ok(attributes)
}

pub(super) fn bond_attributes(bond: &Bond, id: u32, begin: u32, end: u32) -> Attributes {
    let mut attributes = vec![
        ("id", id.to_string()),
        ("B", begin.to_string()),
        ("E", end.to_string()),
        ("Order", order(bond.order).to_owned()),
    ];
    let position = match bond.double_position {
        DoublePosition::Unspecified => None,
        DoublePosition::Center => Some("Center"),
        DoublePosition::Right => Some("Right"),
        DoublePosition::Left => Some("Left"),
    };
    if let Some(position) = position {
        attributes.push(("DoublePosition", position.to_owned()));
    }
    attributes
}

pub(super) fn write_atom(writer: &mut Writer, atom: &Atom, id: u32) -> Result<(), MappingError> {
    if atom.authored_label.is_some() {
        return Err(MappingError::InvalidProperty("authored node text"));
    }
    writer.empty_element("n", &borrowed(&atom_attributes(atom, id)?))?;
    Ok(())
}

pub(super) fn write_bond(
    writer: &mut Writer,
    bond: &Bond,
    id: u32,
    begin: u32,
    end: u32,
) -> Result<(), MappingError> {
    writer.empty_element("b", &borrowed(&bond_attributes(bond, id, begin, end)))?;
    Ok(())
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
