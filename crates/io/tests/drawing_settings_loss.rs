//! Independent data-only loss boundaries. Root owns execution and adoption.
use bondline_io::{
    DocumentError, StoredDocument,
    display::{self, DisplayError},
    mol::{self, MolTransform, Version, v3000},
    smiles::{RecordSelection, SmilesFileDocument},
};
use bondline_model::{DrawingSettings, PageId, PageSize, Point2};
use bondline_render::Cancellation;
use std::{borrow::Cow, fs, sync::Arc};

#[path = "../../../xtask/oracles/files_test_support.rs"]
#[allow(
    dead_code,
    reason = "reuse only the unchanged isolated-directory helper"
)]
mod support;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const V2: &[u8] = include_bytes!("fixtures/mol/ethane-v2000.mol");
const V3: &[u8] = include_bytes!("fixtures/v3000/noncontiguous-cc.mol");
const CDX: &[u8] = include_bytes!("fixtures/cdx/metadata-explicit.cdx");
const SMILES: &[u8] = b"CC first\r\nC1.N.C1 \t\0\xffPRIVATE-META\r\nCC duplicate\tname\n";

fn page() -> TestResult<PageId> {
    PageId::new(1).ok_or_else(|| "page1".into())
}
fn frame() -> TestResult<MolTransform> {
    Ok(MolTransform::new(24.0, Point2::new(72.0, 72.0)?)?)
}
fn settings() -> TestResult<[DrawingSettings; 3]> {
    Ok([
        DrawingSettings::new(Some(30.0), None)?,
        DrawingSettings::new(None, Some(120.0))?,
        DrawingSettings::new(Some(30.0), Some(120.0))?,
    ])
}

// Original canonical A4 ethane, not an output captured from Bondline. A4 is
// deliberate: the existing whole-MOL page guard must not mask settings loss.
fn native(settings: DrawingSettings) -> Vec<u8> {
    let mut attrs = String::new();
    if let Some(value) = settings.bond_length_pt() {
        attrs.push_str(&format!(" BondLength=\"{value}\""));
    }
    if let Some(value) = settings.chain_angle_degrees() {
        attrs.push_str(&format!(" ChainAngle=\"{value}\""));
    }
    format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?><CDXML CreationProgram=\"Bondline 0.1.0\"{attrs}><page id=\"1\" DrawingSpace=\"Poster\" Width=\"{}\" Height=\"{}\"><fragment id=\"2\"><n id=\"3\" NodeType=\"Element\" Element=\"6\" Charge=\"0\" Radical=\"None\" p=\"72 96\" NumHydrogens=\"3\"/><n id=\"4\" NodeType=\"Element\" Element=\"6\" Charge=\"0\" Radical=\"None\" p=\"120 96\" NumHydrogens=\"3\"/><b id=\"5\" B=\"3\" E=\"4\" Order=\"1\"/></fragment></page></CDXML>", PageSize::A4.width_pt(), PageSize::A4.height_pt()).into_bytes()
}
fn plain() -> TestResult<StoredDocument> {
    Ok(StoredDocument::from_cdxml(
        Arc::from(native(DrawingSettings::default())),
        PageSize::A4,
    )?)
}

