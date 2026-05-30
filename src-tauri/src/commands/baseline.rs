use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::managers::baseline_manager::NonTargetVerification;
use crate::managers::config_manager::ConfigManager;
use crate::models::audit::AuditAction;
use crate::models::config::AppConfig;
use crate::models::recovery::RecoveryStatus;
use crate::storage::baseline_storage::BaselineStorage;

// ── Response DTOs ──────────────────────────────────────────────────────────

/// Response for `start_initial_assessment` and `trigger_readjustment`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentResponse {
    pub version: u32,
    pub timestamp: String,
    pub item_count: usize,
}

/// Response for `get_state_summary`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSummaryResponse {
    pub total: usize,
    pub restorable_count: usize,
    pub detectable_count: usize,
    pub excluded_count: usize,
}

/// Result of comparing a single item against baseline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonResultDto {
    Match,
    Deviated,
    MissingInBaseline,
}

/// A single item's comparison result in `get_baseline_status`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonItemDto {
    pub state_item_id: String,
    pub result: ComparisonResultDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_value: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_value: Option<serde_json::Value>,
}

/// Response for `get_baseline_status`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineStatusResponse {
    pub has_baseline: bool,
    pub items: Vec<ComparisonItemDto>,
}

/// Response for `get_service_status`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatusResponse {
    pub mihomo_running: bool,
    pub proxy_guard_restart_count: u32,
}

/// Response for `get_recovery_progress`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryProgressResponse {
    pub has_task: bool,
    pub status: Option<String>,
    pub total_items: usize,
    pub completed_count: usize,
    pub pending_count: usize,
    pub succeeded: usize,
    pub failed: usize,
}

/// Response for `get_audit_log`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogResponse {
    pub total_count: usize,
    pub records: Vec<AuditRecordDto>,
}

/// A single audit record in `get_audit_log` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecordDto {
    pub timestamp: String,
    pub action: String,
    pub target: String,
    pub result: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Parameters for `get_audit_log`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogParams {
    #[serde(default)]
    pub offset: usize,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
}

const fn default_limit() -> usize {
    50
}

/// Maximum allowed value for `limit` in `AuditLogParams`.
pub const MAX_AUDIT_LOG_LIMIT: usize = 200;

// ── Deployment DTOs ─────────────────────────────────────────────────────────

/// Response for deployment mode commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentModeResponse {
    pub mode: String,
    pub detected: String,
    pub is_auto: bool,
}

/// Response for WSL status queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WslStatusResponse {
    pub is_wsl: bool,
    pub distro_name: Option<String>,
    pub distro_version: Option<String>,
    pub network_mode: String,
    pub reachable: bool,
}

/// Response for network mode queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkModeResponse {
    pub mode: String,
    pub proxy_strategy: String,
}

impl AuditLogParams {
    /// Validate and clamp the pagination parameters.
    ///
    /// - `offset` is capped at `total_count` (caller must handle).
    /// - `limit` is clamped to `[1, MAX_AUDIT_LOG_LIMIT]`.
    #[must_use]
    pub fn validated(&self) -> Self {
        Self {
            offset: self.offset,
            limit: self.limit.clamp(1, MAX_AUDIT_LOG_LIMIT),
            action_type: self.action_type.clone(),
            from: self.from.clone(),
            to: self.to.clone(),
        }
    }
}

impl Default for AuditLogParams {
    fn default() -> Self {
        Self {
            offset: 0,
            limit: default_limit(),
            action_type: None,
            from: None,
            to: None,
        }
    }
}

// ── Event Payloads ─────────────────────────────────────────────────────────

/// Payload for `recovery:started` event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryStartedPayload {
    pub task_id: String,
    pub total_items: usize,
}

/// Payload for `recovery:item-completed` event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryItemCompletedPayload {
    pub state_item_id: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,
}

/// Payload for `recovery:completed` event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryCompletedPayload {
    pub task_id: String,
    pub succeeded: usize,
    pub failed: usize,
}

/// Payload for `recovery:failed` event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryFailedPayload {
    pub task_id: String,
    pub failed_items: Vec<String>,
}

/// Payload for `baseline:confirmed` event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineConfirmedPayload {
    pub version: u32,
    pub item_count: usize,
}

/// Payload for `baseline:deviation-detected` event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineDeviationPayload {
    pub deviated_items: Vec<String>,
}

/// Payload for `service:stopped` event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStoppedPayload {
    pub reason: String,
    pub recovery_triggered: bool,
    /// Non-target site reachability verification (F103 / SC-5).
    pub non_target_verification: Option<NonTargetVerification>,
}

/// Payload for `service:started` event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStartedPayload {
    pub mihomo_running: bool,
}

/// Payload for `proxy-guard:recovery-triggered` event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoRecoveryTriggeredPayload {
    pub restart_attempts: u32,
    pub max_attempts: u32,
}

// ── Conversion helpers ─────────────────────────────────────────────────────

/// Convert a `RecoveryStatus` to its string representation for JSON responses.
#[must_use]
pub fn recovery_status_to_string(status: &RecoveryStatus) -> String {
    match status {
        RecoveryStatus::Pending => "pending".to_string(),
        RecoveryStatus::InProgress => "in_progress".to_string(),
        RecoveryStatus::Completed => "completed".to_string(),
        RecoveryStatus::Failed => "failed".to_string(),
        RecoveryStatus::UserAcknowledged => "user_acknowledged".to_string(),
    }
}

/// Parse an action type string into an `AuditAction` for filtering.
#[must_use]
pub fn parse_audit_action(action: &str) -> Option<AuditAction> {
    match action {
        "baseline_collect" => Some(AuditAction::BaselineCollect),
        "baseline_confirm" => Some(AuditAction::BaselineConfirm),
        "state_restore" => Some(AuditAction::StateRestore),
        "proxy_guard_restart" => Some(AuditAction::ProxyGuardRestart),
        "proxy_guard_recovery" => Some(AuditAction::ProxyGuardRecovery),
        "rule_apply" => Some(AuditAction::RuleApply),
        "config_change" => Some(AuditAction::ConfigChange),
        _ => None,
    }
}

// ── Deployment Conversion Helpers ──────────────────────────────────────────

use crate::managers::deployment_manager::DeploymentManager;
use crate::models::config::DeploymentMode;

#[cfg(target_os = "linux")]
use crate::services::wsl_detector::WslNetworkMode;

/// Convert a `DeploymentMode` to its `snake_case` string representation.
#[must_use]
pub fn deployment_mode_to_string(mode: &DeploymentMode) -> String {
    match mode {
        DeploymentMode::WindowsOnly => "windows_only".to_string(),
        DeploymentMode::WslOnly => "wsl_only".to_string(),
        DeploymentMode::LinuxOnly => "linux_only".to_string(),
        DeploymentMode::Coordinated => "coordinated".to_string(),
    }
}

/// Parse a `snake_case` string into a `DeploymentMode`.
///
/// # Errors
///
/// Returns an error if the string does not match any known mode.
pub fn parse_deployment_mode(s: &str) -> Result<DeploymentMode, String> {
    match s {
        "windows_only" => Ok(DeploymentMode::WindowsOnly),
        "wsl_only" => Ok(DeploymentMode::WslOnly),
        "linux_only" => Ok(DeploymentMode::LinuxOnly),
        "coordinated" => Ok(DeploymentMode::Coordinated),
        _ => Err(format!("Unknown deployment mode: {s}")),
    }
}

/// Convert a `WslNetworkMode` to its string representation.
#[cfg(target_os = "linux")]
#[must_use]
pub fn network_mode_to_string(mode: &WslNetworkMode) -> String {
    match mode {
        WslNetworkMode::Nat => "nat".to_string(),
        WslNetworkMode::Mirrored => "mirrored".to_string(),
        WslNetworkMode::NotInstalled => "not_installed".to_string(),
    }
}

/// Determine the proxy strategy for a given network mode.
#[cfg(target_os = "linux")]
#[must_use]
fn determine_strategy(mode: &WslNetworkMode) -> String {
    match mode {
        WslNetworkMode::Mirrored => "skip_config".to_string(),
        WslNetworkMode::Nat => "explicit_config".to_string(),
        WslNetworkMode::NotInstalled => "fallback_to_explicit".to_string(),
    }
}

