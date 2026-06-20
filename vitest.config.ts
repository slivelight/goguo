import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'jsdom',
    globals: true,
    // F115 T-10：显式 include 仅 src/，避免扫到 e2e/（wdio 全局缺失）和
    // .opencode/node_modules（zod 自带 test 拖累）
    include: ['src/**/*.{test,spec}.{js,ts,jsx,tsx}'],
  },
});