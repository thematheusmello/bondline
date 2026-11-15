//! Whole SMILES file Read/Save and explicit conversion/export without session association.

use super::{
    Failure, FileError, LoadedFile, ReadLimit, SaveReceipt, Stage, check_cancel, read_source_bytes,
    read_source_bytes_with_limit, save_cdxml_file,
};
use crate::{
    StoredDocument,
    smiles::{SmilesDocument, SmilesFileDocument},
};
use bondline_model::{Fragment, PageSize, Point2};
use bondline_render::Cancellation;
use std::{
    borrow::Cow,
    io,
    path::{Path, PathBuf},
};

/// Read and validate every meaningful notation while retaining the complete 64 MiB-bounded
/// file, including ignored lines, opaque application data and original record endings.
/// This library ownership operation creates no drawing or application association.
pub fn read_smiles_file(
    path: &Path,
    cancel: &Cancellation,
) -> Result<LoadedFile<SmilesFileDocument>, FileError> {
    let (path, bytes) = read_source_bytes(path, cancel)?;
    let document = SmilesFileDocument::parse_with_cancel(bytes.into(), cancel)
        .map_err(|error| FileError::new(&path, Stage::Decode, Failure::SmilesFile(error)))?;
    check_cancel(&path, Stage::Decode, cancel)?;
    Ok(LoadedFile { path, document })
}

/// Publish the exact complete owned source; never regenerate notation, metadata or endings.
/// Overwrite and publication-aware failures use the existing native atomic save contract.
pub fn save_smiles_file(
    path: &Path,
    document: &SmilesFileDocument,
    overwrite: bool,
    cancel: &Cancellation,
) -> Result<SaveReceipt, FileError> {
    save_smiles_file_with_hook(path, document, overwrite, cancel, &mut |_| Ok(()))
}

fn save_smiles_file_with_hook(
    path: &Path,
    document: &SmilesFileDocument,
    overwrite: bool,
    cancel: &Cancellation,
    hook: &mut impl FnMut(Stage) -> io::Result<()>,
) -> Result<SaveReceipt, FileError> {
    super::save_encoded_with_hook(
        path,
        overwrite,
        cancel,
        || Ok(Cow::Borrowed(document.as_bytes())),
        hook,
    )
}

/// Export one explicitly selected structure to a new single-record SMILES file.
/// The unchanged noncanonical writer supplies the notation; one LF terminates the file
/// record. Coordinates/document context are excluded, and the fragment stays unchanged.
/// The writer's 65,536-byte notation cap stays intact; framing can add one more byte.
/// This pure Fragment API grants no source-backed Save authority or session association.
pub fn export_smiles_fragment_file(
    output: &Path,
    fragment: &Fragment,
    cancel: &Cancellation,
) -> Result<SaveReceipt, FileError> {
    export_smiles_with_hook(output, fragment, cancel, &mut |_| Ok(()))
}

