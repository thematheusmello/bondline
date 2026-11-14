//! A bounded, source-preserving nonaromatic SMILES notation graph.
//!
//! Parsing is syntax and connectivity, not chemical validation, hydrogen perception,
//! model projection, layout or canonicalization. Source spellings stay authoritative.

use bondline_model::Element;
use std::{collections::HashSet, ops::Range, sync::Arc};

mod layout;
mod normalize;
mod write;
pub use layout::LayoutError;
pub use normalize::NormalizeError;
pub use write::{WriteError, write_fragment};

pub const MAX_SMILES_BYTES: usize = 65_536;
pub const MAX_SMILES_ATOMS: usize = 4096;
pub const MAX_SMILES_BONDS: usize = 8192;
pub const MAX_SMILES_BRANCHES: usize = 128;
pub const MAX_SMILES_CHAINS: usize = 1024;
pub const MAX_SMILES_RINGS: usize = 100;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Limit {
    Bytes,
    Atoms,
    Bonds,
    Branches,
    Chains,
    Rings,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SmilesError {
    #[error("invalid SMILES at byte {offset}: {reason}")]
    Syntax { offset: usize, reason: &'static str },
    #[error("unsupported SMILES at byte {offset}: {feature}")]
    Unsupported {
        offset: usize,
        feature: &'static str,
    },
    #[error("SMILES {kind:?} limit exceeded at byte {offset}")]
    Limit { offset: usize, kind: Limit },
}

impl SmilesError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Syntax { .. } => "invalid_smiles",
            Self::Unsupported { .. } => "unsupported_content",
            Self::Limit { .. } => "resource_limit",
        }
    }

    pub fn offset(&self) -> usize {
        match self {
            Self::Syntax { offset, .. }
            | Self::Unsupported { offset, .. }
            | Self::Limit { offset, .. } => *offset,
        }
    }
}

/// Implicit means a bare organic-subset token, not an inferred numeric H count.
/// Bracket omission has count zero; a separate [H] is an independent graph atom.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hydrogens {
    Implicit,
    Bracket(u8),
}

/// All ranges are byte offsets in the retained source. No field is a model atom ID.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Atom {
    pub range: Range<usize>,
    pub symbol_range: Range<usize>,
    pub element: Element,
    pub isotope: Option<u16>,
    pub isotope_range: Option<Range<usize>>,
    pub hydrogens: Hydrogens,
    pub hydrogen_range: Option<Range<usize>>,
    pub charge: i8,
    pub charge_range: Option<Range<usize>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BondOrder {
    Single,
    Double,
    Triple,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RingClosure {
    pub label: u8,
    pub open_range: Range<usize>,
    pub close_range: Range<usize>,
    pub open_bond_range: Option<Range<usize>>,
    pub close_bond_range: Option<Range<usize>>,
}

/// Zero-based atom indices. For a ring, both declarations stay in `ring`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bond {
    pub atoms: [usize; 2],
    pub order: BondOrder,
    pub bond_range: Option<Range<usize>>,
    pub ring: Option<RingClosure>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct SmilesDocument {
    source: Arc<[u8]>,
    notation_end: usize,
    atoms: Arc<[Atom]>,
    bonds: Arc<[Bond]>,
    chain_starts: usize,
}

impl std::fmt::Debug for SmilesDocument {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SmilesDocument")
            .field("byte_length", &self.source.len())
            .field("atom_count", &self.atoms.len())
            .field("bond_count", &self.bonds.len())
            .finish()
    }
}

