# FireEffects — AI Contributor Guide

FireEffects is an independent, Rust-first desktop application for professional video editing, compositing, and motion graphics. The goal is a capable, polished creative tool with an efficient workflow and original implementation.

This document is the shared working agreement for human and AI contributors, including ChatGPT, Claude, Freebuff, and other invited agents.

## 1. Project vision

- Build a professional editor for video, compositing, animation, and motion graphics.
- Prioritize a responsive, reliable desktop experience and a clear, consistent interface.
- Use familiar creative-workflow concepts where useful while making FireEffects its own product.
- Treat quality, usability, maintainability, and stability as core requirements.

## 2. Technical direction

- Rust is the preferred foundation for core application logic and performance-sensitive systems.
- Inspect the existing architecture before choosing frameworks, dependencies, or patterns.
- Follow repository conventions unless there is a clear, documented reason to change them.
- Keep modules focused and interfaces understandable.
- Avoid large dependencies for small tasks; explain significant dependency choices.
- Do not assume a UI framework, rendering engine, or media pipeline until the repository establishes it.

## 3. Originality, licensing, and intellectual property

FireEffects must remain an independent project.

- Do not copy Adobe or another product's proprietary source code, internal implementation, assets, icons, branding, or proprietary interface artwork.
- Do not imply that FireEffects is made by, endorsed by, or affiliated with Adobe.
- Familiar editing concepts and user-configurable keyboard workflows may be implemented independently.
- Check the license and provenance of every third-party dependency, asset, font, sample, and code contribution.
- Do not add material with unclear licensing or commit secrets, credentials, private keys, or personal data.

## 4. Adobe .ffx preset compatibility

.ffx compatibility is an investigation and possible limited-compatibility goal, not a promise of complete After Effects compatibility.

- First research and document what can be safely and reliably understood from the format.
- Do not claim support for every preset, effect, expression, plugin, or After Effects version.
- Separate format parsing from translating supported properties into FireEffects features.
- Report unsupported or ambiguous properties clearly rather than silently producing misleading results.
- Do not copy proprietary Adobe implementation details. Use independently developed code and appropriately licensed references.
- Document tested examples, supported properties, limitations, and failure behavior.

## 5. Required workflow before changing code

Before making any change:

1. Confirm the repository and current branch.
2. Inspect the working tree and recent commits.
3. Read relevant source files, project documentation, and existing tests.
4. Identify whether another contributor may already be working on the same area.
5. State the intended scope and files likely to change.
6. Preserve unrelated user and contributor changes.

Never assume the repository is unchanged since a previous session. Re-fetch the latest content before editing.

## 6. Multi-AI collaboration and Git practices

Multiple AI contributors may work in this repository. Coordinate rather than competing.

- Treat commits from other contributors as intentional work; do not overwrite, revert, or reformat unrelated changes.
- Check recent commits, branches, and pull requests before starting overlapping work.
- Prefer a focused task branch and pull request for non-trivial work. Work directly on the default branch only when the repository owner explicitly directs it and the change is appropriately scoped.
- Keep each change small and focused so another contributor can review it.
- If two agents work on the same files or feature, coordinate ownership and integrate deliberately.
- Do not force-push, rewrite shared history, or delete another contributor's branch without explicit authorization.
- Resolve conflicts by understanding both changes, not by automatically choosing one side.
- Commit completed changes with a concise, meaningful message. Do not create empty commits.
- Do not claim a change was committed, pushed, tested, or merged unless the operation actually succeeded.

## 7. Rust quality and verification

Use checks supported by the current repository. For a Cargo workspace, run the applicable commands:

    cargo fmt --all -- --check
    cargo check --workspace
    cargo test --workspace
    cargo clippy --workspace --all-targets --all-features -- -D warnings

- If a command cannot run, report why; do not imply it passed.
- Add or update tests for behavior changes where practical.
- Keep error handling explicit and avoid swallowing failures.
- Review changes for regressions, unnecessary complexity, and platform-specific assumptions.
- Do not disable warnings or tests merely to make a check pass without explaining and justifying the change.

## 8. Architecture and implementation principles

- Keep UI, application state, media processing, rendering, and file-format concerns separated where the architecture allows.
- Prefer explicit data models and predictable state transitions.
- Design long-running media operations so the interface can remain responsive.
- Consider project-file integrity, autosave/recovery, and safe handling of user media.
- Avoid destructive edits to user projects or source media.
- Make unsupported formats and features fail visibly and safely.
- Document important architectural decisions and meaningful trade-offs.

## 9. Product and UX quality

- Keep controls consistent and discoverable.
- Provide clear feedback for loading, progress, errors, and unavailable features.
- Consider keyboard navigation, accessibility, and different display sizes.
- Avoid placeholder controls that look functional but do nothing.
- Prefer a complete, reliable workflow over a large collection of unfinished features.
- Do not add unrelated features to a narrowly scoped task.

## 10. Communication and handoff

Every contributor handoff should include:

- **Summary:** what changed and why.
- **Files:** important files created or modified.
- **Verification:** exact checks run and whether they passed.
- **Commit:** commit hash and branch, if a commit was created.
- **Limitations:** known gaps, assumptions, or untested behavior.
- **Next steps:** the most useful follow-up, if any.

Be precise about what is implemented versus planned. Flag uncertainty instead of presenting guesses as facts.

## Golden rule

Inspect first. Coordinate with other contributors. Make focused, original changes. Test honestly. Commit completed work. Report exactly what happened.
