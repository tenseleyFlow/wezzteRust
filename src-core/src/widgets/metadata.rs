//! Widget metadata and constraints system
//!
//! Defines metadata that describes widget capabilities, constraints, and properties.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Constraints that can be applied to widget values
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WidgetConstraint {
    /// Minimum numeric value
    Min { value: f64 },
    /// Maximum numeric value  
    Max { value: f64 },
    /// Step/increment for numeric values
    Step { value: f64 },
    /// Minimum string length
    MinLength { length: usize },
    /// Maximum string length
    MaxLength { length: usize },
    /// Regular expression pattern for strings
    Pattern { regex: String },
    /// Allowed values (enumeration)
    Options { values: Vec<String> },
    /// Required field
    Required,
}

/// Properties that widgets can expose to the UI
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WidgetProperty {
    /// Property name
    pub name: String,
    /// Property description
    pub description: String,
    /// Property data type
    pub data_type: String,
    /// Default value if any
    pub default_value: Option<serde_json::Value>,
    /// Whether this property is required
    pub required: bool,
    /// Constraints on this property
    pub constraints: Vec<WidgetConstraint>,
}

impl WidgetProperty {
    /// Create a new numeric property
    pub fn number(name: &str, description: &str, min: Option<f64>, max: Option<f64>, step: Option<f64>) -> Self {
        let mut constraints = Vec::new();
        
        if let Some(min_val) = min {
            constraints.push(WidgetConstraint::Min { value: min_val });
        }
        if let Some(max_val) = max {
            constraints.push(WidgetConstraint::Max { value: max_val });
        }
        if let Some(step_val) = step {
            constraints.push(WidgetConstraint::Step { value: step_val });
        }

        Self {
            name: name.to_string(),
            description: description.to_string(),
            data_type: "number".to_string(),
            default_value: None,
            required: false,
            constraints,
        }
    }

    /// Create a new string property
    pub fn string(name: &str, description: &str, max_length: Option<usize>) -> Self {
        let mut constraints = Vec::new();
        
        if let Some(max_len) = max_length {
            constraints.push(WidgetConstraint::MaxLength { length: max_len });
        }

        Self {
            name: name.to_string(),
            description: description.to_string(),
            data_type: "string".to_string(),
            default_value: None,
            required: false,
            constraints,
        }
    }

    /// Create a new boolean property
    pub fn boolean(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            data_type: "boolean".to_string(),
            default_value: None,
            required: false,
            constraints: vec![],
        }
    }

    /// Create a new options property (select/dropdown)
    pub fn options(name: &str, description: &str, values: Vec<&str>) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            data_type: "string".to_string(),
            default_value: None,
            required: false,
            constraints: vec![WidgetConstraint::Options {
                values: values.into_iter().map(|s| s.to_string()).collect(),
            }],
        }
    }

    /// Mark this property as required
    pub fn required(mut self) -> Self {
        self.required = true;
        self.constraints.push(WidgetConstraint::Required);
        self
    }

    /// Set a default value
    pub fn with_default(mut self, value: serde_json::Value) -> Self {
        self.default_value = Some(value);
        self
    }
}

/// Comprehensive metadata for a widget type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetMetadata {
    /// Widget type identifier (e.g., "slider", "select")
    pub widget_type: String,
    /// Human-readable name
    pub name: String,
    /// Detailed description
    pub description: String,
    /// Category for grouping in UI
    pub category: String,
    /// Properties this widget exposes
    pub properties: Vec<WidgetProperty>,
    /// Examples of usage
    pub examples: Vec<WidgetExample>,
    /// Version when this widget was introduced
    pub version: String,
    /// Whether this is a core widget or plugin
    pub core: bool,
}

/// Example usage of a widget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetExample {
    /// Example title
    pub title: String,
    /// Example description
    pub description: String,
    /// Example decorator line
    pub decorator: String,
    /// Example config line
    pub config_line: String,
}

impl WidgetMetadata {
    /// Create new widget metadata
    pub fn new(
        widget_type: &str,
        name: &str,
        description: &str,
        category: &str,
    ) -> Self {
        Self {
            widget_type: widget_type.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            category: category.to_string(),
            properties: Vec::new(),
            examples: Vec::new(),
            version: "1.0.0".to_string(),
            core: true,
        }
    }

    /// Add a property to this widget
    pub fn with_property(mut self, property: WidgetProperty) -> Self {
        self.properties.push(property);
        self
    }

    /// Add an example to this widget
    pub fn with_example(mut self, example: WidgetExample) -> Self {
        self.examples.push(example);
        self
    }

    /// Get property by name
    pub fn get_property(&self, name: &str) -> Option<&WidgetProperty> {
        self.properties.iter().find(|p| p.name == name)
    }

    /// Get all required properties
    pub fn required_properties(&self) -> Vec<&WidgetProperty> {
        self.properties.iter().filter(|p| p.required).collect()
    }

    /// Generate documentation for this widget
    pub fn generate_docs(&self) -> String {
        let mut docs = format!("# {}\n\n{}\n\n", self.name, self.description);
        
        if !self.properties.is_empty() {
            docs.push_str("## Properties\n\n");
            for prop in &self.properties {
                docs.push_str(&format!(
                    "- **{}** ({}): {}\n",
                    prop.name, prop.data_type, prop.description
                ));
                if prop.required {
                    docs.push_str("  - Required\n");
                }
                for constraint in &prop.constraints {
                    match constraint {
                        WidgetConstraint::Min { value } => docs.push_str(&format!("  - Minimum: {}\n", value)),
                        WidgetConstraint::Max { value } => docs.push_str(&format!("  - Maximum: {}\n", value)),
                        WidgetConstraint::Step { value } => docs.push_str(&format!("  - Step: {}\n", value)),
                        WidgetConstraint::Options { values } => {
                            docs.push_str(&format!("  - Options: {}\n", values.join(", ")));
                        }
                        _ => {}
                    }
                }
                docs.push('\n');
            }
        }

        if !self.examples.is_empty() {
            docs.push_str("## Examples\n\n");
            for example in &self.examples {
                docs.push_str(&format!(
                    "### {}\n{}\n\n```lua\n{}\n{}\n```\n\n",
                    example.title, example.description, example.decorator, example.config_line
                ));
            }
        }

        docs
    }
}

