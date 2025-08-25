//! Abstract Syntax Tree types for wezzte decorations
//!
//! Defines the structure of parsed decorator annotations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// UI component type for decorations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiType {
    /// Slider widget for numeric ranges
    Slider,
    /// Integer-specific slider
    IntSlider,
    /// Numerical input field
    Numerical,
    /// Text input field
    Text,
    /// Dropdown selector
    Select,
    /// Theme selector (special select)
    ThemeSelect,
    /// Color picker widget
    ColorPicker,
    /// Color scheme selector
    ColorScheme,
    /// Table path handler
    TablePath,
    /// Custom/unknown type
    Custom(String),
}

impl From<&str> for UiType {
    fn from(s: &str) -> Self {
        match s {
            "slider" => UiType::Slider,
            "int_slider" => UiType::IntSlider,
            "numerical" => UiType::Numerical,
            "text" => UiType::Text,
            "select" => UiType::Select,
            "theme_select" => UiType::ThemeSelect,
            "color_picker" => UiType::ColorPicker,
            "color_scheme" => UiType::ColorScheme,
            "table_path" => UiType::TablePath,
            other => UiType::Custom(other.to_string()),
        }
    }
}

impl UiType {
    pub fn as_str(&self) -> &str {
        match self {
            UiType::Slider => "slider",
            UiType::IntSlider => "int_slider",
            UiType::Numerical => "numerical",
            UiType::Text => "text",
            UiType::Select => "select",
            UiType::ThemeSelect => "theme_select",
            UiType::ColorPicker => "color_picker",
            UiType::ColorScheme => "color_scheme",
            UiType::TablePath => "table_path",
            UiType::Custom(s) => s,
        }
    }
}

/// Parameter values that can appear in decorations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParamValue {
    /// String value
    String(String),
    /// Numeric value (stored as f64 to handle both int and float)
    Number(f64),
    /// Boolean value
    Boolean(bool),
    /// List/array value
    List(Vec<ParamValue>),
    /// Object/dictionary value
    Object(HashMap<String, ParamValue>),
}

impl ParamValue {
    /// Get value as string if possible
    pub fn as_string(&self) -> Option<&String> {
        if let ParamValue::String(s) = self {
            Some(s)
        } else {
            None
        }
    }

    /// Get value as number if possible
    pub fn as_number(&self) -> Option<f64> {
        if let ParamValue::Number(n) = self {
            Some(*n)
        } else {
            None
        }
    }

    /// Get value as integer if possible
    pub fn as_int(&self) -> Option<i64> {
        self.as_number().map(|n| n as i64)
    }

    /// Get value as boolean if possible
    pub fn as_bool(&self) -> Option<bool> {
        if let ParamValue::Boolean(b) = self {
            Some(*b)
        } else {
            None
        }
    }

    /// Get value as list if possible
    pub fn as_list(&self) -> Option<&Vec<ParamValue>> {
        if let ParamValue::List(list) = self {
            Some(list)
        } else {
            None
        }
    }

    /// Get value as object if possible
    pub fn as_object(&self) -> Option<&HashMap<String, ParamValue>> {
        if let ParamValue::Object(obj) = self {
            Some(obj)
        } else {
            None
        }
    }
}

impl From<String> for ParamValue {
    fn from(s: String) -> Self {
        ParamValue::String(s)
    }
}

impl From<&str> for ParamValue {
    fn from(s: &str) -> Self {
        ParamValue::String(s.to_string())
    }
}

impl From<f64> for ParamValue {
    fn from(n: f64) -> Self {
        ParamValue::Number(n)
    }
}

impl From<i64> for ParamValue {
    fn from(n: i64) -> Self {
        ParamValue::Number(n as f64)
    }
}

impl From<bool> for ParamValue {
    fn from(b: bool) -> Self {
        ParamValue::Boolean(b)
    }
}

impl From<Vec<ParamValue>> for ParamValue {
    fn from(list: Vec<ParamValue>) -> Self {
        ParamValue::List(list)
    }
}

