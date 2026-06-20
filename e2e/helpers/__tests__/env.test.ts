/**
 * F115 T-03: env.ts L1 单测（FR-2.2.4-R3 / OQ-7）
 *
 * 覆盖 design §3.2.1 全部边界用例：
 *   isWSL:            WSL / 原生 Linux / 文件不存在
 *   ensureX11Backend: 已设 / 未设 / 非 WSL
 *   getTauriDriverPort: 未设 / 空 / 非数字 / 超范围 / 默认 4444
 *   shouldReuseDriver: 未设 / "0" / "1" / 其它值
 *
 * ESM 限制：不能用 vi.spyOn(node:fs)，改用 vi.mock（hoisted）。
 */

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

// vi.mock 是 hoisted（提升到 import 之前）
vi.mock("node:fs", () => ({
  readFileSync: vi.fn(),
}));

import { readFileSync } from "node:fs";
import {
  ensureX11Backend,
  getTauriDriverPort,
  isWSL,
  shouldReuseDriver,
} from "../env";

const mockedReadFileSync = readFileSync as unknown as ReturnType<typeof vi.fn>;

// 保存原始 env，每个 it 后恢复
const ORIGINAL_ENV = { ...process.env };
const ORIGINAL_GDK = process.env.GDK_BACKEND;
const ORIGINAL_PORT = process.env.TAURI_DRIVER_PORT;
const ORIGINAL_REUSE = process.env.TAURI_DRIVER_REUSE;

beforeEach(() => {
  delete process.env.GDK_BACKEND;
  delete process.env.TAURI_DRIVER_PORT;
  delete process.env.TAURI_DRIVER_REUSE;
  mockedReadFileSync.mockReset();
});

afterEach(() => {
  process.env = { ...ORIGINAL_ENV };
  if (ORIGINAL_GDK === undefined) {
    delete process.env.GDK_BACKEND;
  } else {
    process.env.GDK_BACKEND = ORIGINAL_GDK;
  }
  if (ORIGINAL_PORT === undefined) {
    delete process.env.TAURI_DRIVER_PORT;
  } else {
    process.env.TAURI_DRIVER_PORT = ORIGINAL_PORT;
  }
  if (ORIGINAL_REUSE === undefined) {
    delete process.env.TAURI_DRIVER_REUSE;
  } else {
    process.env.TAURI_DRIVER_REUSE = ORIGINAL_REUSE;
  }
  vi.restoreAllMocks();
});

describe("isWSL", () => {
  it("WSL: /proc/version 含 'microsoft' 返回 true", () => {
    mockedReadFileSync.mockReturnValue(
      "Linux version 5.15.153.1-microsoft-standard-WSL2 ...",
    );
    expect(isWSL()).toBe(true);
  });

  it("原生 Linux: /proc/version 不含 'microsoft' 返回 false", () => {
    mockedReadFileSync.mockReturnValue(
      "Linux version 6.6.0-generic ...",
    );
    expect(isWSL()).toBe(false);
  });

  it("文件不存在: readFileSync 抛错时返回 false（不崩）", () => {
    mockedReadFileSync.mockImplementation(() => {
      throw new Error("ENOENT: no such file");
    });
    expect(isWSL()).toBe(false);
  });
});

describe("ensureX11Backend", () => {
  it("非 WSL: 不做任何操作，返回 false", () => {
    mockedReadFileSync.mockReturnValue(
      "Linux version 6.6.0-generic ...",
    );
    expect(ensureX11Backend()).toBe(false);
    expect(process.env.GDK_BACKEND).toBeUndefined();
  });

  it("WSL + 未设 GDK_BACKEND: 设为 x11，返回 true", () => {
    mockedReadFileSync.mockReturnValue(
      "Linux version 5.15-microsoft-standard-WSL2 ...",
    );
    expect(ensureX11Backend()).toBe(true);
    expect(process.env.GDK_BACKEND).toBe("x11");
  });

  it("WSL + 已设 GDK_BACKEND=wayland: 保持现状，返回 false", () => {
    mockedReadFileSync.mockReturnValue(
      "Linux version 5.15-microsoft-standard-WSL2 ...",
    );
    process.env.GDK_BACKEND = "wayland";
    expect(ensureX11Backend()).toBe(false);
    expect(process.env.GDK_BACKEND).toBe("wayland");
  });
});

describe("getTauriDriverPort", () => {
  it("env 未设: 返回默认 4444", () => {
    expect(getTauriDriverPort()).toBe(4444);
  });

  it("env 为空字符串: 返回默认 4444", () => {
    process.env.TAURI_DRIVER_PORT = "";
    expect(getTauriDriverPort()).toBe(4444);
  });

  it("env 为非数字 'abc': 返回默认 4444（容错）", () => {
    process.env.TAURI_DRIVER_PORT = "abc";
    expect(getTauriDriverPort()).toBe(4444);
  });

  it("env 超范围 '0': 返回默认 4444", () => {
    process.env.TAURI_DRIVER_PORT = "0";
    expect(getTauriDriverPort()).toBe(4444);
  });

  it("env 超范围 '70000': 返回默认 4444", () => {
    process.env.TAURI_DRIVER_PORT = "70000";
    expect(getTauriDriverPort()).toBe(4444);
  });

  it("env 负数 '-1': 返回默认 4444（Number.isInteger(-1)=true 但范围非法）", () => {
    process.env.TAURI_DRIVER_PORT = "-1";
    expect(getTauriDriverPort()).toBe(4444);
  });

  it("env 合法 '8080': 返回 8080", () => {
    process.env.TAURI_DRIVER_PORT = "8080";
    expect(getTauriDriverPort()).toBe(8080);
  });

  it("env 浮点 '80.5': 返回默认 4444（非整数）", () => {
    process.env.TAURI_DRIVER_PORT = "80.5";
    expect(getTauriDriverPort()).toBe(4444);
  });

  it("边界值 '1': 返回 1（最小合法端口）", () => {
    process.env.TAURI_DRIVER_PORT = "1";
    expect(getTauriDriverPort()).toBe(1);
  });

  it("边界值 '65535': 返回 65535（最大合法端口）", () => {
    process.env.TAURI_DRIVER_PORT = "65535";
    expect(getTauriDriverPort()).toBe(65535);
  });
});

describe("shouldReuseDriver", () => {
  it("env 未设: 返回 false（默认自启模式）", () => {
    expect(shouldReuseDriver()).toBe(false);
  });

  it("env '0': 返回 false", () => {
    process.env.TAURI_DRIVER_REUSE = "0";
    expect(shouldReuseDriver()).toBe(false);
  });

  it("env '1': 返回 true", () => {
    process.env.TAURI_DRIVER_REUSE = "1";
    expect(shouldReuseDriver()).toBe(true);
  });

  it("env 'true': 返回 false（严格匹配，不接受 true/yes 等变体）", () => {
    process.env.TAURI_DRIVER_REUSE = "true";
    expect(shouldReuseDriver()).toBe(false);
  });

  it("env 'yes': 返回 false", () => {
    process.env.TAURI_DRIVER_REUSE = "yes";
    expect(shouldReuseDriver()).toBe(false);
  });

  it("env 空字符串: 返回 false", () => {
    process.env.TAURI_DRIVER_REUSE = "";
    expect(shouldReuseDriver()).toBe(false);
  });
});
