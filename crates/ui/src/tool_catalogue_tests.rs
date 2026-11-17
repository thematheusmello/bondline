use super::tool_catalogue::ENTRIES;
use std::collections::BTreeSet;

#[test]
fn every_documented_tool_role_and_explicit_variant_has_one_honest_catalogue_entry() {
    let source_ids: BTreeSet<_> = include_str!("../../../docs/parity-checklist.md")
        .lines()
        .filter_map(|line| line.split('|').nth(1))
        .map(str::trim)
        .filter(|id| {
            ["TD-", "TQ-", "TB-"]
                .iter()
                .any(|prefix| id.starts_with(prefix))
        })
        .collect();
    assert_eq!(source_ids.len(), 124);
    assert_eq!(ENTRIES.len(), 140);
    assert_eq!(
        ENTRIES.iter().map(|e| e.source_id).collect::<BTreeSet<_>>(),
        source_ids
    );
    assert_eq!(
        ENTRIES.iter().map(|e| e.id).collect::<BTreeSet<_>>().len(),
        140
    );
    assert_eq!(
        ENTRIES
            .iter()
            .map(|e| e.label)
            .collect::<BTreeSet<_>>()
            .len(),
        140
    );
    assert!(ENTRIES.iter().all(|e| !e.label.is_empty()
        && !e.group.is_empty()
        && !e.label.contains("ChemDraw")
        && !e.label.contains("BioDraw")));
    let real: Vec<_> = ENTRIES.iter().filter_map(|e| e.binding_id).collect();
    assert_eq!(
        real,
        [
            "tool.bond.single",
            "tool.bond.double",
            "tool.bond.triple",
            "tool.selection",
            "tool.ring.3",
            "tool.ring.4",
            "tool.ring.5",
            "tool.ring.6",
            "tool.ring.7",
            "tool.ring.8",
            "tool.ring.benzene",
            "tool.ring.cyclopentadiene",
            "chain.acyclic",
            "chain.snaking"
        ]
    );
    assert_eq!(
        ENTRIES.iter().filter(|e| e.binding_id.is_none()).count(),
        126
    );
    for (id, count) in [
        ("TD-026", 6),
        ("TD-027", 2),
        ("TD-041", 4),
        ("TD-042", 4),
        ("TD-043", 3),
        ("TD-044", 3),
    ] {
        assert_eq!(
            ENTRIES.iter().filter(|e| e.source_id == id).count(),
            count,
            "{id}"
        );
    }
}

#[test]
fn every_icon_text_character_exists_in_the_actual_bundled_font() {
    use super::{configure_context, egui, tool_icons};
    let context = egui::Context::default();
    configure_context(&context);
    let output = context.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(120.0, 120.0),
            )),
            ..Default::default()
        },
        |context| {
            egui::CentralPanel::default().show(context, |ui| {
                for entry in ENTRIES {
                    tool_icons::paint(
                        ui.painter(),
                        egui::Rect::from_min_size(egui::pos2(20.0, 20.0), egui::vec2(28.0, 28.0)),
                        entry,
                        egui::Color32::GRAY,
                    );
                }
            });
        },
    );
    let mut missing = BTreeSet::new();
    for paint in &output.shapes {
        if let egui::Shape::Text(text) = &paint.shape {
            for section in &text.galley.job.sections {
                for ch in text.galley.job.text[section.byte_range.clone()].chars() {
                    if !context.fonts(|fonts| fonts.has_glyph(&section.format.font_id, ch)) {
                        missing.insert(ch);
                    }
                }
            }
        }
    }
    assert!(
        missing.is_empty(),
        "native icons must never show missing-font replacements: {missing:?}"
    );
}
