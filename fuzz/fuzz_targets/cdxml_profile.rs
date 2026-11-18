#![no_main]
// SPDX-License-Identifier: MIT OR Apache-2.0
// Bounded IO invariants; independent external literal consumers remain separate.
use bondline_io::{DocumentError, StoredDocument};
use bondline_model::{
    Atom, AtomId, AtomLabel, Bond, BondId, BondOrder, Document, DoublePosition, DrawingSettings,
    DrawingStyle, Element, Fragment, FragmentId, LabelAlignment, PageSize, Point2, Radical,
    StyledText, TextId, TextJustification, TextRun, TextStyle,
};
use libfuzzer_sys::fuzz_target;
use std::{borrow::Cow, collections::BTreeMap, num::NonZeroU16, sync::Arc};

const INPUT_LIMIT: usize = 16 * 1024;
const ORIGINAL: &[u8] = br##"<?xml version="1.0" encoding="UTF-8"?>
<CDXML CreationProgram="Bondline 0.1.0">
 <page id='91' Width='612.00' Height="792.000" DrawingSpace='Poster'>
  <fragment id="400">
   <b id='900' E='207' Order='1' B='101' />
   <n Radical='None' id='101' p='100.00 200.000' NumHydrogens='3' Charge='0' Element='6' NodeType='Element'/>
   <n Element="6" NumHydrogens="3" NodeType="Element" p="140 200" id="207" Radical="None" Charge="0"></n>
  </fragment>
 </page>
</CDXML>
"##;
const SCOPED: &[u8] = br##"<CDXML CreationProgram='Bondline 0.1.0'><page id='4294967295' Width='612.00' DrawingSpace='Poster' Height='792.000'><fragment id='7'><!--a--><b id='900' E='207' B='101' Order='1'/><n id='101' NodeType='Element' Element='6' Charge='0' Radical='None' p='72.00 96.000' NumHydrogens='3'/><n id='207' NodeType='Element' Element='6' Charge='0' Radical='None' p='120 96' NumHydrogens='3'/></fragment><fragment id='8'><n id='101' NodeType='Element' Element='8' Charge='0' Radical='None' p='192 192' NumHydrogens='2'/></fragment></page></CDXML>"##;
const CAPTION: &str = "<CDXML CreationProgram='Original caption fuzz'><fonttable><font id='42' name='Roboto'/></fonttable><colortable><color r='1' g='1' b='1'/><color r='0' g='0' b='0'/><color r='.2' g='.4' b='.8'/></colortable><page id='90' DrawingSpace='Poster' Width='256.00' Height='256'><t id='21' p='72.00 96.000' Justification='Left'><s font='42' size='12.00' face='0' color='4'>Phase β &amp; A</s><s font='42' size='18' face='0' color='3'> q</s></t></page></CDXML>";

fn original_unchanged(base: &StoredDocument, source: &[u8], snapshot: &Arc<Document>) {
    let info = base.source_info().unwrap();
    assert_eq!(info.format, "cdxml");
    assert_eq!(info.byte_length, source.len());
    assert!(!info.modified);
    assert_eq!(base.export_cdxml().unwrap(), source);
    let bytes = base.to_cdxml().unwrap();
    assert!(matches!(bytes, Cow::Borrowed(_)));
    assert_eq!(bytes.as_ref(), source);
    assert!(Arc::ptr_eq(snapshot, &base.model_snapshot()));
}

// Rebind by explicit array position, not fresh dense imported object identities.
fn same_semantics(expected: &Document, actual: &Document) {
    assert_eq!(expected.drawing_settings(), actual.drawing_settings());
    assert_eq!(expected.drawing_style(), actual.drawing_style());
    assert_eq!(expected.pages().len(), actual.pages().len());
    for (left, right) in expected.pages().iter().zip(actual.pages()) {
        assert_eq!(left.size(), right.size());
        assert_eq!(left.texts().len(), right.texts().len());
        // Native source IDs, including TextId, rebind by their ordered object position.
        for (a, b) in left.texts().iter().zip(right.texts()) {
            assert_eq!(a.position(), b.position());
            assert_eq!(a.runs(), b.runs());
        }
        assert_eq!(left.fragments().len(), right.fragments().len());
        for (lf, rf) in left.fragments().iter().zip(right.fragments()) {
            assert_eq!(lf.atoms().len(), rf.atoms().len());
            assert_eq!(lf.bonds().len(), rf.bonds().len());
            let li: BTreeMap<_, _> = lf
                .atoms()
                .iter()
                .enumerate()
                .map(|(i, a)| (a.id, i))
                .collect();
            let ri: BTreeMap<_, _> = rf
                .atoms()
                .iter()
                .enumerate()
                .map(|(i, a)| (a.id, i))
                .collect();
            for (a, b) in lf.atoms().iter().zip(rf.atoms()) {
                assert_eq!(
                    (
                        a.element,
                        a.position,
                        a.isotope,
                        a.charge,
                        a.radical,
                        a.explicit_hydrogens,
                        a.show_terminal_carbon_labels,
                        a.show_non_terminal_carbon_labels,
                        a.authored_label.as_deref()
                    ),
                    (
                        b.element,
                        b.position,
                        b.isotope,
                        b.charge,
                        b.radical,
                        b.explicit_hydrogens,
                        b.show_terminal_carbon_labels,
                        b.show_non_terminal_carbon_labels,
                        b.authored_label.as_deref()
                    )
                );
            }
            for (a, b) in lf.bonds().iter().zip(rf.bonds()) {
                assert_eq!((a.order, a.double_position), (b.order, b.double_position));
                assert_eq!(*li.get(&a.begin).unwrap(), *ri.get(&b.begin).unwrap());
                assert_eq!(*li.get(&a.end).unwrap(), *ri.get(&b.end).unwrap());
            }
        }
    }
}

fn changed_complete(edited: &StoredDocument) {
    assert!(edited.is_fully_mapped());
    assert!(edited.source_info().unwrap().modified);
    assert!(matches!(
        edited.export_cdxml(),
        Err(DocumentError::UnsupportedEdit)
    ));
    let snapshot = edited.model_snapshot();
    let encoded = edited.to_cdxml().unwrap();
    assert!(matches!(encoded, Cow::Owned(_)));
    assert_eq!(encoded.as_ref(), edited.to_cdxml().unwrap().as_ref());
    let reopened = StoredDocument::from_cdxml(encoded.into_owned().into(), PageSize::A4).unwrap();
    assert!(reopened.is_fully_mapped());
    same_semantics(edited.model(), reopened.model());
    assert!(Arc::ptr_eq(&snapshot, &edited.model_snapshot()));
    assert!(edited.source_info().unwrap().modified);
    assert!(matches!(
        edited.export_cdxml(),
        Err(DocumentError::UnsupportedEdit)
    ));
}

