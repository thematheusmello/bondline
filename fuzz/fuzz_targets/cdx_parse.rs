#![no_main]

use bondline_cdx::{Document, Error, Framing, Record, RecordKind, Writer};
use libfuzzer_sys::fuzz_target;

fn literal_metadata(fields: [Option<&[u8]>; 4], atom: &bondline_model::Atom) {
    let [isotope, charge, radical, hydrogens] = fields;
    let isotope = isotope.and_then(|raw| {
        let mass = i16::from_le_bytes(raw.try_into().unwrap());
        std::num::NonZeroU16::new(u16::try_from(mass).unwrap())
    });
    let charge = charge.map_or(0, |raw| match raw {
        [value] => i8::from_le_bytes([*value]),
        [a, b, c, d] => i8::try_from(i32::from_le_bytes([*a, *b, *c, *d]) >> 24).unwrap(),
        _ => panic!("unsupported charge width admitted"),
    });
    let radical = radical.map_or(bondline_model::Radical::None, |raw| match raw {
        [0] => bondline_model::Radical::None,
        [1] => bondline_model::Radical::Singlet,
        [2] => bondline_model::Radical::Doublet,
        [3] => bondline_model::Radical::Triplet,
        _ => panic!("unsupported radical value/width admitted"),
    });
    let hydrogens = hydrogens.map(|raw| u16::from_le_bytes(raw.try_into().unwrap()));
    assert_eq!(atom.isotope, isotope);
    assert_eq!(atom.charge, charge);
    assert_eq!(atom.radical, radical);
    assert_eq!(
        atom.explicit_hydrogens, hydrogens,
        "absence stays unresolved"
    );
}

fn body<'a>(document: &'a Document<'_>) -> Vec<&'a Record> {
    let explicit = document.framing().has_encoded_document();
    document.records().iter().enumerate().filter_map(|(index, record)| {
        let root = explicit && index == 0
            || matches!(record.kind(), RecordKind::ObjectEnd { start } if start.is_none() || explicit && *start == Some(0))
            || matches!(record.kind(), RecordKind::FileEnd);
        (!root).then_some(record)
    }).collect()
}

fn construct(document: &Document<'_>) -> Result<Vec<u8>, Error> {
    let body = body(document);
    assert_eq!(
        body.iter()
            .filter(|record| matches!(record.kind(), RecordKind::Property { .. }))
            .count(),
        document
            .records()
            .iter()
            .filter(|record| matches!(record.kind(), RecordKind::Property { .. }))
            .count(),
        "new-tree reconstruction must retain every source property"
    );
    let mut writer = Writer::new()?;
    for record in body {
        match record.kind() {
            RecordKind::ObjectStart { tag, id } => writer.start_object(*tag, *id)?,
            RecordKind::Property { tag, .. } => writer.property(*tag, document.payload(record)?)?,
            RecordKind::ObjectEnd { .. } => writer.end_object()?,
            RecordKind::FileEnd => {}
        }
    }
    writer.finish()
}

