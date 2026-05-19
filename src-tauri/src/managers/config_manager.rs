use std::fs;
use std::path::PathBuf;

use crate::models::config::{AppConfig, DeploymentMode};

/// Manages application configuration persistence.
///
/// Configuration is stored as JSON at `{config_dir}/settings.json`.
/// If the file does not exist, a default configuration is used.
pub struct ConfigManager {
    config_dir: PathBuf,
}

impl ConfigManager {
    /// # Errors
    ///
    /// Returns an error if the config file exists but cannot be read or parsed.
    pub fn new(config_dir: PathBuf) -> std::io::Result<Self> {
        fs::create_dir_all(&config_dir)?;
        Ok(Self { config_dir })
    }

    /// Load the current configuration, falling back to defaults if absent.
    ///
    /// # Errors
    ///
    /// Returns an error if the config file exists but cannot be read or parsed.
    pub fn load(&self, install_root: PathBuf) -> std::io::Result<AppConfig> {
        let path = self.config_dir.join("settings.json");
        if !path.exists() {
            let config = AppConfig::default_for(install_root);
            self.save(&config)?;
            return Ok(config);
        }
        let data = fs::read_to_string(path)?;
        let config: AppConfig = serde_json::from_str(&data)?;
        Ok(config)
    }

    /// Save the configuration to disk.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or file write fails.
    pub fn save(&self, config: &AppConfig) -> std::io::Result<()> {
        let path = self.config_dir.join("settings.json");
        let json = serde_json::to_string_pretty(config)?;
        fs::write(path, json)
    }

    /// Switch the deployment mode and persist the updated configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the current config cannot be loaded or the update cannot be saved.
    pub fn set_deployment_mode(
        &self,
        install_root: PathBuf,
        mode: DeploymentMode,
    ) -> std::io::Result<AppConfig> {
        let mut config = self.load(install_root)?;
        config.deployment_mode = mode;
        self.save(&config)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn default_config_created_when_absent() {
        let dir = TempDir::new().expect("temp dir");
        let config_dir = dir.path().join("config");
        let install_root = dir.path().join("app");
        let mgr = ConfigManager::new(config_dir.clone()).expect("create");

        let config = mgr.load(install_root.clone()).expect("load");
        assert_eq!(config.deployment_mode, DeploymentMode::WindowsOnly);
        assert_eq!(config.install_root, install_root);

        // Verify file was created.
        assert!(config_dir.join("settings.json").exists());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = TempDir::new().expect("temp dir");
        let config_dir = dir.path().join("config");
        let install_root = dir.path().join("app");
        let mgr = ConfigManager::new(config_dir).expect("create");

        let mut config = AppConfig::default_for(install_root);
        config.deployment_mode = DeploymentMode::Coordinated;
        config.mihomo.mixed_port = 9090;
        mgr.save(&config).expect("save");

        let loaded = mgr.load(dir.path().join("app")).expect("load");
        assert_eq!(loaded.deployment_mode, DeploymentMode::Coordinated);
        assert_eq!(loaded.mihomo.mixed_port, 9090);
    }

    #[test]
    fn set_deployment_mode_persists() {
        let dir = TempDir::new().expect("temp dir");
        let config_dir = dir.path().join("config");
        let install_root = dir.path().join("app");
        let mgr = ConfigManager::new(config_dir).expect("create");

        // Trigger default creation.
        let _ = mgr.load(install_root.clone()).expect("load");

        let updated = mgr
            .set_deployment_mode(install_root.clone(), DeploymentMode::WslOnly)
            .expect("set mode");

        assert_eq!(updated.deployment_mode, DeploymentMode::WslOnly);
        assert_eq!(updated.install_root, install_root);

        // Reload from disk to verify persistence.
        let reloaded = mgr.load(install_root).expect("reload");
        assert_eq!(reloaded.deployment_mode, DeploymentMode::WslOnly);
    }
}
