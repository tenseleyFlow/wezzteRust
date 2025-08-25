//! Widget factory for creating widgets from annotations
//!
//! Provides high-level interface for widget creation and management.

use crate::{
    ast::Annotation,
    error::{Result, WezzteError},
    parser::ConfigEntry,
    layout::{Layout, AutoLayoutGenerator},
    widgets::{
        registry::global_registry,
        traits::{Widget, WidgetValue},
    },
};
use std::collections::HashMap;

/// High-level widget factory
pub struct WidgetFactory;

impl WidgetFactory {
    /// Create a widget from a config entry
    pub fn create_from_entry(entry: &ConfigEntry) -> Result<Box<dyn Widget>> {
        let current_value = Self::parse_config_value(&entry.value, &entry.annotation)?;
        
        Self::create_widget(
            Self::generate_widget_id(&entry.key),
            entry.key.clone(),
            &entry.annotation,
            current_value,
        )
    }

    /// Create a widget from annotation and current value
    pub fn create_widget(
        id: String,
        config_key: String,
        annotation: &Annotation,
        current_value: WidgetValue,
    ) -> Result<Box<dyn Widget>> {
        let registry = global_registry();
        registry.create_widget(id, config_key, annotation, current_value)
    }

    /// Parse a configuration value string into a WidgetValue
    pub fn parse_config_value(value_str: &str, annotation: &Annotation) -> Result<WidgetValue> {
        let data_type = annotation.get_data_type();
        let trimmed = value_str.trim();

        match data_type.as_str() {
            "int" | "integer" => {
                // Handle function calls like wezterm.font("...")
                if trimmed.contains('(') && !trimmed.starts_with('"') {
                    Ok(WidgetValue::String(trimmed.to_string()))
                } else {
                    // Parse as number
                    trimmed
                        .parse::<i64>()
                        .map(|n| WidgetValue::Number(n as f64))
                        .map_err(|_| {
                            WezzteError::invalid_parameter(
                                "value",
                                format!("cannot parse '{}' as integer", trimmed),
                            )
                        })
                }
            }
            "float" | "number" => {
                // Handle function calls
                if trimmed.contains('(') && !trimmed.starts_with('"') {
                    Ok(WidgetValue::String(trimmed.to_string()))
                } else {
                    // Parse as number
                    trimmed
                        .parse::<f64>()
                        .map(WidgetValue::Number)
                        .map_err(|_| {
                            WezzteError::invalid_parameter(
                                "value",
                                format!("cannot parse '{}' as number", trimmed),
                            )
                        })
                }
            }
            "bool" | "boolean" => {
                match trimmed.to_lowercase().as_str() {
                    "true" => Ok(WidgetValue::Boolean(true)),
                    "false" => Ok(WidgetValue::Boolean(false)),
                    _ => Err(WezzteError::invalid_parameter(
                        "value",
                        format!("cannot parse '{}' as boolean", trimmed),
                    )),
                }
            }
            "color" => {
                // Remove quotes from color strings
                let color_val = if trimmed.starts_with('"') && trimmed.ends_with('"') {
                    &trimmed[1..trimmed.len() - 1]
                } else {
                    trimmed
                };
                Ok(WidgetValue::String(color_val.to_string()))
            }
            "string" | _ => {
                // Handle quoted strings
                if trimmed.starts_with('"') && trimmed.ends_with('"') {
                    Ok(WidgetValue::String(trimmed[1..trimmed.len() - 1].to_string()))
                } else {
                    Ok(WidgetValue::String(trimmed.to_string()))
                }
            }
        }
    }

    /// Generate a widget ID from a config key
    pub fn generate_widget_id(config_key: &str) -> String {
        // Convert "config.font_size" to "widget-font-size"
        config_key
            .replace("config.", "widget-")
            .replace('.', "-")
            .replace('_', "-")
    }

    /// Get list of registered widget types
    pub fn registered_types() -> Result<Vec<String>> {
        global_registry().registered_types()
    }

    /// Check if a widget type is registered
    pub fn is_registered(ui_type: &str) -> bool {
        global_registry().is_registered(ui_type)
    }

    /// Get debug information
    pub fn debug_info() -> Result<String> {
        global_registry().debug_info()
    }
}

/// High-level widget builder for creating multiple widgets
pub struct WidgetBuilder {
    widgets: HashMap<String, Box<dyn Widget>>,
    layout: Option<Layout>,
}

impl WidgetBuilder {
    /// Create a new widget builder
    pub fn new() -> Self {
        Self {
            widgets: HashMap::new(),
            layout: None,
        }
    }

    /// Add a widget from a config entry
    pub fn add_from_entry(&mut self, entry: &ConfigEntry) -> Result<String> {
        let widget = WidgetFactory::create_from_entry(entry)?;
        let widget_id = widget.config().id.clone();
        self.widgets.insert(widget_id.clone(), widget);
        Ok(widget_id)
    }

    /// Add multiple widgets from config entries
    pub fn add_from_entries(&mut self, entries: &[ConfigEntry]) -> Result<Vec<String>> {
        let mut widget_ids = Vec::new();
        
        for entry in entries {
            let widget_id = self.add_from_entry(entry)?;
            widget_ids.push(widget_id);
        }
        
        Ok(widget_ids)
    }

    /// Get a widget by ID
    pub fn get_widget(&self, id: &str) -> Option<&dyn Widget> {
        self.widgets.get(id).map(|w| w.as_ref())
    }

