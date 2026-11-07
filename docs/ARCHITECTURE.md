# FireEffects architecture

Status: **Stage 0 (Foundation)**. Last reviewed with the repository at the Stage 0
commit. This document records what exists, what was chosen deliberately, and what
is intentionally still undecided. Update it in the same change that alters one of
those decisions.

## What exists today

- A Cargo workspace with two crates and no application features.
- `crates/app`: opens a native window showing an honest status read-out.
- `crates/core`: platform-independent types with no third-party dependencies.

There is no project format, no media pipeline, no compositor, and no exporter. The
UI does not draw controls for any of those, because they do not work yet.

## Crate layout and dependency direction

```text
fireeffects-app  ──▶  fireeffects-core
```

- **`fireeffects-core`** holds editing logic that must be testable without a
  display, a GPU, or media files: the time model today; project, timeline, and
  effect-parameter models as those stages land.
- **`fireeffects-app`** owns the window, event loop, theme, and panels.
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

Nothing decodes media in Stage 0, and the decision belongs to the stage that first
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

### Project file format: not decided yet

Stage 2 decides the format. The constraints are already fixed by the README:
documented, versioned, validated on load, migration path for persisted structures,
and never destructive to user media. No serialization schema is committed to
before the model's shape is known.

### Packaging and distribution: not decided yet

Decided once there is something worth shipping (after Stage 7). The likely shape is
`cargo-dist` for archives and platform installers, plus code signing and
notarization per platform. Not set up in Stage 0.

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
- **App:** kept thin on purpose. Behaviour that can be tested belongs in `core`;
  a UI feature that cannot be checked without a human being is written carefully
  and reported as manually verified or unverified, never as tested.
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