fn arbitrary_source(data: &[u8]) {
    let input: Arc<[u8]> = data.into();
    let Ok(base) = StoredDocument::from_cdxml(Arc::clone(&input), PageSize::A4) else {
        return;
    };
    let snapshot = base.model_snapshot();
    original_unchanged(&base, data, &snapshot);
    assert_eq!(base.export_cdxml().unwrap().as_ptr(), input.as_ptr());
    let page = base.model().pages()[0].id();
    let target = if base.model().pages()[0].size() == PageSize::new(613.0, 799.0).unwrap() {
        PageSize::new(614.0, 799.0).unwrap()
    } else {
        PageSize::new(613.0, 799.0).unwrap()
    };
    let mut edited = base.clone();
    edited.model_mut().set_page_size(page, target).unwrap();
    assert_ne!(edited.model(), base.model());
    if base.is_fully_mapped() {
        // Two existing scalar replacements cannot reach any resource limit from <=16KiB.
        changed_complete(&edited);
        assert_eq!(edited.model().pages()[0].size(), target);
    } else {
        assert!(!edited.is_fully_mapped());
        let native_label_codec = base.model().has_authored_atom_labels()
            && bondline_io::cdxml::write_document(base.model())
                .ok()
                .as_deref()
                == Some(data);
        if native_label_codec {
            // Exact native data encoding is separate from editing/display authority.
            let encoded = edited.to_cdxml().unwrap();
            assert!(matches!(encoded, Cow::Owned(_)));
            let reopened =
                StoredDocument::from_cdxml(encoded.into_owned().into(), PageSize::A4).unwrap();
            same_semantics(edited.model(), reopened.model());
            assert!(!reopened.is_fully_mapped());
            assert_eq!(
                reopened.has_boxed_atom_labels(),
                base.has_boxed_atom_labels()
            );
            // The old source certificate forbids page drift; fresh native data is requalified.
            assert!(!edited.has_boxed_atom_labels());
            assert_eq!(
                reopened.has_display_projection(),
                reopened.has_boxed_atom_labels() || reopened.has_flagged_atom_labels()
            );
        } else {
            assert!(matches!(
                edited.to_cdxml(),
                Err(DocumentError::UnsupportedEdit)
            ));
        }
    }
    original_unchanged(&base, data, &snapshot);
    assert_eq!(input.as_ref(), data);
    *edited.model_mut() = base.model().clone();
    assert_eq!(edited.export_cdxml().unwrap(), data);
    assert!(!edited.source_info().unwrap().modified);
}

fn refused_caption_drift(base: &StoredDocument, source: &[u8], mut bad: StoredDocument) {
    let snapshot = base.model_snapshot();
    let bad_snapshot = bad.model_snapshot();
    assert!(!bad.has_caption_projection());
    assert!(!bad.is_fully_mapped());
    assert!(matches!(
        bad.to_cdxml(),
        Err(DocumentError::UnsupportedEdit)
    ));
    assert!(Arc::ptr_eq(&bad_snapshot, &bad.model_snapshot()));
    original_unchanged(base, source, &snapshot);
    *bad.model_mut() = base.model().clone();
    assert!(bad.has_caption_projection());
    assert!(!bad.is_fully_mapped());
    assert_eq!(bad.export_cdxml().unwrap(), source);
    assert!(!bad.source_info().unwrap().modified);
}

fn escaped_text(value: &str) -> String {
    let mut output = String::new();
    for character in value.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '\"' => output.push_str("&quot;"),
            '\'' => output.push_str("&apos;"),
            '\t' => output.push_str("&#x9;"),
            '\r' => output.push_str("&#xD;"),
            '\n' => output.push_str("&#xA;"),
            other => output.push(other),
        }
    }
    output
}

fn structured_caption(data: &[u8]) {
    let base = StoredDocument::from_cdxml(CAPTION.as_bytes().into(), PageSize::A4).unwrap();
    let snapshot = base.model_snapshot();
    assert!(base.has_caption_projection());
    assert!(!base.is_fully_mapped());
    let page = &base.model().pages()[0];
    let page_id = page.id();
    let text = page.texts()[0].as_ref();
    assert_eq!(text.runs().len(), 2);
    let index = usize::from(data.first().copied().unwrap_or(0) % 2);
    let value = format!(
        "edit:{}",
        std::str::from_utf8(data).unwrap_or("Phase β & q")
    );
    let mut edited = base.clone();
    match text.with_run_text(index, value.clone()) {
        Ok(replacement) => {
            edited
                .model_mut()
                .replace_text(page_id, replacement)
                .unwrap();
            assert!(edited.has_caption_projection());
            assert!(!edited.is_fully_mapped());
            assert!(edited.source_info().unwrap().modified);
            assert!(matches!(
                edited.export_cdxml(),
                Err(DocumentError::UnsupportedEdit)
            ));
            let inner = ["Phase β &amp; A", " q"][index];
            let expected = CAPTION.replacen(
                &format!(">{inner}</s>"),
                &format!(">{}</s>", escaped_text(&value)),
                1,
            );
            let output = edited.to_cdxml().unwrap().into_owned();
            assert_eq!(output, expected.as_bytes());
            assert_eq!(output, edited.to_cdxml().unwrap().as_ref());
            let reopened = StoredDocument::from_cdxml(output.into(), PageSize::A4).unwrap();
            assert!(reopened.has_caption_projection() || reopened.is_fully_mapped());
            same_semantics(edited.model(), reopened.model());
        }
        Err(error) => {
            assert!(matches!(error, bondline_model::TextError::InvalidCharacter));
            assert!(Arc::ptr_eq(&snapshot, &edited.model_snapshot()));
        }
    }
    original_unchanged(&base, CAPTION.as_bytes(), &snapshot);
    *edited.model_mut() = base.model().clone();
    assert!(edited.has_caption_projection());
    assert!(!edited.is_fully_mapped());
    assert_eq!(edited.export_cdxml().unwrap(), CAPTION.as_bytes());

    let mut bad = base.clone();
    bad.model_mut()
        .replace_text(
            page_id,
            text.with_position(Point2::new(73.0, 96.0).unwrap())
                .unwrap(),
        )
        .unwrap();
    refused_caption_drift(&base, CAPTION.as_bytes(), bad);

    let mut runs = text.runs().to_vec();
    runs[0] = TextRun::new(
        runs[0].text().to_owned(),
        TextStyle::new("Roboto".into(), 13.0, 0, [0.2, 0.4, 0.8]).unwrap(),
    )
    .unwrap();
    let changed = StyledText::new(text.id(), text.position(), runs).unwrap();
    let mut bad = base.clone();
    bad.model_mut().replace_text(page_id, changed).unwrap();
    refused_caption_drift(&base, CAPTION.as_bytes(), bad);

    let mut bad = base.clone();
    let mut changed_ids = Document::default();
    changed_ids.set_page_size(page_id, page.size()).unwrap();
    changed_ids
        .append_texts(
            page_id,
            vec![
                StyledText::new(
                    TextId::new(2).unwrap(),
                    text.position(),
                    text.runs().to_vec(),
                )
                .unwrap(),
            ],
        )
        .unwrap();
    *bad.model_mut() = changed_ids;
    refused_caption_drift(&base, CAPTION.as_bytes(), bad);

    let mut bad = base.clone();
    bad.model_mut()
        .set_page_size(page_id, PageSize::new(257.0, 256.0).unwrap())
        .unwrap();
    refused_caption_drift(&base, CAPTION.as_bytes(), bad);

    let mut bad = base.clone();
    bad.model_mut()
        .set_drawing_style(DrawingStyle::new(Some(1.0)).unwrap());
    refused_caption_drift(&base, CAPTION.as_bytes(), bad);

    let mut bad = base.clone();
    bad.model_mut()
        .set_drawing_settings(DrawingSettings::new(Some(48.0), None).unwrap());
    refused_caption_drift(&base, CAPTION.as_bytes(), bad);

    let mut bad = base.clone();
    bad.model_mut()
        .append_texts(
            page_id,
            vec![
                StyledText::new(
                    TextId::new(2).unwrap(),
                    text.position(),
                    text.runs().to_vec(),
                )
                .unwrap(),
            ],
        )
        .unwrap();
    refused_caption_drift(&base, CAPTION.as_bytes(), bad);

    let mut bad = base.clone();
    let mut atom = Atom::new(
        AtomId::new(1).unwrap(),
        Element::new(6).unwrap(),
        Some(Point2::new(192.0, 192.0).unwrap()),
    );
    atom.explicit_hydrogens = Some(4);
    bad.model_mut()
        .append_fragments(
            page_id,
            vec![Fragment::new(FragmentId::new(1).unwrap(), vec![atom], vec![]).unwrap()],
        )
        .unwrap();
    refused_caption_drift(&base, CAPTION.as_bytes(), bad);
}

