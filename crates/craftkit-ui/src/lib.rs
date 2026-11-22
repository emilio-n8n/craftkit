//! Native egui user interface shared by CraftKit applications.
//!
//! The crate provides presentation scaffolding, not a document model: a dark
//! theme, an application shell with status bar and windows, standard file
//! dialogs, CLI parsing and an eframe entry point.

pub mod cli;
pub mod dialogs;
pub mod run;
pub mod shell;
pub mod theme;
pub mod widgets;

pub use eframe;
pub use eframe::egui;
pub use run::{WindowOptions, run_native};
pub use shell::{AppIdentity, AppShell, Notice, Tone};
