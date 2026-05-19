use std::fs;
use std::path::PathBuf;

use crate::models::baseline::BaselineSnapshot;

/// Handles persistence of baseline snapshots as JSON files.
///
/// Storage layout:
/// - `{base_dir}/initial-snapshot.json` — first assessment snapshot
/// - `{base_dir}/baseline-v{N}.json` — confirmed baselines (versioned)
pub struct BaselineStorage {
    pub(crate) base_dir: PathBuf,
}

impl BaselineStorage {
    #[must_use]
    pub const fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Save the initial assessment snapshot.
    ///
    /// # Errors
    ///
    /// Returns an error if directory creation or file write fails.
    pub fn save_initial_snapshot(&self, snapshot: &BaselineSnapshot) -> std::io::Result<()> {
        fs::create_dir_all(&self.base_dir)?;
        let path = self.base_dir.join("initial-snapshot.json");
        let json = serde_json::to_string_pretty(snapshot)?;
        fs::write(path, json)
    }

    /// Load the initial assessment snapshot.
    ///
    /// # Errors
    ///
    /// Returns an error if the file exists but cannot be read or parsed.
    pub fn load_initial_snapshot(&self) -> std::io::Result<Option<BaselineSnapshot>> {
        let path = self.base_dir.join("initial-snapshot.json");
        if !path.exists() {
            return Ok(None);
        }
        let data = fs::read_to_string(path)?;
        let snapshot: BaselineSnapshot = serde_json::from_str(&data)?;
        Ok(Some(snapshot))
    }

    /// Save a confirmed baseline with the snapshot's version number.
    ///
    /// # Errors
    ///
    /// Returns an error if directory creation or file write fails.
    pub fn save_baseline(&self, snapshot: &BaselineSnapshot) -> std::io::Result<()> {
        fs::create_dir_all(&self.base_dir)?;
        let filename = format!("baseline-v{}.json", snapshot.version);
        let path = self.base_dir.join(filename);
        let json = serde_json::to_string_pretty(snapshot)?;
        fs::write(path, json)
    }

    /// Load a specific version of the confirmed baseline.
    ///
    /// # Errors
    ///
    /// Returns an error if the file exists but cannot be read or parsed.
    pub fn load_baseline(&self, version: u32) -> std::io::Result<Option<BaselineSnapshot>> {
        let filename = format!("baseline-v{version}.json");
        let path = self.base_dir.join(filename);
        if !path.exists() {
            return Ok(None);
        }
        let data = fs::read_to_string(path)?;
        let snapshot: BaselineSnapshot = serde_json::from_str(&data)?;
        Ok(Some(snapshot))
    }