impl SmilesDocument {
    /// Own one complete ASCII notation, with at most one terminal LF or CRLF.
    pub fn parse(source: Arc<[u8]>) -> Result<Self, SmilesError> {
        if source.len() > MAX_SMILES_BYTES {
            return Err(SmilesError::Limit {
                offset: MAX_SMILES_BYTES,
                kind: Limit::Bytes,
            });
        }
        let notation = source
            .strip_suffix(b"\r\n")
            .or_else(|| source.strip_suffix(b"\n"))
            .unwrap_or(&source);
        for (offset, byte) in notation.iter().enumerate() {
            if !byte.is_ascii() {
                return Err(SmilesError::Unsupported {
                    offset,
                    feature: "non-ASCII source",
                });
            }
            if byte.is_ascii_whitespace() || byte.is_ascii_control() {
                return Err(SmilesError::Syntax {
                    offset,
                    reason: "only one terminal LF or CRLF is allowed; no names or extra lines",
                });
            }
        }
        let notation_end = notation.len();
        let (atoms, bonds, chain_starts) = Parser::new(notation).parse()?;
        Ok(Self {
            source,
            notation_end,
            atoms: atoms.into(),
            bonds: bonds.into(),
            chain_starts,
        })
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.source
    }
    pub fn notation_range(&self) -> Range<usize> {
        0..self.notation_end
    }
    pub fn atoms(&self) -> &[Atom] {
        &self.atoms
    }
    pub fn bonds(&self) -> &[Bond] {
        &self.bonds
    }
    /// Initial chain plus dot-separated starts, not the graph's component count.
    pub fn chain_starts(&self) -> usize {
        self.chain_starts
    }
}

struct Parser<'a> {
    bytes: &'a [u8],
    offset: usize,
    atoms: Vec<Atom>,
    bonds: Vec<Bond>,
    current: Option<usize>,
    pending: Option<(BondOrder, Range<usize>)>,
    chain_starts: usize,
    branches: Vec<(usize, usize)>,
    rings: [Option<OpenRing>; MAX_SMILES_RINGS],
    open_rings: usize,
    edges: HashSet<(usize, usize)>,
    expect_atom: bool,
    branch_start: bool,
    after_branches: bool,
}

struct OpenRing {
    atom: usize,
    range: Range<usize>,
    bond: Option<(BondOrder, Range<usize>)>,
}

