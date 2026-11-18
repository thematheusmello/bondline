//! Existing IO API runtime witness; no proposed model type is referenced.
use bondline_io::StoredDocument;
use bondline_model::PageSize;
use serde_json::json;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const SOURCE: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?><CDXML CreationProgram="Bondline 0.1.0" BondLength="24.5" ChainAngle="120.5"><page id="1" DrawingSpace="Poster" Width="612" Height="792"><fragment id="2"><n id="3" NodeType="Element" Element="6" Charge="0" Radical="None" p="100 200" NumHydrogens="3"/><n id="4" NodeType="Element" Element="6" Charge="0" Radical="None" p="140 200" NumHydrogens="3"/><b id="5" B="3" E="4" Order="1"/></fragment></page></CDXML>"#;

#[test]
fn literal_root_style_projects_unscaled_points_and_fractional_degrees_without_losing_source()
-> TestResult {
    let stored = StoredDocument::from_cdxml(SOURCE.into(), PageSize::A4)?;
    assert_eq!(
        serde_json::to_value(stored.model())?["drawing_settings"],
        json!({"bond_length_pt":24.5,"chain_angle_degrees":120.5})
    );
    assert_eq!(stored.export_cdxml()?, SOURCE);
    assert_eq!(stored.to_cdxml()?.as_ref(), SOURCE);
    assert!(
        stored.is_fully_mapped(),
        "the literal canonical native profile must encode every byte"
    );
    let f = &stored.model().pages()[0].fragments()[0];
    assert_eq!((f.atoms().len(), f.bonds().len()), (2, 1));
    assert_eq!(
        f.atoms()
            .iter()
            .map(|a| a.explicit_hydrogens)
            .collect::<Vec<_>>(),
        [Some(3), Some(3)]
    );
    assert!(!stored.source_info().ok_or("source")?.modified);
    Ok(())
}

// Additive data/source declarations retain the exact first1,510-byte runtime case.
use bondline_io::DocumentError;
use bondline_model::DrawingSettings;
use serde_json::Value;
use std::{
    borrow::Cow,
    io::Write as _,
    process::{Command, Stdio},
};
const LEGACY: &[u8] = include_bytes!("fixtures/bondline-v0.cdxml");

