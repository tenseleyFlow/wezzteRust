//! Color picker widget implementation
//!
//! Provides a color picker widget for selecting colors in various formats.

use crate::{
    ast::Annotation,
    error::{Result, WezzteError},
    widgets::traits::{Widget, WidgetConfig, WidgetFromAnnotation, WidgetValue},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported color formats
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorFormat {
    Hex,
    Rgb,
    Rgba,
    Hsl,
    Hsla,
}

impl ColorFormat {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "rgb" => ColorFormat::Rgb,
            "rgba" => ColorFormat::Rgba,
            "hsl" => ColorFormat::Hsl,
            "hsla" => ColorFormat::Hsla,
            _ => ColorFormat::Hex, // Default
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            ColorFormat::Hex => "hex",
            ColorFormat::Rgb => "rgb",
            ColorFormat::Rgba => "rgba",
            ColorFormat::Hsl => "hsl",
            ColorFormat::Hsla => "hsla",
        }
    }
}

/// Color picker widget for color selection
#[derive(Debug, Clone)]
pub struct ColorPickerWidget {
    config: WidgetConfig,
    initial_value: WidgetValue,
    format: ColorFormat,
    alpha_enabled: bool,
}

/// Render data specific to color pickers
#[derive(Debug, Serialize, Deserialize)]
pub struct ColorPickerRenderData {
    pub format: ColorFormat,
    pub current_value: String,
    pub alpha_enabled: bool,
    pub parsed_color: Option<ParsedColor>,
}

/// Parsed color representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedColor {
    pub hex: String,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f32,
}

impl ColorPickerWidget {
    /// Create a new color picker widget
    pub fn new(
        config: WidgetConfig,
        format: ColorFormat,
        alpha_enabled: bool,
    ) -> Result<Self> {
        // Validate current value is a string
        if !matches!(config.current_value, WidgetValue::String(_)) {
            return Err(WezzteError::invalid_parameter(
                "current_value",
                "color picker requires string value",
            ));
        }

        let initial_value = config.current_value.clone();

        let widget = Self {
            config,
            initial_value,
            format,
            alpha_enabled,
        };

        // Validate the current color value
        widget.validate_value(widget.get_value())?;

        Ok(widget)
    }

    /// Get the color as a string
    pub fn color_string(&self) -> &str {
        match &self.config.current_value {
            WidgetValue::String(s) => s,
            _ => "#000000", // Fallback
        }
    }