impl<'a> Parser<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            offset: 0,
            atoms: Vec::new(),
            bonds: Vec::new(),
            current: None,
            pending: None,
            chain_starts: 1,
            branches: Vec::new(),
            rings: std::array::from_fn(|_| None),
            open_rings: 0,
            edges: HashSet::new(),
            expect_atom: true,
            branch_start: false,
            after_branches: false,
        }
    }

    fn syntax(&self, reason: &'static str) -> SmilesError {
        SmilesError::Syntax {
            offset: self.offset,
            reason,
        }
    }
    fn unsupported(&self, feature: &'static str) -> SmilesError {
        SmilesError::Unsupported {
            offset: self.offset,
            feature,
        }
    }
    fn byte(&self) -> Option<u8> {
        self.bytes.get(self.offset).copied()
    }
    fn advance(&mut self, count: usize) -> Result<(), SmilesError> {
        self.offset = self
            .offset
            .checked_add(count)
            .filter(|end| *end <= self.bytes.len())
            .ok_or_else(|| self.syntax("unexpected end of notation"))?;
        Ok(())
    }

    fn parse(mut self) -> Result<(Vec<Atom>, Vec<Bond>, usize), SmilesError> {
        while let Some(byte) = self.byte() {
            match byte {
                b'-' | b'=' | b'#' => {
                    if self.current.is_none()
                        || self.pending.is_some()
                        || self.expect_atom && !self.branch_start
                    {
                        return Err(self.syntax("bond requires preceding and following atoms"));
                    }
                    let start = self.offset;
                    self.advance(1)?;
                    self.pending = Some((
                        match byte {
                            b'=' => BondOrder::Double,
                            b'#' => BondOrder::Triple,
                            _ => BondOrder::Single,
                        },
                        start..self.offset,
                    ));
                    self.branch_start = false;
                }
                b'(' => {
                    if self.expect_atom || self.pending.is_some() {
                        return Err(self.syntax("branch requires a preceding atom"));
                    }
                    if self.branches.len() >= MAX_SMILES_BRANCHES {
                        return Err(SmilesError::Limit {
                            offset: self.offset,
                            kind: Limit::Branches,
                        });
                    }
                    let parent = self
                        .current
                        .ok_or_else(|| self.syntax("branch requires a preceding atom"))?;
                    self.branches.push((parent, self.offset));
                    self.advance(1)?;
                    self.expect_atom = true;
                    self.branch_start = true;
                }
                b')' => {
                    if self.expect_atom || self.pending.is_some() {
                        return Err(self.syntax("branch must end after an atom"));
                    }
                    let (parent, _) = self
                        .branches
                        .pop()
                        .ok_or_else(|| self.syntax("unmatched closing branch"))?;
                    self.current = Some(parent);
                    self.advance(1)?;
                    self.after_branches = true;
                    self.branch_start = false;
                }
                b'.' => {
                    if self.pending.is_some()
                        || self.current.is_none()
                        || self.expect_atom && !self.branch_start
                    {
                        return Err(self.syntax("dot requires a chain on both sides"));
                    }
                    self.chain_starts = self
                        .chain_starts
                        .checked_add(1)
                        .filter(|count| *count <= MAX_SMILES_CHAINS)
                        .ok_or(SmilesError::Limit {
                            offset: self.offset,
                            kind: Limit::Chains,
                        })?;
                    self.current = None;
                    self.expect_atom = true;
                    self.branch_start = false;
                    self.advance(1)?;
                }
                b'0'..=b'9' | b'%' => self.ring()?,
                _ => {
                    if self.atoms.len() >= MAX_SMILES_ATOMS {
                        return Err(SmilesError::Limit {
                            offset: self.offset,
                            kind: Limit::Atoms,
                        });
                    }
                    let atom = self.atom()?;
                    let index = self.atoms.len();
                    if let Some(previous) = self.current {
                        let (order, range) = match self.pending.take() {
                            Some((order, range)) => (order, Some(range)),
                            None => (BondOrder::Single, None),
                        };
                        self.add_bond(
                            Bond {
                                atoms: [previous, index],
                                order,
                                bond_range: range,
                                ring: None,
                            },
                            atom.range.start,
                        )?;
                    }
                    self.atoms.push(atom);
                    self.current = Some(index);
                    self.expect_atom = false;
                    self.branch_start = false;
                    self.after_branches = false;
                }
            }
        }
        if self.atoms.is_empty() || self.pending.is_some() || self.expect_atom {
            return Err(self.syntax("expected atom"));
        }
        if let Some((_, offset)) = self.branches.first() {
            return Err(SmilesError::Syntax {
                offset: *offset,
                reason: "unclosed branch",
            });
        }
        if let Some(ring) = self
            .rings
            .iter()
            .flatten()
            .min_by_key(|ring| ring.range.start)
        {
            return Err(SmilesError::Syntax {
                offset: ring.range.start,
                reason: "unclosed ring label",
            });
        }
        Ok((self.atoms, self.bonds, self.chain_starts))
    }

    fn add_bond(&mut self, bond: Bond, offset: usize) -> Result<(), SmilesError> {
        let [left, right] = bond.atoms;
        if left == right {
            return Err(SmilesError::Syntax {
                offset,
                reason: "self bond",
            });
        }
        let key = (left.min(right), left.max(right));
        if self.edges.contains(&key) {
            return Err(SmilesError::Syntax {
                offset,
                reason: "duplicate bond",
            });
        }
        if self.bonds.len() >= MAX_SMILES_BONDS {
            return Err(SmilesError::Limit {
                offset,
                kind: Limit::Bonds,
            });
        }
        self.edges.insert(key);
        self.bonds.push(bond);
        Ok(())
    }

    fn ring(&mut self) -> Result<(), SmilesError> {
        if self.expect_atom || self.after_branches {
            return Err(self.syntax("ring labels must follow an atom and precede its branches"));
        }
        let atom = self
            .current
            .ok_or_else(|| self.syntax("ring label requires an atom"))?;
        let start = self.offset;
        let label = if self.byte() == Some(b'%') {
            self.advance(1)?;
            let tens = self
                .byte()
                .filter(u8::is_ascii_digit)
                .ok_or_else(|| self.syntax("percent ring label requires two digits"))?
                - b'0';
            self.advance(1)?;
            let units = self
                .byte()
                .filter(u8::is_ascii_digit)
                .ok_or_else(|| self.syntax("percent ring label requires two digits"))?
                - b'0';
            self.advance(1)?;
            tens * 10 + units
        } else {
            let label = self
                .byte()
                .filter(u8::is_ascii_digit)
                .ok_or_else(|| self.syntax("ring digit required"))?
                - b'0';
            self.advance(1)?;
            label
        };
        let range = start..self.offset;
        let pending = self.pending.take();
        let old = self
            .rings
            .get_mut(usize::from(label))
            .ok_or(SmilesError::Syntax {
                offset: start,
                reason: "invalid ring label",
            })?
            .take();
        if let Some(open) = old {
            let order = match (&open.bond, &pending) {
                (Some((left, _)), Some((right, _))) if left != right => {
                    return Err(SmilesError::Syntax {
                        offset: start,
                        reason: "ring bond declarations disagree",
                    });
                }
                (Some((order, _)), _) | (_, Some((order, _))) => *order,
                _ => BondOrder::Single,
            };
            let closure = RingClosure {
                label,
                open_range: open.range,
                close_range: range,
                open_bond_range: open.bond.map(|(_, range)| range),
                close_bond_range: pending.map(|(_, range)| range),
            };
            self.add_bond(
                Bond {
                    atoms: [open.atom, atom],
                    order,
                    bond_range: None,
                    ring: Some(closure),
                },
                start,
            )?;
            self.open_rings = self
                .open_rings
                .checked_sub(1)
                .ok_or_else(|| self.syntax("invalid open-ring count"))?;
        } else {
            if self.open_rings >= MAX_SMILES_RINGS {
                return Err(SmilesError::Limit {
                    offset: start,
                    kind: Limit::Rings,
                });
            }
            self.open_rings = self.open_rings.checked_add(1).ok_or(SmilesError::Limit {
                offset: start,
                kind: Limit::Rings,
            })?;
            let slot = self
                .rings
                .get_mut(usize::from(label))
                .ok_or(SmilesError::Syntax {
                    offset: start,
                    reason: "invalid ring label",
                })?;
            *slot = Some(OpenRing {
                atom,
                range,
                bond: pending,
            });
        }
        Ok(())
    }

    fn atom(&mut self) -> Result<Atom, SmilesError> {
        let start = self.offset;
        let bracketed = self.byte() == Some(b'[');
        if bracketed {
            self.advance(1)?;
        }
        let isotope_start = self.offset;
        let mut isotope = None;
        if bracketed {
            while let Some(digit @ b'0'..=b'9') = self.byte() {
                isotope = Some(
                    isotope
                        .unwrap_or(0_u16)
                        .checked_mul(10)
                        .and_then(|value| value.checked_add(u16::from(digit - b'0')))
                        .ok_or_else(|| self.unsupported("isotope exceeds u16 notation profile"))?,
                );
                self.advance(1)?;
            }
        }
        let isotope_range = isotope.map(|_| isotope_start..self.offset);
        let symbol_start = self.offset;
        match self.byte() {
            Some(b'*') => return Err(self.unsupported("wildcard atom")),
            Some(b'a'..=b'z') => return Err(self.unsupported("aromatic atom")),
            Some(b'A'..=b'Z') => self.advance(1)?,
            Some(b'@' | b'/' | b'\\') => return Err(self.unsupported("stereochemistry")),
            Some(b':' | b'$' | b'~' | b'?' | b'!' | b'&' | b'|' | b'>' | b';' | b',' | b'#') => {
                return Err(self.unsupported("query, extended, aromatic or reaction notation"));
            }
            _ => return Err(self.syntax("expected atom")),
        }
        let organic_pair = matches!(
            (self.bytes.get(symbol_start).copied(), self.byte()),
            (Some(b'C'), Some(b'l')) | (Some(b'B'), Some(b'r'))
        );
        if bracketed && self.byte().is_some_and(|byte| byte.is_ascii_lowercase())
            || !bracketed && organic_pair
        {
            self.advance(1)?;
        }
        let symbol_range = symbol_start..self.offset;
        let symbol = std::str::from_utf8(
            self.bytes
                .get(symbol_range.clone())
                .ok_or_else(|| self.syntax("invalid symbol span"))?,
        )
        .map_err(|_| self.unsupported("non-ASCII symbol"))?;
        let element = crate::mol::symbols::element(symbol)
            .ok_or_else(|| self.syntax("unknown element symbol"))?;
        if !bracketed
            && !matches!(
                symbol,
                "B" | "C" | "N" | "O" | "P" | "S" | "F" | "Cl" | "Br" | "I"
            )
        {
            return Err(self.unsupported("unbracketed atom outside organic subset"));
        }
        let mut atom = Atom {
            range: start..self.offset,
            symbol_range,
            element,
            isotope,
            isotope_range,
            hydrogens: if bracketed {
                Hydrogens::Bracket(0)
            } else {
                Hydrogens::Implicit
            },
            hydrogen_range: None,
            charge: 0,
            charge_range: None,
        };
        if bracketed {
            if self.byte() == Some(b'H') {
                let hydrogen_start = self.offset;
                if element.get() == 1 {
                    return Err(self.syntax("hydrogen atom cannot carry a bracket hydrogen count"));
                }
                self.advance(1)?;
                let count = match self.byte() {
                    Some(digit @ b'0'..=b'9') => {
                        self.advance(1)?;
                        digit - b'0'
                    }
                    _ => 1,
                };
                atom.hydrogens = Hydrogens::Bracket(count);
                atom.hydrogen_range = Some(hydrogen_start..self.offset);
            }
            if let Some(sign @ (b'+' | b'-')) = self.byte() {
                let charge_start = self.offset;
                self.advance(1)?;
                let mut magnitude = 1_u8;
                if self.byte() == Some(sign) {
                    self.advance(1)?;
                    magnitude = 2;
                } else if self.byte().is_some_and(|byte| byte.is_ascii_digit()) {
                    magnitude = 0;
                    for _ in 0..2 {
                        if let Some(digit @ b'0'..=b'9') = self.byte() {
                            magnitude = magnitude * 10 + digit - b'0';
                            self.advance(1)?;
                        } else {
                            break;
                        }
                    }
                }
                if magnitude > 15 {
                    return Err(self.unsupported("charge outside -15..15 notation profile"));
                }
                let signed = i8::try_from(magnitude)
                    .map_err(|_| self.unsupported("charge outside notation profile"))?;
                atom.charge = if sign == b'-' { -signed } else { signed };
                atom.charge_range = Some(charge_start..self.offset);
            }
            match self.byte() {
                Some(b']') => self.advance(1)?,
                Some(b'@') => return Err(self.unsupported("atom stereochemistry")),
                Some(b':') => return Err(self.unsupported("atom class or map")),
                Some(b';' | b',' | b'!' | b'&' | b'$' | b'~') => {
                    return Err(self.unsupported("query or extended bracket notation"));
                }
                _ => {
                    return Err(
                        self.syntax("expected closing bracket after ordered atom properties")
                    );
                }
            }
            atom.range.end = self.offset;
        }
        Ok(atom)
    }
}