fn native_caption_transition(data: &[u8]) {
    let mut native = StoredDocument::default();
    let page_id = native.model().pages()[0].id();
    native
        .model_mut()
        .set_page_size(page_id, PageSize::new(256.0, 256.0).unwrap())
        .unwrap();
    native
        .model_mut()
        .append_texts(
            page_id,
            vec![
                StyledText::new(
                    TextId::new(100 + u32::from(data.first().copied().unwrap_or(0))).unwrap(),
                    Point2::new(72.0, 96.0).unwrap(),
                    vec![
                        TextRun::new(
                            String::new(),
                            TextStyle::new("Roboto".into(), 12.0, 0, [0.0; 3]).unwrap(),
                        )
                        .unwrap(),
                        TextRun::new(
                            " β & q".into(),
                            TextStyle::new("Roboto".into(), 18.0, 0, [0.2, 0.4, 0.8]).unwrap(),
                        )
                        .unwrap(),
                    ],
                )
                .unwrap(),
            ],
        )
        .unwrap();
    let bytes = native.to_cdxml().unwrap().into_owned();
    let native_reopened = StoredDocument::from_cdxml(bytes.clone().into(), PageSize::A4).unwrap();
    assert!(native_reopened.is_fully_mapped());
    same_semantics(native.model(), native_reopened.model());
    let canonical = String::from_utf8(bytes).unwrap();
    assert_eq!(canonical.matches("></s>").count(), 1);
    let source = canonical.replacen("></s>", "/>", 1);
    let mut caption = StoredDocument::from_cdxml(source.as_bytes().into(), PageSize::A4).unwrap();
    assert!(caption.has_caption_projection());
    assert!(!caption.is_fully_mapped());
    let text = caption.model().pages()[0].texts()[0]
        .with_run_text(0, "Phase B".into())
        .unwrap();
    caption.model_mut().replace_text(page_id, text).unwrap();
    let output = caption.to_cdxml().unwrap().into_owned();
    let expected = canonical.replacen("></s>", ">Phase B</s>", 1);
    assert_eq!(output, expected.as_bytes());
    let reopened = StoredDocument::from_cdxml(output.into(), PageSize::A4).unwrap();
    assert!(reopened.is_fully_mapped());
    same_semantics(caption.model(), reopened.model());
}

fn structured(source: &[u8], data: &[u8]) {
    let byte = |i| data.get(i).copied().unwrap_or(0);
    let base = StoredDocument::from_cdxml(source.into(), PageSize::A4).unwrap();
    assert!(base.is_fully_mapped());
    let baseline = base.model_snapshot();
    let mut edited = base.clone();
    let page = edited.model().pages()[0].id();
    let f = edited.model().pages()[0].fragments()[0].as_ref();
    assert_eq!(
        f.atoms().iter().map(|a| a.id.get()).collect::<Vec<_>>(),
        [1, 2]
    );
    assert_eq!((f.bonds()[0].begin.get(), f.bonds()[0].end.get()), (1, 2));
    assert!(
        f.atoms()
            .iter()
            .all(|a| a.element.get() == 6 && a.explicit_hydrogens == Some(3))
    );
    let mut atoms = f.atoms().to_vec();
    let mut bonds = f.bonds().to_vec();
    atoms[0].element = Element::new([1, 6, 7, 8, 17][usize::from(byte(2) % 5)]).unwrap();
    atoms[0].isotope = if byte(0) & 1 == 0 {
        None
    } else {
        NonZeroU16::new(1 + u16::from(byte(4)))
    };
    atoms[0].charge = i8::from_le_bytes([byte(3)]);
    atoms[0].radical = [
        Radical::None,
        Radical::Singlet,
        Radical::Doublet,
        Radical::Triplet,
    ][usize::from(byte(1) % 4)];
    atoms[0].position = if byte(0) & 2 == 0 {
        None
    } else {
        Some(Point2::new(100.25 + f64::from(byte(3)), 200.5).unwrap())
    };
    atoms[0].explicit_hydrogens = if byte(0) & 4 == 0 {
        None
    } else {
        Some(u16::from(byte(5)))
    };
    atoms[0].show_terminal_carbon_labels =
        [None, Some(false), Some(true)][usize::from(byte(6) % 3)];
    atoms[0].show_non_terminal_carbon_labels =
        [None, Some(false), Some(true)][usize::from(byte(7) % 3)];
    bonds[0].order =
        [BondOrder::Single, BondOrder::Double, BondOrder::Triple][usize::from(byte(1) % 3)];
    bonds[0].double_position = [
        DoublePosition::Unspecified,
        DoublePosition::Center,
        DoublePosition::Right,
        DoublePosition::Left,
    ][usize::from(byte(2) % 4)];
    if byte(0) & 8 != 0 {
        let mut added = Atom::new(
            AtomId::new(500_001).unwrap(),
            Element::new(6).unwrap(),
            Some(Point2::new(220.0, 240.0).unwrap()),
        );
        added.explicit_hydrogens = Some(3);
        added.show_terminal_carbon_labels = Some(false);
        added.show_non_terminal_carbon_labels = Some(false);
        atoms.push(added);
        atoms[1].explicit_hydrogens = Some(2);
        bonds.push(Bond {
            id: BondId::new(700_001).unwrap(),
            begin: atoms[1].id,
            end: atoms[2].id,
            order: BondOrder::Single,
            double_position: DoublePosition::Unspecified,
        });
    }
    let replacement = Fragment::new(f.id(), atoms, bonds).unwrap();
    edited
        .model_mut()
        .replace_fragment(page, replacement)
        .unwrap();
    if byte(0) & 16 != 0 {
        let mut a = Atom::new(
            AtomId::new(600_001).unwrap(),
            Element::new(6).unwrap(),
            Some(Point2::new(320.0, 320.0).unwrap()),
        );
        a.explicit_hydrogens = Some(4);
        edited
            .model_mut()
            .append_fragments(
                page,
                vec![Fragment::new(FragmentId::new(800_001).unwrap(), vec![a], vec![]).unwrap()],
            )
            .unwrap();
    }
    edited.model_mut().set_drawing_settings(
        DrawingSettings::new(
            (byte(0) & 32 != 0).then_some(48.0 + f64::from(byte(3)) / 4.0),
            (byte(0) & 64 != 0).then_some(1.0 + f64::from(byte(4) % 179)),
        )
        .unwrap(),
    );
    edited.model_mut().set_drawing_style(
        DrawingStyle::new([None, Some(0.5), Some(2.0), Some(100000.0)][usize::from(byte(7) % 4)])
            .unwrap()
            .with_bond_spacing_percent(
                [None, Some(0.0), Some(18.5), Some(3276.7)][usize::from(byte(8) % 4)],
            )
            .unwrap()
            .with_margin_width(
                [None, Some(0.0), Some(2.0), Some(100000.0)][usize::from(byte(9) % 4)],
            )
            .unwrap(),
    );
    edited
        .model_mut()
        .set_page_size(
            page,
            PageSize::new(640.0 + f64::from(byte(5)), 800.0).unwrap(),
        )
        .unwrap();
    changed_complete(&edited);
    original_unchanged(&base, source, &baseline);

    for reverse in [false, true] {
        let mut bad = base.clone();
        let f = bad.model().pages()[0].fragments()[0].as_ref();
        let mut atoms = f.atoms().to_vec();
        let mut bonds = f.bonds().to_vec();
        if reverse {
            let (begin, end) = (bonds[0].begin, bonds[0].end);
            bonds[0].begin = end;
            bonds[0].end = begin;
        } else {
            atoms[0].id = AtomId::new(900_001).unwrap();
            bonds[0].begin = atoms[0].id;
        }
        let replacement = Fragment::new(f.id(), atoms, bonds).unwrap();
        bad.model_mut().replace_fragment(page, replacement).unwrap();
        let before = bad.model_snapshot();
        assert!(!bad.is_fully_mapped());
        assert!(matches!(
            bad.to_cdxml(),
            Err(DocumentError::UnsupportedEdit)
        ));
        assert!(Arc::ptr_eq(&before, &bad.model_snapshot()));
        original_unchanged(&base, source, &baseline);
    }
}

