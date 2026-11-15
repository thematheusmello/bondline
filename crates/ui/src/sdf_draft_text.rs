//! Byte-bounded temporary SDF field text; the command engine validates Apply.

use bondline_commands::MAX_SDF_INSPECT_VALUE_BYTES;
use eframe::egui::{TextBuffer, text::CCursor};
use std::{fmt, ops::Range};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DraftTextError {
    ByteLimit {
        attempted_bytes: usize,
        limit_bytes: usize,
    },
    CharacterLimit {
        attempted_characters: usize,
        limit_characters: usize,
    },
    InvalidCharIndex {
        index: usize,
        character_count: usize,
    },
    InvalidCharRange {
        start: usize,
        end: usize,
        character_count: usize,
    },
}

impl fmt::Display for DraftTextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ByteLimit {
                attempted_bytes,
                limit_bytes,
            } => write!(
                f,
                "Draft text would use {attempted_bytes} UTF-8 bytes; the limit is {limit_bytes} bytes."
            ),
            Self::CharacterLimit {
                attempted_characters,
                limit_characters,
            } => write!(
                f,
                "Draft text would use {attempted_characters} characters; the widget limit is {limit_characters}."
            ),
            Self::InvalidCharIndex {
                index,
                character_count,
            } => write!(
                f,
                "Draft character index {index} is outside 0..={character_count}."
            ),
            Self::InvalidCharRange {
                start,
                end,
                character_count,
            } => write!(
                f,
                "Draft character range {start}..{end} is invalid for {character_count} characters."
            ),
        }
    }
}

impl std::error::Error for DraftTextError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DraftEditOutcome {
    pub(crate) changed: bool,
    pub(crate) rejection: Option<DraftTextError>,
}

/// Literal temporary text, limited by UTF-8 bytes, without SDF grammar inference.
///
/// Wrap each complete `TextEdit::show` in [`Self::edit`]. A rejected mutation then
/// restores the pre-widget text immediately, before the widget can relayout it.
/// The caller must also restore its prior cursor and reset its bounded undoer.
pub(crate) struct SdfDraftText {
    text: String,
    checkpoint: Option<String>,
    in_edit: bool,
    rejection: Option<DraftTextError>,
}

impl fmt::Debug for SdfDraftText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SdfDraftText")
            .field("byte_length", &self.text.len())
            .field("rejection", &self.rejection)
            .finish_non_exhaustive()
    }
}

impl SdfDraftText {
    pub(crate) fn new(text: &str) -> Result<Self, DraftTextError> {
        Self::check_bytes(text.len())?;
        Ok(Self {
            text: text.to_owned(),
            checkpoint: None,
            in_edit: false,
            rejection: None,
        })
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.text
    }

    pub(crate) fn last_rejection(&self) -> Option<DraftTextError> {
        self.rejection
    }

    /// One widget edit transaction, with one bounded temporary original snapshot.
    ///
    /// Any rejected operation rolls back all text changes in this call and freezes
    /// later mutations until it returns. The closure receives only `TextBuffer`,
    /// so it cannot open a nested draft transaction. A no-op reports unchanged.
    pub(crate) fn edit<R>(
        &mut self,
        edit: impl FnOnce(&mut dyn TextBuffer) -> R,
    ) -> (R, DraftEditOutcome) {
        self.checkpoint = Some(self.text.clone());
        self.in_edit = true;
        self.rejection = None;
        let result = edit(self);
        let changed = self
            .checkpoint
            .as_ref()
            .is_some_and(|before| before != &self.text);
        self.checkpoint = None;
        self.in_edit = false;
        (
            result,
            DraftEditOutcome {
                changed,
                rejection: self.rejection,
            },
        )
    }

    fn check_bytes(attempted_bytes: usize) -> Result<(), DraftTextError> {
        if attempted_bytes > MAX_SDF_INSPECT_VALUE_BYTES {
            Err(DraftTextError::ByteLimit {
                attempted_bytes,
                limit_bytes: MAX_SDF_INSPECT_VALUE_BYTES,
            })
        } else {
            Ok(())
        }
    }

