//! The media library: what the project refers to, and what is actually on disk.
//!
//! The project file stores only where media is *supposed* to be — its stored path,
//! display name, and kind. Whether the file is present, and how large it is, is
//! recomputed here and never written to the project: that answer belongs to the
//! machine the project is open on, not to the project.
//!
//! `Library` owns no project. It mirrors one, and the commands that change media
//! take `&mut Project` so the two cannot drift: there is exactly one place that
//! adds or repoints an asset.

use std::fs;
use std::path::{Path, PathBuf};

use fireeffects_core::{
    MediaAsset, MediaKind, Project, display_name, kind_for_path, resolve_stored, store_path,
};

/// What the filesystem said about a media file when it was last checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaStatus {
    /// The file is where the project expects it.
    Available {
        /// Size in bytes.
        size: u64,
    },
    /// Nothing exists at the expected path.
    Missing,
    /// Something exists but could not be inspected.
    Unreadable {
        /// Why the file could not be read.
        reason: String,
    },
}

impl MediaStatus {
    /// Whether the media file can be used.
    pub fn is_available(&self) -> bool {
        matches!(self, Self::Available { .. })
    }

    /// Short description for the media panel, e.g. `"4.2 MB"` or `"Missing"`.
    pub fn summary(&self) -> String {
        match self {
            Self::Available { size } => human_size(*size),
            Self::Missing => "Missing".to_owned(),
            Self::Unreadable { reason } => format!("Unreadable: {reason}"),
        }
    }
}

/// A media item plus its current filesystem status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaEntry {
    /// The item as stored in the project file.
    pub asset: MediaAsset,
    /// Where the file is expected to be, after resolving a relative reference.
    pub path: PathBuf,
    /// What is actually there.
    pub status: MediaStatus,
}

impl MediaEntry {
    /// Looks up the media file and records what it found.
    fn probe(asset: MediaAsset, path: PathBuf) -> Self {
        let status = status_of(&path);
        Self {
            asset,
            path,
            status,
        }
    }

    /// The kind of media this item holds.
    pub fn kind(&self) -> MediaKind {
        self.asset.kind
    }
}

fn status_of(path: &Path) -> MediaStatus {
    match fs::metadata(path) {
        Ok(metadata) if metadata.is_file() => MediaStatus::Available {
            size: metadata.len(),
        },
        Ok(_) => MediaStatus::Unreadable {
            reason: "not a file".to_owned(),
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => MediaStatus::Missing,
        Err(error) => MediaStatus::Unreadable {
            reason: error.to_string(),
        },
    }
}

/// A size in bytes, rounded to something a person can read.
fn human_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let bytes = bytes as f64;
    if bytes >= GB {
        format!("{:.1} GB", bytes / GB)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes / MB)
    } else if bytes >= KB {
        format!("{:.0} kB", bytes / KB)
    } else {
        format!("{bytes:.0} bytes")
    }
}

/// What an import did, so the user can be told rather than left guessing.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ImportReport {
    /// How many media items were added.
    pub added: usize,
    /// How many files were already in the project.
    pub duplicates: usize,
    /// Files that were not added, with the file name and why.
    pub rejected: Vec<(String, &'static str)>,
}

/// What the project refers to, as seen from the media panel.
#[derive(Debug, Default)]
pub struct Library {
    entries: Vec<MediaEntry>,
    selected: Option<u64>,
}

impl Library {
    /// Media items, in the order the project lists them.
    pub fn entries(&self) -> &[MediaEntry] {
        &self.entries
    }

    /// The selected media item, if one is selected.
    pub fn selected(&self) -> Option<&MediaEntry> {
        let id = self.selected?;
        self.entries.iter().find(|entry| entry.asset.id == id)
    }

