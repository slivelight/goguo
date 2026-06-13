# Feature 113: 三层测试重构 — 设计文档

- **Feature**: 113-test-restructure
- **阶段**: `hf-design`
- **状态**: 草稿
- **日期**: 2026-06-11
- **上游规格**: `features/113-test-restructure/spec.md`
- **设计锚点**: `docs/principles/testing-principles.md`

## 1. 设计概述

本设计为 F001~F004 的 142 条 FR 提供三层自动化验收测试方案。按批次交付：

- **Batch 1**: 测试基建 + F001 验收测试（核心路径，覆盖最大 FR 数）
- **Batch 2**: F002 + F003 验收测试
- **Batch 3**: F004 验收测试（前端 + 后端）
- **Batch 4**: 契约测试 + 管道集成测试
- **Batch 5**: 追溯矩阵验证 + CI 集成

## 2. 目录结构

```
src-tauri/tests/
├── common/
│   └── mod.rs                    # setup helper 库
├── fr_acceptance/
│   ├── mod.rs                    # 模块入口
│   ├── f001_baseline.rs          # F001 验收测试（~19 个）
│   ├── f002_wsl.rs               # F002 验收测试（~8 个）
│   ├── f003_site_rules.rs        # F003 验收测试（~12 个）
│   ├── f004_backend.rs           # F004 后端验收测试（~8 个）
│   ├── contract.rs               # 契约测试（~20 个）
│   └── pipeline.rs               # 管道集成测试（~7 个）
├── integration_baseline.rs       # 已有，保留
├── integration_site_rules.rs     # 已有，保留
└── integration_wsl_linux.rs      # 已有，保留

src/__tests__/fr-acceptance/
└── f004-ui.test.tsx              # F004 前端验收测试（~15 个）

docs/
└── test-trace-matrix.md          # FR 追溯矩阵
```

## 3. Batch 1: 测试基建 + F001 验收测试

### 3.1 Setup Helper 设计

```rust
// src-tauri/tests/common/mod.rs

use std::sync::{Arc, Mutex};
use crate::managers::baseline_manager::BaselineManager;
use crate::managers::mihomo_manager::MihomoManager;
// ... 其他必要 import

/// 构建一个 baseline 已确认的测试状态
pub fn setup_baseline_confirmed() -> TestState {
    let adapters = vec![
        Box::new(MockPlatformAdapter::new_baseline_confirmed())
    ];
    let storage = MockBaselineStorage::new_with_confirmed_baseline();
    let audit_logger = MockAuditLogger::new();
    let baseline_mgr = BaselineManager::new(adapters, storage, audit_logger);
    // ... 构建完整 TestState
}

/// 构建一个 mihomo 运行中的测试状态
pub fn setup_service_running() -> TestState {
    let mut state = setup_baseline_confirmed();
    state.mihomo_manager = MockMihomoManager::new_running();
    state
}

/// 构建一个 mihomo 已停止的测试状态
pub fn setup_service_stopped() -> TestState {
    let mut state = setup_baseline_confirmed();
    state.mihomo_manager = MockMihomoManager::new_stopped();
    state
}

/// 构建一个站点规则 + 节点池已填充的测试状态
pub fn setup_site_rules_with_nodes() -> SiteRulesTestState {
    // ... 构建包含节点和规则的测试状态
}

/// 测试状态容器
pub struct TestState {
    pub baseline_manager: Arc<Mutex<BaselineManager>>,
    pub mihomo_manager: Arc<Mutex<MihomoManager>>,
    pub audit_logger: Arc<Mutex<MockAuditLogger>>,
    // ... 其他必要状态
}
```

### 3.2 F001 验收测试分组

F001 有 52 条 FR，按功能节分组后约 19 个验收测试：

