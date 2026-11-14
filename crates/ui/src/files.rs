//! Original structure-file chooser; document I/O belongs to the shared executor.

use bondline_commands::FileFormat;
use eframe::egui;
use std::{
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
        mpsc::{self, Receiver, SyncSender, TrySendError},
    },
    time::Duration,
};

const MAX_PATH_BYTES: usize = 32_768;
const MAX_SCAN_ENTRIES: usize = 2_048;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileMode {
    Open,
    Save,
}

/// A file chooser never reads or writes document bytes. Dispatch this intent through the registry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FileAction {
    Open { path: String, discard_unsaved: bool },
    Save { path: String, overwrite: bool },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FileChooserEvent {
    Submit(FileAction),
    Cancel,
}

enum Confirmation {
    Discard(String),
    Overwrite(String),
}

#[derive(Default)]
pub struct FileChooser {
    mode: Option<FileMode>,
    format: FileFormat,
    directory: PathBuf,
    location: String,
    name: String,
    entries: Vec<DirectoryEntry>,
    worker: Option<DirectoryWorker>,
    pending: Option<ScanRequest>,
    generation: u64,
    loading: bool,
    truncated: bool,
    omitted: usize,
    error: Option<String>,
    confirmation: Option<Confirmation>,
    focus_name: bool,
}

impl FileChooser {
    pub fn begin(&mut self, mode: FileMode, directory: PathBuf, initial_name: String) {
        self.begin_with_format(mode, directory, initial_name, FileFormat::Cdxml);
    }

    pub fn begin_with_format(
        &mut self,
        mode: FileMode,
        directory: PathBuf,
        initial_name: String,
        format: FileFormat,
    ) {
        self.mode = Some(mode);
        self.name = initial_name;
        self.set_format(format);
        self.confirmation = None;
        self.focus_name = true;
        self.navigate(directory);
    }

    pub fn format(&self) -> FileFormat {
        self.format
    }

    fn set_name(&mut self, name: String) {
        self.name = name;
        if self.mode == Some(FileMode::Open)
            && let Some(format) = format_hint(&self.name)
        {
            self.format = format;
        }
    }

    fn set_format(&mut self, format: FileFormat) {
        self.format = match (self.mode, format) {
            (Some(FileMode::Open), FileFormat::MolV3000) => FileFormat::MolV2000,
            _ => format,
        };
        if self.mode == Some(FileMode::Save)
            && !self.name.is_empty()
            && !name_matches_format(&self.name, format)
        {
            let mut name = PathBuf::from(&self.name);
            name.set_extension(format_extension(format));
            if let Some(name) = name.to_str() {
                self.name = name.to_owned();
            }
        }
    }

    pub fn is_open(&self) -> bool {
        self.mode.is_some()
    }

    /// Restore an unsuccessful intent, preserving the exact requested path for an explicit retry.
    pub fn failed(&mut self, action: FileAction, code: &str, message: String) {
        self.failed_with_format(action, FileFormat::Cdxml, code, message);
    }

    pub fn failed_with_format(
        &mut self,
        action: FileAction,
        format: FileFormat,
        code: &str,
        message: String,
    ) {
        let authorized_overwrite = matches!(
            &action,
            FileAction::Save {
                overwrite: true,
                ..
            }
        );
        let (mode, path) = match action {
            FileAction::Open { path, .. } => (FileMode::Open, path),
            FileAction::Save { path, .. } => (FileMode::Save, path),
        };
        let requested = Path::new(&path);
        let directory = requested.parent().unwrap_or(Path::new(".")).to_owned();
        let name = requested
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(&path)
            .to_owned();
        self.begin_with_format(mode, directory, name.clone(), format);
        // A failed operation must display its actual attempted name, including a misleading suffix.
        self.name = name;
        self.confirmation = match (mode, code) {
            (FileMode::Open, "unsaved_changes") => Some(Confirmation::Discard(path)),
            (FileMode::Save, _) if authorized_overwrite => Some(Confirmation::Overwrite(path)),
            (FileMode::Save, "already_exists") => Some(Confirmation::Overwrite(path)),
            _ => None,
        };
        self.error = Some(message);
    }

    pub fn cancel(&mut self) {
        self.mode = None;
        self.confirmation = None;
        self.pending = None;
        if let Some(worker) = &self.worker {
            // No filesystem call or thread join may block the UI during cancellation.
            worker.latest.store(u64::MAX, Ordering::Relaxed);
        }
    }

    fn navigate(&mut self, path: PathBuf) {
        let Some(location) = path.to_str() else {
            self.error = Some(
                "This folder cannot be represented by the UTF-8 file command protocol.".to_owned(),
            );
            return;
        };
        if location.len() > MAX_PATH_BYTES || location.contains('\0') {
            self.error = Some("The folder path is invalid or exceeds 32768 bytes.".to_owned());
            return;
        }
        let Some(generation) = self
            .generation
            .checked_add(1)
            .filter(|value| *value < u64::MAX)
        else {
            self.error =
                Some("Directory navigation limit reached; restart the application.".to_owned());
            return;
        };
        self.generation = generation;
        self.location = location.to_owned();
        self.directory = path.clone();
        self.entries.clear();
        self.truncated = false;
        self.omitted = 0;
        self.error = None;
        self.loading = true;
        if self.worker.is_none() {
            match DirectoryWorker::new() {
                Ok(worker) => self.worker = Some(worker),
                Err(error) => {
                    self.loading = false;
                    self.error = Some(format!(
                        "Cannot start folder browser: {error}. Enter a complete file path instead."
                    ));
                    return;
                }
            }
        }
        if let Some(worker) = &self.worker {
            worker.latest.store(generation, Ordering::Relaxed);
        }
        self.pending = Some(ScanRequest { generation, path });
    }

