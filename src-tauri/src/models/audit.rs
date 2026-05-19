use serde::{Deserialize, Serialize};

/// Types of auditable actions in the system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    BaselineCollect,
    BaselineConfirm,
    StateRestore,
    ProxyGuardRestart,
    ProxyGuardRecovery,
    RuleApply,
    ConfigChange,
}

/// Result of an audited action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditResult {
    Success,
    Failure,
    Skipped,
}

/// A structured audit record (appended to JSONL log).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    pub timestamp: String,
    pub action: AuditAction,
    pub target: String,
    pub result: AuditResult,
    pub reason: Option<String>,
    pub details: serde_json::Value,
}

/// Five-element failure explanation presented to the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureExplanation {
    pub cause: String,
    pub attempted_actions: Vec<String>,
    pub attempt_count: (u32, u32),
    pub suggested_action: String,
    pub needs_manual_intervention: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_action_roundtrip() {
        let actions = vec![
            AuditAction::BaselineCollect,
            AuditAction::BaselineConfirm,
            AuditAction::StateRestore,
            AuditAction::ProxyGuardRestart,
            AuditAction::ProxyGuardRecovery,
            AuditAction::RuleApply,
            AuditAction::ConfigChange,
        ];
        for a in &actions {
            let json = serde_json::to_string(a).expect("serialize");
            let back: AuditAction = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(&back, a);
        }
    }

    #[test]
    fn audit_action_snake_case_serialization() {
        assert_eq!(
            serde_json::to_string(&AuditAction::BaselineCollect).expect("serialize"),
            "\"baseline_collect\""
        );
        assert_eq!(
            serde_json::to_string(&AuditAction::ProxyGuardRestart).expect("serialize"),
            "\"proxy_guard_restart\""
        );
    }

    #[test]
    fn audit_result_roundtrip() {
        let results = vec![AuditResult::Success, AuditResult::Failure, AuditResult::Skipped];
        for r in &results {
            let json = serde_json::to_string(r).expect("serialize");
            let back: AuditResult = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(&back, r);
        }
    }

    #[test]
    fn audit_record_roundtrip() {
        let record = AuditRecord {
            timestamp: "2026-05-19T12:00:00Z".to_string(),
            action: AuditAction::StateRestore,
            target: "win-system-proxy".to_string(),
            result: AuditResult::Success,
            reason: None,
            details: serde_json::json!({
                "before": {"ProxyEnable": 1},
                "after": {"ProxyEnable": 0}
            }),
        };
        let json = serde_json::to_string(&record).expect("serialize");
        let back: AuditRecord = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.action, AuditAction::StateRestore);
        assert_eq!(back.target, "win-system-proxy");
        assert_eq!(back.result, AuditResult::Success);
        assert!(back.reason.is_none());
    }

    #[test]
    fn audit_record_with_failure_reason_roundtrip() {
        let record = AuditRecord {
            timestamp: "2026-05-19T12:00:00Z".to_string(),
            action: AuditAction::StateRestore,
            target: "win-hosts".to_string(),
            result: AuditResult::Failure,
            reason: Some("Permission denied writing hosts file".to_string()),
            details: serde_json::json!({"error": "EACCES"}),
        };
        let json = serde_json::to_string(&record).expect("serialize");
        let back: AuditRecord = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.result, AuditResult::Failure);
        assert_eq!(
            back.reason,
            Some("Permission denied writing hosts file".to_string())
        );
    }

    #[test]
    fn failure_explanation_roundtrip() {
        let explanation = FailureExplanation {
            cause: "Windows hosts file write permission denied".to_string(),
            attempted_actions: vec![
                "Direct file write".to_string(),
                "Elevated write via runas".to_string(),
            ],
            attempt_count: (3, 3),
            suggested_action: "Run GoGuo as administrator or manually edit C:\\Windows\\System32\\drivers\\etc\\hosts".to_string(),
            needs_manual_intervention: true,
        };
        let json = serde_json::to_string(&explanation).expect("serialize");
        let back: FailureExplanation = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.cause, explanation.cause);
        assert_eq!(back.attempted_actions.len(), 2);
        assert_eq!(back.attempt_count, (3, 3));
        assert!(back.needs_manual_intervention);
    }
}
