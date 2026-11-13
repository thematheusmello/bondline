#![no_main]
use bondline_io::mol::{
    MolTransform,
    v3000::{MAX_BYTES, MAX_DEPTH, MAX_ITEMS, MolFile, write_v3000},
};
use bondline_model::Point2;
use libfuzzer_sys::fuzz_target;
use std::collections::BTreeSet;

fuzz_target!(|data: &[u8]| {
    if let Ok(raw) = MolFile::parse(data) {
        assert_eq!(raw.as_bytes(), data);
        assert!(data.len() <= MAX_BYTES);
        assert!(raw.atoms().len() <= MAX_ITEMS && raw.bonds().len() <= MAX_ITEMS);
        let mut ids = BTreeSet::new();
        for atom in raw.atoms() {
            assert!(atom.source_index > 0 && ids.insert(atom.source_index));
            assert!(data.get(atom.range.clone()).is_some());
            assert!(atom.x.is_finite() && atom.y.is_finite() && atom.z.is_finite());
        }
        for bond in raw.bonds() {
            assert!(bond.source_index > 0 && ids.contains(&bond.begin) && ids.contains(&bond.end));
            assert!(data.get(bond.range.clone()).is_some());
        }
        for (index, block) in raw.blocks().iter().enumerate() {
            assert!(data.get(block.range.clone()).is_some());
            let mut parent = block.parent;
            let mut depth = 1;
            while let Some(ancestor) = parent {
                assert!(ancestor < index);
                let outer = &raw.blocks()[ancestor];
                assert!(
                    outer.range.start <= block.range.start && outer.range.end >= block.range.end
                );
                parent = outer.parent;
                depth += 1;
                assert!(depth <= MAX_DEPTH);
            }
        }
        let frame = MolTransform::new(1.0, Point2::new(0.0, 0.0).unwrap()).unwrap();
        let graph = raw.to_fragment(frame);
        if !raw.unsupported().is_empty() {
            assert!(graph.is_err());
        }
        if let Ok(graph) = graph {
            match write_v3000(&graph, "Original V3000 fuzz graph", frame) {
                Ok(encoded) => {
                    assert!(encoded.len() <= MAX_BYTES);
                    assert!(
                        encoded
                            .split(|byte| *byte == b'\n')
                            .all(|line| line.len() <= 80)
                    );
                    let reread = MolFile::parse(&encoded).unwrap();
                    assert!(reread.unsupported().is_empty());
                    assert_eq!(reread.to_fragment(frame).unwrap(), graph);
                }
                Err(error) => assert_eq!(error.code(), "resource_limit"),
            }
        }
        assert_eq!(raw.as_bytes(), data);
    }
});
