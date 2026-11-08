# FireEffects — Complete Project Blueprint & Contributor Guide

FireEffects is an independent, Rust-first desktop application for professional video editing, compositing, animation, and motion graphics. This README is the shared instruction manual for the project owner and every human or AI contributor. Read it before planning or changing the repository. Follow newer, explicit instructions from the owner when they change the agreed direction.

## 1. The complete product goal

Build a polished desktop editor that lets users:

1. Create, open, save, and organize projects.
2. Import video, audio, images, and other supported media.
3. Arrange clips and layers on a multitrack timeline.
4. Preview edits and navigate precisely through time.
5. Create compositions containing visual layers.
6. Animate properties with keyframes and interpolation.
7. Apply effects and adjustments to clips or layers.
8. Add and edit text and basic graphic elements.
9. Work with audio tracks and volume controls.
10. Export finished work to practical media formats.
11. Customize shortcuts and workspace behavior as the application matures.

The goal is a coherent, usable workflow—not disconnected demo screens. Every visible control must work, be clearly marked as unavailable, or be omitted until implemented.

### Product principles
- Professional capability with approachable interaction.
- Responsive preview and timeline.
- Non-destructive editing wherever practical.
- Reliable project saving and recovery.
- Clear errors and safe failure behavior.
- Performance suitable for real media projects.
- Original FireEffects identity, design, code, and assets.

## Current implementation status

The Stage 1 editor shell is complete. Stage 2 supports creating projects, saving and opening validated versioned `.firefx` files, importing media with its type and size, and relinking media that has moved. Media decoding, autosave, and recovery remain to be implemented. See [the project format specification](docs/PROJECT_FORMAT.md) for the current on-disk schema and limitations.

## 2. End-to-end development roadmap

Implement in stages. Do not attempt every advanced feature at once. Preserve the dependency order unless the owner explicitly changes priorities.

### Stage 0 — Foundation
- Inspect the repository and document what already exists.
- Establish or preserve a working Rust project structure.
- Choose and document desktop UI, media, rendering, and packaging technologies based on repository needs.
- Establish formatting, linting, test, and build commands.
- Make a minimal application that launches reliably.
- Isolate platform-specific behavior where practical.

**Done when:** a contributor can clone, build, run, and understand the project by following its documentation.

### Stage 1 — Editor shell
- Build the main application window and original FireEffects visual identity.
- Create a workspace with media/project panel, preview, timeline, and properties inspector.
- Support resizing and sensible layout behavior.
- Add menus and basic project actions.
- Make selection, focus, and keyboard behavior consistent.
- Do not add decorative controls that pretend to work.

**Done when:** the app opens into a stable editor shell and implemented controls work.

### Stage 2 — Projects and media
- Define a versioned project format.
- Create, open, save, and close projects.
- Import supported media and show useful metadata.
- Display imported media in a project panel.
- Track media references safely and handle moved or missing files.
- Validate project data when opening it.
- Add autosave/recovery only when safe and tested.
- Never silently overwrite or discard a user's project.

**Done when:** a user can import media, save a project, close the app, reopen it, and recover the same state.

### Stage 3 — Timeline editing
- Build a time ruler, playhead, tracks, clips, selection, zoom, and scrolling.
- Place imported media on tracks.
- Implement moving, trimming, splitting, deleting, and arranging clips.
- Add basic track controls.
- Keep time calculations precise and consistent.
- Add undo/redo for editing operations.
- Make snapping and its indicators understandable.
- Ensure mouse, keyboard, and touchpad interactions do not conflict.

**Done when:** users can assemble and revise a simple multi-clip edit without corrupting project state.

### Stage 4 — Preview and playback
- Preview the project at the playhead.
- Implement play, pause, seek, and frame/time navigation.
- Keep preview synchronized with the timeline.
- Show missing-media and decode errors clearly.
- Keep long-running media operations off the UI thread.
- State playback limitations honestly while real-time playback is incomplete.

**Done when:** users can reliably preview and seek to the expected position.

