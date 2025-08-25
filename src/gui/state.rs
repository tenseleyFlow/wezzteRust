//! State management for native GUI
//!
//! This module handles the separation of widget state from widget logic,
//! allowing for interactive updates during UI rendering.

use std::collections::HashMap;
use wezztershier_core::widgets::{
    traits::{Widget, WidgetValue},
    factory::WidgetBuilder,
};

/// Manages widget state separately from widget logic for UI updates
pub struct WidgetStateManager {
    /// Current values for each widget (separate from widget objects)
    widget_values: HashMap<String, WidgetValue>,
    /// Track which values have changed since last update
    changed_widgets: Vec<String>,
}

impl WidgetStateManager {
    pub fn new() -> Self {
        Self {
            widget_values: HashMap::new(),
            changed_widgets: Vec::new(),
        }
    }

    /// Initialize state from a widget builder
    pub fn initialize_from_builder(&mut self, widget_builder: &WidgetBuilder) {
        self.widget_values.clear();
        for widget_id in widget_builder.widget_ids() {
            if let Some(widget) = widget_builder.get_widget(&widget_id) {
                self.widget_values.insert(widget_id, widget.get_value().clone());
            }
        }
        self.changed_widgets.clear();
    }

    /// Get the current value for a widget
    pub fn get_value(&self, widget_id: &str) -> Option<&WidgetValue> {
        self.widget_values.get(widget_id)
    }

    /// Update a widget's value and mark it as changed
    pub fn update_value(&mut self, widget_id: String, value: WidgetValue) -> bool {
        let changed = match self.widget_values.get(&widget_id) {
            Some(current_value) => current_value != &value,
            None => true,
        };

        if changed {
            self.widget_values.insert(widget_id.clone(), value);
            if !self.changed_widgets.contains(&widget_id) {
                self.changed_widgets.push(widget_id);
            }
        }

        changed
    }

    /// Apply all changed values back to the widget builder
    pub fn apply_changes_to_builder(&mut self, widget_builder: &mut WidgetBuilder) -> Result<bool, String> {
        let mut any_changes = false;

        for widget_id in self.changed_widgets.drain(..) {
            if let Some(new_value) = self.widget_values.get(&widget_id) {
                match widget_builder.update_widget_value(&widget_id, new_value.clone()) {
                    Ok(_) => {
                        any_changes = true;
                        tracing::debug!("Updated widget {} to {:?}", widget_id, new_value);
                    }
                    Err(e) => {
                        tracing::error!("Failed to update widget {}: {}", widget_id, e);
                        return Err(format!("Failed to update widget {}: {}", widget_id, e));
                    }
                }
            }
        }

        Ok(any_changes)
    }

    /// Get all widget IDs that have current values
    pub fn widget_ids(&self) -> impl Iterator<Item = &String> {
        self.widget_values.keys()
    }

    /// Check if any widgets have changed
    pub fn has_changes(&self) -> bool {
        !self.changed_widgets.is_empty()
    }

    /// Clear all change tracking (but keep values)
    pub fn clear_changes(&mut self) {
        self.changed_widgets.clear();
    }
}

/// Widget renderer that works with the state manager
pub struct StatefulWidgetRenderer<'a> {
    widget: &'a dyn Widget,
    widget_id: String,
    state_manager: &'a mut WidgetStateManager,
}

impl<'a> StatefulWidgetRenderer<'a> {
    pub fn new(
        widget: &'a dyn Widget,
        widget_id: String,
        state_manager: &'a mut WidgetStateManager,
    ) -> Self {
        Self {
            widget,
            widget_id,
            state_manager,
        }
    }

