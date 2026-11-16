use super::{MAX_MOL_BYTES, MAX_MOL_ITEMS, MolError, MolTransform};
use bondline_model::{AtomId, BondOrder, Fragment, Radical};
use std::{collections::BTreeMap, fmt::Write};

fn coordinate(value: f64) -> Result<String, MolError> {
    if !value.is_finite() {
        return Err(MolError::CoordinateRange);
    }
    if value.abs() > 100_000.0 {
        return Err(MolError::CoordinateRange);
    }
    let output = format!("{value:>10.4}");
    if output.len() != 10 {
        return Err(MolError::CoordinateRange);
    }
    Ok(output)
}

/// Encode only the supported elemental graph. Coordinates round to the format's
/// four decimal places (at most 0.00005 raw units plus floating-point representation
/// error). Unchanged source passthrough remains byte-exact, without quantization.
/// Unknown source must be saved from MolFile::as_bytes, never from its projection.
pub fn write_v2000(
    fragment: &Fragment,
    title: &str,
    transform: MolTransform,
) -> Result<Vec<u8>, MolError> {
    write_v2000_with_header(fragment, [title, "  Bondline          2D", ""], transform)
}

/// Retain all three supplied source headers when encoding a supported edited graph.
/// This does not make projection-based export safe for unknown source properties.
pub fn write_v2000_with_header(
    fragment: &Fragment,
    header: [&str; 3],
    transform: MolTransform,
) -> Result<Vec<u8>, MolError> {
    if fragment.atoms().len() > MAX_MOL_ITEMS || fragment.bonds().len() > MAX_MOL_ITEMS {
        return Err(MolError::CountLimit);
    }
    for (index, line) in header.iter().enumerate() {
        if line.len() > 80 || !line.bytes().all(|byte| matches!(byte, b' '..=b'~')) {
            return Err(MolError::InvalidField {
                line: index + 1,
                field: "header",
            });
        }
    }
    if ["$MDL", "$RXN", "$RDFILE", "$$$$"]
        .iter()
        .any(|tag| header.first().is_some_and(|title| title.contains(tag)))
    {
        return Err(MolError::InvalidField {
            line: 1,
            field: "title",
        });
    }
    let mut ids = BTreeMap::<AtomId, usize>::new();
    let mut totals = BTreeMap::<AtomId, Option<u16>>::new();
    for (index, atom) in fragment.atoms().iter().enumerate() {
        if atom.show_non_terminal_carbon_labels.is_some() {
            return Err(MolError::Unsupported {
                line: 0,
                feature: "non-terminal carbon-label display metadata",
            });
        }
        ids.insert(atom.id, index + 1);
        totals.insert(atom.id, Some(0));
    }
    let mut orders = Vec::with_capacity(fragment.bonds().len());
    for bond in fragment.bonds() {
        if !bond.double_position.is_unspecified() {
            return Err(MolError::Unsupported {
                line: 0,
                feature: "double-bond placement metadata",
            });
        }
        let order = match bond.order {
            BondOrder::Single => 1_u8,
            BondOrder::Double => 2,
            BondOrder::Triple => 3,
            BondOrder::Aromatic => 4,
            _ => {
                return Err(MolError::Unsupported {
                    line: 0,
                    feature: "bond order outside V2000 elemental subset",
                });
            }
        };
        for endpoint in [bond.begin, bond.end] {
            let total = totals.get_mut(&endpoint).ok_or(MolError::InvalidField {
                line: 0,
                field: "bond endpoint",
            })?;
            *total = if order == 4 {
                None
            } else {
                total.and_then(|value| value.checked_add(u16::from(order)))
            };
        }
        orders.push(order);
    }
    let mut output = String::new();
    for line in header {
        writeln!(output, "{line}").map_err(|_| MolError::InputTooLarge)?;
    }
    writeln!(
        output,
        "{:>3}{:>3}  0  0  0  0            999 V2000",
        fragment.atoms().len(),
        fragment.bonds().len()
    )
    .map_err(|_| MolError::InputTooLarge)?;
    let mut charges = Vec::new();
    let mut radicals = Vec::new();
    let mut isotopes = Vec::new();
    for (index, atom) in fragment.atoms().iter().enumerate() {
        let position = atom.position.ok_or(MolError::Unsupported {
            line: 0,
            feature: "missing atom position",
        })?;
        let (x, y) = transform.export(position);
        let x = coordinate(x)?;
        let y = coordinate(y)?;
        let symbol = super::symbols::symbol(atom.element).ok_or(MolError::Unsupported {
            line: 0,
            feature: "element symbol",
        })?;
        let valence = match atom.explicit_hydrogens {
            None => 0,
            Some(hydrogens) => {
                let total = totals
                    .get(&atom.id)
                    .copied()
                    .flatten()
                    .ok_or(MolError::Unsupported {
                        line: 0,
                        feature: "explicit H count on aromatic bonds",
                    })?
                    .checked_add(hydrogens)
                    .ok_or(MolError::Unsupported {
                        line: 0,
                        feature: "declared valence overflow",
                    })?;
                match total {
                    0 => 15,
                    1..=14 => total,
                    _ => {
                        return Err(MolError::Unsupported {
                            line: 0,
                            feature: "declared valence above 14",
                        });
                    }
                }
            }
        };
        if !(-15..=15).contains(&atom.charge) {
            return Err(MolError::Unsupported {
                line: 0,
                feature: "charge outside -15..15",
            });
        }
        if atom.charge != 0 {
            charges.push((index + 1, i32::from(atom.charge)));
        }
        let radical = match atom.radical {
            Radical::None => 0,
            Radical::Singlet => 1,
            Radical::Doublet => 2,
            Radical::Triplet => 3,
        };
        if radical != 0 {
            radicals.push((index + 1, radical));
        }
        if let Some(isotope) = atom.isotope {
            if isotope.get() > 999 {
                return Err(MolError::Unsupported {
                    line: 0,
                    feature: "isotope mass above 999",
                });
            }
            isotopes.push((index + 1, i32::from(isotope.get())));
        }
        // Atom charge code duplicates only representable nonradical values. M CHG/RAD
        // below remain authoritative for simultaneous charge and radical information.
        let code = match atom.charge {
            3 => 1,
            2 => 2,
            1 => 3,
            -1 => 5,
            -2 => 6,
            -3 => 7,
            _ if atom.radical == Radical::Doublet => 4,
            _ => 0,
        };
        writeln!(
            output,
            "{x}{y}    0.0000 {symbol:<3} 0{code:>3}  0  0  0{valence:>3}  0  0  0  0  0  0"
        )
        .map_err(|_| MolError::InputTooLarge)?;
    }
    for (bond, order) in fragment.bonds().iter().zip(orders) {
        let begin = ids.get(&bond.begin).ok_or(MolError::InvalidField {
            line: 0,
            field: "bond begin",
        })?;
        let end = ids.get(&bond.end).ok_or(MolError::InvalidField {
            line: 0,
            field: "bond end",
        })?;
        writeln!(output, "{begin:>3}{end:>3}{order:>3}  0  0  0  0")
            .map_err(|_| MolError::InputTooLarge)?;
    }
    for (kind, values) in [("CHG", charges), ("RAD", radicals), ("ISO", isotopes)] {
        for chunk in values.chunks(8) {
            write!(output, "M  {kind}{:>3}", chunk.len()).map_err(|_| MolError::InputTooLarge)?;
            for (index, value) in chunk {
                write!(output, " {index:>3} {value:>3}").map_err(|_| MolError::InputTooLarge)?;
            }
            output.push('\n');
        }
    }
    output.push_str("M  END\n");
    if output.len() > MAX_MOL_BYTES {
        return Err(MolError::InputTooLarge);
    }
    Ok(output.into_bytes())
}
