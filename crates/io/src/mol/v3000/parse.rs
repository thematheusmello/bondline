use super::{
    AtomRecord, Attribute, Block, BondRecord, Error, MAX_BYTES, MAX_DEPTH, MAX_ITEMS, MAX_LINES,
    MAX_LOGICAL_BYTES, MAX_PHYSICAL_LINE, MAX_SEGMENTS, MAX_TOTAL_TOKENS, MolFile,
    UnsupportedFeature, Value,
    tokens::{Token, parts, positional, tokenize},
};
use bondline_model::Radical;
use std::{collections::BTreeSet, num::NonZeroU16, ops::Range};

struct Physical<'a> {
    text: &'a str,
    line: usize,
    range: Range<usize>,
}
struct Entry {
    tokens: Vec<Token>,
    line: usize,
    range: Range<usize>,
}
struct Lines<'a> {
    source: &'a str,
    offset: usize,
    line: usize,
}
impl<'a> Lines<'a> {
    fn physical(&mut self) -> Result<Option<Physical<'a>>, Error> {
        if self.offset == self.source.len() {
            return Ok(None);
        }
        self.line = self
            .line
            .checked_add(1)
            .filter(|value| *value <= MAX_LINES)
            .ok_or(Error::Resource("more than 1000000 physical lines"))?;
        let rest = self.source.get(self.offset..).ok_or(Error::Syntax {
            line: self.line,
            field: "line span",
        })?;
        let newline = rest.find('\n');
        let end = match newline {
            Some(index) => self
                .offset
                .checked_add(index)
                .and_then(|value| value.checked_add(1))
                .ok_or(Error::Resource("line span"))?,
            None => self.source.len(),
        };
        let mut content_end = if newline.is_some() { end - 1 } else { end };
        if self.source.as_bytes().get(content_end.saturating_sub(1)) == Some(&b'\r')
            && content_end > self.offset
        {
            if newline.is_none() {
                return Err(Error::Syntax {
                    line: self.line,
                    field: "line ending",
                });
            }
            content_end -= 1;
        }
        let text = self
            .source
            .get(self.offset..content_end)
            .ok_or(Error::Syntax {
                line: self.line,
                field: "line span",
            })?;
        if text.len() > MAX_PHYSICAL_LINE {
            return Err(Error::Resource("physical line exceeds 80 ASCII characters"));
        }
        if text.contains('\r') {
            return Err(Error::Syntax {
                line: self.line,
                field: "line ending",
            });
        }
        let row = Physical {
            text,
            line: self.line,
            range: self.offset..end,
        };
        self.offset = end;
        Ok(Some(row))
    }
    fn required(&mut self) -> Result<Physical<'a>, Error> {
        self.physical()?.ok_or(Error::Syntax {
            line: self.line + 1,
            field: "truncated file",
        })
    }
    fn entry(&mut self) -> Result<Option<Entry>, Error> {
        let Some(first) = self.physical()? else {
            return Err(Error::Syntax {
                line: self.line + 1,
                field: "missing M  END",
            });
        };
        if first.text.trim_end() == "M  END" {
            return Ok(None);
        }
        let line = first.line;
        let start = first.range.start;
        let mut current = first;
        let mut logical = String::new();
        let mut segments = 0_usize;
        let end = loop {
            segments = segments
                .checked_add(1)
                .filter(|value| *value <= MAX_SEGMENTS)
                .ok_or(Error::Resource("more than 1024 continuation segments"))?;
            let content = current.text.strip_prefix("M  V30 ").ok_or(Error::Syntax {
                line: current.line,
                field: "M  V30 prefix or dangling continuation",
            })?;
            let continuing = content.ends_with('-');
            let content = if continuing {
                content.strip_suffix('-').ok_or(Error::Syntax {
                    line: current.line,
                    field: "continuation",
                })?
            } else {
                content
            };
            if logical
                .len()
                .checked_add(content.len())
                .filter(|value| *value <= MAX_LOGICAL_BYTES)
                .is_none()
            {
                return Err(Error::Resource("logical entry exceeds 65536 bytes"));
            }
            logical.push_str(content);
            if !continuing {
                break current.range.end;
            }
            current = self.required()?;
        };
        Ok(Some(Entry {
            tokens: tokenize(&logical, line)?,
            line,
            range: start..end,
        }))
    }
}
fn invalid(line: usize, field: &'static str) -> Error {
    Error::Syntax { line, field }
}
fn integer(text: &str, line: usize, field: &'static str) -> Result<i64, Error> {
    let digits = text.strip_prefix(['+', '-']).unwrap_or(text);
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(invalid(line, field));
    }
    text.parse()
        .map_err(|_| Error::Resource("integer magnitude exceeds supported range"))
}
fn unsigned(text: &str, line: usize, field: &'static str) -> Result<u32, Error> {
    u32::try_from(integer(text, line, field)?).map_err(|_| invalid(line, field))
}
fn positive(text: &str, line: usize, field: &'static str) -> Result<u32, Error> {
    let value = unsigned(text, line, field)?;
    if value == 0 {
        return Err(invalid(line, field));
    }
    Ok(value)
}
fn real(text: &str, line: usize) -> Result<f64, Error> {
    let bytes = text.as_bytes();
    let mut index = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
    let mut digits = 0;
    while bytes.get(index).is_some_and(u8::is_ascii_digit) {
        index += 1;
        digits += 1;
    }
    if bytes.get(index) == Some(&b'.') {
        index += 1;
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
            digits += 1;
        }
    }
    if digits == 0 {
        return Err(invalid(line, "coordinate"));
    }
    if matches!(bytes.get(index), Some(b'e' | b'E')) {
        index += 1;
        if matches!(bytes.get(index), Some(b'+' | b'-')) {
            index += 1;
        }
        let start = index;
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
        if index == start {
            return Err(invalid(line, "coordinate"));
        }
    }
    if index != bytes.len() {
        return Err(invalid(line, "coordinate"));
    }
    let value = text
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
        .ok_or_else(|| invalid(line, "finite coordinate"))?;
    let mantissa = text
        .split(['e', 'E'])
        .next()
        .ok_or_else(|| invalid(line, "coordinate"))?;
    if value == 0.0 && mantissa.bytes().any(|byte| matches!(byte, b'1'..=b'9')) {
        return Err(Error::Unsupported {
            line,
            feature: "nonzero coordinate underflows the f64 representation",
        });
    }
    Ok(value)
}
fn mark(features: &mut Vec<UnsupportedFeature>, line: usize, feature: &'static str) {
    if features.last().is_none_or(|prior| prior.line != line) {
        features.push(UnsupportedFeature { line, feature });
    }
}
fn attr_text(attribute: &Attribute, line: usize) -> Result<&str, Error> {
    match &attribute.value {
        Value::Text(value) => Ok(value),
        _ => Err(invalid(line, "scalar atom/bond property")),
    }
}
fn atom(entry: Entry, features: &mut Vec<UnsupportedFeature>) -> Result<AtomRecord, Error> {
    let line = entry.line;
    let (positions, attributes) = parts(entry.tokens, 6, line)?;
    let get = |index| {
        positions
            .get(index)
            .map(String::as_str)
            .ok_or_else(|| invalid(line, "atom positional fields"))
    };
    let source_index = positive(get(0)?, line, "atom source index")?;
    let symbol = get(1)?.to_owned();
    let element = super::super::symbols::element(&symbol);
    let x = real(get(2)?, line)?;
    let y = real(get(3)?, line)?;
    let z = real(get(4)?, line)?;
    let mapping = unsigned(get(5)?, line, "atom mapping")?;
    let mut record = AtomRecord {
        source_index,
        symbol,
        element,
        x,
        y,
        z,
        mapping,
        charge: 0,
        radical: Radical::None,
        isotope: None,
        declared_valence: None,
        attributes,
        line,
        range: entry.range,
    };
    if element.is_none() {
        mark(features, line, "query or non-element atom type");
    }
    if z != 0.0 {
        mark(features, line, "3D coordinates");
    }
    if mapping != 0 {
        mark(features, line, "reaction atom mapping");
    }
    for attribute in &record.attributes {
        match attribute.key.as_str() {
            "CHG" => {
                let value = integer(attr_text(attribute, line)?, line, "CHG")?;
                if !(-15..=15).contains(&value) {
                    return Err(invalid(line, "CHG outside -15..15"));
                }
                record.charge = i8::try_from(value).map_err(|_| invalid(line, "CHG"))?;
            }
            "RAD" => {
                record.radical = match integer(attr_text(attribute, line)?, line, "RAD")? {
                    0 => Radical::None,
                    1 => Radical::Singlet,
                    2 => Radical::Doublet,
                    3 => Radical::Triplet,
                    _ => return Err(invalid(line, "RAD outside 0..3")),
                }
            }
            "VAL" => {
                record.declared_valence = match integer(attr_text(attribute, line)?, line, "VAL")? {
                    -1 => Some(0),
                    0 => None,
                    1..=14 => Some(
                        u8::try_from(integer(attr_text(attribute, line)?, line, "VAL")?)
                            .map_err(|_| invalid(line, "VAL"))?,
                    ),
                    _ => return Err(invalid(line, "VAL outside -1..14")),
                }
            }
            "MASS" => {
                let value = attr_text(attribute, line)?;
                record.isotope = value.parse::<u16>().ok().and_then(NonZeroU16::new);
                if !value.bytes().all(|byte| byte.is_ascii_digit()) || record.isotope.is_none() {
                    mark(
                        features,
                        line,
                        "MASS outside positive integer isotope subset",
                    );
                }
            }
            "HCOUNT" => {
                if integer(attr_text(attribute, line)?, line, "HCOUNT")? < -1 {
                    return Err(invalid(line, "HCOUNT below -1"));
                }
                mark(features, line, "hydrogen-count query");
            }
            "CFG" => {
                if !(0..=3).contains(&integer(attr_text(attribute, line)?, line, "atom CFG")?) {
                    return Err(invalid(line, "atom CFG"));
                }
                mark(features, line, "atom stereo configuration");
            }
            _ => mark(features, line, "unmodeled atom property"),
        }
    }
    Ok(record)
}
fn bond(entry: Entry, features: &mut Vec<UnsupportedFeature>) -> Result<BondRecord, Error> {
    let line = entry.line;
    let (positions, attributes) = parts(entry.tokens, 4, line)?;
    let get = |index| {
        positions
            .get(index)
            .map(String::as_str)
            .ok_or_else(|| invalid(line, "bond positional fields"))
    };
    let source_index = positive(get(0)?, line, "bond source index")?;
    let kind = positive(get(1)?, line, "bond type")?;
    let order = u8::try_from(kind).map_err(|_| Error::Unsupported {
        line,
        feature: "bond type outside supported numeric range",
    })?;
    if !(1..=3).contains(&order) {
        mark(features, line, "query or extended bond type");
    }
    let begin = positive(get(2)?, line, "bond begin")?;
    let end = positive(get(3)?, line, "bond end")?;
    for attribute in &attributes {
        if attribute.key == "CFG"
            && !(0..=3).contains(&integer(attr_text(attribute, line)?, line, "bond CFG")?)
        {
            return Err(invalid(line, "bond CFG"));
        }
        mark(features, line, "unmodeled bond property");
    }
    Ok(BondRecord {
        source_index,
        begin,
        end,
        order,
        attributes,
        line,
        range: entry.range,
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Stage {
    Counts,
    Atom,
    Bond,
    After,
}
impl<'a> MolFile<'a> {
    pub fn parse(source: &'a [u8]) -> Result<Self, Error> {
        if source.len() > MAX_BYTES {
            return Err(Error::Resource("input exceeds 64 MiB"));
        }
        if !source
            .iter()
            .all(|byte| matches!(byte, b' '..=b'~' | b'\t' | b'\n' | b'\r'))
        {
            return Err(Error::Unsupported {
                line: 0,
                feature: "encoding outside printable ASCII/LF/CRLF",
            });
        }
        let text = std::str::from_utf8(source).map_err(|_| invalid(0, "encoding"))?;
        let mut lines = Lines {
            source: text,
            offset: 0,
            line: 0,
        };
        let header = [
            lines.required()?.text,
            lines.required()?.text,
            lines.required()?.text,
        ];
        if ["$MDL", "$$$$", "$RXN", "$RDFILE"]
            .iter()
            .any(|tag| header.first().is_some_and(|title| title.contains(tag)))
        {
            return Err(invalid(1, "reserved title"));
        }
        let legacy = lines.required()?;
        if legacy.text.get(33..39) != Some(" V3000") || legacy.text.get(30..33) != Some("999") {
            return Err(invalid(4, "V3000 version and appendix stamp"));
        }
        let mut file = Self {
            source,
            header,
            atoms: Vec::new(),
            bonds: Vec::new(),
            blocks: Vec::new(),
            unsupported: Vec::new(),
        };
        if header
            .get(1)
            .and_then(|value| value.get(20..22))
            .is_some_and(|value| value == "3D")
        {
            mark(&mut file.unsupported, 2, "3D dimensional header");
        }
        let mut stack = Vec::<usize>::new();
        let mut stage = Stage::Counts;
        let mut counts = None;
        let mut started = false;
        let mut closed = false;
        let mut atom_block = None;
        let mut bond_block = None;
        let mut atom_ids = BTreeSet::new();
        let mut bond_ids = BTreeSet::new();
        let mut total_tokens = 0_usize;
        while let Some(entry) = lines.entry()? {
            let line = entry.line;
            let count = entry.tokens.iter().try_fold(0_usize, |sum, token| {
                let length = match token {
                    Token::Value(Value::List(values))
                    | Token::Attribute(Attribute {
                        value: Value::List(values),
                        ..
                    }) => values.len() + 1,
                    _ => 1,
                };
                sum.checked_add(length)
                    .ok_or(Error::Resource("token budget"))
            })?;
            total_tokens = total_tokens
                .checked_add(count)
                .filter(|value| *value <= MAX_TOTAL_TOKENS)
                .ok_or(Error::Resource(
                    "more than 4000000 decoded tokens/list values",
                ))?;
            let key = match entry.tokens.first() {
                Some(Token::Value(Value::Text(value))) => value.as_str(),
                _ => "",
            };
            if !started && key != "BEGIN" {
                return Err(invalid(line, "initial BEGIN CTAB"));
            }
            if key == "BEGIN" {
                let name = positional(&entry.tokens, 1, line)?.to_owned();
                if !started {
                    if name != "CTAB" {
                        return Err(invalid(line, "initial BEGIN CTAB"));
                    }
                    started = true;
                } else if stack.is_empty() && name == "CTAB" {
                    return Err(invalid(line, "multiple top-level CTAB blocks"));
                }
                if stack.len() >= MAX_DEPTH {
                    return Err(Error::Resource("block depth exceeds 256"));
                }
                let direct = stack.as_slice() == [0];
                if direct {
                    match (stage, name.as_str()) {
                        (Stage::Atom, "ATOM") => atom_block = Some(file.blocks.len()),
                        (Stage::Bond, "BOND") => bond_block = Some(file.blocks.len()),
                        (Stage::Bond, _) if counts.is_some_and(|(_, bonds)| bonds == 0) => {
                            stage = Stage::After
                        }
                        (Stage::After, "ATOM" | "BOND") => {
                            return Err(invalid(line, "duplicate or out-of-order core block"));
                        }
                        (Stage::After, _) => {}
                        _ => return Err(invalid(line, "COUNTS/ATOM/BOND ordering")),
                    }
                }
                let core = file.blocks.is_empty()
                    || atom_block == Some(file.blocks.len())
                    || bond_block == Some(file.blocks.len());
                if !core {
                    mark(
                        &mut file.unsupported,
                        line,
                        "unmodeled block (semantic counts/references not projected)",
                    );
                }
                if entry.tokens.len() != 2 {
                    mark(
                        &mut file.unsupported,
                        line,
                        "named or extended block declaration",
                    );
                }
                let index = file.blocks.len();
                file.blocks.push(Block {
                    name,
                    parent: stack.last().copied(),
                    line,
                    range: entry.range,
                });
                stack.push(index);
            } else if key == "END" {
                if entry.tokens.len() != 2 {
                    return Err(invalid(line, "END block tokens"));
                }
                let name = positional(&entry.tokens, 1, line)?;
                let index = stack
                    .pop()
                    .ok_or_else(|| invalid(line, "unmatched END block"))?;
                let block = file
                    .blocks
                    .get_mut(index)
                    .ok_or_else(|| invalid(line, "block span"))?;
                if block.name != name {
                    return Err(invalid(line, "mismatched END block"));
                }
                block.range.end = entry.range.end;
                if Some(index) == atom_block {
                    stage = Stage::Bond;
                }
                if Some(index) == bond_block {
                    stage = Stage::After;
                }
                if index == 0 {
                    if stage != Stage::After
                        && !(stage == Stage::Bond && counts.is_some_and(|(_, bonds)| bonds == 0))
                    {
                        return Err(invalid(line, "missing mandatory atom/bond block"));
                    }
                    closed = true;
                }
            } else if stack.as_slice() == [0] && stage == Stage::Counts {
                if key != "COUNTS" {
                    return Err(invalid(line, "first CTAB entry must be COUNTS"));
                }
                let (positions, attributes) = parts(entry.tokens, 6, line)?;
                let mut values = Vec::new();
                for position in positions.iter().skip(1) {
                    values.push(unsigned(position, line, "COUNTS")?);
                }
                if values
                    .iter()
                    .take(4)
                    .any(|value| usize::try_from(*value).map_or(true, |value| value > MAX_ITEMS))
                {
                    return Err(Error::Resource("COUNTS exceeds 100000 items"));
                }
                if values.get(4).is_none_or(|value| *value > 1) {
                    return Err(invalid(line, "chiral flag"));
                }
                counts = Some((
                    usize::try_from(*values.first().ok_or_else(|| invalid(line, "atom count"))?)
                        .map_err(|_| Error::Resource("atom count"))?,
                    usize::try_from(*values.get(1).ok_or_else(|| invalid(line, "bond count"))?)
                        .map_err(|_| Error::Resource("bond count"))?,
                ));
                if values.iter().skip(2).any(|value| *value != 0) || !attributes.is_empty() {
                    mark(
                        &mut file.unsupported,
                        line,
                        "unmodeled COUNTS property, chirality, Sgroups or 3D constraints",
                    );
                }
                stage = Stage::Atom;
            } else if stack.last().copied() == atom_block && atom_block.is_some() {
                if file.atoms.len() >= MAX_ITEMS {
                    return Err(Error::Resource("more than 100000 atoms"));
                }
                let atom = atom(entry, &mut file.unsupported)?;
                if !atom_ids.insert(atom.source_index) {
                    return Err(invalid(line, "duplicate atom source index"));
                }
                file.atoms.push(atom);
            } else if stack.last().copied() == bond_block && bond_block.is_some() {
                if file.bonds.len() >= MAX_ITEMS {
                    return Err(Error::Resource("more than 100000 bonds"));
                }
                let bond = bond(entry, &mut file.unsupported)?;
                if !bond_ids.insert(bond.source_index) {
                    return Err(invalid(line, "duplicate bond source index"));
                }
                file.bonds.push(bond);
            } else {
                if stack.as_slice() == [0] && key == "COUNTS" {
                    return Err(invalid(line, "duplicate or misplaced COUNTS"));
                }
                if !started
                    || (!closed
                        && stack.as_slice() == [0]
                        && stage != Stage::After
                        && !(stage == Stage::Bond && counts.is_some_and(|(_, bonds)| bonds == 0)))
                {
                    return Err(invalid(line, "core block ordering"));
                }
                mark(
                    &mut file.unsupported,
                    line,
                    "unmodeled entry; raw source retained",
                );
            }
        }
        if !started || !closed || !stack.is_empty() {
            return Err(invalid(lines.line, "incomplete CTAB/block nesting"));
        }
        if lines.physical()?.is_some() {
            return Err(invalid(lines.line, "data after M  END"));
        }
        if counts != Some((file.atoms.len(), file.bonds.len())) {
            return Err(invalid(
                lines.line,
                "COUNTS disagrees with atom/bond entries",
            ));
        }
        for bond in &file.bonds {
            if bond.begin == bond.end
                || !atom_ids.contains(&bond.begin)
                || !atom_ids.contains(&bond.end)
            {
                return Err(invalid(bond.line, "bond endpoint reference"));
            }
        }
        Ok(file)
    }
}
