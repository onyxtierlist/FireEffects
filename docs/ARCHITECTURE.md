# FireEffects architecture

Status: **Stage 2 (Projects and media), project-persistence foundation implemented**. The native versioned project format and create/open/save workflow are in place; media import is still pending. This document records what exists, what was chosen deliberately, and what is intentionally still undecided. Update it in the same change that alters one of those decisions.

## What exists today

- A Cargo workspace with two crates.
- `crates/app`: the editor shell, project dialogs, and filesystem-safe project
  persistence.
- `crates/core`: platform-independent workspace, input, project, and versioned
  project-serialization models.

Media import, the compositor, and export are not implemented yet. The File menu
now provides working New, Open, Save, and Save As project actions.

## Crate layout and dependency direction

```text
fireeffects-app  ──▶  fireeffects-core
```

- **`fireeffects-core`** holds editing logic that must be testable without a
  display, a GPU, or media files: the time model, the workspace focus model, and
  the command table today; project, timeline, and effect-parameter models as those
  stages land.
- **`fireeffects-app`** owns the window, event loop, theme, and widgets. Modules:

  | Module | Responsibility |
  | --- | --- |
  | `main.rs` | Window options, renderer choice, module wiring. |
  | `ui.rs` | Shell layout, command dispatch, status bar, `eframe::App`. |
  | `panels.rs` | The four panels and what each shows today. Stateless. |
  | `menu.rs` | The menu bar. Returns a command, applies nothing. |
  | `dialogs.rs` | Project dialogs, keyboard-shortcut reference, and About windows. |
  | `project_io.rs` | Bounded project loading and safe project-file replacement. |
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

Nothing decodes media yet, and the decision belongs to the stage that first
imports a file (Stage 2) or previews one (Stage 4). Criteria for that decision:

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

### Packaging and distribution: not decided yet

Decided once there is something worth shipping (after Stage 7). The likely shape is
`cargo-dist` for archives and platform installers, plus code signing and
notarization per platform. Not set up yet.

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

No placeholder menu items: `File` contains only Quit. New, Open, and Save appear
when the project format exists in Stage 2, and the Media panel says so. There are no
transport buttons, no timeline ruler, no import button, and no settings dialog,
because none of them could work yet. A disabled-looking entry would imply the
feature is one release away and that its shape is settled, which is not true.

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
- **Later:** round-trip tests for project save/load, and export output validation
  rather than trusting a success message.

## Open decisions for the owner

1. **Project licence.** There is no `LICENSE` file yet. Every dependency so far is
   permissive (MIT OR Apache-2.0), so no choice is forced.
2. **Media backend.** See the criteria above; this decides what Stage 2 can import.
3. **Compositing approach.** Whether `wgpu` used directly is enough, or a dedicated
   renderer crate is warranted at Stage 5.
4. **Platform targets.** Whether the first release supports Windows, macOS, and
   Linux simultaneously or starts with one.
