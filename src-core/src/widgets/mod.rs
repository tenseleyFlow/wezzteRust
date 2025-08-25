//! Widget system for wezzte-core
//! 
//! Provides the foundation for UI widgets that can be generated from decorator annotations.
//! This is designed to be frontend-agnostic and work with any UI framework.

pub mod factory;
pub mod traits;
pub mod registry;
pub mod implementations;
pub mod metadata;

pub use factory::{WidgetFactory, WidgetBuilder};
pub use traits::{Widget, WidgetConfig, WidgetValue, WidgetEvent};
pub use registry::WidgetRegistry;
pub use metadata::{WidgetMetadata, WidgetConstraint, WidgetProperty};
pub use implementations::*;