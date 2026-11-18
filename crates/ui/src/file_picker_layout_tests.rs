//! Deterministic directory-completion layout and last-painted pointer acceptance.
use super::*;
use egui::accesskit::{Action, Node, NodeId, Role};
use std::collections::HashMap;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn frame(
    context: &egui::Context,
    chooser: &mut FileChooser,
    nodes: &mut HashMap<NodeId, Node>,
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
        |context| {
            if let Some(result) = chooser.show(context, false) {
                action = Some(result);
            }
        },
    );
    if let Some(update) = &output.platform_output.accesskit_update {
        nodes.extend(update.nodes.iter().cloned());
    }
    (action, output)
}

fn button(nodes: &HashMap<NodeId, Node>, name: &str) -> TestResult<egui::Rect> {
    let root = NodeId(egui::accesskit_root_id().value());
    let mut seen = std::collections::HashSet::new();
    let mut pending = vec![root];
    while let Some(id) = pending.pop() {
        if seen.insert(id)
            && let Some(node) = nodes.get(&id)
        {
            pending.extend_from_slice(node.children());
        }
    }
    nodes
        .iter()
        .filter(|(id, node)| {
            seen.contains(id)
                && node.role() == Role::Button
                && node.name() == Some(name)
                && !node.is_disabled()
                && node.supports_action(Action::Focus)
                && node.default_action_verb() == Some(egui::accesskit::DefaultActionVerb::Click)
        })
        .find_map(|(_, node)| {
            let bounds = node.bounds()?;
            Some(egui::Rect::from_min_max(
                egui::pos2(bounds.x0 as f32, bounds.y0 as f32),
                egui::pos2(bounds.x1 as f32, bounds.y1 as f32),
            ))
        })
        .ok_or_else(|| format!("current actionable {name} button").into())
}

fn pointer(position: egui::Pos2, pressed: bool) -> egui::Event {
    egui::Event::PointerButton {
        pos: position,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    }
}

fn metadata(context: &egui::Context, nodes: &HashMap<NodeId, Node>, caption: &str) -> Vec<String> {
    let root = NodeId(egui::accesskit_root_id().value());
    let mut seen = std::collections::HashSet::new();
    let mut pending = vec![root];
    while let Some(id) = pending.pop() {
        if seen.insert(id)
            && let Some(node) = nodes.get(&id)
        {
            pending.extend_from_slice(node.children());
        }
    }
    let mut result = nodes
        .iter()
        .filter(|(_, node)| {
            (node.role() == Role::Button && node.name() == Some(caption))
                || matches!(node.role(), Role::Window | Role::Dialog)
        })
        .map(|(id, node)| {
            format!(
                "{id:?} role={:?} reachable={} enabled={} bounds={:?}",
                node.role(),
                seen.contains(id),
                !node.is_disabled(),
                node.bounds()
            )
        })
        .collect::<Vec<_>>();
    result.push(format!(
        "Window area_rect={:?}",
        context.memory(|memory| memory.area_rect(egui::Id::new("bondline-file-chooser")))
    ));
    result.sort();
    result
}

#[test]
fn directory_completion_preserves_clicks_on_the_last_painted_open_and_save_buttons() -> TestResult {
    for mode in [FileMode::Open, FileMode::Save] {
        for (case, count, truncated, omitted) in [
            ("stable-loading-control", None, false, 0),
            ("scan-error", Some(0), false, 0),
            ("one-entry", Some(1), false, 0),
            ("many-entries", Some(32), false, 0),
            ("truncated-and-omitted", Some(32), true, 9),
        ] {
            let context = egui::Context::default();
            crate::configure_context(&context);
            context.enable_accesskit();
            let directory = std::env::temp_dir().join("bondline-picker-layout-independent");
            let name = "selected-λ.cdxml";
            // This is only deterministic directory-result state. There is no worker,
            // document I/O, sleep, direct submit call or fabricated chemical result.
            let mut chooser = FileChooser {
                mode: Some(mode),
                format: FileFormat::Cdxml,
                directory: directory.clone(),
                location: directory.to_str().ok_or("UTF8 directory")?.to_owned(),
                name: name.to_owned(),
                generation: 1,
                loading: true,
                ..Default::default()
            };
            let mut nodes = HashMap::new();
            let caption = if mode == FileMode::Open {
                "Open"
            } else {
                "Save"
            };
            for _ in 0..2 {
                let (action, _) = frame(&context, &mut chooser, &mut nodes, vec![]);
                assert!(action.is_none());
            }
            let mut previous = None;
            let mut painted = None;
            for _ in 0..2 {
                let (action, output) = frame(&context, &mut chooser, &mut nodes, vec![]);
                assert!(action.is_none());
                let bounds = button(&nodes, caption)?;
                let point = bounds.center();
                assert!(
                    output.shapes.iter().any(|paint| {
                        matches!(&paint.shape,egui::Shape::Text(text)
                        if text.galley.text()==caption
                        && paint.clip_rect.contains(point)
                        && paint.shape.visual_bounding_rect().contains(point))
                    }),
                    "button bounds must refer to an actually painted caption"
                );
                previous = painted;
                painted = Some(bounds);
            }
            assert_eq!(previous, painted, "warm loading layout must be stable");
            let old_bounds = painted.ok_or("last painted bounds")?;
            assert!(chooser.loading && chooser.worker.is_none() && chooser.pending.is_none());
            if let Some(count) = count {
                chooser.accept_scan(DirectoryScan {
                    generation: 1,
                    path: directory.clone(),
                    entries: (0..count)
                        .map(|index| DirectoryEntry {
                            name: format!("entry-{index:04}.cdxml"),
                            directory: false,
                        })
                        .collect(),
                    truncated,
                    omitted,
                    error: (case == "scan-error").then(|| "Directory listing failed.".to_owned()),
                });
                assert!(!chooser.loading);
            }
            let old_point = old_bounds.center();
            eprintln!(
                "{mode:?}/{case} BEFORE {:?}",
                metadata(&context, &nodes, caption)
            );
            let (pressed, _) = frame(
                &context,
                &mut chooser,
                &mut nodes,
                vec![
                    egui::Event::PointerMoved(old_point),
                    pointer(old_point, true),
                ],
            );
            eprintln!(
                "{mode:?}/{case} DOWN {:?}",
                metadata(&context, &nodes, caption)
            );
            let moved_bounds = button(&nodes, caption).ok();
            let (released, _) = frame(
                &context,
                &mut chooser,
                &mut nodes,
                vec![pointer(old_point, false)],
            );
            eprintln!(
                "{mode:?}/{case} UP {:?}",
                metadata(&context, &nodes, caption)
            );
            let expected_path = directory
                .join(name)
                .into_os_string()
                .into_string()
                .map_err(|_| "UTF8 expected path")?;
            let expected = FileChooserEvent::Submit(match mode {
                FileMode::Open => FileAction::Open {
                    path: expected_path,
                    discard_unsaved: false,
                },
                FileMode::Save => FileAction::Save {
                    path: expected_path,
                    overwrite: false,
                },
            });
            assert_eq!(
                released.or(pressed),
                Some(expected),
                "{mode:?}/{case}: completion must not lose the click from last painted \
                 bounds {old_bounds:?} to current {moved_bounds:?}"
            );
            assert!(!chooser.is_open());
            assert_eq!(chooser.name, name);
        }
    }
    Ok(())
}
