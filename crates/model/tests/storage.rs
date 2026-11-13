use bondline_model::{Atom, AtomId, Bond, Document, Element, Fragment, FragmentId, PageId};
use std::{mem::size_of, sync::Arc};

fn document(
    atom_capacity: usize,
    bond_capacity: usize,
) -> Result<(Document, usize), Box<dyn std::error::Error>> {
    let mut atoms = Vec::with_capacity(atom_capacity);
    atoms.push(Atom::new(
        AtomId::new(1).ok_or("atom")?,
        Element::try_from(6)?,
        None,
    ));
    let bonds: Vec<Bond> = Vec::with_capacity(bond_capacity);
    let allocated = atoms.capacity() * size_of::<Atom>() + bonds.capacity() * size_of::<Bond>();
    let fragment = Fragment::new(FragmentId::new(1).ok_or("fragment")?, atoms, bonds)?;
    let mut document = Document::default();
    document.append_fragments(PageId::new(1).ok_or("page")?, vec![fragment])?;
    Ok((document, allocated))
}

#[test]
fn graph_accounting_includes_reserved_atom_and_bond_storage()
-> Result<(), Box<dyn std::error::Error>> {
    let (small, small_allocation) = document(1, 0)?;
    let (reserved, large_allocation) = document(16384, 8192)?;
    assert_eq!(
        small, reserved,
        "same authored data, different reserved storage"
    );
    assert_eq!(
        Document::retained_bytes([&reserved]) - Document::retained_bytes([&small]),
        large_allocation - small_allocation,
    );
    Ok(())
}

#[test]
fn repeated_documents_and_shared_fragment_payloads_are_counted_once()
-> Result<(), Box<dyn std::error::Error>> {
    let (document, _) = document(16384, 8192)?;
    let once = Document::retained_bytes([&document]);
    assert_eq!(Document::retained_bytes([&document, &document]), once);
    let clone = document.clone();
    assert!(Arc::ptr_eq(
        document
            .pages()
            .first()
            .ok_or("page")?
            .fragments()
            .first()
            .ok_or("fragment")?,
        clone
            .pages()
            .first()
            .ok_or("page")?
            .fragments()
            .first()
            .ok_or("fragment")?,
    ));
    let empty = Document::default();
    assert_eq!(
        Document::retained_bytes([&document, &clone]) - once,
        Document::retained_bytes([&empty]) + size_of::<Arc<Fragment>>(),
        "a clone adds document/page storage and one fragment handle, not a second payload",
    );
    Ok(())
}