    fn poll(&mut self) {
        let mut completed = Vec::new();
        if let Some(worker) = &self.worker {
            while let Ok(scan) = worker.results.try_recv() {
                completed.push(scan);
            }
            if let Some(request) = self.pending.take() {
                match worker.requests.try_send(request) {
                    Ok(()) => {}
                    Err(TrySendError::Full(request)) => self.pending = Some(request),
                    Err(TrySendError::Disconnected(_)) => {
                        self.loading = false;
                        self.error = Some(
                            "Folder browser stopped. Enter a complete file path instead."
                                .to_owned(),
                        );
                    }
                }
            }
        }
        for scan in completed {
            self.accept_scan(scan);
        }
    }

    fn accept_scan(&mut self, scan: DirectoryScan) {
        if scan.generation == self.generation && scan.path == self.directory {
            self.entries = scan.entries;
            self.loading = false;
            self.truncated = scan.truncated;
            self.omitted = scan.omitted;
            if scan.error.is_some() {
                self.error = scan.error;
            }
        }
    }

    fn submit(&mut self, dirty: bool) -> Option<FileAction> {
        let mode = self.mode?;
        let path = match resolve_path_as(&self.directory, &self.name, mode, self.format) {
            Ok(path) => path,
            Err(error) => {
                self.error = Some(error);
                return None;
            }
        };
        self.error = None;
        if mode == FileMode::Open && dirty {
            self.confirmation = Some(Confirmation::Discard(path));
            return None;
        }
        self.cancel();
        Some(match mode {
            FileMode::Open => FileAction::Open {
                path,
                discard_unsaved: false,
            },
            FileMode::Save => FileAction::Save {
                path,
                overwrite: false,
            },
        })
    }

    fn confirm(&mut self) -> Option<FileAction> {
        let action = match self.confirmation.take()? {
            Confirmation::Discard(path) => FileAction::Open {
                path,
                discard_unsaved: true,
            },
            Confirmation::Overwrite(path) => FileAction::Save {
                path,
                overwrite: true,
            },
        };
        self.cancel();
        Some(action)
    }

    /// Draw after the workspace. The caller must suppress background authoring while this is open.
    pub fn show(&mut self, context: &egui::Context, dirty: bool) -> Option<FileChooserEvent> {
        let mode = self.mode?;
        self.poll();
        if context.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            self.cancel();
            return Some(FileChooserEvent::Cancel);
        }
        if self.loading {
            context.request_repaint_after(Duration::from_millis(40));
        }
        let mut action = None;
        let mut open = true;
        let title = if mode == FileMode::Open {
            "Open structure"
        } else {
            "Save structure as"
        };
        egui::Window::new(title)
            .id(egui::Id::new("bondline-file-chooser"))
            .order(egui::Order::Foreground)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .collapsible(false)
            .resizable(false)
            .default_width((context.screen_rect().width() - 64.0).clamp(240.0, 640.0))
            .open(&mut open)
            .show(context, |ui| {
                if let Some(confirmation) = &self.confirmation {
                    let (message, button, path) = match confirmation {
                        Confirmation::Discard(path) => ("Discard unsaved changes and open this file?", "Discard and Open", path),
                        Confirmation::Overwrite(path) => ("Replace the existing file with the current document?", "Replace File", path),
                    };
                    ui.label(message);
                    ui.label(egui::RichText::new(path).strong());
                    if let Some(error) = &self.error {
                        ui.colored_label(egui::Color32::DARK_RED, error);
                    }
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.cancel();
                            action = Some(FileChooserEvent::Cancel);
                        }
                        if ui.button(button).clicked() {
                            action = self.confirm().map(FileChooserEvent::Submit);
                        }
                    });
                    return;
                }
                ui.horizontal(|ui| {
                    let label = ui.label("Folder");
                    let location = ui.add(egui::TextEdit::singleline(&mut self.location)
                        .id_source("bondline-file-folder").char_limit(MAX_PATH_BYTES)
                        .desired_width(ui.available_width() - 96.0)).labelled_by(label.id);
                    if let Err(error) = accessible_text_value(ui, &location, &mut self.location) {
                        self.error = Some(error);
                    }
                    let enter = location.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
                    if ui.button("Go").clicked() || enter {
                        self.navigate(PathBuf::from(&self.location));
                    }
                    if ui.add_enabled(self.directory.parent().is_some(), egui::Button::new("Up")).clicked()
                        && let Some(parent) = self.directory.parent()
                    {
                        self.navigate(parent.to_owned());
                    }
                });
                let mut selected_format=self.format;
                egui::ComboBox::from_label("Format").selected_text(format_label(self.format,mode))
                    .show_ui(ui,|ui| {
                        for &format in formats_for_mode(mode) {
                            ui.selectable_value(&mut selected_format,format,format_label(format,mode));
                        }
                    });
                if selected_format!=self.format {
                    self.set_format(selected_format);
                }
                ui.label(if mode==FileMode::Open {
                    "Structure files (*.cdxml, *.cdx, *.mol, *.sdf, *.sd). Enter a complete path to open a file outside this list."
                } else {
                    match self.format {
                        FileFormat::Cdxml=>"CDXML files (*.cdxml).",
                        FileFormat::Cdx=>"CDX files (*.cdx); only unchanged imported data can be saved.",
                        FileFormat::MolV2000=>"MOL V2000 files (*.mol); unsupported content cannot be saved in this format.",
                        FileFormat::MolV3000=>"MOL V3000 files (*.mol); unsupported content cannot be saved in this format.",
                        FileFormat::Sdf=>"SDF collections (*.sdf, *.sd); all records and their molecular versions are retained.",
                    }
                });
                if self.loading {
                    ui.horizontal(|ui| {ui.spinner(); ui.label("Reading folder…");});
                }
                let mut navigate = None;
                let mut select_name=None;
                let mut submit = false;
                egui::ScrollArea::vertical().id_salt("bondline-file-list")
                    .max_height((context.screen_rect().height() - 290.0).clamp(80.0, 280.0))
                    .min_scrolled_height(80.0)
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        let visible=|entry:&DirectoryEntry| entry.directory || mode==FileMode::Open || name_matches_format(&entry.name,self.format);
                        if !self.entries.iter().any(visible) && !self.loading {
                            ui.label("No matching structure files or folders listed.");
                        }
                        for entry in self.entries.iter().filter(|entry|visible(entry)) {
                            let label = if entry.directory {format!("[Folder] {}", entry.name)} else {entry.name.clone()};
                            let response = ui.selectable_label(!entry.directory && self.name == entry.name, label);
                            if response.clicked() {
                                if entry.directory {
                                    navigate = Some(self.directory.join(&entry.name));
                                } else {
                                    select_name=Some(entry.name.clone());
                                }
                            }
                            if response.double_clicked() && !entry.directory {
                                submit = true;
                            }
                        }
                    });
                if let Some(path) = navigate {
                    self.navigate(path);
                }
                if let Some(name)=select_name {
                    self.set_name(name);
                }
                if self.truncated {
                    ui.label("Folder list limited to the first 2048 entries. Enter a known file path or choose a smaller folder.");
                }
                if self.omitted > 0 {
                    ui.label(format!("{} entries could not be listed (unsupported names or unreadable metadata).", self.omitted));
                }
                let previous_name=self.name.clone();
                ui.horizontal(|ui| {
                    let label = ui.label("File name");
                    let name = ui.add(egui::TextEdit::singleline(&mut self.name)
                        .id_source("bondline-file-name").char_limit(MAX_PATH_BYTES)
                        .desired_width(ui.available_width())).labelled_by(label.id);
                    if let Err(error) = accessible_text_value(ui, &name, &mut self.name) {
                        self.error = Some(error);
                    }
                    if self.focus_name {
                        name.request_focus();
                        self.focus_name = false;
                    }
                    submit |= name.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
                });
                if self.name!=previous_name {
                    self.set_name(self.name.clone());
                }
                if let Some(error) = &self.error {
                    ui.colored_label(egui::Color32::DARK_RED, error);
                }
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        self.cancel();
                        action = Some(FileChooserEvent::Cancel);
                    }
                    if (ui.button(if mode == FileMode::Open {"Open"} else {"Save"}).clicked() || submit)
                        && let Some(submitted) = self.submit(dirty) {
                        action = Some(FileChooserEvent::Submit(submitted));
                    }
                });
            });
        if !open {
            self.cancel();
            action = Some(FileChooserEvent::Cancel);
        }
        action
    }
}

