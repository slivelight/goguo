use std::path::PathBuf;

use crate::adapters::PlatformAdapter;
use crate::models::baseline::{
    BaselineSnapshot, EnvironmentInfo, Platform, StateItem, StateItemCategory,
};
use crate::models::recovery::{ItemResult, RecoveryItem};
use crate::services::recovery::RecoveryManager;
use crate::storage::baseline_storage::BaselineStorage;

/// Result of comparing a single state item against baseline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComparisonResult {
    Match,
    Deviated {
        baseline_value: serde_json::Value,
        current_value: serde_json::Value,
    },
    MissingInBaseline,
}

/// A single item's comparison outcome.
pub struct ComparisonItem {
    pub state_item_id: String,
    pub result: ComparisonResult,
}

/// Summary of collected state items grouped by category.
pub struct StateSummary {
    pub total: usize,
    pub restorable: Vec<StateItem>,
    pub detectable: Vec<StateItem>,
    pub excluded: Vec<StateItem>,
}

/// Manages the baseline snapshot lifecycle: collect, confirm, compare, restore.
#[allow(dead_code)] // audit_dir used by T4.2/T4.3/T9.1
pub struct BaselineManager {
    adapters: Vec<Box<dyn PlatformAdapter + Send + Sync>>,
    storage: BaselineStorage,
    pub(crate) audit_dir: PathBuf,
}

impl BaselineManager {
    #[must_use]
    pub fn new(
        adapters: Vec<Box<dyn PlatformAdapter + Send + Sync>>,
        storage: BaselineStorage,
        audit_dir: PathBuf,
    ) -> Self {
        Self {
            adapters,
            storage,
            audit_dir,
        }
    }

    /// Collect state items from all registered adapters and persist as the initial snapshot.
    ///
    /// # Errors
    ///
    /// Returns an error if any adapter fails critically or persistence fails.
    pub fn collect_initial_snapshot(&self) -> std::io::Result<BaselineSnapshot> {
        let mut items: Vec<StateItem> = Vec::new();
        for adapter in &self.adapters {
            items.extend(adapter.read_state_items());
        }

        let snapshot = BaselineSnapshot {
            version: 0, // initial, not yet confirmed
            timestamp: chrono::Utc::now().to_rfc3339(),
            environment: EnvironmentInfo {
                os_name: std::env::consts::OS.to_string(),
                os_version: String::new(),
                hostname: hostname::get().to_string_lossy().to_string(),
                deployment_mode: String::new(),
            },
            items,
        };

        self.storage.save_initial_snapshot(&snapshot)?;
        Ok(snapshot)
    }

    /// Get a summary of the current state by collecting fresh items and grouping by category.
    ///
    /// # Errors
    ///
    /// Returns an error if adapter collection fails.
    pub fn get_state_summary(&self) -> std::io::Result<StateSummary> {
        let mut restorable = Vec::new();
        let mut detectable = Vec::new();
        let mut excluded = Vec::new();

        for adapter in &self.adapters {
            for item in adapter.read_state_items() {
                match item.category {
                    StateItemCategory::Restorable => restorable.push(item),
                    StateItemCategory::Detectable => detectable.push(item),
                    StateItemCategory::Excluded => excluded.push(item),
                }
            }
        }

        let total = restorable.len() + detectable.len() + excluded.len();
        Ok(StateSummary {
            total,
            restorable,
            detectable,
            excluded,
        })
    }

    /// Load the initial snapshot if it exists.
    ///
    /// # Errors
    ///
    /// Returns an error if the file exists but cannot be read.
    pub fn get_initial_snapshot(&self) -> std::io::Result<Option<BaselineSnapshot>> {
        self.storage.load_initial_snapshot()
    }

    /// Load the latest confirmed baseline, if any.
    ///
    /// # Errors
    ///
    /// Returns an error if the file exists but cannot be read.
    pub fn get_confirmed_baseline(&self) -> std::io::Result<Option<BaselineSnapshot>> {
        self.storage.load_latest_baseline()
    }

    /// Form a baseline candidate from the initial snapshot (version 1).
    ///
    /// # Errors
    ///
    /// Returns an error if no initial snapshot exists or persistence fails.
    pub fn form_baseline(&self) -> std::io::Result<BaselineSnapshot> {
        let snapshot = self
            .storage
            .load_initial_snapshot()?
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No initial snapshot"))?;

        let baseline = BaselineSnapshot {
            version: 1,
            ..snapshot
        };
        Ok(baseline)
    }

