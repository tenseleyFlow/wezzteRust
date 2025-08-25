//! Theme selector widget implementation
//!
//! Provides a theme selector widget for choosing from predefined themes with color previews.

use crate::{
    ast::Annotation,
    error::{Result, WezzteError},
    widgets::traits::{Widget, WidgetConfig, WidgetFromAnnotation, WidgetValue},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Theme definition with colors and metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub colors: HashMap<String, String>,
    pub is_dark: bool,
    pub author: Option<String>,
}

/// Built-in themes for WezTerm
impl Theme {
    pub fn builtin_themes() -> Vec<Theme> {
        vec![
            Theme {
                name: "dracula".to_string(),
                display_name: "Dracula".to_string(),
                description: "Dark theme inspired by Dracula".to_string(),
                colors: [
                    ("background", "#282a36"),
                    ("foreground", "#f8f8f2"),
                    ("cursor", "#f8f8f2"),
                    ("selection", "#44475a"),
                    ("black", "#000000"),
                    ("red", "#ff5555"),
                    ("green", "#50fa7b"),
                    ("yellow", "#f1fa8c"),
                    ("blue", "#bd93f9"),
                    ("magenta", "#ff79c6"),
                    ("cyan", "#8be9fd"),
                    ("white", "#bfbfbf"),
                ].iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
                is_dark: true,
                author: Some("Dracula Team".to_string()),
            },
            Theme {
                name: "gruvbox-dark".to_string(),
                display_name: "Gruvbox Dark".to_string(),
                description: "Retro groove color scheme".to_string(),
                colors: [
                    ("background", "#282828"),
                    ("foreground", "#ebdbb2"),
                    ("cursor", "#ebdbb2"),
                    ("selection", "#504945"),
                    ("black", "#282828"),
                    ("red", "#cc241d"),
                    ("green", "#98971a"),
                    ("yellow", "#d79921"),
                    ("blue", "#458588"),
                    ("magenta", "#b16286"),
                    ("cyan", "#689d6a"),
                    ("white", "#a89984"),
                ].iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
                is_dark: true,
                author: Some("morhetz".to_string()),
            },
            Theme {
                name: "solarized-light".to_string(),
                display_name: "Solarized Light".to_string(),
                description: "Precision colors for machines and people".to_string(),
                colors: [
                    ("background", "#fdf6e3"),
                    ("foreground", "#657b83"),
                    ("cursor", "#657b83"),
                    ("selection", "#eee8d5"),
                    ("black", "#073642"),
                    ("red", "#dc322f"),
                    ("green", "#859900"),
                    ("yellow", "#b58900"),
                    ("blue", "#268bd2"),
                    ("magenta", "#d33682"),
                    ("cyan", "#2aa198"),
                    ("white", "#eee8d5"),
                ].iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
                is_dark: false,
                author: Some("Ethan Schoonover".to_string()),
            },
            Theme {
                name: "tokyonight".to_string(),
                display_name: "Tokyo Night".to_string(),
                description: "A clean, dark theme that celebrates Tokyo's neon nights".to_string(),
                colors: [
                    ("background", "#1a1b26"),
                    ("foreground", "#c0caf5"),
                    ("cursor", "#c0caf5"),
                    ("selection", "#364a82"),
                    ("black", "#15161e"),
                    ("red", "#f7768e"),
                    ("green", "#9ece6a"),
                    ("yellow", "#e0af68"),
                    ("blue", "#7aa2f7"),
                    ("magenta", "#bb9af7"),
                    ("cyan", "#7dcfff"),
                    ("white", "#a9b1d6"),
                ].iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
                is_dark: true,
                author: Some("folke".to_string()),
            },
            Theme {
                name: "catppuccin-mocha".to_string(),
                display_name: "Catppuccin Mocha".to_string(),
                description: "Soothing pastel theme for the high-spirited!".to_string(),
                colors: [
                    ("background", "#1e1e2e"),
                    ("foreground", "#cdd6f4"),
                    ("cursor", "#f5e0dc"),
                    ("selection", "#45475a"),
                    ("black", "#45475a"),
                    ("red", "#f38ba8"),
                    ("green", "#a6e3a1"),
                    ("yellow", "#f9e2af"),
                    ("blue", "#89b4fa"),
                    ("magenta", "#f5c2e7"),
                    ("cyan", "#94e2d5"),
                    ("white", "#bac2de"),
                ].iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
                is_dark: true,
                author: Some("Catppuccin".to_string()),
            },
        ]
    }