fn accessible_text_value(
    ui: &egui::Ui,
    response: &egui::Response,
    value: &mut String,
) -> Result<(), String> {
    use egui::accesskit::{Action, ActionData};
    let requested = ui.input(|input| {
        input
            .accesskit_action_requests(response.id, Action::SetValue)
            .last()
            .and_then(|request| match &request.data {
                Some(ActionData::Value(value)) => {
                    Some(if value.len() > MAX_PATH_BYTES || value.contains('\0') {
                        Err(
                            "The accessible field value is invalid or exceeds 32768 bytes."
                                .to_owned(),
                        )
                    } else {
                        Ok(value.to_string())
                    })
                }
                _ => None,
            })
    });
    let failure = match requested {
        Some(Ok(requested)) => {
            *value = requested;
            ui.ctx().request_repaint();
            None
        }
        Some(Err(error)) => Some(error),
        None => None,
    };
    // Pinned egui TextEdit exposes selection, but does not implement SetValue itself.
    ui.ctx().accesskit_node_builder(response.id, |builder| {
        builder.add_action(Action::SetValue);
        builder.set_value(value.as_str());
    });
    failure.map_or(Ok(()), Err)
}

pub(crate) fn format_extension(format: FileFormat) -> &'static str {
    match format {
        FileFormat::Cdxml => "cdxml",
        FileFormat::Cdx => "cdx",
        FileFormat::MolV2000 | FileFormat::MolV3000 => "mol",
        FileFormat::Sdf => "sdf",
    }
}

fn formats_for_mode(mode: FileMode) -> &'static [FileFormat] {
    match mode {
        FileMode::Open => &[
            FileFormat::Cdxml,
            FileFormat::Cdx,
            FileFormat::MolV2000,
            FileFormat::Sdf,
        ],
        FileMode::Save => &[
            FileFormat::Cdxml,
            FileFormat::Cdx,
            FileFormat::MolV2000,
            FileFormat::MolV3000,
            FileFormat::Sdf,
        ],
    }
}

fn format_label(format: FileFormat, mode: FileMode) -> &'static str {
    match (format, mode) {
        (FileFormat::Cdxml, _) => "CDXML (*.cdxml)",
        (FileFormat::Cdx, _) => "CDX (*.cdx)",
        (FileFormat::MolV2000 | FileFormat::MolV3000, FileMode::Open) => {
            "MOL V2000 / V3000 (*.mol)"
        }
        (FileFormat::MolV2000, FileMode::Save) => "MOL V2000 (*.mol)",
        (FileFormat::MolV3000, FileMode::Save) => "MOL V3000 (*.mol)",
        (FileFormat::Sdf, _) => "SDF collection (*.sdf, *.sd)",
    }
}

fn name_matches_format(name: &str, format: FileFormat) -> bool {
    Path::new(name)
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case(format_extension(format))
                || format == FileFormat::Sdf && extension.eq_ignore_ascii_case("sd")
        })
}

