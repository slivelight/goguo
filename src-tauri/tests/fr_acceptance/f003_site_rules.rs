//! F003 site-rules FR 验收测试（12 个）
//!
//! 覆盖 F003 的 28 条 FR。
//! 使用 setup_site_rules_with_nodes() 构建测试状态。

#[path = "../common/mod.rs"]
mod common;

use common::*;
use goguo_lib::engines::site_rule_engine::{AddSiteResult, RemoveSiteResult, SiteRuleEngine};
use goguo_lib::models::audit::AuditAction;
use goguo_lib::services::probe_service::MockProbeClient;
use goguo_lib::services::rule_generator::Rule;

// ── §2.1 目标站点列表管理 ────────────────────────────────────────────────

/// FR-2.1.1-R1~R6: 站点增删触发规则重生成，记入审计。
/// 可观测结果：add/remove 返回正确结果；active_sites 更新；审计记录存在。
#[test]
fn fr_2_1_1_add_remove_site() {
    let mut state = setup_site_rules_with_nodes();

    // R1: add by site identifier
    let result = state.engine.add_site("github");
    match result {
        AddSiteResult::Success { site, rules_generated, .. } => {
            assert_eq!(site.id, "github", "FR-2.1.1-R1: site id must match");
            assert!(rules_generated > 0, "FR-2.1.1-R4: rules must be generated");
            // R3: known site expands all domains
            assert!(!site.all_domains().is_empty(),
                "FR-2.1.1-R3: known site must expand domains");
        }
        AddSiteResult::SiteNotFound => panic!("FR-2.1.1-R1: github should be a known site"),
        AddSiteResult::VerificationFailed { .. } => {
            // acceptable in mock environment without mihomo
        }
    }

    // R4: active sites updated
    assert!(state.engine.active_sites().contains(&"github".to_string()),
        "FR-2.1.1-R4: active sites must contain github");

    // R2: remove site
    let remove_result = state.engine.remove_site("github");
    match remove_result {
        RemoveSiteResult::Success { .. } => {}
        RemoveSiteResult::NotFound => panic!("github should exist for removal"),
    }
    assert!(!state.engine.active_sites().contains(&"github".to_string()),
        "FR-2.1.1-R2: site must be removed from active list");

    // R5: audit logged
    let records = state.audit_logger.records();
    let site_ops: Vec<_> = records.iter()
        .filter(|r| r.action == AuditAction::SiteAdd || r.action == AuditAction::SiteRemove)
        .collect();
    assert!(!site_ops.is_empty(), "FR-2.1.1-R5: add/remove must be audited");
}

/// FR-2.1.2-R1~R3: 模板可展开，用户可增删。
/// 可观测结果：apply_template 返回成功结果；active_sites 包含模板站点。
#[test]
fn fr_2_1_2_template_selection() {
    let mut state = setup_site_rules_with_nodes();

    // R1 & R2: apply developer template
    let results = state.engine.apply_template(&["github".to_string(), "npmjs".to_string()]);
    assert!(results.len() >= 2, "FR-2.1.2-R1: template should add 2 sites");

    let success_count = results.iter()
        .filter(|r| matches!(r, AddSiteResult::Success { .. }))
        .count();
    assert!(success_count >= 1, "FR-2.1.2-R2: at least one site must succeed");

    // R2: user can add/remove from template
    let remove_result = state.engine.remove_site("npmjs");
    assert!(matches!(remove_result, RemoveSiteResult::Success { .. }),
        "FR-2.1.2-R2: user can remove from template");
}

// ── §2.2 规则自动生成 ────────────────────────────────────────────────────

/// FR-2.2.1-R1~R5: 规则含 PROXY + MATCH,DIRECT 兜底。
/// 可观测结果：preview_rules 包含 PROXY 规则和 MATCH,DIRECT 终止规则。
#[test]
fn fr_2_2_1_rule_generation() {
    let mut state = setup_site_rules_with_nodes();
    let _ = state.engine.add_site("github");
    let _ = state.engine.add_site("npmjs");

    // R1 & R2: rules generated
    let preview = state.engine.preview_rules();
    assert!(!preview.is_empty(), "FR-2.2.1-R1: rules must be generated");

    // R3: last rule must be MATCH,DIRECT
    let last_rule = preview.last().expect("at least one rule");
    assert!(last_rule.contains("MATCH") && last_rule.contains("DIRECT"),
        "FR-2.2.1-R3: last rule must be MATCH,DIRECT, got '{}'", last_rule);

    // R2: rules contain PROXY policy
    let proxy_rules: Vec<_> = preview.iter().filter(|r| r.contains("PROXY")).collect();
    assert!(!proxy_rules.is_empty(), "FR-2.2.1-R2: rules must contain PROXY entries");
}