    /// Confirm the baseline candidate and persist as versioned baseline.
    ///
    /// # Errors
    ///
    /// Returns an error if no candidate exists or persistence fails.
    pub fn confirm_baseline(&self) -> std::io::Result<BaselineSnapshot> {
        let baseline = self.form_baseline()?;
        self.storage.save_baseline(&baseline)?;
        Ok(baseline)
    }

    /// Compare current state against the latest confirmed baseline.
    ///
    /// # Errors
    ///
    /// Returns an error if no baseline exists or current state cannot be read.
    pub fn compare_with_baseline(&self) -> std::io::Result<Vec<ComparisonItem>> {
        let baseline = self
            .storage
            .load_latest_baseline()?
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No confirmed baseline"))?;

        // Collect current state.
        let mut current_items: Vec<StateItem> = Vec::new();
        for adapter in &self.adapters {
            current_items.extend(adapter.read_state_items());
        }

        // Build a map of baseline values by item ID.
        let baseline_map: std::collections::HashMap<String, &serde_json::Value> = baseline
            .items
            .iter()
            .map(|i| (i.id.clone(), &i.value))
            .collect();

        let mut results = Vec::new();

        // Check each current item against baseline.
        for item in &current_items {
            let result = baseline_map.get(&item.id).map_or(
                ComparisonResult::MissingInBaseline,
                |baseline_value| {
                    if &item.value == *baseline_value {
                        ComparisonResult::Match
                    } else {
                        ComparisonResult::Deviated {
                            baseline_value: (*baseline_value).clone(),
                            current_value: item.value.clone(),
                        }
                    }
                },
            );
            results.push(ComparisonItem {
                state_item_id: item.id.clone(),
                result,
            });
        }

        Ok(results)
    }

    /// Restore all restorable items to their baseline values.
    ///
    /// Iterates through adapters, attempts to write each restorable item's
    /// baseline value back, and tracks results via the `RecoveryManager`.
    ///
    /// Restore all restorable items to their baseline values.
    ///
    /// # Errors
    ///
    /// Returns an error if no baseline exists or the recovery task cannot be created.
    ///
    /// # Panics
    ///
    /// Panics if a restorable item from the task is not found in the baseline (logic error).
    pub fn restore_to_baseline(&self) -> std::io::Result<RestoreResult> {
        let baseline = self
            .storage
            .load_latest_baseline()?
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "No confirmed baseline")
            })?;

        let recovery_mgr = RecoveryManager::new(self.audit_dir.join("state"))?;

        // Collect restorable baseline items as recovery items.
        let pending: Vec<RecoveryItem> = baseline
            .items
            .iter()
            .filter(|i| i.category == StateItemCategory::Restorable)
            .map(|i| RecoveryItem {
                state_item_id: i.id.clone(),
                target_value: i.value.clone(),
                result: None,
                failure_reason: None,
            })
            .collect();

        let task = recovery_mgr.create_task(pending)?;
        recovery_mgr.start_task().map_err(std::io::Error::other)?;

        // Build maps of baseline values and platforms by item ID.
        let baseline_values: std::collections::HashMap<String, &serde_json::Value> = baseline
            .items
            .iter()
            .map(|i| (i.id.clone(), &i.value))
            .collect();
        let baseline_platforms: std::collections::HashMap<String, Platform> = baseline
            .items
            .iter()
            .map(|i| (i.id.clone(), i.platform.clone()))
            .collect();

        // Attempt to restore each restorable item via adapters.
        let mut succeeded = 0usize;
        let mut failed = 0usize;
        for item_id in task.pending_items.iter().map(|i| i.state_item_id.clone()) {
            let target_value = baseline_values
                .get(&item_id)
                .expect("restorable item must exist in baseline");
            let platform = baseline_platforms
                .get(&item_id)
                .cloned()
                .unwrap_or(Platform::Windows);

            let restore_item = StateItem {
                id: item_id.clone(),
                platform,
                category: StateItemCategory::Restorable,
                value: (*target_value).clone(),
                collected_at: String::new(),
                classification_reason: String::new(),
            };

            let mut write_result = Err("No matching adapter".to_string());
            for adapter in &self.adapters {
                let defs = adapter.state_item_definitions();
                if defs.iter().any(|d| d.id == item_id) {
                    write_result = adapter.write_state(&restore_item);
                    break;
                }
            }

            match write_result {
                Ok(()) => {
                    recovery_mgr
                        .complete_item(&item_id, ItemResult::Success, None)
                        .map_err(std::io::Error::other)?;
                    succeeded += 1;
                }
                Err(reason) => {
                    recovery_mgr
                        .complete_item(&item_id, ItemResult::Failure, Some(reason))
                        .map_err(std::io::Error::other)?;
                    failed += 1;
                }
            }
        }

        let final_task = recovery_mgr.finalize_task().map_err(std::io::Error::other)?;

        Ok(RestoreResult {
            task: final_task,
            succeeded,
            failed,
        })
    }
}

