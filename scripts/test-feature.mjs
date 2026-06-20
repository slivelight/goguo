#!/usr/bin/env node
/**
 * F115 T-05: 单 Feature 三层测试编排（FR-2.1.3-R4）
 *
 * 用法：
 *   pnpm test:feature -- <feature-id>          # 3 层全跑（cargo + vitest + e2e）
 *   pnpm test:e2e:feature -- <feature-id>      # 仅 e2e 层（转发到 e2e/scripts/）
 *
 * feature-id 格式：
 *   - 短形式：f114（匹配 f114_* cargo 模块 + 路径含 f114 的测试文件）
 *   - 长形式：f114-baseline（e2e 层用此形式定位 specs/f114-baseline/）
 *
 * 三层职责：
 *   - L1+L2+L3 后端：cargo test --workspace -- <numeric-prefix>
 *     （substring 过滤测试全限定名，如 fr_acceptance::f003_site_rules::xxx）
 *   - L4 前端：vitest run <glob> --passWithNoTests
 *     （glob 形如 src/__tests__ 子路径含 numeric-prefix 的文件）
 *   - L5 e2e：cd e2e && pnpm test:e2e:feature -- <feature-id>
 *     （转发到 e2e/scripts/test-e2e-feature.mjs，由其处理 specs/<id>/ 路径）
 *
 * 容错策略：
 *   - cargo/vitest 层"无测试匹配"不算失败（--passWithNoTests + cargo 自然行为）
 *   - e2e 层 specs/<feature-id>/ 不存在 → SKIP（不阻塞 cargo/vitest 层）
 */

import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";

const args = process.argv.slice(2);
const e2eOnly = args.includes("--e2e-only");
const positional = args.filter((a) => !a.startsWith("--"));
const featureId = positional[0];

if (!featureId) {
  console.error("Usage: pnpm test:feature -- <feature-id>");
  console.error("       pnpm test:e2e:feature -- <feature-id>");
  console.error("Examples: f114, f114-baseline, f201, f201-first-run");
  process.exit(1);
}

// Normalize: lowercase, validate format
const normalized = featureId.toLowerCase();
const formatRe = /^f(\d{3})(-[a-z0-9-]+)?$/;
if (!formatRe.test(normalized)) {
  console.error(`[test:feature] Invalid feature-id: ${featureId}`);
  console.error("  Expected: f<NNN> or f<NNN>-<slug> (e.g. f114, f114-baseline)");
  process.exit(1);
}

// Numeric prefix for cargo/vitest filter (f114-baseline → f114)
const numericPrefix = normalized.match(/^f\d{3}/)[0];

console.log(
  `[test:feature] Running ${e2eOnly ? "e2e-only" : "3-layer"} tests for ${normalized}` +
    (numericPrefix !== normalized ? ` (filter: ${numericPrefix})` : "") +
    "...",
);

/** @type {{name: string; status: number; skipped?: boolean}[]} */
const layers = [];

/**
 * Run cargo layer (L1+L2+L3 backend).
 * Filter by substring on test fully-qualified name.
 */
function runCargo() {
  console.log(`\n[L1+L2+L3] cargo test --workspace -- ${numericPrefix}`);
  const result = spawnSync(
    "cargo",
    ["test", "--manifest-path", "src-tauri/Cargo.toml", "--", numericPrefix],
    { stdio: "inherit" },
  );
  layers.push({
    name: "cargo (L1+L2+L3 backend)",
    status: result.status ?? 1,
  });
}

/**
 * Run vitest layer (L4 frontend).
 * Filter by file path glob; --passWithNoTests makes "0 matches" exit 0.
 */
function runVitest() {
  console.log(`\n[L4] vitest run src/__tests__/**/*${numericPrefix}* --passWithNoTests`);
  const result = spawnSync(
    "pnpm",
    [
      "exec",
      "vitest",
      "run",
      `src/__tests__/**/*${numericPrefix}*`,
      "--passWithNoTests",
    ],
    { stdio: "inherit" },
  );
  layers.push({ name: "vitest (L4 frontend)", status: result.status ?? 1 });
}

/**
 * Run e2e layer (L5). Delegates to e2e/scripts/test-e2e-feature.mjs via
 * `pnpm test:e2e:feature`. The e2e script handles specs/<id>/ existence.
 */
function runE2E() {
  const specDir = `e2e/specs/${normalized}`;
  if (!existsSync(specDir)) {
    console.log(`\n[L5] e2e: SKIP (${specDir} not found, graceful degradation)`);
    layers.push({ name: "e2e (L5)", status: 0, skipped: true });
    return;
  }
  console.log(`\n[L5] cd e2e && pnpm test:e2e:feature -- ${normalized}`);
  const result = spawnSync(
    "pnpm",
    ["test:e2e:feature", "--", normalized],
    { cwd: "e2e", stdio: "inherit" },
  );
  layers.push({ name: "e2e (L5)", status: result.status ?? 1 });
}

if (!e2eOnly) {
  runCargo();
  runVitest();
}
runE2E();

// Summary
console.log("\n[test:feature] Summary:");
for (const l of layers) {
  const flag = l.skipped ? "SKIP" : l.status === 0 ? "PASS" : "FAIL";
  console.log(`  ${l.name}: ${flag}`);
}

const failed = layers.filter((l) => !l.skipped && l.status !== 0);
process.exit(failed.length > 0 ? 1 : 0);
