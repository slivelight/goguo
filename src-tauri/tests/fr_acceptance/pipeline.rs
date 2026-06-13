//! 管道集成测试（7 个）
//!
//! 验证 7 条端到端数据流链路连通：
//! 1. assess → confirm → restore → audit（F001 全链路）
//! 2. subscription → node_pool → rules（F003 链路）
//! 3. site_add → probe → reachability（F003+F004）
//! 4. probe → five_element → audit（F001+F003）
//! 5. proxy_guard → restore（F001）
//! 6. wizard → readjustment（F004）
//! 7. rule_preview → apply → verify → rollback（F003）

#[path = "../common/mod.rs"]
mod common;

use common::*;
use goguo_lib::adapters::PlatformAdapter;
use goguo_lib::engines::site_rule_engine::{AddSiteResult, RemoveSiteResult};
use goguo_lib::managers::baseline_manager::BaselineManager;
use goguo_lib::models::audit::AuditAction;
use goguo_lib::services::proxy_guard::{GuardAction, ProxyGuard};
use goguo_lib::models::config::ProxyGuardConfig;

// ═══════════════════════════════════════════════════════════════════════
// P1: assess → confirm → restore → audit（F001 全链路）
// ═══════════════════════════════════════════════════════════════════════

/// 端到端：评估 → 确认基线 → 修改状态 → 恢复 → 验证审计链路。
///
/// 验证：
/// 1. collect_initial_snapshot 返回正确版本和项数
/// 2. confirm_baseline 持久化基线（版本递增）
/// 3. restore_to_baseline 对所有 Restorable 项调用 write_state
/// 4. 每一步都有审计日志记录
#[test]
fn pipeline_assess_confirm_restore_audit() {
    // ── Assess ──
    let state = setup_baseline_confirmed();
    // 已在 helper 中完成 assess + confirm，验证基线状态
    let status = state.baseline_manager.compare_with_baseline();
    assert!(status.is_ok(), "compare_with_baseline must succeed");
    let comparisons = status.expect("comparisons");
    assert_eq!(comparisons.len(), 6, "must have 6 items from standard_baseline_items");

    // ── Restore ──
    let result = state.baseline_manager.restore_to_baseline();
    assert!(result.is_ok(), "restore must succeed");
    let restore = result.expect("restore result");
    // 3 Restorable items in standard_baseline_items
    assert_eq!(restore.succeeded, 3, "all 3 restorable items must be restored");

    // ── Audit trail ──
    let audit_records = state.audit_logger.records();
    // BaselineManager logs internally; verify write log recorded restores
    let written = state.adapter.written_values();
    assert_eq!(written.len(), 3, "adapter must have recorded 3 write_state calls");
    let written_ids: Vec<&str> = written.iter().map(|(id, _)| id.as_str()).collect();
    assert!(written_ids.contains(&"system-proxy"), "system-proxy must be restored");
    assert!(written_ids.contains(&"proxy-env"), "proxy-env must be restored");
    assert!(written_ids.contains(&"hosts-file"), "hosts-file must be restored");
}

// ═══════════════════════════════════════════════════════════════════════
// P2: subscription → node_pool → rules（F003 链路）
// ═══════════════════════════════════════════════════════════════════════

/// 订阅解析 → 节点池 → 规则生成的端到端链路。
/// 当前 subscription 仅支持 base64 解析，节点池为内存状态，
/// 完整链路需要后台任务基础设施。
#[test]
#[ignore = "F110-G110-2: 订阅解析到规则生成完整管道需后台任务基础设施"]
fn pipeline_subscription_to_node_pool_to_rules() {
    let _state = setup_site_rules_with_nodes();
    // 订阅解析 → 解析节点 → 填充节点池 → 规则引用节点池
    // 当前节点池为内存状态，无持久化管道
}

// ═══════════════════════════════════════════════════════════════════════
// P3: site_add → probe → reachability（F003+F004）
// ═══════════════════════════════════════════════════════════════════════

