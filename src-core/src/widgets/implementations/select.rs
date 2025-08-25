//! Select widget implementation
//!
//! Provides a dropdown/select widget for choosing from predefined options.

use crate::{
    ast::Annotation,
    error::{Result, WezzteError},
    widgets::traits::{Widget, WidgetConfig, WidgetFromAnnotation, WidgetValue},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Select widget for choosing from options
#[derive(Debug, Clone)]
pub struct SelectWidget {
    config: WidgetConfig,
    initial_value: WidgetValue,
    options: Vec<String>,
    allow_custom: bool,
}

/// Render data specific to select widgets
#[derive(Debug, Serialize, Deserialize)]
pub struct SelectRenderData {
    pub options: Vec<String>,
    pub current_value: String,
    pub allow_custom: bool,
    pub placeholder: Option<String>,
}

impl SelectWidget {
    /// Create a new select widget
    pub fn new(
        config: WidgetConfig,
        options: Vec<String>,
        allow_custom: bool,
    ) -> Result<Self> {
        if options.is_empty() {
            return Err(WezzteError::invalid_parameter(
                "options",
                "select widget must have at least one option",
            ));
        }

        // Validate current value is a string
        if !matches!(config.current_value, WidgetValue::String(_)) {
            return Err(WezzteError::invalid_parameter(
                "current_value",
                "select widget requires string value",
            ));
        }

        let initial_value = config.current_value.clone();

        let widget = Self {
            config,
            initial_value,
            options,
            allow_custom,
        };

        // Validate current value is in options (if not allowing custom)
        widget.validate_value(widget.get_value())?;

        Ok(widget)
    }

    /// Get the string value
    pub fn string_value(&self) -> &str {
        match &self.config.current_value {
            WidgetValue::String(s) => s,
            _ => "", // Fallback
        }
    }

    /// Parse options from a comma-separated string
    pub fn parse_options(options_str: &str) -> Vec<String> {
        options_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Check if a value is valid for this select widget
    fn is_valid_option(&self, value: &str) -> bool {
        self.allow_custom || self.options.iter().any(|opt| opt == value)
    }
}

impl Widget for SelectWidget {
    fn config(&self) -> &WidgetConfig {
        &self.config
    }

    fn get_value(&self) -> &WidgetValue {
        &self.config.current_value
    }

    fn set_value(&mut self, value: WidgetValue) -> Result<()> {
        if let WidgetValue::String(string_val) = value {
            if !self.is_valid_option(&string_val) {
                return Err(WezzteError::invalid_parameter(
                    "value",
                    format!(
                        "value '{}' is not in allowed options: {}",
                        string_val,
                        self.options.join(", ")
                    ),
                ));
            }

            self.config.current_value = WidgetValue::String(string_val);
            Ok(())
        } else {
            Err(WezzteError::invalid_parameter(
                "value",
                "select widget requires string value",
            ))
        }
    }

    fn validate_value(&self, value: &WidgetValue) -> Result<()> {
        if let WidgetValue::String(string_val) = value {
            if !self.is_valid_option(string_val) {
                return Err(WezzteError::invalid_parameter(
                    "value",
                    format!(
                        "value '{}' is not in allowed options: {}",
                        string_val,
                        self.options.join(", ")
                    ),
                ));
            }
            Ok(())
        } else {
            Err(WezzteError::invalid_parameter(
                "value",
                "select widget requires string value",
            ))
        }
    }

    fn has_changed(&self) -> bool {
        self.config.current_value != self.initial_value
    }

    fn reset(&mut self) -> Result<()> {
        self.config.current_value = self.initial_value.clone();
        Ok(())
    }

    fn render_data(&self) -> HashMap<String, serde_json::Value> {
        let render_data = SelectRenderData {
            options: self.options.clone(),
            current_value: self.string_value().to_string(),
            allow_custom: self.allow_custom,
            placeholder: self.config.get_param_as_string("placeholder", "").into(),
        };

        let mut data = HashMap::new();
        data.insert(
            "select".to_string(),
            serde_json::to_value(render_data).unwrap_or_default(),
        );
        data
    }
}

impl WidgetFromAnnotation for SelectWidget {
    fn from_annotation(
        id: String,
        config_key: String,
        annotation: &Annotation,
        current_value: WidgetValue,
    ) -> Result<Self> {
        let config = WidgetConfig::from_annotation(id, config_key, annotation, current_value);

        // Extract required options parameter
        let options_str = annotation.get_param_as_string("options", "");
        if options_str.is_empty() {
            return Err(WezzteError::missing_parameter("options"));
        }

        let options = Self::parse_options(&options_str);
        let allow_custom = annotation.get_param_as_bool("allow_custom", false);

        Self::new(config, options, allow_custom)
    }

    fn ui_type() -> &'static str {
        "select"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_decorator_line;

    #[test]
    fn test_select_creation() {
        let annotation = parse_decorator_line(r##"-- @ui: select(options="Dark, Light, Auto") type=string"##).unwrap();
        let current_value = WidgetValue::String("Dark".to_string());

        let select = SelectWidget::from_annotation(
            "test-select".to_string(),
            "config.theme".to_string(),
            &annotation,
            current_value,
        ).unwrap();

        assert_eq!(select.options, vec!["Dark", "Light", "Auto"]);
        assert!(!select.allow_custom);
        assert_eq!(select.string_value(), "Dark");
    }

    #[test]
    fn test_select_options_parsing() {
        let options = SelectWidget::parse_options("Dark, Light, Auto");
        assert_eq!(options, vec!["Dark", "Light", "Auto"]);

        let options_with_spaces = SelectWidget::parse_options("  Dark  ,  Light  ,  Auto  ");
        assert_eq!(options_with_spaces, vec!["Dark", "Light", "Auto"]);

        let empty_options = SelectWidget::parse_options("");
        assert!(empty_options.is_empty());
    }

    #[test]
    fn test_select_value_setting() {
        let annotation = parse_decorator_line(r##"-- @ui: select(options="Red, Green, Blue") type=string"##).unwrap();
        let mut select = SelectWidget::from_annotation(
            "test-select".to_string(),
            "config.color".to_string(),
            &annotation,
            WidgetValue::String("Red".to_string()),
        ).unwrap();

        // Test valid option setting
        select.set_value(WidgetValue::String("Green".to_string())).unwrap();
        assert_eq!(select.string_value(), "Green");

        // Test invalid option setting
        let result = select.set_value(WidgetValue::String("Purple".to_string()));
        assert!(result.is_err());

        // Test non-string value
        let result = select.set_value(WidgetValue::Number(42.0));
        assert!(result.is_err());
    }

    #[test]
    fn test_select_custom_values() {
        let annotation = parse_decorator_line(r##"-- @ui: select(options="Red, Green, Blue", allow_custom=true) type=string"##).unwrap();
        let mut select = SelectWidget::from_annotation(
            "test-select".to_string(),
            "config.color".to_string(),
            &annotation,
            WidgetValue::String("Red".to_string()),
        ).unwrap();

        assert!(select.allow_custom);

        // Test custom value with allow_custom=true
        select.set_value(WidgetValue::String("Purple".to_string())).unwrap();
        assert_eq!(select.string_value(), "Purple");
    }

    #[test]
    fn test_select_validation() {
        let annotation = parse_decorator_line(r##"-- @ui: select(options="Yes, No") type=string"##).unwrap();
        let select = SelectWidget::from_annotation(
            "test-select".to_string(),
            "config.enabled".to_string(),
            &annotation,
            WidgetValue::String("Yes".to_string()),
        ).unwrap();

        // Valid values
        assert!(select.validate_value(&WidgetValue::String("Yes".to_string())).is_ok());
        assert!(select.validate_value(&WidgetValue::String("No".to_string())).is_ok());

        // Invalid values
        assert!(select.validate_value(&WidgetValue::String("Maybe".to_string())).is_err());
        assert!(select.validate_value(&WidgetValue::Number(1.0)).is_err());
    }

    #[test]
    fn test_select_render_data() {
        let annotation = parse_decorator_line(r##"-- @ui: select(options="Small, Medium, Large") type=string"##).unwrap();
        let select = SelectWidget::from_annotation(
            "test-select".to_string(),
            "config.size".to_string(),
            &annotation,
            WidgetValue::String("Medium".to_string()),
        ).unwrap();

        let render_data = select.render_data();
        assert!(render_data.contains_key("select"));

        let select_data: SelectRenderData = serde_json::from_value(
            render_data.get("select").unwrap().clone()
        ).unwrap();

        assert_eq!(select_data.options, vec!["Small", "Medium", "Large"]);
        assert_eq!(select_data.current_value, "Medium");
        assert!(!select_data.allow_custom);
    }
}