/**
 * F115 FR-2.2.3-R3：tauri-plugin-wdio 注册验证。
 *
 * 三条 UX 操作（design §N.2.5）：
 *   1) browser.execute(() => typeof window.wdioTauri) → 期望 "object"
 *   2) browser.execute(() => typeof window.wdioTauri.execute) → 期望 "function"
 *   3) wdio 输出全文 grep "Tauri plugin not available" → 计数 = 0
 *      （由 T-09 build 已保证；本 spec 不在运行时 grep 自身输出，操作 3 在 finalize/T-12 验）
 *
 * 前置条件（T-09 6 步完整集成）：
 *   - Rust: tauri-plugin-wdio = "1" + lib.rs .plugin(tauri_plugin_wdio::init())
 *   - Capability: wdio:default
 *   - tauri.conf.json: withGlobalTauri = true
 *   - 前端: pnpm add @wdio/tauri-plugin + main.tsx import '@wdio/tauri-plugin'
 */
import { resetGoGuoState } from "../../helpers/state";

describe("F115: tauri-plugin-wdio 注册", () => {
  before(async () => {
    await resetGoGuoState();
  });

  it("操作1: window.wdioTauri 应已注册（typeof === 'object'）", async () => {
    const type = await browser.execute(() => typeof window.wdioTauri);
    expect(type).toBe("object");
  });

  it("操作2: window.wdioTauri.execute 应为 function（Execute API 可用）", async () => {
    const type = await browser.execute(() => {
      // @ts-expect-error wdioTauri 由 plugin 运行时注入，TS 无类型声明
      return typeof window.wdioTauri?.execute;
    });
    expect(type).toBe("function");
  });
});
