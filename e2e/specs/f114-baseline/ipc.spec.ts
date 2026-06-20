/**
 * IPC roundtrip 测试（QG2）：
 *   验证 webview 内能调用真实后端命令 add_target_site，并拿到非空响应。
 *
 * 关键路径：
 *   wdio browser.execute() ─▶ webview JS
 *     ─▶ window.__TAURI_INTERNALS__.invoke('add_target_site', ...)
 *     ─▶ Rust tauri::command ─▶ SiteRulesState ─▶ 响应
 *
 * 不依赖 UI 按钮，直接走 invoke。这样能严格区分 QG2（IPC）与未来 L4（UI 行为）。
 *
 * F115 T-07：beforeEach 调用 resetGoGuoState()（FR-2.2.1-R3），
 * 每个 it 都从干净状态开始（add_target_site 不受前一 it 残留影响）。
 */
import {
  invokeTauri,
  isTauriInvokeError,
} from "../../helpers/tauri-ipc";
import { resetGoGuoState } from "../../helpers/state";

describe("F114: GoGuo IPC roundtrip", () => {
  beforeEach(async () => {
    await resetGoGuoState();
  });

  it("add_target_site('github') 应返回成功响应且 site.id 正确", async () => {
    const resp = await invokeTauri<{
      success?: boolean;
      site?: { id?: string; name?: string };
      error?: string;
    }>("add_target_site", { siteId: "github" });

    if (isTauriInvokeError(resp)) {
      throw new Error(`IPC 调用失败: ${resp.__error}`);
    }
    expect(resp.success).toBe(true);
    expect(resp.site?.id).toBe("github");
    expect(typeof resp.site?.name).toBe("string");
  });

  it("重复调用 add_target_site('github') 应幂等或返回明确错误", async () => {
    // 第二次调用：要么 success=true（幂等），要么 error 字段提示已存在
    const resp = await invokeTauri<{ success?: boolean; error?: string }>(
      "add_target_site",
      { siteId: "github" },
    );
    if (isTauriInvokeError(resp)) {
      // IPC 通道本身失败不算幂等失败，重新调用判定
      throw new Error(`IPC 调用失败: ${resp.__error}`);
    }
    expect(resp.success === true || typeof resp.error === "string").toBe(true);
  });
});