    pub fn find_theme(name: &str) -> Option<Theme> {
        Self::builtin_themes().into_iter().find(|t| t.name == name)
    }
}

/// Theme selector widget for choosing predefined themes
#[derive(Debug, Clone)]
pub struct ThemeSelectorWidget {
    config: WidgetConfig,
    initial_value: WidgetValue,
    available_themes: Vec<Theme>,
    allow_custom: bool,
}

/// Render data specific to theme selectors
#[derive(Debug, Serialize, Deserialize)]
pub struct ThemeSelectorRenderData {
    pub available_themes: Vec<Theme>,
    pub current_theme: Option<Theme>,
    pub allow_custom: bool,
    pub filter_dark_themes: Option<bool>,
}

impl ThemeSelectorWidget {
    /// Create a new theme selector widget
    pub fn new(
        config: WidgetConfig,
        available_themes: Vec<Theme>,
        allow_custom: bool,
    ) -> Result<Self> {
        // Validate current value is a string
        if !matches!(config.current_value, WidgetValue::String(_)) {
            return Err(WezzteError::invalid_parameter(
                "current_value",
                "theme selector requires string value",
            ));
        }

        let initial_value = config.current_value.clone();

        Ok(Self {
            config,
            initial_value,
            available_themes,
            allow_custom,
        })
    }

    /// Get the current theme name
    pub fn current_theme_name(&self) -> &str {
        match &self.config.current_value {
            WidgetValue::String(s) => s,
            _ => "default",
        }
    }

    /// Get the current theme definition
    pub fn current_theme(&self) -> Option<Theme> {
        let theme_name = self.current_theme_name();
        self.available_themes.iter().find(|t| t.name == theme_name).cloned()
    }

    /// Validate a theme name
    pub fn validate_theme_name(&self, theme_name: &str) -> Result<()> {
        if self.allow_custom {
            return Ok(()); // Allow any string if custom themes are enabled
        }

        if self.available_themes.iter().any(|t| t.name == theme_name) {
            Ok(())
        } else {
            Err(WezzteError::invalid_parameter(
                "theme_name",
                format!("theme '{}' not found in available themes", theme_name),
            ))
        }
    }

    /// Filter themes by dark/light preference
    pub fn filter_themes(&self, dark_only: Option<bool>) -> Vec<&Theme> {
        match dark_only {
            Some(true) => self.available_themes.iter().filter(|t| t.is_dark).collect(),
            Some(false) => self.available_themes.iter().filter(|t| !t.is_dark).collect(),
            None => self.available_themes.iter().collect(),
        }
    }

    /// Get theme preview colors (limited set for UI display)
    pub fn get_preview_colors(&self, theme: &Theme) -> Vec<(String, String)> {
        let preview_keys = ["background", "foreground", "red", "green", "blue", "yellow"];
        preview_keys
            .iter()
            .filter_map(|key| {
                theme.colors.get(*key).map(|color| (key.to_string(), color.clone()))
            })
            .collect()
    }
}

impl Widget for ThemeSelectorWidget {
    fn config(&self) -> &WidgetConfig {
        &self.config
    }

    fn get_value(&self) -> &WidgetValue {
        &self.config.current_value
    }

    fn set_value(&mut self, value: WidgetValue) -> Result<()> {
        if let WidgetValue::String(theme_name) = value {
            self.validate_theme_name(&theme_name)?;
            self.config.current_value = WidgetValue::String(theme_name);
            Ok(())
        } else {
            Err(WezzteError::invalid_parameter(
                "value",
                "theme selector requires string value",
            ))
        }
    }

