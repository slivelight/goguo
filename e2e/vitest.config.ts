import { defineConfig } from "vitest/config";

/**
 * F115 T-03 / T-04b: e2e helpers L1 单测配置（OQ-7）
 *
 * 范围：仅测 e2e/helpers/*.ts，排除 e2e/specs/**（spec 由 wdio 跑，不归 vitest）。
 *
 * 覆盖率范围说明（FR-2.2.4-R3）：
 *   - env.ts / state.ts：纯 Node 代码，由 vitest L1 单测覆盖（本配置）
 *   - tauri-ipc.ts / wait.ts：依赖 wdio `browser` 全局，L1 无法覆盖，
 *     由 L2/L3 e2e spec（smoke.spec.ts / ipc.spec.ts）覆盖
 *   因此 coverage.include 只列 env.ts + state.ts，避免 false-negative
 */
export default defineConfig({
  test: {
    include: ["helpers/**/*.test.ts"],
    exclude: ["specs/**", "node_modules/**", "test/**"],
    coverage: {
      provider: "v8",
      reporter: ["text", "html"],
      include: ["helpers/env.ts", "helpers/state.ts"],
      thresholds: {
        lines: 80,
        functions: 80,
        branches: 80,
        statements: 80,
      },
    },
    environment: "node", // helpers 是纯 Node 代码（readFileSync / process.env）
  },
});
