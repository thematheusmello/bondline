use super::{is_element, parse_position};
use bondline_cdxml::{ObjectId, parse_integer};
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Element, Fragment, FragmentId, GraphError,
    MAX_GRAPH_ITEMS, Radical,
};
use roxmltree::Node;
use std::{
    collections::{BTreeMap, BTreeSet},
    num::NonZeroU16,
};

#[derive(Debug, thiserror::Error)]
pub enum MappingError {
    #[error(transparent)]
    Syntax(#[from] bondline_cdxml::Error),
    #[error(transparent)]
    Graph(#[from] GraphError),
    #[error("invalid or missing CDXML {0} property")]
    InvalidProperty(&'static str),
    #[error("assigned source ids must be unique within their containing object")]
    DuplicateSourceId,
    #[error("zero is an unassigned source id and cannot be referenced")]
    UnassignedReference,
    #[error("bond reference does not resolve to an atom in the same fragment")]
    UnresolvedReference,
    #[error("only Element nodes are currently mapped to the chemical model")]
    UnsupportedNodeType,
    #[error("a labeled node without explicit Element metadata needs chemical label parsing")]
    UnsupportedLabelOnlyAtom,
    #[error("unsupported bond order; query combinations and undefined spellings are not mapped")]
    UnsupportedBondOrder,
    #[error("only fragment nodes/bonds under page/group containment are currently mapped")]
    UnsupportedLayout,
    #[error("elemental mapping exceeds the application identity budget")]
    IdentityLimit,
    #[error("the current application projection supports at most one direct document page")]
    UnsupportedPageLayout,
    #[error(transparent)]
    PreviewModel(#[from] bondline_model::Error),
}

/// Read-only elemental projection plus authoritative, unchanged source backing.
/// Nonchemical objects and unmodelled properties remain in the source, not in the projection.
#[derive(Debug)]
pub struct ElementalGraph<'input> {
    source: bondline_cdxml::Document<'input>,
    fragments: Vec<Fragment>,
}

impl<'input> ElementalGraph<'input> {
    pub fn parse(input: &'input [u8]) -> Result<Self, MappingError> {
        let source = bondline_cdxml::Document::parse(input)?;
        let root = source.xml().root_element();
        for node in source
            .xml()
            .descendants()
            .filter(|node| known_object(*node))
        {
            validate_scope(node)?;
            if (is_element(node, "n") || is_element(node, "b"))
                && !node
                    .parent()
                    .is_some_and(|parent| is_element(parent, "fragment"))
            {
                return Err(MappingError::UnsupportedLayout);
            }
            if is_element(node, "fragment") && !supported_fragment_parent(node, root) {
                return Err(MappingError::UnsupportedLayout);
            }
        }
        let (mut next_atom, mut next_bond, mut next_fragment) = (0, 0, 0);
        let mut fragments = Vec::new();
        for fragment in source
            .xml()
            .descendants()
            .filter(|node| is_element(*node, "fragment"))
        {
            let mut atoms = Vec::new();
            let mut references = BTreeMap::new();
            for node in fragment.children().filter(|node| is_element(*node, "n")) {
                let id =
                    AtomId::new(allocate(&mut next_atom)?).ok_or(MappingError::IdentityLimit)?;
                atoms.push(read_atom(node, id)?);
                if let Some(source_id) = source_id(node)?.filter(|id| id.is_assigned()) {
                    references.insert(source_id, id);
                }
            }
            let mut bonds = Vec::new();
            for node in fragment.children().filter(|node| is_element(*node, "b")) {
                let id =
                    BondId::new(allocate(&mut next_bond)?).ok_or(MappingError::IdentityLimit)?;
                let endpoint = |name| -> Result<AtomId, MappingError> {
                    let source_id = node
                        .attribute(name)
                        .ok_or(MappingError::InvalidProperty(name))?
                        .parse::<ObjectId>()
                        .map_err(|_| MappingError::InvalidProperty(name))?;
                    if !source_id.is_assigned() {
                        return Err(MappingError::UnassignedReference);
                    }
                    references
                        .get(&source_id)
                        .copied()
                        .ok_or(MappingError::UnresolvedReference)
                };
                bonds.push(Bond {
                    id,
                    begin: endpoint("B")?,
                    end: endpoint("E")?,
                    order: read_order(node.attribute("Order"))?,
                });
            }
            let id = FragmentId::new(allocate(&mut next_fragment)?)
                .ok_or(MappingError::IdentityLimit)?;
            fragments.push(Fragment::new(id, atoms, bonds)?);
        }
        Ok(Self { source, fragments })
    }
    pub fn fragments(&self) -> &[Fragment] {
        &self.fragments
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
        let root = self.source.xml().root_element();
        let mut pages = self
            .source
            .xml()
            .descendants()
            .filter(|node| is_element(*node, "page"));
        if let Some(page) = pages.next()
            && (page.parent() != Some(root) || pages.next().is_some())
        {
            return Err(MappingError::UnsupportedPageLayout);
        }
        let mut model = bondline_model::Document::default();
        let id = model
            .pages()
            .first()
            .ok_or(MappingError::UnsupportedPageLayout)?
            .id();
        model.set_page_size(id, size)?;
        model.append_fragments(id, self.fragments.clone())?;
        Ok(model)
    }
}

fn allocate(counter: &mut u32) -> Result<u32, MappingError> {
    *counter = counter
        .checked_add(1)
        .filter(|&value| u64::from(value) <= MAX_GRAPH_ITEMS as u64)
        .ok_or(MappingError::IdentityLimit)?;
    Ok(*counter)
}

fn source_id(node: Node<'_, '_>) -> Result<Option<ObjectId>, MappingError> {
    node.attribute("id")
        .map(|value| {
            value
                .parse()
                .map_err(|_| MappingError::InvalidProperty("id"))
        })
        .transpose()
}

fn known_object(node: Node<'_, '_>) -> bool {
    node.is_element()
        && node.tag_name().namespace().is_none()
        && matches!(
            node.tag_name().name(),
            "CDXML"
                | "page"
                | "group"
                | "fragment"
                | "n"
                | "b"
                | "t"
                | "graphic"
                | "curve"
                | "objecttag"
        )
}

fn validate_scope(node: Node<'_, '_>) -> Result<(), MappingError> {
    let mut ids = BTreeSet::new();
    for child in node.children().filter(|child| known_object(*child)) {
        if let Some(id) = source_id(child)?.filter(|id| id.is_assigned())
            && !ids.insert(id)
        {
            return Err(MappingError::DuplicateSourceId);
        }
    }
    Ok(())
}

fn supported_fragment_parent(fragment: Node<'_, '_>, root: Node<'_, '_>) -> bool {
    let mut ancestors = fragment.ancestors().skip(1);
    for ancestor in &mut ancestors {
        if is_element(ancestor, "group") {
            continue;
        }
        return is_element(ancestor, "page") && ancestor.parent() == Some(root);
    }
    false
}

fn integer(node: Node<'_, '_>, name: &'static str) -> Result<Option<i64>, MappingError> {
    node.attribute(name)
        .map(|value| parse_integer(value).map_err(|_| MappingError::InvalidProperty(name)))
        .transpose()
}

fn read_atom(node: Node<'_, '_>, id: AtomId) -> Result<Atom, MappingError> {
    if node
        .attribute("NodeType")
        .is_some_and(|kind| kind != "Element")
    {
        return Err(MappingError::UnsupportedNodeType);
    }
    if node.attribute("Element").is_none() && node.children().any(|child| is_element(child, "t")) {
        return Err(MappingError::UnsupportedLabelOnlyAtom);
    }
    let element = u8::try_from(integer(node, "Element")?.unwrap_or(6))
        .ok()
        .and_then(Element::new)
        .ok_or(MappingError::InvalidProperty("Element"))?;
    let position = node
        .attribute("p")
        .map(parse_position)
        .transpose()
        .map_err(|_| MappingError::InvalidProperty("p"))?;
    let mut atom = Atom::new(id, element, position);
    let isotope = integer(node, "Isotope")?.unwrap_or(0);
    if !(0..=i64::from(i16::MAX)).contains(&isotope) {
        return Err(MappingError::InvalidProperty("Isotope"));
    }
    atom.isotope = NonZeroU16::new(
        u16::try_from(isotope).map_err(|_| MappingError::InvalidProperty("Isotope"))?,
    );
    atom.charge = i8::try_from(integer(node, "Charge")?.unwrap_or(0))
        .map_err(|_| MappingError::InvalidProperty("Charge"))?;
    atom.explicit_hydrogens = integer(node, "NumHydrogens")?
        .map(|count| {
            u16::try_from(count).map_err(|_| MappingError::InvalidProperty("NumHydrogens"))
        })
        .transpose()?;
    atom.radical = match node.attribute("Radical") {
        None | Some("None") => Radical::None,
        Some("Singlet") => Radical::Singlet,
        Some("Doublet") => Radical::Doublet,
        Some("Triplet") => Radical::Triplet,
        _ => return Err(MappingError::InvalidProperty("Radical")),
    };
    Ok(atom)
}

fn read_order(value: Option<&str>) -> Result<BondOrder, MappingError> {
    let Some(value) = value else {
        return Ok(BondOrder::Single);
    };
    match value {
        "dative" => return Ok(BondOrder::Dative),
        "ionic" => return Ok(BondOrder::Ionic),
        "hydrogen" => return Ok(BondOrder::Hydrogen),
        "threecenter" => return Ok(BondOrder::ThreeCenter),
        _ => {}
    }
    let value = value.trim_matches([' ', '\t', '\r', '\n']);
    if let Ok(integer) = parse_integer(value) {
        return match integer {
            1 => Ok(BondOrder::Single),
            2 => Ok(BondOrder::Double),
            3 => Ok(BondOrder::Triple),
            4 => Ok(BondOrder::Quadruple),
            5 => Ok(BondOrder::Quintuple),
            6 => Ok(BondOrder::Sextuple),
            _ => Err(MappingError::UnsupportedBondOrder),
        };
    }
    if let Some((whole, fraction)) = value.split_once('.')
        && fraction
            .strip_prefix('5')
            .is_some_and(|rest| rest.bytes().all(|byte| byte == b'0'))
        && !whole.starts_with('-')
    {
        return match parse_integer(whole) {
            Ok(0) => Ok(BondOrder::Half),
            Ok(1) => Ok(BondOrder::Aromatic),
            Ok(2) => Ok(BondOrder::TwoAndHalf),
            Ok(3) => Ok(BondOrder::ThreeAndHalf),
            Ok(4) => Ok(BondOrder::FourAndHalf),
            Ok(5) => Ok(BondOrder::FiveAndHalf),
            _ => Err(MappingError::UnsupportedBondOrder),
        };
    }
    Err(MappingError::UnsupportedBondOrder)
}
