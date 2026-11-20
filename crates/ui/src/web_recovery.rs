//! The browser draft is real CDXML, never a filesystem path or a native Save receipt.
use bondline_io::StoredDocument;
use bondline_model::PageSize;

#[cfg(target_arch = "wasm32")]
pub(super) const KEY: &str = "bondline.browser-draft.cdxml.v1";
const MAX_BYTES: usize = 1024 * 1024;

pub(super) fn decode(source: &str) -> Result<StoredDocument, String> {
    if source.len() > MAX_BYTES {
        return Err("The browser draft exceeds the 1 MiB recovery limit.".into());
    }
    let stored = StoredDocument::from_cdxml(source.as_bytes().into(), PageSize::A4)
        .map_err(|error| error.to_string())?;
    if !stored.has_display_projection() {
        return Err("The browser draft contains unsupported drawing data.".into());
    }
    Ok(stored)
}

pub(super) fn encode(document: &StoredDocument) -> Result<String, String> {
    let bytes = document.to_cdxml().map_err(|error| error.to_string())?;
    if bytes.len() > MAX_BYTES {
        return Err("The drawing exceeds the 1 MiB browser recovery limit.".into());
    }
    String::from_utf8(bytes.into_owned()).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bondline_commands::Engine;
    use serde_json::json;

    #[test]
    fn browser_recovery_retains_the_real_registry_drawing_and_redone_state()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut engine = Engine::default();
        let empty = encode(engine.stored_document()?)?;
        engine.dispatch(
            "bond.create",
            json!({"page_id":1,"begin_x_pt":120,"begin_y_pt":180,"end_x_pt":168,"end_y_pt":180}),
        )?;
        let authored = encode(engine.stored_document()?)?;
        assert_ne!(authored, empty);
        let reopened = decode(&authored)?;
        assert_eq!(reopened.model(), engine.stored_document()?.model());
        assert_eq!(reopened.to_cdxml()?.as_ref(), authored.as_bytes());
        engine.dispatch("history.undo", json!({}))?;
        assert_eq!(encode(engine.stored_document()?)?, empty);
        engine.dispatch("history.redo", json!({}))?;
        assert_eq!(encode(engine.stored_document()?)?, authored);
        assert!(
            engine.state().file.dirty,
            "automatic browser recovery never acknowledges native Save"
        );
        Ok(())
    }

    #[test]
    fn recovery_refuses_bad_opaque_and_over_limit_data_without_rewriting_input() {
        for source in [
            "not XML".to_owned(),
            "<CDXML Unknown='keep'><page/></CDXML>".to_owned(),
            "x".repeat(MAX_BYTES + 1),
        ] {
            let original = source.clone();
            assert!(decode(&source).is_err());
            assert_eq!(source, original);
        }
    }
}
