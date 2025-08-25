//! Widget registry for dynamic widget creation and management
//!
//! Provides a type-safe registry system for widgets that can be created from annotations.

use crate::{
    ast::Annotation,
    error::{Result, WezzteError},
    widgets::{
        traits::{Widget, WidgetFromAnnotation, WidgetValue},
        metadata::WidgetMetadata,
    },
};
use std::{
    any::TypeId,
    collections::HashMap,
    sync::{Arc, RwLock},
};

/// Type-erased widget creator function
type WidgetCreator = dyn Fn(String, String, &Annotation, WidgetValue) -> Result<Box<dyn Widget>>
    + Send
    + Sync;

/// Registration entry for a widget type
struct WidgetRegistration {
    /// UI type this widget handles
    ui_type: String,
    /// Widget metadata
    metadata: WidgetMetadata,
    /// Creator function
    creator: Box<WidgetCreator>,
    /// Type information for debugging
    type_id: TypeId,
    /// Type name for debugging
    type_name: &'static str,
}

/// Thread-safe widget registry
pub struct WidgetRegistry {
    /// Registered widgets by UI type
    widgets: Arc<RwLock<HashMap<String, WidgetRegistration>>>,
}

impl WidgetRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            widgets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a widget type
    pub fn register<W>(&self, metadata: WidgetMetadata) -> Result<()>
    where
        W: WidgetFromAnnotation + 'static,
    {
        let ui_type = W::ui_type().to_string();

        // Validate metadata matches UI type
        if metadata.widget_type != ui_type {
            return Err(WezzteError::config(format!(
                "Widget metadata type '{}' doesn't match UI type '{}'",
                metadata.widget_type, ui_type
            )));
        }

        let creator: Box<WidgetCreator> = Box::new(
            |id: String, config_key: String, annotation: &Annotation, current_value: WidgetValue| {
                W::from_annotation(id, config_key, annotation, current_value)
                    .map(|w| Box::new(w) as Box<dyn Widget>)
            },
        );

        let registration = WidgetRegistration {
            ui_type: ui_type.clone(),
            metadata,
            creator,
            type_id: TypeId::of::<W>(),
            type_name: std::any::type_name::<W>(),
        };

        let mut widgets = self.widgets.write().map_err(|e| {
            WezzteError::config(format!("Failed to acquire registry lock: {}", e))
        })?;

        if widgets.contains_key(&ui_type) {
            return Err(WezzteError::config(format!(
                "Widget type '{}' is already registered",
                ui_type
            )));
        }

        widgets.insert(ui_type.clone(), registration);
        tracing::info!("Registered widget: {} -> {}", ui_type, std::any::type_name::<W>());
        
        Ok(())
    }

    /// Create a widget from annotation
    pub fn create_widget(
        &self,
        id: String,
        config_key: String,
        annotation: &Annotation,
        current_value: WidgetValue,
    ) -> Result<Box<dyn Widget>> {
        let ui_type = annotation.ui_type.as_str();
        
        let widgets = self.widgets.read().map_err(|e| {
            WezzteError::config(format!("Failed to acquire registry lock: {}", e))
        })?;

        let registration = widgets.get(ui_type).ok_or_else(|| {
            WezzteError::config(format!("No widget registered for UI type: {}", ui_type))
        })?;

        // Validate annotation against metadata
        self.validate_annotation(annotation, &registration.metadata)?;

        // Create the widget
        (registration.creator)(id, config_key, annotation, current_value)
    }

    /// Get metadata for a widget type
    pub fn get_metadata(&self, ui_type: &str) -> Result<WidgetMetadata> {
        let widgets = self.widgets.read().map_err(|e| {
            WezzteError::config(format!("Failed to acquire registry lock: {}", e))
        })?;

        widgets
            .get(ui_type)
            .map(|reg| reg.metadata.clone())
            .ok_or_else(|| WezzteError::config(format!("No widget registered for UI type: {}", ui_type)))
    }

    /// Get all registered UI types
    pub fn registered_types(&self) -> Result<Vec<String>> {
        let widgets = self.widgets.read().map_err(|e| {
            WezzteError::config(format!("Failed to acquire registry lock: {}", e))
        })?;

        Ok(widgets.keys().cloned().collect())
    }

    /// Get all registered metadata
    pub fn all_metadata(&self) -> Result<HashMap<String, WidgetMetadata>> {
        let widgets = self.widgets.read().map_err(|e| {
            WezzteError::config(format!("Failed to acquire registry lock: {}", e))
        })?;

        Ok(widgets
            .iter()
            .map(|(ui_type, reg)| (ui_type.clone(), reg.metadata.clone()))
            .collect())
    }

    /// Check if a widget type is registered
    pub fn is_registered(&self, ui_type: &str) -> bool {
        self.widgets
            .read()
            .map(|widgets| widgets.contains_key(ui_type))
            .unwrap_or(false)
    }

    /// Get debug information about the registry
    pub fn debug_info(&self) -> Result<String> {
        let widgets = self.widgets.read().map_err(|e| {
            WezzteError::config(format!("Failed to acquire registry lock: {}", e))
        })?;

        let mut info = format!("=== Widget Registry ===\nRegistered widgets: {}\n\n", widgets.len());

        for (ui_type, reg) in widgets.iter() {
            info.push_str(&format!(
                "{}: {} ({})\n  Metadata: {}\n  Properties: {}\n\n",
                ui_type,
                reg.type_name,
                reg.metadata.name,
                reg.metadata.description,
                reg.metadata.properties.len()
            ));
        }

        Ok(info)
    }

    /// Validate annotation against widget metadata
    fn validate_annotation(&self, annotation: &Annotation, metadata: &WidgetMetadata) -> Result<()> {
        // Check required properties
        for prop in metadata.required_properties() {
            if !annotation.has_param(&prop.name) {
                return Err(WezzteError::missing_parameter(format!(
                    "{} (required by {} widget)",
                    prop.name, metadata.name
                )));
            }
        }

        // TODO: Add more sophisticated validation based on constraints
        Ok(())
    }
}

