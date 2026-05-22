//! Deployment mode manager for `GoGuo`.
//!
//! Manages deployment mode detection, persistence, and adapter creation
//! across Windows, Linux, and WSL environments.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use crate::adapters::command_executor::{PowershellBridgeExecutor, WslBridgeExecutor};
use crate::adapters::PlatformAdapter;
use crate::adapters::windows_remote::WindowsRemoteAdapter;
#[allow(unused_imports)]
use crate::adapters::wsl_remote::WslRemoteAdapter;
use crate::managers::config_manager::ConfigManager;
use crate::models::config::{AppConfig, DeploymentMode};

#[cfg(target_os = "linux")]
use crate::services::wsl_detector::{DistroInfo, SystemFileReader, WslDetector, WslNetworkMode};

// ---------------------------------------------------------------------------
// WslStatus
// ---------------------------------------------------------------------------

/// WSL status information for UI display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WslStatus {
    pub is_wsl: bool,
    pub distro: Option<DistroInfo>,
    pub network_mode: WslNetworkMode,
    pub reachable: bool,
}

// ---------------------------------------------------------------------------
// DeploymentManager
// ---------------------------------------------------------------------------

/// Manages deployment mode detection, persistence, and adapter creation.
pub struct DeploymentManager {
    config_manager: ConfigManager,
    install_root: PathBuf,
}

impl DeploymentManager {
    /// Create a new `DeploymentManager`.
    #[must_use]
    pub const fn new(config_manager: ConfigManager, install_root: PathBuf) -> Self {
        Self {
            config_manager,
            install_root,
        }
    }