    /// How many media files are missing.
    pub fn missing(&self) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry.status == MediaStatus::Missing)
            .count()
    }

    /// Selects a media item. Selection is interface state, so it never dirties the
    /// project.
    pub fn select(&mut self, id: u64) {
        self.selected = Some(id);
    }

    /// Forgets everything, for when no project is open.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.selected = None;
    }

    /// Re-reads every media reference against the filesystem.
    ///
    /// `directory` is the project file's directory, which relative references
    /// resolve against. A project that has never been saved has none, so its
    /// references are absolute.
    pub fn reload(&mut self, project: &Project, directory: Option<&Path>) {
        self.entries = project
            .media
            .iter()
            .map(|asset| MediaEntry::probe(asset.clone(), resolve_stored(directory, &asset.path)))
            .collect();

        if let Some(selected) = self.selected
            && !self.entries.iter().any(|entry| entry.asset.id == selected)
        {
            self.selected = None;
        }
    }

    /// Adds supported media files to `project`.
    ///
    /// Files that are already in the project are skipped rather than added twice,
    /// and files whose extension is not supported are reported back instead of
    /// being silently ignored. Returns how many were added.
    pub fn import(
        &mut self,
        project: &mut Project,
        directory: Option<&Path>,
        paths: &[PathBuf],
    ) -> ImportReport {
        let mut report = ImportReport::default();

        for path in paths {
            let Some(kind) = kind_for_path(path) else {
                report
                    .rejected
                    .push((display_name(path), "unsupported format"));
                continue;
            };

            let stored = store_reference(directory, path);

            if project.media.iter().any(|asset| asset.path == stored) {
                report.duplicates += 1;
                continue;
            }

            let id = next_id(project.media.iter().map(|asset| asset.id));
            project.media.push(MediaAsset {
                id,
                path: stored,
                display_name: display_name(path),
                kind,
            });
            report.added += 1;
        }

        if report.added > 0 {
            self.reload(project, directory);
        }

        report
    }

    /// Points a media item at a new file.
    ///
    /// The item keeps its id, so references to it — clips, from Stage 3 — follow the
    /// relink instead of breaking.
    pub fn relink(
        &mut self,
        project: &mut Project,
        directory: Option<&Path>,
        id: u64,
        replacement: &Path,
    ) -> bool {
        let Some(asset) = project.media.iter_mut().find(|asset| asset.id == id) else {
            return false;
        };

        asset.path = store_reference(directory, replacement);
        asset.display_name = display_name(replacement);
        if let Some(kind) = kind_for_path(replacement) {
            asset.kind = kind;
        }

        self.reload(project, directory);
        true
    }
}

/// Re-expresses every media reference for a new project location.
///
/// Called before a save-as, so a project that is moved to another folder still
/// finds media that travels with it. References are resolved against the old
/// directory first, then stored relative to the new one.
pub fn reexpress_references(project: &mut Project, old: Option<&Path>, new: Option<&Path>) {
    for asset in &mut project.media {
        let resolved = resolve_stored(old, &asset.path);
        asset.path = store_reference(new, &resolved);
    }
}

/// How to store a media reference: relative to the project file when there is one
/// and the media is inside its directory, absolute otherwise.
fn store_reference(project_directory: Option<&Path>, media: &Path) -> String {
    match project_directory {
        Some(directory) => store_path(directory, media),
        None => media.to_string_lossy().into_owned(),
    }
}

