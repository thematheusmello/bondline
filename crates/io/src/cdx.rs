//! Read-only elemental CDX projection with authoritative borrowed binary source.

use bondline_cdx::{Document, RecordKind};
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Element, Fragment, FragmentId, GraphError,
    MAX_GRAPH_ITEMS, Point2,
};
use std::collections::{BTreeMap, BTreeSet};

const DOCUMENT: u16 = 0x8000;
const PAGE: u16 = 0x8001;
const GROUP: u16 = 0x8002;
const FRAGMENT: u16 = 0x8003;
const NODE: u16 = 0x8004;
const BOND: u16 = 0x8005;
const POSITION: u16 = 0x0200;
const NODE_TYPE: u16 = 0x0400;
const ELEMENT: u16 = 0x0402;
const ORDER: u16 = 0x0600;
const BEGIN: u16 = 0x0604;
const END: u16 = 0x0605;

#[derive(Debug, thiserror::Error)]
pub enum MappingError {
    #[error(transparent)]
    Syntax(#[from] bondline_cdx::Error),
    #[error(transparent)]
    Graph(#[from] GraphError),
    #[error(transparent)]
    PreviewModel(#[from] bondline_model::Error),
    #[error("invalid CDX property {tag:#06x} at record {record}")]
    InvalidProperty { tag: u16, record: usize },
    #[error("missing CDX property {tag:#06x} on object record {record}")]
    MissingProperty { tag: u16, record: usize },
    #[error("duplicate mapped CDX property {tag:#06x} at record {record}")]
    DuplicateProperty { tag: u16, record: usize },
    #[error("source id {id} repeats in its containing object at record {record}")]
    DuplicateSourceId { id: u32, record: usize },
    #[error("zero is an unassigned source id, not a bond endpoint, at record {record}")]
    UnassignedReference { record: usize },
    #[error("bond endpoint {id} is not an atom in the same fragment at record {record}")]
    UnresolvedReference { id: u32, record: usize },
    #[error("unsupported object {tag:#06x} at record {record}")]
    UnsupportedObject { tag: u16, record: usize },
    #[error("unsupported object containment at record {record}")]
    UnsupportedLayout { record: usize },
    #[error("the elemental view supports at most one direct page, at record {record}")]
    UnsupportedPageLayout { record: usize },
    #[error("unmapped standard chemical/fragment property {tag:#06x} at record {record}")]
    UnsupportedProperty { tag: u16, record: usize },
    #[error("only Element nodes are mapped, at record {record}")]
    UnsupportedNodeType { record: usize },
    #[error("only single bonds are mapped, at record {record}")]
    UnsupportedBondOrder { record: usize },
    #[error("elemental mapping exceeds the application identity budget")]
    IdentityLimit,
}

impl MappingError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Syntax(
                bondline_cdx::Error::InputTooLarge
                | bondline_cdx::Error::RecordLimit
                | bondline_cdx::Error::DepthLimit
                | bondline_cdx::Error::Allocation,
            )
            | Self::Graph(GraphError::Limit(_))
            | Self::PreviewModel(bondline_model::Error::Graph(GraphError::Limit(_)))
            | Self::IdentityLimit => "resource_limit",
            Self::UnsupportedObject { .. }
            | Self::UnsupportedLayout { .. }
            | Self::UnsupportedPageLayout { .. }
            | Self::UnsupportedProperty { .. }
            | Self::UnsupportedNodeType { .. }
            | Self::UnsupportedBondOrder { .. } => "unsupported_content",
            _ => "invalid_document",
        }
    }
}

/// A limited immutable view, not a complete scene or an editable/native file profile.
/// Unknown properties/custom objects remain in `source`; no model-to-CDX writer is provided.
#[derive(Debug)]
pub struct ElementalGraph<'input> {
    source: Document<'input>,
    fragments: Vec<Fragment>,
}

