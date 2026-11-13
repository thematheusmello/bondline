use super::{
    MAX_MOL_BYTES, MAX_MOL_ITEMS, MAX_MOL_LINE_BYTES, MAX_MOL_LINES, MolAtom, MolBond, MolError,
    MolFile, UnsupportedFeature,
};
use bondline_model::Radical;
use std::{collections::BTreeSet, num::NonZeroU16};

struct Lines<'a> {
    iterator: std::str::SplitTerminator<'a, char>,
    line: usize,
}
impl<'a> Lines<'a> {
    fn next(&mut self) -> Result<Option<(usize, &'a str)>, MolError> {
        let Some(value) = self.iterator.next() else {
            return Ok(None);
        };
        self.line = self.line.checked_add(1).ok_or(MolError::LineLimit)?;
        if self.line > MAX_MOL_LINES {
            return Err(MolError::LineLimit);
        }
        let value = value.strip_suffix('\r').unwrap_or(value);
        if value.len() > MAX_MOL_LINE_BYTES {
            return Err(MolError::LineTooLong { line: self.line });
        }
        if value.contains('\r') {
            return Err(MolError::InvalidEncoding);
        }
        Ok(Some((self.line, value)))
    }
    fn required(&mut self, field: &'static str) -> Result<(usize, &'a str), MolError> {
        self.next()?.ok_or(MolError::Truncated {
            line: self.line.saturating_add(1),
            field,
        })
    }
}

fn field<'a>(
    value: &'a str,
    start: usize,
    end: usize,
    line: usize,
    name: &'static str,
) -> Result<&'a str, MolError> {
    value
        .get(start..end)
        .ok_or(MolError::Truncated { line, field: name })
}
fn optional(value: &str, start: usize, end: usize) -> &str {
    value
        .get(start..end)
        .or_else(|| value.get(start..))
        .unwrap_or("")
}
fn number(value: &str, line: usize, name: &'static str) -> Result<i32, MolError> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(0);
    }
    let digits = value.strip_prefix(['-', '+']).unwrap_or(value);
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(MolError::InvalidField { line, field: name });
    }
    value
        .parse()
        .map_err(|_| MolError::InvalidField { line, field: name })
}
fn integer(
    value: &str,
    start: usize,
    end: usize,
    line: usize,
    name: &'static str,
) -> Result<i32, MolError> {
    number(optional(value, start, end), line, name)
}
fn real(value: &str, line: usize) -> Result<f64, MolError> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(0.0);
    }
    let unsigned = value.strip_prefix(['-', '+']).unwrap_or(value);
    if unsigned.is_empty()
        || unsigned.bytes().filter(|byte| *byte == b'.').count() > 1
        || !unsigned.bytes().any(|byte| byte.is_ascii_digit())
        || !unsigned
            .bytes()
            .all(|byte| byte.is_ascii_digit() || byte == b'.')
    {
        return Err(MolError::InvalidField {
            line,
            field: "coordinate",
        });
    }
    let parsed: f64 = value.parse().map_err(|_| MolError::InvalidField {
        line,
        field: "coordinate",
    })?;
    if !parsed.is_finite() {
        return Err(MolError::InvalidField {
            line,
            field: "coordinate",
        });
    }
    Ok(parsed)
}
fn unsupported(output: &mut Vec<UnsupportedFeature>, line: usize, feature: &'static str) {
    output.push(UnsupportedFeature { line, feature });
}