    /// Load the latest (highest version) confirmed baseline.
    ///
    /// # Errors
    ///
    /// Returns an error if directory listing or file reading fails.
    pub fn load_latest_baseline(&self) -> std::io::Result<Option<BaselineSnapshot>> {
        if !self.base_dir.exists() {
            return Ok(None);
        }
        let mut max_version: u32 = 0;
        let mut latest: Option<BaselineSnapshot> = None;
        for entry in fs::read_dir(&self.base_dir)? {
            let entry = entry?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if let Some(v) = name_str
                .strip_prefix("baseline-v")
                .and_then(|s| s.strip_suffix(".json"))
                .and_then(|s| s.parse::<u32>().ok())
            {
                if v >= max_version {
                    max_version = v;
                    latest = self.load_baseline(v)?;
                }
            }
        }
        Ok(latest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::baseline::{EnvironmentInfo, Platform, StateItem, StateItemCategory};
    use tempfile::TempDir;

    fn sample_snapshot(version: u32) -> BaselineSnapshot {
        BaselineSnapshot {
            version,
            timestamp: "2026-05-19T12:00:00Z".to_string(),
            environment: EnvironmentInfo {
                os_name: "Windows".to_string(),
                os_version: "10.0.19045".to_string(),
                hostname: "TEST-PC".to_string(),
                deployment_mode: "windows_only".to_string(),
            },
            items: vec![StateItem {
                id: "win-system-proxy".to_string(),
                platform: Platform::Windows,
                category: StateItemCategory::Restorable,
                value: serde_json::json!({"ProxyEnable": 0}),
                collected_at: "2026-05-19T12:00:00Z".to_string(),
                classification_reason: "Registry key, writable".to_string(),
            }],
        }
    }

    #[test]
    fn save_and_load_initial_snapshot() {
        let dir = TempDir::new().expect("temp dir");
        let storage = BaselineStorage::new(dir.path().to_path_buf());
        let snapshot = sample_snapshot(1);

        storage
            .save_initial_snapshot(&snapshot)
            .expect("save initial");

        let loaded = storage
            .load_initial_snapshot()
            .expect("load initial")
            .expect("should exist");

        assert_eq!(loaded.version, snapshot.version);
        assert_eq!(loaded.items.len(), 1);
        assert_eq!(loaded.items[0].id, "win-system-proxy");
    }

    #[test]
    fn load_initial_snapshot_returns_none_when_absent() {
        let dir = TempDir::new().expect("temp dir");
        let storage = BaselineStorage::new(dir.path().to_path_buf());

        let result = storage.load_initial_snapshot().expect("no error");
        assert!(result.is_none());
    }

    #[test]
    fn save_and_load_baseline_versioned() {
        let dir = TempDir::new().expect("temp dir");
        let storage = BaselineStorage::new(dir.path().to_path_buf());

        let v1 = sample_snapshot(1);
        let v2 = sample_snapshot(2);

        storage.save_baseline(&v1).expect("save v1");
        storage.save_baseline(&v2).expect("save v2");

        let loaded_v1 = storage
            .load_baseline(1)
            .expect("load v1")
            .expect("should exist");
        assert_eq!(loaded_v1.version, 1);

        let loaded_v2 = storage
            .load_baseline(2)
            .expect("load v2")
            .expect("should exist");
        assert_eq!(loaded_v2.version, 2);
    }

    #[test]
    fn load_nonexistent_version_returns_none() {
        let dir = TempDir::new().expect("temp dir");
        let storage = BaselineStorage::new(dir.path().to_path_buf());

        let result = storage.load_baseline(99).expect("no error");
        assert!(result.is_none());
    }

    #[test]
    fn load_latest_baseline_returns_highest_version() {
        let dir = TempDir::new().expect("temp dir");
        let storage = BaselineStorage::new(dir.path().to_path_buf());

        storage.save_baseline(&sample_snapshot(1)).expect("save v1");
        storage.save_baseline(&sample_snapshot(3)).expect("save v3");
        storage.save_baseline(&sample_snapshot(2)).expect("save v2");

        let latest = storage
            .load_latest_baseline()
            .expect("load latest")
            .expect("should exist");
        assert_eq!(latest.version, 3);
    }

    #[test]
    fn load_latest_baseline_returns_none_when_empty() {
        let dir = TempDir::new().expect("temp dir");
        let storage = BaselineStorage::new(dir.path().to_path_buf());

        let result = storage.load_latest_baseline().expect("no error");
        assert!(result.is_none());
    }

    #[test]
    fn overwrite_initial_snapshot() {
        let dir = TempDir::new().expect("temp dir");
        let storage = BaselineStorage::new(dir.path().to_path_buf());

        let first = sample_snapshot(1);
        let second = BaselineSnapshot {
            version: 1,
            items: vec![],
            ..sample_snapshot(1)
        };

        storage.save_initial_snapshot(&first).expect("save first");
        storage
            .save_initial_snapshot(&second)
            .expect("overwrite");

        let loaded = storage
            .load_initial_snapshot()
            .expect("load")
            .expect("should exist");
        assert!(loaded.items.is_empty(), "should be the overwritten version");
    }
}
