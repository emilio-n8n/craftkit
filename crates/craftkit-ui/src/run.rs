//! Native window setup and entry point.

use eframe::egui;

use crate::theme;

/// Size and title of the main window.
#[derive(Debug, Clone)]
pub struct WindowOptions {
    /// Native window title.
    pub title: String,
    /// Initial inner size in logical pixels.
    pub size: [f32; 2],
    /// Minimum inner size in logical pixels.
    pub min_size: [f32; 2],
}

impl WindowOptions {
    /// Default window size for a desktop editor.
    #[must_use]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            size: [1280.0, 820.0],
            min_size: [900.0, 600.0],
        }
    }
}

/// Start the native event loop.
///
/// `build` receives the egui creation context and returns the application.
/// The CraftKit dark theme is installed before the application is built.
///
/// # Errors
///
/// Returns an eframe error when the window or graphics backend cannot start.
pub fn run_native<F>(options: WindowOptions, build: F) -> eframe::Result<()>
where
    F: FnOnce(
            &eframe::CreationContext<'_>,
        ) -> Result<Box<dyn eframe::App>, Box<dyn std::error::Error + Send + Sync>>
        + 'static,
{
    let title = options.title;
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(title.clone())
            .with_inner_size(options.size)
            .with_min_inner_size(options.min_size)
            .with_icon(default_icon()),
        ..Default::default()
    };
    eframe::run_native(
        &title,
        native_options,
        Box::new(move |context| {
            theme::apply(&context.egui_ctx);
            build(context)
        }),
    )
}

/// A neutral application icon generated at runtime.
fn default_icon() -> egui::IconData {
    let size = 64_u32;
    let mut rgba = vec![0_u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size {
            let index = ((y * size + x) * 4) as usize;
            let inside = (8..56).contains(&x) && (8..56).contains(&y);
            let bar = inside
                && ((20..=44).contains(&x) && (18..=21).contains(&y)
                    || (20..=44).contains(&x) && (30..=33).contains(&y)
                    || (20..=44).contains(&x) && (42..=45).contains(&y)
                    || (20..=23).contains(&x) && (18..=45).contains(&y)
                    || (41..=44).contains(&x) && (18..=45).contains(&y));
            let (red, green, blue, alpha) = if bar {
                (255, 255, 255, 255)
            } else if inside {
                (32, 36, 45, 255)
            } else {
                (232, 87, 63, 255)
            };
            rgba[index] = red;
            rgba[index + 1] = green;
            rgba[index + 2] = blue;
            rgba[index + 3] = alpha;
        }
    }
    egui::IconData {
        rgba,
        width: size,
        height: size,
    }
}
