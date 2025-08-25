//! Text input widget implementation (placeholder)
//!
//! Provides a text input widget for string value entry.

use crate::{
    ast::Annotation,
    error::{Result, WezzteError},
    widgets::traits::{Widget, WidgetConfig, WidgetFromAnnotation, WidgetValue},
};
use std::collections::HashMap;

/// Text input widget
#[derive(Debug, Clone)]
pub struct TextWidget {
    config: WidgetConfig,
    initial_value: WidgetValue,
}

impl Widget for TextWidget {
    fn config(&self) -> &WidgetConfig {
        &self.config
    }

    fn get_value(&self) -> &WidgetValue {
        &self.config.current_value
    }

    fn set_value(&mut self, value: WidgetValue) -> Result<()> {
        if matches!(value, WidgetValue::String(_)) {
            self.config.current_value = value;
            Ok(())
        } else {
            Err(WezzteError::invalid_parameter("value", "text widget requires string"))
        }
    }

    fn validate_value(&self, value: &WidgetValue) -> Result<()> {
        if matches!(value, WidgetValue::String(_)) {
            Ok(())
        } else {
            Err(WezzteError::invalid_parameter("value", "text widget requires string"))
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
        HashMap::new()
    }
}

impl WidgetFromAnnotation for TextWidget {
    fn from_annotation(
        id: String,
        config_key: String,
        annotation: &Annotation,
        current_value: WidgetValue,
    ) -> Result<Self> {
        let config = WidgetConfig::from_annotation(id, config_key, annotation, current_value.clone());
        Ok(Self { config, initial_value: current_value })
    }

    fn ui_type() -> &'static str {
        "text"
    }
}