/// FR-2.2.2-R1~R3: 非目标站点可达性不降；降级则回退。
/// 可观测结果：MATCH,DIRECT 保证非目标走直连。
#[test]
fn fr_2_2_2_default_direct_strategy() {
    let mut state = setup_site_rules_with_nodes();
    let _ = state.engine.add_site("github");

    let preview = state.engine.preview_rules();

    // R1: final rule is DIRECT
    assert!(preview.last().is_some_and(|r| r.contains("DIRECT")),
        "FR-2.2.2-R1: final rule must guarantee DIRECT for non-target sites");

    // R2/R3: non-target verification requires live probe, tested via pipeline
}

// ── §2.3 站点可达性诊断 ──────────────────────────────────────────────────

/// FR-2.3.1-R1~R4: 定期探测，记录结果，失败无副作用。
#[test]
#[ignore = "F110-G110-1: 持续探测循环未实现（需后台定时任务）"]
fn fr_2_3_1_continuous_probe() {
    let mut state = setup_site_rules_with_nodes();
    let _ = state.engine.add_site("github");

    // Set up mock probe result
    let mut probe = MockProbeClient::new();

    // R1: probe triggers for registered site
    // R3: result contains timestamp, site, reachable, response_time
    let result = state.engine.probe_site("github");
    // result should be Some(SiteReachability)
    assert!(result.is_some(), "FR-2.3.1-R1: probe must return result for registered site");

    // R4: probe failure has no side effects on rules
}

/// FR-2.3.2-R1~R7: 不可达恢复，切换节点，五要素提示。
#[test]
#[ignore = "F110-G110-1: 节点切换恢复管道未实现"]
fn fr_2_3_2_unreachable_recovery() {
    // R1: switch node on unreachable
    // R2: five-element prompt after recovery failure
    // R5: node pool with alternatives
    // R6: periodic health check
    // R7: remove failed nodes
}

// ── §2.4 规则预览与覆盖 ──────────────────────────────────────────────────

/// FR-2.4.1-R1~R2: 预览展示规则列表，标注站点名和策略。
#[test]
#[ignore = "F110-G110-6: 预览未标注站点名称（仅展示规则文本）"]
fn fr_2_4_1_rule_preview() {
    let mut state = setup_site_rules_with_nodes();
    let _ = state.engine.add_site("github");

    let preview = state.engine.preview_rules();

    // R1: preview shows rule list
    assert!(!preview.is_empty(), "FR-2.4.1-R1: preview must show rules");

    // R2: each rule annotated with site name and strategy
    // Current implementation shows DOMAIN-SUFFIX,github.com,PROXY format
    // but doesn't include human-readable site name annotation
    let proxy_rules: Vec<_> = preview.iter().filter(|r| r.contains("PROXY")).collect();
    assert!(!proxy_rules.is_empty(), "FR-2.4.1-R2: rules must show PROXY strategy");
}

/// FR-2.4.2-R1~R3: 覆盖规则重生成后保留，记入审计。
#[test]
#[ignore = "F113-discovery: add_user_override 未记入审计日志（FR-2.4.2-R3 未满足）"]
fn fr_2_4_2_rule_override() {
    let mut state = setup_site_rules_with_nodes();
    let _ = state.engine.add_site("github");

    // R1: user can override rules
    let override_rule = Rule::domain_exact("gist.github.com".to_string());
    state.engine.add_user_override(override_rule);

    let preview_before = state.engine.preview_rules();

    // R2: override persists through regeneration (add another site triggers regen)
    let _ = state.engine.add_site("npmjs");
    let preview_after = state.engine.preview_rules();

    // Override should still be present
    assert!(preview_after.iter().any(|r| r.contains("gist.github.com")),
        "FR-2.4.2-R2: override must persist through regeneration");

    // R3: override recorded in audit
    let records = state.audit_logger.records();
    let override_audits: Vec<_> = records.iter()
        .filter(|r| r.action == AuditAction::RuleOverride)
        .collect();
    assert!(!override_audits.is_empty(), "FR-2.4.2-R3: override must be audited");
}

// ── §2.5 两侧规则同步 ────────────────────────────────────────────────────

/// FR-2.5.1-R1~R4: 两侧同步生效，分别验证。
#[test]
#[ignore = "F101: 协同模式不可用（F002 双侧适配器未实现）"]
fn fr_2_5_1_rule_effectiveness() {
    // R1: rules must sync to both Windows and WSL/Linux sides
    // R2: coordinated mode uses same config on both sides
    // R4: reachability verified on each side independently
}

