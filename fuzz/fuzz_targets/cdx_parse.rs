#![no_main]

use bondline_cdx::{
    Document, Error, Framing, Record, RecordKind, ResourceError, StringError, Writer,
};
use libfuzzer_sys::fuzz_target;

fn check_string(document: &Document<'_>, index: usize, record: &Record) {
    let RecordKind::Property { payload, .. } = record.kind() else {
        return;
    };
    let raw = &document.as_bytes()[payload.clone()];
    let shape = raw.first_chunk::<2>().and_then(|count| {
        let runs_end = 2 + usize::from(u16::from_le_bytes(*count)) * 10;
        raw.get(2..runs_end).zip(raw.get(runs_end..))
    });
    let result = document.string(record);
    let Some((styles, text)) = shape else {
        assert_eq!(
            result.unwrap_err(),
            StringError::InvalidPayload { record: index }
        );
        return;
    };
    let view = result.unwrap();
    assert!(std::ptr::eq(view.record(), record));
    assert!(std::ptr::eq(view.text_bytes(), text));
    assert_eq!(view.text_range(), payload.end - text.len()..payload.end);
    let runs = view.runs();
    assert_eq!(runs.len(), styles.len() / 10);
    for (ordinal, (run, raw_run)) in runs.zip(styles.chunks_exact(10)).enumerate() {
        let start = payload.start + 2 + ordinal * 10;
        assert_eq!(run.range(), start..start + 10);
        assert!(
            [
                run.start(),
                run.font(),
                run.face(),
                run.size_twentieths(),
                run.color(),
            ]
            .into_iter()
            .flat_map(u16::to_le_bytes)
            .eq(raw_run.iter().copied())
        );
    }
}

