#![no_main]

use bondline_cdx::{
    Document, Error, Framing, MAX_INPUT_BYTES, MAX_RECORDS, PropertyInsertion, Record, RecordKind,
    ResourceError, StringError, Writer,
};
use libfuzzer_sys::fuzz_target;
use std::collections::BTreeMap;

fn check_child_splice(document: &Document<'_>, input: &[u8]) {
    let Some((index, end)) = document
        .records()
        .iter()
        .enumerate()
        .find(|(_, record)| matches!(record.kind(), RecordKind::ObjectEnd { start: Some(_) }))
    else {
        return;
    };
    let value = &input[..input.len().min(64)];
    let output = match document.append_child(end, 0xc132, 0, &[(0x4132, value)]) {
        Ok(bytes) => bytes,
        Err(Error::InputTooLarge | Error::RecordLimit | Error::DepthLimit | Error::Allocation) => {
            return;
        }
        Err(error) => panic!("valid owned child splice refused: {error}"),
    };
    let (prefix, suffix) = input.split_at(end.range().start);
    let mut insertion = vec![0x32, 0xc1, 0, 0, 0, 0, 0x32, 0x41, value.len() as u8, 0];
    insertion.extend_from_slice(value);
    insertion.extend_from_slice(&[0, 0]);
    assert_eq!(output, [prefix, &insertion, suffix].concat());
    let reopened = Document::parse(&output).unwrap();
    assert_eq!(reopened.framing(), document.framing());
    let RecordKind::ObjectEnd { start } = end.kind() else {
        unreachable!();
    };
    assert_eq!(reopened.records()[index].parent(), *start);
    assert_eq!(reopened.records().len(), document.records().len() + 3);
}

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

fn child_label_records(document: &Document<'_>) -> BTreeMap<usize, [Option<usize>; 3]> {
    let mut owners = BTreeMap::new();
    let mut fields = BTreeMap::new();
    for (index, record) in document.records().iter().enumerate() {
        match *record.kind() {
            RecordKind::ObjectStart { tag: 0x8006, .. } => {
                let node = record.parent().unwrap();
                assert!(matches!(
                    document.records()[node].kind(),
                    RecordKind::ObjectStart { tag: 0x8004, .. }
                ));
                owners.insert(index, node);
                assert!(fields.insert(node, [None; 3]).is_none());
            }
            RecordKind::Property { tag, .. } => {
                if let Some(node) = record.parent().and_then(|parent| owners.get(&parent)) {
                    let slot = match tag {
                        0x0700 => 0,
                        0x0200 => 1,
                        0x0204 => 2,
                        _ => continue,
                    };
                    assert!(fields.get_mut(node).unwrap()[slot].replace(index).is_none());
                }
            }
            _ => {}
        }
    }
    fields
}

fn check_label(
    document: &Document<'_>,
    resources: &bondline_cdx::Resources<'_, '_>,
    [text, position, bounds]: [Option<usize>; 3],
    label: &bondline_model::AtomLabel,
) {
    let record = |index| document.records().get(index).unwrap();
    let coordinate =
        |bytes: &[u8]| f64::from(i32::from_le_bytes(bytes.try_into().unwrap())) / 65536.0;
    let position = position.map(|index| {
        let raw = document.payload(record(index)).unwrap();
        [coordinate(&raw[4..]), coordinate(&raw[..4])]
    });
    let bounds = bounds.map(|index| {
        let raw = document.payload(record(index)).unwrap();
        [
            coordinate(&raw[4..8]),
            coordinate(&raw[..4]),
            coordinate(&raw[12..]),
            coordinate(&raw[8..12]),
        ]
    });
    assert_eq!(label.position().map(|p| [p.x_pt(), p.y_pt()]), position);
    assert_eq!(label.bounding_box(), bounds);

    let value = document.string(record(text.unwrap())).unwrap();
    assert_eq!(value.runs().len(), label.runs().len());
    assert_eq!(resources.font_platform(), Some(1));
    let mut offset = 0;
    for (raw, projected) in value.runs().zip(label.runs()) {
        assert_eq!(usize::from(raw.start()), offset);
        let end = offset + projected.text().len();
        assert_eq!(
            &value.text_bytes()[offset..end],
            projected.text().as_bytes()
        );
        offset = end;
        let font = &resources.fonts()[usize::from(raw.font())];
        assert_eq!(font.id(), raw.font());
        assert_eq!(font.charset(), 65001);
        let style = projected.style();
        assert_eq!(font.name_bytes(), style.font_family().as_bytes());
        assert_eq!(style.face(), raw.face());
        assert_eq!(style.size_pt(), f64::from(raw.size_twentieths()) / 20.0);
        assert_eq!(
            style.rgb(),
            resources
                .color(u32::from(raw.color()))
                .unwrap()
                .map(|n| f64::from(n) / 65535.0)
        );
    }
    assert_eq!(offset, value.text_bytes().len());
}