    /// Parse a hex color string into RGB components
    pub fn parse_hex_color(hex: &str) -> Option<ParsedColor> {
        let hex = hex.trim_start_matches('#');
        
        if hex.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                return Some(ParsedColor {
                    hex: format!("#{}", hex),
                    r,
                    g,
                    b,
                    a: 1.0,
                });
            }
        }
        
        if hex.len() == 8 {
            if let (Ok(r), Ok(g), Ok(b), Ok(a)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
                u8::from_str_radix(&hex[6..8], 16),
            ) {
                return Some(ParsedColor {
                    hex: format!("#{}", hex),
                    r,
                    g,
                    b,
                    a: a as f32 / 255.0,
                });
            }
        }
        
        None
    }

    /// Validate a color string based on the current format
    pub fn validate_color_string(&self, color: &str) -> Result<()> {
        match self.format {
            ColorFormat::Hex => {
                if !color.starts_with('#') {
                    return Err(WezzteError::invalid_parameter(
                        "color",
                        "hex colors must start with #",
                    ));
                }
                
                let hex_part = &color[1..];
                if hex_part.len() != 6 && hex_part.len() != 8 {
                    return Err(WezzteError::invalid_parameter(
                        "color",
                        "hex colors must be 6 or 8 characters after #",
                    ));
                }
                
                if hex_part.chars().any(|c| !c.is_ascii_hexdigit()) {
                    return Err(WezzteError::invalid_parameter(
                        "color",
                        "hex colors must contain only hexadecimal characters",
                    ));
                }
                
                if hex_part.len() == 8 && !self.alpha_enabled {
                    return Err(WezzteError::invalid_parameter(
                        "color",
                        "alpha channel not enabled for this color picker",
                    ));
                }
            }
            ColorFormat::Rgb => {
                if !color.starts_with("rgb(") || !color.ends_with(')') {
                    return Err(WezzteError::invalid_parameter(
                        "color",
                        "RGB colors must be in format rgb(r, g, b)",
                    ));
                }
            }
            ColorFormat::Rgba => {
                if !color.starts_with("rgba(") || !color.ends_with(')') {
                    return Err(WezzteError::invalid_parameter(
                        "color",
                        "RGBA colors must be in format rgba(r, g, b, a)",
                    ));
                }
            }
            ColorFormat::Hsl => {
                if !color.starts_with("hsl(") || !color.ends_with(')') {
                    return Err(WezzteError::invalid_parameter(
                        "color",
                        "HSL colors must be in format hsl(h, s%, l%)",
                    ));
                }
            }
            ColorFormat::Hsla => {
                if !color.starts_with("hsla(") || !color.ends_with(')') {
                    return Err(WezzteError::invalid_parameter(
                        "color",
                        "HSLA colors must be in format hsla(h, s%, l%, a)",
                    ));
                }
            }
        }
        
        Ok(())
    }

    /// Parse RGB color string into components
    pub fn parse_rgb_color(rgb: &str) -> Option<ParsedColor> {
        if !rgb.starts_with("rgb(") || !rgb.ends_with(')') {
            return None;
        }
        
        let inner = &rgb[4..rgb.len() - 1];
        let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
        
        if parts.len() != 3 {
            return None;
        }
        
        let r = parts[0].parse::<u8>().ok()?;
        let g = parts[1].parse::<u8>().ok()?;
        let b = parts[2].parse::<u8>().ok()?;
        
        Some(ParsedColor {
            hex: format!("#{:02x}{:02x}{:02x}", r, g, b),
            r,
            g,
            b,
            a: 1.0,
        })
    }
    
    /// Parse RGBA color string into components
    pub fn parse_rgba_color(rgba: &str) -> Option<ParsedColor> {
        if !rgba.starts_with("rgba(") || !rgba.ends_with(')') {
            return None;
        }
        
        let inner = &rgba[5..rgba.len() - 1];
        let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
        
        if parts.len() != 4 {
            return None;
        }
        
        let r = parts[0].parse::<u8>().ok()?;
        let g = parts[1].parse::<u8>().ok()?;
        let b = parts[2].parse::<u8>().ok()?;
        let a = parts[3].parse::<f32>().ok()?;
        
        Some(ParsedColor {
            hex: format!("#{:02x}{:02x}{:02x}{:02x}", r, g, b, (a * 255.0) as u8),
            r,
            g,
            b,
            a,
        })
    }
    
    /// Parse HSL color string into RGB components
    pub fn parse_hsl_color(hsl: &str) -> Option<ParsedColor> {
        if !hsl.starts_with("hsl(") || !hsl.ends_with(')') {
            return None;
        }
        
        let inner = &hsl[4..hsl.len() - 1];
        let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
        
        if parts.len() != 3 {
            return None;
        }
        
        let h = parts[0].parse::<f32>().ok()? / 360.0;
        let s = parts[1].trim_end_matches('%').parse::<f32>().ok()? / 100.0;
        let l = parts[2].trim_end_matches('%').parse::<f32>().ok()? / 100.0;
        
        let (r, g, b) = Self::hsl_to_rgb(h, s, l);
        
        Some(ParsedColor {
            hex: format!("#{:02x}{:02x}{:02x}", r, g, b),
            r,
            g,
            b,
            a: 1.0,
        })
    }
    
    /// Convert HSL to RGB
    fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;
        
        let (r_prime, g_prime, b_prime) = if h < 1.0 / 6.0 {
            (c, x, 0.0)
        } else if h < 2.0 / 6.0 {
            (x, c, 0.0)
        } else if h < 3.0 / 6.0 {
            (0.0, c, x)
        } else if h < 4.0 / 6.0 {
            (0.0, x, c)
        } else if h < 5.0 / 6.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };
        
        let r = ((r_prime + m) * 255.0).round() as u8;
        let g = ((g_prime + m) * 255.0).round() as u8;
        let b = ((b_prime + m) * 255.0).round() as u8;
        
        (r, g, b)
    }

    /// Parse the current color value
    pub fn parse_current_color(&self) -> Option<ParsedColor> {
        let color_str = self.color_string();
        
        match self.format {
            ColorFormat::Hex => Self::parse_hex_color(color_str),
            ColorFormat::Rgb => Self::parse_rgb_color(color_str),
            ColorFormat::Rgba => Self::parse_rgba_color(color_str),
            ColorFormat::Hsl => Self::parse_hsl_color(color_str),
            ColorFormat::Hsla => {
                // Similar to HSL but with alpha - implement if needed
                Self::parse_hsl_color(color_str)
            }
        }
    }
    
    /// Convert current color to specified format
    pub fn convert_to_format(&self, target_format: &ColorFormat) -> String {
        if let Some(parsed) = self.parse_current_color() {
            match target_format {
                ColorFormat::Hex => {
                    if self.alpha_enabled && parsed.a != 1.0 {
                        format!("#{:02x}{:02x}{:02x}{:02x}", 
                               parsed.r, parsed.g, parsed.b, (parsed.a * 255.0) as u8)
                    } else {
                        format!("#{:02x}{:02x}{:02x}", parsed.r, parsed.g, parsed.b)
                    }
                }
                ColorFormat::Rgb => {
                    format!("rgb({}, {}, {})", parsed.r, parsed.g, parsed.b)
                }
                ColorFormat::Rgba => {
                    format!("rgba({}, {}, {}, {:.2})", parsed.r, parsed.g, parsed.b, parsed.a)
                }
                ColorFormat::Hsl => {
                    let (h, s, l) = Self::rgb_to_hsl(parsed.r, parsed.g, parsed.b);
                    format!("hsl({:.0}, {:.0}%, {:.0}%)", h * 360.0, s * 100.0, l * 100.0)
                }
                ColorFormat::Hsla => {
                    let (h, s, l) = Self::rgb_to_hsl(parsed.r, parsed.g, parsed.b);
                    format!("hsla({:.0}, {:.0}%, {:.0}%, {:.2})", h * 360.0, s * 100.0, l * 100.0, parsed.a)
                }
            }
        } else {
            self.color_string().to_string()
        }
    }
    
    /// Convert RGB to HSL
    fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
        let r = r as f32 / 255.0;
        let g = g as f32 / 255.0;
        let b = b as f32 / 255.0;
        
        let max = r.max(g.max(b));
        let min = r.min(g.min(b));
        let delta = max - min;
        
        let l = (max + min) / 2.0;
        
        if delta == 0.0 {
            return (0.0, 0.0, l); // Grayscale
        }
        
        let s = if l < 0.5 {
            delta / (max + min)
        } else {
            delta / (2.0 - max - min)
        };
        
        let h = if max == r {
            ((g - b) / delta + if g < b { 6.0 } else { 0.0 }) / 6.0
        } else if max == g {
            ((b - r) / delta + 2.0) / 6.0
        } else {
            ((r - g) / delta + 4.0) / 6.0
        };
        
        (h, s, l)
    }
}

