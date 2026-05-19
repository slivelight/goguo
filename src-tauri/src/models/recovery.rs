use serde::{Deserialize, Serialize};

/// Status of a recovery task in the 5-state machine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    UserAcknowledged,
}

/// Result of a single recovery item attempt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemResult {
    Success,
    Failure,
    Skipped,
}

/// A single item within a recovery task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryItem {
    pub state_item_id: String,
    pub target_value: serde_json::Value,
    pub result: Option<ItemResult>,
    pub failure_reason: Option<String>,
}

/// A recovery task tracking progress of restoring state items to baseline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryTask {
    pub id: String,
    pub created_at: String,
    pub status: RecoveryStatus,
    pub pending_items: Vec<RecoveryItem>,
    pub completed_items: Vec<RecoveryItem>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_status_all_five_states_roundtrip() {
        let states = vec![
            RecoveryStatus::Pending,
            RecoveryStatus::InProgress,
            RecoveryStatus::Completed,
            RecoveryStatus::Failed,
            RecoveryStatus::UserAcknowledged,
        ];
        assert_eq!(states.len(), 5, "must cover all 5 states");
        for s in &states {
            let json = serde_json::to_string(s).expect("serialize");
            let back: RecoveryStatus = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(&back, s);
        }
    }

    #[test]
    fn recovery_status_snake_case_serialization() {
        assert_eq!(
            serde_json::to_string(&RecoveryStatus::InProgress).expect("serialize"),
            "\"in_progress\""
        );
        assert_eq!(
            serde_json::to_string(&RecoveryStatus::UserAcknowledged).expect("serialize"),
            "\"user_acknowledged\""
        );
    }

    #[test]
    fn item_result_roundtrip() {
        let results = vec![ItemResult::Success, ItemResult::Failure, ItemResult::Skipped];
        for r in &results {
            let json = serde_json::to_string(r).expect("serialize");
            let back: ItemResult = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(&back, r);
        }
    }

    #[test]
    fn recovery_item_roundtrip() {
        let item = RecoveryItem {
            state_item_id: "win-system-proxy".to_string(),
            target_value: serde_json::json!({"ProxyEnable": 0}),
            result: Some(ItemResult::Success),
            failure_reason: None,
        };
        let json = serde_json::to_string(&item).expect("serialize");
        let back: RecoveryItem = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.state_item_id, item.state_item_id);
        assert_eq!(back.result, Some(ItemResult::Success));
        assert!(back.failure_reason.is_none());
    }

    #[test]
    fn recovery_item_with_failure_roundtrip() {
        let item = RecoveryItem {
            state_item_id: "win-hosts".to_string(),
            target_value: serde_json::json!("original content"),
            result: Some(ItemResult::Failure),
            failure_reason: Some("Permission denied".to_string()),
        };
        let json = serde_json::to_string(&item).expect("serialize");
        let back: RecoveryItem = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.result, Some(ItemResult::Failure));
        assert_eq!(back.failure_reason, Some("Permission denied".to_string()));
    }

    #[test]
    fn recovery_task_roundtrip() {
        let task = RecoveryTask {
            id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            created_at: "2026-05-19T12:00:00Z".to_string(),
            status: RecoveryStatus::InProgress,
            pending_items: vec![RecoveryItem {
                state_item_id: "win-pac".to_string(),
                target_value: serde_json::json!(""),
                result: None,
                failure_reason: None,
            }],
            completed_items: vec![RecoveryItem {
                state_item_id: "win-system-proxy".to_string(),
                target_value: serde_json::json!({"ProxyEnable": 0}),
                result: Some(ItemResult::Success),
                failure_reason: None,
            }],
        };
        let json = serde_json::to_string_pretty(&task).expect("serialize");
        let back: RecoveryTask = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.id, task.id);
        assert_eq!(back.status, RecoveryStatus::InProgress);
        assert_eq!(back.pending_items.len(), 1);
        assert_eq!(back.completed_items.len(), 1);
    }
}