/// Result of a restore-to-baseline operation.
pub struct RestoreResult {
    pub task: crate::models::recovery::RecoveryTask,
    pub succeeded: usize,
    pub failed: usize,
}

// hostname helper — avoids adding a dependency for a simple call.
mod hostname {
    use std::ffi::OsString;

    pub fn get() -> OsString {
        #[cfg(target_os = "linux")]
        {
            let name = std::fs::read_to_string("/proc/sys/kernel/hostname")
                .unwrap_or_default()
                .trim()
                .to_string();
            OsString::from(name)
        }
        #[cfg(target_os = "windows")]
        {
            std::env::var("COMPUTERNAME")
                .map_or_else(|_| OsString::from("unknown"), OsString::from)
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            OsString::from("unknown")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::{PlatformAdapter, StateItemDefinition};
    use crate::models::baseline::Platform;
    use tempfile::TempDir;

    struct MockAdapter {
        platform: Platform,
        items: Vec<StateItem>,
    }

    impl MockAdapter {
        fn with_items(platform: Platform, items: Vec<StateItem>) -> Self {
            Self { platform, items }
        }
    }

    impl PlatformAdapter for MockAdapter {
        fn platform(&self) -> Platform {
            self.platform.clone()
        }
        fn state_item_definitions(&self) -> Vec<StateItemDefinition> {
            self.items
                .iter()
                .map(|i| StateItemDefinition {
                    id: i.id.clone(),
                    category: i.category.clone(),
                    description: String::new(),
                })
                .collect()
        }
        fn read_state_items(&self) -> Vec<StateItem> {
            self.items.clone()
        }
        fn write_state(&self, _item: &StateItem) -> Result<(), String> {
            Ok(())
        }
    }

    fn make_item(id: &str, platform: Platform, category: StateItemCategory) -> StateItem {
        StateItem {
            id: id.to_string(),
            platform,
            category,
            value: serde_json::json!("value"),
            collected_at: "2026-05-19T12:00:00Z".to_string(),
            classification_reason: "test".to_string(),
        }
    }

    fn setup_manager(
        adapters: Vec<Box<dyn PlatformAdapter + Send + Sync>>,
    ) -> (TempDir, BaselineManager) {
        let dir = TempDir::new().expect("temp dir");
        let storage = BaselineStorage::new(dir.path().join("baseline"));
        let mgr = BaselineManager::new(
            adapters,
            storage,
            dir.path().join("audit"),
        );
        (dir, mgr)
    }

    #[test]
    fn collect_from_single_adapter() {
        let items = vec![
            make_item("win-proxy", Platform::Windows, StateItemCategory::Restorable),
            make_item("win-dns", Platform::Windows, StateItemCategory::Detectable),
        ];
        let adapter = MockAdapter::with_items(Platform::Windows, items);
        let (_dir, mgr) = setup_manager(vec![Box::new(adapter)]);

        let snapshot = mgr.collect_initial_snapshot().expect("collect");
        assert_eq!(snapshot.items.len(), 2);
        assert_eq!(snapshot.version, 0);

        // Verify persisted.
        let loaded = mgr
            .storage
            .load_initial_snapshot()
            .expect("load")
            .expect("exists");
        assert_eq!(loaded.items.len(), 2);
    }

    #[test]
    fn collect_from_multiple_adapters() {
        let win_items = vec![
            make_item("win-proxy", Platform::Windows, StateItemCategory::Restorable),
        ];
        let wsl_items = vec![
            make_item("wsl-env", Platform::Wsl, StateItemCategory::Restorable),
            make_item("wsl-reach", Platform::Wsl, StateItemCategory::Detectable),
        ];
        let adapters: Vec<Box<dyn PlatformAdapter + Send + Sync>> = vec![
            Box::new(MockAdapter::with_items(Platform::Windows, win_items)),
            Box::new(MockAdapter::with_items(Platform::Wsl, wsl_items)),
        ];
        let (_dir, mgr) = setup_manager(adapters);

        let snapshot = mgr.collect_initial_snapshot().expect("collect");
        assert_eq!(snapshot.items.len(), 3);
    }

    #[test]
    fn get_state_summary_groups_by_category() {
        let items = vec![
            make_item("a", Platform::Windows, StateItemCategory::Restorable),
            make_item("b", Platform::Windows, StateItemCategory::Restorable),
            make_item("c", Platform::Windows, StateItemCategory::Detectable),
            make_item("d", Platform::Windows, StateItemCategory::Excluded),
        ];
        let adapter = MockAdapter::with_items(Platform::Windows, items);
        let (_dir, mgr) = setup_manager(vec![Box::new(adapter)]);

        let summary = mgr.get_state_summary().expect("summary");
        assert_eq!(summary.total, 4);
        assert_eq!(summary.restorable.len(), 2);
        assert_eq!(summary.detectable.len(), 1);
        assert_eq!(summary.excluded.len(), 1);
    }

    #[test]
    fn form_and_confirm_baseline() {
        let items = vec![
            make_item("win-proxy", Platform::Windows, StateItemCategory::Restorable),
        ];
        let adapter = MockAdapter::with_items(Platform::Windows, items);
        let (_dir, mgr) = setup_manager(vec![Box::new(adapter)]);

        mgr.collect_initial_snapshot().expect("collect");

        let baseline = mgr.confirm_baseline().expect("confirm");
        assert_eq!(baseline.version, 1);
        assert_eq!(baseline.items.len(), 1);

        // Verify persisted as baseline-v1.
        let loaded = mgr
            .storage
            .load_baseline(1)
            .expect("load")
            .expect("exists");
        assert_eq!(loaded.version, 1);
    }

    #[test]
    fn form_baseline_fails_without_initial_snapshot() {
        let adapter = MockAdapter::with_items(Platform::Windows, vec![]);
        let (_dir, mgr) = setup_manager(vec![Box::new(adapter)]);

        let result = mgr.form_baseline();
        assert!(result.is_err());
    }

    #[test]
    fn collect_empty_adapters() {
        let (_dir, mgr) = setup_manager(vec![]);
        let snapshot = mgr.collect_initial_snapshot().expect("collect");
        assert!(snapshot.items.is_empty());
    }

    #[test]
    fn compare_all_match() {
        let items = vec![
            make_item("a", Platform::Windows, StateItemCategory::Restorable),
            make_item("b", Platform::Windows, StateItemCategory::Detectable),
        ];
        let adapter = MockAdapter::with_items(Platform::Windows, items);
        let (_dir, mgr) = setup_manager(vec![Box::new(adapter)]);

        mgr.collect_initial_snapshot().expect("collect");
        mgr.confirm_baseline().expect("confirm");

        let results = mgr.compare_with_baseline().expect("compare");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].result, ComparisonResult::Match);
        assert_eq!(results[1].result, ComparisonResult::Match);
    }