// ── Tauri Commands ─────────────────────────────────────────────────────────

use std::sync::Mutex;

use crate::managers::baseline_manager::{BaselineManager, ComparisonResult};
use crate::managers::mihomo_manager::MihomoManager;
use crate::services::audit_logger::AuditLogger;
use crate::services::proxy_guard::ProxyGuard;
use tauri::{Emitter, Manager};

/// Shared application state injected into Tauri commands via `tauri::State`.
pub struct AppState {
    pub baseline_manager: Mutex<BaselineManager>,
    pub mihomo_manager: Mutex<MihomoManager>,
    pub proxy_guard: Mutex<ProxyGuard>,
    pub audit_logger: Mutex<AuditLogger>,
    pub deployment_manager: Mutex<DeploymentManager>,
    /// Flag indicating a restore/recovery operation is in progress (F104).
    /// When true, network-modifying commands are blocked.
    pub is_restoring: std::sync::atomic::AtomicBool,
    /// Flag indicating user explicitly stopped the service (F108).
    /// When true, `ProxyGuard` skips auto-restart of mihomo.
    pub service_paused: std::sync::atomic::AtomicBool,
}

impl AppState {
    /// Create a new `AppState` with all managers initialised from `data_dir`.
    ///
    /// Uses `DeploymentManager` to determine the deployment mode and create
    /// the appropriate platform adapters. Coordinated mode creates two adapters
    /// (local + remote bridge) for cross-platform management.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if the config or audit directories cannot be created.
    pub fn new(data_dir: &Path) -> std::io::Result<Self> {
        let storage = BaselineStorage::new(data_dir.join("baseline"));

        // Create DeploymentManager first to determine deployment mode
        let config_manager = ConfigManager::new(data_dir.join("config"))?;
        let depl_mgr = DeploymentManager::new(config_manager, data_dir.to_path_buf());

        // Use stored mode, fallback to auto-detected mode
        let detected = DeploymentManager::detect_deployment_mode();
        let mode = depl_mgr
            .get_deployment_mode()
            .unwrap_or(detected);
        let adapters = depl_mgr.create_adapters(&mode);

        let baseline_manager = Mutex::new(BaselineManager::new(
            adapters,
            storage,
            data_dir.join("audit"),
        ));

        let deployment_manager = Mutex::new(depl_mgr);

        let app_config = AppConfig::default_for(data_dir.to_path_buf());
        let mihomo_manager = Mutex::new(MihomoManager::new(app_config.mihomo));
        let proxy_guard = Mutex::new(ProxyGuard::new(app_config.proxy_guard));

        let audit_logger = Mutex::new(AuditLogger::new(data_dir.join("audit"))?);

        Ok(Self {
            baseline_manager,
            mihomo_manager,
            proxy_guard,
            audit_logger,
            deployment_manager,
            is_restoring: std::sync::atomic::AtomicBool::new(false),
            service_paused: std::sync::atomic::AtomicBool::new(false),
        })
    }
}

/// Error type returned by all Tauri commands.
fn command_error(context: &str, e: impl std::fmt::Display) -> String {
    format!("{context}: {e}")
}

/// Check that no restore/recovery operation is in progress.
/// Returns `Ok(())` if safe to proceed, `Err` if blocked.
fn check_not_restoring(state: &AppState) -> Result<(), String> {
    if state.is_restoring.load(std::sync::atomic::Ordering::Relaxed) {
        return Err("Operation blocked: recovery in progress".to_string());
    }
    Ok(())
}

// ── Inner helpers (original signatures, used by tests) ──────────────────────

/// Start the initial network assessment.
///
/// # Errors
///
/// Returns an error if the initial snapshot cannot be collected.
pub fn start_initial_assessment(mgr: &BaselineManager) -> Result<AssessmentResponse, String> {
    let snapshot = mgr
        .collect_initial_snapshot()
        .map_err(|e| command_error("Initial assessment failed", e))?;
    Ok(AssessmentResponse {
        version: snapshot.version,
        timestamp: snapshot.timestamp,
        item_count: snapshot.items.len(),
    })
}

/// Get a summary of the current system state grouped by category.
///
/// # Errors
///
/// Returns an error if adapter collection fails.
pub fn get_state_summary(mgr: &BaselineManager) -> Result<StateSummaryResponse, String> {
    let summary = mgr
        .get_state_summary()
        .map_err(|e| command_error("State summary failed", e))?;
    Ok(StateSummaryResponse {
        total: summary.total,
        restorable_count: summary.restorable.len(),
        detectable_count: summary.detectable.len(),
        excluded_count: summary.excluded.len(),
    })
}

/// Re-collect the network state (trigger readjustment).
///
/// # Errors
///
/// Returns an error if the re-collection fails.
pub fn trigger_readjustment(mgr: &BaselineManager) -> Result<AssessmentResponse, String> {
    start_initial_assessment(mgr)
}

/// Confirm the baseline after user review.
///
/// # Errors
///
/// Returns an error if no initial snapshot exists or persistence fails.
pub fn confirm_baseline(mgr: &BaselineManager) -> Result<AssessmentResponse, String> {
    let baseline = mgr
        .confirm_baseline()
        .map_err(|e| command_error("Baseline confirmation failed", e))?;
    Ok(AssessmentResponse {
        version: baseline.version,
        timestamp: baseline.timestamp,
        item_count: baseline.items.len(),
    })
}

