/**
 * F115 T-02: 环境探测 helper（FR-2.1.2-R3 / FR-2.2.2-R1 / FR-2.2.4-R3）
 *
 * 集中管理 e2e 运行环境相关的读取与副作用。所有函数纯函数（除 ensureX11Backend 有
 * process.env 副作用），便于 L1 单测（T-03 vitest）。
 *
 * 契约边界见 design §3.2.1：
 *   isWSL:            WSL / 原生 Linux / 文件不存在
 *   ensureX11Backend: 已设 / 未设 / 非 WSL
 *   getTauriDriverPort: 未设 / 空 / 非数字 / 超范围 / 默认 4444
 *   shouldReuseDriver: 未设 / "0" / "1" / 其它值
 */

import { readFileSync } from "node:fs";

/**
 * 检测当前是否运行在 WSL2 环境。
 *
 * 判据：/proc/version 内容含 "microsoft"（WSL2 内核标识）。
 *
 * @returns true=WSL；false=原生 Linux 或其它；文件不存在返回 false
 */
export function isWSL(): boolean {
  try {
    const version = readFileSync("/proc/version", "utf-8");
    return version.includes("microsoft");
  } catch {
    return false;
  }
}

/**
 * 在 WSL 环境下强制设置 GDK_BACKEND=x11（F111 教训：Weston 合成器在 VM resume
 * 后不恢复 Wayland 事件投递，XWayland 不受影响）。
 *
 * - WSL + 未设 GDK_BACKEND：设为 "x11"，返回 true（已变更）
 * - WSL + 已设 GDK_BACKEND：保持现状，返回 false（未变更）
 * - 非 WSL：不做任何操作，返回 false（不适用）
 *
 * @returns 是否实际变更了 GDK_BACKEND
 */
export function ensureX11Backend(): boolean {
  if (!isWSL()) {
    return false;
  }
  if (process.env.GDK_BACKEND !== undefined) {
    return false;
  }
  process.env.GDK_BACKEND = "x11";
  return true;
}

const DEFAULT_DRIVER_PORT = 4444;
const VALID_PORT_MIN = 1;
const VALID_PORT_MAX = 65535;

/**
 * 读取 tauri-driver 端口（仅复用模式生效，T-08）。
 *
 * 边界：
 *   - env 未设 / 空字符串：返回默认 4444
 *   - 非数字（如 "abc"）：返回默认 4444（容错，避免崩 wdio）
 *   - 超范围（0 / >65535 / 负数）：返回默认 4444
 *   - 合法数字（1~65535）：返回解析后的数字
 *
 * @returns tauri-driver 端口号
 */
export function getTauriDriverPort(): number {
  const raw = process.env.TAURI_DRIVER_PORT;
  if (raw === undefined || raw === "") {
    return DEFAULT_DRIVER_PORT;
  }
  const parsed = Number(raw);
  if (!Number.isInteger(parsed)) {
    return DEFAULT_DRIVER_PORT;
  }
  if (parsed < VALID_PORT_MIN || parsed > VALID_PORT_MAX) {
    return DEFAULT_DRIVER_PORT;
  }
  return parsed;
}

/**
 * 判断是否启用 tauri-driver 复用模式（T-08）。
 *
 * 仅当 env TAURI_DRIVER_REUSE === "1" 时启用；其它值（包括 "0" / "true" /
 * "yes" / 未设）一律视为不启用，保持自启模式为默认（保守策略）。
 *
 * @returns 是否启用复用模式
 */
export function shouldReuseDriver(): boolean {
  return process.env.TAURI_DRIVER_REUSE === "1";
}
