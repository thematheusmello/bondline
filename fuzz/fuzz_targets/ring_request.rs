#![no_main]

use bondline_commands::{Command, Engine, NoParams};
use bondline_model::BondOrder;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|params: &[u8]| {
    if params.len() > 16_384 {
        return;
    }
    for command in ["ring.create", "ui.ring_tool"] {
        let mut frame = format!(r#"{{"command":"{command}","params":"#).into_bytes();
        frame.extend_from_slice(params);
        frame.push(b'}');
        let mut engine = Engine::default();
        let before = engine.content().clone();
        let stamp = engine.capture_stamp();
        let tool = engine.effective_tool();
        // Preparation never completes file/render work, even for injected envelopes.
        match engine.prepare_json(&frame) {
            Err(_) => {
                assert_eq!(engine.content(), &before);
                assert!(engine.capture_stamp_matches(&stamp));
                assert_eq!(engine.effective_tool(), tool);
                let state = engine.state();
                assert_eq!(
                    (state.revision, state.undo_depth, state.redo_depth),
                    (0, 0, 0)
                );
            }
            Ok(prepared) => {
                drop(prepared);
                let state = engine.state();
                if command == "ui.ring_tool" {
                    assert_eq!(engine.content(), &before);
                    assert!(engine.capture_stamp_matches(&stamp));
                    assert_eq!(
                        (state.revision, state.undo_depth, state.redo_depth),
                        (0, 0, 0)
                    );
                    continue;
                }
                assert_eq!(
                    (state.revision, state.undo_depth, state.redo_depth),
                    (1, 1, 0)
                );
                let authored = engine.content().clone();
                let page = engine.document().unwrap().pages().first().unwrap();
                assert_eq!(page.fragments().len(), 1);
                let fragment = page.fragments().first().unwrap();
                assert!((3..=8).contains(&fragment.atoms().len()));
                assert_eq!(fragment.bonds().len(), fragment.atoms().len());
                for atom in fragment.atoms() {
                    assert_eq!(atom.element.get(), 6);
                    assert!(atom.position.is_some());
                    let mut degree = 0;
                    let mut weight = 0;
                    for bond in fragment.bonds() {
                        assert!(matches!(bond.order, BondOrder::Single | BondOrder::Double));
                        if bond.begin == atom.id || bond.end == atom.id {
                            degree += 1;
                            weight += if bond.order == BondOrder::Double {
                                2
                            } else {
                                1
                            };
                        }
                    }
                    assert_eq!(degree, 2);
                    assert!(weight <= 4);
                    assert_eq!(atom.explicit_hydrogens, Some(4 - weight));
                }
                engine.execute(Command::Undo(NoParams {})).unwrap();
                assert_eq!(engine.content(), &before);
                engine.execute(Command::Redo(NoParams {})).unwrap();
                assert_eq!(engine.content(), &authored);
            }
        }
    }
});
