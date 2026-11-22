# Changelog

All notable changes to CraftKit are documented here.

## [0.1.0] - 2026-11-22

### Added

- `craftkit-core`: generic bounded `History<T>` with dirty tracking and
  `ProjectEnvelope<T>` format/schema validation.
- `craftkit-io`: atomic project save/load through `DocumentHost<T>`, platform
  application paths, JSON settings store and bounded recent-files list.
- `craftkit-jobs`: thread-backed job runner with progress, status messages and
  cooperative cancellation.
- `craftkit-ui`: egui application shell, dark theme, about/notice/settings
  windows, status bar, file dialogs, CLI parsing and `run_native`.
- `craftkit-demo`: reference notes application exercising the whole lifecycle.
- `tools/new-app.py`: generator for standalone applications with pinned
  CraftKit dependencies, CI and release workflows.
