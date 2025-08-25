//! Configuration file management for wezzte
//!
//! Handles reading, writing, and backing up WezTerm configuration files.

use crate::{error::Result, WezzteError};
use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

/// Configuration file manager
pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    /// Create a new config manager with the given path
    pub fn new(config_path: impl Into<PathBuf>) -> Self {
        Self {
            config_path: config_path.into(),
        }
    }

    /// Create a config manager with the default WezTerm config path
    pub fn with_default_path() -> Result<Self> {
        let config_path = Self::get_default_config_path()?;
        Ok(Self::new(config_path))
    }

    /// Get the default WezTerm configuration path
    pub fn get_default_config_path() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| {
            WezzteError::config("Could not determine home directory")
        })?;

        // Check XDG_CONFIG_HOME first, then fall back to ~/.config
        let config_dir = match std::env::var("XDG_CONFIG_HOME") {
            Ok(xdg_config) => PathBuf::from(xdg_config),
            Err(_) => home.join(".config"),
        };

        Ok(config_dir.join("wezterm").join("wezterm.lua"))
    }

    /// Read the configuration file content
    pub async fn read_config(&self) -> Result<String> {
        if !self.config_path.exists() {
            return Err(WezzteError::FileNotFound {
                path: self.config_path.display().to_string(),
            });
        }

        let content = fs::read_to_string(&self.config_path).await.map_err(|e| {
            WezzteError::io(format!("Failed to read config file: {}", e))
        })?;

        Ok(content)
    }

    /// Write content to the configuration file
    pub async fn write_config(&self, content: &str) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                WezzteError::io(format!("Failed to create config directory: {}", e))
            })?;
        }

        // Write atomically by writing to temp file first
        let temp_path = self.config_path.with_extension("tmp");
        
        {
            let mut file = File::create(&temp_path).await.map_err(|e| {
                WezzteError::io(format!("Failed to create temp file: {}", e))
            })?;

            file.write_all(content.as_bytes()).await.map_err(|e| {
                WezzteError::io(format!("Failed to write to temp file: {}", e))
            })?;

            file.sync_all().await.map_err(|e| {
                WezzteError::io(format!("Failed to sync temp file: {}", e))
            })?;
        }

        // Atomically replace the original file
        fs::rename(&temp_path, &self.config_path).await.map_err(|e| {
            WezzteError::io(format!("Failed to replace config file: {}", e))
        })?;

        Ok(())
    }

    /// Get the path to the config file
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// Check if the config file exists
    pub fn exists(&self) -> bool {
        self.config_path.exists()
    }
}

/// Backup manager for configuration files
pub struct BackupManager {
    backup_dir: PathBuf,
    config_path: PathBuf,
    max_backups: usize,
}

impl BackupManager {
    /// Create a new backup manager
    pub fn new(
        config_path: impl Into<PathBuf>,
        backup_dir: Option<PathBuf>,
        max_backups: usize,
    ) -> Result<Self> {
        let config_path = config_path.into();
        let backup_dir = backup_dir.unwrap_or_else(|| Self::get_default_backup_dir());

        Ok(Self {
            config_path,
            backup_dir,
            max_backups,
        })
    }

