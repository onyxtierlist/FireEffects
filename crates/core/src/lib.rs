//! Platform-independent foundation types for FireEffects.
//!
//! This crate must stay free of UI, windowing, codec, and filesystem code so that
//! editing logic can be tested without a display or media files. The desktop
//! application depends on this crate; it never depends on the application.

mod error;
mod frame_rate;
mod input;
mod media;
mod project;
mod project_file;
mod workspace;

pub use error::{Error, Result};
pub use frame_rate::FrameRate;
pub use input::{Action, BINDINGS, Binding, Key, Modifiers, binding_for, resolve};
pub use media::{
    SUPPORTED_EXTENSIONS, display_name, kind_for_path, resolve_path, resolve_stored, store_path,
};
pub use project::{
    Clip, MediaAsset, MediaKind, Project, ProjectSettings, ProjectValidationError, Track, TrackKind,
};
pub use project_file::{
    PROJECT_FILE_EXTENSION, PROJECT_FORMAT_ID, PROJECT_FORMAT_VERSION, ProjectFileError,
    decode_project, encode_project,
};
pub use workspace::{Panel, Workspace};
