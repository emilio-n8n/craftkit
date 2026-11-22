//! Application shell: identity, top bar, status bar, about and settings.

use craftkit_io::{AppPaths, SettingsStore};
use eframe::egui::{self, Align, Color32, Layout, RichText, Stroke};
use serde::{Deserialize, Serialize};

use crate::theme;

/// Static identity of an application.
#[derive(Debug, Clone, Copy)]
pub struct AppIdentity {
    /// Short slug used for settings paths, for example `musiccraft`.
    pub id: &'static str,
    /// Display name, for example `MusicCraft`.
    pub name: &'static str,
    /// Version string, usually `env!("CARGO_PKG_VERSION")`.
    pub version: &'static str,
    /// One-line description.
    pub tagline: &'static str,
    /// Optional public repository.
    pub repository: Option<&'static str>,
}

/// Severity of a notice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    /// Information.
    Info,
    /// Success.
    Success,
    /// Warning.
    Warning,
    /// Error.
    Error,
}

impl Tone {
    /// Display colour for the tone.
    #[must_use]
    pub fn color(self) -> Color32 {
        match self {
            Self::Info => theme::INFO,
            Self::Success => theme::SUCCESS,
            Self::Warning => theme::WARNING,
            Self::Error => theme::ERROR,
        }
    }
}

/// A notice shown in its own window.
#[derive(Debug, Clone)]
pub struct Notice {
    /// Severity.
    pub tone: Tone,
    /// Window title.
    pub title: String,
    /// Body text.
    pub body: String,
}

/// Persisted shell preferences.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShellSettings {
    /// Ask before discarding unsaved changes.
    pub confirm_discard: bool,
    /// Open the About window at startup.
    pub show_about_on_start: bool,
}

impl Default for ShellSettings {
    fn default() -> Self {
        Self {
            confirm_discard: true,
            show_about_on_start: false,
        }
    }
}

/// Reusable shell state and drawing helpers.
///
/// The application owns the panels, menus and document. The shell owns the
/// identity, status line, notice/about/settings windows and persisted UI
/// preferences.
pub struct AppShell {
    identity: AppIdentity,
    status: String,
    notice: Option<Notice>,
    about_open: bool,
    settings_open: bool,
    settings: SettingsStore<ShellSettings>,
    last_title: String,
}

impl AppShell {
    /// Create a shell and load persisted preferences.
    #[must_use]
    pub fn new(identity: AppIdentity) -> Self {
        let settings_path =
            AppPaths::for_app(identity.id).map(|paths| paths.settings_file("ui.json"));
        let settings: SettingsStore<ShellSettings> = SettingsStore::load(settings_path);
        let about_open = settings.value().show_about_on_start;
        Self {
            identity,
            status: String::new(),
            notice: None,
            about_open,
            settings_open: false,
            settings,
            last_title: String::new(),
        }
    }

    /// Application identity.
    #[must_use]
    pub fn identity(&self) -> AppIdentity {
        self.identity
    }

    /// Persisted shell preferences.
    #[must_use]
    pub fn settings(&self) -> &ShellSettings {
        self.settings.value()
    }

    /// Apply the theme; call once from the eframe app constructor.
    pub fn install_theme(&self, ctx: &egui::Context) {
        theme::apply(ctx);
    }

    /// Set the status-bar message.
    pub fn set_status(&mut self, status: impl Into<String>) {
        self.status = status.into();
    }

    /// Current status-bar message.
    #[must_use]
    pub fn status(&self) -> &str {
        &self.status
    }

    /// Show a notice window.
    pub fn notify(&mut self, tone: Tone, title: impl Into<String>, body: impl Into<String>) {
        self.notice = Some(Notice {
            tone,
            title: title.into(),
            body: body.into(),
        });
    }

    /// Open the About window.
    pub fn open_about(&mut self) {
        self.about_open = true;
    }

    /// Open the Settings window.
    pub fn open_settings(&mut self) {
        self.settings_open = true;
    }

