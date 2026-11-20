//! Public-format CDX witnesses exercised through native App pointer and form events.
use super::{
    Fixture, TestResult, canvas, click_alt, close_form, painted, projected, set, shortcut,
    state_without_form, value,
};
use eframe::egui::{Event, Key, Modifiers, PointerButton};
use serde_json::json;
use std::fs;

struct Example {
    source: &'static [u8],
    expected: &'static [u8],
    atom_id: u32,
    point: [f64; 2],
    original_h: &'static str,
    symbol: &'static str,
    hydrogens: &'static str,
}

fn exercise(example: Example, retained_text: bool) -> TestResult {
    let mut f = Fixture::new(1160., 800., 1.)?;
    let input = f.directory.path("skeletal-原.cdx");
    let output = f.directory.path("new-label-保存.cdx");
    fs::write(&input, example.source)?;
    if retained_text {
        f.choose("Text")?;
    } else {
        f.command("ui.ring_tool", json!({"size":8}))?;
    }
    f.command("document.open_cdx", json!({"path":input}))?;
    f.command("ui.view", json!({"action":"actual"}))?;
    let before = state_without_form(&f)?;
    let screen = projected(canvas(&f.warm())?, [512., 512.], 100., None, example.point);
    let extra_modifier = Modifiers {
        alt: true,
        shift: true,
        ..Modifiers::NONE
    };
    for (button, modifiers) in [
        (PointerButton::Primary, extra_modifier),
        (PointerButton::Secondary, Modifiers::ALT),
    ] {
        for pressed in [true, false] {
            f.frame(vec![
                Event::PointerMoved(screen),
                Event::PointerButton {
                    pos: screen,
                    button,
                    pressed,
                    modifiers,
                },
            ]);
        }
        assert!(f.state()?["ui_atom_label"].is_null());
        assert_eq!(state_without_form(&f)?, before);
    }
    click_alt(&mut f, [350., 300.])?;
    assert!(f.state()?["ui_atom_label"].is_null());
    click_alt(&mut f, example.point)?;
    let form = f.state()?["ui_atom_label"].clone();
    assert_eq!(
        form["atom_id"], example.atom_id,
        "exact Alt on the exposed carbon must open its label form"
    );
    assert_eq!(
        form["creation_style"],
        json!({"font_family":"Roboto", "size_pt":12.0, "face":0, "rgb":[0.0,0.0,0.0]})
    );
    let shown = f.warm();
    assert!(painted(
        &shown,
        "New printed label · Roboto Regular · 12 pt · Black"
    ));
    assert_eq!(value(&f, "Element")?, "C");
    assert_eq!(value(&f, "Attached hydrogens")?, example.original_h);
    close_form(&mut f, "Cancel")?;
    assert_eq!(state_without_form(&f)?, before);

    click_alt(&mut f, example.point)?;
    f.warm();
    set(&mut f, "Element", example.symbol)?;
    set(&mut f, "Attached hydrogens", example.hydrogens)?;
    close_form(&mut f, "Apply")?;
    let edited = f.state()?;
    assert_eq!(edited["undo_depth"], 1);
    assert_eq!(edited["file"]["dirty"], true);
    shortcut(&mut f, Key::Z, false);
    assert_eq!(f.state()?["document"], before["document"]);
    assert_eq!(
        f.app
            .engine
            .lock()
            .map_err(|_| "engine")?
            .stored_document()?
            .export_cdx()?,
        example.source
    );
    shortcut(&mut f, Key::Z, true);
    assert_eq!(f.state()?["document"], edited["document"]);
    f.command("document.save_cdx", json!({"path":output}))?;
    assert_eq!(fs::read(&input)?, example.source);
    assert_eq!(fs::read(&output)?, example.expected);
    f.command("document.open_cdx", json!({"path":output}))?;
    f.command("ui.view", json!({"action":"actual"}))?;
    let reopened = f.state()?;
    assert_eq!(reopened["undo_depth"], 0);
    assert_eq!(reopened["document"], edited["document"]);
    click_alt(&mut f, example.point)?;
    let request = f.state()?["ui_atom_label"].clone();
    assert_eq!(request["atom_id"], example.atom_id);
    assert!(request.get("creation_style").is_none());
    f.warm();
    assert_eq!(value(&f, "Element")?, example.symbol);
    assert_eq!(value(&f, "Attached hydrogens")?, example.hydrogens);
    close_form(&mut f, "Cancel")?;
    assert_eq!(f.state()?["undo_depth"], 0);
    Ok(())
}

#[test]
fn cdx_skeletal_creation_uses_source_absence_in_ethanol() -> TestResult {
    exercise(
        Example {
            source: include_bytes!("../../io/tests/fixtures/cdx-mixed-skeleton/ethanol.cdx"),
            expected: include_bytes!(
                "../../io/tests/fixtures/cdx-skeletal-label-creation/ethanol.cdx"
            ),
            atom_id: 1,
            point: [56., 144.],
            original_h: "",
            symbol: "N",
            hydrogens: "2",
        },
        false,
    )
}

#[test]
fn cdx_skeletal_creation_uses_literal_h_with_text_retained() -> TestResult {
    exercise(
        Example {
            source: include_bytes!("../../io/tests/fixtures/cdx-mixed-skeleton/literal_h.cdx"),
            expected: include_bytes!("../tests/fixtures/literal_h_native.cdx"),
            atom_id: 1,
            point: [128., 192.],
            original_h: "3",
            symbol: "N",
            hydrogens: "2",
        },
        true,
    )
}

#[test]
fn cdx_skeletal_creation_adds_first_label_to_all_carbon_paper() -> TestResult {
    exercise(
        Example {
            source: include_bytes!("../../io/tests/fixtures/cdx-skeletal/propane.cdx"),
            expected: include_bytes!(
                "../../io/tests/fixtures/cdx-skeletal-label-creation/propane.cdx"
            ),
            atom_id: 2,
            point: [224., 192.],
            original_h: "2",
            symbol: "O",
            hydrogens: "0",
        },
        true,
    )
}
