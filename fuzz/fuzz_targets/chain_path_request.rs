#![no_main]

use bondline_commands::{Command, Engine, NoParams};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|params: &[u8]| {
    if params.len() > 16_384 {
        return;
    }
    let mut frame = br#"{"command":"chain.create_path","params":"#.to_vec();
    frame.extend_from_slice(params);
    frame.push(b'}');
    let mut engine = Engine::default();
    let before = engine.content().clone();
    let stamp = engine.capture_stamp();
    // Preparation covers decoding/authoring; never complete file or render work.
    match engine.prepare_json(&frame) {
        Err(_) => {
            assert_eq!(engine.content(), &before);
            assert!(engine.capture_stamp_matches(&stamp));
            let state = engine.state();
            assert_eq!(
                (state.revision, state.undo_depth, state.redo_depth),
                (0, 0, 0)
            );
        }
        Ok(prepared) => {
            drop(prepared);
            let state = engine.state();
            assert_eq!(
                (state.revision, state.undo_depth, state.redo_depth),
                (1, 1, 0)
            );
            let authored = engine.content().clone();
            let page = engine.document().unwrap().pages().first().unwrap();
            assert_eq!(page.fragments().len(), 1);
            let fragment = page.fragments().first().unwrap();
            assert!((2..=64).contains(&fragment.atoms().len()));
            assert_eq!(fragment.bonds().len() + 1, fragment.atoms().len());
            for (i, atom) in fragment.atoms().iter().enumerate() {
                assert_eq!(atom.element.get(), 6);
                assert!(atom.position.is_some());
                assert_eq!(
                    atom.explicit_hydrogens,
                    Some(if i == 0 || i + 1 == fragment.atoms().len() {
                        3
                    } else {
                        2
                    })
                );
                assert_eq!(atom.show_terminal_carbon_labels, Some(false));
                assert_eq!(atom.show_non_terminal_carbon_labels, Some(false));
            }
            engine.execute(Command::Undo(NoParams {})).unwrap();
            assert_eq!(engine.content(), &before);
            engine.execute(Command::Redo(NoParams {})).unwrap();
            assert_eq!(engine.content(), &authored);
        }
    }
});
