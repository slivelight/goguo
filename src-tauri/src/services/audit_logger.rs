use std::path::PathBuf;
use std::sync::Mutex;

use crate::models::audit::{
    AuditAction, AuditRecord, AuditResult, FailureExplanation,
};
use crate::storage::audit_storage::{AuditQueryParams, AuditStorage};

/// Trait for audit logging operations, enabling dependency injection.
pub trait AuditLog: Send + Sync {
    /// Log a successful action.
    ///
    /// # Errors
    ///
    /// Returns an error if the record cannot be appended.
    fn log_success(&self, action: AuditAction, target: &str, details: serde_json::Value) -> std::io::Result<()>;
    /// Log a failed action.
    ///
    /// # Errors
    ///
    /// Returns an error if the record cannot be appended.
    fn log_failure(&self, action: AuditAction, target: &str, reason: &str, details: serde_json::Value) -> std::io::Result<()>;
}

impl AuditLog for AuditLogger {
    fn log_success(&self, action: AuditAction, target: &str, details: serde_json::Value) -> std::io::Result<()> {
        Self::log_success(self, action, target, details)
    }

    fn log_failure(&self, action: AuditAction, target: &str, reason: &str, details: serde_json::Value) -> std::io::Result<()> {
        Self::log_failure(self, action, target, reason, details)
    }
}

/// Mock audit log for testing.
pub struct MockAuditLog {
    records: Mutex<Vec<AuditRecord>>,
}

impl MockAuditLog {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            records: Mutex::new(Vec::new()),
        }
    }

    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    #[must_use]
    pub fn records(&self) -> Vec<AuditRecord> {
        self.records.lock().unwrap().clone()
    }
}

impl Default for MockAuditLog {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditLog for MockAuditLog {
    fn log_success(&self, action: AuditAction, target: &str, details: serde_json::Value) -> std::io::Result<()> {
        let record = AuditRecord {
            timestamp: chrono::Utc::now().to_rfc3339(),
            action,
            target: target.to_string(),
            result: AuditResult::Success,
            reason: None,
            details,
        };
        self.records.lock().unwrap().push(record);
        Ok(())
    }

    fn log_failure(&self, action: AuditAction, target: &str, reason: &str, details: serde_json::Value) -> std::io::Result<()> {
        let record = AuditRecord {
            timestamp: chrono::Utc::now().to_rfc3339(),
            action,
            target: target.to_string(),
            result: AuditResult::Failure,
            reason: Some(reason.to_string()),
            details,
        };
        self.records.lock().unwrap().push(record);
        Ok(())
    }
}

/// High-level audit logging service wrapping `AuditStorage`.
pub struct AuditLogger {
    storage: AuditStorage,
}

impl AuditLogger {
    /// # Errors
    ///
    /// Returns an error if the storage directory cannot be created.
    pub fn new(audit_dir: PathBuf) -> std::io::Result<Self> {
        std::fs::create_dir_all(&audit_dir)?;
        Ok(Self {
            storage: AuditStorage::new(audit_dir),
        })
    }

    /// Log a successful action.
    ///
    /// # Errors
    ///
    /// Returns an error if the record cannot be appended.
    pub fn log_success(
        &self,
        action: AuditAction,
        target: &str,
        details: serde_json::Value,
    ) -> std::io::Result<()> {
        let record = AuditRecord {
            timestamp: chrono::Utc::now().to_rfc3339(),
            action,
            target: target.to_string(),
            result: AuditResult::Success,
            reason: None,
            details,
        };
        self.storage.append(&record)
    }

    /// Log a failed action.
    ///
    /// # Errors
    ///
    /// Returns an error if the record cannot be appended.
    pub fn log_failure(
        &self,
        action: AuditAction,
        target: &str,
        reason: &str,
        details: serde_json::Value,
    ) -> std::io::Result<()> {
        let record = AuditRecord {
            timestamp: chrono::Utc::now().to_rfc3339(),
            action,
            target: target.to_string(),
            result: AuditResult::Failure,
            reason: Some(reason.to_string()),
            details,
        };
        self.storage.append(&record)
    }

