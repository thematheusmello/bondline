//! Scope the four existing fields of a source-certified CDX label.
use super::{
    BOUNDING_BOX, ELEMENT, FRAGMENT, HYDROGENS, MappingError, NODE, TEXT, TEXT_OBJECT, object,
};
use bondline_cdx::{Document, Record, RecordKind};
use bondline_model::{AtomId, Element};

pub(crate) struct LabelFields<'record, 'input> {
    element: &'record Record,
    hydrogens: &'record Record,
    text: &'record Record,
    bounds: &'record Record,
    run_prefix: &'input [u8],
}

impl<'record, 'input> LabelFields<'record, 'input> {
    // The closed Page → Fragment → Node projection assigns AtomIds in this order.
    pub(crate) fn find(source: &'record Document<'input>, atom: AtomId) -> Option<Self> {
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
        let hydrogens = owned_property(source, node, HYDROGENS)?;
        let text = owned_property(source, label, TEXT)?;
        let bounds = owned_property(source, label, BOUNDING_BOX)?;
        let string = source.string(text).ok()?;
        let mut runs = string.runs();
        if source.payload(element).ok()?.len() != 2
            || source.payload(hydrogens).ok()?.len() != 2
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
    ) -> Result<Vec<u8>, MappingError> {
        let mut text = self.run_prefix.to_vec();
        text.extend_from_slice(body.as_bytes());
        Ok(source.replace_properties(&[
            (self.element, &i16::from(element.get()).to_le_bytes()),
            (self.hydrogens, &hydrogens.to_le_bytes()),
            (self.text, &text),
            (self.bounds, bounds),
        ])?)
    }
}

fn owned_property<'record>(
    source: &'record Document<'_>,
    parent: usize,
    tag: u16,
) -> Option<&'record Record> {
    let mut records = source.records().iter().filter(|record| {
        record.parent() == Some(parent)
            && matches!(record.kind(), RecordKind::Property { tag: field, .. } if *field == tag)
    });
    let record = records.next()?;
    records.next().is_none().then_some(record)
}