const ATOM_LABEL: &str = "<CDXML CreationProgram='Original owned label fuzz'><fonttable><font id='42' name='Roboto'/><font id='7' name='Other'/></fonttable><colortable><color r='1' g='1' b='1'/><color r='0' g='0' b='0'/><color r='.2' g='.4' b='.8'/></colortable><page id='90' DrawingSpace='Poster' Width='256' Height='256'><fragment id='51'><n id='73' Element='8' NumHydrogens='1' p='72 96'><t id='21' p='71.5 96.25' Justification='Left' LabelJustification='Auto' LabelAlignment='Right'><s font='42' size='12.5' face='96' color='4'>OH</s><s font='7' size='18' face='0' color='3'>β &amp; q</s></t></n></fragment><t id='84' p='72 144' Justification='Left'><s font='42' size='12' face='0' color='4'>Caption</s></t></page></CDXML>";

fn structured_atom_label(data: &[u8]) {
    let byte = |i| data.get(i).copied().unwrap_or(0);
    let base = StoredDocument::from_cdxml(ATOM_LABEL.as_bytes().into(), PageSize::A4).unwrap();
    let snapshot = base.model_snapshot();
    assert!(base.model().has_authored_atom_labels());
    assert!(!base.is_fully_mapped());
    assert!(!base.has_display_projection());
    assert!(!base.has_caption_projection());
    original_unchanged(&base, ATOM_LABEL.as_bytes(), &snapshot);
    let page = &base.model().pages()[0];
    let fragment = &page.fragments()[0];
    let old = fragment.atoms()[0].authored_label.as_ref().unwrap();
    assert_eq!(old.runs()[0].text(), "OH");
    assert_eq!(old.runs()[1].text(), "β & q");
    assert_eq!(old.position(), Some(Point2::new(71.5, 96.25).unwrap()));
    let value = std::str::from_utf8(data).unwrap_or("OH β & q").to_owned();
    let changed = match old.with_run_text(usize::from(byte(0) % 2), value) {
        Ok(value) => value,
        Err(error) => {
            assert!(matches!(error, bondline_model::TextError::InvalidCharacter));
            original_unchanged(&base, ATOM_LABEL.as_bytes(), &snapshot);
            return;
        }
    };
    let options = [
        None,
        Some(TextJustification::Right),
        Some(TextJustification::Left),
        Some(TextJustification::Center),
        Some(TextJustification::Full),
        Some(TextJustification::Above),
        Some(TextJustification::Below),
        Some(TextJustification::Auto),
        Some(TextJustification::Best),
    ];
    let alignments = [
        None,
        Some(LabelAlignment::Auto),
        Some(LabelAlignment::Left),
        Some(LabelAlignment::Center),
        Some(LabelAlignment::Right),
        Some(LabelAlignment::Above),
        Some(LabelAlignment::Below),
        Some(LabelAlignment::Best),
    ];
    let label = AtomLabel::new(
        (byte(1) & 1 != 0).then_some(Point2::new(71.5 + f64::from(byte(5)), 96.25).unwrap()),
        options[usize::from(byte(2) % 9)],
        options[usize::from(byte(3) % 9)],
        alignments[usize::from(byte(4) % 8)],
        changed.runs().to_vec(),
    )
    .unwrap();
    let mut atoms = fragment.atoms().to_vec();
    atoms[0].authored_label = Some(Arc::new(label));
    let replacement = Fragment::new(fragment.id(), atoms, fragment.bonds().to_vec()).unwrap();
    let mut native = StoredDocument::default();
    *native.model_mut() = base.model().clone();
    native
        .model_mut()
        .replace_fragment(page.id(), replacement)
        .unwrap();
    assert!(!native.is_fully_mapped());
    assert!(!native.has_display_projection());
    assert!(!native.has_caption_projection());
    let encoded = native.to_cdxml().unwrap().into_owned();
    let reopened = StoredDocument::from_cdxml(encoded.clone().into(), PageSize::A4).unwrap();
    same_semantics(native.model(), reopened.model());
    assert!(!reopened.is_fully_mapped());
    assert!(!reopened.has_display_projection());
    assert!(!reopened.has_caption_projection());
    assert_eq!(reopened.to_cdxml().unwrap().as_ref(), encoded);
    let page = &native.model().pages()[0];
    let fragment = &page.fragments()[0];
    let frame = bondline_io::mol::MolTransform::new(1.0, Point2::new(0.0, 0.0).unwrap()).unwrap();
    assert_eq!(
        bondline_io::mol::write_v2000(fragment, "label", frame)
            .unwrap_err()
            .code(),
        "unsupported_content"
    );
    assert_eq!(
        bondline_io::mol::v3000::write_v3000(fragment, "label", frame)
            .unwrap_err()
            .code(),
        "unsupported_content"
    );
    for version in [
        bondline_io::mol::Version::V2000,
        bondline_io::mol::Version::V3000,
    ] {
        assert_eq!(
            native.to_mol_as(version, frame).unwrap_err().code(),
            "unsupported_content"
        );
    }
    let error = bondline_io::display::drawing_page_scene(
        1024,
        1024,
        &native,
        page.id(),
        &Default::default(),
    )
    .err()
    .unwrap();
    assert_eq!(error.code(), "unsupported_content");
    original_unchanged(&base, ATOM_LABEL.as_bytes(), &snapshot);
}