    /// Render the widget and handle state updates
    pub fn render(&mut self, ui: &mut egui::Ui) -> bool {
        use eframe::egui;
        
        // Get current value from state manager (or fallback to widget's value)
        let current_value = self.state_manager
            .get_value(&self.widget_id)
            .unwrap_or_else(|| self.widget.get_value())
            .clone();

        let render_data = self.widget.render_data();
        let mut value_changed = false;

        ui.group(|ui| {
            ui.vertical(|ui| {
                // Widget label
                ui.label(egui::RichText::new(&self.widget.config().label).strong());

                // Optional tooltip
                if let Some(tooltip) = &self.widget.config().tooltip {
                    ui.label(egui::RichText::new(tooltip).small().weak());
                    ui.add_space(4.0);
                }

                // Render the appropriate widget type
                if let Some(slider_data) = render_data.get("slider") {
                    if let Some(new_value) = self.render_slider(ui, slider_data, &current_value) {
                        value_changed = self.state_manager.update_value(self.widget_id.clone(), new_value);
                    }
                } else if let Some(select_data) = render_data.get("select") {
                    if let Some(new_value) = self.render_select(ui, select_data, &current_value) {
                        value_changed = self.state_manager.update_value(self.widget_id.clone(), new_value);
                    }
                } else if let Some(_color_data) = render_data.get("color_picker") {
                    if let Some(new_value) = self.render_color_picker(ui, &current_value) {
                        value_changed = self.state_manager.update_value(self.widget_id.clone(), new_value);
                    }
                } else if let Some(theme_data) = render_data.get("theme_selector") {
                    if let Some(new_value) = self.render_select(ui, theme_data, &current_value) {
                        value_changed = self.state_manager.update_value(self.widget_id.clone(), new_value);
                    }
                } else {
                    if let Some(new_value) = self.render_text_input(ui, &current_value) {
                        value_changed = self.state_manager.update_value(self.widget_id.clone(), new_value);
                    }
                }
            });
        });

        value_changed
    }

    /// Render a slider widget
    fn render_slider(&self, ui: &mut egui::Ui, slider_data: &serde_json::Value, current_value: &WidgetValue) -> Option<WidgetValue> {
        use wezztershier_core::widgets::implementations::slider::SliderRenderData;
        
        let data: SliderRenderData = match serde_json::from_value(slider_data.clone()) {
            Ok(data) => data,
            Err(_) => return None,
        };

        let mut value = match current_value {
            WidgetValue::Number(n) => *n,
            _ => data.min,
        };
        let original_value = value;

        let response = if data.is_integer {
            // Integer slider
            let mut int_value = value as i64;
            let slider_response = ui.add(
                egui::Slider::new(&mut int_value, data.min as i64..=data.max as i64)
                    .step_by(data.step.max(1.0) as f64)
                    .show_value(true)
            );
            value = int_value as f64;
            slider_response
        } else {
            // Float slider
            let step = if data.step > 0.0 { data.step } else { 0.1 };
            ui.add(
                egui::Slider::new(&mut value, data.min..=data.max)
                    .step_by(step)
                    .show_value(true)
            )
        };

        // Show current value
        ui.horizontal(|ui| {
            ui.label("Value:");
            ui.label(format!("{:.1$}", value, data.precision.unwrap_or(1)));
        });

        if response.changed() && value != original_value {
            Some(WidgetValue::Number(value))
        } else {
            None
        }
    }

    /// Render a select/dropdown widget
    fn render_select(&self, ui: &mut egui::Ui, select_data: &serde_json::Value, current_value: &WidgetValue) -> Option<WidgetValue> {
        use wezztershier_core::widgets::implementations::select::SelectRenderData;
        
        let data: SelectRenderData = match serde_json::from_value(select_data.clone()) {
            Ok(data) => data,
            Err(_) => return None,
        };

        let current_str = match current_value {
            WidgetValue::String(s) => s.clone(),
            _ => String::new(),
        };

        let mut selected_value = current_str.clone();
        let mut changed = false;

        egui::ComboBox::from_id_source(&self.widget_id)
            .selected_text(&current_str)
            .show_ui(ui, |ui| {
                for option in &data.options {
                    let response = ui.selectable_value(&mut selected_value, option.clone(), option);
                    if response.clicked() && selected_value != current_str {
                        changed = true;
                    }
                }
            });

        if changed {
            Some(WidgetValue::String(selected_value))
        } else {
            None
        }
    }

    /// Render a color picker widget
    fn render_color_picker(&self, ui: &mut egui::Ui, current_value: &WidgetValue) -> Option<WidgetValue> {
        let current_str = match current_value {
            WidgetValue::Color(c) | WidgetValue::String(c) => c.clone(),
            _ => "#000000".to_string(),
        };

        // Parse current color
        let mut color = parse_hex_color(&current_str).unwrap_or([0, 0, 0]);
        let original_color = color;

        let response = ui.color_edit_button_srgb(&mut color);

        // Show current color value
        ui.horizontal(|ui| {
            ui.label("Hex:");
            ui.monospace(&current_str);
        });

        if response.changed() && color != original_color {
            let hex_color = format!("#{:02x}{:02x}{:02x}", color[0], color[1], color[2]);
            Some(WidgetValue::Color(hex_color))
        } else {
            None
        }
    }