| 验收测试 | 覆盖 FR | 断言的可观测结果 | 预期状态 |
|---------|---------|----------------|---------|
| `fr_2_1_1_snapshot_before_modification` | FR-2.1.1-R1~R4 | 快照时间戳早于任何 write_state；含 OS 信息；失败项标记 | 通过 |
| `fr_2_1_2_assessment_readonly` | FR-2.1.2-R1~R4 | 评估后网络配置未变；结果保存 | 通过 |
| `fr_2_2_1_baseline_formation` | FR-2.2.1-R1~R4 | baseline 确认后只读；版本递增 | 通过 |
| `fr_2_2_2_confirmation_interaction` | FR-2.2.2-R1~R4 | 确认前展示摘要；确认记入审计 | 通过 |
| `fr_2_3_1_state_item_classification` | FR-2.3.1-R1~R3 | 分类含理由；持久化可查 | 通过 |
| `fr_2_3_2_compare_with_baseline` | FR-2.3.2-R1~R3 | 对比输出差异清单；区分偏离/一致 | 通过 |
| `fr_2_3_3_restore_only_restorable` | FR-2.3.3-R1~R4 | 仅 Restorable 项被恢复；不可恢复项仅提示 | `#[ignore = "F109-P2-109-9"]` |
| `fr_2_4_1_stop_triggers_restore` | FR-2.4.1-R1~R3 | 停止操作触发恢复检查 | 通过 |
| `fr_2_4_2_restore_execution_and_audit` | FR-2.4.2-R1~R5 | 恢复结果记入审计；结果展示区分已恢复/需手动 | `#[ignore = "F109-P1-109-3"]` |
| `fr_2_4_3_non_target_verification` | FR-2.4.3-R1~R2 | 恢复后非目标站点可达性不降 | `#[ignore = "F109-P1-109-5"]` |
| `fr_2_5_1_proxy_guard_scope` | FR-2.5.1-R1 | 监控对象列表完整 | `#[ignore = "F109-P1-109-4"]` |
| `fr_2_5_2_proxy_guard_response` | FR-2.5.2-R1~R5 | 异常检测→系统代理清除；结果记审计 | `#[ignore = "F109-P1-109-4"]` |
| `fr_2_5_3_proxy_guard_strategy` | FR-2.5.3-R1~R4 | 重启策略；阈值可配置 | `#[ignore = "F109-P1-109-4"]` |
| `fr_2_6_1_recovery_task_persistence` | FR-2.6.1-R1~R3 | 恢复任务持久化可读 | `#[ignore = "F109-续跑未实现"]` |
| `fr_2_7_1_audit_scope` | FR-2.7.1-R1~R8 | 审计包含所有操作类型；不含隐私数据 | `#[ignore = "F109-P1-109-3"]` |
| `fr_2_7_2_audit_format` | FR-2.7.2-R1~R3 | 审计含 5 要素；支持筛选 | `#[ignore = "F110-G110-15"]` |
| `fr_2_8_1_five_element_prompt` | FR-2.8.1-R1~R3 | 五要素完整；建议可执行 | `#[ignore = "F110-G110-5"]` |
| `fr_2_9_1_deployment_identification` | FR-2.9.1~2.9.2 | 三种部署模式识别；双侧采集 | 通过 |
| `fr_sc_1_restore_all_restorable` | SC-1 | 停止后 100% Restorable 项回 baseline | `#[ignore = "F109-P0-109-1"]` |

## 4. Batch 2: F002 + F003 验收测试

### 4.1 F002 验收测试（~8 个）

| 验收测试 | 覆盖 FR | 预期状态 |
|---------|---------|---------|
| `fr_2_1_1_config_within_baseline_scope` | FR-2.1.1-R1~R4 | 配置仅操作 Restorable 项 | 通过 |
| `fr_2_1_2_config_execution` | FR-2.1.2-R1~R4 | 逐项配置+验证；结果展示 | 通过 |
| `fr_2_1_3_config_restore` | FR-2.1.3-R1~R3 | 停止时恢复 4 项到 baseline | 通过 |
| `fr_2_2_1_coordinated_mode` | FR-2.2.1-R1~R5 | 协同模式双侧管理 | `#[ignore = "F101"]` |
| `fr_2_2_2_state_item_reuse` | FR-2.2.2-R1~R2 | 复用 F001 核心抽象 | 通过 |
| `fr_2_3_1_network_mode_detection` | FR-2.3.1-R1~R2 | NAT/Mirrored/未安装 正确识别 | 通过 |
| `fr_2_3_2_strategy_selection` | FR-2.3.2-R1~R3 | 按模式选策略；记入审计 | 通过 |
| `fr_2_5_1_wsl_failure_prompt` | FR-2.5.1-R1~R3 | WSL 失败含五要素提示 | `#[ignore = "F110-G110-5"]` |

### 4.2 F003 验收测试（~12 个）