### Stage 5 — Compositing and motion graphics
- Introduce a composition/layer model suited to the architecture.
- Add image, video, text, and shape layers as capabilities are implemented.
- Provide position, scale, rotation, and opacity controls.
- Implement keyframes for supported properties.
- Begin with understandable linear and eased interpolation.
- Add a functional properties inspector and layer ordering.
- Keep rendering deterministic and testable.

**Done when:** users can create a layered composition and animate its properties over time.

### Stage 6 — Effects, titles, and audio
- Establish an extensible effect model and effect stack.
- Implement a small set of reliable foundational effects before expanding the library.
- Support editable text/title elements and basic shapes.
- Support audio tracks, playback, and appropriate mute/volume controls.
- Serialize effect parameters in project files.
- Distinguish implemented effects from placeholders and unsupported effects.

**Done when:** effects, titles, and audio work in real projects and survive save/reopen.

### Stage 7 — Export
- Define and validate export settings.
- Support a practical initial set of formats and quality settings based on the selected media backend.
- Show progress, safe cancellation, and actionable errors.
- Report success only after output is actually produced and validated.
- Do not overwrite an existing output without confirmation.
- Test exports with sample projects and varied media.

**Done when:** users can export a finished project to a playable file and understand limitations.

### Stage 8 — Professional expansion
Only after the core workflow is reliable, consider workspace customization, saved layouts, shortcut editor, advanced keyframe curves, graph editor, masks, transitions, nested compositions, reusable presets, proxy media, cache management, color-management tools, scopes, GPU acceleration, plugin architecture, scripting, and additional codecs/platforms.

A button or prototype does not mean a feature is complete.

## 3. Technical architecture rules

- Rust is the preferred foundation for core logic and performance-sensitive systems.
- Inspect the existing repository before choosing or replacing frameworks and dependencies.
- Do not assume a UI framework, rendering engine, codec stack, or media pipeline before the repository establishes it.
- Separate UI, project model, timeline, media I/O, playback, rendering, effects, and export where practical.
- Use clear subsystem interfaces; avoid unnecessary abstraction.
- Keep long-running work off the UI thread and report progress/errors.
- Use explicit types and error handling. Avoid panics for ordinary user input or file failures.
- Prefer deterministic, testable core logic.
- Explain significant dependency choices and check maintenance, platform support, and licenses.
- Avoid large dependencies for small tasks.

## 4. Project data and reliability

- Use a documented, versioned project format.
- Evolve serialization without silently losing project data.
- Validate malformed or untrusted project files before use.
- Keep source media non-destructive; represent edits as project state.
- Treat saving, autosave, recovery, undo, and redo as data-integrity features.
- Never delete, move, or rewrite user media as a side effect of editing.
- Handle missing files, unsupported codecs, invalid paths, and failed exports with useful messages.
- Add migration logic when persisted project structures change, or document breaking changes during early development.

## 5. Originality, licensing, and IP

FireEffects must remain independent.

- Never copy Adobe or another product's proprietary source code, internal implementation, assets, icons, branding, or proprietary interface artwork.
- Never imply FireEffects is made by, endorsed by, or affiliated with Adobe.
- Familiar editing concepts and configurable keyboard workflows may be implemented independently.
- Check licenses and provenance for every dependency, asset, font, sample, and code contribution.
- Record required notices and attribution.
- Do not add material with unclear licensing or commit secrets, credentials, private keys, or personal data.
- Use original names, visual design, icons, and documentation.

## 6. Adobe .ffx preset compatibility

Investigating Adobe After Effects .ffx preset compatibility is a research goal, not a promise of complete compatibility.

- Research and document what can be safely and reliably understood from the format.
- Do not assume .ffx is a simple, fully documented interchange format.
- Separate file parsing from translating supported properties into FireEffects features.
- Use only test files the project is permitted to inspect and use.
- Report unsupported, ambiguous, or version-specific properties clearly.
- Never claim support for every preset, effect, expression, plugin, or After Effects version.
- Do not copy proprietary Adobe implementation details. Use independently developed code and appropriately licensed references.
- Document tested examples, supported properties, limitations, and failure behavior.
- If full interpretation is infeasible, consider a limited importer or a FireEffects-native preset format.

## 7. Required workflow before every task

