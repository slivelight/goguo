use serde::{Deserialize, Serialize};

/// Target platform for a state item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Windows,
    Wsl,
    Linux,
}

/// Classification of a state item's recoverability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StateItemCategory {
    /// Can be restored to its baseline value.
    Restorable,
    /// Read-only; can be detected but not restored automatically.
    Detectable,
    /// Excluded from baseline management.
    Excluded,
}

/// System environment information captured at assessment time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentInfo {
    pub os_name: String,
    pub os_version: String,
    pub hostname: String,
    pub deployment_mode: String,
}

/// A single state item collected from the platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateItem {
    pub id: String,
    pub platform: Platform,
    pub category: StateItemCategory,
    pub value: serde_json::Value,
    pub collected_at: String,
    pub classification_reason: String,
}

/// A complete baseline snapshot containing all collected state items.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineSnapshot {
    pub version: u32,
    pub timestamp: String,
    pub environment: EnvironmentInfo,
    pub items: Vec<StateItem>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_serde_roundtrip() {
        let platforms = vec![Platform::Windows, Platform::Wsl, Platform::Linux];
        for p in &platforms {
            let json = serde_json::to_string(p).expect("serialize");
            let back: Platform = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(&back, p);
        }
    }

    #[test]
    fn platform_snake_case_serialization() {
        assert_eq!(
            serde_json::to_string(&Platform::Windows).expect("serialize"),
            "\"windows\""
        );
    }

    #[test]
    fn state_item_category_serde_roundtrip() {
        let cats = vec![
            StateItemCategory::Restorable,
            StateItemCategory::Detectable,
            StateItemCategory::Excluded,
        ];
        for c in &cats {
            let json = serde_json::to_string(c).expect("serialize");
            let back: StateItemCategory = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(&back, c);
        }
    }

    #[test]
    fn environment_info_serde_roundtrip() {
        let env = EnvironmentInfo {
            os_name: "Windows".to_string(),
            os_version: "10.0.19045".to_string(),
            hostname: "DESKTOP-ABC".to_string(),
            deployment_mode: "windows_only".to_string(),
        };
        let json = serde_json::to_string(&env).expect("serialize");
        let back: EnvironmentInfo = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.os_name, env.os_name);
        assert_eq!(back.os_version, env.os_version);
        assert_eq!(back.hostname, env.hostname);
        assert_eq!(back.deployment_mode, env.deployment_mode);
    }

    #[test]
    fn state_item_serde_roundtrip() {
        let item = StateItem {
            id: "win-system-proxy".to_string(),
            platform: Platform::Windows,
            category: StateItemCategory::Restorable,
            value: serde_json::json!({"ProxyEnable": 0, "ProxyServer": ""}),
            collected_at: "2026-05-19T12:00:00Z".to_string(),
            classification_reason: "Registry key, writable".to_string(),
        };
        let json = serde_json::to_string(&item).expect("serialize");
        let back: StateItem = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.id, item.id);
        assert_eq!(back.platform, item.platform);
        assert_eq!(back.category, item.category);
        assert_eq!(back.value, item.value);
    }

    #[test]
    fn baseline_snapshot_serde_roundtrip() {
        let snapshot = BaselineSnapshot {
            version: 1,
            timestamp: "2026-05-19T12:00:00Z".to_string(),
            environment: EnvironmentInfo {
                os_name: "Windows".to_string(),
                os_version: "10.0.19045".to_string(),
                hostname: "DESKTOP-ABC".to_string(),
                deployment_mode: "windows_only".to_string(),
            },
            items: vec![StateItem {
                id: "win-hosts".to_string(),
                platform: Platform::Windows,
                category: StateItemCategory::Restorable,
                value: serde_json::json!("127.0.0.1 localhost"),
                collected_at: "2026-05-19T12:00:00Z".to_string(),
                classification_reason: "File, writable".to_string(),
            }],
        };
        let json = serde_json::to_string_pretty(&snapshot).expect("serialize");
        let back: BaselineSnapshot = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.version, 1);
        assert_eq!(back.items.len(), 1);
        assert_eq!(back.items[0].id, "win-hosts");
    }
}
