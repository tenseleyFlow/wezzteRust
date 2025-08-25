//! Configuration management for wezztershier
//!
//! Handles user preferences and settings storage.

use anyhow::{Context, Result};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// User configuration for wezztershier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WezztershierConfig {
    /// Default GUI backend preference
    pub default_gui_backend: GuiBackend,
}

/// Available GUI backends
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GuiBackend {
    /// Native egui-based GUI
    Native,
    /// Web-based GUI
    Web,
}

impl Default for WezztershierConfig {
    fn default() -> Self {
        Self {
            // Default to native GUI as it's more performant
            default_gui_backend: GuiBackend::Native,
        }
    }
}

impl GuiBackend {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "native" => Ok(GuiBackend::Native),
            "web" => Ok(GuiBackend::Web),
            _ => anyhow::bail!("Invalid GUI backend: {}. Valid options: native, web", s),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            GuiBackend::Native => "native",
            GuiBackend::Web => "web",
        }
    }
}

/// Configuration manager
#[derive(Debug)]
pub struct ConfigManager {
    config_path: PathBuf,
    config: WezztershierConfig,
}

impl ConfigManager {
    /// Load configuration from the standard location
    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;
        
        let config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .context("Failed to read config file")?;
            toml::from_str(&content)
                .context("Failed to parse config file")?
        } else {
            WezztershierConfig::default()
        };

        Ok(Self { config_path, config })
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        // Ensure config directory exists
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }

        let content = toml::to_string_pretty(&self.config)
            .context("Failed to serialize config")?;
        
        fs::write(&self.config_path, content)
            .context("Failed to write config file")?;

        Ok(())
    }

    /// Get current configuration
    pub fn config(&self) -> &WezztershierConfig {
        &self.config
    }

    /// Get mutable configuration
    pub fn config_mut(&mut self) -> &mut WezztershierConfig {
        &mut self.config
    }

    /// Reset to default configuration
    pub fn reset(&mut self) -> Result<()> {
        self.config = WezztershierConfig::default();
        self.save()
    }

    /// Get the path to the config file
    fn config_file_path() -> Result<PathBuf> {
        let config_dir = config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
        
        Ok(config_dir.join("wezztershier").join("config.toml"))
    }

    /// Get the config file path for this manager
    pub fn path(&self) -> &PathBuf {
        &self.config_path
    }

    /// Create a new config manager with the given path and config
    pub fn new(config_path: PathBuf, config: WezztershierConfig) -> Self {
        Self { config_path, config }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gui_backend_parsing() {
        assert_eq!(GuiBackend::from_str("native").unwrap(), GuiBackend::Native);
        assert_eq!(GuiBackend::from_str("web").unwrap(), GuiBackend::Web);
        assert_eq!(GuiBackend::from_str("NATIVE").unwrap(), GuiBackend::Native);
        assert!(GuiBackend::from_str("invalid").is_err());
    }

    #[test]
    fn test_gui_backend_as_str() {
        assert_eq!(GuiBackend::Native.as_str(), "native");
        assert_eq!(GuiBackend::Web.as_str(), "web");
    }

    #[test]
    fn test_default_config() {
        let config = WezztershierConfig::default();
        assert_eq!(config.default_gui_backend, GuiBackend::Native);
    }
}