Before editing, every contributor must:

1. Read this README and relevant project documentation.
2. Confirm the repository, current branch, and latest commit.
3. Inspect the working tree and all existing uncommitted changes.
4. Read relevant source files, tests, and architecture.
5. Check recent commits and branches for overlapping work.
6. Describe the task, scope, likely files, risks, and verification plan.
7. Stay within the owner's authorization. Do not begin broad implementation without explicit approval.
8. Fetch the latest file contents immediately before editing; never rely on stale copies.

## 8. Branch ownership and collaboration

The repository owner coordinates contributors and authorizes when work begins.

- Main is the stable shared baseline. Do not develop directly on main unless the owner explicitly asks.
- ChatGPT's assigned development branch is chatgpt. ChatGPT must keep its implementation work there.
- Any other contributor must use the branch assigned by the owner.
- Never put one contributor's work on another contributor's branch unless the owner explicitly requests integration.
- Verify the branch before every write. If it does not exist, ask the owner or create it only when authorized.
- Never overwrite, revert, reformat, or discard another contributor's changes.
- Compare recent commits and diffs before integrating work.
- Prefer pull requests for merging into main. The owner decides when to merge.
- Do not force-push, rewrite shared history, delete branches, or merge without explicit authorization.
- Resolve conflicts by understanding both sides and preserving intended behavior.
- If the owner supplies code from another contributor, review it, identify its intended branch/files, check conflicts, and explain concerns before publishing.
- Never claim a push, commit, branch creation, or merge succeeded unless it actually succeeded.

## 9. Implementation rules

- Make small, focused, reviewable changes.
- Preserve existing conventions unless a deliberate improvement is justified.
- Avoid replacing whole files when a targeted edit is safer.
- Do not introduce unrelated refactors or features.
- Remove dead code, debug output, temporary files, and misleading placeholders.
- Never hardcode machine-specific paths, credentials, or personal information.
- Keep user-facing text clear and consistent.
- Update documentation when behavior, setup, architecture, or supported features change.
- Add tests for important logic and regressions.
- If a task cannot be completed safely, stop and report the blocker instead of improvising a risky workaround.

## 10. Testing and verification

Use checks supported by the actual repository. For a Cargo workspace, run applicable commands:

    cargo fmt --all -- --check
    cargo check --workspace
    cargo test --workspace
    cargo clippy --workspace --all-targets --all-features -- -D warnings

Also, as relevant:
- Build and launch the application.
- Test the exact user-facing workflow changed.
- Test invalid input, missing files, and failure paths.
- Verify save/reopen behavior for project-model changes.
- Verify undo/redo for editing operations.
- Validate export output instead of trusting a success message.
- Review the final diff for accidental changes, secrets, generated clutter, and unrelated formatting.
- Report checks that could not run and why.
- Never claim a check passed unless it actually passed.
- Do not disable warnings or tests merely to make checks pass without explaining and justifying the change.

## 11. Definition of done

A task is complete only when:
- Requested behavior is implemented, not merely represented by UI.
- The change fits the architecture and does not break unrelated behavior.
- Relevant tests or manual verification have been performed.
- Formatting, build, and lint checks have been run where possible.
- Documentation has been updated where needed.
- The final diff has been reviewed.
- The change is committed to the correct authorized branch.
- The owner receives a handoff with commit hash and limitations.

If any point is incomplete, report the task as partial or blocked—not complete.

## 12. Required contributor handoff

Every contributor report must include:

- **Task:** what was requested.
- **Summary:** what changed and why.
- **Branch:** branch used.
- **Files:** important files created or modified.
- **Verification:** exact checks run and results.
- **Commit:** commit hash, if created.
- **Limitations:** known gaps, assumptions, and untested behavior.
- **Next step:** most useful follow-up, if any.

Be precise about what is implemented versus planned. Flag uncertainty instead of presenting guesses as facts.

---

## Golden rule

Understand the existing project first. Follow the owner's instructions. Build the complete product in sensible stages. Coordinate branch ownership. Preserve user data and other contributors' work. Use original, properly licensed implementation. Test honestly. Commit only to the correct authorized branch. Report exactly what happened.