impl<'input> ElementalGraph<'input> {
    pub fn parse(input: &'input [u8]) -> Result<Self, MappingError> {
        let source = Document::parse(input)?;
        let layout = Layout::read(&source)?;
        let (mut next_fragment, mut next_atom, mut next_bond) = (0, 0, 0);
        let mut fragments = Vec::new();
        for index in layout.fragments.iter().copied() {
            let children = layout
                .children
                .get(index)
                .ok_or(MappingError::UnsupportedLayout { record: index })?;
            properties(&source, children, &[])?;
            let mut atoms = Vec::new();
            let mut references = BTreeMap::new();
            for child in children.iter().copied() {
                if let Some((NODE, source_id)) = object(&source, Some(child)) {
                    let id = AtomId::new(allocate(&mut next_atom)?)
                        .ok_or(MappingError::IdentityLimit)?;
                    atoms.push(read_atom(&source, &layout, child, id)?);
                    if source_id != 0 {
                        references.insert(source_id, id);
                    }
                }
            }
            let mut bonds = Vec::new();
            for child in children.iter().copied() {
                if let Some((BOND, _)) = object(&source, Some(child)) {
                    let fields = properties(
                        &source,
                        layout
                            .children
                            .get(child)
                            .ok_or(MappingError::UnsupportedLayout { record: child })?,
                        &[ORDER, BEGIN, END],
                    )?;
                    if let Some(record) = fields.get(&ORDER)
                        && u16::from_le_bytes(payload(&source, *record, ORDER)?) != 1
                    {
                        return Err(MappingError::UnsupportedBondOrder { record: *record });
                    }
                    let endpoint = |tag| -> Result<AtomId, MappingError> {
                        let record = fields
                            .get(&tag)
                            .copied()
                            .ok_or(MappingError::MissingProperty { tag, record: child })?;
                        let id = u32::from_le_bytes(payload(&source, record, tag)?);
                        if id == 0 {
                            return Err(MappingError::UnassignedReference { record });
                        }
                        references
                            .get(&id)
                            .copied()
                            .ok_or(MappingError::UnresolvedReference { id, record })
                    };
                    bonds.push(Bond {
                        id: BondId::new(allocate(&mut next_bond)?)
                            .ok_or(MappingError::IdentityLimit)?,
                        begin: endpoint(BEGIN)?,
                        end: endpoint(END)?,
                        order: BondOrder::Single,
                    });
                }
            }
            fragments.push(Fragment::new(
                FragmentId::new(allocate(&mut next_fragment)?)
                    .ok_or(MappingError::IdentityLimit)?,
                atoms,
                bonds,
            )?);
        }
        Ok(Self { source, fragments })
    }

    pub fn fragments(&self) -> &[Fragment] {
        &self.fragments
    }

    pub fn source(&self) -> &Document<'input> {
        &self.source
    }

    pub fn as_bytes(&self) -> &'input [u8] {
        self.source.as_bytes()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.source.to_bytes()
    }

    pub(crate) fn preview_document(
        &self,
        size: bondline_model::PageSize,
    ) -> Result<bondline_model::Document, MappingError> {
        let mut model = bondline_model::Document::default();
        let page = model
            .pages()
            .first()
            .ok_or(MappingError::UnsupportedPageLayout { record: 0 })?
            .id();
        model.set_page_size(page, size)?;
        model.append_fragments(page, self.fragments.clone())?;
        Ok(model)
    }
}

struct Layout {
    children: Vec<Vec<usize>>,
    fragments: Vec<usize>,
}

impl Layout {
    fn read(source: &Document<'_>) -> Result<Self, MappingError> {
        let mut children = vec![Vec::new(); source.records().len()];
        let mut fragments = Vec::new();
        let mut identities = BTreeSet::new();
        let mut page_seen = false;
        for (index, record) in source.records().iter().enumerate() {
            if let RecordKind::Property { tag, .. } = record.kind() {
                let owner = match *tag {
                    POSITION | NODE_TYPE | ELEMENT => Some(NODE),
                    ORDER | BEGIN | END => Some(BOND),
                    _ => None,
                };
                if let Some(owner) = owner
                    && object(source, record.parent()).map(|(tag, _)| tag) != Some(owner)
                {
                    return Err(MappingError::UnsupportedProperty {
                        tag: *tag,
                        record: index,
                    });
                }
            }
            if matches!(
                record.kind(),
                RecordKind::ObjectStart { .. } | RecordKind::Property { .. }
            ) && let Some(parent) = record.parent()
            {
                children
                    .get_mut(parent)
                    .ok_or(MappingError::UnsupportedLayout { record: index })?
                    .push(index);
            }
            let RecordKind::ObjectStart { tag, id } = *record.kind() else {
                continue;
            };
            if id != 0 && !identities.insert((record.parent(), id)) {
                return Err(MappingError::DuplicateSourceId { id, record: index });
            }
            let parent = object(source, record.parent()).map(|(tag, _)| tag);
            match tag {
                DOCUMENT if index == 0 && record.parent().is_none() => {}
                PAGE if record.parent().is_none() || parent == Some(DOCUMENT) => {
                    if page_seen {
                        return Err(MappingError::UnsupportedPageLayout { record: index });
                    }
                    page_seen = true;
                }
                GROUP | FRAGMENT if matches!(parent, Some(PAGE | GROUP)) => {
                    if tag == FRAGMENT {
                        fragments.push(index);
                    }
                }
                NODE | BOND if parent == Some(FRAGMENT) => {}
                DOCUMENT | PAGE | GROUP | FRAGMENT | NODE | BOND => {
                    return Err(MappingError::UnsupportedLayout { record: index });
                }
                tag if tag & 0x4000 != 0 => {}
                _ => return Err(MappingError::UnsupportedObject { tag, record: index }),
            }
        }
        Ok(Self {
            children,
            fragments,
        })
    }
}

