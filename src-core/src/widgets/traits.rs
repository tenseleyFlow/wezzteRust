//! Core widget traits for the wezzte widget system
//!
//! Defines the fundamental interfaces that all widgets must implement.
//! These traits are designed to be UI framework agnostic.

use crate::{ast::{Annotation, ParamValue}, error::Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Events that widgets can emit
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WidgetEvent {
    /// Value changed event
    ValueChanged {
        widget_id: String,
        old_value: WidgetValue,
        new_value: WidgetValue,
    },
    /// Widget gained focus
    FocusGained { widget_id: String },
    /// Widget lost focus
    FocusLost { widget_id: String },
    /// Validation error occurred
    ValidationError {
        widget_id: String,
        error: String,
    },
}

/// Values that widgets can hold and emit
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WidgetValue {
    /// String value
    String(String),
    /// Numeric value
    Number(f64),
    /// Boolean value
    Boolean(bool),
    /// Color value (hex string)
    Color(String),
    /// List of values
    List(Vec<WidgetValue>),
    /// Key-value pairs
    Object(HashMap<String, WidgetValue>),
}

impl WidgetValue {
    /// Convert to configuration string representation
    pub fn to_config_string(&self) -> String {
        match self {
            WidgetValue::String(s) => format!("\"{}\"", s),
            WidgetValue::Number(n) => {
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            WidgetValue::Boolean(b) => b.to_string(),
            WidgetValue::Color(c) => format!("\"{}\"", c),
            WidgetValue::List(list) => {
                let items: Vec<String> = list.iter().map(|v| v.to_config_string()).collect();
                format!("[{}]", items.join(", "))
            }
            WidgetValue::Object(obj) => {
                let items: Vec<String> = obj
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_config_string()))
                    .collect();
                format!("{{{}}}", items.join(", "))
            }
        }
    }

    /// Get the data type name for this value
    pub fn type_name(&self) -> &'static str {
        match self {
            WidgetValue::String(_) => "string",
            WidgetValue::Number(_) => "number",
            WidgetValue::Boolean(_) => "boolean",
            WidgetValue::Color(_) => "color",
            WidgetValue::List(_) => "list",
            WidgetValue::Object(_) => "object",
        }
    }
}

impl From<&str> for WidgetValue {
    fn from(s: &str) -> Self {
        WidgetValue::String(s.to_string())
    }
}

impl From<String> for WidgetValue {
    fn from(s: String) -> Self {
        WidgetValue::String(s)
    }
}

impl From<f64> for WidgetValue {
    fn from(n: f64) -> Self {
        WidgetValue::Number(n)
    }
}

impl From<i64> for WidgetValue {
    fn from(n: i64) -> Self {
        WidgetValue::Number(n as f64)
    }
}

impl From<bool> for WidgetValue {
    fn from(b: bool) -> Self {
        WidgetValue::Boolean(b)
    }
}

/// Configuration for a widget instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetConfig {
    /// Unique identifier for this widget
    pub id: String,
    /// Configuration key this widget controls (e.g., "config.font_size")
    pub config_key: String,
    /// Current value
    pub current_value: WidgetValue,
    /// Widget-specific parameters from annotation
    pub parameters: HashMap<String, ParamValue>,
    /// Display label for the widget
    pub label: String,
    /// Optional tooltip text
    pub tooltip: Option<String>,
    /// Whether the widget is enabled
    pub enabled: bool,
}

impl WidgetConfig {
    /// Create a new widget config from annotation and current value
    pub fn from_annotation(
        id: String,
        config_key: String,
        annotation: &Annotation,
        current_value: WidgetValue,
    ) -> Self {
        // Generate a friendly label from the config key
        let label = Self::generate_label(&config_key);
        
        Self {
            id,
            config_key,
            current_value,
            parameters: annotation.params.clone(),
            label,
            tooltip: annotation.get_param("tooltip").and_then(|p| p.as_string().cloned()),
            enabled: annotation.get_param_as_bool("enabled", true),
        }
    }

