# FireEffects project file format

Status: version 1, introduced with the Stage 2 project-persistence foundation.

## File identity

- Extension: .firefx
- Encoding: UTF-8 JSON
- Format identifier: fireeffects
- Current schema version: 1

A project document has this envelope:

```json
{
  "format": "fireeffects",
  "version": 1,
  "project": {
    "name": "Untitled",
    "settings": {
      "width": 1920,
      "height": 1080,
      "frame_rate_num": 30,
      "frame_rate_den": 1,
      "duration_frames": 150
    },
    "media": [],
    "tracks": []
  }
}
```

This example shows the default empty project. Actual documents contain every
field required by the current model. Enum values use snake_case. Unknown fields
are rejected rather than silently discarded, so a newer project cannot
accidentally be opened and then saved by an older build with data loss.

## Validation and compatibility

The loader checks the format identifier and schema version before decoding the
project. It then validates project invariants: non-empty name, bounded non-zero
resolution, non-zero frame rate and duration, unique IDs, valid media references,
compatible track/media kinds, and clip bounds.

Version 1 has no migration path because it is the first persisted schema. Any
future incompatible schema change must add an explicit migration or fail with a
clear unsupported-version message. Never silently reinterpret a newer version.

## Filesystem behavior

The desktop application reads only native .firefx files and limits project files
to 64 MiB. Saving writes a temporary sibling file and syncs it before replacing
the destination. When replacing a previously saved project, the old file is
retained as a temporary backup until the new file is in place. Save As does not
overwrite a different existing file; choose a new path or open that file first.

Project files store media paths as references. Saving never moves, deletes, or
rewrites source media.

Media inside the project file's directory is stored relative, with `/` separators, so
a project folder can be moved or copied and still find its media. Media anywhere else
is stored as the absolute path the user chose. Saving re-expresses every reference
for the destination, so Save As keeps a project self-contained when its media travels
with it. There is no separate relative flag: an absolute path is its own marker.

## Media references

Media items carry an id, the stored path, a display name, and a kind (video, audio,
or image). Types are classified by file extension only: nothing is decoded, because
the media backend is still Stage 4's decision. A project that lists media this build
cannot decode is still a valid project.

Whether a file is currently present, and its size, are never written to the project
file. They are recomputed when the project is opened, and a media file that has moved
is reported with a warning and can be relinked to a new location without changing its
id.

## Current limitations

- Media is listed but not decoded; preview and metadata such as duration or frame
  size arrive with the media backend in Stage 4.
- Autosave/recovery is intentionally deferred until recovery behaviour and crash
  tests can be run end-to-end. The unsaved-changes question is the interim
  protection.
- Project dialogs accept a typed path; the `Browse…` button uses the platform file
  dialog, which needs a desktop portal on Linux.
