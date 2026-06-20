#!/usr/bin/env bash
# F115 FR-2.2.4 / SC-2：5 次连跑度量脚本。
#
# 用法：
#   ./scripts/benchmark.sh                # 默认 5 次，自启模式（SC-2 适用场景）
#   ./scripts/benchmark.sh --runs 3       # 自定义次数
#   ./scripts/benchmark.sh --reuse        # 复用模式（不走 SC-2 阈值）
#   ./scripts/benchmark.sh --runs 5 --reuse
#
# 输出：
#   1. stdout：每次耗时 + 均值 + 标准差
#   2. features/115-ux-e2e-infrastructure/evidence/benchmark-M4.md：追加 markdown 表格
#
# SC-2 阈值（仅自启模式）：5 次均值 ≤ 70s，否则 exit 1
set -euo pipefail

# ---- 参数解析 ----
RUNS=5
REUSE_MODE=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --runs) RUNS="$2"; shift 2 ;;
    --reuse) REUSE_MODE=1; shift ;;
    -h|--help)
      sed -n '2,15p' "$0"; exit 0 ;;
    *) echo "[benchmark] unknown arg: $1" >&2; exit 2 ;;
  esac
done

if [[ "${RUNS}" -lt 1 ]]; then
  echo "[benchmark] --runs must be ≥ 1" >&2; exit 2
fi

# ---- 路径 ----
E2E_DIR="$(cd "$(dirname "$0")/.." && pwd)"
FEATURE_EVIDENCE_DIR="$(cd "$(dirname "$0")/../../features/115-ux-e2e-infrastructure" && pwd)/evidence"
EVIDENCE_FILE="${FEATURE_EVIDENCE_DIR}/benchmark-M4.md"
mkdir -p "${FEATURE_EVIDENCE_DIR}"

cd "${E2E_DIR}"

# ---- 元信息 ----
TIMESTAMP="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
GIT_SHA="$(git -C "${E2E_DIR}" rev-parse --short HEAD 2>/dev/null || echo 'unknown')"
MODE_LABEL=$([[ "${REUSE_MODE}" -eq 1 ]] && echo "reuse" || echo "self-spawn")

echo "[benchmark] mode=${MODE_LABEL}, runs=${RUNS}, ts=${TIMESTAMP}, sha=${GIT_SHA}"

# ---- 复用模式 driver 预启 ----
if [[ "${REUSE_MODE}" -eq 1 ]]; then
  ./scripts/start-driver.sh
  trap './scripts/stop-driver.sh' EXIT
fi

# ---- 跑 N 次 ----
DURATIONS=()
PASS_FAIL=()
for i in $(seq 1 "${RUNS}"); do
  echo "----- run ${i}/${RUNS} -----"
  START=$(date +%s.%N)
  if [[ "${REUSE_MODE}" -eq 1 ]]; then
    RUN_LOG=$(TAURI_DRIVER_REUSE=1 pnpm test 2>&1) || RUN_LOG_SET_FAIL=1
  else
    RUN_LOG=$(pnpm test 2>&1) || RUN_LOG_SET_FAIL=1
  fi
  END=$(date +%s.%N)

  DUR=$(awk -v s="${START}" -v e="${END}" 'BEGIN{printf "%.2f", e-s}')
  DURATIONS+=("${DUR}")

  if echo "${RUN_LOG}" | grep -qE "Spec Files:\s+[1-9] passed"; then
    PASS_FAIL+=("PASS")
  else
    PASS_FAIL+=("FAIL")
  fi
  TESTS_COUNT=$(echo "${RUN_LOG}" | grep -oE '[0-9]+ passing' | head -1 || echo "0 passing")
  echo "[benchmark] run ${i}: ${DUR}s ${PASS_FAIL[-1]} (${TESTS_COUNT})"
done

# ---- 统计（awk 算 mean + stddev）----
STATS=$(awk -v runs="${RUNS}" -v arr="${DURATIONS[*]}" '
  BEGIN {
    n = split(arr, a, " ")
    sum = 0; sumsq = 0
    for (i = 1; i <= n; i++) { sum += a[i]; sumsq += a[i]*a[i] }
    mean = sum / n
    if (n > 1) {
      var = (sumsq - n*mean*mean) / (n-1)
      sd = (var > 0) ? sqrt(var) : 0
    } else { sd = 0 }
    printf "%.2f %.2f", mean, sd
  }
')
MEAN=$(echo "${STATS}" | awk '{print $1}')
STDDEV=$(echo "${STATS}" | awk '{print $2}')

echo "----- summary -----"
echo "[benchmark] runs=${RUNS}, mode=${MODE_LABEL}"
echo "[benchmark] mean=${MEAN}s, stddev=${STDDEV}s"
echo "[benchmark] durations: ${DURATIONS[*]}"

# ---- SC-2 阈值检查（仅自启模式）----
EXIT_CODE=0
if [[ "${REUSE_MODE}" -eq 0 ]]; then
  SC2_THRESHOLD=70
  SC2_RESULT=$(awk -v m="${MEAN}" -v t="${SC2_THRESHOLD}" 'BEGIN{print (m <= t) ? "PASS" : "FAIL"}')
  echo "[benchmark] SC-2 (self-spawn mean ≤ ${SC2_THRESHOLD}s): ${SC2_RESULT}"
  if [[ "${SC2_RESULT}" != "PASS" ]]; then
    EXIT_CODE=1
  fi
fi

# ---- 追加 markdown 表格到 evidence ----
# 表格行：每次一行
ROWS=""
for i in $(seq 1 "${RUNS}"); do
  ROWS+="| ${i} | ${MODE_LABEL} | ${DURATIONS[$((i-1))]} | ${PASS_FAIL[$((i-1))]} | ${TIMESTAMP} | ${GIT_SHA} |\n"
done

# 追加（不覆盖）：文件不存在则加 header
if [[ ! -f "${EVIDENCE_FILE}" ]]; then
  cat > "${EVIDENCE_FILE}" <<'EOF'
# F115 M4 Benchmark Evidence

来源：`e2e/scripts/benchmark.sh`（FR-2.2.4 / SC-2）。

## 单次记录

| 序号 | 模式 | 耗时(s) | 结果 | 时间戳(UTC) | git sha |
|------|------|--------|------|------------|---------|
EOF
fi

printf "%b" "${ROWS}" >> "${EVIDENCE_FILE}"

# 追加 summary 块（分隔一次完整 benchmark run）
cat >> "${EVIDENCE_FILE}" <<EOF

### run summary @ ${TIMESTAMP} (sha ${GIT_SHA})
- 模式: ${MODE_LABEL}, 次数: ${RUNS}
- 均值: **${MEAN}s** | 标准差: ${STDDEV}s
- SC-2 阈值（仅自启模式）: $([[ "${REUSE_MODE}" -eq 0 ]] && echo "${SC2_RESULT} (≤ ${SC2_THRESHOLD}s)" || echo "N/A (reuse mode)")

EOF

echo "[benchmark] evidence appended: ${EVIDENCE_FILE}"
exit "${EXIT_CODE}"