    #[test]
    fn compare_detects_deviation() {
        let dir = TempDir::new().expect("temp dir");
        let baseline_dir = dir.path().join("baseline");

        // Step 1: collect + confirm with baseline values.
        let baseline_items = vec![
            make_item("a", Platform::Windows, StateItemCategory::Restorable),
            make_item("b", Platform::Windows, StateItemCategory::Restorable),
        ];
        let adapter = MockAdapter::with_items(Platform::Windows, baseline_items);
        let storage = BaselineStorage::new(baseline_dir.clone());
        let mgr = BaselineManager::new(
            vec![Box::new(adapter)],
            storage,
            dir.path().join("audit"),
        );
        mgr.collect_initial_snapshot().expect("collect");
        mgr.confirm_baseline().expect("confirm");

        // Step 2: create new manager with modified adapter (item "a" changed).
        let modified_items = vec![
            StateItem {
                id: "a".to_string(),
                platform: Platform::Windows,
                category: StateItemCategory::Restorable,
                value: serde_json::json!("changed"),
                collected_at: "2026-05-19T12:01:00Z".to_string(),
                classification_reason: "test".to_string(),
            },
            make_item("b", Platform::Windows, StateItemCategory::Restorable),
        ];
        let modified_adapter =
            MockAdapter::with_items(Platform::Windows, modified_items);
        let storage2 = BaselineStorage::new(baseline_dir);
        let mgr2 = BaselineManager::new(
            vec![Box::new(modified_adapter)],
            storage2,
            dir.path().join("audit"),
        );

        let results = mgr2.compare_with_baseline().expect("compare");
        assert_eq!(results.len(), 2);

        let a_result = results.iter().find(|r| r.state_item_id == "a").expect("a");
        assert!(matches!(a_result.result, ComparisonResult::Deviated { .. }));

        let b_result = results.iter().find(|r| r.state_item_id == "b").expect("b");
        assert_eq!(b_result.result, ComparisonResult::Match);
    }

