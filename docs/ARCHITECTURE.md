# FireEffects architecture

Status: **Stage 3 (Timeline editing) in progress**. Projects/media and the first track/clip editing controls are implemented. Timeline ruler/playhead, direct drag editing, undo/redo, preview/playback, and export remain pending. This document records what exists, what was chosen deliberately, and what is intentionally still undecided. Update it in the same change that alters one of those decisions.

## What exists today

- A Cargo workspace with two crates.
- `crates/app`: the editor shell, project dialogs, filesystem-safe project
  persistence, the media library, and native file pickers.
- `crates/core`: platform-independent workspace, input, project, versioned
  project-serialization models, and media path/format rules.

Media is imported, listed, and relinked but not decoded. The timeline can create
tracks, place selected media, move/resize/delete clips, and mute tracks. The File
menu provides New, Open, Save, Save As, Import Media, and Close Project.

## Crate layout and dependency direction

```text
fireeffects-app  ──▶  fireeffects-core
```

- **`fireeffects-core`** holds editing logic that must be testable without a
  display, a GPU, or media files: the time model, the workspace focus model, and
  the command table, project model, and timeline clip/track data today; effect
  parameters arrive in later stages.
- **`fireeffects-app`** owns the window, event loop, theme, and widgets. Modules:

  | Module | Responsibility |
  | --- | --- |
  | `main.rs` | Window options, renderer choice, module wiring. |
  | `ui.rs` | Shell layout, command dispatch, status bar, `eframe::App`. |
  | `panels.rs` | The four panels and what each shows today. Stateless. |
  | `menu.rs` | The menu bar. Returns a command, applies nothing. |
  | `dialogs.rs` | Project dialogs, the unsaved-changes question, shortcut reference, and About. |
  | `project_io.rs` | Bounded project loading and safe project-file replacement. |
  | `library.rs` | Media entries, their on-disk status, import, and relinking. |
  | `files.rs` | File pickers: native dialogs, and the scripted picker tests use. |
  | `keys.rs` | Maps egui input onto the core command table. |
  | `theme.rs` | Palette and the egui style applied at start-up. |
- **Dependency rule:** `core` never depends on `app`, and never on windowing,
  rendering, or codec crates. If a piece of logic needs a GPU context to test, it
  does not belong in `core`.
- **Not created yet, on purpose:** media I/O, renderer, and project-schema crates.
  A crate appears in the stage that first needs it, so the layout stays honest.

## Chosen technologies

### Language and toolchain

Rust stable with edition 2024, minimum supported Rust version 1.95 (the floor set
by `eframe` 0.37). `rust-toolchain.toml` requests `stable` plus `rustfmt` and
`clippy` so contributors get the same tooling without manual setup.

### Desktop UI: `egui` 0.37 through `eframe` 0.37

Chosen because FireEffects is a *canvas* application rather than a form
application: a timeline is clip rectangles, waveforms, and handles painted from
measured time values, and a preview is a textured surface. Immediate-mode `egui`
handles that styling well and gives every contributor a single toolchain — no
webview, no JavaScript build chain, no per-platform UI toolkit.

Alternatives considered and not taken:

| Option | Why not now |
| --- | --- |
| Tauri + web UI | Better widget styling, but adds a second language and JS toolchain, varies by OS webview, and a GPU compositor surface would still be native. |
| Slint | Good declarative tooling, but pixel-level canvas work is more awkward and the licence terms vary by version and tier; needs review before adoption. |
| GTK4 | Genuinely native, but heavy system dependencies and awkward distribution on Windows and macOS. |
| Iced | Sound architecture, but a weaker fit for pixel-level drawing and a faster-moving API than we want for a long-lived editor. |

`egui`/`eframe` are dual-licensed MIT OR Apache-2.0, so they impose no licence
constraint on the project's own choice.

### Rendering: `wgpu` (through `eframe`'s `wgpu` feature)

`main.rs` selects `eframe::Renderer::Wgpu` explicitly instead of relying on a
default. One GPU abstraction then covers UI painting and the future compositor,
with Vulkan, Metal, DX12, and GL backends behind it.

**Deliberately deferred:** the shape of the compositing graph — texture formats,
pass structure, colour management, and rendering at arbitrary output resolutions.
Stage 4 and Stage 5 decide that. This document fixes only the GPU layer, so the
compositor does not also have to justify a second graphics stack.

### Media I/O: not decided yet

Stage 2 imports media by reference: it records where a file is, never what is inside
it. Decoding is needed to preview or to read durations and frame sizes, so the
decision belongs to Stage 4. Criteria for that decision:

1. Accurate seeking, not just sequential playback.
2. Coverage of the formats real editors receive from cameras.
3. Licence compatibility with the project's distribution model (LGPL/GPL terms
   especially, and whether dynamic linking is practical on every target).
