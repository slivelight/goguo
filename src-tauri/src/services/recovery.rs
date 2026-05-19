use std::fs;
use std::path::PathBuf;

use crate::adapters::PlatformAdapter;
use crate::models::baseline::{StateItem, StateItemCategory};
use crate::models::recovery::{ItemResult, RecoveryItem, RecoveryStatus, RecoveryTask};

/// Manages the recovery task state machine and single-task persistence.
///
/// Storage: `{state_dir}/recovery-task.json` — at most one task at a time.
pub struct RecoveryManager {
    state_dir: PathBuf,
}

impl RecoveryManager {
    /// # Errors
    ///
    /// Returns an error if the state directory cannot be created.
    pub fn new(state_dir: PathBuf) -> std::io::Result<Self> {
        fs::create_dir_all(&state_dir)?;
        Ok(Self { state_dir })
    }

    /// Load the current recovery task, if any.
    ///
    /// Returns `Ok(None)` if no task file exists or the task is in a terminal state.
    ///
    /// # Errors
    ///
    /// Returns an error if the file exists but cannot be read or parsed.
    pub fn load_task(&self) -> std::io::Result<Option<RecoveryTask>> {
        let path = self.state_dir.join("recovery-task.json");
        if !path.exists() {
            return Ok(None);
        }
        let data = fs::read_to_string(&path)?;
        let task: RecoveryTask = serde_json::from_str(&data)?;
        if task.status == RecoveryStatus::Completed
            || task.status == RecoveryStatus::UserAcknowledged
        {
            // Terminal state — clean up file.
            let _ = fs::remove_file(path);
            return Ok(None);
        }
        Ok(Some(task))
    }

    /// Save (upsert) the current recovery task to disk.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or file write fails.
    pub fn save_task(&self, task: &RecoveryTask) -> std::io::Result<()> {
        let path = self.state_dir.join("recovery-task.json");
        let json = serde_json::to_string_pretty(task)?;
        fs::write(path, json)
    }

    /// Create a new recovery task with `Pending` status.
    ///
    /// # Errors
    ///
    /// Returns an error if the task cannot be persisted.
    pub fn create_task(&self, pending_items: Vec<RecoveryItem>) -> std::io::Result<RecoveryTask> {
        let task = RecoveryTask {
            id: uuid::Uuid::new_v4().to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            status: RecoveryStatus::Pending,
            pending_items,
            completed_items: vec![],
        };
        self.save_task(&task)?;
        Ok(task)
    }

    /// Transition the task to `InProgress`.
    ///
    /// # Errors
    ///
    /// Returns an error if the transition is invalid or persistence fails.
    pub fn start_task(&self) -> Result<RecoveryTask, String> {
        let mut task = self
            .load_task()
            .map_err(|e| format!("IO error: {e}"))?
            .ok_or("No active recovery task")?;

        match task.status {
            RecoveryStatus::Pending | RecoveryStatus::Failed => {
                task.status = RecoveryStatus::InProgress;
            }
            other => {
                return Err(format!(
                    "Cannot start task in {other:?} state"
                ));
            }
        }

        self.save_task(&task)
            .map_err(|e| format!("IO error: {e}"))?;
        Ok(task)
    }

    /// Mark an item as completed (success or failure) and move it to the completed list.
    ///
    /// # Errors
    ///
    /// Returns an error if the task is not in `InProgress` state or persistence fails.
    pub fn complete_item(
        &self,
        state_item_id: &str,
        result: ItemResult,
        failure_reason: Option<String>,
    ) -> Result<RecoveryTask, String> {
        let mut task = self
            .load_task()
            .map_err(|e| format!("IO error: {e}"))?
            .ok_or("No active recovery task")?;

        if task.status != RecoveryStatus::InProgress {
            return Err(format!(
                "Cannot complete item when task is in {:?} state",
                task.status
            ));
        }

        let idx = task
            .pending_items
            .iter()
            .position(|i| i.state_item_id == state_item_id)
            .ok_or_else(|| format!("Item '{state_item_id}' not found in pending list"))?;

        let mut item = task.pending_items.remove(idx);
        item.result = Some(result);
        item.failure_reason = failure_reason;
        task.completed_items.push(item);

        self.save_task(&task)
            .map_err(|e| format!("IO error: {e}"))?;
        Ok(task)
    }