#[test]
fn whole_mol_refuses_each_present_setting_before_atomic_publication() -> TestResult {
    let unstyled_native = plain()?;
    assert!(
        unstyled_native.is_fully_mapped(),
        "native authority precondition"
    );
    let mut unstyled_free = StoredDocument::default();
    unstyled_free.model_mut().append_fragments(
        page()?,
        vec![
            unstyled_native.model().pages()[0].fragments()[0]
                .as_ref()
                .clone(),
        ],
    )?;
    let directory = support::FileTestDirectory::new()?;
    let existing = directory.path("settings-保留-λ.mol");
    let absent = directory.path("settings-new-λ.mol");
    let sentinel = b"original destination must not be replaced\0\xff";
    fs::write(&existing, sentinel)?;
    for unstyled in [&unstyled_native, &unstyled_free] {
        for version in [Version::V2000, Version::V3000] {
            assert!(
                unstyled.to_mol_as(version, frame()?).is_ok(),
                "unstyled control"
            );
        }
        for value in settings()? {
            let mut styled = unstyled.clone();
            styled.model_mut().set_drawing_settings(value);
            let before = styled.clone();
            // Context-free fragment codecs are deliberately unchanged.
            let fragment = &styled.model().pages()[0].fragments()[0];
            assert!(mol::write_v2000(fragment, "", frame()?).is_ok());
            assert!(v3000::write_v3000(fragment, "", frame()?).is_ok());
            for version in [Version::V2000, Version::V3000] {
                assert!(matches!(
                    styled.to_mol_as(version, frame()?),
                    Err(DocumentError::UnsupportedMolLayout(_))
                ));
                for destination in [&existing, &absent] {
                    let error = bondline_io::files::save_mol_file_as(
                        destination,
                        &styled,
                        true,
                        version,
                        frame()?,
                        &Cancellation::default(),
                    )
                    .err()
                    .ok_or("whole-file settings loss was published")?;
                    assert_eq!(error.code(), "unsupported_content");
                    assert!(!error.published());
                    assert_eq!(fs::read(&existing)?, sentinel);
                    assert!(!absent.exists());
                }
            }
            assert_eq!(styled, before);
        }
    }
    // Native root settings are authored data even before a detached model edit.
    for value in settings()? {
        let source = native(value);
        let stored = StoredDocument::from_cdxml(Arc::from(source.clone()), PageSize::A4)?;
        assert!(stored.is_fully_mapped());
        assert_eq!(stored.model().drawing_settings(), value);
        assert_eq!(stored.export_cdxml()?, source);
        for version in [Version::V2000, Version::V3000] {
            assert!(matches!(
                stored.to_mol_as(version, frame()?),
                Err(DocumentError::UnsupportedMolLayout(_))
            ));
        }
    }
    Ok(())
}

#[test]
fn mol_style_drift_cannot_reuse_the_unchanged_fragment_or_original_file() -> TestResult {
    for (bytes, version) in [(V2, Version::V2000), (V3, Version::V3000)] {
        let original = StoredDocument::from_mol(Arc::from(bytes), PageSize::A4, frame()?)?;
        assert!(original.is_fully_mapped(), "regenerable MOL control");
        let snapshot = original.model_snapshot();
        assert_eq!(original.export_mol()?, bytes);
        assert!(matches!(
            original.to_mol_as(version, frame()?)?,
            Cow::Borrowed(_)
        ));
        let previous = original.model().pages()[0].fragments()[0].as_ref();
        for value in settings()? {
            let mut drift = original.clone();
            drift.model_mut().set_drawing_settings(value);
            assert!(drift.source_info().ok_or("source")?.modified);
            assert!(drift.export_mol().is_err());
            assert!(matches!(
                drift.to_mol_as(version, frame()?),
                Err(DocumentError::UnsupportedMolLayout(_))
            ));
            drift
                .model_mut()
                .replace_fragment(page()?, previous.clone())?;
            assert_eq!(drift.model().pages(), original.model().pages());
            assert!(matches!(
                drift.to_mol_as(version, frame()?),
                Err(DocumentError::UnsupportedMolLayout(_))
            ));
            if version == Version::V3000 {
                assert!(matches!(
                    drift.to_mol_as(Version::V2000, frame()?),
                    Err(DocumentError::CrossFormatSource {
                        from: "mol_v3000",
                        to: "mol_v2000"
                    })
                ));
            }
            assert_eq!(original.export_mol()?, bytes);
            assert!(Arc::ptr_eq(&snapshot, &original.model_snapshot()));
            drift
                .model_mut()
                .set_drawing_settings(DrawingSettings::default());
            assert_eq!(drift.export_mol()?, bytes);
        }
    }
    Ok(())
}