/// The next free id, one past the highest in use.
fn next_id(ids: impl Iterator<Item = u64>) -> u64 {
    ids.max().map_or(1, |highest| highest + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project() -> Project {
        Project::new("Reel")
    }

    fn write_media(directory: &Path, name: &str) -> PathBuf {
        let path = directory.join(name);
        fs::write(&path, b"not real media").expect("wrote fixture");
        path
    }

    #[test]
    fn importing_supported_media_records_paths_and_kinds() {
        let directory = tempfile::tempdir().expect("temp dir");
        let video = write_media(directory.path(), "clip.mp4");
        let image = write_media(directory.path(), "still.png");

        let mut project = project();
        let mut library = Library::default();
        let report = library.import(
            &mut project,
            Some(directory.path()),
            &[video.clone(), image.clone()],
        );

        assert_eq!(report.added, 2);
        assert!(report.rejected.is_empty());
        assert_eq!(project.media.len(), 2);
        assert_eq!(library.entries().len(), 2);
        assert_eq!(library.entries()[0].kind(), MediaKind::Video);
        assert!(library.entries()[0].status.is_available());
        assert_eq!(
            project.media[0].path, "clip.mp4",
            "stored relative to the project"
        );
    }

    #[test]
    fn unsupported_files_are_reported_rather_than_ignored() {
        let directory = tempfile::tempdir().expect("temp dir");
        let notes = write_media(directory.path(), "notes.txt");

        let mut project = project();
        let mut library = Library::default();
        let report = library.import(&mut project, Some(directory.path()), &[notes]);

        assert_eq!(report.added, 0);
        assert_eq!(
            report.rejected,
            vec![("notes.txt".to_owned(), "unsupported format")]
        );
        assert!(project.media.is_empty());
    }

    #[test]
    fn importing_the_same_file_twice_adds_it_once() {
        let directory = tempfile::tempdir().expect("temp dir");
        let clip = write_media(directory.path(), "clip.mp4");

        let mut project = project();
        let mut library = Library::default();
        library.import(
            &mut project,
            Some(directory.path()),
            std::slice::from_ref(&clip),
        );
        let report = library.import(&mut project, Some(directory.path()), &[clip]);

        assert_eq!(report.added, 0);
        assert_eq!(report.duplicates, 1);
        assert_eq!(project.media.len(), 1);
    }

    #[test]
    fn a_missing_media_file_is_reported_without_invalidating_the_project() {
        let directory = tempfile::tempdir().expect("temp dir");
        let clip = write_media(directory.path(), "clip.mp4");

        let mut project = project();
        let mut library = Library::default();
        library.import(
            &mut project,
            Some(directory.path()),
            std::slice::from_ref(&clip),
        );
        fs::remove_file(&clip).expect("removed");
        library.reload(&project, Some(directory.path()));

        assert!(
            project.validate().is_ok(),
            "missing media must not break the project"
        );
        assert_eq!(library.entries()[0].status, MediaStatus::Missing);
        assert_eq!(library.missing(), 1);
    }

    #[test]
    fn relinking_a_missing_file_makes_it_available_again() {
        let directory = tempfile::tempdir().expect("temp dir");
        let clip = write_media(directory.path(), "clip.mp4");
        let moved = write_media(directory.path(), "clip-renamed.mp4");

        let mut project = project();
        let mut library = Library::default();
        library.import(
            &mut project,
            Some(directory.path()),
            std::slice::from_ref(&clip),
        );
        fs::remove_file(&clip).expect("removed");
        library.reload(&project, Some(directory.path()));
        let id = project.media[0].id;

        assert!(library.relink(&mut project, Some(directory.path()), id, &moved));

        assert!(library.entries()[0].status.is_available());
        assert_eq!(library.entries()[0].asset.display_name, "clip-renamed.mp4");
        assert_eq!(library.entries()[0].asset.id, id, "relinking keeps the id");
    }

    #[test]
    fn relinking_an_unknown_item_changes_nothing() {
        let directory = tempfile::tempdir().expect("temp dir");
        let clip = write_media(directory.path(), "clip.mp4");
        let mut project = project();
        let mut library = Library::default();
        library.import(
            &mut project,
            Some(directory.path()),
            std::slice::from_ref(&clip),
        );

        assert!(!library.relink(&mut project, Some(directory.path()), 999, &clip));
        assert_eq!(project.media.len(), 1);
    }

    #[test]
    fn ids_start_at_one_and_never_repeat() {
        let directory = tempfile::tempdir().expect("temp dir");
        let mut project = project();
        let mut library = Library::default();
        library.import(
            &mut project,
            Some(directory.path()),
            &[
                write_media(directory.path(), "a.mp4"),
                write_media(directory.path(), "b.mp4"),
            ],
        );

        let ids: Vec<u64> = project.media.iter().map(|asset| asset.id).collect();
        assert_eq!(ids, vec![1, 2]);

        project.media.retain(|asset| asset.id != 1);
        library.import(
            &mut project,
            Some(directory.path()),
            &[write_media(directory.path(), "c.mp4")],
        );

        assert_eq!(project.media.last().expect("imported").id, 3);
    }

    #[test]
    fn saving_somewhere_else_reexpresses_references() {
        let directory = tempfile::tempdir().expect("temp dir");
        let clip = write_media(directory.path(), "clip.mp4");

        let mut project = project();
        let mut library = Library::default();
        library.import(&mut project, None, std::slice::from_ref(&clip));
        assert!(Path::new(&project.media[0].path).is_absolute());

        reexpress_references(&mut project, None, Some(directory.path()));

        assert_eq!(project.media[0].path, "clip.mp4");
    }

    #[test]
    fn selection_is_interface_state() {
        let directory = tempfile::tempdir().expect("temp dir");
        let mut project = project();
        let mut library = Library::default();
        library.import(
            &mut project,
            Some(directory.path()),
            &[write_media(directory.path(), "clip.mp4")],
        );

        library.select(1);

        assert_eq!(library.selected().map(|entry| entry.asset.id), Some(1));
        assert_eq!(project.media.len(), 1, "selecting changes no project data");
    }

    #[test]
    fn closing_a_project_forgets_the_media() {
        let directory = tempfile::tempdir().expect("temp dir");
        let mut project = project();
        let mut library = Library::default();
        library.import(
            &mut project,
            Some(directory.path()),
            &[write_media(directory.path(), "clip.mp4")],
        );

        library.clear();

        assert!(library.entries().is_empty());
        assert!(library.selected().is_none());
    }

    #[test]
    fn sizes_are_reported_in_readable_units() {
        assert_eq!(human_size(512), "512 bytes");
        assert_eq!(human_size(2048), "2 kB");
        assert_eq!(human_size(5 * 1024 * 1024), "5.0 MB");
        assert_eq!(human_size(3 * 1024 * 1024 * 1024), "3.0 GB");
    }
}