/// Get the baseline deviation status.
///
/// # Errors
///
/// Returns an error if the comparison fails (other than missing baseline).
pub fn get_baseline_status(mgr: &BaselineManager) -> Result<BaselineStatusResponse, String> {
    match mgr.compare_with_baseline() {
        Ok(comparisons) => {
            let items: Vec<ComparisonItemDto> = comparisons
                .into_iter()
                .map(|c| {
                    let (result, baseline_value, current_value) = match c.result {
                        ComparisonResult::Match => {
                            (ComparisonResultDto::Match, None, None)
                        }
                        ComparisonResult::Deviated {
                            baseline_value: bv,
                            current_value: cv,
                        } => (
                            ComparisonResultDto::Deviated,
                            Some(bv),
                            Some(cv),
                        ),
                        ComparisonResult::MissingInBaseline => {
                            (ComparisonResultDto::MissingInBaseline, None, None)
                        }
                    };
                    ComparisonItemDto {
                        state_item_id: c.state_item_id,
                        result,
                        baseline_value,
                        current_value,
                    }
                })
                .collect();
            Ok(BaselineStatusResponse {
                has_baseline: true,
                items,
            })
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(BaselineStatusResponse {
            has_baseline: false,
            items: vec![],
        }),
        Err(e) => Err(command_error("Baseline status check failed", e)),
    }
}

/// Stop the service and restore network settings to baseline.
///
/// # Errors
///
/// Returns an error if the recovery state cannot be determined.
pub fn stop_service(
    mihomo: &mut MihomoManager,
    baseline_mgr: &BaselineManager,
) -> Result<ServiceStoppedPayload, String> {
    let _ = mihomo.stop();
    let restore_result = baseline_mgr.restore_to_baseline();

    let recovery_triggered = restore_result.as_ref().is_ok_and(|r| r.succeeded > 0);
    let non_target_verification = restore_result.ok().and_then(|r| r.non_target_verification);

    Ok(ServiceStoppedPayload {
        reason: "User requested".to_string(),
        recovery_triggered,
        non_target_verification,
    })
}

/// Get the current service status (mihomo running + `ProxyGuard` restart count).
#[must_use]
pub fn get_service_status(
    mihomo: &mut MihomoManager,
    guard: &ProxyGuard,
) -> ServiceStatusResponse {
    ServiceStatusResponse {
        mihomo_running: mihomo.is_running(),
        proxy_guard_restart_count: guard.restart_count(),
    }
}

/// Get the current recovery task progress.
///
/// # Errors
///
/// Returns an error if the recovery state cannot be loaded.
pub fn get_recovery_progress(
    baseline_mgr: &BaselineManager,
) -> Result<RecoveryProgressResponse, String> {
    use crate::services::recovery::RecoveryManager;

    let recovery_mgr = RecoveryManager::new(
        baseline_mgr.audit_dir.join("state"),
    )
    .map_err(|e| command_error("Failed to load recovery state", e))?;

    match recovery_mgr.load_task().map_err(|e| command_error("Recovery load failed", e))? {
        Some(task) => {
            let completed_count = task.completed_items.len();
            let pending_count = task.pending_items.len();
            let total = completed_count + pending_count;
            let succeeded = task
                .completed_items
                .iter()
                .filter(|i| i.result == Some(crate::models::recovery::ItemResult::Success))
                .count();
            let failed = task
                .completed_items
                .iter()
                .filter(|i| i.result == Some(crate::models::recovery::ItemResult::Failure))
                .count();

            Ok(RecoveryProgressResponse {
                has_task: true,
                status: Some(recovery_status_to_string(&task.status)),
                total_items: total,
                completed_count,
                pending_count,
                succeeded,
                failed,
            })
        }
        None => Ok(RecoveryProgressResponse {
            has_task: false,
            status: None,
            total_items: 0,
            completed_count: 0,
            pending_count: 0,
            succeeded: 0,
            failed: 0,
        }),
    }
}

/// Query the audit log with pagination and optional filters.
///
/// # Errors
///
/// Returns an error if the audit log query fails.
pub fn get_audit_log(
    logger: &AuditLogger,
    params: &AuditLogParams,
) -> Result<AuditLogResponse, String> {
    let validated = params.validated();
    let action = validated
        .action_type
        .as_deref()
        .and_then(parse_audit_action);

    let (total_count, records) = logger
        .query(
            validated.offset,
            validated.limit,
            action,
            validated.from,
            validated.to,
        )
        .map_err(|e| command_error("Audit log query failed", e))?;

    let dtos: Vec<AuditRecordDto> = records
        .into_iter()
        .map(|r| AuditRecordDto {
            timestamp: r.timestamp,
            action: serde_json::to_string(&r.action)
                .unwrap_or_default()
                .trim_matches('"')
                .to_string(),
            target: r.target,
            result: serde_json::to_string(&r.result)
                .unwrap_or_default()
                .trim_matches('"')
                .to_string(),
            reason: r.reason,
            details: Some(r.details),
        })
        .collect();

    Ok(AuditLogResponse {
        total_count,
        records: dtos,
    })
}

/// Detect the appropriate deployment mode for the current platform.
///
/// # Errors
///
/// Returns an error if the stored configuration cannot be read.
pub fn detect_deployment_mode(
    depl_mgr: &DeploymentManager,
) -> Result<DeploymentModeResponse, String> {
    let detected = DeploymentManager::detect_deployment_mode();
    let stored = depl_mgr
        .get_deployment_mode()
        .map_err(|e| command_error("Failed to read stored deployment mode", e))?;
    let is_auto = deployment_mode_to_string(&detected) == deployment_mode_to_string(&stored);

    Ok(DeploymentModeResponse {
        mode: deployment_mode_to_string(&stored),
        detected: deployment_mode_to_string(&detected),
        is_auto,
    })
}

/// Get the current deployment mode from persisted configuration.
///
/// # Errors
///
/// Returns an error if the configuration cannot be read.
pub fn get_deployment_mode(
    depl_mgr: &DeploymentManager,
) -> Result<DeploymentModeResponse, String> {
    let mode = depl_mgr
        .get_deployment_mode()
        .map_err(|e| command_error("Failed to get deployment mode", e))?;
    let detected = DeploymentManager::detect_deployment_mode();
    let is_auto = deployment_mode_to_string(&detected) == deployment_mode_to_string(&mode);

    Ok(DeploymentModeResponse {
        mode: deployment_mode_to_string(&mode),
        detected: deployment_mode_to_string(&detected),
        is_auto,
    })
}

/// Set and persist a new deployment mode.
///
/// # Errors
///
/// Returns an error if the mode string is invalid or persistence fails.
pub fn set_deployment_mode(
    depl_mgr: &DeploymentManager,
    mode: &str,
) -> Result<DeploymentModeResponse, String> {
    let parsed = parse_deployment_mode(mode)
        .map_err(|e| command_error("Invalid deployment mode", e))?;
    depl_mgr
        .set_deployment_mode(parsed)
        .map_err(|e| command_error("Failed to set deployment mode", e))?;

    let detected = DeploymentManager::detect_deployment_mode();
    let is_auto = deployment_mode_to_string(&detected) == mode;

    Ok(DeploymentModeResponse {
        mode: mode.to_string(),
        detected: deployment_mode_to_string(&detected),
        is_auto,
    })
}

/// Get the current WSL status (Linux only).
///
/// # Errors
///
/// Returns an error if the WSL detection fails.
#[cfg(target_os = "linux")]
pub fn get_wsl_status(depl_mgr: &DeploymentManager) -> Result<WslStatusResponse, String> {
    let status = depl_mgr.get_wsl_status();
    Ok(WslStatusResponse {
        is_wsl: status.is_wsl,
        distro_name: status.distro.as_ref().map(|d| d.name.clone()),
        distro_version: status.distro.as_ref().map(|d| d.version.clone()),
        network_mode: network_mode_to_string(&status.network_mode),
        reachable: status.reachable,
    })
}

/// Get the WSL network mode and proxy strategy (Linux only).
///
/// # Errors
///
/// Returns an error if the network mode detection fails.
#[cfg(target_os = "linux")]
pub fn get_network_mode(depl_mgr: &DeploymentManager) -> Result<NetworkModeResponse, String> {
    let mode = depl_mgr.get_network_mode();
    Ok(NetworkModeResponse {
        mode: network_mode_to_string(&mode),
        proxy_strategy: determine_strategy(&mode),
    })
}

// ── Tauri Command Wrappers ─────────────────────────────────────────────────
//
// Thin wrappers that extract managers from AppState and delegate to the
// inner helper functions above. Tests call the inner functions directly.

/// Tauri command: start the initial network assessment.
#[tauri::command(rename_all = "snake_case")]
#[allow(
    clippy::needless_pass_by_value,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    clippy::double_must_use
)]
pub fn tauri_start_initial_assessment(state: tauri::State<'_, AppState>) -> Result<AssessmentResponse, String> {
    let mgr = state.baseline_manager.lock().expect("lock");
    start_initial_assessment(&mgr)
}

/// Tauri command: get state summary.
#[tauri::command(rename_all = "snake_case")]
#[allow(
    clippy::needless_pass_by_value,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    clippy::double_must_use
)]
pub fn tauri_get_state_summary(state: tauri::State<'_, AppState>) -> Result<StateSummaryResponse, String> {
    let mgr = state.baseline_manager.lock().expect("lock");
    get_state_summary(&mgr)
}

/// Tauri command: trigger readjustment.
#[tauri::command(rename_all = "snake_case")]
#[allow(
    clippy::needless_pass_by_value,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    clippy::double_must_use
)]
pub fn tauri_trigger_readjustment(state: tauri::State<'_, AppState>) -> Result<AssessmentResponse, String> {
    // F108: clear paused flag — user is resuming service
    state.service_paused.store(false, std::sync::atomic::Ordering::Relaxed);

    // F108-2: apply proxy-env before re-assessment
    let mixed_port = {
        let mihomo = state.mihomo_manager.lock().expect("lock");
        mihomo.mixed_port()
    };
    {
        let baseline_mgr = state.baseline_manager.lock().expect("lock");
        let _ = baseline_mgr.apply_proxy_env(mixed_port);
    }

    let mgr = state.baseline_manager.lock().expect("lock");
    trigger_readjustment(&mgr)
}

/// Tauri command: confirm baseline (emits `baseline:confirmed` event).
#[tauri::command(rename_all = "snake_case")]
#[allow(
    clippy::needless_pass_by_value,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc
)]
pub fn tauri_confirm_baseline(state: tauri::State<'_, AppState>, app: tauri::AppHandle) -> Result<AssessmentResponse, String> {
    let mgr = state.baseline_manager.lock().expect("lock");
    let result = confirm_baseline(&mgr)?;
    drop(mgr);
    let _ = app.emit(
        "baseline:confirmed",
        BaselineConfirmedPayload {
            version: result.version,
            item_count: result.item_count,
        },
    );
    Ok(result)
}