fn format_hint(name: &str) -> Option<FileFormat> {
    let extension = Path::new(name).extension()?.to_str()?;
    if extension.eq_ignore_ascii_case("cdxml") {
        Some(FileFormat::Cdxml)
    } else if extension.eq_ignore_ascii_case("cdx") {
        Some(FileFormat::Cdx)
    } else if extension.eq_ignore_ascii_case("mol") {
        Some(FileFormat::MolV2000)
    } else if extension.eq_ignore_ascii_case("sdf") || extension.eq_ignore_ascii_case("sd") {
        Some(FileFormat::Sdf)
    } else {
        None
    }
}

#[cfg(test)]
fn resolve_path(directory: &Path, name: &str, mode: FileMode) -> Result<String, String> {
    resolve_path_as(directory, name, mode, FileFormat::Cdxml)
}

fn resolve_path_as(
    directory: &Path,
    name: &str,
    mode: FileMode,
    format: FileFormat,
) -> Result<String, String> {
    if name.trim().is_empty() || name.contains('\0') || name.len() > MAX_PATH_BYTES {
        return Err(
            "Enter a nonempty file name without NUL characters (maximum 32768 bytes).".to_owned(),
        );
    }
    let mut path = directory.join(name);
    match path.extension().and_then(|extension| extension.to_str()) {
        Some(extension)
            if mode == FileMode::Open
                && ["cdxml", "cdx", "mol", "sdf", "sd"]
                    .iter()
                    .any(|supported| extension.eq_ignore_ascii_case(supported)) => {}
        Some(extension)
            if mode == FileMode::Save
                && (extension.eq_ignore_ascii_case(format_extension(format))
                    || format == FileFormat::Sdf && extension.eq_ignore_ascii_case("sd")) => {}
        None if mode == FileMode::Save => {
            path.set_extension(format_extension(format));
        }
        _ => {
            return Err(if mode == FileMode::Open {
                "Choose a .cdxml, .cdx, .mol, .sdf or .sd file.".to_owned()
            } else {
                format!(
                    "Use the .{} extension for the selected format.",
                    format_extension(format)
                )
            });
        }
    }
    let path = path
        .into_os_string()
        .into_string()
        .map_err(|_| "The file path is not valid UTF-8.".to_owned())?;
    if path.len() > MAX_PATH_BYTES {
        return Err("The complete file path exceeds 32768 bytes.".to_owned());
    }
    Ok(path)
}

struct DirectoryEntry {
    name: String,
    directory: bool,
}

struct ScanRequest {
    generation: u64,
    path: PathBuf,
}

struct DirectoryScan {
    generation: u64,
    path: PathBuf,
    entries: Vec<DirectoryEntry>,
    truncated: bool,
    omitted: usize,
    error: Option<String>,
}

struct DirectoryWorker {
    requests: SyncSender<ScanRequest>,
    results: Receiver<DirectoryScan>,
    latest: Arc<AtomicU64>,
}

impl DirectoryWorker {
    fn new() -> std::io::Result<Self> {
        let (requests, receiver) = mpsc::sync_channel(1);
        let (sender, results) = mpsc::sync_channel(1);
        let latest = Arc::new(AtomicU64::new(0));
        let worker_latest = Arc::clone(&latest);
        std::thread::Builder::new()
            .name("bondline-folder-browser".to_owned())
            .spawn(move || {
                while let Ok(request) = receiver.recv() {
                    if let Some(result) = scan_directory(request, &worker_latest)
                        && sender.send(result).is_err()
                    {
                        break;
                    }
                }
            })?;
        Ok(Self {
            requests,
            results,
            latest,
        })
    }
}

impl Drop for DirectoryWorker {
    fn drop(&mut self) {
        self.latest.store(u64::MAX, Ordering::Relaxed);
    }
}

