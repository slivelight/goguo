/**
 * F115 T-02: GoGuo 就绪等待 helper（FR-2.1.2-R2）
 *
 * 等待 GoGuo webview 渲染出非空 body。WebKitGTK 在 invoke() IPC 期间不渲染，
 * 需要让出 GLib 主循环；wdio 的 waitForExist 已封装重试。
 */

/**
 * 等待 GoGuo webview body 存在且非空。
 *
 * @param timeout 等待超时（毫秒），默认 15s（沿用 PoC 实测值）
 * @returns body 元素（可链式调用 getText / 点击等）
 */
export async function waitForGoGuoReady(
  timeout = 15_000,
): Promise<WebdriverIO.Element> {
  const body = await $("body");
  await body.waitForExist({ timeout });
  return body;
}
