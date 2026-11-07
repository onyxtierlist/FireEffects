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
crates/app/             desktop application; binary is `fireeffects`
docs/ARCHITECTURE.md    technology decisions and deferred decisions
docs/DEVELOPMENT.md     this file
```

Adding a crate: add it under `crates/`, list it in `[workspace] members`, inherit
`version`, `edition`, `rust-version`, and `repository` from `[workspace.package]`,
and set `[lints] workspace = true`. Keep the dependency direction pointing at
`core`, never the other way.

## What Stage 0 does and does not do

The application opens one window and reports which stages are implemented. It has
no menus, no project, no media import, no timeline, no preview, and no export. That
is the intended Stage 0 scope: a foundation a contributor can clone, build, run,
and understand.

The window currently has a single status screen; the editor shell (panels, menus,
selection, keyboard behaviour) is Stage 1.

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
