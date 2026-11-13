use super::super::MolTransform;
use super::{
    Error, MAX_BYTES, MAX_ITEMS, MAX_LINES, MAX_LOGICAL_BYTES, MAX_PHYSICAL_LINE, MAX_SEGMENTS,
};
use bondline_model::{AtomId, BondOrder, Fragment, Radical};
use std::collections::BTreeMap;

struct Output {
    bytes: Vec<u8>,
    lines: usize,
}
impl Output {
    fn physical(&mut self, line: &str) -> Result<(), Error> {
        if line.len() > MAX_PHYSICAL_LINE {
            return Err(Error::Resource(
                "physical output line exceeds 80 ASCII characters",
            ));
        }
        self.lines = self
            .lines
            .checked_add(1)
            .filter(|value| *value <= MAX_LINES)
            .ok_or(Error::Resource("more than 1000000 output lines"))?;
        if self
            .bytes
            .len()
            .checked_add(line.len())
            .and_then(|value| value.checked_add(1))
            .filter(|value| *value <= MAX_BYTES)
            .is_none()
        {
            return Err(Error::Resource("output exceeds 64 MiB"));
        }
        self.bytes.extend_from_slice(line.as_bytes());
        self.bytes.push(b'\n');
        Ok(())
    }
    fn entry(&mut self, logical: &str) -> Result<(), Error> {
        if logical.len() > MAX_LOGICAL_BYTES {
            return Err(Error::Resource("logical output entry exceeds 65536 bytes"));
        }
        let mut offset = 0;
        let mut segments = 0_usize;
        loop {
            segments = segments
                .checked_add(1)
                .filter(|value| *value <= MAX_SEGMENTS)
                .ok_or(Error::Resource("output continuation segments"))?;
            let remaining = logical
                .get(offset..)
                .ok_or(Error::Resource("output span"))?;
            if remaining.len() <= 73 {
                self.physical(&format!("M  V30 {remaining}"))?;
                break;
            }
            let end = offset
                .checked_add(72)
                .ok_or(Error::Resource("output span"))?;
            let chunk = logical
                .get(offset..end)
                .ok_or(Error::Resource("output span"))?;
            self.physical(&format!("M  V30 {chunk}-"))?;
            offset = end;
        }
        Ok(())
    }
}