4. Usable prebuilt binaries on Windows, macOS, and Linux, or a build story a
   contributor can follow.
5. Safe use off the UI thread.
6. Evidence of active maintenance.

Leading candidates are FFmpeg (broadest coverage, licence complexity),
GStreamer (strong plugin story, heavier deployment), and platform-native decoders
(clean licensing, much less format coverage, roughly three implementations).
Until one is chosen, unsupported media must fail with a clear message rather than
a guess.

### Project file format: version 1 JSON

Native projects use UTF-8 JSON in a `.firefx` file, wrapped in a format identifier
and schema version. The core crate owns serialization and model validation; the
application owns filesystem access and safe replacement. Unknown fields and
unsupported versions are rejected to avoid silent data loss. See
`docs/PROJECT_FORMAT.md` for the on-disk schema and current limitations.

Media references are stored relative to the project file when the media sits inside
the project's directory, and absolute otherwise. Saving re-expresses every
reference for the new location, so a project folder can be moved or copied and still
find its media. `core::media` owns both directions of that rule; the format stores no
absolute-versus-relative flag, because an absolute path is its own marker.

### Packaging and distribution: not decided yet

Decided once there is something worth shipping (after Stage 7). The likely shape is
`cargo-dist` for archives and platform installers, plus code signing and
notarization per platform. Not set up yet.

## Stage 2 media handling

### Import and metadata

`core::media` classifies a file by extension and `library.rs` records it as a media
asset with an id, the stored path, the display name, and the kind. Nothing decodes
the file: the media backend is still Stage 4's decision, and a project that lists
undecodable media is still a valid project. What the panel shows — kind and file
size — is read from the filesystem and shown honestly as such.

The library never guesses. A file with an unrecognised extension is refused with a
reason rather than imported and hoped over, and importing the same path twice adds
it once.

### Missing media

A project can outlive its media. Every reference is resolved against the project
file's directory and probed when the project is opened or changed; anything missing
is shown in the warning colour, counted in the panel summary and the status bar, and
offered a `Locate…` button that rewrites the reference. Relinking keeps the media
id, so references from future clips follow the new file instead of breaking.

A project with missing media still opens, still validates, and still saves. Refusing
to open work because one file moved would be the worst possible behaviour.

Presence is interface state and is never serialised: the project records where media
is *supposed* to be, not whether it is there on this machine.

### Unsaved changes are never discarded silently

New, open, close, and quit all pass through one question when the project differs
from its file, and so does the window's own close button: `ui.rs` inspects
`close_requested` and cancels the close to ask first. The question offers Save,
Discard changes, and Cancel. Dismissing the window means Cancel, because doing
nothing is the safe answer.

A project that has never been saved has nowhere to save to, so Save from that
question opens Save As and drops the pending request rather than repeating it
forever; the user runs the command again once the project has a home.

### Saving

`project_io::write_project` writes a temporary sibling, syncs it, and renames it into
place, keeping the previous file as a backup until the new one is in place. It also
refuses to replace a file that is not the open project unless overwriting is
intended. `ui.rs` rewrites media references on a *copy* of the project, so a failed
save cannot leave the open project describing a location it was never written to.

### Autosave and recovery: deliberately absent

The README allows autosave "only when safe and tested". A recovery feature that
writes without being asked is exactly the kind of feature that corrupts work when it
is half-built: it needs a recovery question, a location policy, and crash tests that
do not exist yet. What is in place instead is the foundation it needs — atomic
saves, a validated format, and an unsaved-changes question.

## The editor shell (Stage 1)

### Regions

```text
┌──────────────────────────────────────────────────┐
│ wordmark   File  View  Help            menu bar   │
├────────────┬─────────────────────┬───────────────┤
│  Media     │      Preview        │   Inspector   │
│            ├─────────────────────┤               │
│            │      Timeline       │               │
├────────────┴─────────────────────┴───────────────┤
│ Focus: Media                F1 for shortcuts     │
└──────────────────────────────────────────────────┘
```

The panels are added in the order media, inspector, timeline, preview, because
egui gives each panel a slice of what is left. That order is what keeps the
timeline spanning only the middle column instead of the whole window. Every panel
is resizable with a minimum width or height, so a layout cannot be dragged into
an unusable state.

### Focus, selection, and keyboard

One panel holds focus at a time, shown by an ember title and reported in the
status bar. Focusing changes nothing else yet, because there is nothing else to act
on; the model exists so Stage 3 has one place to ask which panel is active instead
of inventing a second notion later.

The rules live in `core::workspace`, not in the widgets: a hidden panel cannot take
focus, hiding the focused panel moves focus to the next visible one, and the last
visible panel cannot be hidden (the status bar says so when it is attempted). They
are unit-tested without a window.

Keyboard shortcuts are declared once, in `core::input::BINDINGS`:

| Chord | Command |
| --- | --- |
| `Ctrl+1` … `Ctrl+4` | Show the Media, Preview, Timeline, or Inspector panel and focus it |
| `F6` / `Shift+F6` | Focus the next / previous visible panel |
| `Ctrl+N` / `Ctrl+O` | New project / open a project file |
| `Ctrl+S` / `Ctrl+Shift+S` | Save / save as |
| `Ctrl+I` | Import media |
| `Ctrl+W` | Close the project |
| `F1` | Open the shortcut reference |
| `Ctrl+Q` | Quit |

`Ctrl` means Command on macOS: the window maps egui's cross-platform `command`
modifier flag onto the shell's control modifier, so one table serves every
platform. The menu, the shortcut reference, and the key handler all read the same
table, and a test asserts no two bindings share a chord. `View` menu visibility
toggles deliberately have no chord of their own: `Ctrl+1`–`Ctrl+4` show and focus,
whereas the menu toggles visibility, and showing both in a menu would imply the key
does something it does not.

### Command pipeline

Menus, panel headers, and key presses all produce a `core::input::Action`. `ui.rs`
has exactly one `apply` method that mutates state for a command. There is no second
path that changes the workspace from a widget, which is how a menu and a shortcut
are kept from drifting apart.

### What is deliberately not in the shell

No placeholder controls. `File` holds the project and media commands that work, the
Media panel's import button imports, and the panels that have no features say which
stage brings them. There are still no transport buttons, no timeline ruler, and no
settings dialog, because none of them could work yet: a disabled-looking entry would
imply the feature is one release away and that its shape is settled, which is not
true.

### Identity

The palette is defined in one module: ember `#E87A3D` as the accent, deep ember for
selection, charcoal `#101115` canvas, `#1A1C22` panels, `#0C0D10` chrome, and warm
off-white text. The wordmark is set in type; there are no icon or artwork assets
yet, so there is nothing whose licensing needs checking. No Adobe asset, colour-coded
layout, or brand mark is imitated.

### Layout persistence

Panel sizes survive within a session but not across restarts, because persistence
needs eframe's `persistence` feature and a decision about where user state lives.
Saved workspaces and a shortcut editor are Stage 8 ("workspace behaviour"), so this
is deferred rather than forgotten.

## Threading and responsiveness

The UI thread never blocks on file I/O, decoding, encoding, or long computation.
When the first long operation lands, use one pattern so the codebase stays
predictable: run the work on a worker thread, report progress and completion over a
channel, and have the UI poll that channel and request a repaint. UI state is only
mutated on the UI thread.

## Error handling

- Core returns `Result<T, fireeffects_core::Error>`. No panics for user input or
  file failures; arithmetic that could overflow saturates instead.
- Enums that describe failures are `#[non_exhaustive]`, so new cases can be added
  without breaking callers.
- `unsafe` is denied workspace-wide by lint. A crate that needs it must opt in
  explicitly and record the justification here.

## Testing strategy

- **Core:** unit tests plus doc tests, all runnable headless — `cargo test
  --workspace` needs no display and no GPU.
- **App:** the shell's interactions are tested headlessly with `egui_kittest`, a
  dev-only dependency (MIT OR Apache-2.0). It runs the real `draw` method in a
  real `egui::Context` without a window or GPU: the tests click menu items and
  panel headers and press keys, then assert on the resulting state and on the
  accessibility tree. That is what lets "implemented controls work" be verified
  in CI rather than by eye. What it cannot check is what the window *looks* like;
  visual appearance still needs a human or a screenshot review.
- **App:** behaviour that can be tested belongs in `core` even when the app could
  test it, which is why the focus rules and the binding table live there.
- **App, project and media behaviour:** the file pickers sit behind the
  `FilePicker` trait, so tests install a scripted picker and drive the real menu
  items, dialogs, buttons, and shortcuts against a temporary directory. That is how
  "import, save, close, reopen, recover the same state" is verified without a
  desktop: the tests write a real project file, remove a real media file, and reopen
  both. The format has round-trip tests, and each refusal has its own test — wrong
  marker, newer version, invalid data, unparseable text.
- **Accessibility:** the headless harness fails a test when a widget has no
  accessible name, which is why the project dialogs label their text fields. That
  check is worth keeping: an unnamed field is unusable with a screen reader.
- **Later:** export output validation rather than trusting a success message.

## Open decisions for the owner

1. **Project licence.** There is no `LICENSE` file yet. Every dependency so far is
   permissive (MIT OR Apache-2.0), so no choice is forced.
2. **Media backend.** See the criteria above; this decides what Stage 4 can preview.
3. **Compositing approach.** Whether `wgpu` used directly is enough, or a dedicated
   renderer crate is warranted at Stage 5.
4. **Platform targets.** Whether the first release supports Windows, macOS, and
   Linux simultaneously or starts with one.
