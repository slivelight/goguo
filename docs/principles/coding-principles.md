# coding-principles

## 代码风格
- 格式与静态检查严格遵循 `rustfmt.toml` 与 `Cargo.toml` 顶部 `[lints.clippy]`
- 提交前强制执行：`cargo fmt --all` 与 `cargo clippy --all-targets --all-features -- -D warnings`
- 详细配置见项目根目录：`rustfmt.toml`、`rust-toolchain.toml`

## 语言版本与依赖管理
- 工具链锁定于 `rust-toolchain.toml`，禁止依赖全局环境
- 依赖统一使用 `^x.y.z` 语义化版本，严格提交 `Cargo.lock`，禁用 `*`

### 1. 版本声明策略
- **统一前缀**：所有依赖使用 `^` 语义化版本（如 `serde = "^1.0.210"`），禁止 `*` 或裸版本号。
- **主版本对齐**：Tauri 核心包（`tauri`, `tauri-build`）必须同主版本；前端 `@tauri-apps/*` npm 包需与 Rust 侧保持相同大版本。
- **依赖分类**：严格区分 `[dependencies]`、`[dev-dependencies]` 与 `[build-dependencies]`，避免构建产物污染。

### 2. `Cargo.lock` 管理
- **强制提交**：Tauri 为应用级项目，`Cargo.lock` 必须纳入 Git，确保多端/CI 构建一致性。
- **冲突处理**：合并冲突时优先运行 `cargo update` 重新生成，禁止手动编辑 `Cargo.lock`。

### 3. 更新与排查工作流
- **安全预览**：`cargo update --dry-run` 查看变更，确认无破坏性升级后再执行 `cargo update`。
- **精准控制**：使用 `cargo update -p <crate>` 升级单一依赖，避免连锁更新。
- **重复依赖**：`cargo tree --duplicates` 定位多版本共存包，通过 `[patch]` 或统一上游版本收敛。
- **闲置清理**：定期运行 `cargo machete` 或依赖 IDE 提示，移除 `Cargo.toml` 中未实际使用的 crate。

### 4. 安全与合规
- **漏洞扫描**：CI 集成 `cargo audit`，拦截 RUSTSEC 通告的高危依赖。
- **许可证审查**：引入新 crate 前核对 `SPDX` 标识（优先 MIT/Apache-2.0/BSL），禁止引入 GPL/AGPL 等传染性协议。
- **供应链安全**：优先使用 `crates.io` 官方源，避免直引 Git 分支（除非提交经官方合并的 PR）。

### 5. Tauri 专项注意
- **ABI 兼容性**：`tauri`、`wry`、`tao` 等底层原生绑定升级前，务必核对官方 Release Notes 的 Breaking Changes。
- **插件版本**：官方插件（`tauri-plugin-*`）与核心 `tauri` 版本严格绑定，不可混用不同代际版本。
- **构建缓存**：频繁变更依赖会触发全量重编译，建议 CI 配置缓存 `~/.cargo/registry` 与 `target/` 目录。