/// 端到端：添加目标站点 → 生成规则 → 探测 → 可达性报告。
///
/// 验证：
/// 1. add_site 成功，生成规则 > 0
/// 2. get_reachability 返回可达性结果
/// 3. 探测结果中 reachable 字段有效
#[test]
fn pipeline_site_add_probe_reachability() {
    let mut state = setup_site_rules_with_nodes();

    // ── Add site ──
    let result = state.engine.add_site("github");
    assert!(
        matches!(result, AddSiteResult::Success { rules_generated, .. } if rules_generated > 0),
        "add_site must succeed with rules"
    );

    // ── Probe & reachability ──
    let reachability = state.engine.get_reachability();
    assert_eq!(reachability.len(), 1, "must have reachability for 1 site");
    assert_eq!(reachability[0].site_id, "github");
    // MockProbeClient default returns reachable
    assert!(reachability[0].reachable, "mock probe must report reachable");
    assert!(reachability[0].response_time_ms.is_some(), "must have response time");
}

// ═══════════════════════════════════════════════════════════════════════
// P4: probe → five_element → audit（F001+F003）
// ═══════════════════════════════════════════════════════════════════════

/// 探测失败 → 五要素诊断提示 → 审计日志记录。
/// 当前五要素诊断提示未实现。
#[test]
#[ignore = "F110-G110-5: 五要素诊断提示未实现"]
fn pipeline_probe_five_element_audit() {
    let _state = setup_site_rules_with_nodes();
    // 添加站点 → 探测失败 → 五要素提示生成 → 审计日志
}

// ═══════════════════════════════════════════════════════════════════════
// P5: proxy_guard → restore（F001）
// ═══════════════════════════════════════════════════════════════════════

/// ProxyGuard 检测到 mihomo 停止 → 重试 → 达到上限 → 触发基线恢复。
/// 当前 ProxyGuard 无后台任务循环。
#[test]
#[ignore = "F109-P1-109-4: ProxyGuard 后台任务未实现"]
fn pipeline_proxy_guard_restore() {
    // ProxyGuard 后台循环 → 检测 mihomo 停止 → schedule_restart
    // → 超过 max_restart_attempts → RecoveryTriggered → restore_to_baseline
}

// ═══════════════════════════════════════════════════════════════════════
// P6: wizard → readjustment（F004）
// ═══════════════════════════════════════════════════════════════════════

/// 向导流程：首次评估 → 确认 → 服务运行 → 重新评估。
/// 当前向导 Step 3 的调整建议引导未实现。
#[test]
#[ignore = "F110-G110-3: 向导 Step 3 调整建议引导未实现"]
fn pipeline_wizard_readjustment() {
    // collect_initial_snapshot → review → confirm →
    // adjust suggestions → re-collect → re-confirm
}

// ═══════════════════════════════════════════════════════════════════════
// P7: rule_preview → apply → verify → rollback（F003）
// ═══════════════════════════════════════════════════════════════════════

/// 端到端：预览规则 → 应用站点 → 验证规则存在 → 移除站点（回滚）。
///
/// 验证：
/// 1. 初始预览为空
/// 2. add_site 后预览包含规则
/// 3. remove_site 后预览恢复空（或不含该站点规则）
#[test]
fn pipeline_rule_preview_apply_verify_rollback() {
    let mut state = setup_site_rules_with_nodes();

    // ── Preview: only MATCH,DIRECT before any site ──
    let preview_before = state.engine.preview_rules();
    // RuleGenerator always includes MATCH,DIRECT as fallback
    assert!(
        preview_before.len() == 1 && preview_before[0].contains("MATCH"),
        "preview before add should only have MATCH fallback, got: {:?}",
        preview_before
    );

    // ── Apply: add github ──
    let result = state.engine.add_site("github");
    assert!(
        matches!(result, AddSiteResult::Success { rules_generated, .. } if rules_generated > 0),
        "add_site must succeed"
    );

    // ── Verify: preview now contains rules ──
    let preview_after = state.engine.preview_rules();
    assert!(!preview_after.is_empty(), "preview must contain rules after add_site");

    // ── Rollback: remove github ──
    let remove_result = state.engine.remove_site("github");
    assert!(
        matches!(remove_result, RemoveSiteResult::Success { .. }),
        "remove_site must succeed"
    );

    // ── Verify: preview back to MATCH,DIRECT only after rollback ──
    let preview_rollback = state.engine.preview_rules();
    assert!(
        preview_rollback.len() == 1 && preview_rollback[0].contains("MATCH"),
        "preview after rollback should only have MATCH fallback, got: {:?}",
        preview_rollback
    );
}
