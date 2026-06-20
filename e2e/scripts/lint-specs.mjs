#!/usr/bin/env node
/**
 * F115 FR-2.4.2-R1: e2e 接入规范 lint。
 *
 * 校验项（4 条）：
 *   1. 所有 spec 在 specs/f<NNN>-<slug>/ 目录下（路径结构）
 *   2. 所有 spec 顶部 describe() 含 Feature ID（F\d{3} 模式，不要求与目录名一致）
 *   3. 所有 spec 代码层不直接使用 __TAURI_INTERNALS__（JSDoc 注释允许）
 *   4. 所有 spec 不直接 import @tauri-apps/api/core（须走 helpers/tauri-ipc）
 *
 * 运行方式：
 *   pnpm --filter e2e lint
 *   node e2e/scripts/lint-specs.mjs   # 从任意 cwd 均可（自动 chdir 到 e2e/）
 *
 * 退出码：0 = 全过，1 = 有违规
 */
import { globSync, readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

// cwd 无关：始终以脚本所在 e2e/ 为基准
const E2E_DIR = resolve(dirname(fileURLToPath(import.meta.url)), "..");
process.chdir(E2E_DIR);

const SPECS_GLOB = "specs/**/*.spec.ts";
const specs = globSync(SPECS_GLOB);

if (specs.length === 0) {
  console.error(`[lint-specs] ❌ 未找到 spec 文件 (glob=${SPECS_GLOB}, cwd=${process.cwd()})`);
  process.exit(1);
}

/** 剥离 JS/TS 注释（块注释 + 行注释），仅校验实际代码 */
const stripComments = (src) =>
  src
    .replace(/\/\*[\s\S]*?\*\//g, "") // 块注释 /* ... */
    .replace(/\/\/.*$/gm, ""); // 行注释 // ...

const errors = [];

for (const spec of specs) {
  const content = readFileSync(spec, "utf-8");
  const code = stripComments(content);

  // 校验 1: 目录结构 specs/f<NNN>-<slug>/
  if (!/specs\/(f\d{3}-[\w-]+)\//.test(spec)) {
    errors.push(`${spec}: ❌ 不在 specs/f<NNN>-<slug>/ 目录下`);
  }

  // 校验 2: describe() 含 Feature ID（F\d{3}，大小写不敏感）
  const describeMatch = code.match(/describe\(\s*["']([^"']+)["']/);
  if (!describeMatch) {
    errors.push(`${spec}: ❌ 未找到 describe() 调用`);
  } else if (!/F\d{3}/i.test(describeMatch[1])) {
    errors.push(
      `${spec}: ❌ describe "${describeMatch[1]}" 不含 Feature ID（期望 F\\d{3} 模式，如 "F115: ..."）`,
    );
  }

  // 校验 3: 代码层不直接使用 __TAURI_INTERNALS__
  if (code.includes("__TAURI_INTERNALS__")) {
    errors.push(
      `${spec}: ❌ 代码直接使用 __TAURI_INTERNALS__，应改为 import { invokeTauri } from "../../helpers/tauri-ipc"`,
    );
  }

  // 校验 4: 不直接 import @tauri-apps/api/core（须走 helpers）
  if (/from\s+["']@tauri-apps\/api\/core["']/.test(code)) {
    errors.push(
      `${spec}: ❌ 直接 import @tauri-apps/api/core，须改为 import { invokeTauri } from "../../helpers/tauri-ipc"`,
    );
  }
}

if (errors.length) {
  console.error(`[lint-specs] ❌ ${errors.length} 个违规（${specs.length} specs 中）：\n` + errors.join("\n"));
  process.exit(1);
}

console.log(`[lint-specs] ✅ ${specs.length} specs 全部合规`);