fn object(source: &Document<'_>, index: Option<usize>) -> Option<(u16, u32)> {
    match source.records().get(index?).map(|record| record.kind()) {
        Some(RecordKind::ObjectStart { tag, id }) => Some((*tag, *id)),
        _ => None,
    }
}

fn properties(
    source: &Document<'_>,
    children: &[usize],
    supported: &[u16],
) -> Result<BTreeMap<u16, usize>, MappingError> {
    let mut fields = BTreeMap::new();
    for index in children.iter().copied() {
        let Some(RecordKind::Property { tag, .. }) = source.records().get(index).map(|r| r.kind())
        else {
            continue;
        };
        if tag & 0x4000 != 0 {
            continue;
        }
        if !supported.contains(tag) {
            return Err(MappingError::UnsupportedProperty {
                tag: *tag,
                record: index,
            });
        }
        if fields.insert(*tag, index).is_some() {
            return Err(MappingError::DuplicateProperty {
                tag: *tag,
                record: index,
            });
        }
    }
    Ok(fields)
}

fn payload<const N: usize>(
    source: &Document<'_>,
    index: usize,
    tag: u16,
) -> Result<[u8; N], MappingError> {
    let invalid = || MappingError::InvalidProperty { tag, record: index };
    let record = source.records().get(index).ok_or_else(invalid)?;
    source.payload(record)?.try_into().map_err(|_| invalid())
}

fn read_atom(
    source: &Document<'_>,
    layout: &Layout,
    index: usize,
    id: AtomId,
) -> Result<Atom, MappingError> {
    let fields = properties(
        source,
        layout
            .children
            .get(index)
            .ok_or(MappingError::UnsupportedLayout { record: index })?,
        &[POSITION, NODE_TYPE, ELEMENT],
    )?;
    if let Some(record) = fields.get(&NODE_TYPE)
        && i16::from_le_bytes(payload(source, *record, NODE_TYPE)?) != 1
    {
        return Err(MappingError::UnsupportedNodeType { record: *record });
    }
    let element = if let Some(record) = fields.get(&ELEMENT) {
        u8::try_from(i16::from_le_bytes(payload(source, *record, ELEMENT)?))
            .ok()
            .and_then(Element::new)
            .ok_or(MappingError::InvalidProperty {
                tag: ELEMENT,
                record: *record,
            })?
    } else {
        Element::new(6).ok_or(MappingError::InvalidProperty {
            tag: ELEMENT,
            record: index,
        })?
    };
    let position = fields
        .get(&POSITION)
        .map(|record| {
            let [a, b, c, d, e, f, g, h] = payload(source, *record, POSITION)?;
            let y = f64::from(i32::from_le_bytes([a, b, c, d])) / 65_536.0;
            let x = f64::from(i32::from_le_bytes([e, f, g, h])) / 65_536.0;
            Point2::new(x, y).map_err(|_| MappingError::InvalidProperty {
                tag: POSITION,
                record: *record,
            })
        })
        .transpose()?;
    Ok(Atom::new(id, element, position))
}

fn allocate(counter: &mut u32) -> Result<u32, MappingError> {
    *counter = counter
        .checked_add(1)
        .filter(|&value| u64::from(value) <= MAX_GRAPH_ITEMS as u64)
        .ok_or(MappingError::IdentityLimit)?;
    Ok(*counter)
}
