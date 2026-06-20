# F115 — UX E2E 自动化测试基础设施正式化

| 字段 | 值 |
|------|----|
| 类型 | 基础设施 Feature |
| 阶段 | `hf-tasks`（进行中）— spec v3 + design M1 已通过评审 |
| 状态 | spec v3 + v3 勘误（2026-06-19，design review id:01~06 回写）/ design.md M1 草稿（6 条结构化标注已全量闭环） |
| Authority Source | [spec.md](./spec.md)（v3 + 勘误，当前活动）/ [design.md](./design.md)（M1，已评审） |
| 评审历史 | [spec-v1.md](./spec-v1.md)（v1，9 项标注）、[spec-v2.md](./spec-v2.md)（v2，3 项标注；FR-2.5 仍含 P1~P4） |
| 上游 | F114（PoC 已验证可行性） |
| 下游 | F201（接入规范首个应用案例） |
| GAP 移交 | 多实例 `/etc/environment` 覆盖 + mihomo config 流量阻断 → F110 §12 + GAP 索引文档 §9（建议 F116+ 修复） |

## 一句话范围

将 F114 PoC 验证过的 tauri-driver + WebDriverIO + @wdio/tauri-service 三件套**正式化为项目级基础设施**：目录规范化 + helpers 分层 + 97s 优化（3 杠杆）+ L1~L5 测试等级矩阵（F201 首案例）+ Feature 接入规范（AGENTS.md §7 强制 + §N 章节模板）+ 开发环境配置文档化（镜像绕过 + 多实例已知限制声明）。**不集中规划 spec 覆盖**、**不引入 CI/CD**、**不修复多实例 GAP**（推到 F116+），各 Feature 自行决定 e2e 覆盖。

详见 [spec.md](./spec.md)。
