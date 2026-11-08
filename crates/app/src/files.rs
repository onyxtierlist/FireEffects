//! Choosing paths: the file pickers behind the project dialogs and media import.
//!
//! Project dialogs accept a typed path, so the window works with no desktop
//! integration at all. A picker is offered alongside for the ordinary case: nobody
//! should have to type a directory to import a clip.
//!
//! The trait exists so the interface can be driven in tests, where a native dialog
//! cannot open: `scripted` provides the picker those tests use.

use std::path::{Path, PathBuf};

/// Asks the user for paths.
pub trait FilePicker {
    /// A project file to open.
    fn open_project(&self) -> Option<PathBuf>;
    /// A location to save to, starting from `current` if the project has one.
    fn save_project(&self, current: Option<&Path>) -> Option<PathBuf>;
    /// Media files to import. An empty list means the user cancelled.
    fn import_media(&self) -> Vec<PathBuf>;
    /// A replacement location for a media file the project cannot find.
    fn relink_media(&self, asset_name: &str) -> Option<PathBuf>;
}

/// The pickers a desktop user sees.
#[derive(Debug, Default)]
pub struct NativePicker;

impl FilePicker for NativePicker {
    fn open_project(&self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Open project")
            .add_filter(
                "FireEffects project",
                &[fireeffects_core::PROJECT_FILE_EXTENSION],
            )
            .add_filter("All files", &["*"])
            .pick_file()
    }

    fn save_project(&self, current: Option<&Path>) -> Option<PathBuf> {
        let mut dialog = rfd::FileDialog::new().set_title("Save project").add_filter(
            "FireEffects project",
            &[fireeffects_core::PROJECT_FILE_EXTENSION],
        );

        dialog = match current.and_then(Path::file_name) {
            Some(name) => dialog.set_file_name(name.to_string_lossy().into_owned()),
            None => dialog.set_file_name(format!(
                "Untitled.{}",
                fireeffects_core::PROJECT_FILE_EXTENSION
            )),
        };

        if let Some(directory) = current.and_then(Path::parent) {
            dialog = dialog.set_directory(directory);
        }

        dialog.save_file()
    }

    fn import_media(&self) -> Vec<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Import media")
            .add_filter("Media", &media_extensions())
            .add_filter("All files", &["*"])
            .pick_files()
            .unwrap_or_default()
    }

    fn relink_media(&self, asset_name: &str) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title(format!("Locate {asset_name}"))
            .add_filter("Media", &media_extensions())
            .add_filter("All files", &["*"])
            .pick_file()
    }
}

/// Every extension the project accepts, for the media dialog's filter.
fn media_extensions() -> Vec<&'static str> {
    fireeffects_core::SUPPORTED_EXTENSIONS
        .iter()
        .map(|(extension, _)| *extension)
        .collect()
}

/// A picker that hands back whatever a test prepared.
///
/// Every response is one-shot: taking it models a user answering a dialog once, so
/// a test that expects two responses has to set two and notices when it forgot.
#[cfg(test)]
pub mod scripted {
    use super::{FilePicker, Path, PathBuf};
    use std::cell::RefCell;

    /// Scripted answers to the four dialogs.
    #[derive(Debug, Default)]
    pub struct ScriptedPicker {
        /// Project file to return from the open dialog.
        pub choose_open: RefCell<Option<PathBuf>>,
        /// Location to return from the save dialog.
        pub choose_save: RefCell<Option<PathBuf>>,
        /// Files to return from the import dialog.
        pub choose_media: RefCell<Vec<PathBuf>>,
        /// Replacement file to return from the relink dialog.
        pub choose_relink: RefCell<Option<PathBuf>>,
    }

    impl FilePicker for ScriptedPicker {
        fn open_project(&self) -> Option<PathBuf> {
            self.choose_open.borrow_mut().take()
        }

        fn save_project(&self, _current: Option<&Path>) -> Option<PathBuf> {
            self.choose_save.borrow_mut().take()
        }

        fn import_media(&self) -> Vec<PathBuf> {
            std::mem::take(&mut *self.choose_media.borrow_mut())
        }

        fn relink_media(&self, _asset_name: &str) -> Option<PathBuf> {
            self.choose_relink.borrow_mut().take()
        }
    }
}
