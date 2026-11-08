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
crates/core/            headless editing logic (no UI, no I/O, no dependencies)
  src/frame_rate.rs       exact rational frame rates and time conversions
  src/workspace.rs        panel visibility and focus rules
  src/input.rs            keys, key chords, commands, and the binding table
crates/app/             desktop application; binary is `fireeffects`
  src/main.rs             window options and module wiring
  src/ui.rs               shell layout, command dispatch, status bar
  src/panels.rs           the four panels and their current contents
  src/menu.rs             menu bar, built from working commands only
  src/dialogs.rs          keyboard-shortcut reference and About
  src/keys.rs             egui input to core command mapping
  src/theme.rs            palette and egui style
docs/ARCHITECTURE.md    technology decisions and deferred decisions
docs/DEVELOPMENT.md     this file
```

Adding a crate: add it under `crates/`, list it in `[workspace] members`, inherit
`version`, `edition`, `rust-version`, and `repository` from `[workspace.package]`,
and set `[lints] workspace = true`. Keep the dependency direction pointing at
`core`, never the other way.

## What Stage 1 does and does not do

The application opens into the editor shell: a menu bar, four resizable panels
(Media, Preview, Timeline, Inspector), a status bar showing which panel has focus,
and two help windows. Every control in it works:

- `View` shows or hides a panel. Hiding the last visible panel is refused, and the
  status bar explains why.
- Clicking a panel header focuses that panel.
- `Ctrl+1`-`Ctrl+4` show a panel and focus it, `F6` and `Shift+F6` cycle focus,
  `F1` opens the shortcut reference, `Ctrl+Q` quits.
- `Help` opens the shortcut reference and About.

There is still no project, no media import, no timeline, no preview, and no export;
each panel says which stage brings its features. That is the intended Stage 1
scope.

Adding a shortcut means editing `BINDINGS` in `crates/core/src/input.rs`. The
window's key handling, the menus, and the help window all read that table, and a
test fails if a new binding's key is not also watched by `crates/app/src/keys.rs`.

## Testing the interface without a display

`crates/app` uses `egui_kittest` (dev-dependency only) to run the real shell in a
headless `egui::Context`. Tests in `crates/app/src/ui.rs` click menu entries,
click panel headers, and press keys, then check the resulting state. They need no
window, display, or GPU, so they run in CI and in a container.

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
