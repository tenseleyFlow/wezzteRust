//! Slider widget implementation
//!
//! Provides a slider widget for selecting numeric values within a range.

use crate::{
    ast::Annotation,
    error::{Result, WezzteError},
    widgets::traits::{Widget, WidgetConfig, WidgetFromAnnotation, WidgetValue},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Slider widget for numeric value selection
#[derive(Debug, Clone)]
pub struct SliderWidget {
    config: WidgetConfig,
    initial_value: WidgetValue,
    min_value: f64,
    max_value: f64,
    step: f64,
    is_integer: bool,
}

/// Render data specific to sliders
#[derive(Debug, Serialize, Deserialize)]
pub struct SliderRenderData {
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub current_value: f64,
    pub is_integer: bool,
    pub precision: Option<usize>,
}

impl SliderWidget {
    /// Create a new slider widget
    pub fn new(
        config: WidgetConfig,
        min_value: f64,
        max_value: f64,
        step: f64,
        is_integer: bool,
    ) -> Result<Self> {
        if min_value >= max_value {
            return Err(WezzteError::invalid_parameter(
                "min/max",
                "min value must be less than max value",
            ));
        }

        if step <= 0.0 {
            return Err(WezzteError::invalid_parameter(
                "step",
                "step must be positive",
            ));
        }

        // Validate current value is within range
        if let WidgetValue::Number(val) = &config.current_value {
            if *val < min_value || *val > max_value {
                return Err(WezzteError::invalid_parameter(
                    "current_value",
                    format!("value {} is outside range [{}, {}]", val, min_value, max_value),
                ));
            }
        } else {
            return Err(WezzteError::invalid_parameter(
                "current_value",
                "slider requires numeric value",
            ));
        }

        let initial_value = config.current_value.clone();

        Ok(Self {
            config,
            initial_value,
            min_value,
            max_value,
            step,
            is_integer,
        })
    }

    /// Get the numeric value, ensuring it's within bounds
    pub fn numeric_value(&self) -> f64 {
        match &self.config.current_value {
            WidgetValue::Number(val) => (*val).clamp(self.min_value, self.max_value),
            _ => self.min_value, // Fallback
        }
    }

    /// Snap value to step increment
    fn snap_to_step(&self, value: f64) -> f64 {
        let snapped = ((value - self.min_value) / self.step).round() * self.step + self.min_value;
        snapped.clamp(self.min_value, self.max_value)
    }

    /// Calculate precision for display
    fn calculate_precision(&self) -> Option<usize> {
        if self.is_integer {
            Some(0)
        } else {
            // Calculate precision from step size
            let step_str = format!("{}", self.step);
            if let Some(dot_pos) = step_str.find('.') {
                Some(step_str.len() - dot_pos - 1)
            } else {
                Some(0)
            }
        }
    }
}

impl Widget for SliderWidget {
    fn config(&self) -> &WidgetConfig {
        &self.config
    }

    fn get_value(&self) -> &WidgetValue {
        &self.config.current_value
    }

    fn set_value(&mut self, value: WidgetValue) -> Result<()> {
        if let WidgetValue::Number(num_val) = value {
            let snapped = self.snap_to_step(num_val);
            let final_val = if self.is_integer {
                snapped.round()
            } else {
                snapped
            };

            self.config.current_value = WidgetValue::Number(final_val);
            Ok(())
        } else {
            Err(WezzteError::invalid_parameter(
                "value",
                "slider requires numeric value",
            ))
        }
    }

    fn validate_value(&self, value: &WidgetValue) -> Result<()> {
        if let WidgetValue::Number(num_val) = value {
            if *num_val < self.min_value || *num_val > self.max_value {
                return Err(WezzteError::invalid_parameter(
                    "value",
                    format!("value {} is outside range [{}, {}]", num_val, self.min_value, self.max_value),
                ));
            }
            Ok(())
        } else {
            Err(WezzteError::invalid_parameter(
                "value",
                "slider requires numeric value",
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
        let render_data = SliderRenderData {
            min: self.min_value,
            max: self.max_value,
            step: self.step,
            current_value: self.numeric_value(),
            is_integer: self.is_integer,
            precision: self.calculate_precision(),
        };

        let mut data = HashMap::new();
        data.insert(
            "slider".to_string(),
            serde_json::to_value(render_data).unwrap_or_default(),
        );
        data
    }
}

impl WidgetFromAnnotation for SliderWidget {
    fn from_annotation(
        id: String,
        config_key: String,
        annotation: &Annotation,
        current_value: WidgetValue,
    ) -> Result<Self> {
        let config = WidgetConfig::from_annotation(id, config_key, annotation, current_value);

        // Extract required parameters
        let min_value = annotation.get_param_as_number("min", f64::NEG_INFINITY);
        let max_value = annotation.get_param_as_number("max", f64::INFINITY);

        if min_value == f64::NEG_INFINITY {
            return Err(WezzteError::missing_parameter("min"));
        }
        if max_value == f64::INFINITY {
            return Err(WezzteError::missing_parameter("max"));
        }

        let step = annotation.get_param_as_number("step", 1.0);
        let data_type = annotation.get_data_type();
        let is_integer = data_type == "int" || data_type == "integer";

        Self::new(config, min_value, max_value, step, is_integer)
    }

    fn ui_type() -> &'static str {
        "slider"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_decorator_line;

    #[test]
    fn test_slider_creation() {
        let annotation = parse_decorator_line("-- @ui: slider(min=0, max=100, step=1) type=int").unwrap();
        let current_value = WidgetValue::Number(50.0);

        let slider = SliderWidget::from_annotation(
            "test-slider".to_string(),
            "config.test".to_string(),
            &annotation,
            current_value,
        ).unwrap();

        assert_eq!(slider.min_value, 0.0);
        assert_eq!(slider.max_value, 100.0);
        assert_eq!(slider.step, 1.0);
        assert!(slider.is_integer);
        assert_eq!(slider.numeric_value(), 50.0);
    }

    #[test]
    fn test_slider_value_setting() {
        let annotation = parse_decorator_line("-- @ui: slider(min=0, max=100, step=5) type=float").unwrap();
        let mut slider = SliderWidget::from_annotation(
            "test-slider".to_string(),
            "config.test".to_string(),
            &annotation,
            WidgetValue::Number(25.0),
        ).unwrap();

        // Test normal value setting
        slider.set_value(WidgetValue::Number(47.3)).unwrap();
        assert_eq!(slider.numeric_value(), 45.0); // Snapped to step

        // Test boundary clamping
        slider.set_value(WidgetValue::Number(150.0)).unwrap();
        assert_eq!(slider.numeric_value(), 100.0); // Clamped to max

        slider.set_value(WidgetValue::Number(-10.0)).unwrap();
        assert_eq!(slider.numeric_value(), 0.0); // Clamped to min
    }

    #[test]
    fn test_slider_validation() {
        let annotation = parse_decorator_line("-- @ui: slider(min=0, max=100) type=int").unwrap();
        let slider = SliderWidget::from_annotation(
            "test-slider".to_string(),
            "config.test".to_string(),
            &annotation,
            WidgetValue::Number(50.0),
        ).unwrap();

        // Valid values
        assert!(slider.validate_value(&WidgetValue::Number(25.0)).is_ok());
        assert!(slider.validate_value(&WidgetValue::Number(0.0)).is_ok());
        assert!(slider.validate_value(&WidgetValue::Number(100.0)).is_ok());

        // Invalid values
        assert!(slider.validate_value(&WidgetValue::Number(-5.0)).is_err());
        assert!(slider.validate_value(&WidgetValue::Number(150.0)).is_err());
        assert!(slider.validate_value(&WidgetValue::String("test".to_string())).is_err());
    }

    #[test]
    fn test_slider_render_data() {
        let annotation = parse_decorator_line("-- @ui: slider(min=0, max=100, step=0.5) type=float").unwrap();
        let slider = SliderWidget::from_annotation(
            "test-slider".to_string(),
            "config.test".to_string(),
            &annotation,
            WidgetValue::Number(25.5),
        ).unwrap();

        let render_data = slider.render_data();
        assert!(render_data.contains_key("slider"));

        let slider_data: SliderRenderData = serde_json::from_value(
            render_data.get("slider").unwrap().clone()
        ).unwrap();

        assert_eq!(slider_data.min, 0.0);
        assert_eq!(slider_data.max, 100.0);
        assert_eq!(slider_data.step, 0.5);
        assert_eq!(slider_data.current_value, 25.5);
        assert!(!slider_data.is_integer);
        assert_eq!(slider_data.precision, Some(1));
    }
}