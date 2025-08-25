//! Main application state and UI implementation

use anyhow::Result;
use eframe::egui;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use wezztershier_core::{
    ConfigManager,
    config::{BackupManager, ConfigUpdater},
    parser::parse_annotations,
    widgets::{factory::WidgetBuilder, implementations::register_core_widgets},
};

use super::state::{WidgetStateManager, StatefulWidgetRenderer};

/// Main application state for the native GUI
pub struct WezztershierApp {
    /// Core widget builder - shared with web interface
    widget_builder: WidgetBuilder,
    /// State manager for interactive widgets
    state_manager: WidgetStateManager,
    /// Configuration manager for file operations
    config_manager: Option<ConfigManager>,
    /// Backup manager for safety
    backup_manager: Option<BackupManager>,
    /// Current config content
    config_content: String,
    /// Current config file path
    config_file_path: Option<PathBuf>,
    /// UI state
    show_file_dialog: bool,
    show_about_dialog: bool,
    status_message: String,
    status_type: StatusType,
    /// Layout state
    left_panel_width: f32,
    /// File writeback debouncing
    last_change_time: Option<Instant>,
    writeback_pending: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum StatusType {
    None,
    Success,
    Warning,
    Error,
}

impl WezztershierApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, config_file: Option<PathBuf>) -> Self {
        // Register core widgets
        if let Err(e) = register_core_widgets() {
            tracing::error!("Failed to register core widgets: {}", e);
        }

        let mut app = Self {
            widget_builder: WidgetBuilder::new(),
            state_manager: WidgetStateManager::new(),
            config_manager: None,
            backup_manager: None,
            config_content: String::new(),
            config_file_path: config_file.clone(),
            show_file_dialog: false,
            show_about_dialog: false,
            status_message: String::new(),
            status_type: StatusType::None,
            left_panel_width: 400.0,
            last_change_time: None,
            writeback_pending: false,
        };

        // Load config file if provided
        if let Some(path) = config_file {
            if let Err(e) = app.load_config_file_sync(&path) {
                app.set_status(&format!("Failed to load {}: {}", path.display(), e), StatusType::Error);
            }
        } else {
            // Load sample config for demonstration
            app.load_sample_config();
        }