    /// Transition to `Completed` if all items succeeded, or `Failed` if any failed.
    ///
    /// # Errors
    ///
    /// Returns an error if the task is not in `InProgress` state.
    pub fn finalize_task(&self) -> Result<RecoveryTask, String> {
        let mut task = self
            .load_task()
            .map_err(|e| format!("IO error: {e}"))?
            .ok_or("No active recovery task")?;

        if task.status != RecoveryStatus::InProgress {
            return Err(format!(
                "Cannot finalize task in {:?} state",
                task.status
            ));
        }

        let has_failures = task.completed_items.iter().any(|i| {
            i.result == Some(ItemResult::Failure) || i.result == Some(ItemResult::Skipped)
        });

        task.status = if has_failures {
            RecoveryStatus::Failed
        } else {
            RecoveryStatus::Completed
        };

        self.save_task(&task)
            .map_err(|e| format!("IO error: {e}"))?;
        Ok(task)
    }

    /// Transition from `Failed` to `UserAcknowledged` (terminal).
    ///
    /// # Errors
    ///
    /// Returns an error if the task is not in `Failed` state.
    pub fn acknowledge_task(&self) -> Result<RecoveryTask, String> {
        let mut task = self
            .load_task()
            .map_err(|e| format!("IO error: {e}"))?
            .ok_or("No active recovery task")?;

        if task.status != RecoveryStatus::Failed {
            return Err(format!(
                "Cannot acknowledge task in {:?} state",
                task.status
            ));
        }

        task.status = RecoveryStatus::UserAcknowledged;
        self.save_task(&task)
            .map_err(|e| format!("IO error: {e}"))?;
        Ok(task)
    }

