use super::{
    Component, Counts, Error, MAX_BYTES, MAX_GRAPH_ITEMS, MAX_LINE_BYTES, MAX_LINES, Role, RxnFile,
};
use crate::mol::{MolFile, Version, v3000};
use std::ops::Range;

pub(super) struct Row<'a> {
    pub text: &'a str,
    pub range: Range<usize>,
    pub line: usize,
}
pub(super) struct Cursor<'a> {
    source: &'a str,
    pub offset: usize,
    pub line: usize,
}
impl<'a> Cursor<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            offset: 0,
            line: 0,
        }
    }
    pub fn next(&mut self) -> Result<Option<Row<'a>>, Error> {
        if self.offset == self.source.len() {
            return Ok(None);
        }
        self.line = self
            .line
            .checked_add(1)
            .filter(|value| *value <= MAX_LINES)
            .ok_or(Error::Resource("more than 1000000 physical lines"))?;
        let rest = self
            .source
            .get(self.offset..)
            .ok_or_else(|| syntax(self.line, "source span"))?;
        let newline = rest.find('\n');
        let end = match newline {
            Some(index) => self
                .offset
                .checked_add(index)
                .and_then(|value| value.checked_add(1))
                .ok_or(Error::Resource("source span"))?,
            None => self.source.len(),
        };
        let mut content_end = if newline.is_some() { end - 1 } else { end };
        if content_end > self.offset && self.source.as_bytes().get(content_end - 1) == Some(&b'\r')
        {
            if newline.is_none() {
                return Err(syntax(self.line, "line ending"));
            }
            content_end -= 1;
        }
        let text = self
            .source
            .get(self.offset..content_end)
            .ok_or_else(|| syntax(self.line, "source span"))?;
        if text.len() > MAX_LINE_BYTES {
            return Err(Error::Resource("physical line exceeds 4096 bytes"));
        }
        if text.contains('\r') {
            return Err(syntax(self.line, "line ending"));
        }
        let row = Row {
            text,
            range: self.offset..end,
            line: self.line,
        };
        self.offset = end;
        Ok(Some(row))
    }
    pub fn required(&mut self, field: &'static str) -> Result<Row<'a>, Error> {
        self.next()?
            .ok_or_else(|| syntax(self.line.saturating_add(1), field))
    }
    pub fn mol_end(&mut self) -> Result<usize, Error> {
        // Free MOL metadata can itself say $MOL or M END; it is not a delimiter.
        for _ in 0..4 {
            self.required("complete MOL header")?;
        }
        loop {
            let row = self.required("complete MOL terminator")?;
            if row.text.trim_end() == "M  END" {
                return Ok(row.range.end);
            }
        }
    }
}
pub(super) fn syntax(line: usize, field: &'static str) -> Error {
    Error::Syntax { line, field }
}
pub(super) fn ascii(source: &[u8]) -> Result<&str, Error> {
    if source.len() > MAX_BYTES {
        return Err(Error::Resource("input exceeds 64 MiB"));
    }
    if source.iter().any(|byte| !byte.is_ascii()) {
        return Err(Error::Unsupported {
            line: 0,
            feature: "encoding outside ASCII",
        });
    }
    if !source
        .iter()
        .all(|byte| matches!(byte, b' '..=b'~' | b'\t' | b'\r' | b'\n'))
    {
        return Err(syntax(0, "control byte"));
    }
    std::str::from_utf8(source).map_err(|_| syntax(0, "encoding"))
}
fn count(value: &str, line: usize) -> Result<usize, Error> {
    let digits = value.trim_matches(' ');
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(syntax(line, "unsigned three-column count"));
    }
    digits.parse().map_err(|_| syntax(line, "count"))
}
fn counts(row: &Row<'_>) -> Result<Counts, Error> {
    let first = row
        .text
        .get(..3)
        .ok_or_else(|| syntax(row.line, "reactant count"))?;
    let second = row
        .text
        .get(3..6)
        .ok_or_else(|| syntax(row.line, "product count"))?;
    let third = if row.text.len() >= 9 {
        if !row
            .text
            .get(9..)
            .is_some_and(|tail| tail.bytes().all(|byte| byte == b' '))
        {
            return Err(syntax(row.line, "data after counts"));
        }
        row.text
            .get(6..9)
            .ok_or_else(|| syntax(row.line, "reagent count"))?
    } else {
        if row.text.len() != 6 {
            return Err(syntax(row.line, "partial reagent count"));
        }
        "0"
    };
    Ok(Counts {
        reactants: count(first, row.line)?,
        products: count(second, row.line)?,
        reagents: count(third, row.line)?,
    })
}
pub(super) struct Stats {
    pub version: Version,
    pub atoms: usize,
    pub bonds: usize,
}
pub(super) fn validate_mol(bytes: &[u8], component: usize) -> Result<Stats, Error> {
    let line = bytes
        .split(|byte| *byte == b'\n')
        .nth(3)
        .ok_or_else(|| syntax(0, "constituent MOL version"))?;
    match line.get(33..39) {
        Some(b" V2000") => {
            let parsed =
                MolFile::parse(bytes).map_err(|source| Error::MolV2000 { component, source })?;
            Ok(Stats {
                version: Version::V2000,
                atoms: parsed.atoms().len(),
                bonds: parsed.bonds().len(),
            })
        }
        Some(b" V3000") => {
            let parsed = v3000::MolFile::parse(bytes)
                .map_err(|source| Error::MolV3000 { component, source })?;
            Ok(Stats {
                version: Version::V3000,
                atoms: parsed.atoms().len(),
                bonds: parsed.bonds().len(),
            })
        }
        _ => Err(syntax(0, "V2000/V3000 constituent signature")),
    }
}
#[derive(Default)]
pub(super) struct Totals {
    atoms: usize,
    bonds: usize,
}
impl Totals {
    pub fn add(&mut self, stats: &Stats) -> Result<(), Error> {
        self.atoms = self
            .atoms
            .checked_add(stats.atoms)
            .filter(|value| *value <= MAX_GRAPH_ITEMS)
            .ok_or(Error::Resource("more than 100000 aggregate atoms"))?;
        self.bonds = self
            .bonds
            .checked_add(stats.bonds)
            .filter(|value| *value <= MAX_GRAPH_ITEMS)
            .ok_or(Error::Resource("more than 100000 aggregate bonds"))?;
        Ok(())
    }
}
impl<'a> RxnFile<'a> {
    pub fn parse(source: &'a [u8]) -> Result<Self, Error> {
        let text = ascii(source)?;
        let mut cursor = Cursor::new(text);
        let signature = cursor.required("RXN signature")?;
        if signature.text.trim_end() == "$RXN V3000" {
            return Err(Error::Unsupported {
                line: 1,
                feature: "V3000 reaction wrapper",
            });
        }
        if signature.text.trim_end() != "$RXN" {
            return Err(syntax(1, "legacy $RXN signature"));
        }
        let a = cursor.required("reaction name")?;
        let b = cursor.required("reaction program metadata")?;
        let c = cursor.required("reaction comment")?;
        let row = cursor.required("reaction counts")?;
        let counts = counts(&row)?;
        let total = counts.total()?;
        let mut components = Vec::with_capacity(total);
        let mut totals = Totals::default();
        for (role, number) in [
            (Role::Reactant, counts.reactants),
            (Role::Product, counts.products),
            (Role::Reagent, counts.reagents),
        ] {
            for _ in 0..number {
                let marker = cursor.required("$MOL marker")?;
                if marker.text != "$MOL" {
                    return Err(syntax(marker.line, "exact $MOL marker"));
                }
                let start = cursor.offset;
                let end = cursor.mol_end()?;
                let mol_range = start..end;
                let mol_bytes = source
                    .get(mol_range.clone())
                    .ok_or_else(|| syntax(marker.line, "component span"))?;
                let stats = validate_mol(mol_bytes, components.len() + 1)?;
                totals.add(&stats)?;
                components.push(Component {
                    role,
                    version: stats.version,
                    range: marker.range.start..end,
                    mol_range,
                    mol_bytes,
                });
            }
        }
        if cursor.offset != source.len() {
            return Err(syntax(
                cursor.line.saturating_add(1),
                "uncounted component or trailing data",
            ));
        }
        Ok(Self {
            source,
            header: [a.text, b.text, c.text],
            header_ranges: [a.range, b.range, c.range],
            counts,
            counts_range: row.range,
            components,
        })
    }
}
