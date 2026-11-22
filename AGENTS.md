# CraftKit — agent and contributor guide

CraftKit is the shared, domain-agnostic foundation for native Rust desktop
applications. It must stay generic: no drawing, audio, GIS, CAD or document
schema leaks into this repository.

## Environment

Cloud Shell workspaces are constrained. Never build large artifacts in `$HOME`.

```sh
export CARGO_HOME=/tmp/cargo-home
export RUSTUP_HOME=/tmp/rustup-home
export PATH=/tmp/cargo-home/bin:$PATH
export CARGO_TARGET_DIR=/tmp/cargo-target
```

- Light checks: `cargo check -p craftkit-core -p craftkit-io -p craftkit-jobs`.
- Full release builds are delegated to GitHub Actions.
- Keep `target/`, archives and `squashfs-root/` out of Git.

## Workspace

```text
crates/craftkit-core   History<T>, ProjectEnvelope<T>, format validation
crates/craftkit-io     Atomic JSON, DocumentHost<T>, AppPaths, settings, recents
crates/craftkit-jobs   Background jobs, progress, cancellation
crates/craftkit-ui     egui shell, theme, CLI, dialogs, run_native
apps/craftkit-demo     Reference application; source used by the generator
templates/             Standalone application scaffolding
tools/                 new-app generator
```

Dependency direction:

```text
craftkit-core <- craftkit-io <- craftkit-ui <- application
craftkit-core <- craftkit-jobs
```

`craftkit-core` must not depend on egui, filesystem, network or threads.

## API rules

- `History<T>` snapshots are bounded and require `T: Clone + PartialEq`.
- `DocumentHost<T>` owns the path, dirty state and history; mutations go through
  `apply` or `history_mut`.
- Project files use `ProjectEnvelope<T>` with `format` and `schema_version`.
  Reject a newer schema; migrate an older one in the application.
- `AppShell` owns presentation only. It never parses an application format.
- `JobRunner<T>` is application-owned; the shell only renders `JobInfo`.
- No `unsafe`. No `unwrap`/`expect` on external data paths.
- Public API has doc comments.

## Tests

- History: undo/redo, bound, dirty transitions, branch truncation.
- IO: round-trip, wrong format, newer schema, atomic save, settings, recents.
- Jobs: completion, progress, cancellation.
- UI: pure helpers only; visual smoke tests via Xvfb.

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Releases

- Annotated `vX.Y.Z` tag after CI is green.
- Applications pin a released CraftKit tag in their `Cargo.toml`.
- Never force-push a published tag; ship a patch release.
- Update `CHANGELOG.md` before tagging.

## Licence and independence

Dual MIT/Apache-2.0. CraftKit is an independent community project and must not
claim affiliation with Storytold, ArtCraft or any vendor whose product it helps
build alternatives to.