    /// Log a skipped action.
    ///
    /// # Errors
    ///
    /// Returns an error if the record cannot be appended.
    pub fn log_skipped(
        &self,
        action: AuditAction,
        target: &str,
        reason: &str,
        details: serde_json::Value,
    ) -> std::io::Result<()> {
        let record = AuditRecord {
            timestamp: chrono::Utc::now().to_rfc3339(),
            action,
            target: target.to_string(),
            result: AuditResult::Skipped,
            reason: Some(reason.to_string()),
            details,
        };
        self.storage.append(&record)
    }

    /// Query audit records with pagination and optional filtering.
    ///
    /// # Errors
    ///
    /// Returns an error if storage cannot be read.
    pub fn query(
        &self,
        offset: usize,
        limit: usize,
        action_type: Option<AuditAction>,
        from: Option<String>,
        to: Option<String>,
    ) -> std::io::Result<(usize, Vec<AuditRecord>)> {
        let result = self.storage.query(&AuditQueryParams {
            offset,
            limit,
            action_type,
            from,
            to,
        })?;
        Ok((result.total_count, result.records))
    }

    /// Generate a five-element failure explanation.
    #[must_use]
    pub fn generate_failure_explanation(
        cause: &str,
        attempted_actions: Vec<String>,
        attempt_count: (u32, u32),
        suggested_action: &str,
        needs_manual_intervention: bool,
    ) -> FailureExplanation {
        FailureExplanation {
            cause: cause.to_string(),
            attempted_actions,
            attempt_count,
            suggested_action: suggested_action.to_string(),
            needs_manual_intervention,
        }
    }
}

/// Thread-safe wrapper for `AuditLogger`.
pub struct SyncAuditLogger {
    inner: Mutex<AuditLogger>,
}

impl SyncAuditLogger {
    /// # Errors
    ///
    /// Returns an error if the storage directory cannot be created.
    pub fn new(audit_dir: PathBuf) -> std::io::Result<Self> {
        Ok(Self {
            inner: Mutex::new(AuditLogger::new(audit_dir)?),
        })
    }

    /// Log a successful action (thread-safe).
    ///
    /// # Errors
    ///
    /// Returns an error if the logger is poisoned or the record cannot be appended.
    pub fn log_success(
        &self,
        action: AuditAction,
        target: &str,
        details: serde_json::Value,
    ) -> std::io::Result<()> {
        self.inner
            .lock()
            .map_err(|e| std::io::Error::other(e.to_string()))?
            .log_success(action, target, details)
    }

