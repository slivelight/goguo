use std::fs;
use std::io::{BufRead, Write};
use std::path::PathBuf;

use crate::models::audit::{AuditAction, AuditRecord};

/// Parameters for querying audit records with pagination and optional filtering.
pub struct AuditQueryParams {
    pub offset: usize,
    pub limit: usize,
    pub action_type: Option<AuditAction>,
    pub from: Option<String>,
    pub to: Option<String>,
}

impl Default for AuditQueryParams {
    fn default() -> Self {
        Self {
            offset: 0,
            limit: 50,
            action_type: None,
            from: None,
            to: None,
        }
    }
}

/// Result of an audit log query.
pub struct AuditQueryResult {
    pub total_count: usize,
    pub records: Vec<AuditRecord>,
}

/// Handles persistence of audit records as daily-rotating JSONL files.
///
/// Storage layout: `{base_dir}/audit-YYYY-MM-DD.jsonl`
pub struct AuditStorage {
    base_dir: PathBuf,
}

impl AuditStorage {
    #[must_use]
    pub const fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Append a single audit record to the appropriate daily JSONL file.
    ///
    /// # Errors
    ///
    /// Returns an error if directory creation, file open, or serialization fails.
    pub fn append(&self, record: &AuditRecord) -> std::io::Result<()> {
        fs::create_dir_all(&self.base_dir)?;
        let date = record.timestamp.get(..10).unwrap_or("unknown");
        let filename = format!("audit-{date}.jsonl");
        let path = self.base_dir.join(filename);
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        let line = serde_json::to_string(record)? + "\n";
        file.write_all(line.as_bytes())
    }

    /// Query audit records with pagination and optional filtering.
    ///
    /// # Errors
    ///
    /// Returns an error if directory listing or file reading fails.
    pub fn query(&self, params: &AuditQueryParams) -> std::io::Result<AuditQueryResult> {
        let limit = params.limit.clamp(1, 200);

        if !self.base_dir.exists() {
            return Ok(AuditQueryResult {
                total_count: 0,
                records: vec![],
            });
        }

        // Collect all records matching filters across all daily files.
        let mut all_records: Vec<AuditRecord> = Vec::new();
        for entry in fs::read_dir(&self.base_dir)? {
            let entry = entry?;
            let path = entry.path();
            let Some(ext) = path.extension() else {
                continue;
            };
            if !ext.eq_ignore_ascii_case("jsonl") {
                continue;
            }
            let file = fs::File::open(&path)?;
            let reader = std::io::BufReader::new(file);
            for line in reader.lines() {
                let line = line?;
                if let Ok(record) = serde_json::from_str::<AuditRecord>(&line) {
                    if matches_filter(&record, params) {
                        all_records.push(record);
                    }
                }
            }
        }

        // Sort by timestamp ascending.
        all_records.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        let total_count = all_records.len();
        let records = all_records
            .into_iter()
            .skip(params.offset)
            .take(limit)
            .collect();

        Ok(AuditQueryResult {
            total_count,
            records,
        })
    }
}