    /// Render a generic text input widget
    fn render_text_input(&self, ui: &mut egui::Ui, current_value: &WidgetValue) -> Option<WidgetValue> {
        let current_str = match current_value {
            WidgetValue::String(s) => s.clone(),
            WidgetValue::Number(n) => n.to_string(),
            WidgetValue::Boolean(b) => b.to_string(),
            _ => String::new(),
        };

        let mut text_value = current_str.clone();
        let response = ui.text_edit_singleline(&mut text_value);

        if response.changed() && text_value != current_str {
            // Convert back to appropriate type based on current value type
            match current_value {
                WidgetValue::Number(_) => {
                    if let Ok(n) = text_value.parse::<f64>() {
                        Some(WidgetValue::Number(n))
                    } else {
                        None // Invalid number
                    }
                }
                WidgetValue::Boolean(_) => {
                    match text_value.to_lowercase().as_str() {
                        "true" | "1" | "yes" | "on" => Some(WidgetValue::Boolean(true)),
                        "false" | "0" | "no" | "off" => Some(WidgetValue::Boolean(false)),
                        _ => None, // Invalid boolean
                    }
                }
                _ => Some(WidgetValue::String(text_value)),
            }
        } else {
            None
        }
    }
}

/// Parse a hex color string to RGB array
fn parse_hex_color(hex: &str) -> Option<[u8; 3]> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some([r, g, b])
}

#[cfg(test)]
mod tests {
    use super::*;
    use wezztershier_core::{
        parser::parse_annotations,
        widgets::{factory::WidgetBuilder, implementations::register_core_widgets},
    };

    #[test]
    fn test_state_manager_initialization() {
        let mut state_manager = WidgetStateManager::new();
        assert!(!state_manager.has_changes());
        assert_eq!(state_manager.widget_ids().count(), 0);
    }

    #[test]
    fn test_value_updates() {
        let mut state_manager = WidgetStateManager::new();
        
        // Test new value
        assert!(state_manager.update_value("test".to_string(), WidgetValue::Number(42.0)));
        assert!(state_manager.has_changes());
        assert_eq!(state_manager.get_value("test"), Some(&WidgetValue::Number(42.0)));
        
        // Test same value (should not change)
        assert!(!state_manager.update_value("test".to_string(), WidgetValue::Number(42.0)));
        
        // Test different value
        assert!(state_manager.update_value("test".to_string(), WidgetValue::Number(43.0)));
    }

    #[test]
    fn test_hex_color_parsing() {
        assert_eq!(parse_hex_color("#ff0000"), Some([255, 0, 0]));
        assert_eq!(parse_hex_color("00ff00"), Some([0, 255, 0]));
        assert_eq!(parse_hex_color("#0000ff"), Some([0, 0, 255]));
        assert_eq!(parse_hex_color("#invalid"), None);
        assert_eq!(parse_hex_color("#12345"), None);
    }

    #[test] 
    fn test_full_widget_state_cycle() {
        // Register widgets for testing
        if let Err(_) = register_core_widgets() {
            // Widgets might already be registered
        }

        // Create a test configuration
        let config_content = r#"-- <<TUNER-START>>
-- @ui: slider(min=8, max=72, step=1) type=int
config.font_size = 14

-- @ui: select(options="left,center,right") type=string
config.tab_position = "left"
-- <<TUNER-END>>"#;

        // Parse and build widgets
        let entries = parse_annotations(config_content).unwrap();
        let mut widget_builder = WidgetBuilder::new();
        widget_builder.add_from_entries(&entries).unwrap();

        // Initialize state manager
        let mut state_manager = WidgetStateManager::new();
        state_manager.initialize_from_builder(&widget_builder);

        // Test that we have the expected widgets
        assert_eq!(state_manager.widget_ids().count(), 2);

        // Test updating a value
        let slider_id = "widget-font-size";
        assert!(state_manager.update_value(slider_id.to_string(), WidgetValue::Number(16.0)));
        assert_eq!(state_manager.get_value(slider_id), Some(&WidgetValue::Number(16.0)));

        // Test applying changes back to builder
        assert!(state_manager.apply_changes_to_builder(&mut widget_builder).unwrap());

        // Verify the change was applied
        if let Some(widget) = widget_builder.get_widget(slider_id) {
            assert_eq!(widget.get_value(), &WidgetValue::Number(16.0));
        } else {
            panic!("Widget not found after update");
        }
    }
}