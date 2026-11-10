# Developing FireEffects

This is the practical setup guide. For why the project is built this way, see
[ARCHITECTURE.md](ARCHITECTURE.md); for the roadmap and contribution rules, see the
[README](../README.md).

## Prerequisites

- **Rust** 1.95 or newer, installed with [rustup](https://rustup.rs):
  ```sh
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
  `rust-toolchain.toml` requests the stable channel with `rustfmt` and `clippy`, so
  rustup installs what the checks need automatically.
- **A C compiler** (`build-essential` or equivalent) for the few dependency crates
  that build native code.
- **A GPU driver with Vulkan, Metal, or DX12 support.** The window is created with
  `wgpu`; no OpenGL fallback is configured.

On Debian or Ubuntu, the packages needed to *run* the window are:

```sh
sudo apt install libvulkan1 mesa-vulkan-drivers libxkbcommon0 libwayland-client0 \
  libx11-6 libxcursor1 libxi6 libxrandr2
```

Windows and macOS need no extra packages beyond the toolchain.

## Build, run, test

Run everything from the repository root:

```sh
cargo build --workspace        # compile both crates
cargo run -p fireeffects-app   # launch the application
cargo test --workspace         # unit and doc tests; no display or GPU needed
```

`cargo run` without `-p` also works; the workspace has one binary, named
`fireeffects`.

## Checks required before committing

```sh
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

`cargo fmt --all` applies formatting. If a check cannot run in your environment,
say so in the handoff instead of marking it passed.

## Repository layout

```text
Cargo.toml              workspace manifest, shared metadata and lints
rust-toolchain.toml     toolchain channel and components
crates/core/            headless editing logic and the project schema (no UI, no I/O)
  src/frame_rate.rs       exact rational frame rates and time conversions
  src/workspace.rs        panel visibility and focus rules
  src/input.rs            keys, key chords, commands, and the binding table
  src/project.rs          project, track, clip and media model with validation
  src/project_file.rs     versioned encode/decode of project documents
  src/media.rs            media kinds by extension and media path rules
crates/app/             desktop application; binary is `fireeffects`
  src/main.rs             window options and module wiring
  src/ui.rs               shell layout, command dispatch, status bar
  src/project_io.rs       bounded project loading and safe saving
  src/library.rs          media entries, filesystem status, import and relink
  src/files.rs            native file pickers, and the scripted picker tests use
  src/panels.rs           the four panels and their current contents
  src/menu.rs             menu bar, built from working commands only
  src/dialogs.rs          project dialogs, unsaved-changes question, help windows
  src/keys.rs             egui input to core command mapping
  src/theme.rs            palette and egui style
docs/ARCHITECTURE.md    technology decisions and deferred decisions
docs/DEVELOPMENT.md     this file
```

Adding a crate: add it under `crates/`, list it in `[workspace] members`, inherit
`version`, `edition`, `rust-version`, and `repository` from `[workspace.package]`,
and set `[lints] workspace = true`. Keep the dependency direction pointing at
`core`, never the other way.

## What the application does today (Stages 0-2)

The window opens into the editor shell: a menu bar, four resizable panels (Media,
Preview, Timeline, Inspector), a status bar, and help windows. Every control in it
works:

- `File` → New Project, Open Project, Save, Save As, Close Project, Import Media,
  and Quit. Project commands report themselves in the status bar when nothing is
  open rather than doing nothing.
- Project dialogs accept a typed path and offer a `Browse…` button for the native
  picker.
- `View` shows or hides a panel. Hiding the last visible panel is refused, and the
  status bar explains why.
- Clicking a panel header focuses that panel.
- Imported media is listed with its kind and size; selecting one shows its details
  in the inspector. A file that has moved is shown in the warning colour with a
  `Locate…` button that relinks it.
- Unsaved changes are never discarded without asking, including when the window is
  closed with its own close button.
- `Ctrl+N`/`Ctrl+O`/`Ctrl+S`/`Ctrl+Shift+S`/`Ctrl+I`/`Ctrl+W` for the project and
  media commands, `Ctrl+1`-`Ctrl+4` to show and focus a panel, `F6` and `Shift+F6`
  to cycle focus, `F1` for the shortcut reference, `Ctrl+Q` to quit.

The timeline's first editing controls can add video/audio tracks, place selected
media, move clips in one-second steps, adjust clip duration, delete clips, and mute
tracks. There is still no media decoding, playback preview, direct drag editing,
undo/redo, or export; these are later Stage 3–7 work.

## Project files

A project is one `.firefx` file: UTF-8 JSON with a format marker, a version, and the
project data. `docs/PROJECT_FORMAT.md` is the specification. The practical rules:

- **Never rename or retype a field without a version decision.** The model types in
  `crates/core/src/project.rs` are the file schema.
- **Keep paths portable.** `core::media` decides whether a reference is stored
  relative to the project file or absolute. Use it instead of formatting paths by
  hand; the format has no separate relative flag.
- **Saving must not truncate.** `project_io::write_project` writes a temporary
  sibling, syncs it, and renames it into place. Do not replace it with `fs::write`.
- **Media presence is not project data.** Whether a file is currently there is
  recomputed by `library.rs`, never serialised.

Adding a shortcut means editing `BINDINGS` in `crates/core/src/input.rs`. The
window's key handling, the menus, and the help window all read that table, and a
test fails if a new binding's key is not also watched by `crates/app/src/keys.rs`.

## Testing the interface without a display

`crates/app` uses `egui_kittest` (dev-dependency only) to run the real shell in a
headless `egui::Context`. Tests in `crates/app/src/ui.rs` click menu entries,
click panel headers, and press keys, then check the resulting state. They need no
window, display, or GPU, so they run in CI and in a container.

File dialogs cannot open in a test, so the pickers sit behind the `FilePicker`
trait in `crates/app/src/files.rs`. Tests install a scripted picker that returns
paths they prepared in a temporary directory, then drive the real menus, dialogs,
buttons, and shortcuts. That is how importing, saving, reopening, and relinking are
verified end to end without a desktop.

The harness also fails a test when a widget has no accessible name, which is why the
project dialogs label their text fields.

They do not check appearance. A layout or colour change still needs someone to look
at the running window, and should be reported as visually unverified if nobody has.

## Troubleshooting

- **`Failed to initialize any backend` / `No suitable adapter found`** — the
  machine has no usable GPU backend. Install a Vulkan driver (`mesa-vulkan-drivers`
  on Linux) or run with a software implementation such as `lavapipe`.
- **`error while loading shared libraries` on Linux** — a runtime package from the
  list above is missing.
- **Nothing appears in a container or over SSH** — a window needs a display.
  `cargo build` and `cargo test` work headless; `cargo run` does not.
- **Unexpected `unsafe_code` lint denial** — `unsafe` is denied workspace-wide. If
  a crate genuinely needs it, opt in deliberately and record the justification in
  `docs/ARCHITECTURE.md` rather than weakening the workspace setting.

## Verifying a change

1. The relevant `cargo` checks above pass.
2. For UI changes, launch the app and exercise the exact workflow you touched.
3. For data changes, verify what happens with invalid input and missing files.
4. Read your own diff for unrelated formatting, debug output, and generated files.
5. Report what you verified, and what you could not, in the handoff.