fn matches_filter(record: &AuditRecord, params: &AuditQueryParams) -> bool {
    if let Some(ref action) = params.action_type {
        if record.action != *action {
            return false;
        }
    }
    if let Some(ref from) = params.from {
        if &record.timestamp < from {
            return false;
        }
    }
    if let Some(ref to) = params.to {
        if &record.timestamp > to {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::audit::AuditResult;
    use tempfile::TempDir;

    fn make_record(timestamp: &str, action: AuditAction, target: &str) -> AuditRecord {
        AuditRecord {
            timestamp: timestamp.to_string(),
            action,
            target: target.to_string(),
            result: AuditResult::Success,
            reason: None,
            details: serde_json::json!({}),
        }
    }

    #[test]
    fn append_and_read_single_record() {
        let dir = TempDir::new().expect("temp dir");
        let storage = AuditStorage::new(dir.path().to_path_buf());

        let record = make_record("2026-05-19T10:00:00Z", AuditAction::BaselineCollect, "all");
        storage.append(&record).expect("append");

        let result = storage
            .query(&AuditQueryParams::default())
            .expect("query");
        assert_eq!(result.total_count, 1);
        assert_eq!(result.records[0].action, AuditAction::BaselineCollect);
        assert_eq!(result.records[0].target, "all");
    }

    #[test]
    fn append_multiple_records_same_day() {
        let dir = TempDir::new().expect("temp dir");
        let storage = AuditStorage::new(dir.path().to_path_buf());

        for i in 0..5 {
            let record = make_record(
                &format!("2026-05-19T10:0{i}:00Z"),
                AuditAction::StateRestore,
                &format!("item-{i}"),
            );
            storage.append(&record).expect("append");
        }

        let result = storage
            .query(&AuditQueryParams::default())
            .expect("query");
        assert_eq!(result.total_count, 5);
    }

    #[test]
    fn daily_rotation_separates_files() {
        let dir = TempDir::new().expect("temp dir");
        let storage = AuditStorage::new(dir.path().to_path_buf());

        let day1 = make_record("2026-05-18T12:00:00Z", AuditAction::BaselineCollect, "a");
        let day2 = make_record("2026-05-19T12:00:00Z", AuditAction::BaselineConfirm, "b");

        storage.append(&day1).expect("append day1");
        storage.append(&day2).expect("append day2");

        assert!(dir.path().join("audit-2026-05-18.jsonl").exists());
        assert!(dir.path().join("audit-2026-05-19.jsonl").exists());

        let result = storage
            .query(&AuditQueryParams::default())
            .expect("query");
        assert_eq!(result.total_count, 2);
    }

    #[test]
    fn pagination_offset_and_limit() {
        let dir = TempDir::new().expect("temp dir");
        let storage = AuditStorage::new(dir.path().to_path_buf());

        for i in 0..10 {
            let record = make_record(
                &format!("2026-05-19T10:{i:02}:00Z"),
                AuditAction::ConfigChange,
                &format!("item-{i}"),
            );
            storage.append(&record).expect("append");
        }

        let page1 = storage
            .query(&AuditQueryParams {
                offset: 0,
                limit: 3,
                ..Default::default()
            })
            .expect("query");
        assert_eq!(page1.total_count, 10);
        assert_eq!(page1.records.len(), 3);
        assert_eq!(page1.records[0].target, "item-0");

        let page2 = storage
            .query(&AuditQueryParams {
                offset: 3,
                limit: 3,
                ..Default::default()
            })
            .expect("query");
        assert_eq!(page2.records[0].target, "item-3");

        let last = storage
            .query(&AuditQueryParams {
                offset: 9,
                limit: 50,
                ..Default::default()
            })
            .expect("query");
        assert_eq!(last.records.len(), 1);
        assert_eq!(last.records[0].target, "item-9");
    }

    #[test]
    fn filter_by_action_type() {
        let dir = TempDir::new().expect("temp dir");
        let storage = AuditStorage::new(dir.path().to_path_buf());

        storage
            .append(&make_record(
                "2026-05-19T10:00:00Z",
                AuditAction::BaselineCollect,
                "a",
            ))
            .expect("append");
        storage
            .append(&make_record(
                "2026-05-19T10:01:00Z",
                AuditAction::StateRestore,
                "b",
            ))
            .expect("append");
        storage
            .append(&make_record(
                "2026-05-19T10:02:00Z",
                AuditAction::StateRestore,
                "c",
            ))
            .expect("append");

        let result = storage
            .query(&AuditQueryParams {
                action_type: Some(AuditAction::StateRestore),
                ..Default::default()
            })
            .expect("query");
        assert_eq!(result.total_count, 2);
        assert!(result
            .records
            .iter()
            .all(|r| r.action == AuditAction::StateRestore));
    }

    #[test]
    fn filter_by_date_range() {
        let dir = TempDir::new().expect("temp dir");
        let storage = AuditStorage::new(dir.path().to_path_buf());

        storage
            .append(&make_record(
                "2026-05-18T10:00:00Z",
                AuditAction::BaselineCollect,
                "a",
            ))
            .expect("append");
        storage
            .append(&make_record(
                "2026-05-19T10:00:00Z",
                AuditAction::BaselineCollect,
                "b",
            ))
            .expect("append");
        storage
            .append(&make_record(
                "2026-05-20T10:00:00Z",
                AuditAction::BaselineCollect,
                "c",
            ))
            .expect("append");

        let result = storage
            .query(&AuditQueryParams {
                from: Some("2026-05-19T00:00:00Z".to_string()),
                to: Some("2026-05-19T23:59:59Z".to_string()),
                ..Default::default()
            })
            .expect("query");
        assert_eq!(result.total_count, 1);
        assert_eq!(result.records[0].target, "b");
    }

    #[test]
    fn query_empty_dir_returns_empty() {
        let dir = TempDir::new().expect("temp dir");
        let storage = AuditStorage::new(dir.path().to_path_buf());

        let result = storage
            .query(&AuditQueryParams::default())
            .expect("query");
        assert_eq!(result.total_count, 0);
        assert!(result.records.is_empty());
    }

    #[test]
    fn limit_clamped_to_max_200() {
        let dir = TempDir::new().expect("temp dir");
        let storage = AuditStorage::new(dir.path().to_path_buf());

        for i in 0..5 {
            storage
                .append(&make_record(
                    &format!("2026-05-19T10:{i:02}:00Z"),
                    AuditAction::ConfigChange,
                    &format!("item-{i}"),
                ))
                .expect("append");
        }

        let result = storage
            .query(&AuditQueryParams {
                limit: 999,
                ..Default::default()
            })
            .expect("query");
        assert_eq!(result.records.len(), 5);
    }

    #[test]
    fn records_sorted_by_timestamp_ascending() {
        let dir = TempDir::new().expect("temp dir");
        let storage = AuditStorage::new(dir.path().to_path_buf());

        storage
            .append(&make_record(
                "2026-05-19T10:02:00Z",
                AuditAction::BaselineCollect,
                "c",
            ))
            .expect("append");
        storage
            .append(&make_record(
                "2026-05-19T10:00:00Z",
                AuditAction::BaselineCollect,
                "a",
            ))
            .expect("append");
        storage
            .append(&make_record(
                "2026-05-19T10:01:00Z",
                AuditAction::BaselineCollect,
                "b",
            ))
            .expect("append");

        let result = storage
            .query(&AuditQueryParams::default())
            .expect("query");
        assert_eq!(result.records[0].target, "a");
        assert_eq!(result.records[1].target, "b");
        assert_eq!(result.records[2].target, "c");
    }

    #[test]
    fn concurrent_appends_no_data_loss() {
        use std::sync::Arc;
        use std::thread;

        let dir = TempDir::new().expect("temp dir");
        let storage = Arc::new(AuditStorage::new(dir.path().to_path_buf()));

        let handles: Vec<_> = (0..10)
            .map(|i| {
                let s = Arc::clone(&storage);
                thread::spawn(move || {
                    let record = make_record(
                        &format!("2026-05-19T10:{i:02}:00Z"),
                        AuditAction::ConfigChange,
                        &format!("thread-{i}"),
                    );
                    s.append(&record).expect("append");
                })
            })
            .collect();

        for h in handles {
            h.join().expect("thread");
        }

        let result = storage
            .query(&AuditQueryParams::default())
            .expect("query");
        assert_eq!(
            result.total_count, 10,
            "all 10 concurrent records must survive"
        );
    }
}
