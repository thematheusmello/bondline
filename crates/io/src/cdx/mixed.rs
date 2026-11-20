//! Project direct Page captions and chemical fragments against one original record table.
use super::{ElementalGraph, FRAGMENT, Layout, PAGE, TEXT_OBJECT, caption, label_preview, object};
use bondline_cdx::{Document, Record, RecordKind};
use bondline_render::MAX_PAGE_TEXT_RUNS;
use std::collections::BTreeSet;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Scope {
    All,
    Captions,
    Fragments,
}

impl Scope {
    pub(super) fn records<'a>(
        self,
        source: &'a Document<'_>,
    ) -> impl Iterator<Item = (usize, &'a Record)> {
        let root = source.framing().has_encoded_document().then_some(0);
        let mut skip = None;
        source
            .records()
            .iter()
            .enumerate()
            .filter(move |&(index, record)| {
                if self == Self::All {
                    return true;
                }
                // End records point to the enclosing parent; match the skipped opening itself.
                if let Some(start) = skip {
                    if *record.kind() == (RecordKind::ObjectEnd { start: Some(start) }) {
                        skip = None;
                    }
                    return false;
                }
                match *record.kind() {
                    RecordKind::ObjectStart { tag, .. }
                        if matches!(
                            (self, tag),
                            (Self::Captions, FRAGMENT) | (Self::Fragments, TEXT_OBJECT)
                        ) && object(source, record.parent())
                            .is_some_and(|(owner, _)| owner == PAGE) =>
                    {
                        skip = Some(index);
                        false
                    }
                    RecordKind::Property { tag, .. }
                        if self == Self::Captions && record.parent() == root =>
                    {
                        !label_preview::is_drawing_property(tag)
                    }
                    _ => true,
                }
            })
    }
}

pub(super) fn preview(source: &Document<'_>) -> Option<bondline_model::Document> {
    preview_with_run_budget(source, MAX_PAGE_TEXT_RUNS)
}

fn preview_with_run_budget(
    source: &Document<'_>,
    mut remaining_runs: usize,
) -> Option<bondline_model::Document> {
    let root = source.framing().has_encoded_document().then_some(0);
    let page = source.records().iter().position(|record| {
        record.parent() == root
            && matches!(record.kind(), RecordKind::ObjectStart { tag: PAGE, .. })
    })?;
    let children = || {
        source
            .records()
            .iter()
            .filter_map(|record| match *record.kind() {
                RecordKind::ObjectStart { tag, id } if record.parent() == Some(page) => {
                    Some((tag, id))
                }
                _ => None,
            })
    };
    if !children().any(|(tag, _)| tag == TEXT_OBJECT) || !children().any(|(tag, _)| tag == FRAGMENT)
    {
        return None;
    }
    let mut named = BTreeSet::new();
    if !children()
        .all(|(tag, id)| matches!(tag, FRAGMENT | TEXT_OBJECT) && (id == 0 || named.insert(id)))
    {
        return None;
    }
    let resources = source.resources().ok()?;
    let texts = caption::mixed_texts(source, &resources, &mut remaining_runs)?;
    let layout = Layout::read(source, true, Scope::Fragments).ok()?;
    let fragments =
        ElementalGraph::decode_fragments(source, &layout, Some(&resources), &mut remaining_runs)
            .ok()?;
    let mut document =
        label_preview::model(source, &fragments, Scope::Fragments, Some(&resources))?;
    let page_id = document.pages().first()?.id();
    document.append_texts(page_id, texts).ok()?;
    super::verify_preview_page(&document)?;
    Some(document)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_caption_and_both_node_labels_share_one_run_budget() -> Result<(), bondline_cdx::Error> {
        // The independent fixture has one Page caption run and two Node label runs.
        let source = Document::parse(include_bytes!(
            "../../tests/fixtures/cdx-mixed-caption/boxed_before.cdx"
        ))?;
        assert!(preview_with_run_budget(&source, 3).is_some());
        assert!(preview_with_run_budget(&source, 2).is_none());
        Ok(())
    }
}
