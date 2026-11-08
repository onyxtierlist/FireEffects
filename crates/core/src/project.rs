//! Core, renderer-independent project data model.
//!
//! Timeline positions are stored as integer frames rather than floating-point
//! seconds so edits remain deterministic at every supported frame rate.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub name: String,
    pub settings: ProjectSettings,
    pub media: Vec<MediaAsset>,
    pub tracks: Vec<Track>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectSettings {
    pub width: u32,
    pub height: u32,
    /// Frame rate numerator. For example, 30/1 or 30000/1001.
    pub frame_rate_num: u32,
    /// Frame rate denominator.
    pub frame_rate_den: u32,
    /// Intended composition length, measured in frames.
    pub duration_frames: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaKind {
    Video,
    Audio,
    Image,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaAsset {
    pub id: u64,
    /// Path as selected by the user. A future project loader will resolve
    /// relative paths against the project file's directory.
    pub path: String,
    pub display_name: String,
    pub kind: MediaKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackKind {
    Video,
    Audio,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Track {
    pub id: u64,
    pub name: String,
    pub kind: TrackKind,
    pub muted: bool,
    pub clips: Vec<Clip>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clip {
    pub id: u64,
    pub media_id: u64,
    /// Position on the timeline, in frames from the composition start.
    pub start_frame: u64,
    /// In-point within the source media, in frames.
    pub source_in_frame: u64,
    /// Visible clip length, in timeline frames.
    pub duration_frames: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectValidationError {
    EmptyProjectName,
    InvalidResolution,
    InvalidFrameRate,
    InvalidProjectDuration,
    DuplicateMediaId(u64),
    DuplicateTrackId(u64),
    DuplicateClipId(u64),
    EmptyMediaPath(u64),
    MissingMediaForClip { clip_id: u64, media_id: u64 },
    InvalidClipDuration(u64),
    ClipOutsideProject { clip_id: u64 },
    ClipOnIncompatibleTrack { clip_id: u64 },
    TimelinePositionOverflow(u64),
}

impl Project {
    /// Create a new 1080p, 30 fps, five-second composition.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            settings: ProjectSettings {
                width: 1920,
                height: 1080,
                frame_rate_num: 30,
                frame_rate_den: 1,
                duration_frames: 150,
            },
            media: Vec::new(),
            tracks: Vec::new(),
        }
    }

    /// Check project invariants before saving, importing, or rendering.
    pub fn validate(&self) -> Result<(), ProjectValidationError> {
        if self.name.trim().is_empty() {
            return Err(ProjectValidationError::EmptyProjectName);
        }

        if self.settings.width == 0
            || self.settings.height == 0
            || self.settings.width > 16_384
            || self.settings.height > 16_384
        {
            return Err(ProjectValidationError::InvalidResolution);
        }

        if self.settings.frame_rate_num == 0 || self.settings.frame_rate_den == 0 {
            return Err(ProjectValidationError::InvalidFrameRate);
        }

        if self.settings.duration_frames == 0 {
            return Err(ProjectValidationError::InvalidProjectDuration);
        }

        let mut media_ids = Vec::with_capacity(self.media.len());
        for media in &self.media {
            if media.path.trim().is_empty() {
                return Err(ProjectValidationError::EmptyMediaPath(media.id));
            }
            if media_ids.contains(&media.id) {
                return Err(ProjectValidationError::DuplicateMediaId(media.id));
            }
            media_ids.push(media.id);
        }

        let mut track_ids = Vec::with_capacity(self.tracks.len());
        let mut clip_ids = Vec::new();

        for track in &self.tracks {
            if track_ids.contains(&track.id) {
                return Err(ProjectValidationError::DuplicateTrackId(track.id));
            }
            track_ids.push(track.id);

            for clip in &track.clips {
                if clip_ids.contains(&clip.id) {
                    return Err(ProjectValidationError::DuplicateClipId(clip.id));
                }
                clip_ids.push(clip.id);

                if !media_ids.contains(&clip.media_id) {
                    return Err(ProjectValidationError::MissingMediaForClip {
                        clip_id: clip.id,
                        media_id: clip.media_id,
                    });
                }

                if clip.duration_frames == 0 {
                    return Err(ProjectValidationError::InvalidClipDuration(clip.id));
                }

                let end_frame = clip
                    .start_frame
                    .checked_add(clip.duration_frames)
                    .ok_or(ProjectValidationError::TimelinePositionOverflow(clip.id))?;

                if end_frame > self.settings.duration_frames {
                    return Err(ProjectValidationError::ClipOutsideProject { clip_id: clip.id });
                }

                let media_kind = self
                    .media
                    .iter()
                    .find(|media| media.id == clip.media_id)
                    .map(|media| media.kind);

                let compatible = matches!(
                    (track.kind, media_kind),
                    (TrackKind::Video, Some(MediaKind::Video | MediaKind::Image))
                        | (TrackKind::Audio, Some(MediaKind::Audio | MediaKind::Video))
                );

                if !compatible {
                    return Err(ProjectValidationError::ClipOnIncompatibleTrack {
                        clip_id: clip.id,
                    });
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_project() -> Project {
        let mut project = Project::new("Sample");
        project.media.push(MediaAsset {
            id: 1,
            path: "media/intro.mp4".into(),
            display_name: "intro.mp4".into(),
            kind: MediaKind::Video,
        });
        project.tracks.push(Track {
            id: 1,
            name: "Video 1".into(),
            kind: TrackKind::Video,
            muted: false,
            clips: vec![Clip {
                id: 1,
                media_id: 1,
                start_frame: 0,
                source_in_frame: 0,
                duration_frames: 90,
            }],
        });
        project
    }

    #[test]
    fn new_project_has_valid_defaults() {
        let project = Project::new("Untitled");
        assert_eq!(project.settings.width, 1920);
        assert_eq!(project.settings.height, 1080);
        assert_eq!(project.settings.frame_rate_num, 30);
        assert_eq!(project.settings.duration_frames, 150);
        assert!(project.validate().is_ok());
    }

    #[test]
    fn valid_project_with_video_clip_passes() {
        assert!(sample_project().validate().is_ok());
    }

    #[test]
    fn rejects_clips_that_reference_missing_media() {
        let mut project = sample_project();
        project.tracks[0].clips[0].media_id = 999;

        assert_eq!(
            project.validate(),
            Err(ProjectValidationError::MissingMediaForClip {
                clip_id: 1,
                media_id: 999,
            })
        );
    }

    #[test]
    fn rejects_clips_that_extend_past_project_duration() {
        let mut project = sample_project();
        project.tracks[0].clips[0].duration_frames = 151;

        assert_eq!(
            project.validate(),
            Err(ProjectValidationError::ClipOutsideProject { clip_id: 1 })
        );
    }

    #[test]
    fn rejects_audio_on_a_video_only_track() {
        let mut project = sample_project();
        project.media[0].kind = MediaKind::Audio;

        assert_eq!(
            project.validate(),
            Err(ProjectValidationError::ClipOnIncompatibleTrack { clip_id: 1 })
        );
    }
}
