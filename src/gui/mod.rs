//! Native GUI implementation using egui
//!
//! This module provides the native GUI interface as an alternative to the web-based GUI.

pub mod app;
pub mod widgets;
pub mod dialogs;
pub mod state;

pub use app::WezztershierApp;

/// GUI initialization and main entry point
pub fn run_native_gui(config_file: Option<std::path::PathBuf>) -> anyhow::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("🎨 Wezztershier - WezTerm Configuration")
            .with_icon(
                // Load icon from embedded data if available
                eframe::icon_data::from_png_bytes(&[]).unwrap_or_default(),
            ),
        ..Default::default()
    };

    eframe::run_native(
        "Wezztershier",
        options,
        Box::new(|cc| Ok(Box::new(WezztershierApp::new(cc, config_file)))),
    ).map_err(|e| anyhow::anyhow!("Failed to run native GUI: {}", e))
}