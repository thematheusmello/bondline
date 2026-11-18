//! Resolve the bounded, explicit UTF-8 subset of CDX styled strings.
use bondline_cdx::{Document, Resources};
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
