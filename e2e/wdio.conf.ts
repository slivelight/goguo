import path from "node:path";

import { ensureX11Backend, getTauriDriverPort, shouldReuseDriver } from "./helpers/env";

const GOGUO_BIN = process.env.GOGUO_BIN
  ?? path.resolve(import.meta.dirname, "..", "target", "release", "goguo");

/**
 * F115 FR-2.2.2-R1（v3 勘误-3，2026-06-20）：双模式配置。
 *
 * 原设计试图给 @wdio/tauri-service 传 `skipDriverSpawn: true`，实测 v1.1.0 不支持。
 * 改走"绕开 service"路径：
 *   - 自启模式（默认）：services 含 @wdio/tauri-service，由其全权 spawn + 管理 tauri-driver
 *   - 复用模式（TAURI_DRIVER_REUSE=1）：services 为空，顶层 hostname/port 直连外部
 *     tauri-driver（由 e2e/scripts/start-driver.sh 预启）
 *
 * helpers 全用 browser.execute() 调 window.__TAURI_INTERNALS__.invoke()，不依赖
 * service 专属 API（browser.tauri.execute / mock store），绕开无功能损失。
 */
const reuseDriver = shouldReuseDriver();
const driverPort = getTauriDriverPort();

export const config: WebdriverIO.Config = {
  runner: "local",
  maxInstances: 1,

  // F115 FR-2.2.1-R1：wdio v9 移除了 `restart` 字段。
  // session 复用通过 specs 双层嵌套实现：`[[...]]` 让多个 spec 文件共享同一 worker
  // process + 同一 WebDriver session。
  specs: [["./specs/**/*.spec.ts"]],

  // 复用模式：直连外部 tauri-driver，不引入 service（v3 勘误-3）
  ...(reuseDriver
    ? {
        services: [] as WebdriverIO.ServiceEntry[],
        hostname: "127.0.0.1",
        port: driverPort,
        // 复用模式不设 browserName：service 在自启模式才删它；tauri-driver 接受裸 caps
        capabilities: [
          {
            maxInstances: 1,
            "tauri:options": {
              application: GOGUO_BIN,
            },
          } as WebdriverIO.Capabilities,
        ],
      }
    : {
        // 自启模式：@wdio/tauri-service 全权管理 tauri-driver 生命周期
        services: [
          [
            "@wdio/tauri-service",
            {
              // v1.x "official" → v2 "external" 已 deprecated，直接用 "external"（v1.x 同样接受）
              driverProvider: "external",
            },
          ],
        ],
        capabilities: [
          {
            maxInstances: 1,
            browserName: "tauri",
            "tauri:options": {
              application: GOGUO_BIN,
            },
          } as WebdriverIO.Capabilities,
        ],
      }),

  logLevel: "warn" as const,
  bail: 0,
  waitforTimeout: 10_000,
  connectionRetryTimeout: 60_000,
  connectionRetryCount: 3,

  framework: "mocha",
  mochaOpts: {
    ui: "bdd",
    timeout: 60_000,
  },

  reporters: ["spec"],

  /**
   * WSL2 F111 教训：tauri-driver 拉起的 GoGuo 子进程继承本环境，
   * 必须 x11 避免 Weston 合成器在 VM resume 后冻结。
   *
   * F115 T-02/T-07：通过 helpers/env.ts 统一封装 WSL 检测 + GDK_BACKEND 设置，
   * 避免每个脚本/spec 各自处理。
   */
  onPrepare() {
    ensureX11Backend();
  },
};
