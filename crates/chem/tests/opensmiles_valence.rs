use bondline_chem::opensmiles_implicit_hydrogens;
use bondline_model::Element;

#[test]
fn normal_valences_count_toward_the_next_allowed_state_and_stop_above_the_last()
-> Result<(), Box<dyn std::error::Error>> {
    for (z, counts) in [
        (5, &[3, 2, 1, 0, 0, 0, 0, 0][..]),
        (6, &[4, 3, 2, 1, 0, 0, 0, 0]),
        (7, &[3, 2, 1, 0, 1, 0, 0, 0]),
        (8, &[2, 1, 0, 0, 0, 0, 0, 0]),
        (15, &[3, 2, 1, 0, 1, 0, 0, 0]),
        (16, &[2, 1, 0, 1, 0, 1, 0, 0]),
        (9, &[1, 0, 0, 0, 0, 0, 0, 0]),
        (17, &[1, 0, 0, 0, 0, 0, 0, 0]),
        (35, &[1, 0, 0, 0, 0, 0, 0, 0]),
        (53, &[1, 0, 0, 0, 0, 0, 0, 0]),
    ] {
        let element = Element::try_from(z)?;
        for (occupied, expected) in counts.iter().enumerate() {
            assert_eq!(
                opensmiles_implicit_hydrogens(element, u16::try_from(occupied)?),
                Some(*expected)
            );
        }
        assert_eq!(opensmiles_implicit_hydrogens(element, u16::MAX), Some(0));
    }
    for z in [1, 11, 14, 26] {
        assert_eq!(
            opensmiles_implicit_hydrogens(Element::try_from(z)?, 0),
            None
        );
    }
    Ok(())
}