// Original FontTools/Expat vector; no explicit child p or formula face is inferred.
const BOXED_ORIGINAL: &[u8] = br#"<CDXML CreationProgram="Original box-label oracle"><fonttable><font id="42" name="Roboto"/></fonttable><colortable><color r="1" g="1" b="1"/><color r="0" g="0" b="0"/><color r="0.2" g="0.4" b="0.8"/></colortable><page id="90" DrawingSpace="Poster" Width="512" Height="512"><fragment id="51"><n id="73" NodeType="Element" Element="8" Charge="0" Radical="None" NumHydrogens="1" p="84.1220703125 95.734375"><t id="21" BoundingBox="80.69140625 91.3515625 95.796875 100.1171875" Justification="Left" LabelJustification="Left"><s font="42" size="12" face="0" color="4">OH</s></t></n></fragment></page></CDXML>"#;

fn boxed_source_workflow(data: &[u8]) {
    let base = StoredDocument::from_cdxml(BOXED_ORIGINAL.into(), PageSize::A4).unwrap();
    let snapshot = base.model_snapshot();
    assert!(!base.is_fully_mapped());
    assert!(base.has_boxed_atom_labels());
    assert!(base.has_display_projection());
    let page = &base.model().pages()[0];
    let fragment = &page.fragments()[0];
    let changed = data.first().copied().unwrap_or(0) & 1 != 0;
    let mut current = base.clone();
    if changed {
        let mut atoms = fragment.atoms().to_vec();
        let atom = atoms.first_mut().unwrap();
        atom.element = Element::new(7).unwrap();
        atom.explicit_hydrogens = Some(2);
        atom.authored_label = Some(Arc::new(
            atom.authored_label
                .as_ref()
                .unwrap()
                .with_run_text(0, "NH2".into())
                .unwrap()
                .with_bounding_box(Some([80.84375, 91.3515625, 103.26171875, 100.0]))
                .unwrap(),
        ));
        current
            .model_mut()
            .replace_fragment(
                page.id(),
                Fragment::new(fragment.id(), atoms, vec![]).unwrap(),
            )
            .unwrap();
    }
    let encoded = current.to_cdxml().unwrap();
    let expected = if changed {
        std::str::from_utf8(BOXED_ORIGINAL)
            .unwrap()
            .replace("Element=\"8\"", "Element=\"7\"")
            .replace("NumHydrogens=\"1\"", "NumHydrogens=\"2\"")
            .replace(
                "80.69140625 91.3515625 95.796875 100.1171875",
                "80.84375 91.3515625 103.26171875 100",
            )
            .replace(">OH</s>", ">NH2</s>")
            .into_bytes()
    } else {
        BOXED_ORIGINAL.to_vec()
    };
    assert_eq!(encoded.as_ref(), expected);
    let reopened = StoredDocument::from_cdxml(encoded.into_owned().into(), PageSize::A4).unwrap();
    assert_eq!(current.model(), reopened.model());
    assert!(reopened.has_boxed_atom_labels());
    assert!(reopened.has_display_projection());
    assert!(!reopened.is_fully_mapped());
    let mut broken = current.clone();
    let fragment = &current.model().pages()[0].fragments()[0];
    let mut atoms = fragment.atoms().to_vec();
    let atom = atoms.first_mut().unwrap();
    let label = atom.authored_label.as_ref().unwrap();
    let bad = if data.get(1).copied().unwrap_or(0) & 1 != 0 {
        label
            .with_position(Some(Point2::new(80.0, 100.0).unwrap()))
            .unwrap()
    } else {
        let mut bounds = label.bounding_box().unwrap();
        bounds[2] += 0.25;
        label.with_bounding_box(Some(bounds)).unwrap()
    };
    atom.authored_label = Some(Arc::new(bad));
    broken
        .model_mut()
        .replace_fragment(
            page.id(),
            Fragment::new(fragment.id(), atoms, vec![]).unwrap(),
        )
        .unwrap();
    assert!(!broken.has_boxed_atom_labels());
    assert!(!broken.has_display_projection());
    assert!(matches!(
        broken.to_cdxml(),
        Err(DocumentError::UnsupportedEdit)
    ));
    let mut native = StoredDocument::default();
    *native.model_mut() = current.model().clone();
    let native_bytes = native.to_cdxml().unwrap().into_owned();
    let native_reopened = StoredDocument::from_cdxml(native_bytes.into(), PageSize::A4).unwrap();
    assert!(native_reopened.has_boxed_atom_labels());
    same_semantics(native.model(), native_reopened.model());
    arbitrary_source(native_reopened.export_cdxml().unwrap());
    original_unchanged(&base, BOXED_ORIGINAL, &snapshot);
}

