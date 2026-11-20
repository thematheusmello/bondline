//! Existing public JSON APIs only: runtime contract RED is separate from new-type compilation.
use bondline_commands::Command;
use serde_json::{Value, json};
use std::collections::BTreeSet;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn common() -> Value {
    json!({"expected_revision":7,"page_id":1,"hydrogen_policy":"rectify_neutral_carbon",
        "placement":{"mode":"free","begin_x_pt":120.0,"begin_y_pt":168.0,
        "end_x_pt":120.0,"end_y_pt":120.0,"side":"positive"}})
}
fn named(kind: &str) -> Value {
    let mut value = common();
    value["kind"] = json!(kind);
    value
}
fn rejected(value: Value) -> TestResult {
    let error = Command::parse("ring.create", value)
        .err()
        .ok_or("invalid ring selector/phase accepted")?;
    assert_eq!(error.info().code, "invalid_params");
    Ok(())
}

#[test]
fn named_and_sized_requests_are_closed_and_phase_is_optional_but_never_null() -> TestResult {
    let mut sized = common();
    sized["size"] = json!(6);
    for params in [sized.clone(), named("benzene"), named("cyclopentadiene")] {
        let command = Command::parse("ring.create", params.clone())?;
        assert_eq!(command.name(), "ring.create");
        assert_eq!(
            serde_json::to_value(command)?,
            json!({"command":"ring.create","params":params})
        );
    }
    for kind in ["benzene", "cyclopentadiene"] {
        for phase in ["double_first", "single_first"] {
            let mut params = named(kind);
            params["phase"] = json!(phase);
            let command = Command::parse("ring.create", params.clone())?;
            assert_eq!(
                serde_json::to_value(command)?,
                json!({"command":"ring.create","params":params})
            );
        }
        for phase in [
            Value::Null,
            json!(""),
            json!("DoubleFirst"),
            json!("automatic"),
            json!(0),
            json!(false),
            json!([]),
            json!({}),
            json!({"double_first":null}),
            json!({"single_first":null}),
        ] {
            let mut params = named(kind);
            params["phase"] = phase;
            rejected(params)?;
        }
        let mut both = named(kind);
        both["size"] = json!(6);
        rejected(both)?;
        let mut unknown = named(kind);
        unknown["extra"] = json!(true);
        rejected(unknown)?;
        for field in [
            "expected_revision",
            "page_id",
            "hydrogen_policy",
            "placement",
        ] {
            let mut missing = named(kind);
            missing.as_object_mut().ok_or("params")?.remove(field);
            rejected(missing)?;
        }
    }
    rejected(common())?;
    for kind in [
        Value::Null,
        json!("cyclohexane"),
        json!("Benzene"),
        json!(6),
        json!({"benzene":null}),
        json!({"cyclopentadiene":null}),
    ] {
        let mut params = common();
        params["kind"] = kind;
        rejected(params)?;
    }
    for phase in [Value::Null, json!("double_first"), json!("single_first")] {
        let mut params = sized.clone();
        params["phase"] = phase;
        rejected(params)?;
    }
    Ok(())
}

fn node<'a>(root: &'a Value, value: &'a Value) -> TestResult<&'a Value> {
    match value.get("$ref").and_then(Value::as_str) {
        None => Ok(value),
        Some(reference) => root
            .pointer(
                reference
                    .strip_prefix('#')
                    .ok_or("nonlocal schema reference")?,
            )
            .ok_or_else(|| "missing schema node".into()),
    }
}
fn names(value: &Value) -> TestResult<BTreeSet<&str>> {
    value
        .as_array()
        .ok_or("required array")?
        .iter()
        .map(|v| v.as_str().ok_or_else(|| "required name".into()))
        .collect()
}

#[test]
fn registry_schema_keeps_the_legacy_object_and_adds_a_closed_nonnullable_named_object() -> TestResult
{
    let registry = bondline_commands::registry();
    assert_eq!(registry.len(), 55);
    let entry = registry
        .iter()
        .find(|v| v.name == "ring.create")
        .ok_or("ring registry")?;
    assert!(!entry.read_only);
    let schema = serde_json::to_value(&entry.input_schema)?;
    assert_eq!(schema["type"], "object");
    let alternatives = schema["anyOf"]
        .as_array()
        .ok_or("three request alternatives")?;
    assert_eq!(alternatives.len(), 3);
    let mut selectors = BTreeSet::new();
    for alternative in alternatives {
        let branch = node(&schema, alternative)?;
        assert_eq!(branch["type"], "object");
        assert_eq!(branch["additionalProperties"], false);
        let props = branch["properties"].as_object().ok_or("properties")?;
        let common = [
            "expected_revision",
            "page_id",
            "hydrogen_policy",
            "placement",
        ];
        let mut required = common.into_iter().collect::<BTreeSet<_>>();
        let mut fields = required.clone();
        if props.contains_key("size") {
            assert!(selectors.insert("size"));
            required.insert("size");
            fields.insert("size");
            assert_eq!(props["size"]["type"], "integer");
            assert_eq!(props["size"]["minimum"].as_f64(), Some(3.0));
            assert_eq!(props["size"]["maximum"].as_f64(), Some(8.0));
        } else if node(&schema, props.get("kind").ok_or("kind selector")?)?["enum"]
            == json!(["cyclohexane_chair_1", "cyclohexane_chair_2"])
        {
            assert!(selectors.insert("chair"));
            required.insert("kind");
            fields.insert("kind");
            assert_eq!(node(&schema, &props["kind"])?["type"], "string");
            assert!(!props.contains_key("size") && !props.contains_key("phase"));
        } else {
            assert!(selectors.insert("kind"));
            required.insert("kind");
            fields.insert("kind");
            fields.insert("phase");
            let kind = node(&schema, &props["kind"])?;
            assert_eq!(kind["type"], "string");
            assert_eq!(kind["enum"], json!(["benzene", "cyclopentadiene"]));
            let phase = node(&schema, &props["phase"])?;
            assert_eq!(phase["type"], "string");
            assert_eq!(phase["enum"], json!(["double_first", "single_first"]));
            assert!(
                props["phase"].get("default").is_none(),
                "contextual omission is not a null enum default"
            );
        }
        assert_eq!(names(&branch["required"])?, required);
        assert_eq!(
            props.keys().map(String::as_str).collect::<BTreeSet<_>>(),
            fields
        );
        assert_eq!(props["page_id"]["minimum"].as_f64(), Some(1.0));
        assert_eq!(
            node(&schema, &props["hydrogen_policy"])?["enum"],
            json!(["rectify_neutral_carbon"])
        );
        let placement = node(&schema, &props["placement"])?;
        assert_eq!(placement["type"], "object");
        assert_eq!(placement["unevaluatedProperties"], false);
        let placements = placement["oneOf"]
            .as_array()
            .ok_or("unchanged placement alternatives")?;
        assert_eq!(placements.len(), 3);
        for placement in placements {
            assert_eq!(placement["type"], "object");
            assert_eq!(placement["additionalProperties"], false);
        }
    }
    assert_eq!(selectors, BTreeSet::from(["size", "kind", "chair"]));
    Ok(())
}
