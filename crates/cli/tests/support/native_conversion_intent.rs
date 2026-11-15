use super::*;

#[test]
fn headless_converter_intent_has_no_native_host() -> Result<(), Box<dyn std::error::Error>> {
    let result = invoke(
        "ui.file",
        json!({"expected_revision":0,"action":"convert_smiles"}),
    )?;
    assert_eq!(result.status.code(), Some(2));
    assert!(result.stdout.is_empty());
    let error: Value = serde_json::from_slice(&result.stderr)?;
    assert_eq!(error["error"]["code"], "ui_unavailable");
    Ok(())
}
