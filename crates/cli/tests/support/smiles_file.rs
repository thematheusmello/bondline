use serde_json::{Value, json};

pub const SOURCE: &[u8] =
    b"CC ethane\r\n\n \tignored PRIVATE_IGNORED\nc1ccccc1\taromatic\xff\n[13CH3][OH]\tPRIVATE_DATA";

pub fn assert_file(value: &Value, revision: u64) {
    assert_eq!(value["revision"], revision);
    assert_eq!(value["document"], Value::Null);
    assert!(value.get("collection").is_none());
    assert_eq!(
        value["smiles_file"],
        json!({"format":"smiles","byte_length":SOURCE.len(),"record_count":3})
    );
    assert_eq!(
        value["source"],
        json!({"format":"smiles","byte_length":SOURCE.len(),"modified":false})
    );
    assert_eq!(value["file"]["format"], "smiles");
    assert_eq!(value["file"]["dirty"], false);
    assert_eq!(value["undo_depth"], 0);
    assert_eq!(value["redo_depth"], 0);
    for private in ["PRIVATE_DATA", "PRIVATE_IGNORED", "c1ccccc1"] {
        assert!(!value.to_string().contains(private));
    }
}