fn connected_source_workflow(data: &[u8]) {
    const SOURCE: &[u8] =
        include_bytes!("../../crates/io/tests/fixtures/connected-labels/water.cdxml");
    const EDITED: &[u8] = include_bytes!(
        "../../crates/io/tests/fixtures/connected-labels/ammonia_after_label_edit.cdxml"
    );
    let base = StoredDocument::from_cdxml(SOURCE.into(), PageSize::A4).unwrap();
    let snapshot = base.model_snapshot();
    assert!(base.has_connected_atom_labels());
    assert!(!base.has_boxed_atom_labels());
    assert!(!base.is_fully_mapped());
    let changed = data.first().copied().unwrap_or(0) & 1 != 0;
    let page = &base.model().pages()[0];
    let fragment = &page.fragments()[0];
    let mut current = base.clone();
    if changed {
        let mut atoms = fragment.atoms().to_vec();
        let atom = atoms.first_mut().unwrap();
        atom.element = Element::new(7).unwrap();
        atom.explicit_hydrogens = Some(1);
        atom.authored_label = Some(Arc::new(
            atom.authored_label
                .as_ref()
                .unwrap()
                .with_run_text(0, "NH".into())
                .unwrap()
                .with_bounding_box(Some([
                    156.7216796875,
                    155.734375,
                    171.8330078125,
                    164.265625,
                ]))
                .unwrap(),
        ));
        current
            .model_mut()
            .replace_fragment(
                page.id(),
                Fragment::new(fragment.id(), atoms, fragment.bonds().to_vec()).unwrap(),
            )
            .unwrap();
    }
    let encoded = current.to_cdxml().unwrap();
    assert_eq!(encoded.as_ref(), if changed { EDITED } else { SOURCE });
    let reopened = StoredDocument::from_cdxml(encoded.into_owned().into(), PageSize::A4).unwrap();
    assert_eq!(reopened.model(), current.model());
    assert!(reopened.has_connected_atom_labels());
    assert!(!reopened.has_boxed_atom_labels());
    assert!(reopened.has_display_projection());
    assert!(!reopened.is_fully_mapped());

    let mut bad = current.clone();
    let page = &current.model().pages()[0];
    let fragment = &page.fragments()[0];
    match data.get(1).copied().unwrap_or(0) % 6 {
        0 => {
            let style = bad
                .model()
                .drawing_style()
                .with_margin_width(Some(3.0))
                .unwrap();
            bad.model_mut().set_drawing_style(style);
        }
        1 => {
            let style = bad
                .model()
                .drawing_style()
                .with_line_width(Some(2.0))
                .unwrap();
            bad.model_mut().set_drawing_style(style);
        }
        mode => {
            let mut atoms = fragment.atoms().to_vec();
            let mut bonds = fragment.bonds().to_vec();
            let atom = atoms.first_mut().unwrap();
            if mode == 2 {
                bonds.first_mut().unwrap().order = BondOrder::Double;
            } else if mode == 3 {
                let old = atom.position.unwrap();
                atom.position = Some(Point2::new(old.x_pt() + 0.25, old.y_pt()).unwrap());
            } else {
                let label = atom.authored_label.as_ref().unwrap();
                atom.authored_label = Some(Arc::new(if mode == 4 {
                    label.with_run_text(0, "unknown".into()).unwrap()
                } else {
                    label
                        .with_position(Some(Point2::new(160.0, 160.0).unwrap()))
                        .unwrap()
                }));
            }
            bad.model_mut()
                .replace_fragment(
                    page.id(),
                    Fragment::new(fragment.id(), atoms, bonds).unwrap(),
                )
                .unwrap();
        }
    }
    assert!(!bad.has_connected_atom_labels());
    assert!(!bad.has_boxed_atom_labels());
    assert!(!bad.has_display_projection());
    assert!(matches!(
        bad.to_cdxml(),
        Err(DocumentError::UnsupportedEdit)
    ));
    *bad.model_mut() = base.model().clone();
    assert_eq!(bad.to_cdxml().unwrap().as_ref(), SOURCE);
    original_unchanged(&base, SOURCE, &snapshot);
}

fn translated_label_source(data: &[u8]) {
    let (source, expected, all): (&[u8], &[u8], bool) = match data.first().copied().unwrap_or(0) % 4
    {
        0 => (
            include_bytes!("../../crates/io/tests/fixtures/dependent-labels/boxed.cdxml"),
            include_bytes!(
                "../../crates/io/tests/fixtures/dependent-labels/boxed_sub_p12_5_m7_25.cdxml"
            ),
            false,
        ),
        1 => (
            include_bytes!("../../crates/io/tests/fixtures/dependent-labels/connected.cdxml"),
            include_bytes!(
                "../../crates/io/tests/fixtures/dependent-labels/connected_sub_p12_5_m7_25.cdxml"
            ),
            false,
        ),
        2 => (
            include_bytes!("../../crates/io/tests/fixtures/dependent-labels/connected_nh.cdxml"),
            include_bytes!(
                "../../crates/io/tests/fixtures/dependent-labels/connected_nh_sub_p12_5_m7_25.cdxml"
            ),
            false,
        ),
        _ => (
            include_bytes!("../../crates/io/tests/fixtures/dependent-labels/connected.cdxml"),
            include_bytes!(
                "../../crates/io/tests/fixtures/dependent-labels/connected_all_p24_m12.cdxml"
            ),
            true,
        ),
    };
    let base = StoredDocument::from_cdxml(source.into(), PageSize::A4).unwrap();
    let snapshot = base.model_snapshot();
    let connected = base.has_connected_atom_labels();
    assert!(connected || base.has_boxed_atom_labels());
    assert!(!base.is_fully_mapped());
    let page = &base.model().pages()[0];
    let fragment = &page.fragments()[0];
    let (dx, dy) = if all { (24.0, -12.0) } else { (12.5, -7.25) };
    let mut atoms = fragment.atoms().to_vec();
    for atom in atoms.iter_mut().take(if all { usize::MAX } else { 1 }) {
        let point = atom.position.unwrap();
        atom.position = Some(Point2::new(point.x_pt() + dx, point.y_pt() + dy).unwrap());
        let label = atom.authored_label.as_ref().unwrap();
        let [l, t, r, b] = label.bounding_box().unwrap();
        atom.authored_label = Some(Arc::new(
            label
                .with_bounding_box(Some([l + dx, t + dy, r + dx, b + dy]))
                .unwrap(),
        ));
    }
    let mut moved = base.clone();
    moved
        .model_mut()
        .replace_fragment(
            page.id(),
            Fragment::new(fragment.id(), atoms, fragment.bonds().to_vec()).unwrap(),
        )
        .unwrap();
    assert_eq!(moved.has_connected_atom_labels(), connected);
    assert_eq!(moved.has_boxed_atom_labels(), !connected);
    assert!(moved.has_display_projection());
    assert!(!moved.is_fully_mapped());
    let encoded = moved.to_cdxml().unwrap();
    assert_eq!(encoded.as_ref(), expected);
    let reopened = StoredDocument::from_cdxml(encoded.into_owned().into(), PageSize::A4).unwrap();
    assert_eq!(moved.model(), reopened.model());
    assert_eq!(reopened.has_connected_atom_labels(), connected);
    let mut bad = moved.clone();
    let fragment = &moved.model().pages()[0].fragments()[0];
    let mut atoms = fragment.atoms().to_vec();
    let atom = atoms.first_mut().unwrap();
    let point = atom.position.unwrap();
    atom.position = Some(Point2::new(point.x_pt() + 0.25, point.y_pt()).unwrap());
    bad.model_mut()
        .replace_fragment(
            page.id(),
            Fragment::new(fragment.id(), atoms, fragment.bonds().to_vec()).unwrap(),
        )
        .unwrap();
    assert!(!bad.has_boxed_atom_labels());
    assert!(!bad.has_connected_atom_labels());
    assert!(matches!(
        bad.to_cdxml(),
        Err(DocumentError::UnsupportedEdit)
    ));
    *bad.model_mut() = base.model().clone();
    assert_eq!(bad.to_cdxml().unwrap().as_ref(), source);
    original_unchanged(&base, source, &snapshot);
}

