//! Literal Node child Text fields. Placement and chemical agreement are unproved.
use super::{
    BOUNDING_BOX, MappingError, POSITION, TEXT, Z_ORDER, coordinate, payload, properties,
    read_position,
    text::{TextRole, decode_runs},
    validate_metadata,
};
use bondline_cdx::{Document, Resources};
use bondline_model::{AtomLabel, Error, LabelAlignment, TextJustification};
use std::collections::BTreeMap;

pub(super) const JUSTIFICATION: u16 = 0x0701;
const LABEL_ALIGNMENT: u16 = 0x0705;
pub(super) const LABEL_JUSTIFICATION: u16 = 0x0823;

pub(super) fn read(
    source: &Document<'_>,
    resources: &Resources<'_, '_>,
    object: usize,
    children: &[usize],
    remaining_runs: &mut usize,
) -> Result<AtomLabel, MappingError> {
    let fields = properties(
        source,
        children,
        &[
            TEXT,
            POSITION,
            BOUNDING_BOX,
            JUSTIFICATION,
            LABEL_JUSTIFICATION,
            LABEL_ALIGNMENT,
            Z_ORDER,
        ],
    )?;
    validate_metadata(source, &fields)?;
    let position = fields
        .get(&POSITION)
        .map(|&record| read_position(source, record))
        .transpose()?;
    let justification = optional_code(source, &fields, JUSTIFICATION, parse_justification)?;
    let label_justification =
        optional_code(source, &fields, LABEL_JUSTIFICATION, parse_justification)?;
    let label_alignment = optional_code(source, &fields, LABEL_ALIGNMENT, parse_alignment)?;
    let record = fields
        .get(&TEXT)
        .copied()
        .ok_or(MappingError::MissingProperty {
            tag: TEXT,
            record: object,
        })?;
    let runs = decode_runs(
        source,
        resources,
        record,
        TextRole::AtomLabel,
        remaining_runs,
    )
    .ok_or(MappingError::InvalidProperty { tag: TEXT, record })?;
    let label = AtomLabel::new(
        position,
        justification,
        label_justification,
        label_alignment,
        runs,
    )
    .map_err(Error::from)?;
    let Some(&record) = fields.get(&BOUNDING_BOX) else {
        return Ok(label);
    };
    let [
        t0,
        t1,
        t2,
        t3,
        l0,
        l1,
        l2,
        l3,
        b0,
        b1,
        b2,
        b3,
        r0,
        r1,
        r2,
        r3,
    ] = payload(source, record, BOUNDING_BOX)?;
    Ok(label
        .with_bounding_box(Some([
            coordinate([l0, l1, l2, l3]),
            coordinate([t0, t1, t2, t3]),
            coordinate([r0, r1, r2, r3]),
            coordinate([b0, b1, b2, b3]),
        ]))
        .map_err(Error::from)?)
}

fn optional_code<T>(
    source: &Document<'_>,
    fields: &BTreeMap<u16, usize>,
    tag: u16,
    parse: impl Fn(i8) -> Option<T>,
) -> Result<Option<T>, MappingError> {
    fields
        .get(&tag)
        .map(|&record| {
            let signed = i8::from_le_bytes(payload(source, record, tag)?);
            parse(signed).ok_or(MappingError::InvalidProperty { tag, record })
        })
        .transpose()
}

fn parse_justification(code: i8) -> Option<TextJustification> {
    Some(match code {
        -1 => TextJustification::Right,
        0 => TextJustification::Left,
        1 => TextJustification::Center,
        2 => TextJustification::Full,
        3 => TextJustification::Above,
        4 => TextJustification::Below,
        5 => TextJustification::Auto,
        6 => TextJustification::Best,
        _ => return None,
    })
}

fn parse_alignment(code: i8) -> Option<LabelAlignment> {
    Some(match code {
        0 => LabelAlignment::Auto,
        1 => LabelAlignment::Left,
        2 => LabelAlignment::Center,
        3 => LabelAlignment::Right,
        4 => LabelAlignment::Above,
        5 => LabelAlignment::Below,
        6 => LabelAlignment::Best,
        _ => return None,
    })
}