#[test]
fn settings_drift_invalidates_cdx_and_synchronized_smiles_without_erasing_source() -> TestResult {
    assert_eq!(&CDX[28..34], &[0, 128, 0, 0, 0, 0]);
    let mut binary = CDX.to_vec();
    // Root 0805, four-byte signed16.16 30pt. Original fixture remains untouched.
    binary.splice(34..34, [5, 8, 4, 0, 0, 0, 30, 0]);
    let original = StoredDocument::from_cdx(Arc::from(binary.clone()), PageSize::A4)?;
    let value = DrawingSettings::new(Some(30.0), None)?;
    assert_eq!(original.model().drawing_settings(), value);
    assert!(!original.is_fully_mapped());
    assert_eq!(original.export_cdx()?, binary);
    assert!(matches!(original.to_cdx()?, Cow::Borrowed(_)));
    let mut drift = original.clone();
    drift
        .model_mut()
        .set_drawing_settings(DrawingSettings::new(Some(45.0), Some(110.0))?);
    assert!(drift.export_cdx().is_err());
    assert!(drift.to_cdx().is_err());
    assert!(drift.source_info().ok_or("binary source")?.modified);
    drift.model_mut().set_drawing_settings(value);
    assert_eq!(drift.export_cdx()?, binary);
    assert_eq!(original.export_cdx()?, binary);

    let projected = StoredDocument::from_smiles_records(
        SmilesFileDocument::parse(Arc::from(SMILES))?,
        RecordSelection::All,
        48.0,
        Point2::new(72.0, 72.0)?,
        PageSize::A4,
        &Cancellation::default(),
    )?;
    let snapshot = projected.model_snapshot();
    let bytes = projected.smiles_file()?.as_bytes().as_ptr();
    assert!(projected.has_smiles_projection());
    let mut drift = projected.clone();
    drift
        .model_mut()
        .set_drawing_settings(DrawingSettings::new(None, Some(120.0))?);
    assert!(!drift.has_smiles_projection());
    assert!(drift.smiles_file().is_err());
    let replacement = projected.model().pages()[0].fragments()[0].as_ref().clone();
    assert!(
        drift
            .with_smiles_fragment(page()?, replacement, &Cancellation::default())
            .is_err()
    );
    assert!(matches!(
        display::drawing_page_scene(1024, 1024, &drift, page()?, &Cancellation::default()),
        Err(DisplayError::IncompleteSource)
    ));
    assert_eq!(projected.smiles_file()?.as_bytes(), SMILES);
    assert_eq!(projected.smiles_file()?.as_bytes().as_ptr(), bytes);
    assert!(Arc::ptr_eq(&snapshot, &projected.model_snapshot()));
    drift
        .model_mut()
        .set_drawing_settings(DrawingSettings::default());
    assert!(drift.has_smiles_projection());
    assert_eq!(drift.smiles_file()?.as_bytes(), SMILES);
    Ok(())
}

#[test]
fn creation_preferences_do_not_change_existing_pages_coordinates_h_or_scene() -> TestResult {
    let native = plain()?;
    let mut stored = StoredDocument::default();
    stored.model_mut().append_fragments(
        page()?,
        vec![native.model().pages()[0].fragments()[0].as_ref().clone()],
    )?;
    let snapshot = stored.model_snapshot();
    let original_scene =
        display::drawing_page_scene(1024, 1024, &stored, page()?, &Cancellation::default())?;
    for value in settings()? {
        let mut styled = stored.clone();
        styled.model_mut().set_drawing_settings(value);
        assert_eq!(styled.model().drawing_settings(), value);
        assert_eq!(styled.model().pages(), snapshot.pages());
        assert!(Arc::ptr_eq(
            &styled.model().pages()[0].fragments()[0],
            &snapshot.pages()[0].fragments()[0]
        ));
        let scene =
            display::drawing_page_scene(1024, 1024, &styled, page()?, &Cancellation::default())?;
        assert_eq!(scene.scene(), original_scene.scene());
    }
    assert!(Arc::ptr_eq(&snapshot, &stored.model_snapshot()));
    Ok(())
}
