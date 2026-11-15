//! A real wheel/keyboard/button interaction must leave the filename error visible.
use super::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn frame(
    context: &egui::Context,
    chooser: &mut FileChooser,
    events: Vec<egui::Event>,
) -> (Option<FileChooserEvent>, egui::FullOutput) {
    let mut action = None;
    let output = context.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1160.0, 800.0),
            )),
            focused: true,
            events,
            ..Default::default()
        },
        |context| action = chooser.show(context, false),
    );
    (action, output)
}

fn painted_text(output: &egui::FullOutput, expected: &str) -> Option<egui::Rect> {
    output.shapes.iter().find_map(|paint| match &paint.shape {
        egui::Shape::Text(text)
            if text.galley.text() == expected
                && paint
                    .clip_rect
                    .contains_rect(paint.shape.visual_bounding_rect()) =>
        {
            Some(paint.shape.visual_bounding_rect())
        }
        _ => None,
    })
}

fn pointer(position: egui::Pos2, pressed: bool) -> egui::Event {
    egui::Event::PointerButton {
        pos: position,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    }
}

fn select_all_key(pressed: bool) -> egui::Event {
    egui::Event::Key {
        key: egui::Key::A,
        physical_key: Some(egui::Key::A),
        pressed,
        repeat: false,
        modifiers: egui::Modifiers::COMMAND,
    }
}

#[test]
fn invalid_filename_error_is_painted_after_scrolling_the_directory_to_its_end() -> TestResult {
    let context = egui::Context::default();
    crate::configure_context(&context);
    let directory = std::env::temp_dir().join("bondline-picker-painted-error");
    let mut chooser = FileChooser {
        mode: Some(FileMode::Open),
        format: FileFormat::Cdxml,
        directory: directory.clone(),
        location: directory.to_str().ok_or("UTF8 directory")?.to_owned(),
        name: "selected.cdxml".to_owned(),
        entries: (0..32)
            .map(|index| DirectoryEntry {
                name: format!("entry-{index:04}.cdxml"),
                directory: false,
            })
            .collect(),
        generation: 1,
        focus_name: true,
        ..Default::default()
    };
    // Deterministic directory metadata only: no worker, document I/O or direct submit.
    for _ in 0..4 {
        assert!(frame(&context, &mut chooser, vec![]).0.is_none());
    }
    let (_, before_scroll) = frame(&context, &mut chooser, vec![]);
    let list_point = painted_text(&before_scroll, "entry-0000.cdxml")
        .ok_or("first directory entry must initially be painted")?
        .center();
    assert!(painted_text(&before_scroll, "entry-0031.cdxml").is_none());
    let focused = context.memory(|memory| memory.focused());
    assert!(
        focused.is_some(),
        "normal chooser startup focuses its filename"
    );
    assert!(
        frame(
            &context,
            &mut chooser,
            vec![
                egui::Event::PointerMoved(list_point),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0.0, -4096.0),
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        )
        .0
        .is_none()
    );
    let (_, scrolled) = frame(&context, &mut chooser, vec![]);
    assert!(painted_text(&scrolled, "entry-0000.cdxml").is_none());
    assert!(
        painted_text(&scrolled, "entry-0031.cdxml").is_some(),
        "real wheel input must reach the final painted entry"
    );
    assert_eq!(context.memory(|memory| memory.focused()), focused);
    assert!(
        frame(
            &context,
            &mut chooser,
            vec![
                select_all_key(true),
                select_all_key(false),
                egui::Event::Text("unsupported.rxn".to_owned())
            ],
        )
        .0
        .is_none()
    );
    assert_eq!(
        chooser.name, "unsupported.rxn",
        "actual filename keyboard edit"
    );
    let (_, named) = frame(&context, &mut chooser, vec![]);
    let open = painted_text(&named, "Open")
        .ok_or("visible Open caption")?
        .center();
    assert!(
        frame(
            &context,
            &mut chooser,
            vec![egui::Event::PointerMoved(open), pointer(open, true)]
        )
        .0
        .is_none()
    );
    assert!(
        frame(&context, &mut chooser, vec![pointer(open, false)])
            .0
            .is_none()
    );
    let expected = "Choose a .cdxml, .cdx, .mol, .sdf, .sd, .smi or .smiles file.";
    assert_eq!(chooser.error.as_deref(), Some(expected));
    assert!(chooser.is_open());
    frame(&context, &mut chooser, vec![]);
    let (_, error) = frame(&context, &mut chooser, vec![]);
    assert!(
        painted_text(&error, expected).is_some(),
        "the filename error must be fully painted inside its clip after a scrolled-list failure"
    );
    assert_eq!(chooser.name, "unsupported.rxn");
    Ok(())
}
