//! Layout system for organizing widgets into groups and sections
//!
//! Provides structures and logic for organizing widgets in a hierarchical layout.

use crate::{
    error::{Result, WezzteError},
    widgets::traits::{Widget, WidgetValue},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Layout container types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContainerType {
    /// Simple vertical stack of widgets
    Column,
    /// Horizontal row of widgets
    Row,
    /// Tabbed interface
    Tabs,
    /// Collapsible section
    Section,
    /// Card/panel container
    Card,
    /// Grid layout with specified columns
    Grid { columns: usize },
}

/// Layout container that holds widgets or other containers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutContainer {
    /// Container identifier
    pub id: String,
    /// Display title
    pub title: String,
    /// Container type
    pub container_type: ContainerType,
    /// Child widgets (by ID)
    pub widgets: Vec<String>,
    /// Child containers
    pub children: Vec<LayoutContainer>,
    /// Container properties
    pub properties: HashMap<String, serde_json::Value>,
    /// Whether this container is expanded/visible
    pub expanded: bool,
}

impl LayoutContainer {
    /// Create a new layout container
    pub fn new(id: String, title: String, container_type: ContainerType) -> Self {
        Self {
            id,
            title,
            container_type,
            widgets: Vec::new(),
            children: Vec::new(),
            properties: HashMap::new(),
            expanded: true,
        }
    }

    /// Add a widget ID to this container
    pub fn add_widget(&mut self, widget_id: String) {
        if !self.widgets.contains(&widget_id) {
            self.widgets.push(widget_id);
        }
    }

    /// Add a child container
    pub fn add_child(&mut self, child: LayoutContainer) {
        self.children.push(child);
    }

    /// Set a property on this container
    pub fn set_property(&mut self, key: String, value: serde_json::Value) {
        self.properties.insert(key, value);
    }

    /// Get a property value
    pub fn get_property(&self, key: &str) -> Option<&serde_json::Value> {
        self.properties.get(key)
    }

    /// Get all widget IDs recursively
    pub fn all_widget_ids(&self) -> Vec<String> {
        let mut ids = self.widgets.clone();
        for child in &self.children {
            ids.extend(child.all_widget_ids());
        }
        ids
    }

    /// Find a container by ID recursively
    pub fn find_container(&self, id: &str) -> Option<&LayoutContainer> {
        if self.id == id {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.find_container(id) {
                return Some(found);
            }
        }
        None
    }

    /// Find a container by ID recursively (mutable)
    pub fn find_container_mut(&mut self, id: &str) -> Option<&mut LayoutContainer> {
        if self.id == id {
            return Some(self);
        }
        for child in &mut self.children {
            if let Some(found) = child.find_container_mut(id) {
                return Some(found);
            }
        }
        None
    }

    /// Toggle expanded state for collapsible containers
    pub fn toggle_expanded(&mut self) {
        self.expanded = !self.expanded;
    }

    /// Check if this container is empty (no widgets or children)
    pub fn is_empty(&self) -> bool {
        self.widgets.is_empty() && self.children.is_empty()
    }
}

/// Complete layout definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layout {
    /// Layout name/identifier
    pub name: String,
    /// Root container
    pub root: LayoutContainer,
    /// Layout metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Layout {
    /// Create a new layout with a root container
    pub fn new(name: String) -> Self {
        let root = LayoutContainer::new(
            "root".to_string(),
            "Configuration".to_string(),
            ContainerType::Column,
        );

        Self {
            name,
            root,
            metadata: HashMap::new(),
        }
    }

    /// Add a widget to a specific container
    pub fn add_widget_to_container(&mut self, container_id: &str, widget_id: String) -> Result<()> {
        if let Some(container) = self.root.find_container_mut(container_id) {
            container.add_widget(widget_id);
            Ok(())
        } else {
            Err(WezzteError::config(format!(
                "Container '{}' not found",
                container_id
            )))
        }
    }

    /// Add a child container to an existing container
    pub fn add_container_to_parent(&mut self, parent_id: &str, child: LayoutContainer) -> Result<()> {
        if let Some(parent) = self.root.find_container_mut(parent_id) {
            parent.add_child(child);
            Ok(())
        } else {
            Err(WezzteError::config(format!(
                "Parent container '{}' not found",
                parent_id
            )))
        }
    }

    /// Get all widget IDs in the layout
    pub fn all_widget_ids(&self) -> Vec<String> {
        self.root.all_widget_ids()
    }

    /// Set layout metadata
    pub fn set_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
    }

    /// Generate layout structure for frontend
    pub fn serialize_for_frontend(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_default()
    }
}

/// Layout builder for creating complex layouts programmatically
pub struct LayoutBuilder {
    layout: Layout,
}

impl LayoutBuilder {
    /// Create a new layout builder
    pub fn new(name: String) -> Self {
        Self {
            layout: Layout::new(name),
        }
    }

    /// Add a section to the root
    pub fn add_section(mut self, id: String, title: String) -> Self {
        let section = LayoutContainer::new(id.clone(), title, ContainerType::Section);
        self.layout.root.add_child(section);
        self
    }