impl From<HashMap<String, ParamValue>> for ParamValue {
    fn from(obj: HashMap<String, ParamValue>) -> Self {
        ParamValue::Object(obj)
    }
}

/// A complete UI annotation from a decorator line
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Annotation {
    /// The UI component type
    pub ui_type: UiType,
    /// Parameters for the widget
    pub params: HashMap<String, ParamValue>,
}

impl Annotation {
    /// Create a new annotation
    pub fn new(ui_type: UiType, params: HashMap<String, ParamValue>) -> Self {
        Self { ui_type, params }
    }

    /// Get a parameter value by name
    pub fn get_param(&self, name: &str) -> Option<&ParamValue> {
        self.params.get(name)
    }

    /// Get a parameter as a specific type with fallback
    pub fn get_param_as_string(&self, name: &str, default: &str) -> String {
        self.get_param(name)
            .and_then(|v| v.as_string())
            .cloned()
            .unwrap_or_else(|| default.to_string())
    }

    pub fn get_param_as_number(&self, name: &str, default: f64) -> f64 {
        self.get_param(name)
            .and_then(|v| v.as_number())
            .unwrap_or(default)
    }

    pub fn get_param_as_int(&self, name: &str, default: i64) -> i64 {
        self.get_param(name)
            .and_then(|v| v.as_int())
            .unwrap_or(default)
    }

    pub fn get_param_as_bool(&self, name: &str, default: bool) -> bool {
        self.get_param(name)
            .and_then(|v| v.as_bool())
            .unwrap_or(default)
    }

    /// Check if a parameter exists
    pub fn has_param(&self, name: &str) -> bool {
        self.params.contains_key(name)
    }

    /// Get the data type for this widget (from 'type' parameter)
    pub fn get_data_type(&self) -> String {
        self.get_param_as_string("type", "float")
    }

    /// Check if this annotation is valid (has required parameters)
    pub fn validate(&self) -> crate::Result<()> {
        // Basic validation - can be extended per widget type
        match &self.ui_type {
            UiType::Slider | UiType::IntSlider => {
                if !self.has_param("min") || !self.has_param("max") {
                    return Err(crate::WezzteError::missing_parameter("min/max for slider"));
                }
            }
            UiType::Select | UiType::ThemeSelect => {
                if !self.has_param("options") {
                    return Err(crate::WezzteError::missing_parameter("options for select"));
                }
            }
            _ => {} // Other types may not have required params
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_type_conversion() {
        assert_eq!(UiType::from("slider"), UiType::Slider);
        assert_eq!(UiType::from("unknown"), UiType::Custom("unknown".to_string()));
    }

    #[test]
    fn test_param_value_conversions() {
        let string_val: ParamValue = "test".into();
        assert_eq!(string_val.as_string(), Some(&"test".to_string()));

        let num_val: ParamValue = 42.5.into();
        assert_eq!(num_val.as_number(), Some(42.5));

        let bool_val: ParamValue = true.into();
        assert_eq!(bool_val.as_bool(), Some(true));
    }

    #[test]
    fn test_annotation_parameters() {
        let mut params = HashMap::new();
        params.insert("min".to_string(), 0.0.into());
        params.insert("max".to_string(), 100.0.into());
        params.insert("step".to_string(), 1.0.into());

        let annotation = Annotation::new(UiType::Slider, params);

        assert_eq!(annotation.get_param_as_number("min", -1.0), 0.0);
        assert_eq!(annotation.get_param_as_number("max", -1.0), 100.0);
        assert_eq!(annotation.get_param_as_number("missing", 42.0), 42.0);
    }

    #[test]
    fn test_validation() {
        let mut params = HashMap::new();
        params.insert("min".to_string(), 0.0.into());
        params.insert("max".to_string(), 100.0.into());

        let valid_slider = Annotation::new(UiType::Slider, params);
        assert!(valid_slider.validate().is_ok());

        let invalid_slider = Annotation::new(UiType::Slider, HashMap::new());
        assert!(invalid_slider.validate().is_err());
    }
}