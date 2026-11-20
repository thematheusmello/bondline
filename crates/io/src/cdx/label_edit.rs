//! Scope the fields of a source-certified CDX label.
use super::{
    BOUNDING_BOX, ELEMENT, FRAGMENT, HYDROGENS, MappingError, NODE, TEXT, TEXT_OBJECT, object,
};
use bondline_cdx::{Document, PropertyInsertion, Record, RecordKind};
use bondline_model::{AtomId, Element};

pub(crate) struct LabelFields<'record, 'input> {
    element: Option<&'record Record>,
    hydrogens: &'record Record,
    text: &'record Record,
    bounds: &'record Record,
    run_prefix: &'input [u8],
}

impl<'record, 'input> LabelFields<'record, 'input> {
    // The closed Page → Fragment → Node projection assigns AtomIds in this order.
    pub(crate) fn find(source: &'record Document<'input>, atom: AtomId) -> Option<Self> {
        let node = node_index(source, atom)?;
        let mut labels = source.records().iter().enumerate().filter(|(_, record)| {
            record.parent() == Some(node)
                && matches!(
                    record.kind(),
                    RecordKind::ObjectStart {
                        tag: TEXT_OBJECT,
                        ..
                    }
                )
        });
        let (label, _) = labels.next()?;
        if labels.next().is_some() {
            return None;
        }
        let element = owned_property(source, node, ELEMENT)?;
        let hydrogens = owned_property(source, node, HYDROGENS).flatten()?;
        let text = owned_property(source, label, TEXT).flatten()?;
        let bounds = owned_property(source, label, BOUNDING_BOX).flatten()?;
        let string = source.string(text).ok()?;
        let mut runs = string.runs();
        if let Some(element) = element
            && source.payload(element).ok()?.len() != 2
        {
            return None;
        }
        if source.payload(hydrogens).ok()?.len() != 2
            || source.payload(bounds).ok()?.len() != 16
            || runs.len() != 1
            || runs.next()?.start() != 0
        {
            return None;
        }
        Some(Self {
            element,
            hydrogens,
            text,
            bounds,
            run_prefix: source
                .payload(text)
                .ok()?
                .strip_suffix(string.text_bytes())?,
        })
    }

    pub(crate) fn replace(
        &self,
        source: &Document<'_>,
        element: Element,
        hydrogens: u16,
        body: &str,
        bounds: &[u8],
        preserve_lengths: bool,
    ) -> Result<Vec<u8>, MappingError> {
        let mut text = self.run_prefix.to_vec();
        text.extend_from_slice(body.as_bytes());
        let element_word = i16::from(element.get()).to_le_bytes();
        let hydrogen_word = hydrogens.to_le_bytes();
        let changes = [
            (self.hydrogens, hydrogen_word.as_slice()),
            (self.text, text.as_slice()),
            (self.bounds, bounds),
        ];
        let output = match self.element {
            Some(record) => {
                let [h, text, bounds] = changes;
                let changes = [(record, element_word.as_slice()), h, text, bounds];
                if preserve_lengths {
                    source.replace_properties_preserving_lengths(&changes)
                } else {
                    source.replace_properties(&changes)
                }
            }
            // The published default carries the element through a carbon H-only edit.
            None if element.get() == 6 => source.replace_properties(&changes),
            None => source.edit_properties(
                &changes,
                &[PropertyInsertion {
                    before: self.hydrogens,
                    tag: ELEMENT,
                    value: &element_word,
                }],
            ),
        }?;
        Ok(output)
    }
}

// Atom IDs follow the source's Fragment/Node encounter order, including repeated raw IDs.
fn node_index(source: &Document<'_>, atom: AtomId) -> Option<usize> {
    let ordinal = usize::try_from(atom.get()).ok()?.checked_sub(1)?;
    let (node, _) = source
        .records()
        .iter()
        .enumerate()
        .filter(|(_, record)| {
            matches!(record.kind(), RecordKind::ObjectStart { tag: NODE, .. })
                && matches!(object(source, record.parent()), Some((FRAGMENT, _)))
        })
        .nth(ordinal)?;
    Some(node)
}