impl Widget for ColorPickerWidget {
    fn config(&self) -> &WidgetConfig {
        &self.config
    }

    fn get_value(&self) -> &WidgetValue {
        &self.config.current_value
    }

    fn set_value(&mut self, value: WidgetValue) -> Result<()> {
        if let WidgetValue::String(color_str) = value {
            self.validate_color_string(&color_str)?;
            self.config.current_value = WidgetValue::String(color_str);
            Ok(())
        } else {
            Err(WezzteError::invalid_parameter(
                "value",
                "color picker requires string value",
            ))
        }
    }

    fn validate_value(&self, value: &WidgetValue) -> Result<()> {
        if let WidgetValue::String(color_str) = value {
            self.validate_color_string(color_str)
        } else {
            Err(WezzteError::invalid_parameter(
                "value",
                "color picker requires string value",
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
        let render_data = ColorPickerRenderData {
            format: self.format.clone(),
            current_value: self.color_string().to_string(),
            alpha_enabled: self.alpha_enabled,
            parsed_color: self.parse_current_color(),
        };

        let mut data = HashMap::new();
        data.insert(
            "color_picker".to_string(),
            serde_json::to_value(render_data).unwrap_or_default(),
        );
        data
    }
}

impl WidgetFromAnnotation for ColorPickerWidget {
    fn from_annotation(
        id: String,
        config_key: String,
        annotation: &Annotation,
        current_value: WidgetValue,
    ) -> Result<Self> {
        let config = WidgetConfig::from_annotation(id, config_key, annotation, current_value);

        let format_str = annotation.get_param_as_string("format", "hex");
        let format = ColorFormat::from_str(&format_str);
        
        let alpha_enabled = annotation.get_param_as_bool("alpha", false);

        Self::new(config, format, alpha_enabled)
    }

    fn ui_type() -> &'static str {
        "color_picker"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_decorator_line;

    #[test]
    fn test_color_picker_creation() {
        let annotation = parse_decorator_line(r##"-- @ui: color_picker(format="hex", alpha=false) type=color"##).unwrap();
        let current_value = WidgetValue::String("#ff0000".to_string());

        let color_picker = ColorPickerWidget::from_annotation(
            "test-color".to_string(),
            "config.color".to_string(),
            &annotation,
            current_value,
        ).unwrap();

        assert_eq!(color_picker.format, ColorFormat::Hex);
        assert!(!color_picker.alpha_enabled);
        assert_eq!(color_picker.color_string(), "#ff0000");
    }

    #[test]
    fn test_hex_color_parsing() {
        let color = ColorPickerWidget::parse_hex_color("#ff0000").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
        assert_eq!(color.a, 1.0);

        let color_with_alpha = ColorPickerWidget::parse_hex_color("#ff000080").unwrap();
        assert_eq!(color_with_alpha.r, 255);
        assert_eq!(color_with_alpha.g, 0);
        assert_eq!(color_with_alpha.b, 0);
        assert!((color_with_alpha.a - 0.5).abs() < 0.01); // Approximately 128/255

        assert!(ColorPickerWidget::parse_hex_color("invalid").is_none());
        assert!(ColorPickerWidget::parse_hex_color("#gggggg").is_none());
    }

    #[test]
    fn test_color_validation() {
        let annotation = parse_decorator_line(r##"-- @ui: color_picker(format="hex", alpha=false) type=color"##).unwrap();
        let color_picker = ColorPickerWidget::from_annotation(
            "test-color".to_string(),
            "config.color".to_string(),
            &annotation,
            WidgetValue::String("#000000".to_string()),
        ).unwrap();

        // Valid hex colors
        assert!(color_picker.validate_value(&WidgetValue::String("#ff0000".to_string())).is_ok());
        assert!(color_picker.validate_value(&WidgetValue::String("#123ABC".to_string())).is_ok());

        // Invalid hex colors
        assert!(color_picker.validate_value(&WidgetValue::String("ff0000".to_string())).is_err()); // Missing #
        assert!(color_picker.validate_value(&WidgetValue::String("#ff00".to_string())).is_err()); // Too short
        assert!(color_picker.validate_value(&WidgetValue::String("#gggggg".to_string())).is_err()); // Invalid hex
        assert!(color_picker.validate_value(&WidgetValue::String("#ff000080".to_string())).is_err()); // Alpha not enabled

        // Non-string values
        assert!(color_picker.validate_value(&WidgetValue::Number(255.0)).is_err());
    }

    #[test]
    fn test_color_with_alpha() {
        let annotation = parse_decorator_line(r##"-- @ui: color_picker(format="hex", alpha=true) type=color"##).unwrap();
        let color_picker = ColorPickerWidget::from_annotation(
            "test-color".to_string(),
            "config.color".to_string(),
            &annotation,
            WidgetValue::String("#ff000080".to_string()),
        ).unwrap();

        assert!(color_picker.alpha_enabled);
        assert!(color_picker.validate_value(&WidgetValue::String("#ff000080".to_string())).is_ok());
    }

    #[test]
    fn test_color_picker_render_data() {
        let annotation = parse_decorator_line(r##"-- @ui: color_picker(format="hex", alpha=true) type=color"##).unwrap();
        let color_picker = ColorPickerWidget::from_annotation(
            "test-color".to_string(),
            "config.color".to_string(),
            &annotation,
            WidgetValue::String("#ff000080".to_string()),
        ).unwrap();

        let render_data = color_picker.render_data();
        assert!(render_data.contains_key("color_picker"));

        let picker_data: ColorPickerRenderData = serde_json::from_value(
            render_data.get("color_picker").unwrap().clone()
        ).unwrap();

        assert_eq!(picker_data.format, ColorFormat::Hex);
        assert_eq!(picker_data.current_value, "#ff000080");
        assert!(picker_data.alpha_enabled);
        assert!(picker_data.parsed_color.is_some());
    }
}