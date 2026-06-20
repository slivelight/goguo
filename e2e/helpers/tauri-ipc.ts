/**
 * F115 T-02: Tauri IPC helper（FR-2.1.2-R1）
 *
 * 在 webview 内调用 Tauri 后端命令。GoGuo 默认 withGlobalTauri=false，
 * 因此必须走 window.__TAURI_INTERNALS__.invoke（而非 window.__TAURI__.invoke）。
 *
 * 关键路径：
 *   wdio browser.executeAsync() ─▶ webview JS
 *     ─▶ window.__TAURI_INTERNALS__.invoke(cmd, args)
 *     ─▶ Rust tauri::command ─▶ 响应
 *
 * 错误处理：若 internals 不存在或 invoke 抛错，返回 { __error: string }，
 * 由调用方决定是否 throw（保持与 PoC 行为一致，便于断言）。
 */

export interface TauriInvokeError {
  __error: string;
}

export type TauriInvokeResult<T> = T | TauriInvokeError;

export function isTauriInvokeError(
  resp: unknown,
): resp is TauriInvokeError {
  return (
    typeof resp === "object" &&
    resp !== null &&
    "__error" in resp &&
    typeof (resp as { __error: unknown }).__error === "string"
  );
}

/**
 * 在 webview 内调用 Tauri 后端命令。
 *
 * @param cmd Tauri 命令名（如 "add_target_site"）
 * @param args 命令参数（Record<string, unknown>）
 * @returns 命令响应；若 IPC 通道缺失或抛错，返回 { __error }
 */
export async function invokeTauri<T = unknown>(
  cmd: string,
  args: Record<string, unknown> = {},
): Promise<TauriInvokeResult<T>> {
  return browser.executeAsync((cmd, args, done) => {
    const internals = (window as unknown as {
      __TAURI_INTERNALS__?: {
        invoke: (c: string, a?: unknown) => Promise<unknown>;
      };
    }).__TAURI_INTERNALS__;
    if (!internals?.invoke) {
      done({ __error: "TAURI_INTERNALS_MISSING" });
      return;
    }
    internals.invoke(cmd, args).then(done, (e: unknown) =>
      done({ __error: String(e) }),
    );
  }, cmd, args) as Promise<TauriInvokeResult<T>>;
}

/**
 * 断言版本：若 IPC 失败直接 throw，否则返回 T（剥离 __error 字段）。
 *
 * 适用于"必须成功"的用例（如验收测试）；幂等/可失败用例用 invokeTauri 自行判断。
 */
export async function invokeTauriOrThrow<T = unknown>(
  cmd: string,
  args: Record<string, unknown> = {},
): Promise<T> {
  const resp = await invokeTauri<T>(cmd, args);
  if (isTauriInvokeError(resp)) {
    throw new Error(`IPC 调用失败 (${cmd}): ${resp.__error}`);
  }
  return resp;
}