    /// Get the default backup directory
    pub fn get_default_backup_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".local")
            .join("share")
            .join("wezzte")
            .join("backups")
    }

    /// Create a backup of the current configuration
    pub async fn create_backup(&self, suffix: &str) -> Result<PathBuf> {
        if !self.config_path.exists() {
            return Err(WezzteError::FileNotFound {
                path: self.config_path.display().to_string(),
            });
        }

        // Ensure backup directory exists
        fs::create_dir_all(&self.backup_dir).await.map_err(|e| {
            WezzteError::io(format!("Failed to create backup directory: {}", e))
        })?;

        // Generate backup filename with timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let backup_filename = format!("wezterm-{}-{}.lua", timestamp, suffix);
        let backup_path = self.backup_dir.join(backup_filename);

        // Copy the file
        fs::copy(&self.config_path, &backup_path).await.map_err(|e| {
            WezzteError::io(format!("Failed to create backup: {}", e))
        })?;

        // Clean up old backups
        self.cleanup_old_backups().await?;

        Ok(backup_path)
    }

    /// Create a temporary backup (for live preview)
    pub async fn create_temp_backup(&self) -> Result<PathBuf> {
        self.create_backup("temp").await
    }

    /// Create a persistent backup (for applied changes)
    pub async fn create_persistent_backup(&self) -> Result<PathBuf> {
        self.create_backup("persistent").await
    }

    /// Restore from the most recent backup
    pub async fn restore_from_backup(&self) -> Result<()> {
        let backups = self.list_backups().await?;
        let latest_backup = backups.first().ok_or_else(|| {
            WezzteError::config("No backups found to restore from")
        })?;

        fs::copy(latest_backup, &self.config_path).await.map_err(|e| {
            WezzteError::io(format!("Failed to restore from backup: {}", e))
        })?;

        Ok(())
    }

    /// List all backups, sorted by modification time (newest first)
    pub async fn list_backups(&self) -> Result<Vec<PathBuf>> {
        if !self.backup_dir.exists() {
            return Ok(Vec::new());
        }

        let mut entries = fs::read_dir(&self.backup_dir).await.map_err(|e| {
            WezzteError::io(format!("Failed to read backup directory: {}", e))
        })?;

        let mut backups = Vec::new();

        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.is_file()
                && path.extension().map_or(false, |ext| ext == "lua")
                && path
                    .file_name()
                    .map_or(false, |name| name.to_string_lossy().starts_with("wezterm-"))
            {
                backups.push(path);
            }
        }

        // Sort by modification time, newest first
        backups.sort_by(|a, b| {
            let a_modified = a.metadata().and_then(|m| m.modified()).unwrap_or(UNIX_EPOCH);
            let b_modified = b.metadata().and_then(|m| m.modified()).unwrap_or(UNIX_EPOCH);
            b_modified.cmp(&a_modified)
        });

        Ok(backups)
    }

    /// Clean up old backups, keeping only the most recent ones
    async fn cleanup_old_backups(&self) -> Result<()> {
        let backups = self.list_backups().await?;

        if backups.len() <= self.max_backups {
            return Ok(());
        }

        // Remove excess backups
        for backup_path in &backups[self.max_backups..] {
            if let Err(e) = fs::remove_file(backup_path).await {
                tracing::warn!("Failed to remove old backup {:?}: {}", backup_path, e);
            }
        }

        Ok(())
    }

    /// Get backup directory path
    pub fn backup_dir(&self) -> &Path {
        &self.backup_dir
    }
}

/// Configuration update manager - handles updating tuner blocks
pub struct ConfigUpdater {
    original_content: String,
}

impl ConfigUpdater {
    /// Create a new config updater with the original content
    pub fn new(content: String) -> Self {
        Self {
            original_content: content,
        }
    }

    /// Update the tuner block with new content
    pub fn update_tuner_block(&self, new_tuner_block: &str) -> Result<String> {
        let start_marker = crate::markers::TUNER_START;
        let end_marker = crate::markers::TUNER_END;

        let start_pos = self.original_content.find(start_marker).ok_or_else(|| {
            WezzteError::config("No tuner start marker found in configuration")
        })?;

        let end_pos = self.original_content.find(end_marker).ok_or_else(|| {
            WezzteError::config("No tuner end marker found in configuration")
        })?;

        if end_pos <= start_pos {
            return Err(WezzteError::config("Invalid tuner marker positions"));
        }

        // Replace the content between markers
        let before = &self.original_content[..start_pos + start_marker.len()];
        let after = &self.original_content[end_pos..];

        let updated_content = format!("{}\n{}\n{}", before, new_tuner_block.trim(), after);
        Ok(updated_content)
    }

    /// Get the original content
    pub fn original_content(&self) -> &str {
        &self.original_content
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_config_manager_read_write() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("wezterm.lua");

        let manager = ConfigManager::new(&config_path);

        // Test writing and reading
        let content = "config.font_size = 14";
        manager.write_config(content).await.unwrap();
        let read_content = manager.read_config().await.unwrap();

        assert_eq!(content, read_content);
    }

    #[tokio::test]
    async fn test_backup_manager() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("wezterm.lua");
        let backup_dir = temp_dir.path().join("backups");

        // Create a test config file
        fs::write(&config_path, "config.font_size = 14").await.unwrap();

        let backup_manager = BackupManager::new(&config_path, Some(backup_dir), 5).unwrap();

        // Create a backup
        let backup_path = backup_manager.create_backup("test").await.unwrap();
        assert!(backup_path.exists());

        // List backups
        let backups = backup_manager.list_backups().await.unwrap();
        assert_eq!(backups.len(), 1);
    }

    #[test]
    fn test_config_updater() {
        let original = r#"local config = {}

-- <<TUNER-START>>
-- @ui: slider(min=10, max=42) type=int
config.font_size = 18
-- <<TUNER-END>>

return config"#;

        let updater = ConfigUpdater::new(original.to_string());
        let new_tuner = r#"-- @ui: slider(min=8, max=72) type=int
config.font_size = 24"#;

        let updated = updater.update_tuner_block(new_tuner).unwrap();

        assert!(updated.contains("config.font_size = 24"));
        assert!(updated.contains("min=8, max=72"));
        assert!(updated.contains("local config = {}"));
        assert!(updated.contains("return config"));
    }
}