fn check_label_preview(document: &Document<'_>, model: &bondline_model::Document) {
    let property = |wanted| {
        let mut records = document
            .records()
            .iter()
            .filter(|r| matches!(r.kind(), RecordKind::Property { tag, .. } if *tag == wanted));
        let record = records.next()?;
        assert!(records.next().is_none());
        Some(document.payload(record).unwrap())
    };
    let coordinate = |tag| {
        property(tag).map(|raw| f64::from(i32::from_le_bytes(raw.try_into().unwrap())) / 65536.0)
    };
    let page = model.pages().first().unwrap();
    assert_eq!(model.pages().len(), 1);
    assert_eq!(property(0x0811).unwrap(), [1]);
    assert_eq!(Some(page.size().width_pt()), coordinate(0x0812));
    assert_eq!(Some(page.size().height_pt()), coordinate(0x0813));
    assert!(page.texts().is_empty());
    let style = model.drawing_style();
    assert_eq!(style.line_width_pt(), coordinate(0x0807));
    assert_eq!(style.margin_width_pt(), coordinate(0x0808));
    assert_eq!(
        style.bond_spacing_percent(),
        property(0x0804).map(|raw| f64::from(i16::from_le_bytes(raw.try_into().unwrap())) / 10.0)
    );
    assert_eq!(
        model.drawing_settings().bond_length_pt(),
        coordinate(0x0805)
    );
    assert_eq!(
        model.drawing_settings().chain_angle_degrees(),
        coordinate(0x0803)
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

fn check_batch(document: &Document<'_>) {
    let mut properties = document
        .records()
        .iter()
        .enumerate()
        .filter(|(_, record)| matches!(record.kind(), RecordKind::Property { .. }));
    let Some((first_index, first)) = properties.next() else {
        assert_eq!(
            document.replace_properties(&[]).unwrap(),
            document.as_bytes()
        );
        return;
    };
    let old_first = document.payload(first).unwrap();
    assert_eq!(
        document.replace_properties(&[(first, old_first), (first, old_first)]),
        Err(Error::ConflictingEdits)
    );
    let Some((last_index, last)) = properties.next_back() else {
        return;
    };
    let replacement = [0xff, 0x80, 0];
    let second = if document.as_bytes().len() % 2 == 0 {
        document.payload(last).unwrap()
    } else {
        &[1, 0x82, 0, 3, 4, 5][..]
    };
    let edits = [(first, replacement.as_slice()), (last, second)];
    let output = document.replace_properties(&edits).unwrap();
    assert_eq!(
        output,
        document.replace_properties(&[edits[1], edits[0]]).unwrap()
    );
    let parsed = Document::parse(&output).unwrap();
    assert_eq!(parsed.framing(), document.framing());
    assert_eq!(parsed.header(), document.header());
    assert_eq!(parsed.records().len(), document.records().len());
    for (index, (old, new)) in document.records().iter().zip(parsed.records()).enumerate() {
        assert_eq!(old.parent(), new.parent());
        let updated = match index {
            index if index == first_index => Some(replacement.as_slice()),
            index if index == last_index => Some(second),
            _ => None,
        };
        if let Some(value) = updated.filter(|&value| document.payload(old).unwrap() != value) {
            assert_eq!(parsed.payload(new).unwrap(), value);
            assert_eq!(
                document
                    .as_bytes()
                    .get(old.range().start..old.range().start + 2),
                output.get(new.range().start..new.range().start + 2)
            );
        } else {
            assert_eq!(
                document.as_bytes().get(old.range()),
                output.get(new.range())
            );
        }
    }
}

fn check_caption_edit(stored: &bondline_io::StoredDocument, source: &Document<'_>) {
    let page = stored.model().pages().first().unwrap();
    let Some(caption) = page.texts().last() else {
        return;
    };
    let run = if source.as_bytes().len() % 2 == 0 {
        caption.runs().len() - 1
    } else {
        0
    };
    let old = caption.runs().get(run).unwrap().text();
    let unchanged = stored
        .with_cdx_caption_run(page.id(), caption.id(), run, old.to_owned())
        .unwrap();
    assert!(std::sync::Arc::ptr_eq(
        &stored.model_snapshot(),
        &unchanged.model_snapshot()
    ));
    assert_eq!(unchanged.export_cdx().unwrap(), source.as_bytes());
    let replacement = if old == "QX" { "MM" } else { "QX" };
    if let Ok(edited) =
        stored.with_cdx_caption_run(page.id(), caption.id(), run, replacement.to_owned())
    {
        assert!(edited.has_cdx_caption_preview());
        assert!(!edited.is_fully_mapped());
        let bytes = edited.export_cdx().unwrap();
        let parsed = Document::parse(bytes).unwrap();
        assert_eq!(parsed.framing(), source.framing());
        assert_eq!(parsed.records().len(), source.records().len());
        assert_eq!(parsed.header(), source.header());
        let mut changed = 0;
        for (before, after) in source.records().iter().zip(parsed.records()) {
            assert_eq!(before.parent(), after.parent());
            if source.as_bytes().get(before.range()) != bytes.get(after.range()) {
                assert!(matches!(
                    before.kind(),
                    RecordKind::Property { tag: 0x0700, .. }
                ));
                assert!(matches!(
                    after.kind(),
                    RecordKind::Property { tag: 0x0700, .. }
                ));
                changed += 1;
            }
        }
        assert_eq!(changed, 1);
        let reopened = bondline_io::StoredDocument::from_cdx(
            std::sync::Arc::from(bytes),
            bondline_model::PageSize::A4,
        )
        .unwrap();
        assert_eq!(reopened.model(), edited.model());
    }
    assert_eq!(stored.export_cdx().unwrap(), source.as_bytes());
}

fn check_caption_append(stored: &bondline_io::StoredDocument, source: &Document<'_>) {
    use bondline_model::{PageSize, Point2, StyledText, TextId, TextRun, TextStyle};

    let page = stored.model().pages().first().unwrap();
    let Some(id) = page
        .texts()
        .iter()
        .map(|caption| caption.id().get())
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .and_then(TextId::new)
    else {
        return;
    };
    let size = page.size();
    let position = Point2::new(size.width_pt() / 3.0, size.height_pt() / 2.0).unwrap();
    let style = TextStyle::new("Roboto".into(), 12.0, 0, [0.0; 3]).unwrap();
    let run = TextRun::new("Phase β & q".into(), style).unwrap();
    let caption = StyledText::new(id, position, vec![run]).unwrap();
    let candidate = stored.with_cdx_caption(page.id(), caption);
    assert_eq!(stored.export_cdx().unwrap(), source.as_bytes());
    let Ok(candidate) = candidate else {
        return;
    };
    let owner = source
        .records()
        .iter()
        .position(|record| matches!(record.kind(), RecordKind::ObjectStart { tag: 0x8001, .. }))
        .unwrap();
    let boundary = source
        .records()
        .iter()
        .find(|record| *record.kind() == RecordKind::ObjectEnd { start: Some(owner) })
        .unwrap()
        .range()
        .start;
    let bytes = candidate.export_cdx().unwrap();
    let added = bytes.len().checked_sub(source.as_bytes().len()).unwrap();
    assert!(added > 0);
    assert_eq!(bytes[..boundary], source.as_bytes()[..boundary]);
    assert_eq!(bytes[boundary + added..], source.as_bytes()[boundary..]);
    assert_eq!(Document::parse(bytes).unwrap().framing(), source.framing());
    let reopened =
        bondline_io::StoredDocument::from_cdx(std::sync::Arc::from(bytes), PageSize::A4).unwrap();
    assert!(reopened.has_cdx_caption_preview());
    assert!(!reopened.is_fully_mapped());
    assert_eq!(reopened.model(), candidate.model());
    assert_eq!(reopened.model().pages()[0].texts().last().unwrap().id(), id);
}

fn check_mixed_preview(stored: &bondline_io::StoredDocument, source: &Document<'_>) {
    use bondline_model::PageSize;
    use std::collections::BTreeSet;

    assert!(stored.has_display_projection());
    assert!(!stored.is_fully_mapped());
    assert!(!stored.has_cdx_caption_preview());
    assert!(!stored.has_cdx_atom_label_preview());
    assert!(!stored.can_create_caption());
    assert!(stored.to_cdxml().is_err());
    assert_eq!(stored.export_cdx().unwrap(), source.as_bytes());

    let records = source.records();
    let page_index = records
        .iter()
        .position(|r| matches!(r.kind(), RecordKind::ObjectStart { tag: 0x8001, .. }))
        .unwrap();
    let [page] = stored.model().pages() else {
        panic!("mixed page count")
    };
    let mut children = vec![Vec::new(); records.len()];
    let mut named = BTreeSet::new();
    let mut captions = Vec::new();
    let mut fragment_count = 0;
    for (index, record) in records.iter().enumerate() {
        if let Some(parent) = record.parent() {
            children[parent].push(index);
        }
        if record.parent() == Some(page_index)
            && let RecordKind::ObjectStart { tag, id } = *record.kind()
        {
            assert!(id == 0 || named.insert(id), "duplicate Page child ID");
            match tag {
                0x8003 => fragment_count += 1,
                0x8006 => captions.push((id, index)),
                _ => panic!("unmapped Page child"),
            }
        }
    }
    assert!(fragment_count > 0 && !captions.is_empty());
    assert_eq!(page.fragments().len(), fragment_count);
    assert_eq!(page.texts().len(), captions.len());
    let property = |owner: usize, tag: u16| {
        children[owner]
            .iter()
            .find_map(|&index| match records[index].kind() {
                RecordKind::Property { tag: actual, .. } if *actual == tag => Some(&records[index]),
                _ => None,
            })
            .unwrap()
    };
    let payload = |owner, tag| source.payload(property(owner, tag)).unwrap();
    let fixed = |bytes: &[u8]| f64::from(i32::from_le_bytes(bytes.try_into().unwrap())) / 65536.0;
    assert_eq!(page.size().width_pt(), fixed(payload(page_index, 0x0812)));
    assert_eq!(page.size().height_pt(), fixed(payload(page_index, 0x0813)));

    // Strip only complete Page Text subtrees and compare every projected Fragment field
    // to the standalone chemical reader. The original file remains the source of truth.
    let caption_owners: BTreeSet<_> = captions.iter().map(|(_, index)| *index).collect();
    let mut chemistry = Vec::new();
    let mut cursor = 0;
    for end in records {
        if let RecordKind::ObjectEnd { start: Some(start) } = *end.kind()
            && caption_owners.contains(&start)
        {
            chemistry.extend_from_slice(&source.as_bytes()[cursor..records[start].range().start]);
            cursor = end.range().end;
        }
    }
    chemistry.extend_from_slice(&source.as_bytes()[cursor..]);
    let graph = bondline_io::cdx::ElementalGraph::parse(&chemistry).unwrap();
    assert!(
        page.fragments()
            .iter()
            .map(std::sync::Arc::as_ref)
            .eq(graph.fragments())
    );

    // Stable source-ID assignment and signed paint depth come from separate raw fields.
    captions.sort_by_key(|(raw, _)| *raw);
    let mut dense = 0u32;
    let mut by_depth = BTreeMap::new();
    let mut live = BTreeSet::new();
    for (_, owner) in captions {
        let marker = children[owner].iter().copied().find(|i| {
            matches!(
                records[*i].kind(),
                RecordKind::ObjectStart { tag: 0x8011, .. }
            )
        });
        let id = match marker {
            Some(tag) => std::str::from_utf8(payload(tag, 0x0d05))
                .unwrap()
                .strip_prefix("1:")
                .unwrap()
                .parse::<u32>()
                .unwrap(),
            None => {
                dense += 1;
                dense
            }
        };
        assert!(id > 0 && live.insert(id));
        let z = i16::from_le_bytes(payload(owner, 0x000a).try_into().unwrap());
        assert!(z > 0 && by_depth.insert(z, (owner, id)).is_none());
    }
    let resources = source.resources().unwrap();
    for (projected, (owner, id)) in page.texts().iter().zip(by_depth.into_values()) {
        assert_eq!(projected.id().get(), id);
        let p = payload(owner, 0x0200);
        assert_eq!(
            [projected.position().y_pt(), projected.position().x_pt()],
            [fixed(&p[..4]), fixed(&p[4..])]
        );
        let text = source.string(property(owner, 0x0700)).unwrap();
        assert_eq!(projected.runs().len(), text.runs().len());
        let mut offset = 0;
        for (run, raw) in projected.runs().iter().zip(text.runs()) {
            assert_eq!(usize::from(raw.start()), offset);
            let end = offset + run.text().len();
            assert_eq!(run.text().as_bytes(), &text.text_bytes()[offset..end]);
            assert_eq!(
                run.style().size_pt(),
                f64::from(raw.size_twentieths()) / 20.0
            );
            assert_eq!(run.style().face(), raw.face());
            assert_eq!(
                run.style().font_family().as_bytes(),
                resources.font_by_id(raw.font()).unwrap().name_bytes()
            );
            assert_eq!(
                run.style().rgb(),
                resources
                    .color(u32::from(raw.color()))
                    .unwrap()
                    .map(|c| f64::from(c) / 65535.0)
            );
            offset = end;
        }
        assert_eq!(offset, text.text_bytes().len());
    }
    let caption = page.texts().first().unwrap();
    assert!(matches!(
        stored.with_cdx_caption(page.id(), caption.as_ref().clone()),
        Err(bondline_io::DocumentError::UnsupportedCdxEdit)
    ));
    assert!(matches!(
        stored.with_cdx_caption_run(page.id(), caption.id(), 0, "changed".into()),
        Err(bondline_io::DocumentError::UnsupportedCdxEdit)
    ));
    let (fragment, atom) = page
        .fragments()
        .iter()
        .find_map(|f| f.atoms().first().map(|a| (f.id(), a)))
        .unwrap();
    assert!(!stored.can_edit_cdx_atom_label(atom.id));
    assert!(
        stored
            .with_cdx_atom_label(
                page.id(),
                fragment,
                atom.id,
                atom.element,
                atom.explicit_hydrogens.unwrap()
            )
            .is_err()
    );

    let mut detached = stored.clone();
    let changed = PageSize::new(page.size().width_pt() + 1.0, page.size().height_pt()).unwrap();
    detached
        .model_mut()
        .set_page_size(page.id(), changed)
        .unwrap();
    assert!(detached.source_info().unwrap().modified);
    assert!(!detached.has_cdx_mixed_preview());
    assert!(!detached.has_display_projection());
    assert!(detached.to_cdx().is_err());
    assert!(detached.to_cdxml().is_err());
    assert_eq!(stored.to_cdx().unwrap().as_ref(), source.as_bytes());
}

fn check_label_edit(stored: &bondline_io::StoredDocument, original: &[u8]) {
    let page = stored.model().pages().first().unwrap();
    let atoms = page
        .fragments()
        .iter()
        .map(|fragment| fragment.atoms().len())
        .sum::<usize>();
    let (fragment, atom) = page
        .fragments()
        .iter()
        .flat_map(|fragment| fragment.atoms().iter().map(|atom| (fragment.id(), atom)))
        .nth(atoms / 2)
        .unwrap();
    let unchanged = stored.with_cdx_atom_label(
        page.id(),
        fragment,
        atom.id,
        atom.element,
        atom.explicit_hydrogens.unwrap(),
    );
    if !stored.can_edit_cdx_atom_label(atom.id) {
        assert!(unchanged.is_err());
        return;
    }
    let unchanged = unchanged.unwrap();
    assert!(std::sync::Arc::ptr_eq(
        &stored.model_snapshot(),
        &unchanged.model_snapshot()
    ));
    assert_eq!(unchanged.export_cdx().unwrap(), original);
    let element =
        bondline_model::Element::new(if atom.element.get() == 7 { 8 } else { 7 }).unwrap();
    if let Ok(edited) = stored.with_cdx_atom_label(page.id(), fragment, atom.id, element, 1) {
        let bytes = edited.export_cdx().unwrap();
        assert_ne!(bytes, original);
        assert!(edited.has_cdx_atom_label_preview());
        assert!(!edited.is_fully_mapped());
        assert!(edited.to_cdxml().is_err());
        let info = edited.source_info().unwrap();
        assert!(info.modified);
        assert_eq!(info.byte_length, original.len());
        let reopened = bondline_io::StoredDocument::from_cdx(
            std::sync::Arc::from(bytes),
            bondline_model::PageSize::A4,
        )
        .unwrap();
        assert_eq!(reopened.model(), edited.model());
        assert_eq!(reopened.export_cdx().unwrap(), bytes);
    }
    assert_eq!(stored.export_cdx().unwrap(), original);
}

fn check_insertion(document: &Document<'_>) {
    let Some((index, before)) = document
        .records()
        .iter()
        .enumerate()
        .find(|(_, record)| matches!(record.kind(), RecordKind::Property { .. }))
    else {
        return;
    };
    let tag: u16 = 0x7fd2;
    let edits = [(before, document.payload(before).unwrap())];
    let insertions = [PropertyInsertion {
        before,
        tag,
        value: &[],
    }];
    let result = document.edit_properties(&edits, &insertions);
    if document.records().len() == MAX_RECORDS {
        assert_eq!(result, Err(Error::RecordLimit));
    } else if document.records().iter().any(|record| {
        record.parent() == before.parent()
            && matches!(record.kind(), RecordKind::Property { tag: value, .. } if *value == tag)
    }) {
        assert_eq!(result, Err(Error::ConflictingEdits));
    } else if document.as_bytes().len() > MAX_INPUT_BYTES - 4 {
        assert_eq!(result, Err(Error::InputTooLarge));
    } else {
        let output = result.unwrap();
        let offset = before.range().start;
        let mut expected = document.as_bytes()[..offset].to_vec();
        expected.extend_from_slice(&tag.to_le_bytes());
        expected.extend_from_slice(&[0, 0]);
        expected.extend_from_slice(&document.as_bytes()[offset..]);
        assert_eq!(output, expected);
        let parsed = Document::parse(&output).unwrap();
        assert_eq!(parsed.framing(), document.framing());
        assert_eq!(parsed.records().len(), document.records().len() + 1);
        assert_eq!(parsed.records()[index].parent(), before.parent());
    }
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
    check_batch(&document);
    check_insertion(&document);

    let stored_page = document
        .records()
        .iter()
        .any(|record| matches!(record.kind(), RecordKind::ObjectStart { tag: 0x8001, .. }))
        .then(|| {
            bondline_io::StoredDocument::from_cdx(
                std::sync::Arc::from(data),
                bondline_model::PageSize::A4,
            )
            .ok()
        })
        .flatten();
    if let Some(stored) = stored_page
        .as_ref()
        .filter(|stored| stored.has_cdx_mixed_preview())
    {
        check_mixed_preview(stored, &document);
    }
    if let Some(mut stored) = stored_page.filter(|stored| stored.has_cdx_caption_preview()) {
        assert_eq!(stored.export_cdx().unwrap(), data);
        assert!(!stored.is_fully_mapped());
        assert!(!stored.has_caption_projection());
        assert!(stored.has_display_projection());
        assert!(stored.to_cdxml().is_err());
        check_caption_edit(&stored, &document);
        check_caption_append(&stored, &document);
        let baseline = stored.clone();
        let page = stored.model().pages().first().unwrap();
        assert!(page.fragments().is_empty());
        let source_texts = document
            .records()
            .iter()
            .filter(|record| matches!(record.kind(), RecordKind::ObjectStart { tag: 0x8006, .. }))
            .count();
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
        let mut labels = child_label_records(&document);
        let label_resources = (!labels.is_empty()).then(|| document.resources().unwrap());
        let has_labels = !labels.is_empty();
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
                let node = nodes.next().unwrap();
                literal_metadata(*fields.get(node).unwrap(), atom);
                match (labels.remove(&node), atom.authored_label.as_deref()) {
                    (Some(raw), Some(label)) => {
                        check_label(&document, label_resources.as_ref().unwrap(), raw, label);
                    }
                    (None, None) => {}
                    _ => panic!("direct Node child Text lost its owner"),
                }
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
        assert!(labels.is_empty());
        let mut stored = bondline_io::StoredDocument::from_cdx(
            std::sync::Arc::from(data),
            bondline_model::PageSize::A4,
        )
        .unwrap();
        assert_eq!(stored.export_cdx().unwrap(), data);
        assert!(!stored.is_fully_mapped());
        let preview = stored.has_cdx_atom_label_preview();
        let empty_caption_preview = stored.has_cdx_caption_preview();
        if empty_caption_preview {
            assert!(!preview);
            assert!(!has_labels);
            assert!(graph.fragments().is_empty());
            assert!(stored.model().pages()[0].texts().is_empty());
        }
        assert_eq!(
            stored.has_display_projection(),
            preview || empty_caption_preview
        );
        if preview {
            assert!(has_labels);
            check_label_preview(&document, stored.model());
            check_label_edit(&stored, data);
        }
        assert_eq!(stored.model().has_authored_atom_labels(), has_labels);
        assert!(
            stored
                .model()
                .pages()
                .first()
                .unwrap()
                .fragments()
                .iter()
                .map(std::sync::Arc::as_ref)
                .eq(graph.fragments())
        );
        let original = stored.clone();
        let page = stored.model().pages().first().unwrap();
        let changed =
            bondline_model::PageSize::new(page.size().width_pt() + 1.0, page.size().height_pt())
                .unwrap();
        let id = page.id();
        stored.model_mut().set_page_size(id, changed).unwrap();
        assert!(stored.source_info().unwrap().modified);
        assert!(!stored.has_display_projection());
        assert!(stored.export_cdx().is_err());
        assert!(stored.to_cdxml().is_err());
        stored = original;
        assert_eq!(stored.has_cdx_atom_label_preview(), preview);
        assert_eq!(stored.has_cdx_caption_preview(), empty_caption_preview);
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

    check_child_splice(&document, data);

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
