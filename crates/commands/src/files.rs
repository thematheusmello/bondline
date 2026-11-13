use crate::{CommandResult, Engine, Error};
use bondline_io::StoredDocument;
use bondline_render::Cancellation;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

#[derive(Clone, Debug, Serialize)]
pub struct FileState {
    pub path: Option<String>,
    pub dirty: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct FileOperation {
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
        revision: u64,
        generation: u64,
    },
    Save {
        params: SaveCdxml,
        document: Arc<StoredDocument>,
        revision: u64,
        generation: u64,
    },
}

/// Completed work must be applied to its engine after releasing the worker's I/O context.
#[derive(Debug)]
pub struct Completed(pub(super) Completion);

#[derive(Debug)]
pub(super) enum Completion {
    Result {
        result: Box<CommandResult>,
        cancel: Option<Cancellation>,
    },
    #[cfg(not(target_arch = "wasm32"))]
    Open {
        loaded: bondline_io::files::LoadedFile,
        revision: u64,
        generation: u64,
        cancel: Cancellation,
        discard_unsaved: bool,
    },
    #[cfg(not(target_arch = "wasm32"))]
    Save {
        receipt: bondline_io::files::SaveReceipt,
        document: Arc<StoredDocument>,
        revision: u64,
        generation: u64,
    },
}

impl FilePlan {
    pub(super) fn complete(self, cancel: &Cancellation) -> Result<Completed, Error> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use bondline_io::files::{canonical_save_path, read_cdxml_file, save_cdxml_file};
            use std::path::Path;
            Ok(Completed(match self {
                Self::Open {
                    params,
                    revision,
                    generation,
                } => Completion::Open {
                    loaded: read_cdxml_file(Path::new(&params.path), cancel)?,
                    revision,
                    generation,
                    cancel: cancel.clone(),
                    discard_unsaved: params.discard_unsaved,
                },
                Self::Save {
                    params,
                    document,
                    revision,
                    generation,
                } => {
                    cancel.check()?;
                    let path = canonical_save_path(Path::new(&params.path))?;
                    if path.to_str().is_none() {
                        return Err(Error::UnrepresentableFilePath { published: false });
                    }
                    Completion::Save {
                        receipt: save_cdxml_file(&path, &document, params.overwrite, cancel)?,
                        document,
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
    pub fn finish(&mut self, completed: Completed) -> Result<CommandResult, Error> {
        match completed.0 {
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
                self.identities = crate::bond::Ids::default();
                self.undo.clear();
                self.redo.clear();
                self.generation = next_generation;
                self.revision = next_revision;
                self.invalidate_close();
                Ok(CommandResult {
                    changed: true,
                    file_operation: Some(FileOperation {
                        operation: "open",
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
                document,
                revision,
                generation,
            } => {
                let path = receipt
                    .path
                    .into_os_string()
                    .into_string()
                    .map_err(|_| Error::UnrepresentableFilePath { published: true })?;
                if generation == self.generation
                    && self.close_state() != crate::CloseState::Committed
                {
                    let baseline_changed =
                        self.saved != document || self.file_path.as_deref() != Some(path.as_str());
                    self.saved = document;
                    self.file_path = Some(path.clone());
                    self.trim_history(crate::MAX_HISTORY_BYTES);
                    if baseline_changed {
                        self.invalidate_close();
                    }
                }
                Ok(CommandResult {
                    file_operation: Some(FileOperation {
                        operation: "save",
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
