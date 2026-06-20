/**
 * F115 T-04b: state.ts L1 单测（FR-2.2.1-R3 / OQ-7）
 *
 * 覆盖 design §2.1.3 + §N.2.1 三种边界：
 *   resetGoGuoState: 空状态 / 单站点 / 多站点
 *
 * 关键路径（mock）：
 *   resetGoGuoState()
 *     ─▶ invokeTauri("list_target_sites")      // 拿到当前 active sites
 *     ─▶ for each site: invokeTauri("remove_target_site", { siteId })
 *
 * 必须用 vi.mock 隔离 tauri-ipc（其内部依赖 wdio `browser.executeAsync`，
 * 在 vitest node 环境会抛错）。ESM 限制下 vi.mock 是 hoisted。
 */

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

// vi.mock 是 hoisted（提升到 import 之前）
vi.mock("../tauri-ipc", () => ({
  invokeTauri: vi.fn(),
  isTauriInvokeError: vi.fn(
    (resp: unknown) =>
      typeof resp === "object" &&
      resp !== null &&
      "__error" in resp &&
      typeof (resp as { __error: unknown }).__error === "string",
  ),
}));

import { invokeTauri } from "../tauri-ipc";
import { resetGoGuoState } from "../state";

const mockedInvokeTauri = invokeTauri as unknown as ReturnType<typeof vi.fn>;

beforeEach(() => {
  mockedInvokeTauri.mockReset();
});

afterEach(() => {
  vi.restoreAllMocks();
});

describe("resetGoGuoState", () => {
  it("空状态: list_target_sites 返回 [] 时无 remove 调用，返回 0", async () => {
    mockedInvokeTauri.mockResolvedValueOnce([]); // list_target_sites

    const removed = await resetGoGuoState();

    expect(removed).toBe(0);
    // 仅 1 次 list 调用，0 次 remove
    expect(mockedInvokeTauri).toHaveBeenCalledTimes(1);
    expect(mockedInvokeTauri).toHaveBeenCalledWith("list_target_sites");
  });

  it("单站点: list 返回 ['github'] 时调用 1 次 remove_target_site，返回 1", async () => {
    mockedInvokeTauri.mockResolvedValueOnce(["github"]); // list
    mockedInvokeTauri.mockResolvedValueOnce({ success: true }); // remove github

    const removed = await resetGoGuoState();

    expect(removed).toBe(1);
    expect(mockedInvokeTauri).toHaveBeenCalledTimes(2);
    expect(mockedInvokeTauri).toHaveBeenNthCalledWith(1, "list_target_sites");
    expect(mockedInvokeTauri).toHaveBeenNthCalledWith(2, "remove_target_site", {
      siteId: "github",
    });
  });

  it("多站点: list 返回 ['github', 'npmjs', 'docker'] 时调用 3 次 remove_target_site（顺序一致），返回 3", async () => {
    mockedInvokeTauri.mockResolvedValueOnce(["github", "npmjs", "docker"]);
    mockedInvokeTauri.mockResolvedValueOnce({ success: true }); // github
    mockedInvokeTauri.mockResolvedValueOnce({ success: true }); // npmjs
    mockedInvokeTauri.mockResolvedValueOnce({ success: true }); // docker

    const removed = await resetGoGuoState();

    expect(removed).toBe(3);
    expect(mockedInvokeTauri).toHaveBeenCalledTimes(4); // 1 list + 3 remove
    expect(mockedInvokeTauri).toHaveBeenNthCalledWith(1, "list_target_sites");
    expect(mockedInvokeTauri).toHaveBeenNthCalledWith(
      2,
      "remove_target_site",
      { siteId: "github" },
    );
    expect(mockedInvokeTauri).toHaveBeenNthCalledWith(
      3,
      "remove_target_site",
      { siteId: "npmjs" },
    );
    expect(mockedInvokeTauri).toHaveBeenNthCalledWith(
      4,
      "remove_target_site",
      { siteId: "docker" },
    );
  });

  it("list_target_sites IPC 失败 (__error) 时: 不抛、不调用 remove，返回 -1（fail-soft）", async () => {
    mockedInvokeTauri.mockResolvedValueOnce({ __error: "TAURI_INTERNALS_MISSING" });

    // 不应抛错（spec before/beforeEach 调用不应因 IPC 不可用而中断测试）
    const removed = await resetGoGuoState();

    expect(removed).toBe(-1);
    // 仅 list 调用，remove 0 次（防御：拿不到列表就放弃清理）
    expect(mockedInvokeTauri).toHaveBeenCalledTimes(1);
    expect(mockedInvokeTauri).toHaveBeenCalledWith("list_target_sites");
  });

  it("remove_target_site 单点失败时: 继续清理其余站点（best-effort），返回成功删除数", async () => {
    mockedInvokeTauri.mockResolvedValueOnce(["github", "npmjs"]);
    mockedInvokeTauri.mockResolvedValueOnce({ __error: "IPC error" }); // github 失败
    mockedInvokeTauri.mockResolvedValueOnce({ success: true }); // npmjs 成功

    // 不应抛错
    const removed = await resetGoGuoState();

    expect(removed).toBe(1); // 只有 npmjs 成功
    // 两次 remove 都尝试了（即便第一次失败）
    expect(mockedInvokeTauri).toHaveBeenCalledTimes(3);
    expect(mockedInvokeTauri).toHaveBeenNthCalledWith(
      2,
      "remove_target_site",
      { siteId: "github" },
    );
    expect(mockedInvokeTauri).toHaveBeenNthCalledWith(
      3,
      "remove_target_site",
      { siteId: "npmjs" },
    );
  });
});
