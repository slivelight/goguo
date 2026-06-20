#!/usr/bin/env node
/**
 * F115 T-05: e2e 单 Feature 测试入口（FR-2.1.3-R5/R6）
 *
 * 用法：pnpm test:e2e:feature -- <feature-id>
 *   （由根 package.json 转发：cd e2e && pnpm test:e2e:feature -- <id>）
 *
 * 行为：
 *   1. 接收 feature-id，校验格式
 *   2. 检查 e2e/specs/<feature-id>/ 是否存在
 *      - 不存在：exit 0 + 提示 SKIP（允许 feature 无 e2e spec）
 *      - 存在：spawn wdio run --spec specs 子目录下所有 spec 文件
 *   3. 透传退出码
 */

import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";

// pnpm forwarding behavior: `pnpm <script> -- <arg>` may pass both `--` and `<arg>`
// to the spawned script. Filter out leading `--` / flag-like tokens.
const positional = process.argv.slice(2).filter((a) => !a.startsWith("--"));
const featureId = positional[0];

if (!featureId) {
  console.error("Usage: pnpm test:e2e:feature -- <feature-id>");
  console.error("Examples: f114-baseline, f201-first-run");
  process.exit(1);
}

const normalized = featureId.toLowerCase();
if (!/^f\d{3}(-[a-z0-9-]+)?$/.test(normalized)) {
  console.error(`[test:e2e:feature] Invalid feature-id: ${featureId}`);
  console.error("  Expected: f<NNN> or f<NNN>-<slug>");
  process.exit(1);
}

const specDir = `specs/${normalized}`;
if (!existsSync(specDir)) {
  console.log(`[test:e2e:feature] SKIP: ${specDir}/ not found (feature has no e2e spec)`);
  process.exit(0);
}

const specGlob = `specs/${normalized}/**/*.spec.ts`;
console.log(`[test:e2e:feature] wdio run ./wdio.conf.ts --spec ${specGlob}`);

const result = spawnSync(
  "pnpm",
  ["exec", "wdio", "run", "./wdio.conf.ts", "--spec", specGlob],
  { stdio: "inherit" },
);

process.exit(result.status ?? 1);
