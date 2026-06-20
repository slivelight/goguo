/**
 * F115 FR-2.2.1-R1/R2：cross-spec session 复用验证。
 *
 * 三条 UX 操作（design §N.2.5）：
 *   1) 读 browser.sessionId → 期望非空 UUID
 *   2) 同 spec 内 invoke('list_target_sites') → 期望返回数组（无需新建 session）
 *   3) 合跑日志 Session ID 计数 = 1 → 由 T-12 benchmark.sh 验证（本 spec 仅打日志便于后续 grep）
 *
 * 实现说明：
 *   - spec 内不能直接 grep 自己的 wdio 输出（stdout 由 runner 持有），故操作 3 在 T-12 脚本层完成
 *   - 本 spec 落地操作 1+2，并在 afterEach 中打印 sessionId 供人工/脚本核对
 */
import { invokeTauri, isTauriInvokeError } from "../../helpers/tauri-ipc";
import { resetGoGuoState } from "../../helpers/state";

const UUID_RE = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

describe("F115: cross-spec session 复用", () => {
  before(async () => {
    await resetGoGuoState();
  });

  it("操作1: browser.sessionId 应为非空 UUID 字符串", async () => {
    const sid = browser.sessionId;
    expect(typeof sid).toBe("string");
    expect(sid.length).toBeGreaterThan(0);
    expect(sid).toMatch(UUID_RE);
    // eslint-disable-next-line no-console
    console.log(`[F115 session-reuse] sessionId = ${sid}`);
  });

  it("操作2: 同 session 内 invoke('list_target_sites') 应返回数组", async () => {
    const resp = await invokeTauri<string[]>("list_target_sites");
    if (isTauriInvokeError(resp)) {
      throw new Error(`IPC 调用失败: ${resp.__error}`);
    }
    expect(Array.isArray(resp)).toBe(true);
  });
});
