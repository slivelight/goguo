import path from "node:path";

const GOGUO_BIN = process.env.GOGUO_BIN
  ?? path.resolve(import.meta.dirname, "..", "target", "release", "goguo");

export const config: WebdriverIO.Config = {
  runner: "local",
  specs: ["./test/specs/**/*.spec.ts"],
  maxInstances: 1,

  // 关键：用 'tauri' 而非 'wry'（@wdio/tauri-service 已注册该 browserName）
  capabilities: [
    {
      maxInstances: 1,
      browserName: "tauri",
      "tauri:options": {
        application: GOGUO_BIN,
      },
    } as WebdriverIO.Capabilities,
  ],

  // @wdio/tauri-service 自动管理 tauri-driver 生命周期
  services: [
    [
      "@wdio/tauri-service",
      {
        // 复用系统已装的 tauri-driver v2.0.6（cargo install 装的）
        driverProvider: "official",
      },
    ],
  ],

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
   */
  onPrepare() {
    process.env.GDK_BACKEND = process.env.GDK_BACKEND ?? "x11";
  },
};
