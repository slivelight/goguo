/**
 * F115 T-04b: e2e 状态隔离 helper（FR-2.2.1-R3 / design §2.1.3）
 *
 * `resetGoGuoState()` 在 spec `beforeEach` 调用，清理 SiteRulesState 等
 * 可变状态，防止跨 spec 状态污染。
 *
 * 实现策略（design §2.1.3）：
 *   1. invokeTauri("list_target_sites") 拿当前 active site id 列表
 *      （T-04a 落地的后端命令，spec FR-2.2.5）
 *   2. 逐个 invokeTauri("remove_target_site", { siteId })
 *
 * 容错策略（best-effort）：
 *   - list 阶段失败：拿不到列表，直接返回（fail-soft；spec 不应因 IPC 不可用中断）
 *   - 单个 remove 失败：记录但继续清理其余（best-effort）
 *   - 最终一致性由下一次 list 时的"漏清理"暴露（不在此 helper 内重试）
 */

import { invokeTauri, isTauriInvokeError } from "./tauri-ipc";

/**
 * 清理 GoGuo 可变状态（目前仅 SiteRulesState）。
 *
 * 幂等：list 为空时是 noop。
 * 安全：不抛错（spec beforeEach 调用不应中断测试）。
 *
 * @returns 清理过程中删除的 site 数（便于断言/调试；list 失败时返回 -1）
 */
export async function resetGoGuoState(): Promise<number> {
  const listResp = await invokeTauri<string[]>("list_target_sites");

  // list 失败：拿不到清单，无法清理；spec 继续跑（fail-soft）
  if (isTauriInvokeError(listResp)) {
    return -1;
  }

  // best-effort 清理：单个失败不阻塞其余
  let removed = 0;
  for (const siteId of listResp) {
    const removeResp = await invokeTauri("remove_target_site", { siteId });
    if (!isTauriInvokeError(removeResp)) {
      removed += 1;
    }
  }
  return removed;
}
