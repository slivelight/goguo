# GoGuo OPP-004 PC 端 Linux/WSL 支持产品发现草稿

- 状态：已确认（discovery-review 通过）
- 当前阶段：`hf-specify`（待进入）
- 主题：`OPP-004` PC 端 Linux/WSL 支持
- 上游输入：
  - `docs/insights/2026-04-29-goguo-strategy-discovery.md`（OPP-004 定义）
  - `docs/insights/2026-04-30-goguo-strategy-discovery-approval.md`（审批记录）
  - `features/001-baseline-restore/spec.md`（FR-2.9 交接边界）
- Discovery 目标：把 `OPP-004` 收敛为可评审的产品发现输入，供 `hf-discovery-review` 审查；通过后再进入 `hf-specify`。
- 下一建议节点：`hf-discovery-review`

## 1. 问题陈述

Feature 001（OPP-002）已建立 Windows 侧的 baseline 评估与恢复闭环，并在 FR-2.9 中完成了 Windows 与 WSL/Linux 协同环境的只读评估和差异报告。但只读评估本身不能解决开发者的核心问题：在 Windows + WSL/Linux 混合环境中，浏览器可能可用而 CLI/VS Code/Git 不可用，且用户无法自行修复两侧网络状态不一致。

本轮问题不是"为 Linux/WSL 构建完整的独立网络工具"，而是回答：GoGuo 能否将 Feature 001 已验证的 baseline 评估与恢复能力扩展到 WSL/Linux 侧，使 PC 端开发者无论从 Windows 还是 WSL 入口操作，都能获得一致的目标站点可达性或明确的诊断提示。

## 2. 目标用户与使用情境

| 用户 | 使用情境 | Struggling Moment | 期望进展 |
|------|----------|-------------------|----------|
| PC 端开发者 | Windows + WSL/Linux 并用，在 WSL 中运行 Git、VS Code Remote、agent 工具、包管理器 | 浏览器可用但 WSL 中 git push 失败、npm install 超时、agent 工具无法连接 | WSL 侧网络状态可被自动配置和恢复，无需手动编辑 .bashrc 或代理环境变量 |
| PC 端知识工作者 | 在 WSL 中撰写 wiki、访问技术文档、使用协作工具 | 不知道为什么 Windows 能访问但 WSL 不行 | 能看到两侧差异的具体原因，并有一键修复路径 |

本轮主用户是 PC 端开发者与知识工作者。这与 OPP-002 的用户定位一致，但本轮聚焦 WSL/Linux 入口的可达性。

## 3. Why now / 当前价值判断

`OPP-004` 是目标用户环境前置适配，原因：

- Feature 001（OPP-002）已完成 baseline 评估与恢复的规格定义，WSL/Linux 侧状态项清单和分类已产出，具备扩展基础。
- 战略发现确认 PC 端 Windows + Linux/WSL 开发者工作流是 MVP 核心场景，若 WSL 侧不可用，MVP 不完整。
- OPP-001（目标站点规则配置）的站点可达性能力需要 Windows 与 WSL/Linux 两侧一致生效才有意义；若 WSL 侧不支持自动配置，OPP-001 的价值会大幅降低。
- 现有原型已有 WSL/Linux 相关的代理配置经验（环境变量、Git 配置），技术风险可控。

## 4. 当前轮 wedge / 最小机会点

当前轮唯一主 wedge：

> PC 端 Linux/WSL 自动配置支持：基于 Feature 001 FR-2.9 的只读评估产出，将 WSL/Linux 侧的"可恢复项"（代理环境变量、Git 代理配置、/etc/resolv.conf、/etc/environment）从只读评估升级为自动配置和恢复，确保 Windows 与 WSL/Linux 协同部署下两侧关键网络状态一致可见和可控。

本轮将 WSL/Linux 侧从"只读评估"推进到"可配置、可恢复"，但限制在已分类为"可恢复项"的状态项范围内，不扩展到 shell 配置代理（可检测不可恢复项）等复杂场景。