fn owned_property<'record>(
    source: &'record Document<'_>,
    parent: usize,
    tag: u16,
) -> Option<Option<&'record Record>> {
    let mut records = source.records().iter().filter(|record| {
        record.parent() == Some(parent)
            && matches!(record.kind(), RecordKind::Property { tag: field, .. } if *field == tag)
    });
    let record = records.next();
    records.next().is_none().then_some(record)
}

pub(crate) fn insert_label(
    source: &Document<'_>,
    atom: AtomId,
    element: Element,
    hydrogens: u16,
    run: &bondline_model::TextRun,
    bounds: &[u8],
) -> Result<Vec<u8>, crate::DocumentError> {
    use super::atom_label::{JUSTIFICATION, LABEL_JUSTIFICATION};
    use crate::DocumentError::UnsupportedCdxLabel;
    let site_error = || UnsupportedCdxLabel("an unlabelled carbon Node with an owned position");
    let node = node_index(source, atom).ok_or_else(site_error)?;
    let position = owned_property(source, node, super::POSITION)
        .flatten()
        .ok_or_else(site_error)?;
    if source
        .records()
        .iter()
        .any(|r| r.parent() == Some(node) && matches!(r.kind(), RecordKind::ObjectStart { .. }))
    {
        return Err(site_error());
    }
    let resources = source
        .resources()
        .map_err(|_| UnsupportedCdxLabel("declared CDX resources"))?;
    let style = run.style();
    let font = super::caption::creation_font_id(&resources)
        .filter(|_| style.font_family() == "Roboto" && style.face() == 0)
        .ok_or(UnsupportedCdxLabel("a declared UTF-8 Roboto Regular font"))?;
    let size = style.size_pt() * 20.0;
    if size.fract() != 0.0 || !(1.0..=f64::from(u16::MAX)).contains(&size) {
        return Err(UnsupportedCdxLabel(
            "an exact positive size in twentieths of a point",
        ));
    }
    let ink = [0, 1]
        .into_iter()
        .chain(resources.colors().iter().map(|item| item.index()))
        .find(|&i| {
            resources
                .color(i)
                .is_ok_and(|rgb| rgb.map(|v| f64::from(v) / f64::from(u16::MAX)) == style.rgb())
        })
        .and_then(|i| u16::try_from(i).ok())
        .ok_or(UnsupportedCdxLabel("an exact existing run colour"))?;
    let words = [1, 0, font, 0, size as u16, ink].map(u16::to_le_bytes);
    let mut text = words.as_flattened().to_vec();
    text.extend_from_slice(run.text().as_bytes());

    let id =
        super::next_object_id(source).ok_or(UnsupportedCdxLabel("an available child object ID"))?;
    let element = i16::from(element.get()).to_le_bytes();
    let hydrogens = hydrogens.to_le_bytes();
    let mut replacements = Vec::new();
    let mut insertions = Vec::new();
    for (tag, value) in [
        (ELEMENT, element.as_slice()),
        (HYDROGENS, hydrogens.as_slice()),
    ] {
        if let Some(property) = owned_property(source, node, tag).ok_or_else(site_error)? {
            replacements.push((property, value));
        } else {
            insertions.push(PropertyInsertion {
                before: position,
                tag,
                value,
            });
        }
    }
    let updated = source
        .edit_properties_preserving_lengths(&replacements, &insertions)
        .map_err(MappingError::from)?;
    let updated = Document::parse(&updated).map_err(MappingError::from)?;
    let updated_node = node_index(&updated, atom).ok_or_else(site_error)?;
    let end = super::caption::matched_end(&updated, updated_node).ok_or_else(site_error)?;
    Ok(updated
        .append_child(
            end,
            TEXT_OBJECT,
            id,
            &[
                (BOUNDING_BOX, bounds),
                (TEXT, &text),
                (JUSTIFICATION, &[0]),
                (LABEL_JUSTIFICATION, &[0]),
            ],
        )
        .map_err(MappingError::from)?)
}
