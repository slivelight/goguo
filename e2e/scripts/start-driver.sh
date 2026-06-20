#!/usr/bin/env bash
# F115 FR-2.2.2-R2: tauri-driver 本地常驻启动脚本（仅 dev 模式适用）。
#
# 使用场景（spec FR-2.2.2-R4）：
#   - 日常开发：先跑本脚本预启 tauri-driver，再 `TAURI_DRIVER_REUSE=1 pnpm test`，
#     每次跑省 ~8s（FR-2.2.2-R3）。
#   - 首次/CI 验证：不跑本脚本，让 @wdio/tauri-service 自启（隔离验证）。
#
# 幂等：端口已监听则直接 exit 0（spec FR-2.2.2-R2 验收点）。
# pid 文件：/tmp/tauri-driver.pid（与 design.md §2.2.3 一致）。
set -euo pipefail

PORT="${TAURI_DRIVER_PORT:-4444}"
NATIVE_PORT="${TAURI_DRIVER_NATIVE_PORT:-4445}"
PID_FILE="${TAURI_DRIVER_PID_FILE:-/tmp/tauri-driver.pid}"

# 端口已监听 → 幂等退出
if ss -ltn 2>/dev/null | grep -q ":${PORT}\b"; then
  echo "[start-driver] tauri-driver already listening on ${PORT} (skip spawn)"
  exit 0
fi

# 校验 tauri-driver 可用
if ! command -v tauri-driver >/dev/null 2>&1; then
  echo "[start-driver] ERROR: tauri-driver not in PATH. Install with: cargo install tauri-driver" >&2
  exit 1
fi

# spawn（后台 + nohup，确保本脚本退出后仍存活）
nohup tauri-driver \
  --port "${PORT}" \
  --native-port "${NATIVE_PORT}" \
  > /tmp/tauri-driver.log 2>&1 &

DRIVER_PID=$!
echo "${DRIVER_PID}" > "${PID_FILE}"
echo "[start-driver] tauri-driver started on port ${PORT} (native ${NATIVE_PORT}, pid ${DRIVER_PID})"
echo "[start-driver] log: /tmp/tauri-driver.log"

# 等端口就绪（最多 10s）
for _ in $(seq 1 20); do
  if ss -ltn 2>/dev/null | grep -q ":${PORT}\b"; then
    echo "[start-driver] port ${PORT} ready"
    exit 0
  fi
  sleep 0.5
done

echo "[start-driver] ERROR: tauri-driver did not start listening within 10s" >&2
echo "[start-driver] see /tmp/tauri-driver.log for details" >&2
exit 1