    /// Update a widget's value directly
    pub fn update_widget_value(&mut self, id: &str, value: WidgetValue) -> Result<()> {
        let widget = self.widgets.get_mut(id).ok_or_else(|| {
            WezzteError::config(format!("Widget not found: {}", id))
        })?;

        widget.set_value(value)
    }

    /// Get all widget IDs
    pub fn widget_ids(&self) -> Vec<String> {
        self.widgets.keys().cloned().collect()
    }

    /// Get number of widgets
    pub fn len(&self) -> usize {
        self.widgets.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.widgets.is_empty()
    }


    /// Generate configuration string for all widgets
    pub fn generate_config(&self) -> String {
        let mut config_lines = Vec::new();
        
        for widget in self.widgets.values() {
            let config_line = format!(
                "{} = {}",
                widget.config().config_key,
                widget.to_config_string()
            );
            config_lines.push(config_line);
        }
        
        config_lines.join("\n")
    }

    /// Get widgets that have changed from their initial values
    pub fn changed_widgets(&self) -> Vec<&dyn Widget> {
        self.widgets
            .values()
            .filter(|w| w.has_changed())
            .map(|w| w.as_ref())
            .collect()
    }

    /// Reset all widgets to their initial values
    pub fn reset_all(&mut self) -> Result<()> {
        for widget in self.widgets.values_mut() {
            widget.reset()?;
        }
        Ok(())
    }

    /// Generate automatic layout based on current widgets
    pub fn generate_auto_layout(&mut self) {
        self.layout = Some(AutoLayoutGenerator::generate_layout(&self.widgets));
    }

    /// Set a custom layout
    pub fn set_layout(&mut self, layout: Layout) {
        self.layout = Some(layout);
    }

    /// Get the current layout
    pub fn get_layout(&self) -> Option<&Layout> {
        self.layout.as_ref()
    }

    /// Get serializable data for all widgets (for Tauri bridge)
    pub fn serialize_widgets(&self) -> Result<serde_json::Value> {
        let mut widget_data = serde_json::Map::new();
        
        for (id, widget) in &self.widgets {
            let mut widget_info = serde_json::Map::new();
            
            // Basic widget info
            widget_info.insert("id".to_string(), serde_json::Value::String(id.clone()));
            widget_info.insert("config_key".to_string(), serde_json::Value::String(widget.config().config_key.clone()));
            widget_info.insert("label".to_string(), serde_json::Value::String(widget.config().label.clone()));
            widget_info.insert("enabled".to_string(), serde_json::Value::Bool(widget.config().enabled));
            widget_info.insert("has_changed".to_string(), serde_json::Value::Bool(widget.has_changed()));
            
            // Current value
            widget_info.insert("current_value".to_string(), serde_json::to_value(widget.get_value())?);
            
            // Widget-specific render data
            let render_data = widget.render_data();
            for (key, value) in render_data {
                widget_info.insert(key, value);
            }
            
            widget_data.insert(id.clone(), serde_json::Value::Object(widget_info));
        }
        
        Ok(serde_json::Value::Object(widget_data))
    }

    /// Get complete application data including layout
    pub fn serialize_app_data(&self) -> Result<serde_json::Value> {
        let mut app_data = serde_json::Map::new();
        
        // Widget data
        let widgets = self.serialize_widgets()?;
        app_data.insert("widgets".to_string(), widgets);
        
        // Layout data
        if let Some(layout) = &self.layout {
            app_data.insert("layout".to_string(), layout.serialize_for_frontend());
        }
        
        // Configuration preview
        app_data.insert("config_preview".to_string(), serde_json::Value::String(self.generate_config()));
        
        Ok(serde_json::Value::Object(app_data))
    }
}

impl Default for WidgetBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parser::parse_annotations, widgets::implementations::register_core_widgets};

    #[test]
    fn test_widget_id_generation() {
        assert_eq!(WidgetFactory::generate_widget_id("config.font_size"), "widget-font-size");
        assert_eq!(
            WidgetFactory::generate_widget_id("config.colors.background"),
            "widget-colors-background"
        );
    }

    #[test]
    fn test_config_value_parsing() {
        let annotation = crate::parser::parse_decorator_line("-- @ui: slider(min=0, max=100) type=int").unwrap();
        
        let value = WidgetFactory::parse_config_value("42", &annotation).unwrap();
        assert_eq!(value, WidgetValue::Number(42.0));
        
        let string_annotation = crate::parser::parse_decorator_line("-- @ui: text type=string").unwrap();
        let string_value = WidgetFactory::parse_config_value("\"hello world\"", &string_annotation).unwrap();
        assert_eq!(string_value, WidgetValue::String("hello world".to_string()));
    }

    #[tokio::test]
    async fn test_widget_builder() {
        // Register core widgets first
        register_core_widgets().unwrap();
        
        let config = r##"
-- <<TUNER-START>>
-- @ui: slider(min=10, max=42, step=1) type=int
config.font_size = 18
-- @ui: select(options="Dark, Light") type=string
config.theme = "Dark"
-- <<TUNER-END>>
"##;

        let entries = parse_annotations(config).unwrap();
        let mut builder = WidgetBuilder::new();
        
        let widget_ids = builder.add_from_entries(&entries).unwrap();
        assert_eq!(widget_ids.len(), 2);
        assert_eq!(builder.len(), 2);
        
        // Test widget retrieval
        let font_size_widget = builder.get_widget(&widget_ids[0]).unwrap();
        assert_eq!(font_size_widget.config().config_key, "config.font_size");
        
        // Test configuration generation
        let config_str = builder.generate_config();
        assert!(config_str.contains("config.font_size = 18"));
        assert!(config_str.contains("config.theme = Dark"));
    }
}