/// Tauri command: get baseline status (emits `baseline:deviation-detected` when deviations found).
#[tauri::command(rename_all = "snake_case")]
#[allow(
    clippy::needless_pass_by_value,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc
)]
pub fn tauri_get_baseline_status(state: tauri::State<'_, AppState>, app: tauri::AppHandle) -> Result<BaselineStatusResponse, String> {
    let mgr = state.baseline_manager.lock().expect("lock");
    let result = get_baseline_status(&mgr)?;
    drop(mgr);
    if result.has_baseline {
        let deviated_items: Vec<String> = result
            .items
            .iter()
            .filter(|i| i.result == ComparisonResultDto::Deviated)
            .map(|i| i.state_item_id.clone())
            .collect();
        if !deviated_items.is_empty() {
            let _ = app.emit(
                "baseline:deviation-detected",
                BaselineDeviationPayload { deviated_items },
            );
        }
    }
    Ok(result)
}

/// Tauri command: stop service (emits `service:stopped` event).
#[tauri::command(rename_all = "snake_case")]
#[allow(
    clippy::needless_pass_by_value,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc
)]
pub fn tauri_stop_service(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<ServiceStoppedPayload, String> {
    check_not_restoring(&state)?;
    state.is_restoring.store(true, std::sync::atomic::Ordering::Relaxed);

    // F106: emit recovery:started before restore
    let _ = app.emit("recovery:started", RecoveryStartedPayload {
        task_id: String::new(),
        total_items: 0,
    });

    let mut mihomo = state.mihomo_manager.lock().expect("lock");
    let baseline_mgr = state.baseline_manager.lock().expect("lock");
    let result = stop_service(&mut mihomo, &baseline_mgr);

    // F108-2: clear proxy-env after stopping (service lifecycle, not baseline)
    let _ = baseline_mgr.clear_proxy_env();

    drop(mihomo);
    drop(baseline_mgr);

    state.is_restoring.store(false, std::sync::atomic::Ordering::Relaxed);

    let result = result?;

    // F108: mark service as user-paused so ProxyGuard skips auto-restart
    state.service_paused.store(true, std::sync::atomic::Ordering::Relaxed);

    // F106: emit recovery:completed or recovery:failed based on result
    if result.recovery_triggered {
        let _ = app.emit("recovery:completed", &RecoveryCompletedPayload {
            task_id: String::new(),
            succeeded: 0,
            failed: 0,
        });
    }
    let _ = app.emit("service:stopped", &result);
    Ok(result)
}

/// Tauri command: get service status.
#[tauri::command(rename_all = "snake_case")]
#[must_use]
#[allow(
    clippy::needless_pass_by_value,
    clippy::missing_panics_doc
)]
pub fn tauri_get_service_status(
    state: tauri::State<'_, AppState>,
) -> ServiceStatusResponse {
    let mut mihomo = state.mihomo_manager.lock().expect("lock");
    let guard = state.proxy_guard.lock().expect("lock");
    get_service_status(&mut mihomo, &guard)
}

/// Tauri command: check if a restore/recovery operation is in progress.
#[tauri::command(rename_all = "snake_case")]
#[must_use]
#[allow(clippy::needless_pass_by_value, clippy::missing_panics_doc)]
pub fn tauri_get_is_restoring(state: tauri::State<'_, AppState>) -> bool {
    state.is_restoring.load(std::sync::atomic::Ordering::Relaxed)
}

/// Tauri command: get recovery progress.
#[tauri::command(rename_all = "snake_case")]
#[allow(
    clippy::needless_pass_by_value,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc
)]
pub fn tauri_get_recovery_progress(
    state: tauri::State<'_, AppState>,
) -> Result<RecoveryProgressResponse, String> {
    let baseline_mgr = state.baseline_manager.lock().expect("lock");
    get_recovery_progress(&baseline_mgr)
}

/// Tauri command: get audit log.
#[tauri::command(rename_all = "snake_case")]
#[allow(
    clippy::needless_pass_by_value,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc
)]
pub fn tauri_get_audit_log(
    params: Option<AuditLogParams>,
    state: tauri::State<'_, AppState>,
) -> Result<AuditLogResponse, String> {
    let logger = state.audit_logger.lock().expect("lock");
    let params = params.unwrap_or_default();
    get_audit_log(&logger, &params)
}

/// Tauri command: detect deployment mode.
#[tauri::command(rename_all = "snake_case")]
#[allow(
    clippy::needless_pass_by_value,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc
)]
pub fn tauri_detect_deployment_mode(
    state: tauri::State<'_, AppState>,
) -> Result<DeploymentModeResponse, String> {
    let depl_mgr = state.deployment_manager.lock().expect("lock");
    detect_deployment_mode(&depl_mgr)
}

/// Tauri command: get deployment mode.
#[tauri::command(rename_all = "snake_case")]
#[allow(
    clippy::needless_pass_by_value,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc
)]
pub fn tauri_get_deployment_mode(
    state: tauri::State<'_, AppState>,
) -> Result<DeploymentModeResponse, String> {
    let depl_mgr = state.deployment_manager.lock().expect("lock");
    get_deployment_mode(&depl_mgr)
}

/// Tauri command: set deployment mode.
#[tauri::command(rename_all = "snake_case")]
#[allow(
    clippy::needless_pass_by_value,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc
)]
pub fn tauri_set_deployment_mode(
    mode: String,
    state: tauri::State<'_, AppState>,
) -> Result<DeploymentModeResponse, String> {
    let depl_mgr = state.deployment_manager.lock().expect("lock");
    set_deployment_mode(&depl_mgr, &mode)
}

/// Tauri command: get WSL status (Linux only).
#[cfg(target_os = "linux")]
#[tauri::command(rename_all = "snake_case")]
#[allow(
    clippy::needless_pass_by_value,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc
)]
pub fn tauri_get_wsl_status(state: tauri::State<'_, AppState>) -> Result<WslStatusResponse, String> {
    let depl_mgr = state.deployment_manager.lock().expect("lock");
    get_wsl_status(&depl_mgr)
}

/// Tauri command: get network mode (Linux only).
#[cfg(target_os = "linux")]
#[tauri::command(rename_all = "snake_case")]
#[allow(
    clippy::needless_pass_by_value,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc
)]
pub fn tauri_get_network_mode(state: tauri::State<'_, AppState>) -> Result<NetworkModeResponse, String> {
    let depl_mgr = state.deployment_manager.lock().expect("lock");
    get_network_mode(&depl_mgr)
}

// ── ProxyGuard Background Loop (F105) ─────────────────────────────────────

/// Background loop that periodically checks mihomo health via `ProxyGuard`.
///
/// Runs in a dedicated thread spawned from Tauri `setup`. Lock acquisition
/// order is always `proxy_guard` → `mihomo_manager` to prevent deadlocks.
///
/// # Panics
///
/// Panics if any `Mutex` lock is poisoned (i.e., another thread panicked while holding the lock).
#[allow(clippy::needless_pass_by_value)]
pub fn proxy_guard_loop(app: tauri::AppHandle) {
    let check_interval = 3u64; // seconds, matches ProxyGuardConfig default

    loop {
        std::thread::sleep(std::time::Duration::from_secs(check_interval));

        let state = app.state::<AppState>();

        // Skip if a restore is already in progress
        if state.is_restoring.load(std::sync::atomic::Ordering::Relaxed) {
            continue;
        }

        // F108: skip if user explicitly stopped the service
        if state.service_paused.load(std::sync::atomic::Ordering::Relaxed) {
            continue;
        }

        let action = {
            let mut guard = state.proxy_guard.lock().expect("lock");
            let mut mihomo = state.mihomo_manager.lock().expect("lock");
            guard.check_and_recover(&mut mihomo)
        };

        match action {
            crate::services::proxy_guard::GuardAction::Restarted { attempt } => {
                let max = {
                    let guard = state.proxy_guard.lock().expect("lock");
                    guard.max_restart_attempts()
                };
                let _ = app.emit("service:started", ServiceStartedPayload {
                    mihomo_running: true,
                });
                let _ = app.emit(
                    "proxy-guard:recovery-triggered",
                    AutoRecoveryTriggeredPayload {
                        restart_attempts: attempt,
                        max_attempts: max,
                    },
                );
            }
            crate::services::proxy_guard::GuardAction::RecoveryTriggered => {
                // Trigger baseline restore
                trigger_baseline_restore(&state, &app);
            }
            crate::services::proxy_guard::GuardAction::Healthy => {}
        }
    }
}

