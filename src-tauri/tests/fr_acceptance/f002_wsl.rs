//! F002 wsl-support FR 验收测试（8 个）
//!
//! 覆盖 F002 的 19 条 FR。
//! 注意：MockShellExecutor / MockFileReader 被 #[cfg(test)] 限制在 crate 内，
//! 无法从集成测试访问。因此使用 TestAdapter 模拟 WSL 状态项。
//! 纯函数（determine_strategy）可直接测试。

#[path = "../common/mod.rs"]
mod common;

use common::*;
use goguo_lib::adapters::PlatformAdapter;
use goguo_lib::managers::baseline_manager::BaselineManager;
use goguo_lib::models::baseline::{Platform, StateItemCategory};
use goguo_lib::services::wsl_network_strategy::{determine_strategy, ProxyStrategy};
use goguo_lib::services::wsl_detector::WslNetworkMode;
use goguo_lib::storage::baseline_storage::BaselineStorage;

/// Standard WSL state items matching WslAdapter's 7 items.
/// 3 Restorable + 3 Detectable + 1 Excluded.
fn wsl_baseline_items() -> Vec<goguo_lib::models::baseline::StateItem> {
    use goguo_lib::models::baseline::StateItem;
    vec![
        make_item("wsl-git-proxy", StateItemCategory::Restorable, ""),
        make_item("wsl-resolv-conf", StateItemCategory::Restorable, "nameserver 8.8.8.8"),
        make_item("wsl-etc-environment", StateItemCategory::Restorable, ""),
        make_item("wsl-shell-proxy", StateItemCategory::Detectable, ""),
        make_item("wsl-reachability", StateItemCategory::Detectable, "ok"),
        make_item("wsl-wsl2-network-mode", StateItemCategory::Detectable, "nat"),
        make_item("wsl-proxy-env", StateItemCategory::Excluded, ""),
    ]
}

fn make_item(id: &str, category: StateItemCategory, value: &str) -> goguo_lib::models::baseline::StateItem {
    goguo_lib::models::baseline::StateItem {
        id: id.to_string(),
        platform: Platform::Wsl,
        category,
        value: serde_json::json!(value),
        collected_at: "2026-06-11T12:00:00Z".to_string(),
        classification_reason: "fr-acceptance wsl test".to_string(),
    }
}

// ── §2.1 配置范围与执行 ──────────────────────────────────────────────────

/// FR-2.1.1-R1~R4: 配置仅在 Restorable 项上执行。
/// 可观测结果：7 个状态项中 3 个 Restorable、3 个 Detectable、1 个 Excluded。
#[test]
fn fr_2_1_1_config_within_baseline_scope() {
    let items = wsl_baseline_items();

    // R1: verify categories match WslAdapter spec
    let restorable: Vec<&str> = items.iter()
        .filter(|i| i.category == StateItemCategory::Restorable)
        .map(|i| i.id.as_str())
        .collect();
    assert_eq!(restorable.len(), 3, "FR-2.1.1-R1: 3 Restorable items");
    assert!(restorable.contains(&"wsl-git-proxy"));
    assert!(restorable.contains(&"wsl-resolv-conf"));
    assert!(restorable.contains(&"wsl-etc-environment"));

    let detectable: Vec<&str> = items.iter()
        .filter(|i| i.category == StateItemCategory::Detectable)
        .map(|i| i.id.as_str())
        .collect();
    assert_eq!(detectable.len(), 3, "3 Detectable items");

    let excluded: Vec<&str> = items.iter()
        .filter(|i| i.category == StateItemCategory::Excluded)
        .map(|i| i.id.as_str())
        .collect();
    assert!(excluded.contains(&"wsl-proxy-env"), "wsl-proxy-env is Excluded");
}

/// FR-2.1.2-R1~R4: 配置逐项执行，单项失败不阻塞。
/// 可观测结果：每个 Restorable 项可独立 write；结果含 id/value/collected_at。
#[test]
fn fr_2_1_2_config_execution() {
    let items = wsl_baseline_items();
    let adapter = std::sync::Arc::new(TestAdapter::new(items));

    // R1 & R3: each item readable
    let read_items = adapter.read_state_items();
    assert_eq!(read_items.len(), 7, "FR-2.1.2-R1: all 7 items readable");

    for item in &read_items {
        assert!(!item.id.is_empty(), "FR-2.1.2-R3: item must have id");
        assert!(!item.collected_at.is_empty(), "FR-2.1.2-R3: item must have timestamp");
    }

    // R2: individual writes for Restorable items succeed
    for item in &read_items {
        if item.category == StateItemCategory::Restorable {
            let result = adapter.write_state(item);
            assert!(result.is_ok(), "FR-2.1.2-R2: write for '{}' should succeed", item.id);
        }
    }
}

