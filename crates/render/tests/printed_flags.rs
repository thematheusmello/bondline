//! Existing-API draft, intended under crates/render/tests after Root contract lock.
#[path = "support/boxed_atom.rs"]
mod support;
use bondline_model::{
    Atom, AtomId, Bond, BondId, BondOrder, Document, DoublePosition, Fragment, FragmentId,
    PageSize, Point2,
};
use bondline_render::{
    Cancellation, cpu_scene, graph_page_scene, hidden_carbon_atoms, position_atom_label,
    validate_atom_label,
};
use support::*;

#[test]
fn literal_child_labels_make_all_carbon_flags_display_dormant() -> TestResult {
    for name in ["CH4-12pt-80-100", "Cl-12pt-80-100"] {
        let v = cases()?
            .into_iter()
            .find(|v| v["name"] == name)
            .ok_or("vector")?;
        let original = atom(&v)?;
        let reference = cpu_scene(graph_page_scene(1088, 1088, &page(original.clone())?)?.scene())?;
        for terminal in [None, Some(false), Some(true)] {
            for nonterminal in [None, Some(false), Some(true)] {
                let mut labelled = original.clone();
                labelled.show_terminal_carbon_labels = terminal;
                labelled.show_non_terminal_carbon_labels = nonterminal;
                let before = labelled.clone();
                validate_atom_label(&labelled)?;
                assert_eq!(
                    position_atom_label(&labelled)?,
                    *original.authored_label.as_deref().ok_or("label")?
                );
                assert_eq!(
                    cpu_scene(graph_page_scene(1088, 1088, &page(labelled.clone())?)?.scene())?,
                    reference
                );
                assert_eq!(labelled, before);
            }
        }
    }
    Ok(())
}

fn c2(printed: Atom) -> TestResult<Fragment> {
    let p = printed.position.ok_or("point")?;
    let mut other = Atom::new(
        AtomId::new(97).ok_or("atom")?,
        printed.element,
        Some(Point2::new(p.x_pt() + 96.0, p.y_pt())?),
    );
    other.explicit_hydrogens = Some(3);
    other.show_terminal_carbon_labels = Some(false);
    let edge = Bond {
        id: BondId::new(87).ok_or("bond")?,
        begin: printed.id,
        end: other.id,
        order: BondOrder::Single,
        double_position: DoublePosition::Unspecified,
    };
    Ok(Fragment::new(
        FragmentId::new(31).ok_or("fragment")?,
        vec![printed, other],
        vec![edge],
    )?)
}

fn scene(fragment: Fragment) -> TestResult<bondline_render::Frame> {
    let mut d = Document::default();
    let p = d.pages()[0].id();
    d.set_page_size(p, PageSize::new(512.0, 512.0)?)?;
    d.append_fragments(p, vec![fragment])?;
    Ok(cpu_scene(
        graph_page_scene(1088, 1088, &d.pages()[0])?.scene(),
    )?)
}

#[test]
fn printed_carbon_skips_own_false_c4_but_remains_a_current_c2_neighbour() -> TestResult {
    let v = cases()?
        .into_iter()
        .find(|v| v["name"] == "C-12pt-80-100")
        .ok_or("vector")?;
    let unflagged = atom(&v)?;
    let reference = c2(unflagged.clone())?;
    let mut printed = unflagged;
    printed.show_terminal_carbon_labels = Some(false);
    let flagged = c2(printed)?;
    assert!(
        hidden_carbon_atoms(&flagged, &Cancellation::default())?.is_empty(),
        "printed C does not run its false-flag C4 proof, but the other C still sees the isolated C2 topology and must stay Visible"
    );
    assert_eq!(scene(flagged)?, scene(reference)?);
    Ok(())
}
