//! Integration tests with real wezterm configuration examples
//!
//! Tests the complete pipeline using actual decorator patterns from the Python implementation.

use crate::{
    parser::{parse_annotations, parse_decorator_line, ConfigEntry},
    ast::{UiType, ParamValue},
};

/// Test data from the original Python implementation README
const SAMPLE_CONFIG: &str = r##"
local wezterm = require 'wezterm'
local config = {}

-- <<TUNER-START>>
-- @ui: slider(min=10, max=42, step=1) type=int
config.font_size = 18
-- @ui: select(options="JetBrains Mono, Fira Code, Cascadia Code, Source Code Pro") type=string
config.font = wezterm.font("JetBrains Mono")
-- @ui: numerical(min=0.5, max=5.5, step=0.01) type=float
config.line_height = 1.29
-- @ui: numerical(min=0.5, max=2.0, step=0.1) type=float
config.cell_width = 1.0
-- @ui: slider(min=0.05, max=1.0, step=0.01) type=float
config.window_background_opacity = 1.0
-- @ui: slider(min=1, max=100, step=1) type=int
config.macos_window_background_blur = 100
-- @ui: color_picker(format="hex", alpha=false) type=color
config.colors = config.colors or {}
config.colors.background = "#333333"

-- @ui: color_picker(format="hex", alpha=false) type=color
config.colors = config.colors or {}
config.colors.tab_bar = config.colors.tab_bar or {}
config.colors.tab_bar.background = "#333333"

-- @ui: color_picker(format="hex", alpha=false) type=color
config.colors = config.colors or {}
config.colors.tab_bar = config.colors.tab_bar or {}
config.colors.tab_bar.active_tab = config.colors.tab_bar.active_tab or {}
config.colors.tab_bar.active_tab.bg_color = "#444444"

-- @ui: color_picker(format="hex", alpha=false) type=color
config.colors = config.colors or {}
config.colors.tab_bar = config.colors.tab_bar or {}
config.colors.tab_bar.active_tab = config.colors.tab_bar.active_tab or {}
config.colors.tab_bar.active_tab.fg_color = "#ffffff"

-- @ui: color_picker(format="hex", alpha=false) type=color
config.colors = config.colors or {}
config.colors.tab_bar = config.colors.tab_bar or {}
config.colors.tab_bar.inactive_tab = config.colors.tab_bar.inactive_tab or {}
config.colors.tab_bar.inactive_tab.bg_color = "#333333"

-- @ui: color_picker(format="hex", alpha=false) type=color
config.colors = config.colors or {}
config.colors.tab_bar = config.colors.tab_bar or {}
config.colors.tab_bar.inactive_tab = config.colors.tab_bar.inactive_tab or {}
config.colors.tab_bar.inactive_tab.fg_color = "#888888"

-- @ui: slider(min=0, max=2000, step=100) type=int
config.cursor_blink_rate = 500
-- @ui: select(options="true, false") type=string
config.hide_tab_bar_if_only_one_tab = true
-- @ui: select(options="NONE, RESIZE, TITLE, RESIZE|TITLE") type=string
config.window_decorations = "RESIZE|TITLE"
-- @ui: select(options="true, false") type=string
config.pane_focus_follows_mouse = false
-- @ui: select(options="true, false") type=string
config.native_macos_fullscreen_mode = true
-- @ui: numerical(min=100, max=10000, step=100) type=int
config.scrollback_lines = 3500
-- @ui: select(options="true, false") type=string
config.enable_scroll_bar = true
-- @ui: slider(min=1, max=60, step=1) type=int
config.animation_fps = 60
-- @ui: slider(min=1, max=120, step=1) type=int
config.max_fps = 60
-- @ui: select(options="true, false") type=string
config.tab_bar_at_bottom = false
-- @ui: numerical(min=50, max=500, step=10) type=int
config.tab_max_width = 200
-- @ui: select(options="SystemBeep, Disabled") type=string
config.audible_bell = "SystemBeep"
-- @ui: select(options="true, false") type=string
config.hide_mouse_cursor_when_typing = true
-- @ui: select(options="true, false") type=string
config.swallow_mouse_click_on_window_focus = false
-- @ui: select(options="true, false") type=string
config.debug_key_events = false
-- <<TUNER-END>>