当前轮不把"完整 Linux 桌面环境支持"或"独立 Linux 服务器部署"作为完成标准。

## 5. 已确认事实

| 事实 | 证据来源 | 对本轮的含义 |
|------|----------|--------------|
| Feature 001 已定义 WSL/Linux 侧 7 个状态项及其分类 | `features/001-baseline-restore/spec.md` FR-2.1.1 | 4 个可恢复项可直接进入自动配置设计；3 个可检测不可恢复项保持只读。 |
| 部署组合已定义（仅 Windows / 仅 WSL / 协同部署） | `features/001-baseline-restore/spec.md` FR-2.9.1 | 需覆盖三种组合的配置和恢复逻辑。 |
| WSL2 网络模式（NAT / 镜像）影响代理行为 | 网络技术事实 | 两种模式下的代理配置策略不同，需要分别处理。 |
| 现有原型已有 PowerShell 模块管理 WSL 代理配置的经验 | `arch.md` 模块地图 | 可复用已有 WSL 交互模式。 |
| WSL 镜像模式下 Windows 代理可能自动对 WSL 生效 | WSL2 技术文档 | 镜像模式可能不需要额外 WSL 侧配置；NAT 模式需要显式配置。 |

## 6. 关键假设与风险

### Desirability

| 假设 | 风险 | 本轮处理 |
|------|------|----------|
| 开发者愿意让 GoGuo 自动修改 WSL 代理环境变量和 Git 配置 | 若修改导致 WSL 侧行为异常，开发者信任度下降 | 沿用 Feature 001 的二次确认和审计机制；首次配置前展示具体修改内容。 |
| 开发者理解 NAT 模式和镜像模式的代理行为差异 | 若文档不清，用户可能误判问题原因 | 在差异报告中明确标注当前 WSL2 网络模式及对应策略。 |

### Feasibility

| 假设 | 风险 | 本轮处理 |
|------|------|----------|
| WSL 代理环境变量、Git 配置、/etc/resolv.conf 可通过 Windows 侧命令可靠写入 | WSL 发行版差异、权限限制可能导致写入失败 | probe 中验证主流发行版（Ubuntu、Debian）的写入路径。 |
| WSL2 镜像模式下代理配置可简化或跳过 | 镜像模式行为可能在 Windows 版本更新中变化 | 识别镜像模式并采用最小侵入策略。 |
| 仅 Windows / 仅 WSL 两种部署下状态项管理可复用 Feature 001 的核心逻辑 | 部署环境差异可能需要适配层 | 设计阶段确保状态项管理抽象层可跨部署复用。 |

### Usability

| 假设 | 风险 | 本轮处理 |
|------|------|----------|
| 开发者能理解"WSL 侧自动配置"的含义和影响范围 | 若用户以为 GoGuo 会修改所有 WSL 配置，会拒绝接入 | 配置前明确列出将被修改的具体项和值。 |

### Viability / 合规

| 假设 | 风险 | 本轮处理 |
|------|------|----------|
| WSL/Linux 侧自动配置不引入额外合规风险 | 若自动修改涉及系统文件权限提升，可能触发安全软件告警 | 配置操作遵循 OS 最小权限原则；审计记录所有写入操作。 |

## 7. 候选方向与排除项

### 当前轮候选方向

