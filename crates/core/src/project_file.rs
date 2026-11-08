//! Versioned, validated serialization for native FireEffects project files.
//!
//! The file is UTF-8 JSON with a .firefx extension. The envelope lets the
//! application reject unknown format versions before interpreting project data.

use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{Project, ProjectValidationError};

/// Extension used by native FireEffects project files, without a leading dot.
pub const PROJECT_FILE_EXTENSION: &str = "firefx";
/// Stable identifier stored in every native project file.
pub const PROJECT_FORMAT_ID: &str = "fireeffects";
/// Current on-disk schema version.
pub const PROJECT_FORMAT_VERSION: u32 = 1;

#[derive(Serialize)]
struct ProjectDocument<'a> {
    format: &'static str,
    version: u32,
    project: &'a Project,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectHeader {
    format: String,
    version: u32,
    project: Value,
}

/// Failures that can occur while encoding or decoding a project document.
#[derive(Debug)]
pub enum ProjectFileError {
    InvalidDocument(String),
    UnsupportedFormat(String),
    UnsupportedVersion(u32),
    InvalidProject(ProjectValidationError),
}

impl fmt::Display for ProjectFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDocument(message) => write!(f, "invalid project document: {message}"),
            Self::UnsupportedFormat(format) => write!(f, "unsupported project format: {format}"),
            Self::UnsupportedVersion(version) => write!(f, "project version {version} is not supported"),
            Self::InvalidProject(error) => write!(f, "project data failed validation: {error:?}"),
        }
    }
}

impl std::error::Error for ProjectFileError {}

/// Encode a validated project as a pretty-printed native project document.
pub fn encode_project(project: &Project) -> Result<Vec<u8>, ProjectFileError> {
    project.validate().map_err(ProjectFileError::InvalidProject)?;
    let document = ProjectDocument {
        format: PROJECT_FORMAT_ID,
        version: PROJECT_FORMAT_VERSION,
        project,
    };
    serde_json::to_vec_pretty(&document)
        .map_err(|error| ProjectFileError::InvalidDocument(error.to_string()))
}

/// Decode a native project document and validate it before returning the model.
pub fn decode_project(bytes: &[u8]) -> Result<Project, ProjectFileError> {
    let header: ProjectHeader = serde_json::from_slice(bytes)
        .map_err(|error| ProjectFileError::InvalidDocument(error.to_string()))?;
    if header.format != PROJECT_FORMAT_ID {
        return Err(ProjectFileError::UnsupportedFormat(header.format));
    }
    if header.version != PROJECT_FORMAT_VERSION {
        return Err(ProjectFileError::UnsupportedVersion(header.version));
    }
    let project: Project = serde_json::from_value(header.project)
        .map_err(|error| ProjectFileError::InvalidDocument(error.to_string()))?;
    project.validate().map_err(ProjectFileError::InvalidProject)?;
    Ok(project)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Clip, MediaAsset, MediaKind, Track, TrackKind};

    fn sample_project() -> Project {
        let mut project = Project::new("Round trip");
        project.media.push(MediaAsset {
            id: 7,
            path: "media/clip.mp4".into(),
            display_name: "clip.mp4".into(),
            kind: MediaKind::Video,
        });
        project.tracks.push(Track {
            id: 2,
            name: "Video 1".into(),
            kind: TrackKind::Video,
            muted: false,
            clips: vec![Clip {
                id: 3,
                media_id: 7,
                start_frame: 12,
                source_in_frame: 4,
                duration_frames: 60,
            }],
        });
        project
    }

    #[test]
    fn project_round_trips_without_losing_data() {
        let original = sample_project();
        let bytes = encode_project(&original).expect("valid project should encode");
        assert_eq!(decode_project(&bytes).expect("document should decode"), original);
    }

    #[test]
    fn rejects_another_format_identifier() {
        let bytes = br#"{"format":"other","version":1,"project":{}}"#;
        assert!(matches!(decode_project(bytes), Err(ProjectFileError::UnsupportedFormat(format)) if format == "other"));
    }

    #[test]
    fn rejects_unsupported_schema_versions() {
        let bytes = br#"{"format":"fireeffects","version":99,"project":{}}"#;
        assert!(matches!(decode_project(bytes), Err(ProjectFileError::UnsupportedVersion(99))));
    }

    #[test]
    fn rejects_malformed_json() {
        assert!(matches!(decode_project(b"{not json"), Err(ProjectFileError::InvalidDocument(_))));
    }

    #[test]
    fn rejects_invalid_project_data() {
        let mut project = sample_project();
        project.settings.width = 0;
        let document = ProjectDocument {
            format: PROJECT_FORMAT_ID,
            version: PROJECT_FORMAT_VERSION,
            project: &project,
        };
        let bytes = serde_json::to_vec(&document).expect("test document should encode");
        assert!(matches!(
            decode_project(&bytes),
            Err(ProjectFileError::InvalidProject(ProjectValidationError::InvalidResolution))
        ));
    }
}
