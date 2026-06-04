# ADR-0004: 数据存储策略 — 安装根目录下文件式 JSON

- **Status**: accepted (amended 2026-06-02)
- **Date**: 2026-05-12
- **Deciders**: 用户
- **Affected Features**: 001, 002, 003, 004

## Context

四个 feature 共享以下数据存储需求：

1. **Baseline 快照**：结构化网络状态快照，需持久化、可读、可对比（Feature 001 FR-2.3.2）
2. **审计日志**：只追加的结构化操作记录（Feature 001 FR-2.7）
3. **站点定义**：JSON 格式的站点域名映射（Feature 003 FR-2.1.2-R3）
4. **规则配置**：生成的 mihomo 配置（Feature 003 FR-2.2）
5. **运行时状态**：恢复任务记录、节点池状态、探测结果（Feature 001 FR-2.6、Feature 003 FR-2.3）
6. **用户配置**：部署模式、已选站点等（Feature 002、003、004）

所有 feature 的 CON-5 约束：数据存储在 GoGuo 安装根目录下的统一数据目录，不上传远程服务器。

## Decision

**采用安装根目录下的文件式 JSON 存储，按数据类型分目录。**

目录结构：

```text
<install-root>/
  data/
    baseline/
      initial-snapshot.json       # 初始状态快照
      baseline-v1.json            # 用户确认的 baseline
      baseline-v2.json            # 后续版本（如有调整）
    audit/
      audit-2026-05-12.jsonl      # 按日期滚动的审计日志（JSONL 格式）
    config/
      settings.json               # 用户配置（部署模式、通知偏好等）
      site-definitions/           # 站点定义
        github.json
        npmjs.json
        ...
        custom/                   # 用户自定义站点定义
    rules/
      current-rules.yaml          # 当前生效的 mihomo 规则
      previous-rules.yaml         # 上一份规则（回退用）
    state/
      recovery-task.json          # 未完成的恢复任务记录
      node-pool.json              # 代理节点池状态
      probe-history.jsonl         # 探测历史（滚动）
    mihomo/
      config.yaml                 # mihomo 运行配置
      mihomo.bin                  # mihomo 二进制
```

格式选择：
- **Baseline / 站点定义 / 配置 / 状态**：JSON（人类可读、结构化、调试友好）
- **审计日志 / 探测历史**：JSONL（JSON Lines，只追加，按日期滚动）
- **mihomo 配置**：YAML（mihomo 原生格式）

## Alternatives Considered

| 方案 | 优势 | 劣势 | 结论 |
|------|------|------|------|
| **文件式 JSON + JSONL** | 可读、可调试、无额外依赖、符合 CON-5 | 无索引、大文件性能衰减、无事务 | **选定**：数据量小（baseline ~KB 级，审计 ~MB 级） |
| SQLite | 索引查询、事务、结构化 | 需依赖、二进制不可人工审查、不符合"可读可调试"偏好 | 排除：当前数据量不需要 |
| TOML | 配置友好 | 数组/嵌套可读性差、不适合日志 | 排除：仅适用于简单配置 |
| 二进制格式 (bincode/MessagePack) | 性能最优 | 不可读、调试困难 | 排除：违反可读性要求 |

## Consequences

- **正面**: 零依赖存储；文件可直接用文本编辑器审查；备份和恢复通过文件复制即可；站点定义直接复用现有 `config/sites/*.json` 格式。
- **负面**: 审计日志增长需定期清理策略；无原子写入保障（需通过 write-to-temp + rename 模式模拟）；并发写入需加文件锁。
- **数据安全**: 所有文件遵循 OS 最小权限原则（Feature 001 NFR-3.3-4）；审计日志不含用户访问明细（Feature 001 FR-2.7.1-R6~R8）。

## Amendment (2026-06-02)

v0.1.0 实现误用 Tauri `app_data_dir()` 作为数据根目录，与本文档的 `<install-root>/data/` 结构不一致。本次修正：

- **生产模式**：`std::env::current_exe().parent()` 作为 `install_root`（用户解压后的目录）
- **开发模式**：`<CARGO_MANIFEST_DIR>/../../release/` 作为 `install_root`
- **打包方式**：从 NSIS/deb/rpm 改为便携包（zip/tar.gz + AppImage）
- **目录结构微调**：mihomo 二进制放 `bin/mihomo`，配置/数据放 `data/mihomo/`（与 `AppConfig::default_for()` 实现对齐）

修正后的目录结构：

```text
<install-root>/
  goguo(.exe)                      # 主二进制
  bin/
    mihomo                         # mihomo 二进制
  data/
    baseline/
    audit/
    config/
      settings.json
      site-definitions/
    rules/
    mihomo/
      config.yaml                  # mihomo 运行配置
```

无数据迁移（当前无外部用户）。

## References

- `features/001-baseline-restore/spec.md`（CON-5, FR-2.3.2, FR-2.7, NFR-3.3-1）
- `features/003-site-rules/spec.md`（FR-2.1.2-R3 站点定义格式）
- `features/004-user-interaction/spec.md`（CON-5）
- `/mnt/d/software/github-host/config/sites/*.json`（现有站点定义参考）