    fn ready(&mut self) -> bool {
        if self.in_edit && self.rejection.is_some() {
            false
        } else {
            self.rejection = None;
            true
        }
    }

    fn reject(&mut self, error: DraftTextError) {
        self.rejection = Some(error);
        if let Some(checkpoint) = self.checkpoint.take() {
            self.text = checkpoint;
        }
    }

    fn checked_cursor(&mut self, cursor: &mut CCursor) -> bool {
        let count = self.text.chars().count();
        let index = cursor.index;
        cursor.index = index.min(count);
        if !self.ready() {
            return false;
        }
        if index > count {
            self.reject(DraftTextError::InvalidCharIndex {
                index,
                character_count: count,
            });
            false
        } else {
            true
        }
    }

    fn checked_range(&self, range: Range<usize>) -> Option<Range<usize>> {
        if range.start > range.end || range.end > self.text.chars().count() {
            return None;
        }
        Some(
            self.text.byte_index_from_char_index(range.start)
                ..self.text.byte_index_from_char_index(range.end),
        )
    }
}

impl TextBuffer for SdfDraftText {
    fn is_mutable(&self) -> bool {
        true
    }

    fn as_str(&self) -> &str {
        self.as_str()
    }

    fn insert_text(&mut self, text: &str, char_index: usize) -> usize {
        let mut cursor = CCursor::new(char_index);
        if !self.checked_cursor(&mut cursor) {
            return 0;
        }
        if let Err(error) = Self::check_bytes(self.text.len().saturating_add(text.len())) {
            self.reject(error);
            return 0;
        }
        let byte_index = self.text.byte_index_from_char_index(char_index);
        self.text.insert_str(byte_index, text);
        text.chars().count()
    }

    fn delete_char_range(&mut self, range: Range<usize>) {
        if !self.ready() {
            return;
        }
        let Some(bytes) = self.checked_range(range.clone()) else {
            self.reject(DraftTextError::InvalidCharRange {
                start: range.start,
                end: range.end,
                character_count: self.text.chars().count(),
            });
            return;
        };
        self.text.replace_range(bytes, "");
    }

    /// Invalid read ranges yield empty text rather than slicing outside UTF-8.
    fn char_range(&self, range: Range<usize>) -> &str {
        self.checked_range(range)
            .and_then(|bytes| self.text.get(bytes))
            .map_or("", |text| text)
    }

    /// Read-only out-of-range indices clamp to the end, matching the SDK helper.
    fn byte_index_from_char_index(&self, char_index: usize) -> usize {
        self.text.byte_index_from_char_index(char_index)
    }

    fn clear(&mut self) {
        if self.ready() {
            self.text.clear();
        }
    }

    fn replace_with(&mut self, text: &str) {
        if !self.ready() {
            return;
        }
        if let Err(error) = Self::check_bytes(text.len()) {
            self.reject(error);
            return;
        }
        self.text.clear();
        self.text.push_str(text);
    }

    fn take(&mut self) -> String {
        if self.ready() {
            std::mem::take(&mut self.text)
        } else {
            String::new()
        }
    }

    fn insert_text_at(&mut self, cursor: &mut CCursor, text: &str, char_limit: usize) {
        if !self.checked_cursor(cursor) {
            return;
        }
        let attempted_characters = self
            .text
            .chars()
            .count()
            .saturating_add(text.chars().count());
        if attempted_characters > char_limit {
            self.reject(DraftTextError::CharacterLimit {
                attempted_characters,
                limit_characters: char_limit,
            });
            return;
        }
        let inserted = self.insert_text(text, cursor.index);
        cursor.index = cursor
            .index
            .saturating_add(inserted)
            .min(self.text.chars().count());
    }

    // The SDK's word/indentation helpers assume a valid character cursor and some
    // use unchecked subtraction. Validate first, then reuse its editing semantics.
    fn decrease_indentation(&mut self, cursor: &mut CCursor) {
        if self.checked_cursor(cursor) {
            self.text.decrease_indentation(cursor);
        }
    }

