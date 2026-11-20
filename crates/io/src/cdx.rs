//! Read-only elemental CDX projection with authoritative borrowed binary source.

use bondline_cdx::{Document, RecordKind};
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, DoublePosition, Element, Fragment, FragmentId,
    GraphError, MAX_GRAPH_ITEMS, Point2, Radical,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    num::NonZeroU16,
    sync::Arc,
};

mod atom_label;
mod caption;
pub(crate) mod label_edit;
mod label_preview;
mod mixed;
use mixed::Scope;
mod text;
pub(crate) use caption::{append_caption, text_record as caption_text_record};
pub(crate) use text::encode_caption_runs;

/// In the certified skeletal layout, every directly owned Node has one position
/// and direct Fragment/Node encounter order supplies the corresponding live atom IDs.
pub(crate) fn skeletal_positions<'a>(
    source: &'a Document<'_>,
) -> impl Iterator<Item = std::ops::Range<usize>> + 'a {
    source.records().iter().filter_map(|r| {
        let RecordKind::Property {
            tag: POSITION,
            payload,
            ..
        } = r.kind()
        else {
            return None;
        };
        matches!(object(source, r.parent()), Some((NODE, _))).then(|| payload.clone())
    })
}

const DOCUMENT: u16 = 0x8000;
const PAGE: u16 = 0x8001;
const GROUP: u16 = 0x8002;
const FRAGMENT: u16 = 0x8003;
const NODE: u16 = 0x8004;
const BOND: u16 = 0x8005;
const TEXT_OBJECT: u16 = 0x8006;
const Z_ORDER: u16 = 0x000a;
const WARNING: u16 = 0x0010;
const FONT_TABLE: u16 = 0x0100;
const POSITION: u16 = 0x0200;
const BOUNDING_BOX: u16 = 0x0204;
const COLOR_TABLE: u16 = 0x0300;
const NODE_TYPE: u16 = 0x0400;
const ELEMENT: u16 = 0x0402;
const ISOTOPE: u16 = 0x0420;
const CHARGE: u16 = 0x0421;
const RADICAL: u16 = 0x0422;
const HYDROGENS: u16 = 0x042b;
const SHOW_TERMINAL_CARBONS: u16 = 0x0442;
const SHOW_NON_TERMINAL_CARBONS: u16 = 0x0443;
const ATOM_CIP: u16 = 0x0437;
const ATOM_ID: u16 = 0x044d;
const ORDER: u16 = 0x0600;
const DISPLAY: u16 = 0x0601;
const DISPLAY2: u16 = 0x0602;
const DOUBLE_POSITION: u16 = 0x0603;
const BEGIN: u16 = 0x0604;
const END: u16 = 0x0605;
const BOND_CIP: u16 = 0x060a;
const TEXT: u16 = 0x0700;
const CHAIN_ANGLE: u16 = 0x0803;
const BOND_LENGTH: u16 = 0x0805;
const DRAWING_SPACE: u16 = 0x0811;
const WIDTH: u16 = 0x0812;
const HEIGHT: u16 = 0x0813;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Preview {
    Elemental,
    Caption,
    AtomLabels,
    Mixed,
    Skeletal,
}

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
    #[error("only plain Single, Double and Triple bond orders are mapped, at record {record}")]
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
        Self::from_source(Document::parse(input)?)
    }

    fn from_source(source: Document<'input>) -> Result<Self, MappingError> {
        let label_candidate = source.records().iter().any(|record| {
            matches!(
                record.kind(),
                RecordKind::ObjectStart {
                    tag: TEXT_OBJECT,
                    ..
                }
            ) && object(&source, record.parent()).is_some_and(|(owner, _)| owner == NODE)
        });
        // Failed label candidates keep the original elemental diagnostic and its precedence.
        let fragments = if label_candidate {
            Self::read_fragments(&source, Nodes::Labelled)
                .or_else(|_| Self::read_fragments(&source, Nodes::Elemental))?
        } else {
            Self::read_fragments(&source, Nodes::Elemental)?
        };
        Ok(Self { source, fragments })
    }

    fn read_fragments(source: &Document<'_>, nodes: Nodes) -> Result<Vec<Fragment>, MappingError> {
        let layout = Layout::read(source, nodes, Scope::All)?;
        let resources = layout
            .node_labels
            .first_key_value()
            .map(|(_, &record)| {
                source
                    .resources()
                    .map_err(|_| MappingError::UnsupportedObject {
                        tag: TEXT_OBJECT,
                        record,
                    })
            })
            .transpose()?;
        let mut remaining_runs = MAX_GRAPH_ITEMS;
        Self::decode_fragments(source, &layout, resources.as_ref(), &mut remaining_runs)
    }

    fn decode_fragments(
        source: &Document<'_>,
        layout: &Layout,
        resources: Option<&bondline_cdx::Resources<'_, '_>>,
        remaining_runs: &mut usize,
    ) -> Result<Vec<Fragment>, MappingError> {
        let (mut next_fragment, mut next_atom, mut next_bond) = (0, 0, 0);
        let mut fragments = Vec::new();
        for index in layout.fragments.iter().copied() {
            let children = layout
                .children
                .get(index)
                .ok_or(MappingError::UnsupportedLayout { record: index })?;
            // Fragment Z is a qualified public graphic-base compatibility case.
            let fields = properties(source, children, &[Z_ORDER, BOUNDING_BOX])?;
            validate_metadata(source, &fields)?;
            let mut atoms = Vec::new();
            let mut references = BTreeMap::new();
            for child in children.iter().copied() {
                if let Some((NODE, source_id)) = object(source, Some(child)) {
                    let id = AtomId::new(allocate(&mut next_atom)?)
                        .ok_or(MappingError::IdentityLimit)?;
                    let mut atom = read_atom(source, layout, child, id)?;
                    if let (Some(resources), Some(&label)) =
                        (resources, layout.node_labels.get(&child))
                    {
                        let children = layout
                            .children
                            .get(label)
                            .ok_or(MappingError::UnsupportedLayout { record: label })?;
                        atom.authored_label = Some(Arc::new(atom_label::read(
                            source,
                            resources,
                            label,
                            children,
                            remaining_runs,
                        )?));
                    }
                    atoms.push(atom);
                    if source_id != 0 {
                        references.insert(source_id, id);
                    }
                }
            }
            let mut bonds = Vec::new();
            for child in children.iter().copied() {
                if let Some((BOND, _)) = object(source, Some(child)) {
                    let fields = properties(
                        source,
                        layout
                            .children
                            .get(child)
                            .ok_or(MappingError::UnsupportedLayout { record: child })?,
                        &[
                            ORDER,
                            DISPLAY,
                            DISPLAY2,
                            DOUBLE_POSITION,
                            BEGIN,
                            END,
                            Z_ORDER,
                            WARNING,
                            BOND_CIP,
                        ],
                    )?;
                    validate_metadata(source, &fields)?;
                    let (order, double_position) = read_bond_style(source, &fields)?;
                    let endpoint = |tag| -> Result<AtomId, MappingError> {
                        let record = fields
                            .get(&tag)
                            .copied()
                            .ok_or(MappingError::MissingProperty { tag, record: child })?;
                        let id = u32::from_le_bytes(payload(source, record, tag)?);
                        if id == 0 {
                            return Err(MappingError::UnassignedReference { record });
                        }
                        references
                            .get(&id)
                            .copied()
                            .ok_or(MappingError::UnresolvedReference { id, record })
                    };
                    bonds.push(Bond {
                        double_position,
                        id: BondId::new(allocate(&mut next_bond)?)
                            .ok_or(MappingError::IdentityLimit)?,
                        begin: endpoint(BEGIN)?,
                        end: endpoint(END)?,
                        order,
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
        Ok(fragments)
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
}

fn project_fragments(
    source: &Document<'_>,
    fragments: &[Fragment],
    size: bondline_model::PageSize,
) -> Result<bondline_model::Document, MappingError> {
    let mut model = bondline_model::Document::default();
    model.set_drawing_settings(crate::drawing_settings::project(
        root_scalar(source, BOND_LENGTH),
        root_scalar(source, CHAIN_ANGLE),
    )?);
    let page = model
        .pages()
        .first()
        .ok_or(MappingError::UnsupportedPageLayout { record: 0 })?
        .id();
    model.set_page_size(page, size)?;
    model.append_fragments(page, fragments.to_vec())?;
    Ok(model)
}

pub(crate) fn project_document(
    input: &[u8],
    preview_size: bondline_model::PageSize,
) -> Result<(bondline_model::Document, Preview), MappingError> {
    let source = Document::parse(input)?;
    if let Some(model) = caption::preview(&source) {
        return Ok((model, Preview::Caption));
    }
    if let Some(model) = mixed::preview(&source) {
        return Ok((model, Preview::Mixed));
    }
    if let Some(model) = label_preview::skeletal(&source) {
        return Ok((model, Preview::Skeletal));
    }
    let graph = ElementalGraph::from_source(source)?;
    if let Some(model) = label_preview::preview(&graph) {
        return Ok((model, Preview::AtomLabels));
    }
    Ok((
        project_fragments(graph.source(), graph.fragments(), preview_size)?,
        Preview::Elemental,
    ))
}

fn has_preview_header(source: &Document<'_>) -> bool {
    source
        .header()
        .get(12..)
        .is_some_and(|bytes| bytes.iter().all(|&byte| byte == 0))
}

fn poster_size(
    source: &Document<'_>,
    fields: &BTreeMap<u16, usize>,
) -> Option<bondline_model::PageSize> {
    if payload(source, *fields.get(&DRAWING_SPACE)?, DRAWING_SPACE).ok()? != [1] {
        return None;
    }
    let width = coordinate(payload(source, *fields.get(&WIDTH)?, WIDTH).ok()?);
    let height = coordinate(payload(source, *fields.get(&HEIGHT)?, HEIGHT).ok()?);
    bondline_model::PageSize::new(width, height).ok()
}

pub(crate) fn preview_view(model: &bondline_model::Document) -> Option<bondline_render::PageView> {
    let size = model.pages().first()?.size();
    let center = Point2::new(size.width_pt() / 2.0, size.height_pt() / 2.0).ok()?;
    Some(
        bondline_render::PageView::zoom(1, 1, size, 100.0, center)
            .ok()?
            .with_drawing_style(model.drawing_style())
            .with_drawing_settings(model.drawing_settings()),
    )
}

fn verify_preview_page(model: &bondline_model::Document) -> Option<()> {
    // Check the source paper before clipping to the admission camera.
    preview_view(model)?
        .graph_scene(
            model.pages().first()?,
            &bondline_render::Cancellation::default(),
        )
        .ok()?;
    Some(())
}

fn root_scalar(source: &Document<'_>, tag: u16) -> Option<f64> {
    let mut fields = source.records().iter().filter(|record| {
        matches!(record.kind(), RecordKind::Property { tag: field, .. } if *field == tag)
            && (record.parent().is_none()
                || object(source, record.parent()).is_some_and(|(owner, _)| owner == DOCUMENT))
    });
    let record = fields.next()?;
    if fields.next().is_some() {
        return None;
    }
    let bytes: [u8; 4] = source.payload(record).ok()?.try_into().ok()?;
    Some(f64::from(i32::from_le_bytes(bytes)) / 65_536.0)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Nodes {
    Elemental,
    Labelled,
    Skeletal,
}

struct Layout {
    nodes: Nodes,
    children: Vec<Vec<usize>>,
    fragments: Vec<usize>,
    node_labels: BTreeMap<usize, usize>,
}

impl Layout {
    fn read(source: &Document<'_>, nodes: Nodes, scope: Scope) -> Result<Self, MappingError> {
        let mut children = vec![Vec::new(); source.records().len()];
        let mut fragments = Vec::new();
        let mut node_labels = BTreeMap::new();
        let mut identities = BTreeSet::new();
        let mut page_seen = false;
        for (index, record) in scope.records(source) {
            if let RecordKind::Property { tag, .. } = record.kind() {
                let owner = match *tag {
                    POSITION | NODE_TYPE | ELEMENT | ISOTOPE | CHARGE | RADICAL | HYDROGENS
                    | ATOM_CIP | ATOM_ID => Some(NODE),
                    SHOW_TERMINAL_CARBONS | SHOW_NON_TERMINAL_CARBONS
                        if nodes == Nodes::Skeletal =>
                    {
                        Some(NODE)
                    }
                    ORDER | DISPLAY | DISPLAY2 | DOUBLE_POSITION | BEGIN | END | BOND_CIP
                    | WARNING => Some(BOND),
                    _ => None,
                };
                if let Some(owner) = owner {
                    let actual = object(source, record.parent()).map(|(tag, _)| tag);
                    let label_position =
                        nodes == Nodes::Labelled && *tag == POSITION && actual == Some(TEXT_OBJECT);
                    if actual != Some(owner) && !label_position {
                        return Err(MappingError::UnsupportedProperty {
                            tag: *tag,
                            record: index,
                        });
                    }
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
                TEXT_OBJECT if nodes == Nodes::Labelled && parent == Some(NODE) => {
                    let node = record
                        .parent()
                        .ok_or(MappingError::UnsupportedLayout { record: index })?;
                    if node_labels.insert(node, index).is_some() {
                        return Err(MappingError::UnsupportedObject { tag, record: index });
                    }
                }
                DOCUMENT | PAGE | GROUP | FRAGMENT | NODE | BOND => {
                    return Err(MappingError::UnsupportedLayout { record: index });
                }
                tag if tag & 0x4000 != 0 => {}
                _ => return Err(MappingError::UnsupportedObject { tag, record: index }),
            }
        }
        Ok(Self {
            nodes,
            children,
            fragments,
            node_labels,
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
    property_bytes(source, index, tag)?
        .try_into()
        .map_err(|_| MappingError::InvalidProperty { tag, record: index })
}

fn coordinate(bytes: [u8; 4]) -> f64 {
    f64::from(i32::from_le_bytes(bytes)) / 65_536.0
}

fn read_position(source: &Document<'_>, record: usize) -> Result<Point2, MappingError> {
    let [a, b, c, d, e, f, g, h] = payload(source, record, POSITION)?;
    Point2::new(coordinate([e, f, g, h]), coordinate([a, b, c, d])).map_err(|_| {
        MappingError::InvalidProperty {
            tag: POSITION,
            record,
        }
    })
}

fn property_bytes<'input>(
    source: &Document<'input>,
    index: usize,
    tag: u16,
) -> Result<&'input [u8], MappingError> {
    let invalid = || MappingError::InvalidProperty { tag, record: index };
    let record = source.records().get(index).ok_or_else(invalid)?;
    Ok(source.payload(record)?)
}

fn validate_metadata(
    source: &Document<'_>,
    fields: &BTreeMap<u16, usize>,
) -> Result<(), MappingError> {
    for (&tag, &record) in fields {
        match tag {
            Z_ORDER => {
                payload::<2>(source, record, tag)?;
            }
            BOUNDING_BOX => {
                payload::<16>(source, record, tag)?;
            }
            ATOM_ID => {
                payload::<4>(source, record, tag)?;
            }
            ATOM_CIP | BOND_CIP => {
                let [state] = payload(source, record, tag)?;
                // U (undetermined) and N (symmetric) remain distinct in retained source.
                if state > 1 {
                    return Err(MappingError::UnsupportedProperty { tag, record });
                }
            }
            WARNING => {
                let invalid = || MappingError::InvalidProperty { tag, record };
                let bytes = property_bytes(source, record, tag)?;
                let count = u16::from_le_bytes(
                    bytes
                        .get(..2)
                        .ok_or_else(invalid)?
                        .try_into()
                        .map_err(|_| invalid())?,
                );
                let prefix = usize::from(count)
                    .checked_mul(10)
                    .and_then(|size| size.checked_add(2))
                    .filter(|size| *size <= bytes.len())
                    .ok_or_else(invalid)?;
                if prefix != 2 {
                    return Err(MappingError::UnsupportedProperty { tag, record });
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn read_bond_style(
    source: &Document<'_>,
    fields: &BTreeMap<u16, usize>,
) -> Result<(BondOrder, DoublePosition), MappingError> {
    let order = if let Some(&record) = fields.get(&ORDER) {
        match u16::from_le_bytes(payload(source, record, ORDER)?) {
            1 => BondOrder::Single,
            2 => BondOrder::Double,
            4 => BondOrder::Triple,
            _ => return Err(MappingError::UnsupportedBondOrder { record }),
        }
    } else {
        BondOrder::Single
    };
    let mut position = DoublePosition::Unspecified;
    for tag in [DISPLAY, DISPLAY2, DOUBLE_POSITION] {
        let Some(&record) = fields.get(&tag) else {
            continue;
        };
        let unsupported = || MappingError::UnsupportedProperty { tag, record };
        let bytes = property_bytes(source, record, tag)?
            .try_into()
            .map_err(|_| unsupported())?;
        let value = i16::from_le_bytes(bytes);
        if tag != DISPLAY && order != BondOrder::Double {
            return Err(unsupported());
        }
        if tag == DOUBLE_POSITION {
            position = match value {
                0 | 256 => DoublePosition::Center,
                1 | 257 => DoublePosition::Right,
                2 | 258 => DoublePosition::Left,
                _ => return Err(unsupported()),
            };
        } else if value != 0 {
            return Err(unsupported());
        }
    }
    Ok((order, position))
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
        match layout.nodes {
            Nodes::Skeletal => &[
                POSITION,
                NODE_TYPE,
                ELEMENT,
                ISOTOPE,
                CHARGE,
                RADICAL,
                HYDROGENS,
                SHOW_TERMINAL_CARBONS,
                SHOW_NON_TERMINAL_CARBONS,
            ],
            Nodes::Elemental | Nodes::Labelled => &[
                POSITION, NODE_TYPE, ELEMENT, ISOTOPE, CHARGE, RADICAL, HYDROGENS, Z_ORDER,
                ATOM_CIP, ATOM_ID,
            ],
        },
    )?;
    validate_metadata(source, &fields)?;
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
        .map(|record| read_position(source, *record))
        .transpose()?;
    let mut atom = Atom::new(id, element, position);
    read_literal_metadata(source, &fields, &mut atom)?;
    Ok(atom)
}

fn read_literal_metadata(
    source: &Document<'_>,
    fields: &BTreeMap<u16, usize>,
    atom: &mut Atom,
) -> Result<(), MappingError> {
    for (&tag, &record) in fields {
        let widths: &[usize] = match tag {
            SHOW_TERMINAL_CARBONS | SHOW_NON_TERMINAL_CARBONS => &[0],
            ISOTOPE | HYDROGENS => &[2],
            CHARGE => &[1, 4],
            RADICAL => &[1],
            _ => continue,
        };
        let width = property_bytes(source, record, tag)?.len();
        if !widths.contains(&width) {
            return Err(MappingError::UnsupportedProperty { tag, record });
        }
        let invalid = || MappingError::InvalidProperty { tag, record };
        match tag {
            SHOW_TERMINAL_CARBONS => atom.show_terminal_carbon_labels = Some(true),
            SHOW_NON_TERMINAL_CARBONS => atom.show_non_terminal_carbon_labels = Some(true),
            ISOTOPE => {
                let mass = i16::from_le_bytes(payload(source, record, tag)?);
                atom.isotope = NonZeroU16::new(u16::try_from(mass).map_err(|_| invalid())?);
            }
            CHARGE => {
                atom.charge = if width == 1 {
                    i8::from_le_bytes(payload(source, record, tag)?)
                } else {
                    // The property-specific SDK recovers the signed high byte for 8.0 data.
                    let saved = i32::from_le_bytes(payload(source, record, tag)?);
                    i8::try_from(saved >> 24).map_err(|_| invalid())?
                };
            }
            RADICAL => {
                atom.radical = match payload(source, record, tag)? {
                    [0] => Radical::None,
                    [1] => Radical::Singlet,
                    [2] => Radical::Doublet,
                    [3] => Radical::Triplet,
                    _ => return Err(invalid()),
                };
            }
            HYDROGENS => {
                atom.explicit_hydrogens = Some(u16::from_le_bytes(payload(source, record, tag)?));
            }
            _ => {}
        }
    }
    Ok(())
}

fn allocate(counter: &mut u32) -> Result<u32, MappingError> {
    *counter = counter
        .checked_add(1)
        .filter(|&value| u64::from(value) <= MAX_GRAPH_ITEMS as u64)
        .ok_or(MappingError::IdentityLimit)?;
    Ok(*counter)
}
