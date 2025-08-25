use wezzte_core::{parser::parse_decorator_line, ast::UiType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test basic functionality
    let decorator = "-- @ui: slider(min=10, max=42, step=1) type=int";
    let annotation = parse_decorator_line(decorator)?;
    
    println!("🎉 WezzteRust Phase 1 Complete!");
    println!("📊 Parsed decorator: {} -> {:?}", decorator, annotation.ui_type);
    println!("🔧 Parameters: min={}, max={}, step={}, type={}", 
        annotation.get_param_as_number("min", 0.0),
        annotation.get_param_as_number("max", 0.0), 
        annotation.get_param_as_number("step", 0.0),
        annotation.get_param_as_string("type", "")
    );
    
    // Test another type
    let color_decorator = r#"-- @ui: color_picker(format="hex", alpha=false) type=color"#;
    let color_annotation = parse_decorator_line(color_decorator)?;
    println!("🎨 Color picker: format={}, alpha={}", 
        color_annotation.get_param_as_string("format", ""),
        color_annotation.get_param_as_bool("alpha", true)
    );

    println!("\n✅ Parser foundation is working correctly!");
    println!("🚀 Ready for Phase 2: Core System & Widget Factory");
    
    Ok(())
}
