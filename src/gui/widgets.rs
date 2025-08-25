//! Widget implementations for egui
//!
//! This module contains egui-specific widget rendering that interfaces 
//! with the core widget system.

use eframe::egui;
use wezztershier_core::widgets::{
    traits::{Widget, WidgetValue},
    implementations::{
        slider::SliderRenderData,
        select::SelectRenderData,
    }
};

/// Trait for rendering core widgets in egui
pub trait EguiWidget {
    /// Render the widget in egui and return true if the value changed
    fn render(&mut self, ui: &mut egui::Ui) -> bool;
}

/// Wrapper for rendering any core widget in egui
pub struct EguiWidgetRenderer<'a> {
    widget: &'a mut dyn Widget,
    widget_id: String,
}

impl<'a> EguiWidgetRenderer<'a> {
    pub fn new(widget: &'a mut dyn Widget, widget_id: String) -> Self {
        Self { widget, widget_id }
    }

    /// Render the widget based on its render_data type
    pub fn render(&mut self, ui: &mut egui::Ui) -> bool {
        let render_data = self.widget.render_data();
        
        // Add widget group with label
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
                    self.render_slider(ui, slider_data)
                } else if let Some(select_data) = render_data.get("select") {
                    self.render_select(ui, select_data)
                } else if let Some(color_data) = render_data.get("color_picker") {
                    self.render_color_picker(ui, color_data)
                } else if let Some(theme_data) = render_data.get("theme_selector") {
                    self.render_theme_selector(ui, theme_data)
                } else {
                    self.render_text_input(ui)
                }
            }).inner
        }).inner
    }

    /// Render a slider widget
    fn render_slider(&mut self, ui: &mut egui::Ui, slider_data: &serde_json::Value) -> bool {
        let data: SliderRenderData = match serde_json::from_value(slider_data.clone()) {
            Ok(data) => data,
            Err(_) => return false,
        };

        let mut value = data.current_value;
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

        // Show current value next to slider
        ui.horizontal(|ui| {
            ui.label("Value:");
            ui.label(format!("{:.1$}", value, data.precision.unwrap_or(1)));
        });

        if response.changed() && value != original_value {
            if let Err(e) = self.widget.set_value(WidgetValue::Number(value)) {
                tracing::error!("Failed to set widget value: {}", e);
                return false;
            }
            return true;
        }

        false
    }

    /// Render a select/dropdown widget
    fn render_select(&mut self, ui: &mut egui::Ui, select_data: &serde_json::Value) -> bool {
        let data: SelectRenderData = match serde_json::from_value(select_data.clone()) {
            Ok(data) => data,
            Err(_) => return false,
        };

        let current_value = match self.widget.get_value() {
            WidgetValue::String(s) => s.clone(),
            _ => String::new(),
        };

        let mut selected_value = current_value.clone();
        let mut changed = false;

        egui::ComboBox::from_id_source(&self.widget_id)
            .selected_text(&current_value)
            .show_ui(ui, |ui| {
                for option in &data.options {
                    let response = ui.selectable_value(&mut selected_value, option.clone(), option);
                    if response.clicked() && selected_value != current_value {
                        changed = true;
                    }
                }
            });

        if changed {
            if let Err(e) = self.widget.set_value(WidgetValue::String(selected_value)) {
                tracing::error!("Failed to set widget value: {}", e);
                return false;
            }
            return true;
        }

        false
    }

    /// Render a color picker widget
    fn render_color_picker(&mut self, ui: &mut egui::Ui, _color_data: &serde_json::Value) -> bool {
        let current_value = match self.widget.get_value() {
            WidgetValue::Color(c) | WidgetValue::String(c) => c.clone(),
            _ => "#000000".to_string(),
        };

        // Parse current color (basic hex parsing)
        let mut color = parse_hex_color(&current_value).unwrap_or([0, 0, 0]);
        let original_color = color;

        let response = ui.color_edit_button_srgb(&mut color);

        if response.changed() && color != original_color {
            let hex_color = format!("#{:02x}{:02x}{:02x}", color[0], color[1], color[2]);
            if let Err(e) = self.widget.set_value(WidgetValue::Color(hex_color)) {
                tracing::error!("Failed to set widget value: {}", e);
                return false;
            }
            return true;
        }

        // Show current color value
        ui.horizontal(|ui| {
            ui.label("Hex:");
            ui.monospace(&current_value);
        });

        false
    }

    /// Render a theme selector (similar to select but with preview)
    fn render_theme_selector(&mut self, ui: &mut egui::Ui, theme_data: &serde_json::Value) -> bool {
        // For now, treat it like a regular select
        self.render_select(ui, theme_data)
    }

    /// Render a generic text input widget
    fn render_text_input(&mut self, ui: &mut egui::Ui) -> bool {
        let current_value = match self.widget.get_value() {
            WidgetValue::String(s) => s.clone(),
            WidgetValue::Number(n) => n.to_string(),
            WidgetValue::Boolean(b) => b.to_string(),
            _ => String::new(),
        };

        let mut text_value = current_value.clone();
        let response = ui.text_edit_singleline(&mut text_value);

        if response.changed() && text_value != current_value {
            // Try to convert back to appropriate type
            let widget_value = match self.widget.get_value() {
                WidgetValue::Number(_) => {
                    if let Ok(n) = text_value.parse::<f64>() {
                        WidgetValue::Number(n)
                    } else {
                        return false; // Invalid number
                    }
                }
                WidgetValue::Boolean(_) => {
                    match text_value.to_lowercase().as_str() {
                        "true" | "1" | "yes" | "on" => WidgetValue::Boolean(true),
                        "false" | "0" | "no" | "off" => WidgetValue::Boolean(false),
                        _ => return false, // Invalid boolean
                    }
                }
                _ => WidgetValue::String(text_value),
            };

            if let Err(e) = self.widget.set_value(widget_value) {
                tracing::error!("Failed to set widget value: {}", e);
                return false;
            }
            return true;
        }

        false
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

    #[test]
    fn test_hex_color_parsing() {
        assert_eq!(parse_hex_color("#ff0000"), Some([255, 0, 0]));
        assert_eq!(parse_hex_color("00ff00"), Some([0, 255, 0]));
        assert_eq!(parse_hex_color("#0000ff"), Some([0, 0, 255]));
        assert_eq!(parse_hex_color("#invalid"), None);
        assert_eq!(parse_hex_color("#12345"), None);
    }
}