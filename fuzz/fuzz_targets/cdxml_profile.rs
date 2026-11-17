#![no_main]
// SPDX-License-Identifier: MIT OR Apache-2.0
// Bounded IO invariants; independent external literal consumers remain separate.
use bondline_io::{DocumentError, StoredDocument};
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Document, DoublePosition, DrawingSettings, DrawingStyle,
    Element, Fragment, FragmentId, PageSize, Point2, Radical,
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
                        a.show_non_terminal_carbon_labels
                    ),
                    (
                        b.element,
                        b.position,
                        b.isotope,
                        b.charge,
                        b.radical,
                        b.explicit_hydrogens,
                        b.show_terminal_carbon_labels,
                        b.show_non_terminal_carbon_labels
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
        assert!(matches!(
            edited.to_cdxml(),
            Err(DocumentError::UnsupportedEdit)
        ));
    }
    original_unchanged(&base, data, &snapshot);
    assert_eq!(input.as_ref(), data);
    *edited.model_mut() = base.model().clone();
    assert_eq!(edited.export_cdxml().unwrap(), data);
    assert!(!edited.source_info().unwrap().modified);
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

fuzz_target!(|data: &[u8]| {
    if data.len() > INPUT_LIMIT {
        return;
    }
    assert_eq!(ORIGINAL.len(), 464);
    arbitrary_source(data);
    structured(ORIGINAL, data);
    structured(SCOPED, data);
    let styled =
        std::str::from_utf8(ORIGINAL)
            .unwrap()
            .replacen("<CDXML ", "<CDXML LineWidth='.500' ", 1);
    structured(styled.as_bytes(), data);
});