    /// Generate a user-friendly label from a config key
    fn generate_label(config_key: &str) -> String {
        // Convert "config.font_size" to "Font Size"
        // Convert "config.colors.background" to "Colors › Background"
        let parts: Vec<&str> = config_key.split('.').skip(1).collect(); // Skip "config"
        
        if parts.len() > 1 {
            parts
                .iter()
                .map(|part| Self::humanize_key(part))
                .collect::<Vec<_>>()
                .join(" › ")
        } else if let Some(part) = parts.first() {
            Self::humanize_key(part)
        } else {
            config_key.to_string()
        }
    }

    /// Convert snake_case to Title Case
    fn humanize_key(key: &str) -> String {
        key.split('_')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Get a parameter as a specific type with fallback
    pub fn get_param_as_string(&self, key: &str, default: &str) -> String {
        self.parameters
            .get(key)
            .and_then(|p| p.as_string())
            .cloned()
            .unwrap_or_else(|| default.to_string())
    }

    pub fn get_param_as_number(&self, key: &str, default: f64) -> f64 {
        self.parameters
            .get(key)
            .and_then(|p| p.as_number())
            .unwrap_or(default)
    }

    pub fn get_param_as_bool(&self, key: &str, default: bool) -> bool {
        self.parameters
            .get(key)
            .and_then(|p| p.as_bool())
            .unwrap_or(default)
    }
}

/// Core widget trait that all widgets must implement
pub trait Widget: Send + Sync {
    /// Get the widget's configuration
    fn config(&self) -> &WidgetConfig;
    
    /// Get the current value of the widget
    fn get_value(&self) -> &WidgetValue;
    
    /// Set the widget's value (validates and may emit events)
    fn set_value(&mut self, value: WidgetValue) -> Result<()>;
    
    /// Validate a potential value for this widget
    fn validate_value(&self, value: &WidgetValue) -> Result<()>;
    
    /// Generate the configuration string for this widget's current value
    fn to_config_string(&self) -> String {
        self.get_value().to_config_string()
    }
    
    /// Check if the widget's value has changed from its initial value
    fn has_changed(&self) -> bool;
    
    /// Reset the widget to its initial value
    fn reset(&mut self) -> Result<()>;
    
    /// Get widget-specific rendering data for the UI
    fn render_data(&self) -> HashMap<String, serde_json::Value>;
}

/// Trait for widgets that can be created from annotations
pub trait WidgetFromAnnotation: Widget + Sized {
    /// Create a new widget instance from an annotation and current value
    fn from_annotation(
        id: String,
        config_key: String,
        annotation: &Annotation,
        current_value: WidgetValue,
    ) -> Result<Self>;
    
    /// Get the UI type that this widget handles
    fn ui_type() -> &'static str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_value_config_string() {
        assert_eq!(WidgetValue::String("test".to_string()).to_config_string(), "\"test\"");
        assert_eq!(WidgetValue::Number(42.0).to_config_string(), "42");
        assert_eq!(WidgetValue::Number(42.5).to_config_string(), "42.5");
        assert_eq!(WidgetValue::Boolean(true).to_config_string(), "true");
        assert_eq!(WidgetValue::Color("#ff0000".to_string()).to_config_string(), "\"#ff0000\"");
    }

    #[test]
    fn test_label_generation() {
        assert_eq!(WidgetConfig::generate_label("config.font_size"), "Font Size");
        assert_eq!(
            WidgetConfig::generate_label("config.colors.background"),
            "Colors › Background"
        );
        assert_eq!(
            WidgetConfig::generate_label("config.colors.tab_bar.active_tab.bg_color"),
            "Colors › Tab Bar › Active Tab › Bg Color"
        );
    }

    #[test]
    fn test_humanize_key() {
        assert_eq!(WidgetConfig::humanize_key("font_size"), "Font Size");
        assert_eq!(WidgetConfig::humanize_key("background_color"), "Background Color");
        assert_eq!(WidgetConfig::humanize_key("simple"), "Simple");
    }
}