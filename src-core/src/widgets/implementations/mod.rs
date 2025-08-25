//! Core widget implementations
//!
//! Contains the built-in widget types that handle common UI patterns.

pub mod slider;
pub mod select;
pub mod color_picker;
pub mod theme_selector;
pub mod numerical;
pub mod text;

pub use slider::SliderWidget;
pub use select::SelectWidget;
pub use color_picker::ColorPickerWidget;
pub use theme_selector::ThemeSelectorWidget;
pub use numerical::NumericalWidget;
pub use text::TextWidget;

// Re-export the implementations sub-module
pub mod implementations {
    pub use super::*;
}

/// Register all core widgets with the global registry
pub fn register_core_widgets() -> crate::error::Result<()> {
    use crate::widgets::{registry::global_registry, metadata::CoreWidgets};

    let registry = global_registry();

    // Register core widget implementations
    registry.register::<SliderWidget>(CoreWidgets::slider())?;
    registry.register::<SelectWidget>(CoreWidgets::select())?;
    registry.register::<ColorPickerWidget>(CoreWidgets::color_picker())?;
    registry.register::<ThemeSelectorWidget>(CoreWidgets::theme_selector())?;

    tracing::info!("Registered {} core widgets", registry.registered_types()?.len());
    Ok(())
}