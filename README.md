# CraftKit

CraftKit is a small, domain-agnostic foundation for native Rust desktop
applications built with egui/eframe. It packages the parts every application
repeats: application shell, project lifecycle, undo/redo, background jobs,
settings, file dialogs, theming and release scaffolding.

It intentionally contains **no document schema and no domain geometry**. A
drawing app, a DAW, a GIS tool and a FEM editor can all use the same base.

CraftKit is an independent, community open-source project. It is not affiliated
with, endorsed by, or an official product of any third-party vendor.

## Crates

| Crate | Role |
| --- | --- |
| `craftkit-core` | Bounded `History<T>` and versioned project envelope |
| `craftkit-io` | Atomic JSON project files, app paths, settings, recent files |
| `craftkit-jobs` | Background jobs with progress and cancellation |
| `craftkit-ui` | egui shell, theme, dialogs, CLI and `run_native` |
| `apps/craftkit-demo` | Reference application demonstrating every feature |

Dependency direction is strictly one-way:

```text
craftkit-core <- craftkit-io <- craftkit-ui <- application
craftkit-core <- craftkit-jobs
```

`craftkit-core` has no UI, filesystem, network or threading dependency.

## Quick start

```bash
cargo run -p craftkit-demo -- --sample
cargo run -p craftkit-demo -- --open project.json
```

The demo shows a small notes document. It implements the full lifecycle:

- new / open / save / save as, with unsaved-changes confirmation;
- undo / redo through `History<T>`;
- atomic JSON save with a versioned envelope;
- recent files and persisted UI settings;
- background job with progress and cancellation;
- about, notice and error windows;
- dark theme and keyboard shortcuts.

## Use it in an application

```toml
[dependencies]
craftkit-core = { git = "https://github.com/emilio-n8n/craftkit", tag = "v0.1.0" }
craftkit-io = { git = "https://github.com/emilio-n8n/craftkit", tag = "v0.1.0" }
craftkit-jobs = { git = "https://github.com/emilio-n8n/craftkit", tag = "v0.1.0" }
craftkit-ui = { git = "https://github.com/emilio-n8n/craftkit", tag = "v0.1.0" }
serde = { version = "1", features = ["derive"] }
```

`craftkit-ui` re-exports `eframe` and `egui`, so an application only needs the
CraftKit crates and `serde`.

Generate a new application from the reference app:

```bash
cargo run -p craftkit-demo -- --version
./tools/new-app.py musiccraft "MusicCraft" emilio-n8n
```

The generator creates a standalone Cargo project with git-pinned CraftKit
dependencies, CI, release workflow, licences, documentation, a `craft-app.json`
launcher descriptor, a 128×128 icon and a `tools/launcher_check.py` validator.
See [docs/launcher.md](docs/launcher.md) for the two ArtCraft onboarding paths
and the release naming contract.

## Design rules

- The document is owned by a `DocumentHost<T>`; every mutation goes through
  `History<T>`.
- Project files are self-sufficient JSON with a format name and schema version.
- Long operations never block the UI thread; use `craftkit-jobs`.
- Panels are built by the application; CraftKit provides the shell, status bar,
  windows, dialogs and theme.
- No `unsafe`. No `unwrap` on external data paths.
- Public API is documented and tested.

## Build

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

On Linux the GUI dependencies must be installed; see the CI workflow.

## Licence

Licensed under either of Apache-2.0 (`LICENSE-APACHE`) or MIT (`LICENSE-MIT`),
at your option.