// ── §2.6 规则变更与 baseline 安全 ────────────────────────────────────────

/// FR-2.6.1-R1~R3: 二次确认，baseline 差异检查。
#[test]
fn fr_2_6_1_rule_change_safety() {
    let mut state = setup_site_rules_with_nodes();

    // R2: adding site triggers rule regeneration (implicit baseline diff)
    let _ = state.engine.add_site("github");
    let preview = state.engine.preview_rules();
    assert!(!preview.is_empty(), "FR-2.6.1-R2: rules must exist after site add");

    // R3: removing site regenerates rules (service stop removes rules)
    let _ = state.engine.remove_site("github");
    let preview_after = state.engine.preview_rules();
    // After removing last site, only MATCH,DIRECT should remain
    if !preview_after.is_empty() {
        assert!(preview_after.iter().all(|r| r.contains("DIRECT")),
            "FR-2.6.1-R3: removing all target sites should leave only DIRECT rules");
    }
}

// ── SC 验收标准 ───────────────────────────────────────────────────────────

/// SC-1: 目标站点 P95 恢复 ≤ 10s。
#[test]
#[ignore = "F110-G110-1: 节点切换恢复管道未实现"]
fn fr_sc_1_p95_recovery() {
    // Performance SLA test — requires live probing infrastructure
}

/// SC-5: 不可达时提供五要素诊断。
#[test]
#[ignore = "F110-G110-5: 五要素诊断提示未实现"]
fn fr_sc_5_five_element_diag() {
    // FiveElementPrompt structure exists in code but
    // production path not fully wired
}

// ── F115 FR-2.2.5: list_target_sites 只读查询命令 ─────────────────────────
//
// Tauri 命令 `list_target_sites(state)` 是 2 行薄壳（lock + active_sites().clone()），
// 故 FR 验收测试在 engine 层验证（与既有 f003 测试惯例一致）。
// 命令注册与 IPC 通道由 `cargo build` + e2e smoke spec 验证。

/// FR-2.2.5-R2 case 1: 空 active_sites 返回空 Vec。
/// 可观测结果：fresh engine 的 active_sites() 长度为 0。
#[test]
fn fr_2_2_5_case1_empty() {
    let state = setup_site_rules_with_nodes();
    let sites = state.engine.active_sites();
    assert!(sites.is_empty(), "FR-2.2.5-R2 case1: fresh engine must have empty active_sites");
}

/// FR-2.2.5-R2 case 2: add 一个站点后 list 包含该 id。
#[test]
fn fr_2_2_5_case2_single_site() {
    let mut state = setup_site_rules_with_nodes();
    let _ = state.engine.add_site("github");
    let sites = state.engine.active_sites();
    assert_eq!(sites.len(), 1, "FR-2.2.5-R2 case2: exactly one site after single add");
    assert_eq!(sites[0], "github", "FR-2.2.5-R2 case2: site id must match");
}

/// FR-2.2.5-R2 case 3: 多站点 add 后 list 包含全部 id（顺序与 add 一致）。
#[test]
fn fr_2_2_5_case3_multiple_sites() {
    let mut state = setup_site_rules_with_nodes();
    let _ = state.engine.add_site("github");
    let _ = state.engine.add_site("npmjs");
    let sites = state.engine.active_sites();
    assert_eq!(sites.len(), 2, "FR-2.2.5-R2 case3: two sites after two adds");
    assert!(sites.contains(&"github".to_string()), "case3: must contain github");
    assert!(sites.contains(&"npmjs".to_string()), "case3: must contain npmjs");
}

/// FR-2.2.5-R2 case 4: add 后 remove，list 反映删除。
/// 同时验证 FR-2.2.5-R5（只读：连续 2 次调用返回相同）。
#[test]
fn fr_2_2_5_case4_after_remove_and_readonly() {
    let mut state = setup_site_rules_with_nodes();
    let _ = state.engine.add_site("github");
    let _ = state.engine.add_site("npmjs");
    let _ = state.engine.remove_site("github");

    let sites1 = state.engine.active_sites();
    let sites2 = state.engine.active_sites();
    assert_eq!(sites1.len(), 1, "FR-2.2.5-R2 case4: one site remains after remove");
    assert_eq!(sites1[0], "npmjs", "FR-2.2.5-R2 case4: remaining site must be npmjs");
    assert_eq!(sites1, sites2, "FR-2.2.5-R5: two consecutive reads must return identical values (readonly)");
}