/// Built-in widget metadata definitions
pub struct CoreWidgets;

impl CoreWidgets {
    /// Get metadata for slider widget
    pub fn slider() -> WidgetMetadata {
        WidgetMetadata::new(
            "slider",
            "Slider",
            "A slider widget for selecting numeric values within a range",
            "Input"
        )
        .with_property(
            WidgetProperty::number("min", "Minimum value", None, None, None).required()
        )
        .with_property(
            WidgetProperty::number("max", "Maximum value", None, None, None).required()
        )
        .with_property(
            WidgetProperty::number("step", "Step increment", Some(0.001), None, None)
                .with_default(serde_json::json!(1.0))
        )
        .with_example(WidgetExample {
            title: "Font Size Slider".to_string(),
            description: "Integer slider for font size".to_string(),
            decorator: "-- @ui: slider(min=8, max=72, step=1) type=int".to_string(),
            config_line: "config.font_size = 14".to_string(),
        })
    }

    /// Get metadata for select widget
    pub fn select() -> WidgetMetadata {
        WidgetMetadata::new(
            "select",
            "Select Dropdown",
            "A dropdown selector for choosing from predefined options",
            "Input"
        )
        .with_property(
            WidgetProperty::string("options", "Comma-separated list of options", Some(1000)).required()
        )
        .with_example(WidgetExample {
            title: "Theme Selector".to_string(),
            description: "Select from available themes".to_string(),
            decorator: r#"-- @ui: select(options="Dark, Light, Auto") type=string"#.to_string(),
            config_line: r#"config.color_scheme = "Dark""#.to_string(),
        })
    }

    /// Get metadata for color picker widget
    pub fn color_picker() -> WidgetMetadata {
        WidgetMetadata::new(
            "color_picker",
            "Color Picker",
            "A color picker widget for selecting colors",
            "Input"
        )
        .with_property(
            WidgetProperty::options("format", "Color format", vec!["hex", "rgb", "hsl", "rgba", "hsla"])
                .with_default(serde_json::json!("hex"))
        )
        .with_property(
            WidgetProperty::boolean("alpha", "Enable alpha channel")
                .with_default(serde_json::json!(false))
        )
        .with_example(WidgetExample {
            title: "Background Color".to_string(),
            description: "Pick a background color".to_string(),
            decorator: r#"-- @ui: color_picker(format="hex", alpha=false) type=color"#.to_string(),
            config_line: r##"config.colors.background = "#333333""##.to_string(),
        })
    }
    
    /// Get metadata for theme selector widget
    pub fn theme_selector() -> WidgetMetadata {
        WidgetMetadata::new(
            "theme_selector",
            "Theme Selector",
            "A theme selector widget with visual previews of predefined color schemes",
            "Theme"
        )
        .with_property(
            WidgetProperty::options("themes", "Available theme sets", vec!["builtin", "custom"])
                .with_default(serde_json::json!("builtin"))
        )
        .with_property(
            WidgetProperty::boolean("allow_custom", "Allow custom theme names")
                .with_default(serde_json::json!(false))
        )
        .with_property(
            WidgetProperty::options("filter", "Filter themes by type", vec!["all", "dark", "light"])
                .with_default(serde_json::json!("all"))
        )
        .with_example(WidgetExample {
            title: "Color Scheme Selector".to_string(),
            description: "Choose from built-in color schemes".to_string(),
            decorator: r#"-- @ui: theme_selector(themes="builtin", filter="all") type=string"#.to_string(),
            config_line: r#"config.color_scheme = "dracula""#.to_string(),
        })
    }

    /// Get all core widget metadata
    pub fn all() -> HashMap<String, WidgetMetadata> {
        let mut widgets = HashMap::new();
        
        let slider = Self::slider();
        widgets.insert(slider.widget_type.clone(), slider);
        
        let select = Self::select();
        widgets.insert(select.widget_type.clone(), select);
        
        let color_picker = Self::color_picker();
        widgets.insert(color_picker.widget_type.clone(), color_picker);
        
        let theme_selector = Self::theme_selector();
        widgets.insert(theme_selector.widget_type.clone(), theme_selector);

        widgets
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_property_creation() {
        let prop = WidgetProperty::number("min", "Minimum value", Some(0.0), Some(100.0), Some(1.0))
            .required();
        
        assert_eq!(prop.name, "min");
        assert_eq!(prop.data_type, "number");
        assert!(prop.required);
        assert_eq!(prop.constraints.len(), 4); // min, max, step, required
    }

    #[test]
    fn test_core_widgets_metadata() {
        let slider = CoreWidgets::slider();
        assert_eq!(slider.widget_type, "slider");
        assert!(slider.core);
        
        let min_prop = slider.get_property("min").unwrap();
        assert!(min_prop.required);
        
        let step_prop = slider.get_property("step").unwrap();
        assert!(!step_prop.required);
        assert!(step_prop.default_value.is_some());
    }

    #[test]
    fn test_generate_docs() {
        let slider = CoreWidgets::slider();
        let docs = slider.generate_docs();
        
        assert!(docs.contains("# Slider"));
        assert!(docs.contains("## Properties"));
        assert!(docs.contains("## Examples"));
        assert!(docs.contains("Font Size Slider"));
    }
}