    /// Resume an incomplete recovery task.
    ///
    /// If a `Pending` or `InProgress` task exists, attempt to restore each
    /// remaining pending item via the provided adapters. Returns `Ok(None)` if
    /// no task needs resuming.
    ///
    /// # Errors
    ///
    /// Returns an error string if state transitions or persistence fail.
    pub fn resume_recovery(
        &self,
        adapters: &[Box<dyn PlatformAdapter>],
        baseline_values: &std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<Option<RecoveryTask>, String> {
        let Some(task) = self.load_task().map_err(|e| format!("IO: {e}"))? else {
            return Ok(None);
        };

        match task.status {
            RecoveryStatus::Pending | RecoveryStatus::InProgress => {}
            _ => return Ok(None),
        }

        // Ensure InProgress.
        let mut task = if task.status == RecoveryStatus::Pending {
            self.start_task()?
        } else {
            task
        };

        // Process remaining pending items.
        let item_ids: Vec<String> = task
            .pending_items
            .iter()
            .map(|i| i.state_item_id.clone())
            .collect();

        for item_id in item_ids {
            let target_value = baseline_values
                .get(&item_id)
                .ok_or_else(|| format!("Baseline value missing for '{item_id}'"))?
                .clone();

            let restore_item = StateItem {
                id: item_id.clone(),
                platform: adapters
                    .iter()
                    .find(|a| {
                        a.state_item_definitions()
                            .iter()
                            .any(|d| d.id == item_id)
                    })
                    .map_or(
                        crate::models::baseline::Platform::Windows,
                        |a| a.platform(),
                    ),
                category: StateItemCategory::Restorable,
                value: target_value,
                collected_at: String::new(),
                classification_reason: String::new(),
            };

            let mut write_result = Err("No matching adapter".to_string());
            for adapter in adapters {
                let defs = adapter.state_item_definitions();
                if defs.iter().any(|d| d.id == item_id) {
                    write_result = adapter.write_state(&restore_item);
                    break;
                }
            }

            match write_result {
                Ok(()) => {
                    self.complete_item(&item_id, ItemResult::Success, None)?;
                }
                Err(reason) => {
                    self.complete_item(&item_id, ItemResult::Failure, Some(reason))?;
                }
            }

            // Reload task state after each item.
            task = self
                .load_task()
                .map_err(|e| format!("IO: {e}"))?
                .unwrap_or(task);
        }

        let final_task = self.finalize_task()?;
        Ok(Some(final_task))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_pending_item(id: &str) -> RecoveryItem {
        RecoveryItem {
            state_item_id: id.to_string(),
            target_value: serde_json::json!("baseline-value"),
            result: None,
            failure_reason: None,
        }
    }

    fn setup() -> (TempDir, RecoveryManager) {
        let dir = TempDir::new().expect("temp dir");
        let state_dir = dir.path().join("state");
        let mgr = RecoveryManager::new(state_dir).expect("create");
        (dir, mgr)
    }

    #[test]
    fn no_task_returns_none() {
        let (_dir, mgr) = setup();
        assert!(mgr.load_task().expect("load").is_none());
    }

    #[test]
    fn create_and_load_task() {
        let (_dir, mgr) = setup();
        let _task = mgr
            .create_task(vec![make_pending_item("item-1")])
            .expect("create");

        let loaded = mgr.load_task().expect("load").expect("exists");
        assert_eq!(loaded.status, RecoveryStatus::Pending);
        assert_eq!(loaded.pending_items.len(), 1);
    }

    #[test]
    fn pending_to_in_progress() {
        let (_dir, mgr) = setup();
        mgr.create_task(vec![make_pending_item("item-1")])
            .expect("create");

        let task = mgr.start_task().expect("start");
        assert_eq!(task.status, RecoveryStatus::InProgress);
    }

    #[test]
    fn failed_to_in_progress_retry() {
        let (_dir, mgr) = setup();
        mgr.create_task(vec![make_pending_item("item-1")])
            .expect("create");
        mgr.start_task().expect("start");
        mgr.complete_item("item-1", ItemResult::Failure, Some("error".to_string()))
            .expect("complete");
        let task = mgr.finalize_task().expect("finalize");
        assert_eq!(task.status, RecoveryStatus::Failed);

        // Retry from Failed.
        let retried = mgr.start_task().expect("retry");
        assert_eq!(retried.status, RecoveryStatus::InProgress);
    }

    #[test]
    fn complete_all_items_to_completed() {
        let (_dir, mgr) = setup();
        mgr.create_task(vec![
            make_pending_item("item-1"),
            make_pending_item("item-2"),
        ])
        .expect("create");
        mgr.start_task().expect("start");

        mgr.complete_item("item-1", ItemResult::Success, None)
            .expect("complete 1");
        mgr.complete_item("item-2", ItemResult::Success, None)
            .expect("complete 2");

        let task = mgr.finalize_task().expect("finalize");
        assert_eq!(task.status, RecoveryStatus::Completed);

        // Completed task is terminal — load returns None and file is cleaned up.
        assert!(mgr.load_task().expect("load").is_none());
    }

    #[test]
    fn partial_failure_to_failed_then_acknowledge() {
        let (_dir, mgr) = setup();
        mgr.create_task(vec![
            make_pending_item("item-1"),
            make_pending_item("item-2"),
        ])
        .expect("create");
        mgr.start_task().expect("start");

        mgr.complete_item("item-1", ItemResult::Success, None)
            .expect("ok");
        mgr.complete_item("item-2", ItemResult::Failure, Some("denied".into()))
            .expect("fail");

        let task = mgr.finalize_task().expect("finalize");
        assert_eq!(task.status, RecoveryStatus::Failed);

        let ack = mgr.acknowledge_task().expect("ack");
        assert_eq!(ack.status, RecoveryStatus::UserAcknowledged);

        // UserAcknowledged is terminal.
        assert!(mgr.load_task().expect("load").is_none());
    }

    #[test]
    fn cannot_start_completed_task() {
        let (_dir, mgr) = setup();
        mgr.create_task(vec![make_pending_item("item-1")])
            .expect("create");
        mgr.start_task().expect("start");
        mgr.complete_item("item-1", ItemResult::Success, None)
            .expect("complete");
        mgr.finalize_task().expect("finalize");

        // Task file is cleaned up — no task to start.
        let result = mgr.start_task();
        assert!(result.is_err());
    }

    #[test]
    fn cannot_acknowledge_non_failed_task() {
        let (_dir, mgr) = setup();
        mgr.create_task(vec![make_pending_item("item-1")])
            .expect("create");
        mgr.start_task().expect("start");

        let result = mgr.acknowledge_task();
        assert!(result.is_err());
    }

    #[test]
    fn cannot_complete_item_when_not_in_progress() {
        let (_dir, mgr) = setup();
        mgr.create_task(vec![make_pending_item("item-1")])
            .expect("create");

        let result = mgr.complete_item("item-1", ItemResult::Success, None);
        assert!(result.is_err());
    }

    #[test]
    fn single_task_overwrite() {
        let (_dir, mgr) = setup();
        mgr.create_task(vec![make_pending_item("item-1")])
            .expect("create 1");
        mgr.start_task().expect("start");
        mgr.complete_item("item-1", ItemResult::Success, None)
            .expect("complete");
        mgr.finalize_task().expect("finalize");

        // Task is cleaned up — can create a new one.
        let task2 = mgr
            .create_task(vec![make_pending_item("item-2")])
            .expect("create 2");
        assert_eq!(task2.status, RecoveryStatus::Pending);
        assert_eq!(task2.pending_items[0].state_item_id, "item-2");
    }

    // --- Resume recovery tests ---

    struct ResumeMockAdapter {
        items: Vec<String>,
        should_fail: Vec<String>,
    }

    impl ResumeMockAdapter {
        fn new(items: Vec<&str>) -> Self {
            Self {
                items: items.into_iter().map(String::from).collect(),
                should_fail: vec![],
            }
        }

        fn with_failures(items: Vec<&str>, fail_ids: Vec<&str>) -> Self {
            Self {
                items: items.into_iter().map(String::from).collect(),
                should_fail: fail_ids.into_iter().map(String::from).collect(),
            }
        }
    }

    impl PlatformAdapter for ResumeMockAdapter {
        fn platform(&self) -> crate::models::baseline::Platform {
            crate::models::baseline::Platform::Windows
        }
        fn state_item_definitions(&self) -> Vec<crate::adapters::StateItemDefinition> {
            self.items
                .iter()
                .map(|id| crate::adapters::StateItemDefinition {
                    id: id.clone(),
                    category: crate::models::baseline::StateItemCategory::Restorable,
                    description: String::new(),
                })
                .collect()
        }
        fn read_state_items(&self) -> Vec<crate::models::baseline::StateItem> {
            vec![]
        }
        fn write_state(
            &self,
            item: &crate::models::baseline::StateItem,
        ) -> Result<(), String> {
            if self.should_fail.contains(&item.id) {
                Err(format!("Simulated failure for {}", item.id))
            } else {
                Ok(())
            }
        }
    }

    fn make_baseline_values(ids: &[&str]) -> std::collections::HashMap<String, serde_json::Value> {
        ids.iter()
            .map(|id| ((*id).to_string(), serde_json::json!("baseline")))
            .collect()
    }

    #[test]
    fn resume_pending_task_completes_all() {
        let (_dir, mgr) = setup();
        mgr.create_task(vec![
            make_pending_item("a"),
            make_pending_item("b"),
        ])
        .expect("create");

        let adapters: Vec<Box<dyn PlatformAdapter>> =
            vec![Box::new(ResumeMockAdapter::new(vec!["a", "b"]))];
        let baseline = make_baseline_values(&["a", "b"]);

        let result = mgr.resume_recovery(&adapters, &baseline).expect("resume");
        let task = result.expect("should have task");
        assert_eq!(task.status, RecoveryStatus::Completed);
        assert_eq!(task.completed_items.len(), 2);
    }

    #[test]
    fn resume_no_task_returns_none() {
        let (_dir, mgr) = setup();
        let adapters: Vec<Box<dyn PlatformAdapter>> =
            vec![Box::new(ResumeMockAdapter::new(vec![]))];
        let baseline = std::collections::HashMap::new();

        let result = mgr.resume_recovery(&adapters, &baseline).expect("resume");
        assert!(result.is_none());
    }

    #[test]
    fn resume_completed_task_returns_none() {
        let (_dir, mgr) = setup();
        mgr.create_task(vec![make_pending_item("a")])
            .expect("create");
        mgr.start_task().expect("start");
        mgr.complete_item("a", ItemResult::Success, None)
            .expect("complete");
        mgr.finalize_task().expect("finalize");

        let adapters: Vec<Box<dyn PlatformAdapter>> =
            vec![Box::new(ResumeMockAdapter::new(vec!["a"]))];
        let baseline = make_baseline_values(&["a"]);

        let result = mgr.resume_recovery(&adapters, &baseline).expect("resume");
        assert!(result.is_none());
    }

    #[test]
    fn resume_partial_failure_returns_failed() {
        let (_dir, mgr) = setup();
        mgr.create_task(vec![
            make_pending_item("a"),
            make_pending_item("b"),
        ])
        .expect("create");

        let adapters: Vec<Box<dyn PlatformAdapter>> = vec![Box::new(
            ResumeMockAdapter::with_failures(vec!["a", "b"], vec!["b"]),
        )];
        let baseline = make_baseline_values(&["a", "b"]);

        let result = mgr.resume_recovery(&adapters, &baseline).expect("resume");
        let task = result.expect("task");
        assert_eq!(task.status, RecoveryStatus::Failed);
        assert_eq!(task.completed_items.len(), 2);
    }
}
