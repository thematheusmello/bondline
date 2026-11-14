#![no_main]

use bondline_io::smiles::{BondOrder, Hydrogens, SmilesDocument};
use libfuzzer_sys::fuzz_target;
use std::{collections::BTreeSet, ops::Range, sync::Arc};

fn span<'a>(data: &'a [u8], range: &Range<usize>, outer: &Range<usize>) -> &'a [u8] {
    assert!(outer.start <= range.start && range.start < range.end && range.end <= outer.end);
    data.get(range.clone()).expect("accepted source span")
}

fn order(data: &[u8]) -> BondOrder {
    match data {
        b"-" => BondOrder::Single,
        b"=" => BondOrder::Double,
        b"#" => BondOrder::Triple,
        _ => panic!("accepted explicit bond must retain one supported token"),
    }
}

fn label(data: &[u8]) -> u8 {
    assert!(data.len() == 1 || (data.len() == 3 && data.starts_with(b"%")));
    let digits = data.strip_prefix(b"%").unwrap_or(data);
    assert!(digits.iter().all(u8::is_ascii_digit));
    std::str::from_utf8(digits)
        .expect("ASCII ring label")
        .parse()
        .expect("ring label is in 0..99")
}

fuzz_target!(|data: &[u8]| {
    if data.len() > 65_536 {
        return;
    }
    let source: Arc<[u8]> = Arc::from(data);
    let document = match SmilesDocument::parse(source.clone()) {
        Ok(document) => document,
        Err(error) => {
            assert!(error.offset() <= data.len());
            assert!(matches!(
                error.code(),
                "invalid_smiles" | "unsupported_content" | "resource_limit"
            ));
            return;
        }
    };
    assert_eq!(document.as_bytes(), data);
    assert_eq!(document.as_bytes().as_ptr(), source.as_ptr());
    assert!(data.is_ascii());
    let notation = data
        .strip_suffix(b"\r\n")
        .or_else(|| data.strip_suffix(b"\n"))
        .unwrap_or(data);
    assert_eq!(document.notation_range(), 0..notation.len());
    assert!(!notation.is_empty());
    assert!(!notation.iter().any(u8::is_ascii_whitespace));
    assert!((1..=4096).contains(&document.atoms().len()));
    assert!(document.bonds().len() <= 8192);
    assert!((1..=1024).contains(&document.chain_starts()));

    let cloned = document.clone();
    assert_eq!(document, cloned);
    assert!(std::ptr::eq(document.as_bytes(), cloned.as_bytes()));
    assert!(std::ptr::eq(document.atoms(), cloned.atoms()));
    assert!(std::ptr::eq(document.bonds(), cloned.bonds()));

    let outer = document.notation_range();
    let mut previous_end = 0;
    for atom in document.atoms() {
        assert!(previous_end <= atom.range.start);
        let raw = span(data, &atom.range, &outer);
        let _ = span(data, &atom.symbol_range, &atom.range);
        previous_end = atom.range.end;
        if raw.starts_with(b"[") {
            assert!(raw.ends_with(b"]"));
            let Hydrogens::Bracket(count) = atom.hydrogens else {
                panic!("bracket H semantics cannot become implicit");
            };
            assert!(count <= 9);
            if let Some(range) = &atom.hydrogen_range {
                assert_ne!(atom.element.get(), 1);
                let raw_h = span(data, range, &atom.range);
                assert!(raw_h.starts_with(b"H"));
                let expected = if raw_h == b"H" {
                    1
                } else {
                    assert_eq!(raw_h.len(), 2);
                    std::str::from_utf8(raw_h.get(1..).expect("H count"))
                        .expect("ASCII H count")
                        .parse::<u8>()
                        .expect("single numeric H count")
                };
                assert_eq!(count, expected);
            } else {
                assert_eq!(count, 0);
            }
        } else {
            assert_eq!(atom.hydrogens, Hydrogens::Implicit);
            assert_eq!(atom.isotope, None);
            assert_eq!(atom.charge, 0);
            assert_eq!(atom.hydrogen_range, None);
            assert_eq!(atom.charge_range, None);
        }
        if let Some(range) = &atom.isotope_range {
            let raw_isotope = span(data, range, &atom.range);
            assert!(raw_isotope.iter().all(u8::is_ascii_digit));
            assert_eq!(
                atom.isotope,
                Some(
                    std::str::from_utf8(raw_isotope)
                        .expect("ASCII isotope")
                        .parse::<u16>()
                        .expect("bounded numeric isotope")
                )
            );
        } else {
            assert_eq!(atom.isotope, None);
        }
        if let Some(range) = &atom.charge_range {
            let raw_charge = span(data, range, &atom.range);
            assert!(raw_charge.starts_with(b"+") || raw_charge.starts_with(b"-"));
        } else {
            assert_eq!(atom.charge, 0);
        }
        assert!((-15..=15).contains(&atom.charge));
    }

    let mut edges = BTreeSet::new();
    for bond in document.bonds() {
        let [begin, end] = bond.atoms;
        assert!(begin < document.atoms().len() && end < document.atoms().len());
        assert_ne!(begin, end);
        assert!(edges.insert((begin.min(end), begin.max(end))));
        if let Some(range) = &bond.bond_range {
            assert_eq!(order(span(data, range, &outer)), bond.order);
        }
        if let Some(ring) = &bond.ring {
            assert!(ring.label < 100);
            assert!(ring.open_range.end <= ring.close_range.start);
            assert_eq!(label(span(data, &ring.open_range, &outer)), ring.label);
            assert_eq!(label(span(data, &ring.close_range, &outer)), ring.label);
            for range in [&ring.open_bond_range, &ring.close_bond_range]
                .into_iter()
                .flatten()
            {
                assert_eq!(order(span(data, range, &outer)), bond.order);
            }
        }
    }
    assert_eq!(source.as_ref(), data);
});
