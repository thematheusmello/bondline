//! Exact-source legacy RXN containers, not an editable reaction model.
//! Framing follows BIOVIA Chemistry2026 CTfile Formats, printed91–92. Mixed
//! constituent MOL versions and omitted zero-reagent counts are explicit tested
//! public-consumer extensions. Neither role nor molecule metadata is flattened.

mod parse;
mod write;
use crate::mol::{self, Version};
use std::ops::Range;
pub use write::write_v2000;

pub const MAX_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_COMPONENTS: usize = 1024;
pub const MAX_GRAPH_ITEMS: usize = 100_000;
pub const MAX_LINES: usize = 1_000_000;
pub const MAX_LINE_BYTES: usize = 4096;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("RXN resource limit: {0}")]
    Resource(&'static str),
    #[error("invalid RXN {field} at line {line}")]
    Syntax { line: usize, field: &'static str },
    #[error("unsupported RXN {feature} at line {line}; retain original bytes")]
    Unsupported { line: usize, feature: &'static str },
    #[error("MOL syntax in RXN component {component}: {source}")]
    MolV2000 {
        component: usize,
        #[source]
        source: mol::MolError,
    },
    #[error("MOL syntax in RXN component {component}: {source}")]
    MolV3000 {
        component: usize,
        #[source]
        source: mol::v3000::Error,
    },
}
impl Error {
    pub fn code(&self) -> &'static str {
        let code = match self {
            Self::Resource(_) => "resource_limit",
            Self::Syntax { .. } => "invalid_rxn",
            Self::Unsupported { .. } => "unsupported_content",
            Self::MolV2000 { source, .. } => source.code(),
            Self::MolV3000 { source, .. } => source.code(),
        };
        if code == "invalid_mol" {
            "invalid_rxn"
        } else {
            code
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Reactant,
    Product,
    Reagent,
}
impl Role {
    fn rank(self) -> usize {
        match self {
            Self::Reactant => 0,
            Self::Product => 1,
            Self::Reagent => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub reactants: usize,
    pub products: usize,
    pub reagents: usize,
}
impl Counts {
    fn total(self) -> Result<usize, Error> {
        self.reactants
            .checked_add(self.products)
            .and_then(|value| value.checked_add(self.reagents))
            .filter(|value| *value <= MAX_COMPONENTS)
            .ok_or(Error::Resource("more than 1024 components"))
    }
}

pub struct Component<'a> {
    role: Role,
    version: Version,
    range: Range<usize>,
    mol_range: Range<usize>,
    mol_bytes: &'a [u8],
}
impl Component<'_> {
    pub fn role(&self) -> Role {
        self.role
    }
    pub fn version(&self) -> Version {
        self.version
    }
    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }
    pub fn mol_range(&self) -> Range<usize> {
        self.mol_range.clone()
    }
    pub fn mol_bytes(&self) -> &[u8] {
        self.mol_bytes
    }
}
impl std::fmt::Debug for Component<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RxnComponent")
            .field("role", &self.role)
            .field("version", &self.version)
            .field("range", &self.range)
            .finish()
    }
}
pub struct RxnFile<'a> {
    source: &'a [u8],
    header: [&'a str; 3],
    header_ranges: [Range<usize>; 3],
    counts: Counts,
    counts_range: Range<usize>,
    components: Vec<Component<'a>>,
}
impl<'a> RxnFile<'a> {
    pub fn as_bytes(&self) -> &'a [u8] {
        self.source
    }
    /// Legacy wrapper version; constituent MOL versions are reported independently.
    pub fn container_version(&self) -> Version {
        Version::V2000
    }
    pub fn header(&self) -> [&'a str; 3] {
        self.header
    }
    pub fn header_ranges(&self) -> [Range<usize>; 3] {
        self.header_ranges.clone()
    }
    pub fn counts(&self) -> Counts {
        self.counts
    }
    pub fn counts_range(&self) -> Range<usize> {
        self.counts_range.clone()
    }
    pub fn components(&self) -> &[Component<'a>] {
        &self.components
    }
}
impl std::fmt::Debug for RxnFile<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RxnFile")
            .field("byte_length", &self.source.len())
            .field("counts", &self.counts)
            .finish()
    }
}

/// New wrapper input. Ordered roles and complete original MOL bytes are explicit;
/// this does not authorize regenerating an unmodeled reaction from a projection.
#[derive(Clone, Copy, Debug)]
pub struct ComponentInput<'a> {
    pub role: Role,
    pub mol_bytes: &'a [u8],
}
