use bondline_io::cdxml::{ElementalGraph, MappingError};

#[test]
fn label_only_nodes_are_not_silently_reinterpreted_as_carbon() {
    // CDXML-NODE permits text-only writers; their missing metadata cannot be guessed from a label.
    let source = b"<CDXML><page><fragment><n id='1'><t><s>N</s></t></n></fragment></page></CDXML>";
    assert!(
        matches!(
            ElementalGraph::parse(source),
            Err(MappingError::UnsupportedLabelOnlyAtom)
        ),
        "label-only nitrogen was mapped as default carbon"
    );
}