    /// Draw the application name and tagline.
    pub fn brand(&self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new(self.identity.name)
                .size(13.0)
                .strong()
                .color(theme::ACCENT),
        );
        ui.label(
            RichText::new(self.identity.tagline)
                .size(10.5)
                .color(theme::FAINT),
        );
    }

    /// Draw a top bar with the application name, document title and optional
    /// controls.
    pub fn top_bar(
        &mut self,
        ui: &mut egui::Ui,
        title: &str,
        dirty: bool,
        extra: impl FnOnce(&mut egui::Ui),
    ) {
        egui::Panel::top(format!("{}-top", self.identity.id))
            .frame(
                egui::Frame::new()
                    .fill(theme::PANEL)
                    .inner_margin(egui::Margin::symmetric(12, 7))
                    .stroke(Stroke::new(1.0, theme::BORDER)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(self.identity.name)
                            .size(13.0)
                            .strong()
                            .color(theme::ACCENT),
                    );
                    ui.label(RichText::new(title).size(13.0).color(theme::TEXT));
                    if dirty {
                        ui.label(RichText::new("*").size(15.0).strong().color(theme::WARNING))
                            .on_hover_text("Unsaved changes");
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), extra);
                });
            });
    }

    /// Draw the status bar with an optional application-specific area.
    pub fn status_bar_with(&mut self, ui: &mut egui::Ui, extra: impl FnOnce(&mut egui::Ui)) {
        egui::Panel::bottom(format!("{}-status", self.identity.id))
            .frame(
                egui::Frame::new()
                    .fill(theme::PANEL)
                    .inner_margin(egui::Margin::symmetric(12, 4))
                    .stroke(Stroke::new(1.0, theme::BORDER)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&self.status).size(11.0).color(theme::MUTED));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!("v{}", self.identity.version))
                                .size(10.0)
                                .color(theme::FAINT),
                        );
                        extra(ui);
                    });
                });
            });
    }

    /// Draw the status bar without extra content.
    pub fn status_bar(&mut self, ui: &mut egui::Ui) {
        self.status_bar_with(ui, |_| {});
    }

    /// Draw the about, notice and settings windows.
    pub fn show_windows(&mut self, ctx: &egui::Context) {
        self.about_window(ctx);
        self.notice_window(ctx);
        self.settings_window(ctx);
    }

    /// Update the native window title, avoiding a viewport command per frame.
    pub fn update_title(&mut self, ctx: &egui::Context, title: &str, dirty: bool) {
        let title = format!(
            "{}{} — {}",
            if dirty { "* " } else { "" },
            title,
            self.identity.name
        );
        if title != self.last_title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.last_title = title;
        }
    }

    fn about_window(&mut self, ctx: &egui::Context) {
        let mut open = self.about_open;
        let identity = self.identity;
        egui::Window::new(format!("About {}", identity.name))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(380.0)
            .show(ctx, |ui| {
                ui.heading(identity.name);
                ui.label(
                    RichText::new(identity.tagline)
                        .size(12.0)
                        .color(theme::MUTED),
                );
                ui.add_space(8.0);
                ui.separator();
                ui.label(format!("Version {}", identity.version));
                ui.label(
                    RichText::new("Built with CraftKit, a small native Rust desktop foundation.")
                        .size(11.0)
                        .color(theme::MUTED),
                );
                if let Some(repository) = identity.repository {
                    ui.hyperlink_to("Project repository", repository);
                }
                ui.add_space(6.0);
                ui.label(
                    RichText::new("Licensed under MIT OR Apache-2.0.")
                        .size(10.5)
                        .color(theme::FAINT),
                );
            });
        self.about_open = open;
    }

    fn notice_window(&mut self, ctx: &egui::Context) {
        let Some(notice) = self.notice.take() else {
            return;
        };
        let mut open = true;
        let mut close_clicked = false;
        egui::Window::new(&notice.title)
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(420.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("●").size(12.0).color(notice.tone.color()));
                    ui.label(RichText::new(&notice.body).size(12.0).color(theme::TEXT));
                });
                ui.add_space(8.0);
                if ui.button("Close").clicked() {
                    close_clicked = true;
                }
            });
        if open && !close_clicked {
            self.notice = Some(notice);
        }
    }

    fn settings_window(&mut self, ctx: &egui::Context) {
        let mut open = self.settings_open;
        let mut next = self.settings.value().clone();
        let mut changed = false;
        egui::Window::new("Settings")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(340.0)
            .show(ctx, |ui| {
                changed |= ui
                    .checkbox(
                        &mut next.confirm_discard,
                        "Ask before discarding unsaved changes",
                    )
                    .changed();
                changed |= ui
                    .checkbox(&mut next.show_about_on_start, "Show About on startup")
                    .changed();
            });
        self.settings_open = open;
        if changed {
            *self.settings.value_mut() = next;
            if let Err(error) = self.settings.save() {
                self.notify(Tone::Warning, "Settings not saved", error.to_string());
            }
        }
    }
}