    fn delete_previous_char(&mut self, mut cursor: CCursor) -> CCursor {
        if self.checked_cursor(&mut cursor) {
            self.text.delete_previous_char(cursor)
        } else {
            cursor
        }
    }

    fn delete_next_char(&mut self, mut cursor: CCursor) -> CCursor {
        if self.checked_cursor(&mut cursor) {
            self.text.delete_next_char(cursor)
        } else {
            cursor
        }
    }

    fn delete_previous_word(&mut self, mut cursor: CCursor) -> CCursor {
        if self.checked_cursor(&mut cursor) {
            self.text.delete_previous_word(cursor)
        } else {
            cursor
        }
    }

    fn delete_next_word(&mut self, mut cursor: CCursor) -> CCursor {
        if self.checked_cursor(&mut cursor) {
            self.text.delete_next_word(cursor)
        } else {
            cursor
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{self, TextBuffer, text::CCursor};
    use std::error::Error;

    const LIMIT: usize = 65_536;

    #[test]
    fn constructor_preserves_literal_utf8_and_checks_bytes_before_copying()
    -> Result<(), Box<dyn Error>> {
        assert_eq!(bondline_commands::MAX_SDF_INSPECT_VALUE_BYTES, LIMIT);
        let literal = " \tα\n水🧪\r\0 ";
        let draft = SdfDraftText::new(literal)?;
        assert_eq!(draft.as_str(), literal);
        assert_eq!(draft.last_rejection(), None);
        let exact = "é".repeat(32_768);
        assert_eq!(exact.len(), LIMIT);
        assert_eq!(SdfDraftText::new(&exact)?.as_str(), exact);
        assert!(matches!(
            SdfDraftText::new(&"x".repeat(65_537)),
            Err(DraftTextError::ByteLimit { .. })
        ));
        assert!(matches!(
            SdfDraftText::new(&"🧪".repeat(16_385)),
            Err(DraftTextError::ByteLimit { .. })
        ));
        Ok(())
    }

    #[test]
    fn trait_edits_use_char_indices_and_return_char_counts() -> Result<(), Box<dyn Error>> {
        let mut draft = SdfDraftText::new("aé水🧪z")?;
        assert!(draft.is_mutable());
        assert_eq!(draft.insert_text(" Ω", 2), 2);
        assert_eq!(draft.as_str(), "aé Ω水🧪z");
        draft.delete_char_range(1..4);
        assert_eq!(draft.as_str(), "a水🧪z");
        assert_eq!(draft.char_range(1..3), "水🧪");
        assert_eq!(draft.byte_index_from_char_index(3), 8);
        let mut cursor = CCursor::new(3);
        draft.insert_text_at(&mut cursor, "é水", usize::MAX);
        assert_eq!(cursor.index, 5);
        assert_eq!(draft.as_str(), "a水🧪é水z");
        assert_eq!(draft.take(), "a水🧪é水z");
        assert_eq!(draft.as_str(), "");
        draft.replace_with("é水🧪");
        draft.clear();
        assert_eq!(draft.as_str(), "");
        Ok(())
    }

    #[test]
    fn exact_fit_insert_and_atomic_replacement_never_truncate() -> Result<(), Box<dyn Error>> {
        let mut draft = SdfDraftText::new(&"a".repeat(65_534))?;
        assert_eq!(draft.insert_text("é", 1), 1);
        let expected = format!("aé{}", "a".repeat(65_533));
        assert_eq!(draft.as_str(), expected);
        assert_eq!(draft.as_str().len(), LIMIT);
        assert_eq!(draft.insert_text("x", 0), 0);
        assert_eq!(draft.as_str(), expected);
        assert!(matches!(
            draft.last_rejection(),
            Some(DraftTextError::ByteLimit { .. })
        ));
        draft.replace_with(&"b".repeat(65_537));
        assert_eq!(draft.as_str(), expected, "replace must not clear first");
        draft.replace_with(&"水".repeat(21_846));
        assert_eq!(draft.as_str(), expected, "UTF-8 bytes are the bound");
        draft.replace_with(" é \n水 ");
        assert_eq!(draft.as_str(), " é \n水 ");
        assert_eq!(draft.last_rejection(), None);
        let mut cursor = CCursor::new(1);
        draft.insert_text_at(&mut cursor, "ab", 7);
        assert_eq!(
            draft.as_str(),
            " é \n水 ",
            "finite char limit is whole-reject"
        );
        assert_eq!(cursor.index, 1);
        assert!(matches!(
            draft.last_rejection(),
            Some(DraftTextError::CharacterLimit { .. })
        ));
        Ok(())
    }

    #[test]
    fn hostile_indices_ranges_and_sdk_cursor_helpers_do_not_panic() -> Result<(), Box<dyn Error>> {
        let mut draft = SdfDraftText::new("é\n水🧪")?;
        for index in [5, usize::MAX] {
            assert_eq!(draft.insert_text("x", index), 0);
            assert_eq!(draft.as_str(), "é\n水🧪");
            let mut cursor = CCursor::new(index);
            draft.insert_text_at(&mut cursor, "x", usize::MAX);
            assert_eq!(draft.as_str(), "é\n水🧪");
            assert!(cursor.index <= 4);
            draft.delete_previous_char(CCursor::new(index));
            draft.delete_next_char(CCursor::new(index));
            draft.delete_previous_word(CCursor::new(index));
            draft.delete_next_word(CCursor::new(index));
            let mut indent_cursor = CCursor::new(index);
            draft.decrease_indentation(&mut indent_cursor);
            assert_eq!(draft.as_str(), "é\n水🧪");
        }
        for range in [
            Range { start: 3, end: 1 },
            0..5,
            usize::MAX..usize::MAX,
            1..usize::MAX,
        ] {
            draft.delete_char_range(range.clone());
            assert_eq!(draft.as_str(), "é\n水🧪");
            assert_eq!(draft.char_range(range), "");
        }
        assert_eq!(draft.byte_index_from_char_index(usize::MAX), 10);
        assert_eq!(draft.byte_index_from_char_index(4), 10);
        draft.delete_char_range(4..4);
        assert_eq!(draft.as_str(), "é\n水🧪");
        Ok(())
    }

    #[test]
    fn rejected_widget_transaction_restores_immediately_and_freezes_later_mutations()
    -> Result<(), Box<dyn Error>> {
        let mut draft = SdfDraftText::new("keep βeta\n水")?;
        let ((), outcome) = draft.edit(|text| {
            text.delete_char_range(5..9);
            assert_eq!(text.as_str(), "keep \n水");
            assert_eq!(text.insert_text(&"x".repeat(65_537), 5), 0);
            assert_eq!(
                text.as_str(),
                "keep βeta\n水",
                "restore before widget relayout"
            );
            text.clear();
            text.replace_with("lost");
            text.delete_char_range(0..1);
            assert_eq!(text.insert_text("later", 0), 0);
            assert_eq!(text.take(), "");
            assert_eq!(text.as_str(), "keep βeta\n水");
        });
        assert!(!outcome.changed);
        assert!(matches!(
            outcome.rejection,
            Some(DraftTextError::ByteLimit { .. })
        ));
        assert_eq!(draft.as_str(), "keep βeta\n水");
        let (value, outcome) = draft.edit(|text| {
            text.delete_char_range(5..9);
            text.insert_text("X", 5)
        });
        assert_eq!(value, 1);
        assert!(outcome.changed);
        assert_eq!(outcome.rejection, None);
        assert_eq!(draft.as_str(), "keep X\n水");
        let (_, outcome) = draft.edit(|text| text.replace_with("keep X\n水"));
        assert!(!outcome.changed);
        Ok(())
    }

    fn bounded_undoer() -> egui::util::undoer::Undoer<(egui::text::CCursorRange, String)> {
        egui::util::undoer::Undoer::with_settings(egui::util::undoer::Settings {
            max_undos: 8,
            ..Default::default()
        })
    }

    fn widget_frame(
        draft: &mut SdfDraftText,
        context: &egui::Context,
        id: egui::Id,
        events: Vec<egui::Event>,
        time: f64,
    ) -> Result<(egui::text_edit::TextEditOutput, DraftEditOutcome), Box<dyn Error>> {
        let mut shown = None;
        let _output = context.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(640.0, 480.0),
                )),
                events,
                time: Some(time),
                focused: true,
                ..Default::default()
            },
            |context| {
                egui::CentralPanel::default().show(context, |ui| {
                    shown = Some(draft.edit(|text| {
                        egui::TextEdit::multiline(text)
                            .id(id)
                            .desired_width(500.0)
                            .show(ui)
                    }));
                });
            },
        );
        shown.ok_or_else(|| "real TextEdit did not run".into())
    }

    #[test]
    fn actual_text_edit_rejects_selected_oversized_paste_without_same_frame_data_loss()
    -> Result<(), Box<dyn Error>> {
        let context = egui::Context::default();
        crate::configure_context(&context);
        let id = egui::Id::new("sdf-draft-bounded-paste");
        let original = "alpha βeta\n水 tail";
        let mut draft = SdfDraftText::new(original)?;
        let (mut output, _) = widget_frame(&mut draft, &context, id, vec![], 0.0)?;
        let selection = egui::text::CCursorRange::two(CCursor::new(6), CCursor::new(10));
        output.state.cursor.set_char_range(Some(selection));
        output.state.set_undoer(bounded_undoer());
        output.state.store(&context, id);
        context.memory_mut(|memory| memory.request_focus(id));
        let (mut output, outcome) = widget_frame(
            &mut draft,
            &context,
            id,
            vec![
                egui::Event::Paste("é".repeat(32_769)),
                egui::Event::Text("later".to_owned()),
            ],
            1.0,
        )?;
        assert_eq!(draft.as_str(), original);
        assert_eq!(
            output.galley.text(),
            original,
            "actual same-frame widget layout"
        );
        assert!(!outcome.changed);
        assert!(matches!(
            outcome.rejection,
            Some(DraftTextError::ByteLimit { .. })
        ));
        // The host must restore the cursor and discard the SDK's rejected edit undo state.
        output.state.cursor.set_char_range(Some(selection));
        output.state.set_undoer(bounded_undoer());
        output.state.store(&context, id);
        let (output, outcome) = widget_frame(
            &mut draft,
            &context,
            id,
            vec![egui::Event::Paste("X".to_owned())],
            2.0,
        )?;
        assert_eq!(draft.as_str(), "alpha X\n水 tail");
        assert_eq!(output.galley.text(), "alpha X\n水 tail");
        assert!(outcome.changed);
        assert_eq!(outcome.rejection, None);
        Ok(())
    }

    #[test]
    fn public_undo_configuration_is_bounded_and_can_be_cleared_at_draft_end() {
        let cursor = egui::text::CCursorRange::one(CCursor::new(0));
        let mut undoer = bounded_undoer();
        for number in 0..20 {
            undoer.feed_state(f64::from(number * 2), &(cursor, number.to_string()));
        }
        let mut current = (cursor, "19".to_owned());
        let mut count = 0;
        while let Some(previous) = undoer.undo(&current).cloned() {
            current = previous;
            count += 1;
        }
        assert!((1..=8).contains(&count));
        let mut state = egui::text_edit::TextEditState::default();
        state.set_undoer(undoer);
        state.set_undoer(bounded_undoer());
        assert!(!state.undoer().has_undo(&current));
        assert!(!state.undoer().has_redo(&current));
        assert!(!state.undoer().is_in_flux());
    }

    #[test]
    fn debug_does_not_disclose_field_text() -> Result<(), Box<dyn Error>> {
        let draft = SdfDraftText::new("private-field-value")?;
        assert!(!format!("{draft:?}").contains("private-field-value"));
        Ok(())
    }
}
