#![no_main]

use bondline_io::smiles::{BondOrder, Hydrogens, SmilesDocument};
use libfuzzer_sys::fuzz_target;
use std::{collections::BTreeSet, num::NonZeroU16, ops::Range, sync::Arc};

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

// Unique authored isotope markers recover every output endpoint independently of DFS order.
fn normalized_export(document: &SmilesDocument) {
    let fragment = match document.normalize_explicit_h_fragment() {
        Ok(fragment) => fragment,
        Err(error) => {
            assert!(matches!(
                error.code(),
                "unsupported_content" | "resource_limit"
            ));
            match error {
                bondline_io::smiles::NormalizeError::Unsupported { offset, .. }
                | bondline_io::smiles::NormalizeError::Valence { offset, .. } => {
                    assert!(offset < document.notation_range().end);
                }
                _ => {}
            }
            return;
        }
    };
    assert_eq!(fragment.atoms().len(), document.atoms().len());
    assert_eq!(fragment.bonds().len(), document.bonds().len());
    let mut sums = vec![0_u16; document.atoms().len()];
    for (actual, raw) in fragment.bonds().iter().zip(document.bonds()) {
        let [left, right] = raw
            .atoms
            .map(|at| u32::try_from(at + 1).expect("dense source ID"));
        assert_eq!([actual.begin.get(), actual.end.get()], [left, right]);
        let (expected_order, weight) = match raw.order {
            BondOrder::Single => (bondline_model::BondOrder::Single, 1),
            BondOrder::Double => (bondline_model::BondOrder::Double, 2),
            BondOrder::Triple => (bondline_model::BondOrder::Triple, 3),
        };
        assert_eq!(actual.order, expected_order);
        for endpoint in raw.atoms {
            let sum = sums.get_mut(endpoint).expect("source endpoint");
            *sum = sum.checked_add(weight).expect("bounded bond sum");
        }
    }
    for (index, (actual, raw)) in fragment.atoms().iter().zip(document.atoms()).enumerate() {
        assert_eq!(
            actual.id.get(),
            u32::try_from(index + 1).expect("dense source ID")
        );
        assert_eq!(actual.element, raw.element);
        assert_eq!(raw.charge, 0);
        assert_ne!(raw.isotope, Some(0));
        assert_eq!(actual.isotope.map(NonZeroU16::get), raw.isotope);
        let hydrogens = match raw.hydrogens {
            Hydrogens::Bracket(count) => u16::from(count),
            Hydrogens::Implicit => {
                let target: u16 = match raw.element.get() {
                    6 => 4,
                    7 => 3,
                    8 => 2,
                    9 | 17 | 35 | 53 => 1,
                    _ => panic!("unsupported implicit element cannot normalize"),
                };
                target
                    .checked_sub(*sums.get(index).expect("source sum"))
                    .expect("closed valence")
            }
        };
        assert_eq!(actual.explicit_hydrogens, Some(hydrogens));
    }
    let atoms = fragment
        .atoms()
        .iter()
        .map(|atom| {
            assert_eq!(atom.position, None);
            assert_eq!(atom.charge, 0);
            assert_eq!(atom.radical, bondline_model::Radical::None);
            assert!(atom.explicit_hydrogens.is_some());
            let mut marked = atom.clone();
            marked.isotope =
                NonZeroU16::new(u16::try_from(atom.id.get()).expect("bounded dense ID"));
            marked
        })
        .collect();
    let marked = bondline_model::Fragment::new(fragment.id(), atoms, fragment.bonds().to_vec())
        .expect("only isotope metadata changed");
    let output = match bondline_io::smiles::write_fragment(&marked) {
        Ok(output) => output,
        Err(error) => {
            assert_eq!(error.code(), "resource_limit");
            return;
        }
    };
    assert_eq!(
        output,
        bondline_io::smiles::write_fragment(&marked).expect("deterministic export")
    );
    let parsed = SmilesDocument::parse(Arc::from(output)).expect("generated notation must parse");
    assert_eq!(parsed.atoms().len(), marked.atoms().len());
    assert_eq!(parsed.bonds().len(), marked.bonds().len());
    let mut seen = BTreeSet::new();
    for atom in parsed.atoms() {
        let tag = atom
            .isotope
            .expect("every authored isotope marker retained");
        assert!(tag > 0 && seen.insert(tag));
        let original = marked
            .atoms()
            .get(usize::from(tag - 1))
            .expect("known marker");
        assert_eq!(u32::from(tag), original.id.get());
        assert_eq!(atom.element, original.element);
        assert_eq!(atom.charge, original.charge);
        assert_eq!(
            atom.hydrogens,
            Hydrogens::Bracket(
                u8::try_from(original.explicit_hydrogens.expect("literal H"))
                    .expect("closed profile H fits token")
            )
        );
    }
    let actual = parsed
        .bonds()
        .iter()
        .map(|bond| {
            let [left, right] = bond.atoms.map(|at| {
                u32::from(
                    parsed
                        .atoms()
                        .get(at)
                        .expect("parsed endpoint")
                        .isotope
                        .expect("marker"),
                )
            });
            let order = match bond.order {
                BondOrder::Single => 1,
                BondOrder::Double => 2,
                BondOrder::Triple => 3,
            };
            (left.min(right), left.max(right), order)
        })
        .collect::<BTreeSet<_>>();
    let expected = marked
        .bonds()
        .iter()
        .map(|bond| {
            let left = bond.begin.get();
            let right = bond.end.get();
            let order = match bond.order {
                bondline_model::BondOrder::Single => 1,
                bondline_model::BondOrder::Double => 2,
                bondline_model::BondOrder::Triple => 3,
                _ => panic!("normalization cannot invent other bond orders"),
            };
            (left.min(right), left.max(right), order)
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(actual, expected);
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
    normalized_export(&document);
    assert_eq!(source.as_ref(), data);
});
