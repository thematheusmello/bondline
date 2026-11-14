use bondline_model::Element;

#[test]
fn iupac_catalog_symbols_are_exact_case_sensitive_facts() -> Result<(), Box<dyn std::error::Error>>
{
    // Atomic-number/symbol facts, IUPAC periodic table dated4May2022.
    for (number, symbol) in [
        (1, "H"),
        (2, "He"),
        (6, "C"),
        (7, "N"),
        (8, "O"),
        (9, "F"),
        (14, "Si"),
        (17, "Cl"),
        (35, "Br"),
        (53, "I"),
        (113, "Nh"),
        (115, "Mc"),
        (117, "Ts"),
        (118, "Og"),
    ] {
        let element = Element::new(number).ok_or("element")?;
        assert_eq!(element.symbol(), Some(symbol));
        assert_eq!(Element::from_symbol(symbol), Some(element));
    }
    for symbol in ["", "cl", "CL", "D", "T", "Xx", " C", "C\n", "13C"] {
        assert_eq!(Element::from_symbol(symbol), None, "{symbol:?}");
    }
    for number in 1..=118 {
        assert!(Element::new(number).ok_or("element")?.symbol().is_some());
    }
    assert!(Element::new(0).is_none());
    assert!(Element::new(119).is_none());
    Ok(())
}