| 方向 | 说明 | 当前判断 |
|------|------|----------|
| S1：WSL/Linux 可恢复项自动配置 | 将代理环境变量、Git 代理、/etc/resolv.conf、/etc/environment 四个可恢复项从只读升级为自动配置和恢复 | 主路径，必须进入 Bridge to Spec。 |
| S2：三种部署组合的统一管理 | 仅 Windows / 仅 WSL / 协同部署三种组合下的状态项采集、配置和恢复逻辑统一化 | 主路径，必须进入 Bridge to Spec。 |
| S3：WSL2 网络模式感知策略 | 根据当前 WSL2 网络模式（NAT / 镜像）自动选择配置策略，减少不必要的 WSL 侧配置 | 主路径，必须进入 Bridge to Spec。 |
| S4：WSL/Linux 侧失败解释与手动接管 | 自动配置失败时提供与 Feature 001 一致的五要素失败提示 | 主路径，必须进入 Bridge to Spec。 |
| S5：WSL/Linux 侧审计与 Feature 001 审计统一 | WSL/Linux 侧配置和恢复操作记入同一审计流 | 主路径，必须进入 Bridge to Spec。 |

### 当前轮排除项

| 排除项 | 剪枝理由 |
|--------|----------|
| shell 配置代理（.bashrc/.zshrc/.profile）自动修改 | 可检测不可恢复项，修改风险高（可能影响用户其他 shell 配置），留给后续评估 |
| 包管理器代理配置（apt/pip/npm） | 场景碎片化，不属于本轮核心 wedge |
| Docker 代理配置 | 特定用户群，不属于当前用户主路径 |
| 独立 Linux 服务器（无 WSL）部署 | 当前轮聚焦 WSL 场景；纯 Linux 服务器部署作为远期假设 |
| 完整 Linux 桌面环境 GUI | 属于 OPP-003 的扩展范围 |
<!-- [?] id:01;status:close;date:2026-05-11T15:00  从计算机网络技术视角审视，确认这些排除项，是否影响最终开发者用户使用体验：GoGuo可运行在windows下，或者是WSL，二者选其一，但要求用户无论是在windows还是wsl下，都可以成功访问目标网站，用户不用关注wsl与windows的网络配置细节；任务处理结果：4个可恢复项（代理环境变量+/etc/environment+Git代理+/etc/resolv.conf）覆盖git/npm/pip/curl/wget/VS Code/agent工具等~90%主流开发工作流。apt不读环境变量、Docker独立配置是真实间隙但影响特定场景。shell rc覆盖属可检测可警告范围。排除项不影响核心wedge的用户体验目标，在§7排除项表格后补充覆盖度说明-->
>
> **排除项覆盖度评估**：4 个可恢复项（代理环境变量、Git 代理、/etc/resolv.conf、/etc/environment）构成的标准代理配置链覆盖 git、npm、pip、curl、wget、VS Code Remote、agent CLI 等主流开发者工作流。排除项中的 apt（不读环境变量）和 Docker（独立配置机制）是真实间隙，但影响系统包管理和容器场景而非核心开发工作流，可在后续迭代中根据用户反馈扩展。shell rc 代理覆盖属于"可检测不可恢复项"，GoGuo 可检测并警告冲突，不需要自动修改。
## 8. 建议 probe / 验证优先级

| 优先级 | Probe | 验证对象 | 成功信号 |
|--------|-------|----------|----------|
| P0 | WSL/Linux 可恢复项读写矩阵 | 代理环境变量、Git 代理、/etc/resolv.conf、/etc/environment 在主流 WSL 发行版上的读写可行性 | 每个状态项在 Ubuntu/Debian 上可稳定读写，写入后可恢复到 baseline。 |
| P0 | WSL2 NAT 与镜像模式行为差异 | 两种模式下 Windows 代理对 WSL 的影响 | 能明确两种模式下的配置策略差异和最小配置集。 |
| P1 | 仅 WSL 部署场景验证 | GoGuo 运行在 WSL 内，管理 WSL 侧网络状态 | 无 Windows 侧依赖也能完成采集、配置和恢复。 |
| P1 | 自动配置用户接受度 | 开发者是否接受 WSL 侧自动配置 | 3-5 个开发者样本能理解并接受自动配置的范围和风险。 |

## 9. 成功度量