    fn validate_value(&self, value: &WidgetValue) -> Result<()> {
        if let WidgetValue::String(theme_name) = value {
            self.validate_theme_name(theme_name)
        } else {
            Err(WezzteError::invalid_parameter(
                "value",
                "theme selector requires string value",
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
        let render_data = ThemeSelectorRenderData {
            available_themes: self.available_themes.clone(),
            current_theme: self.current_theme(),
            allow_custom: self.allow_custom,
            filter_dark_themes: None, // Could be set via parameters
        };

        let mut data = HashMap::new();
        data.insert(
            "theme_selector".to_string(),
            serde_json::to_value(render_data).unwrap_or_default(),
        );
        data
    }
}

impl WidgetFromAnnotation for ThemeSelectorWidget {
    fn from_annotation(
        id: String,
        config_key: String,
        annotation: &Annotation,
        current_value: WidgetValue,
    ) -> Result<Self> {
        let config = WidgetConfig::from_annotation(id, config_key, annotation, current_value);

        // Get available themes - could be customized via parameters
        let theme_names = annotation.get_param_as_string("themes", "builtin");
        let available_themes = if theme_names == "builtin" {
            Theme::builtin_themes()
        } else {
            // Could parse custom theme list from parameter
            Theme::builtin_themes()
        };

        let allow_custom = annotation.get_param_as_bool("allow_custom", false);

        Self::new(config, available_themes, allow_custom)
    }

    fn ui_type() -> &'static str {
        "theme_selector"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_decorator_line;

    #[test]
    fn test_builtin_themes() {
        let themes = Theme::builtin_themes();
        assert!(!themes.is_empty());
        
        // Check that dracula theme exists
        let dracula = themes.iter().find(|t| t.name == "dracula").unwrap();
        assert_eq!(dracula.display_name, "Dracula");
        assert!(dracula.is_dark);
        assert!(dracula.colors.contains_key("background"));
    }

    #[test]
    fn test_theme_selector_creation() {
        let annotation = parse_decorator_line(r##"-- @ui: theme_selector(themes="builtin") type=string"##).unwrap();
        let current_value = WidgetValue::String("dracula".to_string());

        let theme_selector = ThemeSelectorWidget::from_annotation(
            "test-theme".to_string(),
            "config.color_scheme".to_string(),
            &annotation,
            current_value,
        ).unwrap();

        assert_eq!(theme_selector.current_theme_name(), "dracula");
        assert!(theme_selector.current_theme().is_some());
        assert!(!theme_selector.allow_custom);
    }

    #[test]
    fn test_theme_validation() {
        let themes = Theme::builtin_themes();
        let config = WidgetConfig {
            id: "test".to_string(),
            config_key: "config.theme".to_string(),
            label: "Theme".to_string(),
            current_value: WidgetValue::String("dracula".to_string()),
            enabled: true,
        };

        let theme_selector = ThemeSelectorWidget::new(config, themes, false).unwrap();

        // Valid theme
        assert!(theme_selector.validate_value(&WidgetValue::String("dracula".to_string())).is_ok());
        
        // Invalid theme
        assert!(theme_selector.validate_value(&WidgetValue::String("nonexistent".to_string())).is_err());
        
        // Non-string value
        assert!(theme_selector.validate_value(&WidgetValue::Number(42.0)).is_err());
    }

    #[test]
    fn test_theme_filtering() {
        let themes = Theme::builtin_themes();
        let config = WidgetConfig {
            id: "test".to_string(),
            config_key: "config.theme".to_string(),
            label: "Theme".to_string(),
            current_value: WidgetValue::String("dracula".to_string()),
            enabled: true,
        };

        let theme_selector = ThemeSelectorWidget::new(config, themes, false).unwrap();

        let dark_themes = theme_selector.filter_themes(Some(true));
        let light_themes = theme_selector.filter_themes(Some(false));
        let all_themes = theme_selector.filter_themes(None);

        assert!(!dark_themes.is_empty());
        assert!(!light_themes.is_empty());
        assert_eq!(all_themes.len(), theme_selector.available_themes.len());
        
        // Check that filtering works correctly
        assert!(dark_themes.iter().all(|t| t.is_dark));
        assert!(light_themes.iter().all(|t| !t.is_dark));
    }

    #[test]
    fn test_theme_preview_colors() {
        let themes = Theme::builtin_themes();
        let config = WidgetConfig {
            id: "test".to_string(),
            config_key: "config.theme".to_string(),
            label: "Theme".to_string(),
            current_value: WidgetValue::String("dracula".to_string()),
            enabled: true,
        };

        let theme_selector = ThemeSelectorWidget::new(config, themes, false).unwrap();
        
        if let Some(theme) = theme_selector.current_theme() {
            let preview_colors = theme_selector.get_preview_colors(&theme);
            assert!(!preview_colors.is_empty());
            
            // Should have background and foreground at minimum
            assert!(preview_colors.iter().any(|(key, _)| key == "background"));
            assert!(preview_colors.iter().any(|(key, _)| key == "foreground"));
        }
    }
}