    /// Add a tab container to the root
    pub fn add_tabs(mut self, id: String, title: String) -> Self {
        let tabs = LayoutContainer::new(id.clone(), title, ContainerType::Tabs);
        self.layout.root.add_child(tabs);
        self
    }

    /// Add a grid container
    pub fn add_grid(mut self, id: String, title: String, columns: usize) -> Self {
        let grid = LayoutContainer::new(
            id.clone(),
            title,
            ContainerType::Grid { columns },
        );
        self.layout.root.add_child(grid);
        self
    }

    /// Add a widget to a container
    pub fn add_widget_to(mut self, container_id: &str, widget_id: String) -> Result<Self> {
        self.layout.add_widget_to_container(container_id, widget_id)?;
        Ok(self)
    }

    /// Build the final layout
    pub fn build(self) -> Layout {
        self.layout
    }
}

/// Auto-layout generator that creates layouts based on widget metadata
pub struct AutoLayoutGenerator;

impl AutoLayoutGenerator {
    /// Generate automatic layout based on widget types and config keys
    pub fn generate_layout(widgets: &HashMap<String, Box<dyn Widget>>) -> Layout {
        let mut layout = Layout::new("Auto-Generated Layout".to_string());
        
        // Group widgets by their config key prefixes
        let groups = Self::group_widgets_by_prefix(widgets);
        
        for (group_name, widget_ids) in groups {
            if widget_ids.len() == 1 {
                // Single widgets go directly to root
                layout.root.add_widget(widget_ids[0].clone());
            } else {
                // Multiple widgets get their own section
                let section_id = format!("section-{}", group_name.replace('.', "-"));
                let section = LayoutContainer::new(
                    section_id,
                    Self::format_section_title(&group_name),
                    ContainerType::Section,
                );
                
                let mut section = section;
                for widget_id in widget_ids {
                    section.add_widget(widget_id);
                }
                
                layout.root.add_child(section);
            }
        }
        
        layout
    }
    
    /// Group widgets by their configuration key prefixes
    fn group_widgets_by_prefix(widgets: &HashMap<String, Box<dyn Widget>>) -> HashMap<String, Vec<String>> {
        let mut groups: HashMap<String, Vec<String>> = HashMap::new();
        
        for (widget_id, widget) in widgets {
            let config_key = &widget.config().config_key;
            
            // Extract the prefix (e.g., "config.colors" from "config.colors.background")
            let prefix = if let Some(dot_pos) = config_key.rfind('.') {
                if config_key.starts_with("config.") {
                    let after_config = &config_key[7..]; // Remove "config."
                    if let Some(next_dot) = after_config.find('.') {
                        format!("config.{}", &after_config[..next_dot])
                    } else {
                        "config".to_string()
                    }
                } else {
                    config_key.clone()
                }
            } else {
                config_key.clone()
            };
            
            groups.entry(prefix).or_default().push(widget_id.clone());
        }
        
        groups
    }
    
    /// Format a section title from a config key prefix
    fn format_section_title(prefix: &str) -> String {
        if prefix == "config" {
            return "General Settings".to_string();
        }
        
        if let Some(after_config) = prefix.strip_prefix("config.") {
            // Convert snake_case or camelCase to Title Case
            after_config
                .replace('_', " ")
                .replace('-', " ")
                .split_whitespace()
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            prefix.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::traits::WidgetConfig;

    #[test]
    fn test_layout_container_creation() {
        let container = LayoutContainer::new(
            "test".to_string(),
            "Test Container".to_string(),
            ContainerType::Section,
        );

        assert_eq!(container.id, "test");
        assert_eq!(container.title, "Test Container");
        assert!(matches!(container.container_type, ContainerType::Section));
        assert!(container.expanded);
    }

    #[test]
    fn test_layout_builder() {
        let layout = LayoutBuilder::new("Test Layout".to_string())
            .add_section("colors".to_string(), "Colors".to_string())
            .add_section("fonts".to_string(), "Fonts".to_string())
            .build();

        assert_eq!(layout.name, "Test Layout");
        assert_eq!(layout.root.children.len(), 2);
        
        let colors_section = layout.root.find_container("colors").unwrap();
        assert_eq!(colors_section.title, "Colors");
    }

    #[test]
    fn test_auto_layout_section_title_formatting() {
        assert_eq!(
            AutoLayoutGenerator::format_section_title("config.colors"),
            "Colors"
        );
        assert_eq!(
            AutoLayoutGenerator::format_section_title("config.font_settings"),
            "Font Settings"
        );
        assert_eq!(
            AutoLayoutGenerator::format_section_title("config"),
            "General Settings"
        );
    }

    #[test]
    fn test_container_widget_management() {
        let mut container = LayoutContainer::new(
            "test".to_string(),
            "Test".to_string(),
            ContainerType::Column,
        );

        container.add_widget("widget1".to_string());
        container.add_widget("widget2".to_string());
        container.add_widget("widget1".to_string()); // Duplicate - should be ignored

        assert_eq!(container.widgets.len(), 2);
        assert!(container.widgets.contains(&"widget1".to_string()));
        assert!(container.widgets.contains(&"widget2".to_string()));
    }
}