    /// Detect the appropriate deployment mode for the current platform.
    #[must_use]
    pub fn detect_deployment_mode() -> DeploymentMode {
        #[cfg(target_os = "linux")]
        {
            let detector = WslDetector::new(SystemFileReader);
            if detector.is_running_in_wsl() {
                DeploymentMode::WslOnly
            } else {
                DeploymentMode::LinuxOnly
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            DeploymentMode::WindowsOnly
        }
    }

    /// Load the current deployment mode from persisted configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration file cannot be read or parsed.
    pub fn get_deployment_mode(&self) -> Result<DeploymentMode, String> {
        self.config_manager
            .load(self.install_root.clone())
            .map(|config| config.deployment_mode)
            .map_err(|e| e.to_string())
    }

    /// Set and persist a new deployment mode.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration cannot be loaded or saved.
    pub fn set_deployment_mode(&self, mode: DeploymentMode) -> Result<AppConfig, String> {
        self.config_manager
            .set_deployment_mode(self.install_root.clone(), mode)
            .map_err(|e| e.to_string())
    }

    /// Create platform adapters appropriate for the given deployment mode.
    ///
    /// Coordinated mode creates **two** adapters (local + remote bridge):
    /// - On Windows host: `WindowsAdapter` + `WslRemoteAdapter` (via `wsl -e`)
    /// - On WSL host: `WslAdapter` + `WindowsRemoteAdapter` (via `powershell.exe`)
    #[must_use]
    pub fn create_adapters(&self, mode: &DeploymentMode) -> Vec<Box<dyn PlatformAdapter + Send + Sync>> {
        let mut adapters: Vec<Box<dyn PlatformAdapter + Send + Sync>> = Vec::new();

        match mode {
            DeploymentMode::WindowsOnly => {
                #[cfg(target_os = "windows")]
                {
                    adapters.push(Box::new(
                        crate::adapters::windows::WindowsAdapter::new(),
                    ));
                }
                #[cfg(target_os = "linux")]
                {
                    adapters.push(Box::new(
                        WindowsRemoteAdapter::new(PowershellBridgeExecutor::new()),
                    ));
                }
            }
            DeploymentMode::WslOnly => {
                #[cfg(target_os = "windows")]
                {
                    adapters.push(Box::new(
                        WslRemoteAdapter::new(WslBridgeExecutor::new()),
                    ));
                }
                #[cfg(target_os = "linux")]
                {
                    use crate::adapters::linux_base::SystemShellExecutor;
                    adapters.push(Box::new(
                        crate::adapters::wsl::WslAdapter::<SystemShellExecutor>::new(
                            SystemShellExecutor,
                        ),
                    ));
                }
            }
            DeploymentMode::LinuxOnly => {
                #[cfg(target_os = "linux")]
                {
                    use crate::adapters::linux_base::SystemShellExecutor;
                    adapters.push(Box::new(
                        crate::adapters::linux::LinuxAdapter::<SystemShellExecutor>::new(
                            SystemShellExecutor,
                        ),
                    ));
                }
            }
            DeploymentMode::Coordinated => {
                #[cfg(target_os = "windows")]
                {
                    // Windows host: local WindowsAdapter + remote WslRemoteAdapter
                    adapters.push(Box::new(
                        crate::adapters::windows::WindowsAdapter::new(),
                    ));
                    adapters.push(Box::new(
                        WslRemoteAdapter::new(WslBridgeExecutor::new()),
                    ));
                }
                #[cfg(target_os = "linux")]
                {
                    // WSL host: local WslAdapter + remote WindowsRemoteAdapter
                    let detector = WslDetector::new(SystemFileReader);
                    if detector.is_running_in_wsl() {
                        use crate::adapters::linux_base::SystemShellExecutor;
                        adapters.push(Box::new(
                            crate::adapters::wsl::WslAdapter::<SystemShellExecutor>::new(
                                SystemShellExecutor,
                            ),
                        ));
                        adapters.push(Box::new(
                            WindowsRemoteAdapter::new(PowershellBridgeExecutor::new()),
                        ));
                    }
                    // Pure Linux: Coordinated mode is not applicable
                }
            }
        }

        adapters
    }

    /// Get the current WSL status (Linux only).
    #[cfg(target_os = "linux")]
    #[must_use]
    pub fn get_wsl_status(&self) -> WslStatus {
        let detector = WslDetector::new(SystemFileReader);
        let is_wsl = detector.is_running_in_wsl();
        let distro = detector.get_distro_info();
        let network_mode = detector.detect_network_mode();
        let reachable = true; // Placeholder; actual reachability is checked by adapters

        WslStatus {
            is_wsl,
            distro,
            network_mode,
            reachable,
        }
    }

    /// Get the WSL network mode (Linux only).
    #[cfg(target_os = "linux")]
    #[must_use]
    pub fn get_network_mode(&self) -> WslNetworkMode {
        let detector = WslDetector::new(SystemFileReader);
        detector.detect_network_mode()
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // -----------------------------------------------------------------------
    // new() creates manager
    // -----------------------------------------------------------------------

    #[test]
    fn new_creates_manager_with_fields() {
        let dir = TempDir::new().expect("temp dir");
        let config_dir = dir.path().join("config");
        let install_root = dir.path().join("app");
        let cm = ConfigManager::new(config_dir).expect("create config manager");
        let mgr = DeploymentManager::new(cm, install_root.clone());
        assert_eq!(mgr.install_root, install_root);
    }

    // -----------------------------------------------------------------------
    // detect_deployment_mode returns correct mode for current platform
    // -----------------------------------------------------------------------

    #[test]
    fn detect_deployment_mode_returns_valid_mode() {
        let mode = DeploymentManager::detect_deployment_mode();
        // On Linux (including WSL), it should be either WslOnly or LinuxOnly
        // On Windows, it should be WindowsOnly
        #[cfg(target_os = "linux")]
        {
            assert!(
                matches!(mode, DeploymentMode::WslOnly | DeploymentMode::LinuxOnly),
                "Expected WslOnly or LinuxOnly on Linux, got {mode:?}"
            );
        }
        #[cfg(target_os = "windows")]
        {
            assert_eq!(mode, DeploymentMode::WindowsOnly);
        }
    }

    // -----------------------------------------------------------------------
    // get_deployment_mode loads from config
    // -----------------------------------------------------------------------

    #[test]
    fn get_deployment_mode_loads_from_config() {
        let dir = TempDir::new().expect("temp dir");
        let config_dir = dir.path().join("config");
        let install_root = dir.path().join("app");
        let cm = ConfigManager::new(config_dir).expect("create config manager");
        let mgr = DeploymentManager::new(cm, install_root);

        let mode = mgr.get_deployment_mode().expect("get mode");
        // Default is WindowsOnly
        assert_eq!(mode, DeploymentMode::WindowsOnly);
    }

    // -----------------------------------------------------------------------
    // set_deployment_mode persists
    // -----------------------------------------------------------------------

    #[test]
    fn set_deployment_mode_persists() {
        let dir = TempDir::new().expect("temp dir");
        let config_dir = dir.path().join("config");
        let install_root = dir.path().join("app");
        let cm = ConfigManager::new(config_dir).expect("create config manager");
        let mgr = DeploymentManager::new(cm, install_root.clone());

        let result = mgr
            .set_deployment_mode(DeploymentMode::LinuxOnly)
            .expect("set mode");
        assert_eq!(result.deployment_mode, DeploymentMode::LinuxOnly);

        // Reload to verify persistence
        let cm2 = ConfigManager::new(dir.path().join("config")).expect("create config manager 2");
        let mgr2 = DeploymentManager::new(cm2, install_root);
        let reloaded = mgr2.get_deployment_mode().expect("reload mode");
        assert_eq!(reloaded, DeploymentMode::LinuxOnly);
    }

    // -----------------------------------------------------------------------
    // create_adapters returns correct count per mode
    // -----------------------------------------------------------------------

    #[cfg(target_os = "linux")]
    #[test]
    fn create_adapters_wsl_only_returns_one() {
        let dir = TempDir::new().expect("temp dir");
        let config_dir = dir.path().join("config");
        let install_root = dir.path().join("app");
        let cm = ConfigManager::new(config_dir).expect("create config manager");
        let mgr = DeploymentManager::new(cm, install_root);
        let adapters = mgr.create_adapters(&DeploymentMode::WslOnly);
        assert_eq!(adapters.len(), 1);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn create_adapters_linux_only_returns_one() {
        let dir = TempDir::new().expect("temp dir");
        let config_dir = dir.path().join("config");
        let install_root = dir.path().join("app");
        let cm = ConfigManager::new(config_dir).expect("create config manager");
        let mgr = DeploymentManager::new(cm, install_root);
        let adapters = mgr.create_adapters(&DeploymentMode::LinuxOnly);
        assert_eq!(adapters.len(), 1);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn create_adapters_coordinated_returns_two_in_wsl() {
        let dir = TempDir::new().expect("temp dir");
        let config_dir = dir.path().join("config");
        let install_root = dir.path().join("app");
        let cm = ConfigManager::new(config_dir).expect("create config manager");
        let mgr = DeploymentManager::new(cm, install_root);
        let adapters = mgr.create_adapters(&DeploymentMode::Coordinated);
        // In WSL, Coordinated creates WslAdapter + WindowsRemoteAdapter = 2
        // In pure Linux, Coordinated is not applicable = 0
        let detector = WslDetector::new(SystemFileReader);
        if detector.is_running_in_wsl() {
            assert_eq!(adapters.len(), 2, "Coordinated mode should return 2 adapters in WSL");
            let platforms: Vec<_> = adapters.iter().map(|a| a.platform()).collect();
            assert!(platforms.contains(&crate::models::baseline::Platform::Wsl));
            assert!(platforms.contains(&crate::models::baseline::Platform::Windows));
        } else {
            assert_eq!(adapters.len(), 0, "Coordinated mode not applicable on pure Linux");
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn create_adapters_windows_only_returns_remote_on_linux() {
        let dir = TempDir::new().expect("temp dir");
        let config_dir = dir.path().join("config");
        let install_root = dir.path().join("app");
        let cm = ConfigManager::new(config_dir).expect("create config manager");
        let mgr = DeploymentManager::new(cm, install_root);
        let adapters = mgr.create_adapters(&DeploymentMode::WindowsOnly);
        // On Linux, WindowsOnly creates WindowsRemoteAdapter (via powershell.exe)
        assert_eq!(adapters.len(), 1);
        assert_eq!(adapters[0].platform(), crate::models::baseline::Platform::Windows);
    }

    // -----------------------------------------------------------------------
    // WslStatus serde roundtrip
    // -----------------------------------------------------------------------

    #[cfg(target_os = "linux")]
    #[test]
    fn wsl_status_serde_roundtrip() {
        let status = WslStatus {
            is_wsl: true,
            distro: Some(DistroInfo {
                name: "Ubuntu".to_string(),
                version: "22.04".to_string(),
                id: "ubuntu".to_string(),
                is_default: true,
            }),
            network_mode: WslNetworkMode::Nat,
            reachable: true,
        };
        let json = serde_json::to_string(&status).expect("serialize");
        let back: WslStatus = serde_json::from_str(&json).expect("deserialize");
        assert!(back.is_wsl);
        assert!(back.distro.is_some());
        assert_eq!(back.network_mode, WslNetworkMode::Nat);
        assert!(back.reachable);
    }

    // -----------------------------------------------------------------------
    // WslNetworkMode serde roundtrip (imported via wsl_detector)
    // -----------------------------------------------------------------------

    #[cfg(target_os = "linux")]
    #[test]
    fn wsl_network_mode_serde_roundtrip() {
        let modes = vec![
            WslNetworkMode::Nat,
            WslNetworkMode::Mirrored,
            WslNetworkMode::NotInstalled,
        ];
        for m in &modes {
            let json = serde_json::to_string(m).expect("serialize");
            let back: WslNetworkMode = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(&back, m);
        }
    }

    // -----------------------------------------------------------------------
    // get_wsl_status returns valid struct on Linux
    // -----------------------------------------------------------------------

    #[cfg(target_os = "linux")]
    #[test]
    fn get_wsl_status_returns_valid_struct() {
        let dir = TempDir::new().expect("temp dir");
        let config_dir = dir.path().join("config");
        let install_root = dir.path().join("app");
        let cm = ConfigManager::new(config_dir).expect("create config manager");
        let mgr = DeploymentManager::new(cm, install_root);
        let status = mgr.get_wsl_status();
        // reachable is always true (placeholder)
        assert!(status.reachable);
        // network_mode should be a valid variant
        match status.network_mode {
            WslNetworkMode::Nat
            | WslNetworkMode::Mirrored
            | WslNetworkMode::NotInstalled => {} // ok
        }
    }
}
