/**
 * F115 FR-2.2.2-R1：driver 复用模式（TAURI_DRIVER_REUSE=1）验证。
 *
 * 三条 UX 操作（design §N.2.5）：
 *   1) 检测 env TAURI_DRIVER_REUSE → 非 "1" 跳过（this.skip()），避免污染自启模式
 *   2) 复用模式下 browser.sessionId → 非空字符串（证明连到外部 tauri-driver）
 *   3) 同 spec 内 invoke('add_target_site') → success=true（复用模式下 IPC 路径完整）
 *
 * 运行方式（README §运行 - 复用模式）：
 *   ./scripts/start-driver.sh
 *   TAURI_DRIVER_REUSE=1 pnpm test
 *   ./scripts/stop-driver.sh
 */
import { invokeTauri, isTauriInvokeError } from "../../helpers/tauri-ipc";
import { resetGoGuoState } from "../../helpers/state";

describe("F115: driver 复用模式", () => {
  before(function () {
    // 操作1: 非 "1" 则跳过——本 spec 仅在复用模式下有意义
    if (process.env.TAURI_DRIVER_REUSE !== "1") {
      this.skip();
    }
  });

  beforeEach(async () => {
    await resetGoGuoState();
  });

  it("操作2: 复用模式下 browser.sessionId 应为非空字符串", async () => {
    const sid = browser.sessionId;
    expect(typeof sid).toBe("string");
    expect(sid.length).toBeGreaterThan(0);
    // eslint-disable-next-line no-console
    console.log(`[F115 driver-reuse] sessionId = ${sid} (reuse mode)`);
  });

  it("操作3: 复用模式下 add_target_site('github') 应返回 success=true", async () => {
    const resp = await invokeTauri<{ success?: boolean; site?: { id?: string } }>(
      "add_target_site",
      { siteId: "github" },
    );
    if (isTauriInvokeError(resp)) {
      throw new Error(`IPC 调用失败: ${resp.__error}`);
    }
    expect(resp.success).toBe(true);
    expect(resp.site?.id).toBe("github");
  });
});