impl<'a> MolFile<'a> {
    pub fn parse(source: &'a [u8]) -> Result<Self, MolError> {
        if source.len() > MAX_MOL_BYTES {
            return Err(MolError::InputTooLarge);
        }
        if !source
            .iter()
            .all(|byte| matches!(byte, b'\n' | b'\r' | b' '..=b'~'))
        {
            return Err(MolError::InvalidEncoding);
        }
        let text = std::str::from_utf8(source).map_err(|_| MolError::InvalidEncoding)?;
        let mut lines = Lines {
            iterator: text.split_terminator('\n'),
            line: 0,
        };
        let (_, title) = lines.required("title header")?;
        let (_, program) = lines.required("program header")?;
        let (_, comment) = lines.required("comment header")?;
        if ["$MDL", "$RXN", "$RDFILE", "$$$$"]
            .iter()
            .any(|tag| title.contains(tag))
        {
            return Err(MolError::Unsupported {
                line: 1,
                feature: "non-MOL CTfile header",
            });
        }
        let (line, counts) = lines.required("counts line")?;
        if field(counts, 33, 39, line, "V2000 version")? != " V2000" {
            return Err(MolError::Unsupported {
                line,
                feature: "CTfile version other than V2000",
            });
        }
        let atom_count =
            usize::try_from(integer(counts, 0, 3, line, "atom count")?).map_err(|_| {
                MolError::InvalidField {
                    line,
                    field: "atom count",
                }
            })?;
        let bond_count =
            usize::try_from(integer(counts, 3, 6, line, "bond count")?).map_err(|_| {
                MolError::InvalidField {
                    line,
                    field: "bond count",
                }
            })?;
        if atom_count > MAX_MOL_ITEMS || bond_count > MAX_MOL_ITEMS {
            return Err(MolError::CountLimit);
        }
        let mut unmodeled = Vec::new();
        if optional(program, 20, 22) == "3D" {
            unsupported(&mut unmodeled, 2, "3D dimensional header");
        }
        for (start, name) in [
            (6, "atom lists"),
            (9, "obsolete counts field"),
            (12, "chiral flag"),
            (15, "obsolete counts field"),
            (18, "obsolete counts field"),
            (21, "obsolete counts field"),
            (24, "obsolete counts field"),
            (27, "obsolete counts field"),
        ] {
            if integer(counts, start, start + 3, line, name)? != 0 {
                unsupported(&mut unmodeled, line, name);
            }
        }
        let appendices = integer(counts, 30, 33, line, "property count")?;
        if appendices < 0 {
            return Err(MolError::InvalidField {
                line,
                field: "property count",
            });
        }
        if !optional(counts, 39, counts.len()).trim().is_empty() {
            unsupported(&mut unmodeled, line, "counts extension");
        }
        let mut atoms = Vec::with_capacity(atom_count);
        for _ in 0..atom_count {
            let (line, record) = lines.required("atom record")?;
            let symbol = field(record, 31, 34, line, "element symbol")?.trim();
            if symbol.is_empty() {
                return Err(MolError::InvalidField {
                    line,
                    field: "element symbol",
                });
            }
            let mass = integer(record, 34, 36, line, "mass difference")?;
            if !(-3..=4).contains(&mass) {
                return Err(MolError::InvalidField {
                    line,
                    field: "mass difference",
                });
            }
            let charge = integer(record, 36, 39, line, "atom charge code")?;
            if !(0..=7).contains(&charge) {
                return Err(MolError::InvalidField {
                    line,
                    field: "atom charge code",
                });
            }
            for (start, name) in [
                (39, "atom stereo parity"),
                (42, "hydrogen query"),
                (45, "stereo care"),
                (51, "H0 designator"),
                (54, "unused atom field"),
                (57, "unused atom field"),
                (60, "atom mapping"),
                (63, "inversion flag"),
                (66, "exact change flag"),
            ] {
                if integer(record, start, start + 3, line, name)? != 0 {
                    unsupported(&mut unmodeled, line, name);
                }
            }
            if !optional(record, 69, record.len()).trim().is_empty() {
                unsupported(&mut unmodeled, line, "atom extension");
            }
            let valence = integer(record, 48, 51, line, "declared valence")?;
            let declared_valence = match valence {
                0 => None,
                15 => Some(0),
                1..=14 => Some(u8::try_from(valence).map_err(|_| MolError::InvalidField {
                    line,
                    field: "declared valence",
                })?),
                _ => {
                    return Err(MolError::InvalidField {
                        line,
                        field: "declared valence",
                    });
                }
            };
            atoms.push(MolAtom {
                symbol,
                element: super::symbols::element(symbol),
                x: real(field(record, 0, 10, line, "x coordinate")?, line)?,
                y: real(field(record, 10, 20, line, "y coordinate")?, line)?,
                z: real(field(record, 20, 30, line, "z coordinate")?, line)?,
                mass_difference: i8::try_from(mass).map_err(|_| MolError::InvalidField {
                    line,
                    field: "mass difference",
                })?,
                isotope: None,
                charge: match charge {
                    1 => 3,
                    2 => 2,
                    3 => 1,
                    5 => -1,
                    6 => -2,
                    7 => -3,
                    _ => 0,
                },
                radical: if charge == 4 {
                    Radical::Doublet
                } else {
                    Radical::None
                },
                declared_valence,
                line,
            });
        }
        let mut bonds = Vec::with_capacity(bond_count);
        for _ in 0..bond_count {
            let (line, record) = lines.required("bond record")?;
            field(record, 0, 9, line, "bond endpoints/order")?;
            let begin = integer(record, 0, 3, line, "bond begin")?;
            let end = integer(record, 3, 6, line, "bond end")?;
            let order = integer(record, 6, 9, line, "bond order")?;
            if begin < 1
                || end < 1
                || usize::try_from(begin).ok().is_none_or(|n| n > atom_count)
                || usize::try_from(end).ok().is_none_or(|n| n > atom_count)
            {
                return Err(MolError::InvalidField {
                    line,
                    field: "bond endpoint",
                });
            }
            if !(1..=8).contains(&order) {
                return Err(MolError::InvalidField {
                    line,
                    field: "bond order",
                });
            }
            if order > 4 {
                unsupported(&mut unmodeled, line, "query bond order");
            }
            for (start, name) in [
                (9, "bond stereochemistry"),
                (12, "unused bond field"),
                (15, "bond topology"),
                (18, "reaction center"),
            ] {
                if integer(record, start, start + 3, line, name)? != 0 {
                    unsupported(&mut unmodeled, line, name);
                }
            }
            if !optional(record, 21, record.len()).trim().is_empty() {
                unsupported(&mut unmodeled, line, "bond extension");
            }
            bonds.push(MolBond {
                begin: u16::try_from(begin).map_err(|_| MolError::InvalidField {
                    line,
                    field: "bond begin",
                })?,
                end: u16::try_from(end).map_err(|_| MolError::InvalidField {
                    line,
                    field: "bond end",
                })?,
                order: u8::try_from(order).map_err(|_| MolError::InvalidField {
                    line,
                    field: "bond order",
                })?,
                line,
            });
        }
        let mut charges = Vec::new();
        let mut radicals = Vec::new();
        let mut isotopes = Vec::new();
        let mut charge_ids = BTreeSet::new();
        let mut radical_ids = BTreeSet::new();
        let mut isotope_ids = BTreeSet::new();
        let mut charge_override = false;
        let mut isotope_override = false;
        let mut ended = false;
        while let Some((line, record)) = lines.next()? {
            if record.trim_end() == "M  END" {
                ended = true;
                break;
            }
            if record.trim().is_empty() {
                continue;
            }
            let kind = match record.get(..6) {
                Some("M  CHG") => 0,
                Some("M  RAD") => 1,
                Some("M  ISO") => 2,
                _ => {
                    unsupported(
                        &mut unmodeled,
                        line,
                        "unmapped property or legacy extension",
                    );
                    continue;
                }
            };
            let count = integer(record, 6, 9, line, "property entry count")?;
            if !(1..=8).contains(&count) {
                return Err(MolError::InvalidField {
                    line,
                    field: "property entry count",
                });
            }
            let count = usize::try_from(count).map_err(|_| MolError::InvalidField {
                line,
                field: "property entry count",
            })?;
            field(record, 0, 9 + count * 8, line, "property entries")?;
            if !optional(record, 9 + count * 8, record.len())
                .trim()
                .is_empty()
            {
                unsupported(&mut unmodeled, line, "property extension");
            }
            if kind == 2 {
                isotope_override = true;
            } else {
                charge_override = true;
            }
            for entry in 0..count {
                let start = 9 + entry * 8;
                if field(record, start, start + 1, line, "property separator")? != " "
                    || field(record, start + 4, start + 5, line, "property separator")? != " "
                {
                    return Err(MolError::InvalidField {
                        line,
                        field: "property separator",
                    });
                }
                let index = integer(record, start + 1, start + 4, line, "property atom index")?;
                if index < 1 || usize::try_from(index).ok().is_none_or(|n| n > atom_count) {
                    return Err(MolError::InvalidField {
                        line,
                        field: "property atom index",
                    });
                }
                let index = usize::try_from(index - 1).map_err(|_| MolError::InvalidField {
                    line,
                    field: "property atom index",
                })?;
                let value = integer(record, start + 5, start + 8, line, "property value")?;
                let ids = match kind {
                    0 => &mut charge_ids,
                    1 => &mut radical_ids,
                    _ => &mut isotope_ids,
                };
                if !ids.insert(index) {
                    return Err(MolError::InvalidField {
                        line,
                        field: "duplicate atom property",
                    });
                }
                match kind {
                    0 if (-15..=15).contains(&value) => charges.push((
                        index,
                        i8::try_from(value).map_err(|_| MolError::InvalidField {
                            line,
                            field: "charge",
                        })?,
                    )),
                    1 if (0..=3).contains(&value) => radicals.push((
                        index,
                        match value {
                            1 => Radical::Singlet,
                            2 => Radical::Doublet,
                            3 => Radical::Triplet,
                            _ => Radical::None,
                        },
                    )),
                    2 if (1..=999).contains(&value) => isotopes.push((
                        index,
                        NonZeroU16::new(u16::try_from(value).map_err(|_| {
                            MolError::InvalidField {
                                line,
                                field: "isotope",
                            }
                        })?),
                    )),
                    _ => {
                        return Err(MolError::InvalidField {
                            line,
                            field: "property value",
                        });
                    }
                }
            }
        }
        if !ended {
            return Err(MolError::MissingEnd);
        }
        while let Some((line, record)) = lines.next()? {
            if !record.trim().is_empty() {
                return Err(MolError::TrailingData { line });
            }
        }
        if charge_override {
            for atom in &mut atoms {
                atom.charge = 0;
                atom.radical = Radical::None;
            }
        }
        for (index, charge) in charges {
            if let Some(atom) = atoms.get_mut(index) {
                atom.charge = charge;
            }
        }
        for (index, radical) in radicals {
            if let Some(atom) = atoms.get_mut(index) {
                atom.radical = radical;
            }
        }
        for (index, isotope) in isotopes {
            if let Some(atom) = atoms.get_mut(index) {
                atom.isotope = isotope;
            }
        }
        let aromatic_endpoints: BTreeSet<_> = bonds
            .iter()
            .filter(|bond| bond.order == 4)
            .flat_map(|bond| [bond.begin, bond.end])
            .collect();
        for (index, atom) in atoms.iter().enumerate() {
            if atom.element.is_none() {
                unsupported(
                    &mut unmodeled,
                    atom.line,
                    "query or non-element atom symbol",
                );
            }
            if atom.z != 0.0 {
                unsupported(&mut unmodeled, atom.line, "3D coordinates");
            }
            if atom.mass_difference != 0 && !isotope_override {
                unsupported(
                    &mut unmodeled,
                    atom.line,
                    "isotope mass difference without a verified reference mass",
                );
            }
            if atom.declared_valence.is_some()
                && u16::try_from(index + 1)
                    .ok()
                    .is_some_and(|id| aromatic_endpoints.contains(&id))
            {
                unsupported(
                    &mut unmodeled,
                    atom.line,
                    "declared valence on aromatic bonds",
                );
            }
        }
        Ok(Self {
            source,
            header: [title, program, comment],
            atoms,
            bonds,
            unsupported: unmodeled,
            isotope_override,
        })
    }
}
