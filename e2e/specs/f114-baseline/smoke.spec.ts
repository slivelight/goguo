/**
 * 冒烟测试：验证 WebDriverIO + tauri-driver + GoGuo 三者能联动。
 * 不依赖具体 UI 文案——只要窗口启动、DOM 可见即视为通过。
 *
 * F115 T-07：spec before 调用 resetGoGuoState()（FR-2.2.1-R3 状态隔离），
 * 防止 cross-spec session 复用场景下前一 spec 的残留状态污染本 spec。
 */
import { resetGoGuoState } from "../../helpers/state";
import { waitForGoGuoReady } from "../../helpers/wait";

describe("F114: GoGuo 启动冒烟", () => {
  // session 复用模式下，spec 入口清理一次（beforeEach 对冒烟无意义，无状态变更）
  before(async () => {
    await resetGoGuoState();
  });

  it("应渲染非空 body", async () => {
    const body = await waitForGoGuoReady();
    const text = (await body.getText()).trim();
    expect(text.length).toBeGreaterThan(0);
  });

  it("窗口标题应包含 GoGuo 或 Tauri", async () => {
    const title = await browser.getTitle();
    expect(title).toMatch(/GoGuo|Tauri/i);
  });
});