impl Default for WidgetRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for WidgetRegistry {
    fn clone(&self) -> Self {
        Self {
            widgets: Arc::clone(&self.widgets),
        }
    }
}

/// Global widget registry instance
static GLOBAL_REGISTRY: std::sync::OnceLock<WidgetRegistry> = std::sync::OnceLock::new();

/// Get the global widget registry
pub fn global_registry() -> &'static WidgetRegistry {
    GLOBAL_REGISTRY.get_or_init(|| {
        let registry = WidgetRegistry::new();
        
        // Register core widgets (will be done when implementations are available)
        // registry.register::<SliderWidget>(CoreWidgets::slider()).unwrap();
        // registry.register::<SelectWidget>(CoreWidgets::select()).unwrap();
        // registry.register::<ColorPickerWidget>(CoreWidgets::color_picker()).unwrap();
        
        registry
    })
}

/// Convenience macro for registering widgets
#[macro_export]
macro_rules! register_widget {
    ($registry:expr, $widget_type:ty, $metadata:expr) => {
        $registry.register::<$widget_type>($metadata)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::metadata::CoreWidgets;

    // Mock widget for testing
    #[derive(Debug)]
    struct MockWidget {
        config: WidgetConfig,
        initial_value: WidgetValue,
    }

    impl Widget for MockWidget {
        fn config(&self) -> &WidgetConfig {
            &self.config
        }

        fn get_value(&self) -> &WidgetValue {
            &self.config.current_value
        }

        fn set_value(&mut self, value: WidgetValue) -> Result<()> {
            self.config.current_value = value;
            Ok(())
        }

        fn validate_value(&self, _value: &WidgetValue) -> Result<()> {
            Ok(())
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

    impl WidgetFromAnnotation for MockWidget {
        fn from_annotation(
            id: String,
            config_key: String,
            annotation: &Annotation,
            current_value: WidgetValue,
        ) -> Result<Self> {
            let config = WidgetConfig::from_annotation(id, config_key, annotation, current_value.clone());
            Ok(Self {
                config,
                initial_value: current_value,
            })
        }

        fn ui_type() -> &'static str {
            "mock"
        }
    }

    #[test]
    fn test_registry_registration() {
        let registry = WidgetRegistry::new();
        let metadata = WidgetMetadata::new("mock", "Mock Widget", "A mock widget for testing", "Test");

        assert!(registry.register::<MockWidget>(metadata).is_ok());
        assert!(registry.is_registered("mock"));
        
        let types = registry.registered_types().unwrap();
        assert!(types.contains(&"mock".to_string()));
    }

    #[test]
    fn test_duplicate_registration() {
        let registry = WidgetRegistry::new();
        let metadata = WidgetMetadata::new("mock", "Mock Widget", "A mock widget for testing", "Test");

        assert!(registry.register::<MockWidget>(metadata.clone()).is_ok());
        assert!(registry.register::<MockWidget>(metadata).is_err());
    }

    #[test]
    fn test_widget_creation() {
        let registry = WidgetRegistry::new();
        let metadata = WidgetMetadata::new("mock", "Mock Widget", "A mock widget for testing", "Test");
        
        registry.register::<MockWidget>(metadata).unwrap();

        let annotation = crate::parser::parse_decorator_line("-- @ui: mock type=string").unwrap();
        let widget = registry.create_widget(
            "test-id".to_string(),
            "config.test".to_string(),
            &annotation,
            WidgetValue::String("test".to_string()),
        );

        assert!(widget.is_ok());
        let widget = widget.unwrap();
        assert_eq!(widget.config().id, "test-id");
        assert_eq!(widget.config().config_key, "config.test");
    }
}