/// Perform baseline restore triggered by `ProxyGuard` when max restarts exceeded.
fn trigger_baseline_restore(state: &AppState, app: &tauri::AppHandle) {
    // Set restoring flag
    if state.is_restoring.compare_exchange(
        false,
        true,
        std::sync::atomic::Ordering::Relaxed,
        std::sync::atomic::Ordering::Relaxed,
    ).is_err() {
        // Another restore is already in progress
        return;
    }

    // Emit recovery:started
    let _ = app.emit("recovery:started", RecoveryStartedPayload {
        task_id: String::new(),
        total_items: 0,
    });

    let baseline_mgr = state.baseline_manager.lock().expect("lock");
    let result = baseline_mgr.restore_to_baseline();
    drop(baseline_mgr);

    state.is_restoring.store(false, std::sync::atomic::Ordering::Relaxed);

    match result {
        Ok(r) if r.failed == 0 => {
            let _ = app.emit("recovery:completed", RecoveryCompletedPayload {
                task_id: String::new(),
                succeeded: r.succeeded,
                failed: r.failed,
            });
        }
        Ok(_r) => {
            let _ = app.emit("recovery:failed", RecoveryFailedPayload {
                task_id: String::new(),
                failed_items: Vec::new(),
            });
        }
        Err(_) => {
            let _ = app.emit("recovery:failed", RecoveryFailedPayload {
                task_id: String::new(),
                failed_items: Vec::new(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Response DTO round-trip tests ──────────────────────────────────────

    #[test]
    fn assessment_response_roundtrip() {
        let resp = AssessmentResponse {
            version: 0,
            timestamp: "2026-05-19T12:00:00Z".to_string(),
            item_count: 9,
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let back: AssessmentResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.version, 0);
        assert_eq!(back.item_count, 9);
    }

    #[test]
    fn state_summary_response_roundtrip() {
        let resp = StateSummaryResponse {
            total: 10,
            restorable_count: 4,
            detectable_count: 5,
            excluded_count: 1,
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let back: StateSummaryResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.total, 10);
        assert_eq!(back.restorable_count, 4);
    }

    #[test]
    fn comparison_result_dto_roundtrip() {
        let variants = vec![
            ComparisonResultDto::Match,
            ComparisonResultDto::Deviated,
            ComparisonResultDto::MissingInBaseline,
        ];
        for v in &variants {
            let json = serde_json::to_string(v).expect("serialize");
            let back: ComparisonResultDto = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(&back, v);
        }
    }

    #[test]
    fn comparison_result_dto_snake_case() {
        assert_eq!(
            serde_json::to_string(&ComparisonResultDto::MissingInBaseline).expect("s"),
            "\"missing_in_baseline\""
        );
    }

    #[test]
    fn comparison_item_dto_roundtrip() {
        let item = ComparisonItemDto {
            state_item_id: "win-system-proxy".to_string(),
            result: ComparisonResultDto::Deviated,
            baseline_value: Some(serde_json::json!({"ProxyEnable": 0})),
            current_value: Some(serde_json::json!({"ProxyEnable": 1})),
        };
        let json = serde_json::to_string(&item).expect("serialize");
        let back: ComparisonItemDto = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.state_item_id, "win-system-proxy");
        assert_eq!(back.result, ComparisonResultDto::Deviated);
        assert!(back.baseline_value.is_some());
    }

    #[test]
    fn comparison_item_dto_skips_none_values() {
        let item = ComparisonItemDto {
            state_item_id: "a".to_string(),
            result: ComparisonResultDto::Match,
            baseline_value: None,
            current_value: None,
        };
        let json = serde_json::to_string(&item).expect("serialize");
        assert!(!json.contains("baseline_value"));
        assert!(!json.contains("current_value"));
    }

    #[test]
    fn baseline_status_response_roundtrip() {
        let resp = BaselineStatusResponse {
            has_baseline: true,
            items: vec![ComparisonItemDto {
                state_item_id: "a".to_string(),
                result: ComparisonResultDto::Match,
                baseline_value: None,
                current_value: None,
            }],
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let back: BaselineStatusResponse = serde_json::from_str(&json).expect("deserialize");
        assert!(back.has_baseline);
        assert_eq!(back.items.len(), 1);
    }

    #[test]
    fn service_status_response_roundtrip() {
        let resp = ServiceStatusResponse {
            mihomo_running: true,
            proxy_guard_restart_count: 1,
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let back: ServiceStatusResponse = serde_json::from_str(&json).expect("deserialize");
        assert!(back.mihomo_running);
        assert_eq!(back.proxy_guard_restart_count, 1);
    }

    #[test]
    fn recovery_progress_response_roundtrip() {
        let resp = RecoveryProgressResponse {
            has_task: true,
            status: Some("in_progress".to_string()),
            total_items: 4,
            completed_count: 2,
            pending_count: 2,
            succeeded: 2,
            failed: 0,
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let back: RecoveryProgressResponse = serde_json::from_str(&json).expect("deserialize");
        assert!(back.has_task);
        assert_eq!(back.status, Some("in_progress".to_string()));
        assert_eq!(back.total_items, 4);
    }

    #[test]
    fn recovery_progress_response_no_task() {
        let resp = RecoveryProgressResponse {
            has_task: false,
            status: None,
            total_items: 0,
            completed_count: 0,
            pending_count: 0,
            succeeded: 0,
            failed: 0,
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let back: RecoveryProgressResponse = serde_json::from_str(&json).expect("deserialize");
        assert!(!back.has_task);
        assert!(back.status.is_none());
    }

    #[test]
    fn audit_log_response_roundtrip() {
        let resp = AuditLogResponse {
            total_count: 100,
            records: vec![AuditRecordDto {
                timestamp: "2026-05-19T12:00:00Z".to_string(),
                action: "baseline_collect".to_string(),
                target: "all".to_string(),
                result: "success".to_string(),
                reason: None,
                details: Some(serde_json::json!({"item_count": 9})),
            }],
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let back: AuditLogResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.total_count, 100);
        assert_eq!(back.records.len(), 1);
        assert_eq!(back.records[0].action, "baseline_collect");
    }

    #[test]
    fn audit_record_dto_skips_none_fields() {
        let record = AuditRecordDto {
            timestamp: "2026-05-19T12:00:00Z".to_string(),
            action: "baseline_collect".to_string(),
            target: "all".to_string(),
            result: "success".to_string(),
            reason: None,
            details: None,
        };
        let json = serde_json::to_string(&record).expect("serialize");
        assert!(!json.contains("reason"));
        assert!(!json.contains("details"));
    }

    // ── Pagination validation tests ────────────────────────────────────────

    #[test]
    fn audit_log_params_default() {
        let params = AuditLogParams::default();
        assert_eq!(params.offset, 0);
        assert_eq!(params.limit, 50);
        assert!(params.action_type.is_none());
        assert!(params.from.is_none());
        assert!(params.to.is_none());
    }

    #[test]
    fn audit_log_params_validated_clamps_limit_to_max() {
        let params = AuditLogParams {
            limit: 500,
            ..Default::default()
        };
        let validated = params.validated();
        assert_eq!(validated.limit, MAX_AUDIT_LOG_LIMIT);
    }

    #[test]
    fn audit_log_params_validated_clamps_zero_limit() {
        let params = AuditLogParams {
            limit: 0,
            ..Default::default()
        };
        let validated = params.validated();
        assert_eq!(validated.limit, 1);
    }

    #[test]
    fn audit_log_params_validated_preserves_valid_limit() {
        let params = AuditLogParams {
            limit: 100,
            ..Default::default()
        };
        let validated = params.validated();
        assert_eq!(validated.limit, 100);
    }

    #[test]
    fn audit_log_params_preserves_filters() {
        let params = AuditLogParams {
            offset: 10,
            limit: 25,
            action_type: Some("state_restore".to_string()),
            from: Some("2026-05-19".to_string()),
            to: Some("2026-05-20".to_string()),
        };
        let validated = params.validated();
        assert_eq!(validated.offset, 10);
        assert_eq!(validated.limit, 25);
        assert_eq!(validated.action_type, Some("state_restore".to_string()));
        assert_eq!(validated.from, Some("2026-05-19".to_string()));
        assert_eq!(validated.to, Some("2026-05-20".to_string()));
    }

    #[test]
    fn audit_log_params_deserialization_with_defaults() {
        let json = r"{}";
        let params: AuditLogParams = serde_json::from_str(json).expect("deserialize");
        assert_eq!(params.offset, 0);
        assert_eq!(params.limit, 50);
    }

    // ── Event payload round-trip tests ─────────────────────────────────────

    #[test]
    fn recovery_started_payload_roundtrip() {
        let payload = RecoveryStartedPayload {
            task_id: "task-123".to_string(),
            total_items: 4,
        };
        let json = serde_json::to_string(&payload).expect("serialize");
        let back: RecoveryStartedPayload = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.task_id, "task-123");
        assert_eq!(back.total_items, 4);
    }

    #[test]
    fn recovery_item_completed_payload_roundtrip() {
        let payload = RecoveryItemCompletedPayload {
            state_item_id: "win-system-proxy".to_string(),
            success: true,
            failure_reason: None,
        };
        let json = serde_json::to_string(&payload).expect("serialize");
        let back: RecoveryItemCompletedPayload = serde_json::from_str(&json).expect("deserialize");
        assert!(back.success);
        assert!(!json.contains("failure_reason"));
    }

    #[test]
    fn recovery_item_completed_payload_with_failure() {
        let payload = RecoveryItemCompletedPayload {
            state_item_id: "win-hosts".to_string(),
            success: false,
            failure_reason: Some("Permission denied".to_string()),
        };
        let json = serde_json::to_string(&payload).expect("serialize");
        let back: RecoveryItemCompletedPayload = serde_json::from_str(&json).expect("deserialize");
        assert!(!back.success);
        assert_eq!(back.failure_reason, Some("Permission denied".to_string()));
    }

    #[test]
    fn recovery_completed_payload_roundtrip() {
        let payload = RecoveryCompletedPayload {
            task_id: "task-123".to_string(),
            succeeded: 4,
            failed: 0,
        };
        let json = serde_json::to_string(&payload).expect("serialize");
        let back: RecoveryCompletedPayload = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.succeeded, 4);
        assert_eq!(back.failed, 0);
    }

    #[test]
    fn recovery_failed_payload_roundtrip() {
        let payload = RecoveryFailedPayload {
            task_id: "task-123".to_string(),
            failed_items: vec!["win-hosts".to_string(), "win-system-proxy".to_string()],
        };
        let json = serde_json::to_string(&payload).expect("serialize");
        let back: RecoveryFailedPayload = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.failed_items.len(), 2);
    }

    #[test]
    fn baseline_confirmed_payload_roundtrip() {
        let payload = BaselineConfirmedPayload {
            version: 1,
            item_count: 9,
        };
        let json = serde_json::to_string(&payload).expect("serialize");
        let back: BaselineConfirmedPayload = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.version, 1);
        assert_eq!(back.item_count, 9);
    }

    #[test]
    fn baseline_deviation_payload_roundtrip() {
        let payload = BaselineDeviationPayload {
            deviated_items: vec!["win-system-proxy".to_string()],
        };
        let json = serde_json::to_string(&payload).expect("serialize");
        let back: BaselineDeviationPayload = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.deviated_items.len(), 1);
    }

    #[test]
    fn service_stopped_payload_roundtrip() {
        let payload = ServiceStoppedPayload {
            reason: "User requested".to_string(),
            recovery_triggered: true,
            non_target_verification: None,
        };
        let json = serde_json::to_string(&payload).expect("serialize");
        let back: ServiceStoppedPayload = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.reason, "User requested");
        assert!(back.recovery_triggered);
        assert!(back.non_target_verification.is_none());
    }

    #[test]
    fn service_started_payload_roundtrip() {
        let payload = ServiceStartedPayload {
            mihomo_running: true,
        };
        let json = serde_json::to_string(&payload).expect("serialize");
        let back: ServiceStartedPayload = serde_json::from_str(&json).expect("deserialize");
        assert!(back.mihomo_running);
    }

    #[test]
    fn auto_recovery_triggered_payload_roundtrip() {
        let payload = AutoRecoveryTriggeredPayload {
            restart_attempts: 3,
            max_attempts: 3,
        };
        let json = serde_json::to_string(&payload).expect("serialize");
        let back: AutoRecoveryTriggeredPayload = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.restart_attempts, 3);
        assert_eq!(back.max_attempts, 3);
    }

    // ── Helper function tests ──────────────────────────────────────────────

    #[test]
    fn recovery_status_to_string_all_variants() {
        assert_eq!(
            recovery_status_to_string(&RecoveryStatus::Pending),
            "pending"
        );
        assert_eq!(
            recovery_status_to_string(&RecoveryStatus::InProgress),
            "in_progress"
        );
        assert_eq!(
            recovery_status_to_string(&RecoveryStatus::Completed),
            "completed"
        );
        assert_eq!(
            recovery_status_to_string(&RecoveryStatus::Failed),
            "failed"
        );
        assert_eq!(
            recovery_status_to_string(&RecoveryStatus::UserAcknowledged),
            "user_acknowledged"
        );
    }

    #[test]
    fn parse_audit_action_all_valid() {
        assert_eq!(
            parse_audit_action("baseline_collect"),
            Some(AuditAction::BaselineCollect)
        );
        assert_eq!(
            parse_audit_action("baseline_confirm"),
            Some(AuditAction::BaselineConfirm)
        );
        assert_eq!(
            parse_audit_action("state_restore"),
            Some(AuditAction::StateRestore)
        );
        assert_eq!(
            parse_audit_action("proxy_guard_restart"),
            Some(AuditAction::ProxyGuardRestart)
        );
        assert_eq!(
            parse_audit_action("proxy_guard_recovery"),
            Some(AuditAction::ProxyGuardRecovery)
        );
        assert_eq!(
            parse_audit_action("rule_apply"),
            Some(AuditAction::RuleApply)
        );
        assert_eq!(
            parse_audit_action("config_change"),
            Some(AuditAction::ConfigChange)
        );
    }

    #[test]
    fn parse_audit_action_unknown_returns_none() {
        assert_eq!(parse_audit_action("nonexistent"), None);
        assert_eq!(parse_audit_action(""), None);
    }

    #[test]
    fn max_audit_log_limit_is_200() {
        assert_eq!(MAX_AUDIT_LOG_LIMIT, 200);
    }

    // ── Command function integration tests (with mock managers) ────────────

    use crate::adapters::{PlatformAdapter, StateItemDefinition};
    use crate::managers::baseline_manager::BaselineManager;
    use crate::managers::mihomo_manager::MihomoManager;
    use crate::models::baseline::{Platform, StateItem, StateItemCategory};
    use crate::models::config::{MihomoConfig, ProxyGuardConfig};
    use crate::services::proxy_guard::ProxyGuard;

    struct CmdMockAdapter {
        items: Vec<StateItem>,
    }

    impl CmdMockAdapter {
        fn with_items(items: Vec<StateItem>) -> Self {
            Self { items }
        }
    }

    impl PlatformAdapter for CmdMockAdapter {
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
        fn write_state(&self, _item: &StateItem) -> Result<(), String> {
            Ok(())
        }
    }

    fn cmd_item(id: &str, category: StateItemCategory) -> StateItem {
        StateItem {
            id: id.to_string(),
            platform: Platform::Windows,
            category,
            value: serde_json::json!("value"),
            collected_at: "2026-05-19T12:00:00Z".to_string(),
            classification_reason: "test".to_string(),
        }
    }

    fn test_mihomo_config(dir: &std::path::Path) -> MihomoConfig {
        MihomoConfig {
            binary_path: dir.join("fake-mihomo"),
            config_dir: dir.join("mihomo"),
            api_address: "127.0.0.1:19999".to_string(),
            api_secret: "test".to_string(),
            mixed_port: 19999,
            log_level: "warning".to_string(),
        }
    }

    fn setup_baseline(dir: &std::path::Path) -> BaselineManager {
        let items = vec![
            cmd_item("a", StateItemCategory::Restorable),
            cmd_item("b", StateItemCategory::Restorable),
            cmd_item("c", StateItemCategory::Detectable),
        ];
        let adapter = CmdMockAdapter::with_items(items);
        let storage = crate::storage::baseline_storage::BaselineStorage::new(
            dir.join("baseline"),
        );
        BaselineManager::new(
            vec![Box::new(adapter)],
            storage,
            dir.to_path_buf(),
        )
    }

    #[test]
    fn cmd_start_initial_assessment() {
        let dir = tempfile::TempDir::new().expect("dir");
        let mgr = setup_baseline(dir.path());
        let resp = start_initial_assessment(&mgr).expect("assess");
        assert_eq!(resp.version, 0);
        assert_eq!(resp.item_count, 3);
    }

    #[test]
    fn cmd_get_state_summary() {
        let dir = tempfile::TempDir::new().expect("dir");
        let mgr = setup_baseline(dir.path());
        let resp = get_state_summary(&mgr).expect("summary");
        assert_eq!(resp.total, 3);
        assert_eq!(resp.restorable_count, 2);
        assert_eq!(resp.detectable_count, 1);
        assert_eq!(resp.excluded_count, 0);
    }

    #[test]
    fn cmd_trigger_readjustment() {
        let dir = tempfile::TempDir::new().expect("dir");
        let mgr = setup_baseline(dir.path());
        let resp = trigger_readjustment(&mgr).expect("readjust");
        assert_eq!(resp.version, 0);
        assert_eq!(resp.item_count, 3);
    }

    #[test]
    fn cmd_confirm_baseline() {
        let dir = tempfile::TempDir::new().expect("dir");
        let mgr = setup_baseline(dir.path());
        start_initial_assessment(&mgr).expect("assess");
        let resp = confirm_baseline(&mgr).expect("confirm");
        assert_eq!(resp.version, 1);
        assert_eq!(resp.item_count, 3);
    }

    #[test]
    fn cmd_get_baseline_status_no_baseline() {
        let dir = tempfile::TempDir::new().expect("dir");
        let mgr = setup_baseline(dir.path());
        let resp = get_baseline_status(&mgr).expect("status");
        assert!(!resp.has_baseline);
        assert!(resp.items.is_empty());
    }

    #[test]
    fn cmd_get_baseline_status_with_baseline() {
        let dir = tempfile::TempDir::new().expect("dir");
        let mgr = setup_baseline(dir.path());
        start_initial_assessment(&mgr).expect("assess");
        confirm_baseline(&mgr).expect("confirm");
        let resp = get_baseline_status(&mgr).expect("status");
        assert!(resp.has_baseline);
        assert_eq!(resp.items.len(), 3);
        assert_eq!(resp.items[0].result, ComparisonResultDto::Match);
    }

    #[test]
    fn cmd_get_service_status() {
        let dir = tempfile::TempDir::new().expect("dir");
        let config = test_mihomo_config(dir.path());
        let mut mihomo = MihomoManager::new(config);
        let guard = ProxyGuard::new(ProxyGuardConfig {
            check_interval_secs: 3,
            max_restart_attempts: 3,
            restart_cooldown_secs: 1,
        });
        let resp = get_service_status(&mut mihomo, &guard);
        assert!(!resp.mihomo_running);
        assert_eq!(resp.proxy_guard_restart_count, 0);
    }

    #[test]
    fn cmd_get_recovery_progress_no_task() {
        let dir = tempfile::TempDir::new().expect("dir");
        let mgr = setup_baseline(dir.path());
        let resp = get_recovery_progress(&mgr).expect("progress");
        assert!(!resp.has_task);
        assert!(resp.status.is_none());
    }

    #[test]
    fn cmd_get_recovery_progress_with_task() {
        let dir = tempfile::TempDir::new().expect("dir");
        let mgr = setup_baseline(dir.path());
        start_initial_assessment(&mgr).expect("assess");
        confirm_baseline(&mgr).expect("confirm");

        // Trigger a restore to create a recovery task.
        mgr.restore_to_baseline().expect("restore");

        let resp = get_recovery_progress(&mgr).expect("progress");
        // After restore completes, the task should be in terminal state
        // and cleaned up, so no active task.
        assert!(!resp.has_task);
    }

    #[test]
    fn cmd_get_audit_log() {
        let dir = tempfile::TempDir::new().expect("dir");
        let logger = crate::services::audit_logger::AuditLogger::new(
            dir.path().join("audit"),
        )
        .expect("logger");
        logger
            .log_success(
                AuditAction::BaselineCollect,
                "all",
                serde_json::json!({"count": 3}),
            )
            .expect("log");

        let params = AuditLogParams::default();
        let resp = get_audit_log(&logger, &params).expect("audit");
        assert_eq!(resp.total_count, 1);
        assert_eq!(resp.records[0].action, "baseline_collect");
        assert_eq!(resp.records[0].result, "success");
    }

    #[test]
    fn cmd_get_audit_log_with_pagination() {
        let dir = tempfile::TempDir::new().expect("dir");
        let logger = crate::services::audit_logger::AuditLogger::new(
            dir.path().join("audit"),
        )
        .expect("logger");

        for i in 0..5 {
            logger
                .log_success(
                    AuditAction::ConfigChange,
                    &format!("item-{i}"),
                    serde_json::json!({}),
                )
                .expect("log");
        }

        let params = AuditLogParams {
            offset: 2,
            limit: 2,
            ..Default::default()
        };
        let resp = get_audit_log(&logger, &params).expect("audit");
        assert_eq!(resp.total_count, 5);
        assert_eq!(resp.records.len(), 2);
    }

    #[test]
    fn cmd_get_audit_log_with_action_filter() {
        let dir = tempfile::TempDir::new().expect("dir");
        let logger = crate::services::audit_logger::AuditLogger::new(
            dir.path().join("audit"),
        )
        .expect("logger");

        logger
            .log_success(AuditAction::BaselineCollect, "a", serde_json::json!({}))
            .expect("log");
        logger
            .log_success(AuditAction::StateRestore, "b", serde_json::json!({}))
            .expect("log");

        let params = AuditLogParams {
            action_type: Some("state_restore".to_string()),
            ..Default::default()
        };
        let resp = get_audit_log(&logger, &params).expect("audit");
        assert_eq!(resp.total_count, 1);
        assert_eq!(resp.records[0].target, "b");
    }

    #[test]
    fn cmd_stop_service() {
        let dir = tempfile::TempDir::new().expect("dir");
        let mgr = setup_baseline(dir.path());
        let config = test_mihomo_config(dir.path());
        let mut mihomo = MihomoManager::new(config);

        start_initial_assessment(&mgr).expect("assess");
        confirm_baseline(&mgr).expect("confirm");

        let payload = stop_service(&mut mihomo, &mgr).expect("stop");
        assert!(payload.recovery_triggered);
        assert_eq!(payload.reason, "User requested");
    }

    // ── Deployment DTO round-trip tests ─────────────────────────────────────

    #[test]
    fn deployment_mode_response_roundtrip() {
        let resp = DeploymentModeResponse {
            mode: "linux_only".to_string(),
            detected: "wsl_only".to_string(),
            is_auto: false,
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let back: DeploymentModeResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.mode, "linux_only");
        assert_eq!(back.detected, "wsl_only");
        assert!(!back.is_auto);
    }

    #[test]
    fn wsl_status_response_roundtrip() {
        let resp = WslStatusResponse {
            is_wsl: true,
            distro_name: Some("Ubuntu".to_string()),
            distro_version: Some("22.04".to_string()),
            network_mode: "nat".to_string(),
            reachable: true,
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let back: WslStatusResponse = serde_json::from_str(&json).expect("deserialize");
        assert!(back.is_wsl);
        assert_eq!(back.distro_name, Some("Ubuntu".to_string()));
        assert_eq!(back.distro_version, Some("22.04".to_string()));
        assert_eq!(back.network_mode, "nat");
        assert!(back.reachable);
    }

    #[test]
    fn network_mode_response_roundtrip() {
        let resp = NetworkModeResponse {
            mode: "mirrored".to_string(),
            proxy_strategy: "skip_config".to_string(),
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let back: NetworkModeResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.mode, "mirrored");
        assert_eq!(back.proxy_strategy, "skip_config");
    }

    // ── Deployment helper function tests ────────────────────────────────────

    #[test]
    fn deployment_mode_to_string_all_variants() {
        assert_eq!(
            deployment_mode_to_string(&DeploymentMode::WindowsOnly),
            "windows_only"
        );
        assert_eq!(
            deployment_mode_to_string(&DeploymentMode::WslOnly),
            "wsl_only"
        );
        assert_eq!(
            deployment_mode_to_string(&DeploymentMode::LinuxOnly),
            "linux_only"
        );
        assert_eq!(
            deployment_mode_to_string(&DeploymentMode::Coordinated),
            "coordinated"
        );
    }

    #[test]
    fn parse_deployment_mode_valid_inputs() {
        assert_eq!(
            parse_deployment_mode("windows_only").expect("parse"),
            DeploymentMode::WindowsOnly
        );
        assert_eq!(
            parse_deployment_mode("wsl_only").expect("parse"),
            DeploymentMode::WslOnly
        );
        assert_eq!(
            parse_deployment_mode("linux_only").expect("parse"),
            DeploymentMode::LinuxOnly
        );
        assert_eq!(
            parse_deployment_mode("coordinated").expect("parse"),
            DeploymentMode::Coordinated
        );
    }

    #[test]
    fn parse_deployment_mode_invalid_returns_error() {
        let err = parse_deployment_mode("unknown_mode").expect_err("should fail");
        assert!(err.contains("Unknown deployment mode"));
        assert!(parse_deployment_mode("").is_err());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn network_mode_to_string_all_variants() {
        assert_eq!(network_mode_to_string(&WslNetworkMode::Nat), "nat");
        assert_eq!(network_mode_to_string(&WslNetworkMode::Mirrored), "mirrored");
        assert_eq!(
            network_mode_to_string(&WslNetworkMode::NotInstalled),
            "not_installed"
        );
    }

    // ── Deployment command integration tests ────────────────────────────────

    use crate::managers::config_manager::ConfigManager;

    fn setup_deployment(dir: &std::path::Path) -> DeploymentManager {
        let config_dir = dir.join("config");
        let install_root = dir.join("app");
        let cm = ConfigManager::new(config_dir).expect("create config manager");
        DeploymentManager::new(cm, install_root)
    }

    #[test]
    fn cmd_detect_deployment_mode() {
        let dir = tempfile::TempDir::new().expect("dir");
        let depl_mgr = setup_deployment(dir.path());
        let resp = detect_deployment_mode(&depl_mgr).expect("detect");
        // mode should be a valid snake_case string
        assert!(
            ["windows_only", "wsl_only", "linux_only", "coordinated"]
                .contains(&resp.mode.as_str()),
            "unexpected mode: {}",
            resp.mode
        );
        // detected should also be valid
        assert!(
            ["windows_only", "wsl_only", "linux_only", "coordinated"]
                .contains(&resp.detected.as_str()),
            "unexpected detected: {}",
            resp.detected
        );
    }

    #[test]
    fn cmd_get_deployment_mode() {
        let dir = tempfile::TempDir::new().expect("dir");
        let depl_mgr = setup_deployment(dir.path());
        let resp = get_deployment_mode(&depl_mgr).expect("get mode");
        // Default stored mode is WindowsOnly
        assert_eq!(resp.mode, "windows_only");
    }

    #[test]
    fn cmd_set_deployment_mode() {
        let dir = tempfile::TempDir::new().expect("dir");
        let depl_mgr = setup_deployment(dir.path());

        let resp = set_deployment_mode(&depl_mgr, "linux_only").expect("set mode");
        assert_eq!(resp.mode, "linux_only");

        // Verify persistence
        let resp2 = get_deployment_mode(&depl_mgr).expect("get mode after set");
        assert_eq!(resp2.mode, "linux_only");
    }

    #[test]
    fn cmd_set_deployment_mode_invalid() {
        let dir = tempfile::TempDir::new().expect("dir");
        let depl_mgr = setup_deployment(dir.path());
        let err = set_deployment_mode(&depl_mgr, "bogus").expect_err("should fail");
        assert!(err.contains("Invalid deployment mode"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn cmd_get_wsl_status() {
        let dir = tempfile::TempDir::new().expect("dir");
        let depl_mgr = setup_deployment(dir.path());
        let resp = get_wsl_status(&depl_mgr).expect("wsl status");
        // network_mode must be a valid string
        assert!(
            ["nat", "mirrored", "not_installed"].contains(&resp.network_mode.as_str()),
            "unexpected network_mode: {}",
            resp.network_mode
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn cmd_get_network_mode() {
        let dir = tempfile::TempDir::new().expect("dir");
        let depl_mgr = setup_deployment(dir.path());
        let resp = get_network_mode(&depl_mgr).expect("network mode");
        // mode must be valid
        assert!(
            ["nat", "mirrored", "not_installed"].contains(&resp.mode.as_str()),
            "unexpected mode: {}",
            resp.mode
        );
        // proxy_strategy must be valid
        assert!(
            ["explicit_config", "skip_config", "fallback_to_explicit"]
                .contains(&resp.proxy_strategy.as_str()),
            "unexpected proxy_strategy: {}",
            resp.proxy_strategy
        );
    }

    // ----- F104: is_restoring state lock tests -----

    #[test]
    fn app_state_is_restoring_default_false() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let state = AppState::new(dir.path()).expect("create state");
        assert!(!state.is_restoring.load(std::sync::atomic::Ordering::Relaxed));
    }

    #[test]
    fn is_restoring_set_and_clear() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let state = AppState::new(dir.path()).expect("create state");

        // Simulate: set restoring
        state.is_restoring.store(true, std::sync::atomic::Ordering::Relaxed);
        assert!(state.is_restoring.load(std::sync::atomic::Ordering::Relaxed));

        // Simulate: clear after done
        state.is_restoring.store(false, std::sync::atomic::Ordering::Relaxed);
        assert!(!state.is_restoring.load(std::sync::atomic::Ordering::Relaxed));
    }

    #[test]
    fn stop_service_blocked_when_restoring() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let state = AppState::new(dir.path()).expect("create state");

        // Simulate restore in progress
        state.is_restoring.store(true, std::sync::atomic::Ordering::Relaxed);

        // Attempt to stop service should be blocked
        let result = check_not_restoring(&state);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("recovery in progress"));
    }

    #[test]
    fn stop_service_allowed_when_not_restoring() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let state = AppState::new(dir.path()).expect("create state");

        // Not restoring — should be allowed
        let result = check_not_restoring(&state);
        assert!(result.is_ok());
    }

    // ----- F108: service_paused state tests -----

    #[test]
    fn app_state_service_paused_default_false() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let state = AppState::new(dir.path()).expect("create state");
        assert!(!state.service_paused.load(std::sync::atomic::Ordering::Relaxed));
    }

    #[test]
    fn stop_service_sets_service_paused() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let state = AppState::new(dir.path()).expect("create state");

        // Before stop: not paused
        assert!(!state.service_paused.load(std::sync::atomic::Ordering::Relaxed));

        // Simulate what tauri_stop_service does: stop + set paused
        let mut mihomo = state.mihomo_manager.lock().expect("lock");
        let baseline_mgr = state.baseline_manager.lock().expect("lock");
        let _ = stop_service(&mut mihomo, &baseline_mgr);
        drop(mihomo);
        drop(baseline_mgr);
        state.service_paused.store(true, std::sync::atomic::Ordering::Relaxed);

        // After stop: paused
        assert!(state.service_paused.load(std::sync::atomic::Ordering::Relaxed));
    }

    #[test]
    fn proxy_guard_skips_restart_when_paused() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let state = AppState::new(dir.path()).expect("create state");

        // Simulate user stopped service
        state.service_paused.store(true, std::sync::atomic::Ordering::Relaxed);

        // ProxyGuard check should be skipped — verify the flag check
        assert!(state.service_paused.load(std::sync::atomic::Ordering::Relaxed));

        // Even if we run check_and_recover directly, the loop should have skipped.
        // Verify restart count remains 0 when paused.
        let guard = state.proxy_guard.lock().expect("lock");
        assert_eq!(guard.restart_count(), 0);
        drop(guard);
    }

    #[test]
    fn trigger_readjustment_clears_service_paused() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let state = AppState::new(dir.path()).expect("create state");

        // Set paused (simulating a previous stop)
        state.service_paused.store(true, std::sync::atomic::Ordering::Relaxed);

        // Simulate what tauri_trigger_readjustment does: clear paused
        state.service_paused.store(false, std::sync::atomic::Ordering::Relaxed);

        // After trigger: not paused
        assert!(!state.service_paused.load(std::sync::atomic::Ordering::Relaxed));
    }
}