        app
    }

    fn load_sample_config(&mut self) {
        let sample_config = "-- Sample WezTerm configuration with GUI annotations\n\
-- @ui: slider(min=8, max=72, step=1) type=int\n\
config.font_size = 14\n\
\n\
-- @ui: theme_selector(themes=builtin, filter=all) type=string\n\
config.color_scheme = \"dracula\"\n\
\n\
-- @ui: color_picker(format=hex, alpha=false) type=color\n\
config.colors.background = \"#282a36\"\n\
\n\
-- @ui: slider(min=0.1, max=2.0, step=0.1) type=float\n\
config.window_background_opacity = 0.95";

        if let Err(e) = self.parse_config(sample_config) {
            self.set_status(&format!("Failed to load sample config: {}", e), StatusType::Error);
        } else {
            self.set_status("Sample configuration loaded", StatusType::Success);
        }
    }

    fn load_config_file_sync(&mut self, path: &PathBuf) -> Result<()> {
        let config_manager = ConfigManager::new(path);
        let content = std::fs::read_to_string(path)?;
        
        let backup_manager = BackupManager::new(path, None, 10)?;
        
        self.parse_config(&content)?;
        
        self.config_manager = Some(config_manager);
        self.backup_manager = Some(backup_manager);
        self.config_file_path = Some(path.clone());
        
        self.set_status(
            &format!("Loaded {} - Live saving enabled", path.file_name().unwrap_or_default().to_string_lossy()), 
            StatusType::Success
        );
        
        Ok(())
    }

    fn parse_config(&mut self, content: &str) -> Result<()> {
        let entries = parse_annotations(content)?;
        
        self.widget_builder = WidgetBuilder::new();
        self.widget_builder.add_from_entries(&entries)?;
        self.widget_builder.generate_auto_layout();
        self.config_content = content.to_string();
        
        // Initialize state manager with new widgets
        self.state_manager.initialize_from_builder(&self.widget_builder);
        
        Ok(())
    }

    fn set_status(&mut self, message: &str, status_type: StatusType) {
        self.status_message = message.to_string();
        self.status_type = status_type;
    }

    fn show_menu_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.menu_button("File", |ui| {
                if ui.button("📂 Open Config...").clicked() {
                    self.show_file_dialog = true;
                    ui.close_menu();
                }
                
                if ui.button("💾 Save").clicked() {
                    if self.config_manager.is_some() {
                        // Force immediate save (bypass debouncing)
                        self.last_change_time = Some(Instant::now() - Duration::from_secs(1));
                        self.writeback_pending = true;
                        self.handle_file_writeback();
                    } else {
                        self.set_status("No file loaded to save", StatusType::Warning);
                    }
                    ui.close_menu();
                }
                
                ui.separator();
                
                if ui.button("🔄 Reload Sample").clicked() {
                    self.load_sample_config();
                    ui.close_menu();
                }
            });
            
            ui.menu_button("Help", |ui| {
                if ui.button("ℹ️ About").clicked() {
                    self.show_about_dialog = true;
                    ui.close_menu();
                }
            });

            // Status display
            if !self.status_message.is_empty() {
                ui.separator();
                let color = match self.status_type {
                    StatusType::Success => egui::Color32::from_rgb(34, 197, 94),
                    StatusType::Warning => egui::Color32::from_rgb(251, 146, 60),
                    StatusType::Error => egui::Color32::from_rgb(239, 68, 68),
                    StatusType::None => egui::Color32::GRAY,
                };
                
                ui.colored_label(color, &self.status_message);
                
                // Auto-clear status after some time
                // TODO: Implement proper timing mechanism
            }
        });
    }

    fn show_file_dialog(&mut self, ctx: &egui::Context) {
        if self.show_file_dialog {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Lua files", &["lua"])
                .pick_file() 
            {
                if let Err(e) = self.load_config_file_sync(&path) {
                    self.set_status(&format!("Failed to load file: {}", e), StatusType::Error);
                }
            }
            self.show_file_dialog = false;
            ctx.request_repaint();
        }
    }

    fn show_about_dialog(&mut self, ctx: &egui::Context) {
        if self.show_about_dialog {
            egui::Window::new("About Wezztershier")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("🎨 Wezztershier");
                        ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
                        ui.add_space(10.0);
                        ui.label("Beautiful GUI for WezTerm configuration");
                        ui.label("Built with Rust and egui");
                        ui.add_space(10.0);
                        if ui.button("Close").clicked() {
                            self.show_about_dialog = false;
                        }
                    });
                });
        }
    }

    fn show_widgets_panel(&mut self, ui: &mut egui::Ui) -> bool {
        ui.heading("Configuration Widgets");
        
        let mut any_changes = false;
        
        egui::ScrollArea::vertical()
            .auto_shrink([false, true])
            .show(ui, |ui| {
                if self.widget_builder.len() == 0 {
                    ui.label("No widgets found. Load a configuration file with @ui annotations.");
                    return;
                }
                
                // Render interactive widgets using the state manager
                let widget_ids: Vec<String> = self.state_manager.widget_ids().cloned().collect();
                for widget_id in widget_ids {
                    if let Some(widget) = self.widget_builder.get_widget(&widget_id) {
                        let mut renderer = StatefulWidgetRenderer::new(
                            widget,
                            widget_id.clone(),
                            &mut self.state_manager,
                        );
                        
                        if renderer.render(ui) {
                            any_changes = true;
                        }
                        ui.add_space(8.0);
                    }
                }
            });
            
        // Apply any changes back to the widget builder
        if any_changes {
            match self.state_manager.apply_changes_to_builder(&mut self.widget_builder) {
                Ok(true) => {
                    tracing::debug!("Applied widget changes to builder");
                    // Mark change time for debounced writeback
                    self.last_change_time = Some(Instant::now());
                    self.writeback_pending = true;
                }
                Ok(false) => {
                    tracing::debug!("No changes to apply to builder");
                }
                Err(e) => {
                    tracing::error!("Failed to apply changes to builder: {}", e);
                    self.set_status(&format!("Error updating widgets: {}", e), StatusType::Error);
                }
            }
        }
            
        any_changes
    }

    fn show_preview_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Live Preview");
        
        // Get generated configuration from widget builder
        let config_preview = if self.widget_builder.len() > 0 {
            self.widget_builder.generate_config()
        } else {
            "-- Configuration will appear here --".to_string()
        };
        
        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // Make the preview read-only but copyable
                let mut config_text = config_preview.clone();
                ui.add(
                    egui::TextEdit::multiline(&mut config_text)
                        .font(egui::TextStyle::Monospace)
                        .desired_rows(20)
                        .interactive(false)
                );
                
                // Add copy button
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("📋 Copy to Clipboard").clicked() {
                        ui.output_mut(|o| o.copied_text = config_preview.clone());
                        self.set_status("Configuration copied to clipboard", StatusType::Success);
                    }
                    
                    ui.label(format!("{} lines", config_preview.lines().count()));
                });
            });
    }

    /// Handle debounced file writeback (similar to web interface)
    fn handle_file_writeback(&mut self) {
        if !self.writeback_pending {
            return;
        }

        // Check if enough time has passed since last change (500ms debounce)
        if let Some(last_change) = self.last_change_time {
            if last_change.elapsed() < Duration::from_millis(500) {
                return; // Still debouncing
            }
        }

        // Perform the writeback
        if let (Some(config_manager), Some(backup_manager)) = (&self.config_manager, &self.backup_manager) {
            let generated_config = self.widget_builder.generate_config();
            {
                    let config_updater = ConfigUpdater::new(self.config_content.clone());
                    
                    match config_updater.update_tuner_block(&generated_config) {
                        Ok(updated_content) => {
                            // Update stored content
                            self.config_content = updated_content.clone();
                            
                            // Write to file asynchronously (blocking for now, could be improved)
                            match std::fs::write(config_manager.config_path(), &updated_content) {
                                Ok(_) => {
                                    tracing::info!("Successfully wrote config to file");
                                    self.set_status("Configuration saved to file", StatusType::Success);
                                    self.writeback_pending = false;
                                }
                                Err(e) => {
                                    tracing::error!("Failed to write config file: {}", e);
                                    self.set_status(&format!("Error saving: {}", e), StatusType::Error);
                                    self.writeback_pending = false;
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to update tuner block: {}", e);
                            self.set_status(&format!("Error updating config: {}", e), StatusType::Error);
                            self.writeback_pending = false;
                        }
                    }
            }
        } else {
            // No file loaded, just clear the pending state
            self.writeback_pending = false;
        }
    }
}

impl eframe::App for WezztershierApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle file dialog
        self.show_file_dialog(ctx);
        
        // Show about dialog
        self.show_about_dialog(ctx);
        
        // Handle debounced file writeback
        self.handle_file_writeback();
        
        // Top panel with menu
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            self.show_menu_bar(ui);
        });

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            // Split into left (widgets) and right (preview) panels
            let available_rect = ui.available_rect_before_wrap();
            
            let left_width = self.left_panel_width.clamp(200.0, available_rect.width() - 200.0);
            let mut value_changed = false;
            
            ui.horizontal(|ui| {
                // Left panel - Widgets
                ui.allocate_ui_with_layout(
                    egui::Vec2::new(left_width, available_rect.height()),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        ui.set_width(left_width);
                        if self.show_widgets_panel(ui) {
                            value_changed = true;
                        }
                    },
                );
                
                // Separator
                let separator_response = ui.separator();
                if separator_response.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                }
                if separator_response.dragged() {
                    self.left_panel_width += separator_response.drag_delta().x;
                }
                
                // Right panel - Preview
                ui.allocate_ui_with_layout(
                    egui::Vec2::new(ui.available_width(), available_rect.height()),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        self.show_preview_panel(ui);
                    },
                );
            });
            
            // Handle value changes
            if value_changed {
                self.set_status("Configuration updated", StatusType::Success);
            }
        });
        
        // Request repaint for smooth updates
        if self.writeback_pending {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }
}