use bondline_io::StoredDocument;
use bondline_model::{PageSize, Point2, StyledText, TextId, TextRun, TextStyle};
use serde_json::{Value, json};
use std::{
    io::Write,
    process::{Command, Stdio},
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const SOURCE: &[u8] = include_bytes!("fixtures/captions/explicit.cdxml");

fn original() -> TestResult<StoredDocument> {
    Ok(StoredDocument::from_cdxml(SOURCE.into(), PageSize::A4)?)
}

#[test]
fn explicit_foreign_caption_is_owned_model_data_without_chemical_edit_permission() -> TestResult {
    let stored = original()?;
    let value = serde_json::to_value(stored.model())?;
    assert_eq!(
        value["pages"][0]["texts"],
        json!([{"id":1,"position":{"x_pt":72.0,"y_pt":96.0},
        "runs":[{"text":"Phase A","style":{"font_family":"Roboto","size_pt":12.0,"face":0,"rgb":[0.2,0.4,0.8]}}]}])
    );
    assert!(!stored.is_fully_mapped());
    assert_eq!(stored.to_cdxml()?.as_ref(), SOURCE);
    Ok(())
}

fn inspect(raw: &[u8]) -> TestResult<Value> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child=Command::new(python).args(["-X","utf8","-B","-c",r#"
import json,sys,xml.parsers.expat as expat
from decimal import Decimal
raw=sys.stdin.buffer.read(67108865);assert 0<len(raw)<=67108864
fonts={};colors=[];texts=[];stack=[];current=None;run=None
def start(name,attrs):
 global current,run
 assert name in ('CDXML','fonttable','font','colortable','color','page','t','s')
 stack.append(name)
 if name=='font':
  i=int(attrs['id']);assert 0<=i<=65535 and i not in fonts;fonts[i]=attrs['name']
 if name=='color':
  rgb=[Decimal(attrs[c]) for c in ('r','g','b')]
  assert all(c.is_finite() and 0<=c<=1 for c in rgb);colors.append(rgb)
 if name=='page':assert Decimal(attrs['Width'])==Decimal(256) and Decimal(attrs['Height'])==Decimal(256)
 if name=='t':
  assert attrs['Justification']=='Left';current={'p':[str(Decimal(x)) for x in attrs['p'].split()],'runs':[]};texts.append(current)
 if name=='s':
  size=Decimal(attrs['size']);assert size.is_finite() and size>0
  color=int(attrs['color']);rgb=[Decimal(0)]*3 if color==0 else [Decimal(1)]*3 if color==1 else colors[color-2]
  run={'text':'','font':fonts[int(attrs['font'])],'size':str(size),'face':int(attrs['face']),'rgb':list(map(str,rgb))};current['runs'].append(run)
def end(name):
 global run
 assert stack.pop()==name
 if name=='s':run=None
def chars(text):
 if run is not None:run['text']+=text
 else:assert not text.strip()
p=expat.ParserCreate(namespace_separator='|');p.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
p.StartElementHandler=start;p.EndElementHandler=end;p.CharacterDataHandler=chars;p.Parse(raw,True)
print(json.dumps(texts,ensure_ascii=False))
"#]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    child.stdin.take().ok_or("Expat stdin")?.write_all(raw)?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(format!(
            "independent caption reader: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}

#[test]
fn native_encoding_cannot_silently_drop_owned_caption_data() -> TestResult {
    let expected = json!([{"p":["72","96"],"runs":[{"text":"Phase A","font":"Roboto","size":"12","face":0,"rgb":["0.2","0.4","0.8"]}]}]);
    assert_eq!(inspect(SOURCE)?, expected);
    let bad = String::from_utf8(SOURCE.to_vec())?.replace("<s font=\"42\"", "<s font=\"43\"");
    assert!(
        inspect(bad.as_bytes()).is_err(),
        "qualify unresolved-font refusal independently"
    );
    let mut stored = StoredDocument::default();
    let id = stored.model().pages()[0].id();
    stored
        .model_mut()
        .set_page_size(id, PageSize::new(256.0, 256.0)?)?;
    stored.model_mut().append_texts(
        id,
        vec![StyledText::new(
            TextId::new(7).ok_or("text id")?,
            Point2::new(72.0, 96.0)?,
            vec![TextRun::new(
                "Phase A".into(),
                TextStyle::new("Roboto".into(), 12.0, 0, [0.2, 0.4, 0.8])?,
            )?],
        )?],
    )?;
    assert_eq!(inspect(&stored.to_cdxml()?)?, expected);
    Ok(())
}

// Independently declared from the Caption -> Canonical classification branches.
// Source-only reviewer authored this file without running Cargo or changing production.
#[test]
fn self_closed_empty_caption_run_may_become_exact_native_xml_after_a_text_edit()
-> Result<(), Box<dyn std::error::Error>> {
    use bondline_io::StoredDocument;
    use bondline_model::{PageSize, Point2, StyledText, TextId, TextRun, TextStyle};

    let size = PageSize::new(256.0, 256.0)?;
    let mut native = StoredDocument::default();
    let page_id = native.model().pages().first().ok_or("native page")?.id();
    native.model_mut().set_page_size(page_id, size)?;
    native.model_mut().append_texts(
        page_id,
        vec![StyledText::new(
            TextId::new(1).ok_or("text id")?,
            Point2::new(72.0, 96.0)?,
            vec![TextRun::new(
                String::new(),
                TextStyle::new("Roboto".into(), 12.0, 0, [0.0; 3])?,
            )?],
        )?],
    )?;
    let canonical = String::from_utf8(native.to_cdxml()?.into_owned())?;
    assert_eq!(
        canonical.matches("></s>").count(),
        1,
        "one explicit empty run"
    );
    let source = canonical.replacen("></s>", "/>", 1);
    assert_ne!(source, canonical, "qualification must change source syntax");
    let mut stored = StoredDocument::from_cdxml(source.into_bytes().into(), size)?;
    assert_eq!(
        stored.model(),
        native.model(),
        "empty-element spelling preserves semantics"
    );
    assert!(stored.has_caption_projection());
    assert!(
        !stored.is_fully_mapped(),
        "the lexical source has operation-only caption authority"
    );

    let replacement = stored
        .model()
        .pages()
        .first()
        .ok_or("mapped page")?
        .texts()
        .first()
        .ok_or("mapped text")?
        .with_run_text(0, "Phase B".into())?;
    stored
        .model_mut()
        .replace_text(page_id, replacement.clone())?;
    native.model_mut().replace_text(page_id, replacement)?;
    let expected = native.to_cdxml()?.into_owned();
    let encoded = stored.to_cdxml()?.into_owned();
    assert_eq!(
        encoded, expected,
        "SetText legitimately emits exact native canonical XML"
    );
    let reopened = StoredDocument::from_cdxml(encoded.into(), size)?;
    assert!(
        reopened.is_fully_mapped(),
        "classification may upgrade after a fully proven splice"
    );
    assert_eq!(
        reopened.model(),
        stored.model(),
        "the upgrade preserves the complete model"
    );
    Ok(())
}