| 验收测试 | 覆盖 FR | 预期状态 |
|---------|---------|---------|
| `fr_2_1_1_add_remove_site` | FR-2.1.1-R1~R6 | 站点增删触发规则重生成；记入审计 | 通过 |
| `fr_2_1_2_template_selection` | FR-2.1.2-R1~R3 | 模板可展开；用户可增删 | 通过 |
| `fr_2_2_1_rule_generation` | FR-2.2.1-R1~R5 | 规则含 PROXY+MATCH,DIRECT 兜底；语法校验 | 通过 |
| `fr_2_2_2_default_direct_strategy` | FR-2.2.2-R1~R3 | 应用后非目标可达性不降；降级则回退 | 通过 |
| `fr_2_3_1_continuous_probe` | FR-2.3.1-R1~R4 | 定期探测；记录结果；失败无副作用 | `#[ignore = "F110-G110-1"]` |
| `fr_2_3_2_unreachable_recovery` | FR-2.3.2-R1~R7 | 切换节点恢复；五要素提示；健康检查 | `#[ignore = "F110-G110-1"]` |
| `fr_2_4_1_rule_preview` | FR-2.4.1-R1~R2 | 预览展示规则列表；标注站点名+策略 | `#[ignore = "F110-G110-6"]` |
| `fr_2_4_2_rule_override` | FR-2.4.2-R1~R3 | 覆盖规则重生成后保留；记入审计 | 通过 |
| `fr_2_5_1_rule_effectiveness` | FR-2.5.1-R1~R4 | 两侧同步生效；分别验证 | `#[ignore = "F101"]` |
| `fr_2_6_1_rule_change_safety` | FR-2.6.1-R1~R3 | 二次确认；baseline 差异检查 | 通过 |
| `fr_sc_1_p95_recovery` | SC-1 | 目标站点 P95 恢复 ≤ 10s | `#[ignore = "F110-G110-1"]` |
| `fr_sc_5_five_element_diag` | SC-5 | 不可达时提供五要素诊断 | `#[ignore = "F110-G110-5"]` |

## 5. Batch 3: F004 验收测试

### 5.1 F004 后端验收测试（~8 个）

| 验收测试 | 覆盖 FR | 预期状态 |
|---------|---------|---------|
| `fr_2_1_1_cross_platform_app` | FR-2.1.1-R1~R6 | 桌面应用形态；UI 数据来源本地 | 通过 |
| `fr_2_2_1_wizard_baseline_flow` | FR-2.2.1-R1~R5 | 首次引导流程；Step 3 调整建议 | `#[ignore = "F110-G110-3"]` |
| `fr_2_3_1_service_control` | FR-2.3.1-R1~R3 | 启停控制；二次确认；恢复进度展示 | 通过 |
| `fr_2_4_1_site_add_remove` | FR-2.4.1-R1~R5 | 站点标识/域名添加；关联域名展示 | 通过 |
| `fr_2_5_1_rule_preview` | FR-2.5.1-R1~R4 | 规则预览含站点名+策略 | `#[ignore = "F110-G110-6"]` |
| `fr_2_6_1_site_reachability` | FR-2.6.1-R1~R3 | 可达性状态展示；响应时间 | `#[ignore = "F110-G110-1"]` |
| `fr_2_7_1_notification` | FR-2.7.1-R1~R4 | 通知含时间戳；4 类语义化 | `#[ignore = "F110-G110-7"]` |
| `fr_sc_2_status_matches_backend` | SC-2 | UI 状态与后端实际一致 | 通过 |

### 5.2 F004 前端验收测试（~15 个）

前端验收测试使用 vitest + jest-dom，断言 DOM 渲染内容。

```typescript
// src/__tests__/fr-acceptance/f004-ui.test.tsx

describe('F004 FR Acceptance - UI', () => {
  // FR-2.3.2-R1~R4: 状态展示
  it('fr_2_3_2_displays_service_status', () => {
    renderWithRouter(<App />, { serviceRunning: true });
    expect(screen.getByText(/运行中/)).toBeInTheDocument();
  });

  // FR-2.2.1-R3: Step 3 调整引导
  it('fr_2_2_1_r3_wizard_step3_shows_adjustment_suggestions', () => {
    // ...
  });

  // FR-2.5.1-R1~R2: 规则预览
  it('fr_2_5_1_rule_preview_shows_site_name_and_strategy', () => {
    // ...
  });

  // ... 其余前端验收测试
});
```

## 6. Batch 4: 契约测试 + 管道集成测试

### 6.1 契约测试设计（~20 个）