/// Encode only a supported authored graph. Finite coordinates use Rust f64 Display's
/// shortest round-trip decimal; no V2000 four-decimal quantization is applied.
/// Frame arithmetic may round: compare reconstructed drawing points within
/// 16*EPSILON*max(1,|point|,|origin|,|raw*scale|), not bit identity across frames.
/// Unknown original source must use MolFile::as_bytes, never partial regeneration.
pub fn write_v3000(
    fragment: &Fragment,
    title: &str,
    frame: MolTransform,
) -> Result<Vec<u8>, Error> {
    write_v3000_with_header(fragment, [title, "  Bondline          2D", ""], frame)
}
pub fn write_v3000_with_header(
    fragment: &Fragment,
    header: [&str; 3],
    frame: MolTransform,
) -> Result<Vec<u8>, Error> {
    if fragment.atoms().len() > MAX_ITEMS || fragment.bonds().len() > MAX_ITEMS {
        return Err(Error::Resource("more than 100000 atoms/bonds"));
    }
    for (index, line) in header.iter().enumerate() {
        if line.len() > 80 || !line.bytes().all(|byte| matches!(byte, b' '..=b'~')) {
            return Err(Error::Syntax {
                line: index + 1,
                field: "ASCII header at most 80 characters",
            });
        }
    }
    if ["$MDL", "$$$$", "$RXN", "$RDFILE"]
        .iter()
        .any(|tag| header.first().is_some_and(|title| title.contains(tag)))
    {
        return Err(Error::Syntax {
            line: 1,
            field: "reserved title",
        });
    }
    if header
        .get(1)
        .and_then(|value| value.get(20..22))
        .is_some_and(|value| value == "3D")
    {
        return Err(Error::Unsupported {
            line: 2,
            feature: "3D dimensional header",
        });
    }
    let mut ids = BTreeMap::<AtomId, usize>::new();
    let mut totals = BTreeMap::<AtomId, u32>::new();
    for (index, atom) in fragment.atoms().iter().enumerate() {
        ids.insert(atom.id, index + 1);
        totals.insert(atom.id, 0);
    }
    let mut orders = Vec::with_capacity(fragment.bonds().len());
    for bond in fragment.bonds() {
        let order = match bond.order {
            BondOrder::Single => 1_u8,
            BondOrder::Double => 2,
            BondOrder::Triple => 3,
            _ => {
                return Err(Error::Unsupported {
                    line: 0,
                    feature: "query or extended bond type",
                });
            }
        };
        for id in [bond.begin, bond.end] {
            let total = totals.get_mut(&id).ok_or(Error::Syntax {
                line: 0,
                field: "bond endpoint",
            })?;
            *total = total
                .checked_add(u32::from(order))
                .ok_or(Error::Resource("bond-order sum"))?;
        }
        orders.push(order);
    }
    let mut output = Output {
        bytes: Vec::new(),
        lines: 0,
    };
    for line in header {
        output.physical(line)?;
    }
    output.physical("  0  0  0  0  0  0            999 V3000")?;
    output.entry("BEGIN CTAB")?;
    output.entry(&format!(
        "COUNTS {} {} 0 0 0",
        fragment.atoms().len(),
        fragment.bonds().len()
    ))?;
    output.entry("BEGIN ATOM")?;
    for (index, atom) in fragment.atoms().iter().enumerate() {
        let symbol = super::super::symbols::symbol(atom.element).ok_or(Error::Unsupported {
            line: 0,
            feature: "element symbol",
        })?;
        let position = atom.position.ok_or(Error::Unsupported {
            line: 0,
            feature: "missing atom position",
        })?;
        let (x, y) = frame.export(position);
        if !x.is_finite() || !y.is_finite() {
            return Err(Error::CoordinateRange);
        }
        let mut row = format!("{} {symbol} {x} {y} 0 0", index + 1);
        if !(-15..=15).contains(&atom.charge) {
            return Err(Error::Unsupported {
                line: 0,
                feature: "charge outside -15..15",
            });
        }
        if atom.charge != 0 {
            row.push_str(&format!(" CHG={}", atom.charge));
        }
        let radical = match atom.radical {
            Radical::None => 0,
            Radical::Singlet => 1,
            Radical::Doublet => 2,
            Radical::Triplet => 3,
        };
        if radical != 0 {
            row.push_str(&format!(" RAD={radical}"));
        }
        if let Some(isotope) = atom.isotope {
            row.push_str(&format!(" MASS={}", isotope.get()));
        }
        if let Some(hydrogens) = atom.explicit_hydrogens {
            let valence = totals
                .get(&atom.id)
                .copied()
                .ok_or(Error::Syntax {
                    line: 0,
                    field: "atom ID",
                })?
                .checked_add(u32::from(hydrogens))
                .filter(|value| *value <= 14)
                .ok_or(Error::Unsupported {
                    line: 0,
                    feature: "explicit H needs declared valence above 14",
                })?;
            let value = if valence == 0 {
                -1
            } else {
                i32::try_from(valence).map_err(|_| Error::Resource("valence"))?
            };
            row.push_str(&format!(" VAL={value}"));
        }
        output.entry(&row)?;
    }
    output.entry("END ATOM")?;
    if !fragment.bonds().is_empty() {
        output.entry("BEGIN BOND")?;
        for (index, (bond, order)) in fragment.bonds().iter().zip(orders).enumerate() {
            let begin = ids.get(&bond.begin).ok_or(Error::Syntax {
                line: 0,
                field: "bond begin",
            })?;
            let end = ids.get(&bond.end).ok_or(Error::Syntax {
                line: 0,
                field: "bond end",
            })?;
            output.entry(&format!("{} {order} {begin} {end}", index + 1))?;
        }
        output.entry("END BOND")?;
    }
    output.entry("END CTAB")?;
    output.physical("M  END")?;
    Ok(output.bytes)
}