    #[test]
    fn compare_fails_without_baseline() {
        let adapter = MockAdapter::with_items(Platform::Windows, vec![]);
        let (_dir, mgr) = setup_manager(vec![Box::new(adapter)]);

        let result = mgr.compare_with_baseline();
        assert!(result.is_err());
    }

    /// Mock adapter where write always fails.
    struct FailWriteAdapter {
        items: Vec<StateItem>,
    }

    impl FailWriteAdapter {
        fn with_items(items: Vec<StateItem>) -> Self {
            Self { items }
        }
    }

    impl PlatformAdapter for FailWriteAdapter {
        fn platform(&self) -> Platform {
            Platform::Windows
        }
        fn state_item_definitions(&self) -> Vec<StateItemDefinition> {
            self.items
                .iter()
                .map(|i| StateItemDefinition {
                    id: i.id.clone(),
                    category: i.category.clone(),
                    description: String::new(),
                })
                .collect()
        }
        fn read_state_items(&self) -> Vec<StateItem> {
            self.items.clone()
        }
        fn write_state(&self, item: &StateItem) -> Result<(), String> {
            Err(format!("Write failed for {}", item.id))
        }
    }

    #[test]
    fn restore_all_succeed() {
        let items = vec![
            make_item("a", Platform::Windows, StateItemCategory::Restorable),
            make_item("b", Platform::Windows, StateItemCategory::Restorable),
        ];
        let adapter = MockAdapter::with_items(Platform::Windows, items);
        let dir = TempDir::new().expect("temp dir");
        let storage = BaselineStorage::new(dir.path().join("baseline"));
        let mgr = BaselineManager::new(
            vec![Box::new(adapter)],
            storage,
            dir.path().to_path_buf(),
        );

        mgr.collect_initial_snapshot().expect("collect");
        mgr.confirm_baseline().expect("confirm");

        let result = mgr.restore_to_baseline().expect("restore");
        assert_eq!(result.succeeded, 2);
        assert_eq!(result.failed, 0);
    }

    #[test]
    fn restore_partial_failure() {
        let items = vec![
            make_item("a", Platform::Windows, StateItemCategory::Restorable),
            make_item("b", Platform::Windows, StateItemCategory::Restorable),
        ];
        // Use FailWriteAdapter to simulate write failures.
        let fail_adapter = FailWriteAdapter::with_items(items.clone());
        let dir = TempDir::new().expect("temp dir");
        let storage = BaselineStorage::new(dir.path().join("baseline"));

        // First, collect + confirm with a working adapter.
        let ok_adapter = MockAdapter::with_items(Platform::Windows, items);
        let mgr = BaselineManager::new(
            vec![Box::new(ok_adapter)],
            storage,
            dir.path().to_path_buf(),
        );
        mgr.collect_initial_snapshot().expect("collect");
        mgr.confirm_baseline().expect("confirm");

        // Now restore with the failing adapter.
        let storage2 = BaselineStorage::new(dir.path().join("baseline"));
        let mgr2 = BaselineManager::new(
            vec![Box::new(fail_adapter)],
            storage2,
            dir.path().to_path_buf(),
        );
        let result = mgr2.restore_to_baseline().expect("restore");
        assert_eq!(result.succeeded, 0);
        assert_eq!(result.failed, 2);
    }

    #[test]
    fn restore_fails_without_baseline() {
        let adapter = MockAdapter::with_items(Platform::Windows, vec![]);
        let (_dir, mgr) = setup_manager(vec![Box::new(adapter)]);
        let result = mgr.restore_to_baseline();
        assert!(result.is_err());
    }
}
