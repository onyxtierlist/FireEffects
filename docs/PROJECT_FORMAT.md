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
rewrites source media. Media import and missing-media relinking are subsequent
Stage 2 work.

## Current limitations

- Paths are entered directly in project dialogs; native file pickers are not
  included yet.
- Autosave/recovery is intentionally deferred until dirty-state tracking and
  crash-safe recovery behavior can be tested end-to-end.
- There is no media importer yet; projects can persist model data, but the UI
  does not yet create media entries.
