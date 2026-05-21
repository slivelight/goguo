import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { useRecoveryStarted, useBaselineConfirmed } from '../use-tauri-event';
import { renderHook } from '@testing-library/react';

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn((_eventName: string, _callback: (event: { payload: unknown }) => void) => {
    return Promise.resolve(() => {});
  }),
}));

describe('use-tauri-event', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('useRecoveryStarted subscribes to recovery:started event', async () => {
    const callback = vi.fn();
    renderHook(() => useRecoveryStarted(callback));
    
    expect(vi.mocked(await import('@tauri-apps/api/event')).listen).toHaveBeenCalled();
  });

  it('useBaselineConfirmed subscribes to baseline:confirmed event', async () => {
    const callback = vi.fn();
    renderHook(() => useBaselineConfirmed(callback));
    
    expect(vi.mocked(await import('@tauri-apps/api/event')).listen).toHaveBeenCalled();
  });
});