return config
"##;

#[tokio::test]
async fn test_full_sample_config_parsing() {
    let entries = parse_annotations(SAMPLE_CONFIG).expect("Should parse sample config");
    
    // Should find all 23 entries from the sample
    assert_eq!(entries.len(), 23, "Should parse all 23 configuration entries");
    
    // Test specific entries
    let font_size_entry = entries.iter().find(|e| e.key == "config.font_size").unwrap();
    assert_eq!(font_size_entry.value, "18");
    assert_eq!(font_size_entry.annotation.ui_type, UiType::Slider);
    assert_eq!(font_size_entry.annotation.get_param_as_number("min", -1.0), 10.0);
    assert_eq!(font_size_entry.annotation.get_param_as_number("max", -1.0), 42.0);
    assert_eq!(font_size_entry.annotation.get_param_as_string("type", ""), "int");
    
    let font_entry = entries.iter().find(|e| e.key == "config.font").unwrap();
    assert_eq!(font_entry.annotation.ui_type, UiType::Select);
    assert_eq!(
        font_entry.annotation.get_param_as_string("options", ""),
        "JetBrains Mono, Fira Code, Cascadia Code, Source Code Pro"
    );
    
    // Test nested table entries
    let bg_color_entry = entries.iter()
        .find(|e| e.key == "config.colors.background")
        .unwrap();
    assert_eq!(bg_color_entry.value, r##""#333333""##);
    assert_eq!(bg_color_entry.annotation.ui_type, UiType::ColorPicker);
    assert_eq!(bg_color_entry.annotation.get_param_as_string("format", ""), "hex");
    assert_eq!(bg_color_entry.annotation.get_param_as_bool("alpha", true), false);
}

#[test]
fn test_individual_decorator_patterns() {
    // Test slider with all parameters
    let slider = parse_decorator_line("-- @ui: slider(min=10, max=42, step=1) type=int").unwrap();
    assert_eq!(slider.ui_type, UiType::Slider);
    assert_eq!(slider.get_param_as_number("min", -1.0), 10.0);
    assert_eq!(slider.get_param_as_number("max", -1.0), 42.0);
    assert_eq!(slider.get_param_as_number("step", -1.0), 1.0);
    assert_eq!(slider.get_param_as_string("type", ""), "int");
    
    // Test select with string options
    let select = parse_decorator_line(r##"-- @ui: select(options="Dark, Light, Auto") type=string"##).unwrap();
    assert_eq!(select.ui_type, UiType::Select);
    assert_eq!(select.get_param_as_string("options", ""), "Dark, Light, Auto");
    
    // Test numerical input
    let numerical = parse_decorator_line("-- @ui: numerical(min=0.5, max=5.5, step=0.01) type=float").unwrap();
    assert_eq!(numerical.ui_type, UiType::Numerical);
    assert_eq!(numerical.get_param_as_number("min", -1.0), 0.5);
    assert_eq!(numerical.get_param_as_number("step", -1.0), 0.01);
    
    // Test color picker with boolean params
    let color_picker = parse_decorator_line(r##"-- @ui: color_picker(format="hex", alpha=false) type=color"##).unwrap();
    assert_eq!(color_picker.ui_type, UiType::ColorPicker);
    assert_eq!(color_picker.get_param_as_string("format", ""), "hex");
    assert_eq!(color_picker.get_param_as_bool("alpha", true), false);
    assert_eq!(color_picker.get_param_as_string("type", ""), "color");
}

#[test]
fn test_complex_decorator_patterns() {
    // Test with list parameters (from grammar examples)
    let multi_select = parse_decorator_line(r##"-- @ui: multi_select(options=["resize", "title", "close"], defaults={resize: true}) type=flags"##).unwrap();
    assert_eq!(multi_select.ui_type, UiType::Custom("multi_select".to_string()));
    
    let options = multi_select.get_param("options").unwrap().as_list().unwrap();
    assert_eq!(options.len(), 3);
    assert_eq!(options[0].as_string().unwrap(), "resize");
    assert_eq!(options[1].as_string().unwrap(), "title");
    assert_eq!(options[2].as_string().unwrap(), "close");
    
    let defaults = multi_select.get_param("defaults").unwrap().as_object().unwrap();
    assert_eq!(defaults.get("resize").unwrap().as_bool().unwrap(), true);
    
    // Test range slider with linked parameter
    let range_slider = parse_decorator_line("-- @ui: range_slider(min=0, max=50, linked=true) type=padding").unwrap();
    assert_eq!(range_slider.ui_type, UiType::Custom("range_slider".to_string()));
    assert_eq!(range_slider.get_param_as_bool("linked", false), true);
    
    // Test font picker with constraints
    let font_picker = parse_decorator_line(r##"-- @ui: font_picker(monospace_only=true, size_range=[8, 72]) type=font"##).unwrap();
    assert_eq!(font_picker.ui_type, UiType::Custom("font_picker".to_string()));
    assert_eq!(font_picker.get_param_as_bool("monospace_only", false), true);
    
    let size_range = font_picker.get_param("size_range").unwrap().as_list().unwrap();
    assert_eq!(size_range[0].as_number().unwrap(), 8.0);
    assert_eq!(size_range[1].as_number().unwrap(), 72.0);
}

#[test]
fn test_edge_cases_and_error_handling() {
    // Test empty parameters
    let simple = parse_decorator_line("-- @ui: text type=string").unwrap();
    assert_eq!(simple.ui_type, UiType::Text);
    assert_eq!(simple.get_param_as_string("type", ""), "string");
    
    // Test trailing comma
    let trailing_comma = parse_decorator_line("-- @ui: slider(min=0, max=100, step=1,) type=int").unwrap();
    assert_eq!(trailing_comma.get_param_as_number("min", -1.0), 0.0);
    
    // Test invalid decorator prefix
    assert!(parse_decorator_line("@ui: slider(min=0, max=100)").is_err());
    
    // Test empty annotation
    assert!(parse_decorator_line("-- @ui:").is_err());
    
    // Test invalid syntax
    assert!(parse_decorator_line("-- @ui: slider(min=invalid)").is_err());
}

#[test]
fn test_widget_validation() {
    // Valid slider should pass validation
    let valid_slider = parse_decorator_line("-- @ui: slider(min=0, max=100) type=int").unwrap();
    assert!(valid_slider.validate().is_ok());
    
    // Invalid slider missing required parameters should fail
    let invalid_slider = parse_decorator_line("-- @ui: slider() type=int").unwrap();
    assert!(invalid_slider.validate().is_err());
    
    // Valid select should pass
    let valid_select = parse_decorator_line(r##"-- @ui: select(options="a, b, c") type=string"##).unwrap();
    assert!(valid_select.validate().is_ok());
    
    // Invalid select missing options should fail
    let invalid_select = parse_decorator_line("-- @ui: select() type=string").unwrap();
    assert!(invalid_select.validate().is_err());
}

#[test]
fn test_type_coercion_and_defaults() {
    let annotation = parse_decorator_line("-- @ui: slider(min=10, max=100, step=1.5) custom_param=test").unwrap();
    
    // Test number coercion
    assert_eq!(annotation.get_param_as_int("min", -1), 10);
    assert_eq!(annotation.get_param_as_number("step", -1.0), 1.5);
    
    // Test defaults for missing parameters
    assert_eq!(annotation.get_param_as_string("missing", "default"), "default");
    assert_eq!(annotation.get_param_as_bool("missing", true), true);
    assert_eq!(annotation.get_param_as_number("missing", 42.0), 42.0);
    
    // Test data type detection
    assert_eq!(annotation.get_data_type(), "float"); // Default when no type specified
}