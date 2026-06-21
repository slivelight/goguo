#!/usr/bin/env bash
# F115 FR-2.5.2-R2 / T-19: 开发环境首次配置一键脚本
#
# 处理 GAP-F115-2 的开发态缓解（mihomo config 阻断 cargo/pnpm 流量）。
# 根因修复推到 F116+（详见 features/110-design-gap-closure/design.md §12.2）。
#
# 平台适用性（id:05 周边，design §6.1 顶部）：
#   - WSL2 / Linux：适用（mihomo 阻断影响）
#   - macOS / Windows：不适用（直连 crates.io / npmjs.org 可达）
#
# 用法：
#   bash e2e/scripts/setup-dev-env.sh
#
# 幂等：重复执行不产生重复写入（通过 grep 检查已存在的镜像配置）。
set -euo pipefail

# ---- 平台检测（仅 WSL2 / Linux 执行）----
detect_platform() {
  local kernel os_release
  kernel="$(uname -s)"
  if [[ "$kernel" == "Darwin" ]]; then
    echo "[setup-dev-env] SKIP: macOS 不适用（直连 crates.io / npmjs.org 可达）"
    exit 0
  fi
  if [[ "$kernel" == "MINGW"* || "$kernel" == "MSYS"* || "$kernel" == "CYGWIN"* ]]; then
    echo "[setup-dev-env] SKIP: Windows 不适用（直连 crates.io / npmjs.org 可达）"
    exit 0
  fi
  if [[ "$kernel" != "Linux" ]]; then
    echo "[setup-dev-env] SKIP: 未识别平台 $kernel，跳过"
    exit 0
  fi
  # Linux：进一步检测 WSL（用于日志提示，不改变行为）
  os_release="$(grep -l microsoft /proc/version 2>/dev/null || true)"
  if [[ -n "$os_release" ]]; then
    echo "[setup-dev-env] 平台: WSL2 (Linux kernel + microsoft 标记)"
  else
    echo "[setup-dev-env] 平台: 原生 Linux"
  fi
}

# ---- 路径 ----
E2E_DIR="$(cd "$(dirname "$0")/.." && pwd)"
CARGO_CONFIG="${CARGO_HOME:-$HOME/.cargo}/config.toml"
E2E_NPMRC="${E2E_DIR}/.npmrc"

# ---- 步骤 1: cargo 镜像配置（rsproxy-sparse，幂等）----
configure_cargo_mirror() {
  echo "[setup-dev-env] 步骤 1/3: 检查 cargo 镜像配置..."

  local mirror_block='# F115 setup-dev-env.sh 自动添加（rsproxy 镜像，绕过 mihomo 阻断）
[source.rsproxy-sparse]
registry = "sparse+https://rsproxy.cn/index/"

[source.crates-io]
registry = "sparse+https://index.crates.io/"
replace-with = "rsproxy-sparse"
'

  if [[ ! -f "$CARGO_CONFIG" ]]; then
    mkdir -p "$(dirname "$CARGO_CONFIG")"
    printf '%s\n' "$mirror_block" > "$CARGO_CONFIG"
    echo "[setup-dev-env]   [OK] 已创建 $CARGO_CONFIG（rsproxy-sparse 镜像）"
    return
  fi

  # 宽容检测：用户的 cargo config 可能含 rsproxy / rsproxy-sparse / replace-with 任一形式
  if grep -qE 'rsproxy|replace-with\s*=\s*"rsproxy' "$CARGO_CONFIG"; then
    echo "[setup-dev-env]   [SKIP] $CARGO_CONFIG 已含 rsproxy 镜像配置"
    return
  fi

  # 文件存在但无镜像配置：追加（不覆盖用户已有内容）
  {
    echo ""
    echo "$mirror_block"
  } >> "$CARGO_CONFIG"
  echo "[setup-dev-env]   [OK] 已追加 rsproxy 镜像配置到 $CARGO_CONFIG"
}

# ---- 步骤 2: e2e/.npmrc 校验（C-I4 强制项）----
verify_npmrc() {
  echo "[setup-dev-env] 步骤 2/3: 校验 e2e/.npmrc (C-I4 隔离策略)..."

  if [[ ! -f "$E2E_NPMRC" ]]; then
    echo "[setup-dev-env]   [ERROR] $E2E_NPMRC 不存在"
    echo "[setup-dev-env]   C-I4 隔离策略要求 e2e 目录有独立 .npmrc 指向 npmmirror.com"
    echo "[setup-dev-env]   修复：创建该文件并写入 'registry=https://registry.npmmirror.com/'"
    return 1
  fi

  if ! grep -q "npmmirror" "$E2E_NPMRC"; then
    echo "[setup-dev-env]   [WARN] $E2E_NPMRC 未含 npmmirror 配置，可能 pnpm 仍走 npmjs.org 直连"
    echo "[setup-dev-env]   建议：检查该文件内容是否包含 'registry=https://registry.npmmirror.com/'"
    return
  fi

  echo "[setup-dev-env]   [OK] e2e/.npmrc 校验通过（含 npmmirror）"
}

# ---- 步骤 3: cargo 网络可达验证 ----
verify_cargo_network() {
  echo "[setup-dev-env] 步骤 3/3: 验证 cargo 网络可达..."

  if ! command -v cargo >/dev/null 2>&1; then
    echo "[setup-dev-env]   [WARN] cargo 未安装，跳过网络验证"
    return
  fi

  # --dry-run 只解析依赖元数据不发请求，但会触发对 registry 索引的 fetch
  # 若镜像未生效会报 SSL_ERROR_SYSCALL / network error
  local dry_run_output
  if ! dry_run_output="$(cargo install tauri-driver --dry-run 2>&1)"; then
    if echo "$dry_run_output" | grep -qiE "SSL_ERROR|network|failed to fetch|could not connect"; then
      echo "[setup-dev-env]   [WARN] cargo 仍报网络错误："
      echo "$dry_run_output" | grep -iE "SSL_ERROR|network|failed to fetch|could not connect" | sed 's/^/    /'
      echo "[setup-dev-env]   请检查 mihomo 是否运行中，或 ~/.cargo/config.toml 镜像是否生效"
      return
    fi
    # 非 network 错误（如 tauri-driver 不存在）：视为可达
    echo "[setup-dev-env]   [OK] cargo 网络可达（dry-run 报非网络错误，镜像索引可访问）"
    return
  fi

  echo "[setup-dev-env]   [OK] cargo 网络正常（tauri-driver --dry-run 成功）"
}

# ---- 主流程 ----
main() {
  detect_platform
  echo "[setup-dev-env] e2e 目录: $E2E_DIR"
  echo "[setup-dev-env] cargo config: $CARGO_CONFIG"
  echo ""

  configure_cargo_mirror
  verify_npmrc
  verify_cargo_network

  echo ""
  echo "[setup-dev-env] 配置完成。可运行 'pnpm test:e2e' 或 'cd e2e && pnpm test'。"
}

main "$@"
