use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Deployment mode determining which platform adapters are active.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentMode {
    WindowsOnly,
    WslOnly,
    LinuxOnly,
    Coordinated,
}

/// Mihomo proxy engine configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MihomoConfig {
    pub binary_path: PathBuf,
    pub config_dir: PathBuf,
    pub api_address: String,
    pub api_secret: String,
    pub mixed_port: u16,
    pub log_level: String,
}

/// `ProxyGuard` health monitoring configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyGuardConfig {
    pub check_interval_secs: u64,
    pub max_restart_attempts: u32,
    pub restart_cooldown_secs: u64,
}

/// Site reachability probe configuration (Feature 003).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeConfig {
    pub timeout_secs: u64,
    pub max_concurrent: usize,
}

/// User notification preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub enabled: bool,
    pub sound: bool,
}

/// Top-level application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub deployment_mode: DeploymentMode,
    pub install_root: PathBuf,
    pub mihomo: MihomoConfig,
    pub proxy_guard: ProxyGuardConfig,
    pub probe: ProbeConfig,
    pub non_target_probe_sites: Vec<String>,
    pub notifications: NotificationConfig,
}

impl AppConfig {
    /// Returns default configuration for testing and initial setup.
    #[must_use]
    pub fn default_for(install_root: PathBuf) -> Self {
        Self {
            deployment_mode: DeploymentMode::WindowsOnly,
            mihomo: MihomoConfig {
                binary_path: install_root.join("bin").join("mihomo"),
                config_dir: install_root.join("data").join("mihomo"),
                api_address: "127.0.0.1:9090".to_string(),
                api_secret: String::new(),
                mixed_port: 7890,
                log_level: "warning".to_string(),
            },
            proxy_guard: ProxyGuardConfig {
                check_interval_secs: 3,
                max_restart_attempts: 3,
                restart_cooldown_secs: 10,
            },
            probe: ProbeConfig {
                timeout_secs: 5,
                max_concurrent: 10,
            },
            non_target_probe_sites: vec![
                "https://www.baidu.com".to_string(),
                "https://www.bing.com".to_string(),
            ],
            notifications: NotificationConfig {
                enabled: true,
                sound: true,
            },
            install_root,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deployment_mode_roundtrip() {
        let modes = vec![
            DeploymentMode::WindowsOnly,
            DeploymentMode::WslOnly,
            DeploymentMode::LinuxOnly,
            DeploymentMode::Coordinated,
        ];
        assert_eq!(modes.len(), 4, "must cover all 4 deployment modes");
        for m in &modes {
            let json = serde_json::to_string(m).expect("serialize");
            let back: DeploymentMode = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(&back, m);
        }
    }

    #[test]
    fn deployment_mode_snake_case_serialization() {
        assert_eq!(
            serde_json::to_string(&DeploymentMode::WindowsOnly).expect("serialize"),
            "\"windows_only\""
        );
    }

    #[test]
    fn mihomo_config_roundtrip() {
        let config = MihomoConfig {
            binary_path: PathBuf::from("/opt/goguo/bin/mihomo"),
            config_dir: PathBuf::from("/opt/goguo/data/mihomo"),
            api_address: "127.0.0.1:9090".to_string(),
            api_secret: "secret123".to_string(),
            mixed_port: 7890,
            log_level: "warning".to_string(),
        };
        let json = serde_json::to_string(&config).expect("serialize");
        let back: MihomoConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.binary_path, config.binary_path);
        assert_eq!(back.api_address, config.api_address);
        assert_eq!(back.mixed_port, 7890);
        assert_eq!(back.log_level, "warning");
    }

    #[test]
    fn proxy_guard_config_roundtrip() {
        let config = ProxyGuardConfig {
            check_interval_secs: 3,
            max_restart_attempts: 3,
            restart_cooldown_secs: 10,
        };
        let json = serde_json::to_string(&config).expect("serialize");
        let back: ProxyGuardConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.check_interval_secs, 3);
        assert_eq!(back.max_restart_attempts, 3);
        assert_eq!(back.restart_cooldown_secs, 10);
    }

    #[test]
    fn app_config_default_for_roundtrip() {
        let root = PathBuf::from("/opt/goguo");
        let config = AppConfig::default_for(root.clone());
        let json = serde_json::to_string_pretty(&config).expect("serialize");
        let back: AppConfig = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(back.deployment_mode, DeploymentMode::WindowsOnly);
        assert_eq!(back.install_root, root);
        assert_eq!(back.mihomo.mixed_port, 7890);
        assert_eq!(back.mihomo.binary_path, root.join("bin").join("mihomo"));
        assert_eq!(back.proxy_guard.check_interval_secs, 3);
        assert_eq!(back.proxy_guard.max_restart_attempts, 3);
        assert_eq!(back.probe.timeout_secs, 5);
        assert_eq!(back.non_target_probe_sites.len(), 2);
        assert!(back.notifications.enabled);
    }

    #[test]
    fn app_config_field_completeness() {
        let config = AppConfig::default_for(PathBuf::from("/tmp/goguo"));
        // Verify all major fields are populated (not empty/default-unintentionally)
        assert!(!config.mihomo.api_address.is_empty());
        assert!(!config.mihomo.log_level.is_empty());
        assert!(!config.non_target_probe_sites.is_empty());
    }
}