fn check_resources(document: &Document<'_>) {
    let result = document.resources();
    let parent = document.framing().has_encoded_document().then_some(0);
    let mut tables = [None, None];
    for (index, record) in document.records().iter().enumerate() {
        let RecordKind::Property { tag, .. } = *record.kind() else {
            continue;
        };
        let slot = match tag {
            0x0100 => &mut tables[0],
            0x0300 => &mut tables[1],
            _ => continue,
        };
        if record.parent() != parent {
            assert_eq!(
                result.unwrap_err(),
                ResourceError::UnsupportedTable { tag, record: index }
            );
            return;
        }
        if slot.replace(record).is_some() {
            assert_eq!(
                result.unwrap_err(),
                ResourceError::DuplicateTable { tag, record: index }
            );
            return;
        }
    }

    let Ok(resources) = result else {
        return;
    };
    let [font_table, color_table] = tables;
    let word = |bytes: &[u8]| u16::from_le_bytes(bytes.try_into().unwrap());
    let bytes = document.as_bytes();
    match (font_table, resources.font_table()) {
        (Some(raw), Some(table)) => {
            assert!(std::ptr::eq(raw, table));
            let RecordKind::Property { payload, .. } = raw.kind() else {
                unreachable!();
            };
            let mut cursor = payload.start + 4;
            let platform = word(&bytes[payload.start..payload.start + 2]);
            assert!(matches!(platform, 0 | 1));
            assert_eq!(resources.font_platform(), Some(platform));
            assert_eq!(
                resources.fonts().len(),
                usize::from(word(&bytes[payload.start + 2..cursor]))
            );
            for (ordinal, font) in resources.fonts().iter().enumerate() {
                assert!(std::ptr::eq(font.record(), raw));
                assert_eq!(font.ordinal(), ordinal);
                assert_eq!(font.id(), word(&bytes[cursor..cursor + 2]));
                assert_eq!(font.charset(), word(&bytes[cursor + 2..cursor + 4]));
                let name_len = usize::from(word(&bytes[cursor + 4..cursor + 6]));
                let end = cursor + 6 + name_len;
                assert!(end <= payload.end);
                assert_eq!(font.range(), cursor..end);
                let name = &bytes[cursor + 6..end];
                assert_eq!(font.name_bytes(), name);
                assert!(std::ptr::eq(font.name_bytes(), name));
                assert!(std::ptr::eq(resources.font_by_id(font.id()).unwrap(), font));
                cursor = end;
            }
            assert_eq!(cursor, payload.end);
        }
        (None, None) => {
            assert_eq!(resources.font_platform(), None);
            assert!(resources.fonts().is_empty());
            assert!(matches!(
                resources.font_by_id(0),
                Err(ResourceError::MissingFont(0))
            ));
        }
        _ => panic!("font table presence changed"),
    }
    assert_eq!(resources.color(0).unwrap(), [0; 3]);
    assert_eq!(resources.color(1).unwrap(), [u16::MAX; 3]);
    match (color_table, resources.color_table()) {
        (Some(raw), Some(table)) => {
            assert!(std::ptr::eq(raw, table));
            let RecordKind::Property { payload, .. } = raw.kind() else {
                unreachable!();
            };
            let mut cursor = payload.start + 2;
            assert_eq!(
                resources.colors().len(),
                usize::from(word(&bytes[payload.start..cursor]))
            );
            for (ordinal, color) in resources.colors().iter().enumerate() {
                let index = u32::try_from(ordinal).unwrap() + 2;
                let end = cursor + 6;
                assert!(end <= payload.end);
                let rgb = [
                    word(&bytes[cursor..cursor + 2]),
                    word(&bytes[cursor + 2..cursor + 4]),
                    word(&bytes[cursor + 4..end]),
                ];
                assert!(std::ptr::eq(color.record(), raw));
                assert_eq!(color.index(), index);
                assert_eq!(color.range(), cursor..end);
                assert_eq!(color.rgb(), rgb);
                assert_eq!(resources.color(index).unwrap(), rgb);
                cursor = end;
            }
            assert_eq!(cursor, payload.end);
            let missing = u32::try_from(resources.colors().len()).unwrap() + 2;
            assert_eq!(
                resources.color(missing),
                Err(ResourceError::MissingColor(missing))
            );
        }
        (None, None) => {
            assert!(resources.colors().is_empty());
            assert_eq!(resources.color(2).unwrap(), [u16::MAX; 3]);
            assert_eq!(resources.color(3).unwrap(), [0; 3]);
            assert_eq!(resources.color(4), Err(ResourceError::MissingColor(4)));
        }
        _ => panic!("color table presence changed"),
    }
}

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

