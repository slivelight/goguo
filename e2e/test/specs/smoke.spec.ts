/**
 * 冒烟测试：验证 WebDriverIO + tauri-driver + GoGuo 三者能联动。
 * 不依赖具体 UI 文案——只要窗口启动、DOM 可见即视为通过。
 */
describe("GoGuo 启动冒烟", () => {
  it("应渲染非空 body", async () => {
    const body = await $("body");
    await body.waitForExist({ timeout: 15_000 });
    const text = (await body.getText()).trim();
    expect(text.length).toBeGreaterThan(0);
  });

  it("窗口标题应包含 GoGuo 或 Tauri", async () => {
    const title = await browser.getTitle();
    expect(title).toMatch(/GoGuo|Tauri/i);
  });
});