fn mixed_label_source(data: &[u8]) {
    const SOURCE: &[u8] =
        include_bytes!("../../crates/io/tests/fixtures/mixed-labels/amino_alcohol.cdxml");
    let variant = data.first().copied().unwrap_or(0) % 5;
    let source: &[u8] = if variant == 3 {
        include_bytes!("../../crates/io/tests/fixtures/mixed-labels/amino_diamine.cdxml")
    } else {
        SOURCE
    };
    let expected: &[u8] = match variant {
        0 => include_bytes!(
            "../../crates/io/tests/fixtures/mixed-labels/hidden_middle_p12_5_m7_25.cdxml"
        ),
        1 => include_bytes!("../../crates/io/tests/fixtures/mixed-labels/all_p24_m12.cdxml"),
        2 => include_bytes!(
            "../../crates/io/tests/fixtures/mixed-labels/subset_1_3_5_p12_5_m7_25.cdxml"
        ),
        3 => include_bytes!("../../crates/io/tests/fixtures/mixed-labels/edited_all_p24_m12.cdxml"),
        _ => include_bytes!("../../crates/io/tests/fixtures/mixed-labels/amino_diamine.cdxml"),
    };
    let base = StoredDocument::from_cdxml(source.into(), PageSize::A4).unwrap();
    let snapshot = base.model_snapshot();
    assert!(base.has_mixed_atom_labels());
    assert!(!base.has_boxed_atom_labels());
    assert!(!base.has_connected_atom_labels());
    assert!(!base.is_fully_mapped());
    let page = &base.model().pages()[0];
    let fragment = &page.fragments()[0];
    let mut atoms = fragment.atoms().to_vec();
    if variant == 4 {
        let atom = atoms.first_mut().unwrap();
        atom.element = Element::new(7).unwrap();
        atom.explicit_hydrogens = Some(2);
        let old = atom.authored_label.as_ref().unwrap();
        // Independently frozen amino_diamine FontTools box; no render dependency.
        atom.authored_label = Some(Arc::new(
            AtomLabel::new(
                old.position(),
                old.justification(),
                old.label_justification(),
                old.label_alignment(),
                vec![TextRun::new("NH2".into(), old.runs()[0].style().clone()).unwrap()],
            )
            .unwrap()
            .with_bounding_box(Some([
                92.7216796875,
                155.6171875,
                115.1396484375,
                164.265625,
            ]))
            .unwrap(),
        ));
    } else {
        let (dx, dy) = if variant == 1 || variant == 3 {
            (24., -12.)
        } else {
            (12.5, -7.25)
        };
        for atom in &mut atoms {
            let selected = match variant {
                0 => atom.id.get() == 3,
                2 => [1, 3, 5].contains(&atom.id.get()),
                _ => true,
            };
            if selected {
                let p = atom.position.unwrap();
                atom.position = Some(Point2::new(p.x_pt() + dx, p.y_pt() + dy).unwrap());
                if let Some(label) = &atom.authored_label {
                    let [l, t, r, b] = label.bounding_box().unwrap();
                    atom.authored_label = Some(Arc::new(
                        label
                            .with_bounding_box(Some([l + dx, t + dy, r + dx, b + dy]))
                            .unwrap(),
                    ));
                }
            }
        }
    }
    let mut current = base.clone();
    current
        .model_mut()
        .replace_fragment(
            page.id(),
            Fragment::new(fragment.id(), atoms, fragment.bonds().to_vec()).unwrap(),
        )
        .unwrap();
    assert!(current.has_mixed_atom_labels());
    assert!(!current.is_fully_mapped());
    let encoded = current.to_cdxml().unwrap();
    assert_eq!(encoded.as_ref(), expected);
    let reimport = StoredDocument::from_cdxml(encoded.into_owned().into(), PageSize::A4).unwrap();
    assert_eq!(reimport.model(), current.model());
    assert!(reimport.has_mixed_atom_labels());
    let mut bad = base.clone();
    let mut atoms = fragment.atoms().to_vec();
    atoms.get_mut(2).unwrap().position = Some(Point2::new(240., 208.).unwrap());
    bad.model_mut()
        .replace_fragment(
            page.id(),
            Fragment::new(fragment.id(), atoms, fragment.bonds().to_vec()).unwrap(),
        )
        .unwrap();
    assert!(!bad.has_mixed_atom_labels());
    assert!(!bad.has_display_projection());
    assert!(matches!(
        bad.to_cdxml(),
        Err(DocumentError::UnsupportedEdit)
    ));
    *bad.model_mut() = base.model().clone();
    assert_eq!(bad.to_cdxml().unwrap().as_ref(), source);
    original_unchanged(&base, source, &snapshot);
}

