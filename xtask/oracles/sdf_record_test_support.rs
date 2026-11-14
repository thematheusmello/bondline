//! Original literal expectations for the unchanged690/777-byte SDF vectors.

use serde_json::{Value, json};
use std::{
    io::Write as _,
    process::{Command, Stdio},
};

fn field(
    index: usize,
    header: &str,
    name: Option<&str>,
    dt: Option<&str>,
    bytes: usize,
    lines: &[&str],
) -> Value {
    json!({"index":index,"header":header,"name":name,"dt_number":dt,
        "value_byte_length":bytes,"value_line_count":lines.len(),
        "value_lines":lines,"omitted_reason":null})
}

pub fn expected_fields(mixed: bool, record_id: u32) -> Value {
    match (mixed, record_id) {
        (false, 1) => json!([
            field(0, "> <ID>", Some("ID"), None, 6, &["first"]),
            field(1, "> <EMPTY>", Some("EMPTY"), None, 0, &[]),
            field(
                2,
                "> 7(BL-001)<NOTES>DT12",
                Some("NOTES"),
                Some("12"),
                11,
                &["alpha", "beta"]
            ),
            field(3, "> DT0009", None, Some("0009"), 9, &["numbered"]),
            field(4, "> <DUP>", Some("DUP"), None, 4, &["one"]),
            field(5, "> <DUP>", Some("DUP"), None, 4, &["two"]),
            field(6, "> <SPACES>", Some("SPACES"), None, 4, &["   "])
        ]),
        (true, 1) => json!([
            field(0, "> <EMPTY>", Some("EMPTY"), None, 0, &[]),
            field(1, "> <DUP>", Some("DUP"), None, 6, &["first"]),
            field(2, "> <DUP>", Some("DUP"), None, 7, &["second"]),
            field(
                3,
                "> 17(REF-42)<MASS>DT0009",
                Some("MASS"),
                Some("0009"),
                15,
                &["carbon", "isotope"]
            ),
            field(4, "> DT0012", None, Some("0012"), 4, &["raw"]),
            field(5, "> <SPACES>", Some("SPACES"), None, 4, &["   "])
        ]),
        (true, 2) => json!([field(
            0,
            "> <ORDINARY>",
            Some("ORDINARY"),
            None,
            14,
            &["second record"]
        )]),
        _ => json!([]),
    }
}

/// Independent stdlib compact-JSON sizing, including escaping and vector overhead.
pub fn assert_json_budget(
    page: &Value,
    next_field: Option<Value>,
) -> Result<(), Box<dyn std::error::Error>> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child=Command::new(python).args(["-B","-c",r#"
import json,sys
request=json.load(sys.stdin)
page=request['page']
size=lambda value:len(json.dumps(value,ensure_ascii=False,separators=(',',':')).encode('utf-8'))
assert size(page)<=1048576
for field in page['fields']:
    if field['value_lines'] is not None:
        assert field['value_byte_length']<=65536 and size(field['value_lines'])<=65536
        assert field['omitted_reason'] is None
    elif field['omitted_reason']=='value_byte_limit':
        assert field['value_byte_length']>65536
    else:
        assert field['omitted_reason']=='value_json_limit' and field['value_byte_length']<=65536
if page['page_limited']:
    field=request['next_field']
    assert field is not None and field['index']==page['next_field_offset']
    candidate=dict(page,fields=page['fields']+[field],page_limited=False)
    candidate['next_field_offset']=field['index']+1 if field['index']+1<page['field_count'] else None
    assert size(candidate)>1048576,'page did not disclose the maximal fitting prefix'
"#]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    serde_json::to_writer(
        child.stdin.as_mut().ok_or("JSON oracle stdin")?,
        &json!({"page":page,"next_field":next_field}),
    )?;
    child.stdin.take().ok_or("JSON oracle stdin")?.flush()?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "independent query JSON budget: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}