    /// Log a failed action (thread-safe).
    ///
    /// # Errors
    ///
    /// Returns an error if the logger is poisoned or the record cannot be appended.
    pub fn log_failure(
        &self,
        action: AuditAction,
        target: &str,
        reason: &str,
        details: serde_json::Value,
    ) -> std::io::Result<()> {
        self.inner
            .lock()
            .map_err(|e| std::io::Error::other(e.to_string()))?
            .log_failure(action, target, reason, details)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn log_success_and_query() {
        let dir = TempDir::new().expect("temp dir");
        let logger = AuditLogger::new(dir.path().to_path_buf()).expect("create");

        logger
            .log_success(
                AuditAction::BaselineCollect,
                "all",
                serde_json::json!({"item_count": 9}),
            )
            .expect("log");

        let (total, records) = logger.query(0, 50, None, None, None).expect("query");
        assert_eq!(total, 1);
        assert_eq!(records[0].action, AuditAction::BaselineCollect);
        assert_eq!(records[0].result, AuditResult::Success);
        assert!(records[0].reason.is_none());
    }

    #[test]
    fn log_failure_with_reason() {
        let dir = TempDir::new().expect("temp dir");
        let logger = AuditLogger::new(dir.path().to_path_buf()).expect("create");

        logger
            .log_failure(
                AuditAction::StateRestore,
                "win-hosts",
                "Permission denied",
                serde_json::json!({"path": "/etc/hosts"}),
            )
            .expect("log");

        let (_, records) = logger.query(0, 50, None, None, None).expect("query");
        assert_eq!(records[0].result, AuditResult::Failure);
        assert_eq!(
            records[0].reason,
            Some("Permission denied".to_string())
        );
    }

    #[test]
    fn log_skipped_action() {
        let dir = TempDir::new().expect("temp dir");
        let logger = AuditLogger::new(dir.path().to_path_buf()).expect("create");

        logger
            .log_skipped(
                AuditAction::StateRestore,
                "win-tun-status",
                "Not restorable",
                serde_json::json!({}),
            )
            .expect("log");

        let (_, records) = logger.query(0, 50, None, None, None).expect("query");
        assert_eq!(records[0].result, AuditResult::Skipped);
    }

    #[test]
    fn query_with_action_filter() {
        let dir = TempDir::new().expect("temp dir");
        let logger = AuditLogger::new(dir.path().to_path_buf()).expect("create");

        logger
            .log_success(
                AuditAction::BaselineCollect,
                "a",
                serde_json::json!({}),
            )
            .expect("log");
        logger
            .log_success(
                AuditAction::StateRestore,
                "b",
                serde_json::json!({}),
            )
            .expect("log");

        let (total, records) = logger
            .query(0, 50, Some(AuditAction::StateRestore), None, None)
            .expect("query");
        assert_eq!(total, 1);
        assert_eq!(records[0].target, "b");
    }

    #[test]
    fn generate_failure_explanation_fields() {
        let explanation = AuditLogger::generate_failure_explanation(
            "Hosts file write permission denied",
            vec![
                "Direct file write".to_string(),
                "Elevated write via runas".to_string(),
            ],
            (3, 3),
            "Run as administrator",
            true,
        );

        assert_eq!(explanation.cause, "Hosts file write permission denied");
        assert_eq!(explanation.attempted_actions.len(), 2);
        assert_eq!(explanation.attempt_count, (3, 3));
        assert!(explanation.needs_manual_intervention);
        assert!(!explanation.suggested_action.is_empty());
    }

    #[test]
    fn sync_logger_thread_safety() {
        use std::sync::Arc;
        use std::thread;

        let dir = TempDir::new().expect("temp dir");
        let logger = Arc::new(
            SyncAuditLogger::new(dir.path().to_path_buf()).expect("create"),
        );

        let handles: Vec<_> = (0..5)
            .map(|i| {
                let l = Arc::clone(&logger);
                thread::spawn(move || {
                    l.log_success(
                        AuditAction::ConfigChange,
                        &format!("item-{i}"),
                        serde_json::json!({"i": i}),
                    )
                    .expect("log");
                })
            })
            .collect();

        for h in handles {
            h.join().expect("thread");
        }

        // Read via underlying storage.
        let storage = AuditStorage::new(dir.path().to_path_buf());
        let result = storage
            .query(&AuditQueryParams::default())
            .expect("query");
        assert_eq!(result.total_count, 5);
    }

    #[test]
    fn mock_audit_log_records_success() {
        let mock = MockAuditLog::new();
        mock.log_success(AuditAction::SiteAdd, "github", serde_json::json!({}))
            .expect("log");

        let records = mock.records();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].action, AuditAction::SiteAdd);
        assert_eq!(records[0].target, "github");
        assert_eq!(records[0].result, AuditResult::Success);
    }

    #[test]
    fn mock_audit_log_records_failure() {
        let mock = MockAuditLog::new();
        mock.log_failure(AuditAction::SiteAdd, "github", "verify failed", serde_json::json!({}))
            .expect("log");

        let records = mock.records();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].result, AuditResult::Failure);
        assert_eq!(records[0].reason, Some("verify failed".to_string()));
    }
}