fn created_label_source(data: &[u8]) {
    const BASE: &[u8] = include_bytes!("../../crates/io/tests/fixtures/created-labels/base.cdxml");
    let choice = data.first().copied().unwrap_or(0) % 6;
    let (source, expected): (&[u8], &[u8]) = match choice {
        1 => (
            include_bytes!("../../crates/io/tests/fixtures/created-labels/dense_base.cdxml"),
            include_bytes!("../../crates/io/tests/fixtures/created-labels/dense_created_3.cdxml"),
        ),
        2 => (
            include_bytes!("../../crates/io/tests/fixtures/created-labels/paired_base.cdxml"),
            include_bytes!("../../crates/io/tests/fixtures/created-labels/paired_created_3.cdxml"),
        ),
        3 => (
            include_bytes!("../../crates/io/tests/fixtures/created-labels/caption_base.cdxml"),
            include_bytes!("../../crates/io/tests/fixtures/created-labels/caption_created_3.cdxml"),
        ),
        4 => (
            BASE,
            include_bytes!("../../crates/io/tests/fixtures/created-labels/created_all_black.cdxml"),
        ),
        5 => (
            BASE,
            include_bytes!("../../crates/io/tests/fixtures/created-labels/created_3_blue14.cdxml"),
        ),
        _ => (
            BASE,
            include_bytes!("../../crates/io/tests/fixtures/created-labels/created_3_black.cdxml"),
        ),
    };
    let base = StoredDocument::from_cdxml(source.into(), PageSize::A4).unwrap();
    let snapshot = base.model_snapshot();
    assert!(base.has_mixed_atom_labels());
    assert!(!base.has_flagged_atom_labels());
    let expected_model = StoredDocument::from_cdxml(expected.into(), PageSize::A4).unwrap();
    assert!(expected_model.has_flagged_atom_labels());
    let page = &base.model().pages()[0];
    let fragment = &page.fragments()[0];
    let mut atoms = fragment.atoms().to_vec();
    // Frozen independent source supplies only the explicitly inserted label data.
    for (a, b) in atoms
        .iter_mut()
        .zip(expected_model.model().pages()[0].fragments()[0].atoms())
    {
        if a.authored_label.is_none() {
            a.authored_label = b.authored_label.clone();
        }
    }
    let mut current = base.clone();
    current
        .model_mut()
        .replace_fragment(
            page.id(),
            Fragment::new(fragment.id(), atoms, fragment.bonds().to_vec()).unwrap(),
        )
        .unwrap();
    assert!(current.has_flagged_atom_labels());
    assert!(!current.has_boxed_atom_labels());
    assert!(!current.has_connected_atom_labels());
    assert!(!current.has_mixed_atom_labels());
    assert!(!current.is_fully_mapped());
    let raw = current.to_cdxml().unwrap();
    assert_eq!(raw.as_ref(), expected);
    assert_eq!(current.to_cdxml().unwrap().as_ref(), expected);
    let reopened = StoredDocument::from_cdxml(raw.into_owned().into(), PageSize::A4).unwrap();
    assert_eq!(reopened.model(), current.model());
    assert!(reopened.has_flagged_atom_labels());
    let mut bad = current.clone();
    let mut atoms = bad.model().pages()[0].fragments()[0].atoms().to_vec();
    let target = atoms.get_mut(2).unwrap();
    let old = target.authored_label.as_ref().unwrap();
    let style = TextStyle::new(
        "Roboto".into(),
        old.runs()[0].style().size_pt(),
        0,
        [1., 0., 0.],
    )
    .unwrap();
    target.authored_label = Some(Arc::new(
        AtomLabel::new(
            None,
            Some(TextJustification::Left),
            Some(TextJustification::Left),
            None,
            vec![TextRun::new("CH2".into(), style).unwrap()],
        )
        .unwrap()
        .with_bounding_box(old.bounding_box())
        .unwrap(),
    ));
    bad.model_mut()
        .replace_fragment(
            page.id(),
            Fragment::new(fragment.id(), atoms, fragment.bonds().to_vec()).unwrap(),
        )
        .unwrap();
    assert!(!bad.has_flagged_atom_labels());
    assert!(!bad.has_display_projection());
    assert!(matches!(
        bad.to_cdxml(),
        Err(DocumentError::UnsupportedEdit)
    ));
    *bad.model_mut() = base.model().clone();
    assert!(!bad.source_info().unwrap().modified);
    assert!(matches!(bad.to_cdxml().unwrap(), Cow::Borrowed(_)));
    assert_eq!(bad.to_cdxml().unwrap().as_ref(), source);
    original_unchanged(&base, source, &snapshot);
}

fn label_default_metadata_source(data: &[u8]) {
    const BASE: &[u8] =
        include_bytes!("../../crates/io/tests/fixtures/label-metadata/mixed_a.cdxml");
    const CREATED: &[u8] =
        include_bytes!("../../crates/io/tests/fixtures/label-metadata/created_3.cdxml");
    const ALL: &[u8] =
        include_bytes!("../../crates/io/tests/fixtures/label-metadata/all_printed.cdxml");
    let (source, expected): (&[u8], &[u8]) = match data.first().copied().unwrap_or(0) % 5 {
        0 => (
            BASE,
            include_bytes!("../../crates/io/tests/fixtures/label-metadata/edited_endpoint.cdxml"),
        ),
        1 => (BASE, CREATED),
        2 => (
            CREATED,
            include_bytes!("../../crates/io/tests/fixtures/label-metadata/compound.cdxml"),
        ),
        3 => (
            ALL,
            include_bytes!("../../crates/io/tests/fixtures/label-metadata/all_moved.cdxml"),
        ),
        _ => (
            BASE,
            include_bytes!("../../crates/io/tests/fixtures/label-metadata/mixed_moved.cdxml"),
        ),
    };
    let base = StoredDocument::from_cdxml(source.into(), PageSize::A4).unwrap();
    let snapshot = base.model_snapshot();
    assert!(base.has_label_defaults_projection());
    let frozen = StoredDocument::from_cdxml(expected.into(), PageSize::A4).unwrap();
    let page = &base.model().pages()[0];
    let fragment = &page.fragments()[0];
    let mut atoms = fragment.atoms().to_vec();
    for (a, b) in atoms
        .iter_mut()
        .zip(frozen.model().pages()[0].fragments()[0].atoms())
    {
        // Only the declared edit fields come from independently frozen expected source.
        a.position = b.position;
        a.element = b.element;
        a.explicit_hydrogens = b.explicit_hydrogens;
        a.authored_label = b.authored_label.clone();
    }
    let mut current = base.clone();
    current
        .model_mut()
        .replace_fragment(
            page.id(),
            Fragment::new(fragment.id(), atoms, fragment.bonds().to_vec()).unwrap(),
        )
        .unwrap();
    assert!(current.has_label_defaults_projection());
    assert!(!current.is_fully_mapped());
    assert!(!current.has_boxed_atom_labels());
    assert!(!current.has_connected_atom_labels());
    assert!(!current.has_mixed_atom_labels());
    assert!(!current.has_flagged_atom_labels());
    assert_eq!(current.to_cdxml().unwrap().as_ref(), expected);
    let reopened = StoredDocument::from_cdxml(
        current.to_cdxml().unwrap().into_owned().into(),
        PageSize::A4,
    )
    .unwrap();
    assert!(reopened.has_label_defaults_projection());
    assert_eq!(reopened.model(), current.model());
    let mut bad = current.clone();
    bad.model_mut()
        .set_page_size(page.id(), PageSize::new(513., 512.).unwrap())
        .unwrap();
    assert!(!bad.has_label_defaults_projection());
    assert!(!bad.has_display_projection());
    assert!(matches!(
        bad.to_cdxml(),
        Err(DocumentError::UnsupportedEdit)
    ));
    *bad.model_mut() = base.model().clone();
    assert_eq!(bad.to_cdxml().unwrap().as_ref(), source);
    original_unchanged(&base, source, &snapshot);
}

fuzz_target!(|data: &[u8]| {
    if data.len() > INPUT_LIMIT {
        return;
    }
    assert_eq!(ORIGINAL.len(), 464);
    arbitrary_source(data);
    label_default_metadata_source(data);
    created_label_source(data);
    mixed_label_source(data);
    connected_source_workflow(data);
    translated_label_source(data);
    boxed_source_workflow(data);
    structured_atom_label(data);
    structured_caption(data);
    native_caption_transition(data);
    structured(ORIGINAL, data);
    structured(SCOPED, data);
    let styled =
        std::str::from_utf8(ORIGINAL)
            .unwrap()
            .replacen("<CDXML ", "<CDXML LineWidth='.500' ", 1);
    structured(styled.as_bytes(), data);
    let spaced = std::str::from_utf8(ORIGINAL).unwrap().replacen(
        "<CDXML ",
        "<CDXML LineWidth='.500' BondLength='40.00' BondSpacing='18.00' ",
        1,
    );
    structured(spaced.as_bytes(), data);
});
