#![no_main]

use bondline_commands::{Command, Engine, NoParams};
use bondline_model::BondOrder;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|params: &[u8]| {
    if params.len() > 16_384 {
        return;
    }
    let mut engine = Engine::default();
    // A fixed original C2 gives the decoder actual targets without file or render work.
    drop(engine.prepare_json(br#"{"command":"bond.create","params":{"page_id":1,"begin_x_pt":72,"begin_y_pt":144,"end_x_pt":120,"end_y_pt":144}}"#).unwrap());
    let before = engine.content().clone();
    let stamp = engine.capture_stamp();
    let mut frame = br#"{"command":"ring.sprout","params":"#.to_vec();
    frame.extend_from_slice(params);
    frame.push(b'}');
    match engine.prepare_json(&frame) {
        Err(_) => {
            assert_eq!(engine.content(), &before);
            assert!(engine.capture_stamp_matches(&stamp));
            let state = engine.state();
            assert_eq!(
                (state.revision, state.undo_depth, state.redo_depth),
                (1, 1, 0)
            );
        }
        Ok(prepared) => {
            drop(prepared);
            let state = engine.state();
            assert_eq!(
                (state.revision, state.undo_depth, state.redo_depth),
                (2, 2, 0)
            );
            let authored = engine.content().clone();
            let page = engine.document().unwrap().pages().first().unwrap();
            assert_eq!(page.fragments().len(), 1);
            let fragment = page.fragments().first().unwrap();
            assert!((5..=10).contains(&fragment.atoms().len()));
            assert_eq!(fragment.bonds().len(), fragment.atoms().len());
            for atom in fragment.atoms() {
                assert_eq!(atom.element.get(), 6);
                assert!(atom.position.is_some());
                let mut weight = 0;
                for bond in fragment.bonds() {
                    assert!(matches!(bond.order, BondOrder::Single | BondOrder::Double));
                    if bond.begin == atom.id || bond.end == atom.id {
                        weight += if bond.order == BondOrder::Double {
                            2
                        } else {
                            1
                        };
                    }
                }
                assert!(weight <= 4);
                assert_eq!(atom.explicit_hydrogens, Some(4 - weight));
            }
            engine.execute(Command::Undo(NoParams {})).unwrap();
            assert_eq!(engine.content(), &before);
            engine.execute(Command::Redo(NoParams {})).unwrap();
            assert_eq!(engine.content(), &authored);
        }
    }
});
