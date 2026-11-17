use bondline_io::StoredDocument;
use bondline_model::{PageId, PageSize};
use std::{
    borrow::Cow,
    io::Write,
    process::{Command, Stdio},
};

const ORIGINAL: &[u8] = include_bytes!("fixtures/bondline-v0.cdxml");

#[test]
fn pinned_original_profile_is_complete_and_its_revision_preserves_all_authored_fields()
-> Result<(), Box<dyn std::error::Error>> {
    let mut stored = StoredDocument::from_cdxml(ORIGINAL.into(), PageSize::A4)?;
    assert!(stored.is_fully_mapped());
    assert!(matches!(stored.to_cdxml()?, Cow::Borrowed(_)));
    let before = stored.clone();
    stored
        .model_mut()
        .set_page_size(PageId::new(1).ok_or("page")?, PageSize::new(400.0, 600.0)?)?;
    assert!(
        stored.export_cdxml().is_err(),
        "borrowed-only export cannot return stale bytes"
    );
    let edited = stored.to_cdxml()?;
    assert!(matches!(edited, Cow::Owned(_)));
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child=Command::new(python).args(["-c",r#"
import sys,hashlib,xml.parsers.expat as expat
from pathlib import Path
assert hashlib.sha256(Path(sys.argv[1]).read_bytes()).hexdigest()=='ead553df9d8c6bf7a371e7e36909a9c8895f34b9231f20c3c3a796680d64bbc6'
nodes=[]
p=expat.ParserCreate(namespace_separator='|')
p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
p.StartElementHandler=lambda name,attrs:nodes.append((name,attrs))
p.Parse(sys.stdin.buffer.read(),True)
assert [name for name,_ in nodes]==['CDXML','page','fragment','n','n','b']
assert nodes[0][1]=={'CreationProgram':'Bondline 0.1.0'}
assert nodes[1][1]=={'id':'1','DrawingSpace':'Poster','Width':'400','Height':'600'}
assert nodes[3][1]=={'id':'3','NodeType':'Element','Element':'6','Charge':'0','Radical':'None','p':'100 200','NumHydrogens':'3'}
assert nodes[4][1]=={'id':'4','NodeType':'Element','Element':'6','Charge':'0','Radical':'None','p':'140 200','NumHydrogens':'3'}
assert nodes[5][1]=={'id':'5','B':'3','E':'4','Order':'1'}
"#]).arg(concat!(env!("CARGO_MANIFEST_DIR"),"/tests/fixtures/bondline-v0.cdxml")).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    child.stdin.take().ok_or("stdin")?.write_all(&edited)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(before.to_cdxml()?.as_ref(), ORIGINAL);
    let reloaded = StoredDocument::from_cdxml(edited.into_owned().into(), PageSize::A4)?;
    assert!(reloaded.is_fully_mapped());
    assert_eq!(reloaded.model(), stored.model());
    Ok(())
}

#[test]
fn creator_name_does_not_authorize_dropping_unknown_source_or_unmodeled_styles()
-> Result<(), Box<dyn std::error::Error>> {
    let original = std::str::from_utf8(ORIGINAL)?;
    for source in [
        original.replacen("<CDXML ", "<CDXML Future='opaque' ", 1),
        original.replacen("<CDXML ", "<CDXML LineWidth='9' ", 1),
        original.replacen("</page>", "<t><s>caption</s></t></page>", 1),
        original.replacen("</CDXML>", "<!-- retain --><?future opaque?></CDXML>", 1),
        original.replacen("Bondline 0.1.0", "Bondline 999.0.0", 1),
    ] {
        let mut stored = StoredDocument::from_cdxml(source.as_bytes().into(), PageSize::A4)?;
        assert!(!stored.is_fully_mapped());
        assert_eq!(stored.to_cdxml()?.as_ref(), source.as_bytes());
        stored
            .model_mut()
            .set_page_size(PageId::new(1).ok_or("page")?, PageSize::new(400.0, 600.0)?)?;
        assert!(matches!(
            stored.to_cdxml(),
            Err(bondline_io::DocumentError::UnsupportedEdit)
        ));
    }
    Ok(())
}

#[test]
fn quote_only_native_equivalent_now_edits_without_rewriting_its_original_xml()
-> Result<(), Box<dyn std::error::Error>> {
    let source = std::str::from_utf8(ORIGINAL)?.replace('"', "'");
    let mut stored = StoredDocument::from_cdxml(source.as_bytes().into(), PageSize::A4)?;
    assert!(stored.is_fully_mapped());
    assert_eq!(stored.to_cdxml()?.as_ref(), source.as_bytes());
    let original = stored.clone();
    stored
        .model_mut()
        .set_page_size(PageId::new(1).ok_or("page")?, PageSize::new(400.0, 600.0)?)?;
    let expected = source
        .replace("Width='612'", "Width='400'")
        .replace("Height='792'", "Height='600'");
    assert_ne!(expected, source);
    assert_eq!(stored.to_cdxml()?.as_ref(), expected.as_bytes());
    assert_eq!(original.to_cdxml()?.as_ref(), source.as_bytes());
    Ok(())
}