fuzz_target!(|data: &[u8]| {
    let Ok(document) = Document::parse(data) else {
        return;
    };
    assert_eq!(document.as_bytes(), data);
    assert_eq!(document.to_bytes(), data);
    let mut offset = document.header().len();
    for record in document.records() {
        let range = record.range();
        assert_eq!(range.start, offset);
        assert!(range.end <= data.len());
        offset = range.end;
    }
    assert_eq!(offset, data.len());

    if let Ok(graph) = bondline_io::cdx::ElementalGraph::parse(data) {
        assert_eq!(graph.as_bytes(), data);
        assert_eq!(graph.to_bytes(), data);
        assert_eq!(graph.source().as_bytes(), data);
        let mut fields = vec![[None; 4]; document.records().len()];
        for record in document.records() {
            let RecordKind::Property { tag, .. } = record.kind() else {
                continue;
            };
            let slot = match tag {
                0x0420 => 0,
                0x0421 => 1,
                0x0422 => 2,
                0x042b => 3,
                _ => continue,
            };
            let parent = record.parent().unwrap();
            assert!(matches!(
                document.records().get(parent).unwrap().kind(),
                RecordKind::ObjectStart { tag: 0x8004, .. }
            ));
            let previous = fields
                .get_mut(parent)
                .unwrap()
                .get_mut(slot)
                .unwrap()
                .replace(document.payload(record).unwrap());
            assert!(previous.is_none(), "duplicate admitted literal property");
        }
        let mut nodes =
            graph
                .source()
                .records()
                .iter()
                .enumerate()
                .filter_map(|(index, record)| {
                    matches!(record.kind(), RecordKind::ObjectStart { tag: 0x8004, .. })
                        .then_some(index)
                });
        let (mut atom_id, mut bond_id) = (0_u32, 0_u32);
        for (index, fragment) in graph.fragments().iter().enumerate() {
            assert_eq!(fragment.id().get() as usize, index + 1);
            let mut local_atoms = std::collections::BTreeSet::new();
            for atom in fragment.atoms() {
                atom_id += 1;
                assert_eq!(atom.id.get(), atom_id);
                literal_metadata(*fields.get(nodes.next().unwrap()).unwrap(), atom);
                assert!(local_atoms.insert(atom.id));
            }
            for bond in fragment.bonds() {
                bond_id += 1;
                assert_eq!(bond.id.get(), bond_id);
                assert!(local_atoms.contains(&bond.begin));
                assert!(local_atoms.contains(&bond.end));
            }
        }
        assert!(nodes.next().is_none());
        let mut stored = bondline_io::StoredDocument::from_cdx(
            std::sync::Arc::from(data),
            bondline_model::PageSize::A4,
        )
        .unwrap();
        assert_eq!(stored.export_cdx().unwrap(), data);
        assert!(!stored.is_fully_mapped());
        assert!(!stored.has_display_projection());
        let original = stored.clone();
        let page = stored.model().pages().first().unwrap().id();
        stored
            .model_mut()
            .set_page_size(page, bondline_model::PageSize::new(420.0, 594.0).unwrap())
            .unwrap();
        assert!(stored.source_info().unwrap().modified);
        assert!(stored.export_cdx().is_err());
        assert!(stored.to_cdxml().is_err());
        stored = original;
        assert_eq!(stored.to_cdx().unwrap().as_ref(), data);
    }

    if let Some((index, record)) = document
        .records()
        .iter()
        .enumerate()
        .find(|(_, record)| matches!(record.kind(), RecordKind::Property { .. }))
    {
        let payload = document.payload(record).unwrap();
        assert_eq!(document.replace_property(record, payload).unwrap(), data);
        let replacement = [0, 0xff, 0x80, 0, 1, 2, 3, 4];
        let output = document.replace_property(record, &replacement).unwrap();
        let edited = Document::parse(&output).unwrap();
        assert_eq!(edited.records().len(), document.records().len());
        assert_eq!(edited.framing(), document.framing());
        let changed = edited.records().get(index).unwrap();
        assert_eq!(edited.payload(changed).unwrap(), replacement);
        let original_range = record.range();
        let changed_range = changed.range();
        assert_eq!(
            data.get(..original_range.start),
            output.get(..changed_range.start)
        );
        assert_eq!(
            data.get(original_range.end..),
            output.get(changed_range.end..)
        );
        assert_eq!(document.as_bytes(), data);
    }

    // New syntax uses an explicit ID0 root and canonical lengths, not source passthrough.
    match construct(&document) {
        Ok(bytes) => {
            let written = Document::parse(&bytes).unwrap();
            assert_eq!(written.framing(), Framing::ExplicitDocument);
            let before = body(&document);
            let after = body(&written);
            assert_eq!(before.len(), after.len());
            for (before, after) in before.into_iter().zip(after) {
                match (before.kind(), after.kind()) {
                    (
                        RecordKind::ObjectStart { tag: a, id: b },
                        RecordKind::ObjectStart { tag: c, id: d },
                    ) => assert_eq!((a, b), (c, d)),
                    (RecordKind::Property { tag: a, .. }, RecordKind::Property { tag: b, .. }) => {
                        assert_eq!(a, b);
                        assert_eq!(
                            document.payload(before).unwrap(),
                            written.payload(after).unwrap()
                        );
                    }
                    (RecordKind::ObjectEnd { .. }, RecordKind::ObjectEnd { .. }) => {}
                    _ => panic!("new syntax changed object/property order"),
                }
            }
            assert_eq!(document.as_bytes(), data);
        }
        Err(error) => assert!(matches!(
            error,
            Error::InputTooLarge | Error::DepthLimit | Error::RecordLimit | Error::Allocation
        )),
    }
});