- Desired Outcome：PC 端开发者在 Windows + WSL/Linux 协同环境中，无论从 Windows 还是 WSL 入口操作，都能获得一致的目标站点可达性，或明确的不可达诊断和修复建议。
- North Star 锚定：战略发现 Objective 2 "把现有原型包装成普通用户和开发者可操作的产品"，KR2.4 "Linux/WSL 开发入口可诊断"。
- Leading 指标：
  - WSL/Linux 可恢复项配置成功率。
  - 协同部署下两侧状态一致性比例。
  - WSL2 网络模式识别准确率。
- Lagging 指标：
  - 开发者在 WSL 侧因网络问题中断工作的频率。
- Success Threshold：
  - WSL/Linux 侧 4 个可恢复项（代理环境变量、Git 代理、/etc/resolv.conf、/etc/environment）100% 可配置和恢复。
  - 协同部署下 Windows 与 WSL 的代理状态差异在配置后消除。
  - WSL2 网络模式正确识别并根据模式选择适当策略。
  - 自动配置失败时提供与 Feature 001 一致的五要素失败提示。
  - 所有 WSL/Linux 侧操作记入统一审计流。
- Non-goal Metrics：
  - 不追求 shell 配置代理自动修改。
  - 不追求包管理器代理配置。
  - 不追求独立 Linux 服务器部署。

## 10. JTBD 视图

### Jobs Stories

- When 我在 WSL 中运行 git push 或 npm install 时遇到网络超时，I want GoGuo 自动将 WSL 代理配置调整为与 Windows 一致，so I can 不需要手动编辑 .bashrc 或设置环境变量。
- When 我切换了 WSL2 网络模式（NAT ↔ 镜像），I want GoGuo 自动识别模式变化并调整 WSL 侧配置策略，so I can 不需要理解两种模式下的代理行为差异。
- When GoGuo 无法自动配置 WSL 侧网络状态时，I want 看到具体的失败原因和可执行的修复步骤，so I can 手动修复并继续工作。

### 四力分析

| 四力 | 本轮表现 |
|------|----------|
| Push of the situation | WSL 侧网络问题频繁但排查困难，浏览器可用但 CLI 不可用的不对称体验降低信任。 |
| Pull of the new solution | 自动配置和恢复 WSL 侧网络状态，开发者无需理解代理技术细节。 |
| Anxiety of the new solution | 担心自动修改 WSL 配置文件影响其他工具或破坏自定义设置。 |
| Habit of the present | 开发者继续手动在 .bashrc 中设置代理、每次网络切换后手动更新环境变量。 |

## 11. OST Snapshot

Desired Outcome：PC 端开发者在协同环境中获得一致的目标站点可达性。

Opportunity A：WSL/Linux 可恢复项从只读评估升级为自动配置。

- Solution A1：基于 Feature 001 只读评估产出，自动配置 WSL/Linux 可恢复项。
  - Assumption：可恢复项在主流 WSL 发行版上可稳定读写。
  - Probe：WSL/Linux 可恢复项读写矩阵。
- Solution A2：WSL2 网络模式感知策略。
  - Assumption：镜像模式可简化或跳过 WSL 侧配置。
  - Probe：WSL2 NAT 与镜像模式行为差异验证。

Opportunity B：三种部署组合统一管理。

- Solution B1：状态项管理抽象层跨部署复用。
  - Assumption：Feature 001 的采集/配置/恢复逻辑可参数化为跨平台复用。
  - Probe：仅 WSL 部署场景验证。

## 12. Bridge to Spec

若本 discovery 通过 `hf-discovery-review`，建议进入 `hf-specify` 的范围如下。

### 推荐进入规格的范围边界

