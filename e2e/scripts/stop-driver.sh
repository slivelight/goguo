#!/usr/bin/env bash
# F115 FR-2.2.2-R2: tauri-driver 停止脚本（仅 dev 模式适用）。
#
# 与 start-driver.sh 配套，读取 /tmp/tauri-driver.pid kill 进程并清理。
# 若 pid 文件不存在但端口仍在监听，则 fallback 按端口查 pid kill（防孤儿）。
set -euo pipefail

PORT="${TAURI_DRIVER_PORT:-4444}"
PID_FILE="${TAURI_DRIVER_PID_FILE:-/tmp/tauri-driver.pid}"

KILLED=0

# 主路径：读 pid 文件 kill
if [[ -f "${PID_FILE}" ]]; then
  PID="$(cat "${PID_FILE}" 2>/dev/null || true)"
  if [[ -n "${PID}" ]] && kill -0 "${PID}" 2>/dev/null; then
    kill "${PID}" 2>/dev/null || true
    # 等最多 3s 让进程优雅退出
    for _ in $(seq 1 6); do
      kill -0 "${PID}" 2>/dev/null || break
      sleep 0.5
    done
    # 还活着 → 强杀
    if kill -0 "${PID}" 2>/dev/null; then
      kill -9 "${PID}" 2>/dev/null || true
    fi
    echo "[stop-driver] killed pid ${PID}"
    KILLED=1
  fi
  rm -f "${PID_FILE}"
fi

# Fallback：端口仍在监听则按端口查 pid kill（防 start-driver.sh 异常后留下的孤儿）
if ss -ltn 2>/dev/null | grep -q ":${PORT}\b"; then
  ORPHAN_PIDS="$(ss -ltnp 2>/dev/null | grep ":${PORT}\b" | grep -oE 'pid=[0-9]+' | cut -d= -f2 | sort -u || true)"
  if [[ -n "${ORPHAN_PIDS}" ]]; then
    for OPID in ${ORPHAN_PIDS}; do
      kill "${OPID}" 2>/dev/null || true
      echo "[stop-driver] killed orphan pid ${OPID} on port ${PORT}"
      KILLED=1
    done
  fi
fi

if [[ "${KILLED}" -eq 0 ]]; then
  echo "[stop-driver] no tauri-driver running (port ${PORT} not listening, no pid file)"
fi