```rust
// src-tauri/tests/fr_acceptance/contract.rs

/// 契约：所有适配器对同一 state_item_id 的 category 一致
#[test]
fn contract_adapter_category_consistent() {
    let adapters: Vec<Box<dyn PlatformAdapter>> = vec![
        Box::new(WindowsAdapter::new(mock_base())),
        Box::new(LinuxAdapter::new(mock_base())),
        Box::new(WslAdapter::new(mock_base())),
        Box::new(WslRemoteAdapter::new(mock_base())),
    ];

    let definitions_by_adapter: Vec<Vec<StateItemDefinition>> = adapters
        .iter().map(|a| a.state_item_definitions()).collect();

    // 收集所有 state_item_id
    let all_ids: HashSet<String> = definitions_by_adapter.iter()
        .flat_map(|defs| defs.iter().map(|d| d.id.clone()))
        .collect();

    // 每个出现在多个适配器中的 ID，其 category 必须一致
    for id in &all_ids {
        let categories: Vec<_> = definitions_by_adapter.iter()
            .flat_map(|defs| defs.iter().filter(|d| &d.id == id))
            .map(|d| d.category)
            .collect();
        if categories.len() > 1 {
            let unique: HashSet<_> = categories.iter().collect();
            assert_eq!(unique.len(), 1,
                "state_item_id '{}' has inconsistent categories: {:?}", id, categories);
        }
    }
}

/// 契约：Tauri command DTO 序列化往返
#[test]
fn contract_dto_roundtrip_restore_baseline_response() {
    let original = RestoreBaselineResponse {
        succeeded: 5,
        failed: 1,
        skipped: 2,
        non_target_verification: None,
        dns_flushed: true,
    };
    let json = serde_json::to_string(&original).unwrap();
    let deserialized: RestoreBaselineResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(original.succeeded, deserialized.succeeded);
    assert_eq!(original.failed, deserialized.failed);
    assert_eq!(original.skipped, deserialized.skipped);
}
```

### 6.2 管道集成测试设计（~7 个）

```rust
// src-tauri/tests/fr_acceptance/pipeline.rs

/// 管道：评估 → 确认 → 恢复 → 审计
#[test]
fn pipeline_assess_confirm_restore_audit() {
    let state = setup_baseline_confirmed();

    // 1. 评估
    let assessment = tauri_start_initial_assessment(state.clone()).unwrap();
    assert!(assessment.item_count > 0);

    // 2. 确认 baseline
    let confirm = tauri_confirm_baseline(state.clone()).unwrap();
    assert!(confirm.version > 0);

    // 3. 人工偏离
    inject_deviation(&state, "system-proxy", "deviated_value");

    // 4. 恢复
    let restore = tauri_restore_baseline(state.clone()).unwrap();
    assert!(restore.succeeded > 0);

    // 5. 审计包含完整链路
    let audit = get_audit_log(state.clone()).unwrap();
    assert!(audit.iter().any(|r| r.action == "restore_item"));
}

/// 管道：订阅导入 → NodePool → 规则生成 → mihomo 重载
#[test]
#[ignore = "F110-G110-2: 节点导入管道未闭环"]
fn pipeline_subscription_to_node_pool_to_rules() {
    let state = setup_site_rules_with_nodes();
    let result = import_subscription(state.clone(), test_subscription_url()).unwrap();
    let pool = get_node_pool_status(state.clone()).unwrap();
    assert!(pool.nodes.len() > 0, "NodePool must contain imported nodes");
}
```

## 7. Batch 5: 追溯矩阵 + CI 集成

### 7.1 追溯矩阵格式

```markdown
# FR 追溯矩阵

## F001 baseline-restore（52 FR）

| FR ID | 验收测试 | 状态 | 修复 Feature |
|-------|---------|------|-------------|
| FR-2.1.1-R1~R4 | fr_2_1_1_snapshot_before_modification | 通过 | — |
| FR-2.1.2-R1~R4 | fr_2_1_2_assessment_readonly | 通过 | — |
| FR-2.3.3-R1~R4 | fr_2_3_3_restore_only_restorable | 忽略 | F109-P2-109-9 |
| ... | ... | ... | ... |

## F002 wsl-support（19 FR）
...
## F003 site-rules（28 FR）
...
## F004 user-interaction（43 FR）
...
```

### 7.2 CI 集成

```yaml
# GitHub Actions 追加步骤
- name: FR Acceptance Tests
  run: |
    cargo test --test fr_acceptance -- --include-ignored 2>&1 | tee fr-report.txt
    # 统计忽略数（应与追溯矩阵一致）
    IGNORED=$(grep -c "test result:.*ignored" fr-report.txt || echo "0")
    echo "FR 验收测试忽略数: $IGNORED"
```

## 8. 风险雷达

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| Mock adapter 行为与真实系统不一致 | 中 | 中 | Mock 行为基于 spec 描述而非代码实现 |
| ~8 个验收测试依赖未实现功能 | 低 | 确定 | 标记 `#[ignore]`，作为 F109/F110 验收门 |
| F109/F110 修复后接口变更导致验收测试需更新 | 中 | 中 | 验收测试断言可观测结果而非接口签名，接口变更影响有限 |
| 前端验收测试的 DOM 断言因组件重构失效 | 中 | 中 | F110 G110-4 组件拆分后需同步更新 DOM 查询 |

## 9. 不修改的内容

- 任何 `src-tauri/src/` 下的生产代码
- 任何 `src/` 下的生产代码
- F001~F004 已关闭的 spec/design 文档
- 现有 923 个测试（不修改、不删除）