fn plain_bond(fields: [Option<&[u8]>; 4], bond: &bondline_model::Bond) {
    use bondline_model::{BondOrder, DoublePosition};
    let [order, display, display2, position] = fields;
    let order = match order.map(|raw| u16::from_le_bytes(raw.try_into().unwrap())) {
        None | Some(1) => BondOrder::Single,
        Some(2) => BondOrder::Double,
        Some(4) => BondOrder::Triple,
        _ => panic!("unsupported/query order admitted"),
    };
    for raw in display.into_iter().chain(display2) {
        assert_eq!(i16::from_le_bytes(raw.try_into().unwrap()), 0);
    }
    if display2.is_some() || position.is_some() {
        assert_eq!(order, BondOrder::Double, "non-Double metadata admitted");
    }
    let position = match position.map(|raw| i16::from_le_bytes(raw.try_into().unwrap())) {
        None => DoublePosition::Unspecified,
        Some(0 | 256) => DoublePosition::Center,
        Some(1 | 257) => DoublePosition::Right,
        Some(2 | 258) => DoublePosition::Left,
        _ => panic!("unsupported position admitted"),
    };
    assert_eq!(bond.order, order);
    assert_eq!(bond.double_position, position);
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
    for (index, record) in document.records().iter().enumerate() {
        let range = record.range();
        assert_eq!(range.start, offset);
        assert!(range.end <= data.len());
        offset = range.end;
        check_string(&document, index, record);
    }
    assert_eq!(offset, data.len());
    check_resources(&document);

    if document
        .records()
        .iter()
        .any(|record| matches!(record.kind(), RecordKind::ObjectStart { tag: 0x8006, .. }))
        && let Ok(mut stored) = bondline_io::StoredDocument::from_cdx(
            std::sync::Arc::from(data),
            bondline_model::PageSize::A4,
        )
        && stored.has_cdx_caption_preview()
    {
        assert_eq!(stored.export_cdx().unwrap(), data);
        assert!(!stored.is_fully_mapped());
        assert!(!stored.has_caption_projection());
        assert!(stored.has_display_projection());
        assert!(stored.to_cdxml().is_err());
        let baseline = stored.clone();
        let page = stored.model().pages().first().unwrap();
        assert!(page.fragments().is_empty());
        let source_texts = document
            .records()
            .iter()
            .filter(|record| matches!(record.kind(), RecordKind::ObjectStart { tag: 0x8006, .. }))
            .count();
        assert!(source_texts > 0);
        assert_eq!(page.texts().len(), source_texts);
        let mut original: Vec<_> = document
            .records()
            .iter()
            .filter(|record| matches!(record.kind(), RecordKind::Property { tag: 0x0700, .. }))
            .map(|record| {
                let value = document.string(record).unwrap();
                let starts: Vec<_> = value.runs().map(|run| usize::from(run.start())).collect();
                (value.text_bytes().to_vec(), starts)
            })
            .collect();
        let mut projected: Vec<_> = page
            .texts()
            .iter()
            .map(|text| {
                let mut bytes = Vec::new();
                let mut starts = Vec::new();
                for run in text.runs() {
                    starts.push(bytes.len());
                    bytes.extend_from_slice(run.text().as_bytes());
                }
                (bytes, starts)
            })
            .collect();
        original.sort_unstable();
        projected.sort_unstable();
        assert_eq!(original, projected);
        let page_id = page.id();
        // One point changes the validated positive extent even if the source uses A4.
        let changed =
            bondline_model::PageSize::new(page.size().width_pt() + 1.0, page.size().height_pt())
                .unwrap();
        stored.model_mut().set_page_size(page_id, changed).unwrap();
        assert!(!stored.has_display_projection());
        assert!(stored.to_cdx().is_err());
        assert_eq!(baseline.to_cdx().unwrap().as_ref(), data);
    }

    if let Ok(graph) = bondline_io::cdx::ElementalGraph::parse(data) {
        assert_eq!(graph.as_bytes(), data);
        assert_eq!(graph.to_bytes(), data);
        assert_eq!(graph.source().as_bytes(), data);
        let mut fields = vec![[None; 4]; document.records().len()];
        for record in document.records() {
            let RecordKind::Property { tag, .. } = record.kind() else {
                continue;
            };
            let (owner, slot) = match tag {
                0x0420 => (0x8004, 0),
                0x0421 => (0x8004, 1),
                0x0422 => (0x8004, 2),
                0x042b => (0x8004, 3),
                0x0600 => (0x8005, 0),
                0x0601 => (0x8005, 1),
                0x0602 => (0x8005, 2),
                0x0603 => (0x8005, 3),
                _ => continue,
            };
            let parent = record.parent().unwrap();
            assert!(matches!(
                document.records().get(parent).unwrap().kind(),
                RecordKind::ObjectStart { tag, .. } if *tag == owner
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
        let mut source_bonds =
            document
                .records()
                .iter()
                .enumerate()
                .filter_map(|(index, record)| {
                    matches!(record.kind(), RecordKind::ObjectStart { tag: 0x8005, .. })
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
                plain_bond(*fields.get(source_bonds.next().unwrap()).unwrap(), bond);
                assert!(local_atoms.contains(&bond.begin));
                assert!(local_atoms.contains(&bond.end));
            }
        }
        assert!(nodes.next().is_none());
        assert!(source_bonds.next().is_none());
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
