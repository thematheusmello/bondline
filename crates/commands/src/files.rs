use crate::{CommandResult, Engine, Error, StoredContent};
use bondline_render::Cancellation;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FileFormat {
    #[default]
    Cdxml,
    Cdx,
    MolV2000,
    MolV3000,
    Sdf,
}

impl FileFormat {
    pub(super) fn from_mol(version: bondline_io::mol::Version) -> Self {
        match version {
            bondline_io::mol::Version::V2000 => Self::MolV2000,
            bondline_io::mol::Version::V3000 => Self::MolV3000,
        }
    }
    pub(super) fn mol_version(self) -> Option<bondline_io::mol::Version> {
        match self {
            Self::Cdxml | Self::Cdx | Self::Sdf => None,
            Self::MolV2000 => Some(bondline_io::mol::Version::V2000),
            Self::MolV3000 => Some(bondline_io::mol::Version::V3000),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MolVersionChoice {
    #[default]
    Auto,
    V2000,
    V3000,
}

fn default_mol_scale() -> f64 {
    24.0
}
fn default_mol_origin() -> f64 {
    72.0
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OpenMol {
    pub path: String,
    #[serde(default)]
    pub discard_unsaved: bool,
    #[serde(default = "default_mol_scale")]
    #[schemars(extend("exclusiveMinimum" = 0.0))]
    pub points_per_unit: f64,
    #[serde(default = "default_mol_origin")]
    #[schemars(range(min = -bondline_model::MAX_POSITION_PT, max = bondline_model::MAX_POSITION_PT))]
    pub origin_x_pt: f64,
    #[serde(default = "default_mol_origin")]
    #[schemars(range(min = -bondline_model::MAX_POSITION_PT, max = bondline_model::MAX_POSITION_PT))]
    pub origin_y_pt: f64,
}

impl OpenMol {
    pub(super) fn transform(&self) -> Result<bondline_io::mol::MolTransform, Error> {
        Ok(bondline_io::mol::MolTransform::new(
            self.points_per_unit,
            bondline_model::Point2::new(self.origin_x_pt, self.origin_y_pt)?,
        )?)
    }
}

pub(super) fn native_mol_transform() -> Result<bondline_io::mol::MolTransform, Error> {
    Ok(bondline_io::mol::MolTransform::new(
        default_mol_scale(),
        bondline_model::Point2::new(default_mol_origin(), default_mol_origin())?,
    )?)
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SaveMol {
    pub path: String,
    #[serde(default)]
    pub overwrite: bool,
    #[serde(default)]
    pub version: MolVersionChoice,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum OpenEncoding {
    Cdxml,
    Cdx,
    Mol(bondline_io::mol::MolTransform),
    Sdf,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OpenCdxml {
    pub path: String,
    #[serde(default)]
    pub discard_unsaved: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SaveCdxml {
    pub path: String,
    #[serde(default)]
    pub overwrite: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OpenCdx {
    pub path: String,
    #[serde(default)]
    pub discard_unsaved: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SaveCdx {
    pub path: String,
    #[serde(default)]
    pub overwrite: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OpenSdf {
    pub path: String,
    #[serde(default)]
    pub discard_unsaved: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SaveSdf {
    pub path: String,
    #[serde(default)]
    pub overwrite: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct FileState {
    pub format: FileFormat,
    pub path: Option<String>,
    pub dirty: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct FileOperation {
    pub format: FileFormat,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_request_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_request_current: Option<bool>,
    pub operation: &'static str,
    pub path: String,
    pub input_revision: u64,
    pub byte_length: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directory_synced: Option<bool>,
}

#[derive(Debug)]
pub(super) enum FilePlan {
    Open {
        params: OpenCdxml,
        encoding: OpenEncoding,
        ui_request_id: Option<u64>,
        revision: u64,
        generation: u64,
    },
    Save {
        params: SaveCdxml,
        format: FileFormat,
        ui_request_id: Option<u64>,
        document: Arc<StoredContent>,
        save_slot: Arc<()>,
        revision: u64,
        generation: u64,
    },
}

/// Completed work must be applied to its engine after releasing the worker's I/O context.
#[derive(Debug)]
pub struct Completed(pub(super) Completion);

#[derive(Debug)]
pub(super) enum Completion {
    SdfFieldSet(crate::sdf_edit::FieldCandidate),
    Result {
        result: Box<CommandResult>,
        cancel: Option<Cancellation>,
    },
    #[cfg(not(target_arch = "wasm32"))]
    Open {
        loaded: bondline_io::files::LoadedFile<StoredContent>,
        format: FileFormat,
        ui_request_id: Option<u64>,
        revision: u64,
        generation: u64,
        cancel: Cancellation,
        discard_unsaved: bool,
    },
    #[cfg(not(target_arch = "wasm32"))]
    Save {
        receipt: bondline_io::files::SaveReceipt,
        format: FileFormat,
        ui_request_id: Option<u64>,
        document: Arc<StoredContent>,
        save_slot: Arc<()>,
        revision: u64,
        generation: u64,
    },
}

impl FilePlan {
    pub(super) fn bind_ui_request(&mut self, id: u64) {
        match self {
            Self::Open { ui_request_id, .. } | Self::Save { ui_request_id, .. } => {
                *ui_request_id = Some(id)
            }
        }
    }
    pub(super) fn complete(self, cancel: &Cancellation) -> Result<Completed, Error> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use bondline_io::files::{canonical_save_path, read_cdxml_file, save_cdxml_file};
            use std::path::Path;
            Ok(Completed(match self {
                Self::Open {
                    params,
                    encoding,
                    ui_request_id,
                    revision,
                    generation,
                } => {
                    let drawing =
                        |loaded: bondline_io::files::LoadedFile| bondline_io::files::LoadedFile {
                            path: loaded.path,
                            document: StoredContent::from(loaded.document),
                        };
                    let loaded = match encoding {
                        OpenEncoding::Cdxml => {
                            drawing(read_cdxml_file(Path::new(&params.path), cancel)?)
                        }
                        OpenEncoding::Cdx => drawing(bondline_io::files::read_cdx_file(
                            Path::new(&params.path),
                            cancel,
                        )?),
                        OpenEncoding::Mol(frame) => drawing(bondline_io::files::read_mol_file(
                            Path::new(&params.path),
                            frame,
                            cancel,
                        )?),
                        OpenEncoding::Sdf => {
                            let loaded =
                                bondline_io::files::read_sdf_file(Path::new(&params.path), cancel)?;
                            bondline_io::files::LoadedFile {
                                path: loaded.path,
                                document: StoredContent::from_sdf(loaded.document),
                            }
                        }
                    };
                    let format = match encoding {
                        OpenEncoding::Cdxml => FileFormat::Cdxml,
                        OpenEncoding::Cdx => FileFormat::Cdx,
                        OpenEncoding::Mol(_) => {
                            FileFormat::from_mol(loaded.document.drawing()?.mol_version().ok_or(
                                bondline_io::mol::MolError::InvalidField {
                                    line: 4,
                                    field: "MOL version",
                                },
                            )?)
                        }
                        OpenEncoding::Sdf => FileFormat::Sdf,
                    };
                    Completion::Open {
                        loaded,
                        format,
                        ui_request_id,
                        revision,
                        generation,
                        cancel: cancel.clone(),
                        discard_unsaved: params.discard_unsaved,
                    }
                }
                Self::Save {
                    params,
                    format,
                    ui_request_id,
                    document,
                    save_slot,
                    revision,
                    generation,
                } => {
                    cancel.check()?;
                    let path = canonical_save_path(Path::new(&params.path))?;
                    if path.to_str().is_none() {
                        return Err(Error::UnrepresentableFilePath { published: false });
                    }
                    Completion::Save {
                        receipt: match format {
                            FileFormat::Cdxml => save_cdxml_file(
                                &path,
                                document.drawing()?,
                                params.overwrite,
                                cancel,
                            )?,
                            FileFormat::Cdx => bondline_io::files::save_cdx_file(
                                &path,
                                document.drawing()?,
                                params.overwrite,
                                cancel,
                            )?,
                            FileFormat::MolV2000 | FileFormat::MolV3000 => {
                                bondline_io::files::save_mol_file_as(
                                    &path,
                                    document.drawing()?,
                                    params.overwrite,
                                    format.mol_version().ok_or(
                                        bondline_io::mol::MolError::InvalidField {
                                            line: 4,
                                            field: "MOL output version",
                                        },
                                    )?,
                                    native_mol_transform()?,
                                    cancel,
                                )?
                            }
                            FileFormat::Sdf => bondline_io::files::save_sdf_file(
                                &path,
                                document.sdf()?,
                                params.overwrite,
                                cancel,
                            )?,
                        },
                        format,
                        ui_request_id,
                        document,
                        save_slot,
                        revision,
                        generation,
                    }
                }
            }))
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = (self, cancel);
            Err(Error::FileBackendUnavailable)
        }
    }
}

impl Engine {
    pub(super) fn mol_save_format(&self, choice: MolVersionChoice) -> Result<FileFormat, Error> {
        let document = self.stored_document()?;
        let version = match choice {
            MolVersionChoice::V2000 => bondline_io::mol::Version::V2000,
            MolVersionChoice::V3000 => bondline_io::mol::Version::V3000,
            MolVersionChoice::Auto => self
                .file_format
                .mol_version()
                .or_else(|| document.mol_version())
                .unwrap_or(bondline_io::mol::Version::V2000),
        };
        Ok(FileFormat::from_mol(version))
    }

    pub(super) fn reserve_save(&self) -> Result<Arc<()>, Error> {
        if Arc::strong_count(&self.save_slot) != 1 {
            return Err(Error::SaveBusy);
        }
        Ok(Arc::clone(&self.save_slot))
    }

    pub fn finish(&mut self, completed: Completed) -> Result<CommandResult, Error> {
        match completed.0 {
            Completion::SdfFieldSet(candidate) => self.finish_sdf_field(candidate),
            Completion::Result { mut result, cancel } => {
                if let Some(cancel) = cancel {
                    cancel.check()?;
                }
                self.refresh_close_result(&mut result);
                Ok(*result)
            }
            #[cfg(not(target_arch = "wasm32"))]
            Completion::Open {
                loaded,
                format,
                ui_request_id,
                revision,
                generation,
                cancel,
                discard_unsaved,
            } => {
                if self.close_state() == crate::CloseState::Committed {
                    return Err(Error::CloseCommitted);
                }
                cancel.check()?;
                if revision != self.revision || generation != self.generation {
                    return Err(Error::StaleRevision);
                }
                if self.document != self.saved && !discard_unsaved {
                    return Err(Error::UnsavedChanges);
                }
                let ui_request_current =
                    ui_request_id.map(|id| self.ui_file_request_is_current(id));
                let path = loaded
                    .path
                    .into_os_string()
                    .into_string()
                    .map_err(|_| Error::UnrepresentableFilePath { published: false })?;
                let next_revision = self.next_revision()?;
                let next_generation = self
                    .generation
                    .checked_add(1)
                    .ok_or(Error::RevisionExhausted)?;
                let byte_length = loaded
                    .document
                    .source_info()
                    .map_or(0, |source| source.byte_length);
                let document = Arc::new(loaded.document);
                self.document = Arc::clone(&document);
                self.saved = document;
                self.file_path = Some(path.clone());
                self.file_format = format;
                self.identities = crate::bond::Ids::default();
                self.undo.clear();
                self.redo.clear();
                self.generation = next_generation;
                self.revision = next_revision;
                self.invalidate_close();
                self.mark_ui_file_completed(ui_request_id, ui_request_current);
                Ok(CommandResult {
                    changed: true,
                    file_operation: Some(FileOperation {
                        operation: "open",
                        format,
                        ui_request_id,
                        ui_request_current,
                        path,
                        input_revision: revision,
                        byte_length,
                        directory_synced: None,
                    }),
                    ..self.state()
                })
            }
            #[cfg(not(target_arch = "wasm32"))]
            Completion::Save {
                receipt,
                format,
                ui_request_id,
                document,
                // Keep the reservation through state application, beyond publication.
                save_slot: _save_slot,
                revision,
                generation,
            } => {
                // Validate the request before its own save changes the baseline/path/format.
                let ui_request_current =
                    ui_request_id.map(|id| self.ui_file_request_is_current(id));
                let path = receipt
                    .path
                    .into_os_string()
                    .into_string()
                    .map_err(|_| Error::UnrepresentableFilePath { published: true })?;
                if generation == self.generation
                    && self.close_state() != crate::CloseState::Committed
                {
                    let baseline_changed = self.saved != document
                        || self.file_path.as_deref() != Some(path.as_str())
                        || self.file_format != format;
                    self.saved = document;
                    self.file_path = Some(path.clone());
                    self.file_format = format;
                    self.trim_history(crate::MAX_HISTORY_BYTES);
                    if baseline_changed {
                        self.invalidate_close();
                    }
                    self.mark_ui_file_completed(ui_request_id, ui_request_current);
                }
                Ok(CommandResult {
                    file_operation: Some(FileOperation {
                        operation: "save",
                        format,
                        ui_request_id,
                        ui_request_current,
                        path,
                        input_revision: revision,
                        byte_length: receipt.byte_length,
                        directory_synced: Some(receipt.directory_synced),
                    }),
                    ..self.state()
                })
            }
        }
    }
}