fn source(properties: &str) -> TestResult<Vec<u8>> {
    Ok(std::str::from_utf8(LEGACY)?
        .replacen(
            "<CDXML CreationProgram=\"Bondline 0.1.0\"",
            &format!("<CDXML CreationProgram=\"Bondline 0.1.0\"{properties}"),
            1,
        )
        .into_bytes())
}
fn roots(raw: &[u8]) -> TestResult<Value> {
    let python = std::env::var_os("BONDLINE_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let mut child = Command::new(python)
        .args([
            "-B",
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/drawing_settings_root.py"
            ),
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("independent root stdin")?
        .write_all(raw)?;
    let result = child.wait_with_output()?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(serde_json::from_slice(&result.stdout)?)
}

#[test]
fn independent_root_decimal_reader_and_old_none_single_both_native_profiles_are_exact() -> TestResult
{
    // Qualify this root-only reader against original authored data before generated output.
    let original = roots(SOURCE)?;
    assert_eq!(
        original,
        json!({"root":[["CreationProgram","Bondline 0.1.0"],["BondLength","24.5"],["ChainAngle","120.5"]],
        "decimal":{"BondLength":"24.5","ChainAngle":"120.5"}})
    );
    let altered =
        std::str::from_utf8(SOURCE)?.replacen("BondLength=\"24.5\"", "BondLength=\"30.25\"", 1);
    assert_ne!(roots(altered.as_bytes())?["decimal"], original["decimal"]);
    let nested = std::str::from_utf8(SOURCE)?.replacen(
        "<page ",
        "<page BondLength=\"99\" ChainAngle=\"90\" ",
        1,
    );
    assert_eq!(
        roots(nested.as_bytes())?,
        original,
        "root reader must not flatten child properties"
    );
    for (properties, length, angle, expected) in [
        (
            "",
            None,
            None,
            json!({"root":[["CreationProgram","Bondline 0.1.0"]],"decimal":{}}),
        ),
        (
            " BondLength=\"24.5\"",
            Some(24.5),
            None,
            json!({"root":[["CreationProgram","Bondline 0.1.0"],["BondLength","24.5"]],"decimal":{"BondLength":"24.5"}}),
        ),
        (
            " ChainAngle=\"120.5\"",
            None,
            Some(120.5),
            json!({"root":[["CreationProgram","Bondline 0.1.0"],["ChainAngle","120.5"]],"decimal":{"ChainAngle":"120.5"}}),
        ),
        (
            " BondLength=\"24.5\" ChainAngle=\"120.5\"",
            Some(24.5),
            Some(120.5),
            original,
        ),
    ] {
        let raw = source(properties)?;
        let stored = StoredDocument::from_cdxml(raw.as_slice().into(), PageSize::A4)?;
        let settings = stored.model().drawing_settings();
        assert_eq!(settings.bond_length_pt(), length);
        assert_eq!(settings.chain_angle_degrees(), angle);
        assert_eq!(settings.is_empty(), length.is_none() && angle.is_none());
        assert!(stored.is_fully_mapped());
        assert_eq!(stored.export_cdxml()?, raw);
        assert!(matches!(stored.to_cdxml()?, Cow::Borrowed(_)));
        assert_eq!(roots(&raw)?, expected);
        let legacy = StoredDocument::from_cdxml(LEGACY.into(), PageSize::A4)?;
        assert_eq!(
            stored.model().pages(),
            legacy.model().pages(),
            "properties do not move atoms or resize the page"
        );
        if properties.is_empty() {
            assert!(
                serde_json::to_value(stored.model())?
                    .get("drawing_settings")
                    .is_none()
            );
            assert_eq!(raw, LEGACY);
        }
    }
    Ok(())
}

#[test]
fn style_only_current_and_original_models_do_not_erase_settings_or_change_the_chemical_bytes()
-> TestResult {
    let mut stored = StoredDocument::from_cdxml(SOURCE.into(), PageSize::A4)?;
    let original = stored.clone();
    let graph = stored.model().pages().to_vec();
    stored
        .model_mut()
        .set_drawing_settings(DrawingSettings::new(Some(36.25), None)?);
    assert_eq!(stored.model().pages(), graph.as_slice());
    assert_ne!(stored.model(), original.model());
    assert!(stored.source_info().ok_or("source")?.modified);
    assert!(matches!(
        stored.export_cdxml(),
        Err(DocumentError::UnsupportedEdit)
    ));
    let expected = source(" BondLength=\"36.25\"")?;
    let edited = stored.to_cdxml()?;
    assert!(matches!(edited, Cow::Owned(_)));
    assert_eq!(edited.as_ref(), expected);
    assert_eq!(
        roots(&edited)?,
        json!({"root":[["CreationProgram","Bondline 0.1.0"],["BondLength","36.25"]],"decimal":{"BondLength":"36.25"}})
    );
    assert_eq!(original.to_cdxml()?.as_ref(), SOURCE);
    assert!(!original.source_info().ok_or("source")?.modified);
    let reloaded = StoredDocument::from_cdxml(edited.into_owned().into(), PageSize::A4)?;
    assert!(reloaded.is_fully_mapped());
    assert_eq!(reloaded.model(), stored.model());
    assert!(!reloaded.source_info().ok_or("source")?.modified);
    stored
        .model_mut()
        .set_drawing_settings(original.model().drawing_settings());
    assert_eq!(stored.model(), original.model());
    assert!(!stored.source_info().ok_or("source")?.modified);
    assert!(matches!(stored.to_cdxml()?, Cow::Borrowed(_)));
    assert_eq!(stored.export_cdxml()?, SOURCE);
    Ok(())
}

#[test]
fn invalid_partial_nested_and_opaque_styles_stay_source_preserved_without_authority() -> TestResult
{
    let mut cases = vec![
        (
            source(" BondLength=\"broken\" ChainAngle=\"120.5\"")?,
            None,
            Some(120.5),
        ),
        (
            source(" BondLength=\"24.5\" ChainAngle=\"180\"")?,
            Some(24.5),
            None,
        ),
        (
            source(" BondLength=\"NaN\" ChainAngle=\"-inf\"")?,
            None,
            None,
        ),
        (
            source(" BondLength=\"100001\" ChainAngle=\"0\"")?,
            None,
            None,
        ),
        (
            source(" BondLength=\"0\" ChainAngle=\"1\"")?,
            None,
            Some(1.0),
        ),
    ];
    cases.push((
        std::str::from_utf8(SOURCE)?
            .replacen("<page ", "<page BondLength=\"99\" ChainAngle=\"90\" ", 1)
            .into_bytes(),
        Some(24.5),
        Some(120.5),
    ));
    cases.push((
        std::str::from_utf8(LEGACY)?
            .replacen("<page ", "<page BondLength=\"99\" ChainAngle=\"90\" ", 1)
            .into_bytes(),
        None,
        None,
    ));
    let grouped = std::str::from_utf8(LEGACY)?
        .replacen(
            "<fragment ",
            "<group BondLength=\"48\" ChainAngle=\"90\"><fragment ",
            1,
        )
        .replacen("</fragment>", "</fragment></group>", 1);
    cases.push((grouped.into_bytes(), None, None));
    cases.push((
        std::str::from_utf8(SOURCE)?
            .replacen("<CDXML ", "<CDXML Private=\"retained\" ", 1)
            .into_bytes(),
        Some(24.5),
        Some(120.5),
    ));
    cases.push((
        std::str::from_utf8(SOURCE)?
            .replacen(
                "</page>",
                "<t><s>style note</s></t></page><!--preserve-->",
                1,
            )
            .into_bytes(),
        Some(24.5),
        Some(120.5),
    ));
    for (raw, length, angle) in cases {
        let mut stored = StoredDocument::from_cdxml(raw.as_slice().into(), PageSize::A4)?;
        assert_eq!(stored.model().drawing_settings().bond_length_pt(), length);
        assert_eq!(
            stored.model().drawing_settings().chain_angle_degrees(),
            angle
        );
        assert!(!stored.is_fully_mapped());
        assert_eq!(stored.export_cdxml()?, raw);
        assert_eq!(stored.to_cdxml()?.as_ref(), raw);
        assert!(!stored.source_info().ok_or("source")?.modified);
        let original = stored.clone();
        stored
            .model_mut()
            .set_drawing_settings(DrawingSettings::new(Some(36.25), Some(179.0))?);
        assert!(stored.source_info().ok_or("source")?.modified);
        assert_eq!(stored.model().pages(), original.model().pages());
        assert!(matches!(
            stored.to_cdxml(),
            Err(DocumentError::UnsupportedEdit)
        ));
        assert_eq!(
            original.export_cdxml()?,
            raw,
            "unsupported preference and all other bytes remain authoritative"
        );
    }
    Ok(())
}

#[test]
fn equivalent_numeric_setting_lexemes_now_edit_only_the_changed_typed_value() -> TestResult {
    let raw = source(" BondLength=\"24.500\" ChainAngle=\"120.500\"")?;
    let mut stored = StoredDocument::from_cdxml(raw.as_slice().into(), PageSize::A4)?;
    assert!(stored.is_fully_mapped());
    assert_eq!(
        stored.model().drawing_settings().bond_length_pt(),
        Some(24.5)
    );
    assert_eq!(
        stored.model().drawing_settings().chain_angle_degrees(),
        Some(120.5)
    );
    assert_eq!(stored.to_cdxml()?.as_ref(), raw);
    let before = stored.clone();
    stored
        .model_mut()
        .set_drawing_settings(DrawingSettings::new(Some(36.25), Some(120.5))?);
    let expected =
        std::str::from_utf8(&raw)?.replace("BondLength=\"24.500\"", "BondLength=\"36.25\"");
    assert_ne!(expected.as_bytes(), raw);
    assert_eq!(stored.to_cdxml()?.as_ref(), expected.as_bytes());
    assert_eq!(stored.model().pages(), before.model().pages());
    assert_eq!(before.to_cdxml()?.as_ref(), raw);
    *stored.model_mut() = before.model().clone();
    assert_eq!(stored.export_cdxml()?, raw);
    Ok(())
}