fn scan_directory(request: ScanRequest, latest: &AtomicU64) -> Option<DirectoryScan> {
    if request.generation != latest.load(Ordering::Relaxed) {
        return None;
    }
    let mut result = DirectoryScan {
        generation: request.generation,
        path: request.path,
        entries: Vec::new(),
        truncated: false,
        omitted: 0,
        error: None,
    };
    match std::fs::read_dir(&result.path) {
        Err(error) => result.error = Some(format!("Cannot read folder: {error}")),
        Ok(directory) => {
            // shortcut: list at most 2048 directory entries; add paging for large-library workflows.
            for (index, entry) in directory.take(MAX_SCAN_ENTRIES + 1).enumerate() {
                if request.generation != latest.load(Ordering::Relaxed) {
                    return None;
                }
                if index == MAX_SCAN_ENTRIES {
                    result.truncated = true;
                    break;
                }
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(_) => {
                        result.omitted += 1;
                        continue;
                    }
                };
                let Ok(name) = entry.file_name().into_string() else {
                    result.omitted += 1;
                    continue;
                };
                let metadata = match std::fs::metadata(entry.path()) {
                    Ok(metadata) => metadata,
                    Err(_) => {
                        result.omitted += 1;
                        continue;
                    }
                };
                let directory = metadata.is_dir();
                if directory || metadata.is_file() && format_hint(&name).is_some() {
                    result.entries.push(DirectoryEntry { name, directory });
                }
            }
        }
    }
    result.entries.sort_by(|a, b| {
        b.directory
            .cmp(&a.directory)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            .then_with(|| a.name.cmp(&b.name))
    });
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, sync::atomic::AtomicUsize};

    static NEXT_DIRECTORY: AtomicUsize = AtomicUsize::new(0);

    struct Directory(PathBuf);
    impl Directory {
        fn new() -> Result<Self, Box<dyn std::error::Error>> {
            let path = std::env::temp_dir().join(format!(
                "bondline-chooser-{}-{}",
                std::process::id(),
                NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path)?;
            Ok(Self(path))
        }
    }
    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn directory_scan_filters_and_sorts_without_reading_document_contents()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = Directory::new()?;
        fs::create_dir(directory.0.join("Z folder"))?;
        fs::write(directory.0.join("b.cdxml"), b"not XML")?;
        fs::write(directory.0.join("A.CDXML"), b"also not XML")?;
        fs::write(
            directory.0.join("other.mol"),
            b"not a valid MOL; listing never reads it",
        )?;
        fs::write(
            directory.0.join("binary.CDX"),
            b"opaque listing; never decoded",
        )?;
        fs::write(
            directory.0.join("collection.sdf"),
            b"listing does not decode SDF",
        )?;
        let latest = AtomicU64::new(1);
        let result = scan_directory(
            ScanRequest {
                generation: 1,
                path: directory.0.clone(),
            },
            &latest,
        )
        .ok_or("scan cancelled")?;
        assert_eq!(
            result
                .entries
                .iter()
                .map(|entry| (entry.name.as_str(), entry.directory))
                .collect::<Vec<_>>(),
            vec![
                ("Z folder", true),
                ("A.CDXML", false),
                ("b.cdxml", false),
                ("binary.CDX", false),
                ("collection.sdf", false),
                ("other.mol", false)
            ]
        );
        assert!(!result.truncated);
        assert!(result.error.is_none());
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn directory_scan_lists_usable_symlinks() -> Result<(), Box<dyn std::error::Error>> {
        use std::os::unix::fs::symlink;
        let directory = Directory::new()?;
        let target = Directory::new()?;
        fs::write(target.0.join("real.cdxml"), b"opaque")?;
        symlink(&target.0, directory.0.join("linked folder"))?;
        symlink(target.0.join("real.cdxml"), directory.0.join("alias.cdxml"))?;
        let latest = AtomicU64::new(1);
        let scan = scan_directory(
            ScanRequest {
                generation: 1,
                path: directory.0.clone(),
            },
            &latest,
        )
        .ok_or("scan cancelled")?;
        assert_eq!(
            scan.entries
                .iter()
                .map(|entry| (entry.name.as_str(), entry.directory))
                .collect::<Vec<_>>(),
            vec![("linked folder", true), ("alias.cdxml", false)]
        );
        assert_eq!(scan.omitted, 0);
        Ok(())
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn directory_scan_reports_names_that_the_json_protocol_cannot_represent()
    -> Result<(), Box<dyn std::error::Error>> {
        use std::os::unix::ffi::OsStringExt;
        let directory = Directory::new()?;
        fs::write(
            directory.0.join(std::ffi::OsString::from_vec(vec![0xff])),
            [],
        )?;
        let latest = AtomicU64::new(1);
        let scan = scan_directory(
            ScanRequest {
                generation: 1,
                path: directory.0.clone(),
            },
            &latest,
        )
        .ok_or("scan cancelled")?;
        assert_eq!(scan.omitted, 1);
        Ok(())
    }

    #[test]
    fn scan_cancellation_limits_and_missing_directory_are_explicit()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = Directory::new()?;
        let latest = AtomicU64::new(2);
        assert!(
            scan_directory(
                ScanRequest {
                    generation: 1,
                    path: directory.0.clone()
                },
                &latest
            )
            .is_none()
        );
        let missing = scan_directory(
            ScanRequest {
                generation: 2,
                path: directory.0.join("missing"),
            },
            &latest,
        )
        .ok_or("scan cancelled")?;
        assert!(missing.error.is_some());
        for index in 0..=MAX_SCAN_ENTRIES {
            fs::write(directory.0.join(format!("{index}.cdxml")), [])?;
        }
        let limited = scan_directory(
            ScanRequest {
                generation: 2,
                path: directory.0.clone(),
            },
            &latest,
        )
        .ok_or("scan cancelled")?;
        assert!(limited.truncated);
        assert!(limited.entries.len() <= MAX_SCAN_ENTRIES);
        Ok(())
    }

    #[test]
    fn explicit_path_rules_preserve_spaces_unicode_and_limit_advertised_formats()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = Directory::new()?;
        let expected = directory
            .0
            .join("λ sample.CDXML")
            .into_os_string()
            .into_string()
            .map_err(|_| "unicode")?;
        assert_eq!(
            resolve_path(&directory.0, "λ sample.CDXML", FileMode::Open)?,
            expected
        );
        assert_eq!(
            resolve_path(&directory.0, "fresh", FileMode::Save)?,
            directory.0.join("fresh.cdxml").to_str().ok_or("unicode")?
        );
        for bad in ["", "   ", "bad\0.cdxml", "sample.rxn"] {
            assert!(
                resolve_path(&directory.0, bad, FileMode::Open).is_err(),
                "{bad:?}"
            );
        }
        assert!(
            resolve_path(
                &directory.0,
                &"a".repeat(MAX_PATH_BYTES + 1),
                FileMode::Save
            )
            .is_err()
        );
        Ok(())
    }

    #[test]
    fn sdf_uses_one_complete_collection_family_for_both_extensions_and_retry()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = Directory::new()?;
        for name in ["sample.sdf", "sample.SD"] {
            assert_eq!(format_hint(name), Some(FileFormat::Sdf));
            assert!(name_matches_format(name, FileFormat::Sdf));
            assert_eq!(
                resolve_path_as(&directory.0, name, FileMode::Open, FileFormat::Sdf)?,
                directory.0.join(name).to_str().ok_or("path")?
            );
            assert_eq!(
                resolve_path_as(&directory.0, name, FileMode::Save, FileFormat::Sdf)?,
                directory.0.join(name).to_str().ok_or("path")?
            );
        }
        assert_eq!(
            formats_for_mode(FileMode::Open)
                .iter()
                .filter(|format| **format == FileFormat::Sdf)
                .count(),
            1
        );
        assert_eq!(
            formats_for_mode(FileMode::Save)
                .iter()
                .filter(|format| **format == FileFormat::Sdf)
                .count(),
            1
        );
        assert_eq!(
            format_label(FileFormat::Sdf, FileMode::Open),
            "SDF collection (*.sdf, *.sd)"
        );
        let mut chooser = FileChooser::default();
        chooser.begin_with_format(
            FileMode::Save,
            directory.0.clone(),
            "pair".to_owned(),
            FileFormat::Sdf,
        );
        assert_eq!(chooser.format(), FileFormat::Sdf);
        assert!(
            resolve_path_as(&directory.0, "pair.mol", FileMode::Save, FileFormat::Sdf).is_err()
        );
        let path = directory
            .0
            .join("known.mol")
            .to_str()
            .ok_or("path")?
            .to_owned();
        let action = FileAction::Save {
            path: path.clone(),
            overwrite: true,
        };
        chooser.failed_with_format(
            action.clone(),
            FileFormat::Sdf,
            "permission_denied",
            "denied".to_owned(),
        );
        assert_eq!(chooser.format(), FileFormat::Sdf);
        assert_eq!(chooser.confirm(), Some(action));
        Ok(())
    }

    #[test]
    fn opening_dirty_work_and_overwriting_require_separate_explicit_choices()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = Directory::new()?;
        let mut chooser = FileChooser::default();
        chooser.begin(
            FileMode::Open,
            directory.0.clone(),
            "sample.cdxml".to_owned(),
        );
        chooser.error = Some("Previous file failed to decode".to_owned());
        assert!(chooser.submit(true).is_none());
        assert!(
            chooser.error.is_none(),
            "a valid new intent must clear the previous file's error before confirmation"
        );
        assert!(matches!(
            chooser.confirmation,
            Some(Confirmation::Discard(_))
        ));
        chooser.confirmation = None;
        let action = chooser.submit(false).ok_or("no open action")?;
        assert!(matches!(
            &action,
            FileAction::Open {
                discard_unsaved: false,
                ..
            }
        ));
        assert!(!chooser.is_open());
        chooser.failed(
            action,
            "unsaved_changes",
            "Document changed while opening".to_owned(),
        );
        assert!(chooser.is_open());
        let discard = chooser.confirm().ok_or("no confirmed open")?;
        assert!(matches!(
            discard,
            FileAction::Open {
                discard_unsaved: true,
                ..
            }
        ));
        chooser.begin(
            FileMode::Save,
            directory.0.clone(),
            "sample.cdxml".to_owned(),
        );
        let save = chooser.submit(false).ok_or("no save action")?;
        assert!(matches!(
            &save,
            FileAction::Save {
                overwrite: false,
                ..
            }
        ));
        chooser.failed(
            save,
            "already_exists",
            "This file already exists".to_owned(),
        );
        assert!(matches!(
            chooser.confirmation,
            Some(Confirmation::Overwrite(_))
        ));
        assert!(matches!(
            chooser.confirm(),
            Some(FileAction::Save {
                overwrite: true,
                ..
            })
        ));
        assert!(
            !directory.0.join("sample.cdxml").exists(),
            "chooser never writes document bytes"
        );
        Ok(())
    }

    #[test]
    fn stale_scan_results_never_replace_the_current_directory()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = Directory::new()?;
        let mut chooser = FileChooser::default();
        chooser.begin(FileMode::Open, directory.0.clone(), String::new());
        let old = chooser.generation;
        let old_path = chooser.directory.clone();
        chooser.navigate(directory.0.join("next"));
        chooser.accept_scan(DirectoryScan {
            generation: old,
            path: old_path,
            entries: vec![DirectoryEntry {
                name: "stale.cdxml".to_owned(),
                directory: false,
            }],
            truncated: false,
            omitted: 0,
            error: None,
        });
        assert!(chooser.entries.is_empty());
        assert!(chooser.loading);
        Ok(())
    }

    #[test]
    fn escape_emits_presentation_cancel_without_a_file_submission()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = Directory::new()?;
        let context = egui::Context::default();
        crate::configure_context(&context);
        let mut chooser = FileChooser::default();
        chooser.begin(FileMode::Open, directory.0.clone(), String::new());
        for events in [
            vec![],
            vec![egui::Event::Key {
                key: egui::Key::Escape,
                physical_key: Some(egui::Key::Escape),
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        ] {
            let expected = (!events.is_empty()).then_some(FileChooserEvent::Cancel);
            context.begin_pass(egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1000.0, 700.0),
                )),
                events,
                ..Default::default()
            });
            assert_eq!(chooser.show(&context, false), expected);
            let _ = context.end_pass();
        }
        assert!(!chooser.is_open());
        Ok(())
    }

    #[test]
    fn typed_file_name_and_enter_emit_the_exact_registry_intent()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = Directory::new()?;
        let context = egui::Context::default();
        crate::configure_context(&context);
        let mut chooser = FileChooser::default();
        chooser.begin(FileMode::Save, directory.0.clone(), String::new());
        let mut emitted = None;
        for events in [
            vec![],
            vec![egui::Event::Text("λ sample".to_owned())],
            vec![egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: Some(egui::Key::Enter),
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        ] {
            context.begin_pass(egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1000.0, 700.0),
                )),
                events,
                ..Default::default()
            });
            emitted = chooser.show(&context, false).or(emitted);
            let _ = context.end_pass();
        }
        assert_eq!(
            emitted,
            Some(FileChooserEvent::Submit(FileAction::Save {
                path: directory
                    .0
                    .join("λ sample.cdxml")
                    .to_str()
                    .ok_or("unicode")?
                    .to_owned(),
                overwrite: false
            }))
        );
        assert!(!chooser.is_open());
        Ok(())
    }

    #[test]
    fn failed_known_path_save_retries_the_exact_authorized_overwrite()
    -> Result<(), Box<dyn std::error::Error>> {
        use bondline_commands::{Command, Engine, UiFilePlan};
        let directory = Directory::new()?;
        let path = directory.0.join("known.cdxml");
        let backup = directory.0.join("retained.cdxml");
        let mut engine = Engine::default();
        engine.set_ui_file_host(true);
        engine.dispatch("document.save_cdxml", serde_json::json!({"path":path}))?;
        let bytes = std::fs::read(&path)?;
        let requested = engine.dispatch(
            "ui.file",
            serde_json::json!({"expected_revision":0,"action":"save"}),
        )?;
        let id = requested.ui_file.request.ok_or("request")?.id;
        let UiFilePlan::SaveExisting {
            format: FileFormat::Cdxml,
            params,
        } = engine.claim_ui_file(id)?
        else {
            return Err("expected existing Save".into());
        };
        let actual = Command::SaveCdxml(params.clone());
        let action = FileAction::Save {
            path: params.path,
            overwrite: params.overwrite,
        };
        let prepared = engine.prepare_ui_file(id, actual)?;
        std::fs::rename(&path, &backup)?;
        std::fs::create_dir(&path)?;
        let failure = prepared
            .complete()
            .err()
            .ok_or("directory must reject a real file save")?
            .info();
        assert_eq!(std::fs::read(&backup)?, bytes);
        std::fs::remove_dir(&path)?;
        std::fs::rename(&backup, &path)?;
        assert!(engine.finish_ui_file(id, true));
        let mut chooser = FileChooser::default();
        chooser.failed(action.clone(), failure.code, failure.message);
        assert!(matches!(
            chooser.confirmation,
            Some(Confirmation::Overwrite(_))
        ));
        let retry = chooser.confirm().ok_or("retry action")?;
        assert_eq!(retry, action);
        let FileAction::Save {
            path: retry_path,
            overwrite,
        } = retry
        else {
            return Err("retry changed operation".into());
        };
        let retry = engine.prepare_ui_file(
            id,
            Command::SaveCdxml(bondline_commands::SaveCdxml {
                path: retry_path,
                overwrite,
            }),
        )?;
        engine.finish(retry.complete()?)?;
        assert!(engine.finish_ui_file(id, false));
        assert_eq!(std::fs::read(&path)?, bytes);
        Ok(())
    }

    #[test]
    fn selected_save_format_controls_extension_and_open_hint_allows_explicit_override()
    -> Result<(), Box<dyn std::error::Error>> {
        use bondline_commands::FileFormat;
        let directory = Directory::new()?;
        let mut chooser = FileChooser::default();
        chooser.begin_with_format(
            FileMode::Save,
            directory.0.clone(),
            "λ drawing.cdxml".to_owned(),
            FileFormat::MolV2000,
        );
        assert_eq!(chooser.format(), FileFormat::MolV2000);
        assert_eq!(chooser.name, "λ drawing.mol");
        assert_eq!(
            chooser.submit(false),
            Some(FileAction::Save {
                path: directory
                    .0
                    .join("λ drawing.mol")
                    .to_str()
                    .ok_or("path")?
                    .to_owned(),
                overwrite: false
            })
        );
        chooser.begin(
            FileMode::Save,
            directory.0.clone(),
            "Exact.CDXML".to_owned(),
        );
        assert_eq!(chooser.name, "Exact.CDXML");
        chooser.begin_with_format(
            FileMode::Open,
            directory.0.clone(),
            String::new(),
            FileFormat::Cdxml,
        );
        chooser.set_name("mixed.MOL".to_owned());
        assert_eq!(chooser.format(), FileFormat::MolV2000);
        chooser.set_format(FileFormat::Cdxml);
        assert!(matches!(
            chooser.submit(false),
            Some(FileAction::Open { .. })
        ));
        assert_eq!(
            chooser.format(),
            FileFormat::Cdxml,
            "explicitly selected decoder remains authoritative"
        );
        Ok(())
    }

    #[test]
    fn failed_mol_save_preserves_selected_format_and_exact_authorized_path()
    -> Result<(), Box<dyn std::error::Error>> {
        use bondline_commands::FileFormat;
        let directory = Directory::new()?;
        let action = FileAction::Save {
            path: directory
                .0
                .join("associated.cdxml")
                .to_str()
                .ok_or("path")?
                .to_owned(),
            overwrite: true,
        };
        let mut chooser = FileChooser::default();
        chooser.failed_with_format(
            action.clone(),
            FileFormat::MolV2000,
            "permission_denied",
            "Save was not published".to_owned(),
        );
        assert_eq!(chooser.format(), FileFormat::MolV2000);
        assert_eq!(
            chooser.name, "associated.cdxml",
            "failed known Save must display its exact attempted basename"
        );
        assert_eq!(chooser.confirm(), Some(action));
        assert_eq!(chooser.format(), FileFormat::MolV2000);
        Ok(())
    }

    #[test]
    fn open_uses_one_mol_family_choice_and_save_exposes_both_versions()
    -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            formats_for_mode(FileMode::Open),
            &[
                FileFormat::Cdxml,
                FileFormat::Cdx,
                FileFormat::MolV2000,
                FileFormat::Sdf
            ]
        );
        assert_eq!(
            format_label(FileFormat::MolV2000, FileMode::Open),
            "MOL V2000 / V3000 (*.mol)"
        );
        assert_eq!(
            formats_for_mode(FileMode::Save),
            &[
                FileFormat::Cdxml,
                FileFormat::Cdx,
                FileFormat::MolV2000,
                FileFormat::MolV3000,
                FileFormat::Sdf
            ]
        );
        assert_eq!(
            format_label(FileFormat::MolV3000, FileMode::Save),
            "MOL V3000 (*.mol)"
        );
        let directory = Directory::new()?;
        let mut chooser = FileChooser::default();
        chooser.begin_with_format(
            FileMode::Open,
            directory.0.clone(),
            String::new(),
            FileFormat::MolV3000,
        );
        assert_eq!(
            chooser.format(),
            FileFormat::MolV2000,
            "Open presents a family decoder, not an inferred MOL version"
        );
        Ok(())
    }

    #[test]
    fn cdx_hint_extension_and_failed_known_save_retain_the_chosen_encoding()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = Directory::new()?;
        let mut chooser = FileChooser::default();
        chooser.begin(FileMode::Open, directory.0.clone(), String::new());
        chooser.set_name("original.CDX".to_owned());
        assert_eq!(chooser.format(), FileFormat::Cdx);
        assert!(
            matches!(chooser.submit(false),Some(FileAction::Open {path,..}) if path.ends_with("original.CDX"))
        );
        chooser.begin_with_format(
            FileMode::Save,
            directory.0.clone(),
            "copy".to_owned(),
            FileFormat::Cdx,
        );
        assert_eq!(chooser.name, "copy.cdx");
        assert_eq!(format_label(FileFormat::Cdx, FileMode::Save), "CDX (*.cdx)");
        let action = FileAction::Save {
            path: directory
                .0
                .join("known.mol")
                .to_str()
                .ok_or("path")?
                .to_owned(),
            overwrite: true,
        };
        chooser.failed_with_format(
            action.clone(),
            FileFormat::Cdx,
            "permission_denied",
            "Save failed".to_owned(),
        );
        assert_eq!(chooser.name, "known.mol");
        assert_eq!(chooser.format(), FileFormat::Cdx);
        assert_eq!(chooser.confirm(), Some(action));
        Ok(())
    }

    #[test]
    fn save_as_mol_version_survives_suffix_case_name_edits_and_file_filtering()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = Directory::new()?;
        let mut chooser = FileChooser::default();
        chooser.begin_with_format(
            FileMode::Save,
            directory.0.clone(),
            "Exact.MOL".to_owned(),
            FileFormat::MolV3000,
        );
        assert_eq!(chooser.name, "Exact.MOL");
        chooser.set_name("λ selected.mol".to_owned());
        assert_eq!(chooser.format(), FileFormat::MolV3000);
        assert!(name_matches_format("existing.MOL", FileFormat::MolV3000));
        assert!(!name_matches_format("existing.cdxml", FileFormat::MolV3000));
        chooser.set_format(FileFormat::MolV2000);
        assert_eq!(chooser.name, "λ selected.mol");
        chooser.set_format(FileFormat::MolV3000);
        assert_eq!(chooser.name, "λ selected.mol");
        assert_eq!(
            chooser.submit(false),
            Some(FileAction::Save {
                path: directory
                    .0
                    .join("λ selected.mol")
                    .to_str()
                    .ok_or("path")?
                    .to_owned(),
                overwrite: false,
            })
        );
        assert_eq!(chooser.format(), FileFormat::MolV3000);
        Ok(())
    }

    #[test]
    fn failed_v3000_known_save_displays_and_retries_the_exact_misleading_path()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = Directory::new()?;
        let action = FileAction::Save {
            path: directory
                .0
                .join("associated.cdxml")
                .to_str()
                .ok_or("path")?
                .to_owned(),
            overwrite: true,
        };
        let mut chooser = FileChooser::default();
        chooser.failed_with_format(
            action.clone(),
            FileFormat::MolV3000,
            "permission_denied",
            "Not published".to_owned(),
        );
        assert_eq!(chooser.name, "associated.cdxml");
        assert_eq!(chooser.format(), FileFormat::MolV3000);
        assert_eq!(chooser.confirm(), Some(action));
        Ok(())
    }

    #[test]
    fn accessibility_set_value_is_advertised_applied_and_bounded()
    -> Result<(), Box<dyn std::error::Error>> {
        use egui::accesskit::{Action, ActionData, ActionRequest};
        let directory = Directory::new()?;
        let context = egui::Context::default();
        crate::configure_context(&context);
        context.enable_accesskit();
        let mut chooser = FileChooser::default();
        chooser.begin(
            FileMode::Save,
            directory.0.clone(),
            "Untitled.cdxml".to_owned(),
        );
        let raw = |events| egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1000.0, 700.0),
            )),
            events,
            ..Default::default()
        };
        context.begin_pass(raw(vec![]));
        chooser.show(&context, false);
        let output = context.end_pass();
        let (id, node) = output
            .platform_output
            .accesskit_update
            .as_ref()
            .ok_or("accessibility tree")?
            .nodes
            .iter()
            .find(|(_, node)| {
                node.role() == egui::accesskit::Role::TextInput
                    && node.value() == Some("Untitled.cdxml")
            })
            .ok_or("file-name field")?;
        assert!(
            node.supports_action(Action::SetValue),
            "editable file names must expose SetValue to assistive technology"
        );
        let id = *id;
        for (requested, accepted) in [
            ("λ sample.cdxml".to_owned(), true),
            ("bad\0.cdxml".to_owned(), false),
            ("a".repeat(MAX_PATH_BYTES + 1), false),
        ] {
            context.begin_pass(raw(vec![egui::Event::AccessKitActionRequest(
                ActionRequest {
                    action: Action::SetValue,
                    target: id,
                    data: Some(ActionData::Value(requested.clone().into_boxed_str())),
                },
            )]));
            assert!(chooser.show(&context, false).is_none());
            let output = context.end_pass();
            let node = output
                .platform_output
                .accesskit_update
                .as_ref()
                .ok_or("accessibility update")?
                .nodes
                .iter()
                .find(|(node_id, _)| *node_id == id)
                .ok_or("file field update")?;
            assert!(
                node.1.supports_action(Action::SetValue),
                "invalid input must not disable the next accessible correction"
            );
            assert_eq!(
                chooser.name,
                if accepted {
                    requested.as_str()
                } else {
                    "λ sample.cdxml"
                }
            );
            assert_eq!(chooser.error.is_none(), accepted);
        }
        Ok(())
    }
}
