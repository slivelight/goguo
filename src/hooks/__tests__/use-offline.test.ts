import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useOffline } from '../use-offline';
import * as serviceStore from '../../stores/service-store';

vi.mock('../../stores/service-store', () => ({
  useServiceStore: vi.fn(),
}));

describe('use-offline', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('returns offline status', () => {
    vi.mocked(serviceStore.useServiceStore).mockReturnValue({
      mihomoRunning: true,
      fetchServiceStatus: vi.fn().mockResolvedValue(undefined),
    } as unknown as ReturnType<typeof serviceStore.useServiceStore>);

    const { result } = renderHook(() => useOffline());
    expect(result.current.isOffline).toBe(false);
    expect(result.current.lastKnownStatus).toBe(true);
  });

  it('detects offline when fetch fails', async () => {
    vi.mocked(serviceStore.useServiceStore).mockReturnValue({
      mihomoRunning: true,
      fetchServiceStatus: vi.fn().mockRejectedValue(new Error('Connection failed')),
    } as unknown as ReturnType<typeof serviceStore.useServiceStore>);

    const { result } = renderHook(() => useOffline());
    
    await act(async () => {
      vi.advanceTimersByTime(5000);
    });

    expect(result.current.isOffline).toBe(true);
  });

  it('stores last known status', () => {
    vi.mocked(serviceStore.useServiceStore).mockReturnValue({
      mihomoRunning: false,
      fetchServiceStatus: vi.fn().mockResolvedValue(undefined),
    } as unknown as ReturnType<typeof serviceStore.useServiceStore>);

    const { result } = renderHook(() => useOffline());
    expect(result.current.lastKnownStatus).toBe(false);
  });

  it('periodically checks connection', async () => {
    const fetchServiceStatus = vi.fn().mockResolvedValue(undefined);
    vi.mocked(serviceStore.useServiceStore).mockReturnValue({
      mihomoRunning: true,
      fetchServiceStatus,
    } as unknown as ReturnType<typeof serviceStore.useServiceStore>);

    renderHook(() => useOffline());
    
    await act(async () => {
      vi.advanceTimersByTime(5000);
    });

    expect(fetchServiceStatus).toHaveBeenCalledTimes(2);
  });
});