- WSL/Linux 侧 4 个可恢复项（代理环境变量、Git 代理、/etc/resolv.conf、/etc/environment）的自动配置与恢复。
- 三种部署组合（仅 Windows / 仅 WSL / 协同部署）下的统一管理逻辑。
- WSL2 网络模式（NAT / 镜像）感知策略。
- WSL/Linux 侧自动配置的二次确认和审计（与 Feature 001 统一）。
- WSL/Linux 侧失败解释（五要素，与 Feature 001 一致）。
- Feature 001 FR-2.9 交接边界的消费：只读评估产出作为本 feature 的输入。
- 配置前展示具体修改内容，配置后验证与 baseline 一致。

### 可直接转成规格输入的稳定结论

- WSL/Linux 侧可恢复项已在 Feature 001 中定义和分类，直接复用。
- 二次确认、审计、失败解释机制沿用 Feature 001 的设计。
- 部署组合识别逻辑已在 Feature 001 FR-2.9.1 中定义。
- CON-1（平台无关）适用于本 feature：baseline 形成中的用户确认调整和停止/异常时的恢复不在"自动网络配置接管"约束内。

### 需要继续保留为 assumption 的内容

| Assumption | 初始置信度 | 置信度来源 | 后续验证 |
|------------|------------|------------|----------|
| WSL 代理环境变量等可恢复项在主流发行版上可稳定读写 | 中 | Feature 001 已确认技术可行性，但未实际执行写入验证 | P0 读写矩阵 probe |
| WSL2 镜像模式下 Windows 代理对 WSL 自动生效 | 中 | WSL2 文档描述，但版本差异可能影响行为 | P0 模式差异 probe |
| 仅 WSL 部署下 GoGuo 可独立运行 | 低 | 现有原型基于 Windows，WSL 独立运行未经验证 | P1 仅 WSL 部署 probe |
| 开发者接受 WSL 侧自动配置 | 中 | 开发者对自动化的接受度通常高于普通用户，但代理配置敏感 | P1 用户接受度 probe |

### 当前不进入 spec 的候选项

- shell 配置代理（.bashrc/.zshrc/.profile）自动修改
- 包管理器代理配置
- Docker 代理配置
- 独立 Linux 服务器部署
- 完整 Linux 桌面 GUI

### 带入规格阶段的成功标准锚点

- WSL/Linux 侧 4 个可恢复项 100% 可配置和恢复。
- 协同部署下配置后两侧状态一致。
- WSL2 网络模式正确识别并选择适当策略。
- 自动配置失败提供五要素失败提示。
- 所有 WSL/Linux 侧操作记入统一审计。

## 13. 开放问题

### 阻塞项

| 问题 | 为什么阻塞 | 建议处理 |
|------|------------|----------|
| WSL/Linux 可恢复项写入路径是否在主流发行版上稳定可用 | 若写入失败，自动配置承诺不成立 | P0 probe 验证 Ubuntu/Debian 写入路径 |

### 非阻塞项

| 问题 | 说明 |
|------|------|
| 是否需要支持更多 WSL 发行版（Arch、Fedora 等） | 当前轮聚焦 Ubuntu/Debian，其他发行版作为后续扩展 |
| 仅 WSL 部署下 GoGuo 的安装和运行方式 | 属于设计阶段决定 |
| WSL2 镜像模式是否完全不需要额外配置 | 需要 probe 验证，可能存在边缘场景 |

## 14. 评审前自检

- [x] 上游前置条件已满足：Feature 001（OPP-002）规格已确认。
- [x] 当前轮只收敛 `OPP-004`，未把 `OPP-001` 合并成同一份 discovery。
- [x] 已明确问题、用户、why-now、wedge 和当前不做的候选项。
- [x] 已区分已确认事实、假设与风险。
- [x] 已写出 Desired Outcome、Success Threshold、North Star 锚定和 Non-goal Metrics。
- [x] 已包含 Jobs Stories、四力分析和 OST Snapshot。
- [x] 已写出 Bridge to Spec，但未进入 formal spec、design 或 tasks。
- [x] 已明确与 Feature 001 FR-2.9 的交接关系。
- [x] 下一建议节点已标记为 `hf-discovery-review`。
