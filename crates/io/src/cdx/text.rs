//! Resolve the bounded, explicit UTF-8 subset of CDX styled strings.
use bondline_cdx::{Document, Record, Resources};
use bondline_model::{MAX_TEXT_BYTES, MAX_TEXT_RUNS, TextRun, TextStyle};

pub(super) enum TextRole {
    Caption,
    AtomLabel,
}

pub(super) fn decode_runs(
    source: &Document<'_>,
    resources: &Resources<'_, '_>,
    property: usize,
    role: TextRole,
    remaining_runs: &mut usize,
) -> Option<Vec<TextRun>> {
    if resources.font_platform() != Some(1) {
        return None;
    }
    let value = source.string(source.records().get(property)?).ok()?;
    let text = value.text_bytes();
    let caption = matches!(role, TextRole::Caption);
    if text.len() > MAX_TEXT_BYTES || (caption && text.is_empty()) {
        return None;
    }
    let body = std::str::from_utf8(text).ok()?;
    let mut source_runs = value.runs().peekable();
    let count = source_runs.len();
    if count > MAX_TEXT_RUNS || (body.is_empty() && count != 1) {
        return None;
    }
    *remaining_runs = remaining_runs.checked_sub(count)?;
    if source_runs.peek()?.start() != 0 {
        return None;
    }
    let mut runs = Vec::with_capacity(count);
    while let Some(run) = source_runs.next() {
        let next = source_runs.peek().map(|next| usize::from(next.start()));
        let part = body.get(usize::from(run.start())..next.unwrap_or(body.len()))?;
        if (part.is_empty() && !body.is_empty()) || (next.is_some() && !part.is_ascii()) {
            return None;
        }
        let font = resources.font_by_id(run.font()).ok()?;
        let name = font.name_bytes();
        if font.ordinal() != usize::from(run.font())
            || font.charset() != 65001
            || (caption && (run.face() != 0 || name != b"Roboto"))
            || !name.iter().all(|byte| (b' '..=b'~').contains(byte))
        {
            return None;
        }
        let rgb = resources
            .color(u32::from(run.color()))
            .ok()?
            .map(|channel| f64::from(channel) / f64::from(u16::MAX));
        let style = TextStyle::new(
            std::str::from_utf8(name).ok()?.to_owned(),
            f64::from(run.size_twentieths()) / 20.0,
            run.face(),
            rgb,
        )
        .ok()?;
        runs.push(TextRun::new(part.to_owned(), style).ok()?);
    }
    Some(runs)
}

pub(crate) fn encode_caption_runs(
    source: &Document<'_>,
    record: &Record,
    runs: &[TextRun],
) -> Option<Vec<u8>> {
    let original = source.string(record).ok()?;
    let styles = original.runs();
    if runs.is_empty() || styles.len() != runs.len() {
        return None;
    }
    let mut start = 0_usize;
    let payload = source.payload(record).ok()?;
    let prefix = payload.strip_suffix(original.text_bytes())?;
    let body_len = runs
        .iter()
        .try_fold(0_usize, |size, run| size.checked_add(run.text().len()))?;
    if body_len > MAX_TEXT_BYTES {
        return None;
    }
    let mut encoded = Vec::new();
    encoded
        .try_reserve_exact(prefix.len().checked_add(body_len)?)
        .ok()?;
    encoded.extend_from_slice(prefix.get(..2)?);
    for (index, (run, style)) in runs.iter().zip(styles).enumerate() {
        let text = run.text();
        if text.is_empty() || (index + 1 < runs.len() && !text.is_ascii()) {
            return None;
        }
        encoded.extend_from_slice(&u16::try_from(start).ok()?.to_le_bytes());
        let range = style.range();
        encoded.extend_from_slice(
            source
                .as_bytes()
                .get(range.start.checked_add(2)?..range.end)?,
        );
        start = start.checked_add(text.len())?;
    }
    for run in runs {
        encoded.extend_from_slice(run.text().as_bytes());
    }
    Some(encoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caption_wire_start_limit_and_raw_style_words_are_exact()
    -> Result<(), Box<dyn std::error::Error>> {
        let payload = [
            2, 0, 0, 0, 1, 0, 0, 0, 240, 0, 3, 0, 1, 0, 1, 0, 0, 0, 128, 1, 4, 0, b'A', 0xce, 0xa9,
        ];
        let mut input = b"VjCD0100\x04\x03\x02\x01".to_vec();
        input.resize(28, 0);
        input.extend_from_slice(&[0, 7, 25, 0]);
        input.extend_from_slice(&payload);
        input.extend_from_slice(&[0, 0]);
        let source = Document::parse(&input)?;
        let record = source.records().first().ok_or("Text property")?;
        let style = TextStyle::new("Roboto".into(), 12.0, 0, [0.0; 3])?;
        let last = TextRun::new("Ω".into(), style.clone())?;
        let prefix = " ".repeat(usize::from(u16::MAX));
        let runs = [TextRun::new(prefix.clone(), style.clone())?, last.clone()];
        let mut expected = payload.get(..22).ok_or("test style prefix")?.to_vec();
        expected
            .get_mut(12..14)
            .ok_or("second start")?
            .copy_from_slice(&[0xff, 0xff]);
        expected.extend_from_slice(prefix.as_bytes());
        expected.extend_from_slice("Ω".as_bytes());
        assert_eq!(encode_caption_runs(&source, record, &runs), Some(expected));

        for invalid_prefix in [prefix + " ", "β".into(), String::new()] {
            let runs = [TextRun::new(invalid_prefix, style.clone())?, last.clone()];
            assert!(encode_caption_runs(&source, record, &runs).is_none());
        }
        Ok(())
    }
}
