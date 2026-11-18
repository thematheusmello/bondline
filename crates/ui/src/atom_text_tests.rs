//! Independent private-module byte-limit tests; grammar remains outside the text buffer.
use super::{DraftTextError, SdfDraftText};
use eframe::egui::TextBuffer;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn instance_limits_count_utf8_bytes_and_keep_the_original_sdf_constructor_limit() -> TestResult {
    let mut element = SdfDraftText::with_byte_limit("é", 2)?;
    assert_eq!(element.as_str(), "é");
    assert_eq!(element.insert_text("x", 1), 0);
    assert_eq!(element.as_str(), "é");
    assert_eq!(
        element.last_rejection(),
        Some(DraftTextError::ByteLimit {
            attempted_bytes: 3,
            limit_bytes: 2
        })
    );
    element.replace_with("水");
    assert_eq!(element.as_str(), "é");
    assert!(matches!(
        SdfDraftText::with_byte_limit("水", 2),
        Err(DraftTextError::ByteLimit {
            attempted_bytes: 3,
            limit_bytes: 2
        })
    ));
    element.replace_with("cl");
    assert_eq!(
        element.as_str(),
        "cl",
        "short invalid element draft is still literal/copyable"
    );
    let mut hydrogen = SdfDraftText::with_byte_limit("99999", 5)?;
    assert_eq!(
        hydrogen.as_str(),
        "99999",
        "u16 grammar belongs to Apply, not this buffer"
    );
    hydrogen.replace_with("12a  ");
    assert_eq!(hydrogen.as_str(), "12a  ");
    assert!(matches!(
        SdfDraftText::with_byte_limit("123456", 5),
        Err(DraftTextError::ByteLimit {
            attempted_bytes: 6,
            limit_bytes: 5
        })
    ));
    assert_eq!(SdfDraftText::new(&"x".repeat(65536))?.as_str().len(), 65536);
    assert!(matches!(
        SdfDraftText::new(&"x".repeat(65537)),
        Err(DraftTextError::ByteLimit {
            attempted_bytes: 65537,
            limit_bytes: 65536
        })
    ));
    Ok(())
}

use super::DraftEditOutcome;
use eframe::egui::{
    self,
    text::{CCursor, CCursorRange},
};

fn undoer() -> egui::util::undoer::Undoer<(CCursorRange, String)> {
    egui::util::undoer::Undoer::with_settings(egui::util::undoer::Settings {
        max_undos: 8,
        ..Default::default()
    })
}

fn frame(
    draft: &mut SdfDraftText,
    context: &egui::Context,
    id: egui::Id,
    events: Vec<egui::Event>,
    time: f64,
) -> TestResult<(egui::text_edit::TextEditOutput, DraftEditOutcome)> {
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
                let requested = ui.input(|input| {
                    input
                        .accesskit_action_requests(id, egui::accesskit::Action::SetValue)
                        .last()
                        .and_then(|request| match &request.data {
                            Some(egui::accesskit::ActionData::Value(value)) => {
                                Some(value.to_string())
                            }
                            _ => None,
                        })
                });
                shown = Some(draft.edit(|text| {
                    // Pinned TextEdit lacks SetValue: this is the existing app bridge policy,
                    // with that event first in the fixture, inside the same widget transaction.
                    if let Some(value) = requested {
                        text.replace_with(&value);
                    }
                    egui::TextEdit::singleline(text)
                        .id(id)
                        .char_limit(usize::MAX)
                        .desired_width(300.0)
                        .show(ui)
                }));
            });
        },
    );
    shown.ok_or_else(|| "actual TextEdit did not run".into())
}

fn focus_selection(
    context: &egui::Context,
    id: egui::Id,
    output: &mut egui::text_edit::TextEditOutput,
    selection: CCursorRange,
) {
    output.state.cursor.set_char_range(Some(selection));
    output.state.set_undoer(undoer());
    output.state.clone().store(context, id);
    context.memory_mut(|memory| memory.request_focus(id));
}

#[test]
fn actual_selected_paste_rejects_all_bytes_and_later_packet_text_at_a_two_byte_limit() -> TestResult
{
    let context = egui::Context::default();
    crate::configure_context(&context);
    let id = egui::Id::new("instance-two-byte-paste");
    let mut draft = SdfDraftText::with_byte_limit("Cl", 2)?;
    let (mut output, _) = frame(&mut draft, &context, id, vec![], 0.0)?;
    let selection = CCursorRange::two(CCursor::new(1), CCursor::new(2));
    focus_selection(&context, id, &mut output, selection);
    let (mut output, outcome) = frame(
        &mut draft,
        &context,
        id,
        vec![
            egui::Event::Paste("水".to_owned()),
            egui::Event::Text("F".to_owned()),
        ],
        1.0,
    )?;
    assert_eq!(
        draft.as_str(),
        "Cl",
        "selection deletion must roll back too"
    );
    assert_eq!(output.galley.text(), "Cl", "real same-packet layout");
    assert!(!outcome.changed);
    assert_eq!(
        outcome.rejection,
        Some(DraftTextError::ByteLimit {
            attempted_bytes: 4,
            limit_bytes: 2
        })
    );
    // The caller, not the text buffer, owns cursor restoration and bounded undo reset.
    focus_selection(&context, id, &mut output, selection);
    let (output, outcome) = frame(
        &mut draft,
        &context,
        id,
        vec![egui::Event::Paste(" ".to_owned())],
        2.0,
    )?;
    assert_eq!(
        draft.as_str(),
        "C ",
        "a short invalid symbol remains an exact draft"
    );
    assert_eq!(output.galley.text(), "C ");
    assert!(outcome.changed);
    assert_eq!(outcome.rejection, None);
    let mut hostile = CCursor::new(usize::MAX);
    draft.insert_text_at(&mut hostile, "N", usize::MAX);
    assert_eq!(draft.as_str(), "C ");
    assert!(hostile.index <= 2);
    Ok(())
}

#[test]
fn accesskit_replacement_and_real_widget_later_text_share_one_five_byte_transaction() -> TestResult
{
    let context = egui::Context::default();
    crate::configure_context(&context);
    let id = egui::Id::new("instance-five-byte-ax");
    let mut draft = SdfDraftText::with_byte_limit("65535", 5)?;
    let (mut output, _) = frame(&mut draft, &context, id, vec![], 0.0)?;
    let selection = CCursorRange::two(CCursor::new(0), CCursor::new(5));
    focus_selection(&context, id, &mut output, selection);
    let set_value = |value: &str| {
        egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::SetValue,
            target: egui::accesskit::NodeId(id.value()),
            data: Some(egui::accesskit::ActionData::Value(value.into())),
        })
    };
    let (mut output, outcome) = frame(
        &mut draft,
        &context,
        id,
        vec![set_value("123456"), egui::Event::Text("0".to_owned())],
        1.0,
    )?;
    assert_eq!(draft.as_str(), "65535");
    assert_eq!(output.galley.text(), "65535");
    assert!(!outcome.changed);
    assert_eq!(
        outcome.rejection,
        Some(DraftTextError::ByteLimit {
            attempted_bytes: 6,
            limit_bytes: 5
        })
    );
    focus_selection(&context, id, &mut output, selection);
    let (output, outcome) = frame(&mut draft, &context, id, vec![set_value("abc")], 2.0)?;
    assert_eq!(
        draft.as_str(),
        "abc",
        "decimal grammar is not an implicit buffer filter"
    );
    assert_eq!(output.galley.text(), "abc");
    assert!(outcome.changed);
    assert_eq!(outcome.rejection, None);
    Ok(())
}