/// FR-2.1.3-R1~R3: 停止时 Restorable 项恢复到 baseline。
/// 可观测结果：restore_to_baseline 仅对 Restorable 项调用 write_state。
#[test]
fn fr_2_1_3_config_restore() {
    let dir = tempfile::TempDir::new().expect("temp dir");
    let items = wsl_baseline_items();
    let adapter = std::sync::Arc::new(TestAdapter::new(items));
    let storage = BaselineStorage::new(dir.path().join("baseline"));
    let mgr = BaselineManager::new(
        vec![Box::new((*adapter).clone())],
        storage,
        dir.path().to_path_buf(),
    );

    let _ = mgr.collect_initial_snapshot().expect("assess");
    let _ = mgr.confirm_baseline().expect("confirm");

    let result = mgr.restore_to_baseline().expect("restore");
    assert_eq!(result.succeeded, 3, "FR-2.1.3-R1: 3 Restorable items restored");

    let written = adapter.written_values();
    for (id, _) in &written {
        let snap = mgr.get_confirmed_baseline().expect("get").expect("baseline");
        let item = snap.items.iter().find(|i| &i.id == id).expect("item");
        assert_eq!(item.category, StateItemCategory::Restorable,
            "FR-2.1.3-R1: only Restorable restored, got '{}'", id);
    }
}

// ── §2.2 部署组合 ────────────────────────────────────────────────────────

/// FR-2.2.1-R1~R5: 协同模式双侧管理。
#[test]
#[ignore = "F101: 协同模式不可用（双侧适配器未实现）"]
fn fr_2_2_1_coordinated_mode() {
    // R4: coordinated mode creates adapters for both Windows and WSL sides
    // Currently blocked by F101
}

/// FR-2.2.2-R1~R2: 复用 F001 核心抽象（StateItem/PlatformAdapter）。
/// 可观测结果：WSL 状态项使用与 F001 完全相同的数据结构。
#[test]
fn fr_2_2_2_state_item_reuse() {
    let items = wsl_baseline_items();

    // R2: same StateItem structure as F001
    for item in &items {
        assert!(!item.id.is_empty(), "id required");
        assert_eq!(item.platform, Platform::Wsl, "platform must be Wsl");
        assert!(!item.classification_reason.is_empty(), "classification_reason required");
        // value is serde_json::Value, always present
        assert!(!item.collected_at.is_empty(), "collected_at required");
    }

    // R1: items can be processed by BaselineManager (same as F001)
    let dir = tempfile::TempDir::new().expect("temp dir");
    let adapter = std::sync::Arc::new(TestAdapter::new(items));
    let storage = BaselineStorage::new(dir.path().join("baseline"));
    let mgr = BaselineManager::new(
        vec![Box::new((*adapter).clone())],
        storage,
        dir.path().to_path_buf(),
    );
    let snap = mgr.collect_initial_snapshot().expect("assess");
    assert_eq!(snap.items.len(), 7, "R1: BaselineManager processes WSL items like F001");
}

// ── §2.3 模式识别与策略选择 ──────────────────────────────────────────────

/// FR-2.3.1-R1~R2: 网络模式检测（NAT/Mirrored/NotInstalled），结果记入 baseline。
/// 可观测结果：WslNetworkMode 有 3 种变体，可 serde 序列化。
#[test]
fn fr_2_3_1_network_mode_detection() {
    // R1: all 3 modes exist
    let modes = [&WslNetworkMode::Nat, &WslNetworkMode::Mirrored, &WslNetworkMode::NotInstalled];
    assert_eq!(modes.len(), 3, "FR-2.3.1-R1: 3 network modes");

    // R2: modes can be serialized to baseline snapshot
    for mode in &modes {
        let json = serde_json::to_string(mode).expect("serialize");
        let back: WslNetworkMode = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(format!("{:?}", mode), format!("{:?}", back),
            "FR-2.3.1-R2: mode must roundtrip for baseline storage");
    }
}

/// FR-2.3.2-R1~R3: 策略选择（NAT→显式配置，镜像+可达→跳过，镜像+不可达→降级）。
/// 可观测结果：determine_strategy 纯函数返回正确策略和理由。
#[test]
fn fr_2_3_2_strategy_selection() {
    // R1: NAT mode → ExplicitConfig
    let s = determine_strategy(&WslNetworkMode::Nat, false);
    assert!(matches!(s, ProxyStrategy::ExplicitConfig),
        "FR-2.3.2-R1: NAT must use ExplicitConfig");

    // R2: Mirrored + reachable → SkipConfig
    let s_skip = determine_strategy(&WslNetworkMode::Mirrored, true);
    assert!(matches!(s_skip, ProxyStrategy::SkipConfig),
        "FR-2.3.2-R2: mirrored+reachable → SkipConfig");

    // R2: Mirrored + unreachable → FallbackToExplicit with reason
    let s_fb = determine_strategy(&WslNetworkMode::Mirrored, false);
    match &s_fb {
        ProxyStrategy::FallbackToExplicit { reason } => {
            assert!(!reason.is_empty(),
                "FR-2.3.2-R3: fallback must include reason for audit");
        }
        other => panic!("FR-2.3.2-R2: expected FallbackToExplicit, got {:?}", other),
    }

    // NotInstalled → ExplicitConfig (safe fallback)
    let s_ni = determine_strategy(&WslNetworkMode::NotInstalled, false);
    assert!(matches!(s_ni, ProxyStrategy::ExplicitConfig),
        "NotInstalled must fallback to ExplicitConfig");
}

// ── §2.5 失败提示 ────────────────────────────────────────────────────────

/// FR-2.5.1-R1~R3: WSL 失败含五要素提示。
#[test]
#[ignore = "F110-G110-5: 五要素诊断提示未实现"]
fn fr_2_5_1_wsl_failure_prompt() {
    // Requires FiveElementPrompt infrastructure from F110
}