fn export_smiles_with_hook(
    output: &Path,
    fragment: &Fragment,
    cancel: &Cancellation,
    hook: &mut impl FnMut(Stage) -> io::Result<()>,
) -> Result<SaveReceipt, FileError> {
    super::save_encoded_with_hook(
        output,
        false,
        cancel,
        || {
            let mut notation = crate::smiles::write_fragment(fragment)?;
            notation.push(b'\n');
            Ok(Cow::Owned(notation))
        },
        hook,
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmilesConversionReceipt {
    pub input_path: PathBuf,
    pub input_byte_length: usize,
    pub atom_count: usize,
    pub bond_count: usize,
    pub output: SaveReceipt,
}

/// Convert one complete supported SMILES notation into a new native CDXML drawing.
/// The whole input is ASCII, at most 64 KiB, and contains only the existing optional
/// single final LF/CRLF. Layout accepts only a connected neutral nonisotopic carbon/
/// Single path of 1–12 atoms; bond length, origin and page are supplied explicitly.
/// The input is untouched, and existing outputs are never overwritten. No original
/// SMILES backing or application session is created. Publication errors retain their
/// real published/recovery metadata; a successful receipt is not a current-document Save.
pub fn convert_smiles_to_cdxml_file(
    input: &Path,
    output: &Path,
    bond_length_pt: f64,
    origin: Point2,
    page: PageSize,
    cancel: &Cancellation,
) -> Result<SmilesConversionReceipt, FileError> {
    convert_with_publisher(input, bond_length_pt, origin, page, cancel, |document| {
        save_cdxml_file(output, document, false, cancel)
    })
}

fn convert_with_publisher(
    input: &Path,
    bond_length_pt: f64,
    origin: Point2,
    page: PageSize,
    cancel: &Cancellation,
    publish: impl FnOnce(&StoredDocument) -> Result<SaveReceipt, FileError>,
) -> Result<SmilesConversionReceipt, FileError> {
    convert_with_layout(
        input,
        bond_length_pt,
        origin,
        page,
        cancel,
        |source, length, origin, page, _| source.layout_carbon_path(length, origin, page),
        publish,
    )
}

/// Derive a new native drawing from one bounded connected neutral acyclic notation.
/// Retains the complete input through the same create-only publisher as the legacy path.
/// The atom-center bounds start at origin; rendered ink must independently fit the page.
/// Unsupported chemistry/topology or an unsolved layout refuses without publishing.
pub fn convert_smiles_acyclic_to_cdxml_file(
    input: &Path,
    output: &Path,
    bond_length_pt: f64,
    origin: Point2,
    page: PageSize,
    cancel: &Cancellation,
) -> Result<SmilesConversionReceipt, FileError> {
    convert_with_layout(
        input,
        bond_length_pt,
        origin,
        page,
        cancel,
        SmilesDocument::layout_neutral_acyclic,
        |document| save_cdxml_file(output, document, false, cancel),
    )
}

/// Derive one bounded simple neutral ring and its attached trees into new native CDXML.
/// Uses the same whole-notation owner, cancellation and create-only publisher.
pub fn convert_smiles_simple_ring_to_cdxml_file(
    input: &Path,
    output: &Path,
    bond_length_pt: f64,
    origin: Point2,
    page: PageSize,
    cancel: &Cancellation,
) -> Result<SmilesConversionReceipt, FileError> {
    convert_with_layout(
        input,
        bond_length_pt,
        origin,
        page,
        cancel,
        SmilesDocument::layout_neutral_simple_ring,
        |document| save_cdxml_file(output, document, false, cancel),
    )
}

/// Derive all bounded neutral tree/simple-ring components into one new native drawing.
/// Retains the whole notation and existing create-only publication/error semantics.
pub fn convert_smiles_components_to_cdxml_file(
    input: &Path,
    output: &Path,
    bond_length_pt: f64,
    origin: Point2,
    page: PageSize,
    cancel: &Cancellation,
) -> Result<SmilesConversionReceipt, FileError> {
    convert_with_layout(
        input,
        bond_length_pt,
        origin,
        page,
        cancel,
        SmilesDocument::layout_neutral_components,
        |document| save_cdxml_file(output, document, false, cancel),
    )
}

fn convert_with_layout(
    input: &Path,
    bond_length_pt: f64,
    origin: Point2,
    page: PageSize,
    cancel: &Cancellation,
    layout: impl FnOnce(
        &SmilesDocument,
        f64,
        Point2,
        PageSize,
        &Cancellation,
    ) -> Result<bondline_model::Document, crate::smiles::LayoutError>,
    publish: impl FnOnce(&StoredDocument) -> Result<SaveReceipt, FileError>,
) -> Result<SmilesConversionReceipt, FileError> {
    let (input_path, bytes) = read_source_bytes_with_limit(input, cancel, ReadLimit::Smiles)?;
    let original = SmilesDocument::parse(bytes.into())
        .map_err(|error| FileError::new(&input_path, Stage::Decode, Failure::Smiles(error)))?;
    check_cancel(&input_path, Stage::Decode, cancel)?;
    let model = layout(&original, bond_length_pt, origin, page, cancel)
        .map_err(|error| FileError::new(&input_path, Stage::Decode, Failure::Layout(error)))?;
    check_cancel(&input_path, Stage::Decode, cancel)?;
    let mut drawing = StoredDocument::default();
    *drawing.model_mut() = model;
    let output = publish(&drawing)?;
    // Reading metadata after publication keeps the complete raw/IR owner alive through work.
    Ok(SmilesConversionReceipt {
        input_path,
        input_byte_length: original.as_bytes().len(),
        atom_count: original.atoms().len(),
        bond_count: original.bonds().len(),
        output,
    })
}

#[cfg(test)]
mod tests {
    use super::super::{
        Encoding, ReadLimit, Stage, read_bounded_bytes, read_source_bytes, save_format_with_hook,
    };
    use super::*;
    use std::{
        fs,
        io::{self, Cursor, Read},
        sync::atomic::{AtomicU64, Ordering},
    };
    struct Directory(PathBuf);
    impl Directory {
        fn new() -> io::Result<Self> {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let p = std::env::temp_dir().join(format!(
                "bondline-smiles-conversion-unit-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&p)?;
            Ok(Self(p))
        }
    }
    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn whole_file_save_keeps_exact_bytes_and_original_publication_errors()
    -> Result<(), Box<dyn std::error::Error>> {
        let source = b"CC\topaque\xff\0\r\nO final EOF";
        let document = SmilesFileDocument::parse(source.as_slice().into())?;
        let before = document.clone();
        for overwrite in [false, true] {
            for failure in [Stage::Write, Stage::Cleanup, Stage::SyncDirectory] {
                let directory = Directory::new()?;
                let path = directory.0.join("whole-file.smi");
                if overwrite {
                    fs::write(&path, b"old destination")?;
                }
                let error = save_smiles_file_with_hook(
                    &path,
                    &document,
                    overwrite,
                    &Cancellation::default(),
                    &mut |stage| {
                        if stage == failure {
                            Err(io::Error::other("injected whole-file publication error"))
                        } else {
                            Ok(())
                        }
                    },
                )
                .err()
                .ok_or("discarded publication failure")?;
                let published = failure != Stage::Write;
                assert_eq!(error.code(), "io_error");
                assert_eq!(error.stage(), failure.name());
                assert_eq!(error.path(), super::super::canonical_save_path(&path)?);
                assert_eq!(error.published(), published);
                assert!(error.recovery_path().is_none());
                if published {
                    assert_eq!(fs::read(&path)?, source);
                } else if overwrite {
                    assert_eq!(fs::read(&path)?, b"old destination");
                } else {
                    assert!(!path.exists());
                }
                assert_eq!(document, before);
                assert_eq!(
                    fs::read_dir(&directory.0)?.count(),
                    usize::from(published || overwrite)
                );
            }
        }
        Ok(())
    }

    fn origin() -> Result<Point2, bondline_model::Error> {
        Point2::new(72.0, 72.0)
    }
    struct PartialRead(Cursor<Vec<u8>>);
    impl Read for PartialRead {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            let n = buffer.len().min(1100);
            self.0.read(&mut buffer[..n])
        }
    }

    #[test]
    fn smiles_export_retains_real_publication_error_metadata_and_exact_bytes()
    -> Result<(), Box<dyn std::error::Error>> {
        let model = SmilesDocument::parse(b"[CH3][CH3]".as_slice().into())?
            .normalize_explicit_h_fragment()?;
        let before = model.clone();
        for failure in [Stage::Write, Stage::Cleanup, Stage::SyncDirectory] {
            let directory = Directory::new()?;
            let path = directory.0.join("export.smi");
            let error =
                export_smiles_with_hook(&path, &model, &Cancellation::default(), &mut |stage| {
                    if stage == failure {
                        Err(io::Error::other("injected export publication error"))
                    } else {
                        Ok(())
                    }
                })
                .err()
                .ok_or("ignored publication failure")?;
            let published = failure != Stage::Write;
            assert_eq!(error.code(), "io_error");
            assert_eq!(error.stage(), failure.name());
            assert_eq!(error.path(), super::super::canonical_save_path(&path)?);
            assert_eq!(error.published(), published);
            assert!(error.recovery_path().is_none());
            if published {
                assert_eq!(fs::read(&path)?, b"[CH3][CH3]\n");
            } else {
                assert!(!path.exists());
            }
            assert_eq!(fs::read_dir(&directory.0)?.count(), usize::from(published));
            assert_eq!(model, before);
        }
        Ok(())
    }

    #[test]
    fn smiles_export_racing_creator_and_cancellation_use_the_shared_publisher()
    -> Result<(), Box<dyn std::error::Error>> {
        let model = SmilesDocument::parse(b"[CH3][CH3]".as_slice().into())?
            .normalize_explicit_h_fragment()?;
        let before = model.clone();
        for action in ["create", "cancel-before", "cancel-after"] {
            let directory = Directory::new()?;
            let path = directory.0.join("export.smi");
            let cancel = Cancellation::default();
            let result = export_smiles_with_hook(&path, &model, &cancel, &mut |stage| {
                if stage == Stage::Publish {
                    if action == "create" {
                        fs::write(&path, b"racing creator")?;
                    } else if action == "cancel-before" {
                        cancel.cancel();
                    }
                }
                if stage == Stage::Cleanup && action == "cancel-after" {
                    cancel.cancel();
                }
                Ok(())
            });
            match action {
                "create" => {
                    let error = result.err().ok_or("replaced creator")?;
                    assert_eq!(error.code(), "already_exists");
                    assert_eq!(error.stage(), "publish");
                    assert!(!error.published());
                    assert_eq!(fs::read(&path)?, b"racing creator");
                }
                "cancel-before" => {
                    let error = result.err().ok_or("ignored cancellation")?;
                    assert_eq!(error.code(), "cancelled");
                    assert_eq!(error.stage(), "publish");
                    assert!(!error.published());
                    assert!(!path.exists());
                }
                _ => {
                    let receipt = result?;
                    assert!(cancel.is_cancelled());
                    assert_eq!(receipt.path, fs::canonicalize(&path)?);
                    assert_eq!(receipt.byte_length, 11);
                    assert_eq!(fs::read(&path)?, b"[CH3][CH3]\n");
                }
            }
            assert_eq!(model, before);
            assert_eq!(
                fs::read_dir(&directory.0)?.count(),
                usize::from(action != "cancel-before")
            );
        }
        Ok(())
    }

    #[test]
    fn active_read_budget_bounds_growth_and_keeps_legacy_limit_label()
    -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new("growing-source.smi");
        let cancel = Cancellation::default();
        let bytes = read_bounded_bytes(
            path,
            &mut PartialRead(Cursor::new(vec![b'C'; 65_536])),
            &cancel,
            ReadLimit::Smiles,
        )?;
        assert_eq!(bytes.len(), 65_536);
        assert!(bytes.capacity() <= 65_536);
        let error = read_bounded_bytes(
            path,
            &mut PartialRead(Cursor::new(vec![b'C'; 65_537])),
            &cancel,
            ReadLimit::Smiles,
        )
        .err()
        .ok_or("read growth passed cap")?;
        assert_eq!(error.code(), "file_too_large");
        assert_eq!(error.stage(), "read");
        assert!(error.to_string().contains("64 KiB"));
        let directory = Directory::new()?;
        let large = directory.0.join("large.cdxml");
        fs::File::create(&large)?.set_len(64 * 1024 * 1024 + 1)?;
        let error = read_source_bytes(&large, &cancel)
            .err()
            .ok_or("native cap disappeared")?;
        assert_eq!(error.code(), "file_too_large");
        assert!(
            error
                .to_string()
                .starts_with("file exceeds the 64 MiB input limit")
        );
        Ok(())
    }

    #[test]
    fn shared_publication_stage_errors_keep_actual_output_and_input_truth()
    -> Result<(), Box<dyn std::error::Error>> {
        for failure in [
            Stage::CreateTemporary,
            Stage::Write,
            Stage::Metadata,
            Stage::SyncFile,
            Stage::Publish,
            Stage::Cleanup,
            Stage::SyncDirectory,
        ] {
            let directory = Directory::new()?;
            let input = directory.0.join("source.smi");
            let output = directory.0.join("new.cdxml");
            fs::write(&input, b"CCC\r\n")?;
            let error = convert_with_publisher(
                &input,
                48.0,
                origin()?,
                PageSize::A4,
                &Cancellation::default(),
                |document| {
                    save_format_with_hook(
                        &output,
                        document,
                        false,
                        &Cancellation::default(),
                        Encoding::Cdxml,
                        &mut |stage| {
                            if stage == failure {
                                Err(io::Error::other("injected conversion publication failure"))
                            } else {
                                Ok(())
                            }
                        },
                    )
                },
            )
            .err()
            .ok_or("ignored stage failure")?;
            let published = matches!(failure, Stage::Cleanup | Stage::SyncDirectory);
            assert_eq!(error.published(), published);
            assert_eq!(error.stage(), failure.name());
            assert_eq!(error.path(), super::super::canonical_save_path(&output)?);
            assert!(error.recovery_path().is_none());
            assert_eq!(fs::read(&input)?, b"CCC\r\n");
            if published {
                let document =
                    super::super::read_cdxml_file(&output, &Cancellation::default())?.document;
                assert!(document.is_fully_mapped());
                assert_eq!(document.model().pages()[0].fragments()[0].atoms().len(), 3);
            } else {
                assert!(!output.exists());
            }
            assert_eq!(
                fs::read_dir(&directory.0)?.count(),
                if published { 2 } else { 1 }
            );
        }
        Ok(())
    }

    #[test]
    fn competing_leafs_and_cancellation_use_real_create_only_publisher()
    -> Result<(), Box<dyn std::error::Error>> {
        for action in ["create", "hardlink", "cancel-before", "cancel-after"] {
            let directory = Directory::new()?;
            let input = directory.0.join("source.smi");
            let output = directory.0.join("new.cdxml");
            fs::write(&input, b"CC\n")?;
            let cancel = Cancellation::default();
            let result = convert_with_publisher(
                &input,
                48.0,
                origin()?,
                PageSize::A4,
                &cancel,
                |document| {
                    save_format_with_hook(
                        &output,
                        document,
                        false,
                        &cancel,
                        Encoding::Cdxml,
                        &mut |stage| {
                            if stage == Stage::Publish {
                                match action {
                                    "create" => fs::write(&output, b"competing creator")?,
                                    "hardlink" => fs::hard_link(&input, &output)?,
                                    "cancel-before" => cancel.cancel(),
                                    _ => {}
                                }
                            }
                            if stage == Stage::Cleanup && action == "cancel-after" {
                                cancel.cancel();
                            }
                            Ok(())
                        },
                    )
                },
            );
            if action == "cancel-after" {
                assert!(cancel.is_cancelled());
                let receipt = result?;
                assert_eq!(receipt.output.path, fs::canonicalize(&output)?);
                assert!(
                    super::super::read_cdxml_file(&output, &Cancellation::default())?
                        .document
                        .is_fully_mapped()
                );
            } else {
                let error = result.err().ok_or("race/cancel published")?;
                assert_eq!(
                    error.code(),
                    if action == "cancel-before" {
                        "cancelled"
                    } else {
                        "already_exists"
                    }
                );
                assert!(!error.published());
                if action == "create" {
                    assert_eq!(fs::read(&output)?, b"competing creator");
                } else if action == "hardlink" {
                    assert_eq!(fs::read(&output)?, b"CC\n");
                } else {
                    assert!(!output.exists());
                }
            }
            assert_eq!(fs::read(&input)?, b"CC\n");
            assert!(
                !fs::read_dir(&directory.0)?
                    .filter_map(Result::ok)
                    .any(|entry| entry
                        .path()
                        .extension()
                        .is_some_and(|extension| extension == "tmp"))
            );
        }
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn late_symlink_cannot_redirect_create_only_conversion()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = Directory::new()?;
        let input = directory.0.join("source.smi");
        let output = directory.0.join("late.cdxml");
        fs::write(&input, b"CC\n")?;
        let error = convert_with_publisher(
            &input,
            48.0,
            origin()?,
            PageSize::A4,
            &Cancellation::default(),
            |document| {
                save_format_with_hook(
                    &output,
                    document,
                    false,
                    &Cancellation::default(),
                    Encoding::Cdxml,
                    &mut |stage| {
                        if stage == Stage::Publish {
                            std::os::unix::fs::symlink(&input, &output)?;
                        }
                        Ok(())
                    },
                )
            },
        )
        .err()
        .ok_or("followed late symlink")?;
        assert_eq!(error.code(), "already_exists");
        assert!(!error.published());
        assert_eq!(fs::read_link(&output)?, input);
        assert_eq!(fs::read(&input)?, b"CC\n");
        assert_eq!(fs::read_dir(&directory.0)?.count(), 2);
        Ok(())
    }

    #[test]
    fn publisher_error_metadata_is_forwarded_without_rebuilding_the_error()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = Directory::new()?;
        let input = directory.0.join("source.smi");
        let output = directory.0.join("synthetic.cdxml");
        let recovery = directory.0.join("retained-original");
        fs::write(&input, b"CC\n")?;
        // Synthetic future-publisher metadata probe, not an observed Windows recovery.
        let mut failure = FileError::new(
            &output,
            Stage::Recovery,
            Failure::Recovery {
                publication: io::Error::other("publication"),
                restoration: io::Error::other("restoration"),
            },
        );
        failure.published = true;
        failure.recovery_path = Some(recovery.clone());
        let text = failure.to_string();
        let error = convert_with_publisher(
            &input,
            48.0,
            origin()?,
            PageSize::A4,
            &Cancellation::default(),
            |_| Err(failure),
        )
        .err()
        .ok_or("publisher error discarded")?;
        assert_eq!(error.code(), "recovery_required");
        assert_eq!(error.path(), output);
        assert_eq!(error.stage(), "recovery");
        assert!(error.published());
        assert_eq!(error.recovery_path(), Some(recovery.as_path()));
        assert_eq!(error.to_string(), text);
        assert_eq!(fs::read(&input)?, b"CC\n");
        Ok(())
    }
}
