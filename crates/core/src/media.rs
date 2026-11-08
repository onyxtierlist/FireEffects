//! How the project refers to media files, and which files it accepts.
//!
//! Project files must survive being moved, copied to another machine, or handed to
//! a collaborator. Storing absolute paths makes that impossible, so a media file
//! inside the project's own directory is stored relative to the project file and
//! everything else is stored absolute. Both decisions are made here, in one place,
//! so loading and saving cannot disagree.
//!
//! Nothing here decodes media. The media backend is still undecided (see
//! `docs/ARCHITECTURE.md`), so this module only classifies files by extension and
//! reports what the filesystem already knows.

use std::path::{Component, Path, PathBuf};

use crate::MediaKind;

/// Extensions the editor accepts, and the kind each one is treated as.
///
/// A file with one of these extensions can be added to a project. Whether it can
/// be previewed depends on the media backend landing in Stage 4; a project that
/// lists an undecodable file is still valid and still saves.
pub const SUPPORTED_EXTENSIONS: &[(&str, MediaKind)] = &[
    ("mp4", MediaKind::Video),
    ("m4v", MediaKind::Video),
    ("mov", MediaKind::Video),
    ("mkv", MediaKind::Video),
    ("webm", MediaKind::Video),
    ("wav", MediaKind::Audio),
    ("mp3", MediaKind::Audio),
    ("m4a", MediaKind::Audio),
    ("aac", MediaKind::Audio),
    ("flac", MediaKind::Audio),
    ("ogg", MediaKind::Audio),
    ("png", MediaKind::Image),
    ("jpg", MediaKind::Image),
    ("jpeg", MediaKind::Image),
    ("webp", MediaKind::Image),
    ("bmp", MediaKind::Image),
    ("tif", MediaKind::Image),
    ("tiff", MediaKind::Image),
    ("gif", MediaKind::Image),
];

/// The kind a file is treated as, based on its extension.
///
/// Matching ignores case, because `CLIP.MOV` and `clip.mov` are the same format to
/// every camera and every filesystem that allows both.
pub fn kind_for_path(path: &Path) -> Option<MediaKind> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();

    SUPPORTED_EXTENSIONS
        .iter()
        .find(|(supported, _)| *supported == extension)
        .map(|(_, kind)| *kind)
}

/// The name shown in the media panel: the file name, or the whole path when the
/// file has no name of its own.
pub fn display_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}

/// The string to store in a project file for `media`, given the directory the
/// project file lives in.
///
/// Media inside `project_dir` is stored relative, with `/` separators, so the
/// project and its media can be moved together. Everything else is stored as the
/// absolute path the user chose, because a relative path would be meaningless.
pub fn store_path(project_dir: &Path, media: &Path) -> String {
    let relative = media
        .strip_prefix(project_dir)
        .ok()
        .filter(|relative| !relative.as_os_str().is_empty());

    match relative {
        Some(relative) => relative
            .components()
            .filter_map(component_text)
            .collect::<Vec<_>>()
            .join("/"),
        None => media.to_string_lossy().into_owned(),
    }
}

/// The path to use for a stored media reference.
///
/// Relative references resolve against the project file's directory, which is what
/// makes a moved project still find its own media. Absolute references are used as
/// stored.
pub fn resolve_path(project_dir: &Path, stored: &str) -> PathBuf {
    let path = Path::new(stored);

    if path.is_absolute() {
        path.to_path_buf()
    } else {
        project_dir.join(path)
    }
}

/// Resolves a stored reference against the project file's directory, when the
/// project has one.
///
/// A project that has never been saved has no directory, and everything it stores
/// is absolute, so the reference stands as written.
pub fn resolve_stored(project_dir: Option<&Path>, stored: &str) -> PathBuf {
    match project_dir {
        Some(directory) => resolve_path(directory, stored),
        None => PathBuf::from(stored),
    }
}

/// Text for one path component, skipping `.` and refusing to invent anything for
/// parts that are not valid UTF-8.
fn component_text(component: Component<'_>) -> Option<String> {
    match component {
        Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extensions_are_classified_case_insensitively() {
        assert_eq!(kind_for_path(Path::new("clip.MP4")), Some(MediaKind::Video));
        assert_eq!(
            kind_for_path(Path::new("sound.Wav")),
            Some(MediaKind::Audio)
        );
        assert_eq!(
            kind_for_path(Path::new("still.jpeg")),
            Some(MediaKind::Image)
        );
    }

    #[test]
    fn unknown_and_extensionless_files_are_refused() {
        assert_eq!(kind_for_path(Path::new("notes.txt")), None);
        assert_eq!(kind_for_path(Path::new("README")), None);
        assert_eq!(kind_for_path(Path::new("archive.mp4.zip")), None);
    }

    #[test]
    fn media_inside_the_project_directory_is_stored_relative() {
        let stored = store_path(Path::new("/work/reel"), Path::new("/work/reel/media/a.mp4"));

        assert_eq!(stored, "media/a.mp4");
    }

    #[test]
    fn media_outside_the_project_directory_stays_absolute() {
        let stored = store_path(Path::new("/work/reel"), Path::new("/other/a.mp4"));

        assert_eq!(stored, "/other/a.mp4");
    }

    #[test]
    fn a_relative_reference_resolves_against_the_project_directory() {
        let resolved = resolve_path(Path::new("/work/reel"), "media/a.mp4");

        assert_eq!(resolved, PathBuf::from("/work/reel/media/a.mp4"));
    }

    #[test]
    fn an_absolute_reference_resolves_to_itself() {
        let resolved = resolve_path(Path::new("/work/reel"), "/other/a.mp4");

        assert_eq!(resolved, PathBuf::from("/other/a.mp4"));
    }

    #[test]
    fn storing_and_resolving_round_trips() {
        let project_dir = Path::new("/work/reel");
        let media = Path::new("/work/reel/footage/shot 01.mov");

        let stored = store_path(project_dir, media);

        assert_eq!(stored, "footage/shot 01.mov");
        assert_eq!(resolve_path(project_dir, &stored), media);
    }

    #[test]
    fn an_unsaved_project_resolves_references_as_written() {
        assert_eq!(
            resolve_stored(None, "/other/a.mp4"),
            PathBuf::from("/other/a.mp4")
        );
        assert_eq!(
            resolve_stored(Some(Path::new("/work/reel")), "media/a.mp4"),
            PathBuf::from("/work/reel/media/a.mp4")
        );
    }

    #[test]
    fn display_names_are_the_file_name() {
        assert_eq!(display_name(Path::new("/a/b/clip.mp4")), "clip.mp4");
        assert_eq!(display_name(Path::new("clip.mp4")), "clip.mp4");
    }
}
