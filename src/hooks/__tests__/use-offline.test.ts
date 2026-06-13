import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useOffline } from '../use-offline';
import * as serviceStore from '../../stores/service-store';

vi.mock('../../stores/service-store', () => ({
  useServiceStore: vi.fn(),
}));

function mockStore(overrides: { mihomoRunning?: boolean; fetchServiceStatus?: ReturnType<typeof vi.fn> } = {}) {
  const state = {
    mihomoRunning: overrides.mihomoRunning ?? true,
    fetchServiceStatus: overrides.fetchServiceStatus ?? vi.fn().mockResolvedValue(undefined),
  };
  vi.mocked(serviceStore.useServiceStore).mockImplementation(
    (selector?: (s: typeof state) => unknown) => selector ? selector(state) : state,
  );
  return state;
}

describe('use-offline', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('returns offline status', () => {
    mockStore({ mihomoRunning: true });

    const { result } = renderHook(() => useOffline());
    expect(result.current.isOffline).toBe(false);
    expect(result.current.lastKnownStatus).toBe(true);
  });

  it('detects offline when fetch fails', async () => {
    mockStore({
      mihomoRunning: true,
      fetchServiceStatus: vi.fn().mockRejectedValue(new Error('Connection failed')),
    });

    const { result } = renderHook(() => useOffline());

    await act(async () => {
      vi.advanceTimersByTime(5000);
    });

    expect(result.current.isOffline).toBe(true);
  });

  it('stores last known status', () => {
    mockStore({ mihomoRunning: false });

    const { result } = renderHook(() => useOffline());
    expect(result.current.lastKnownStatus).toBe(false);
  });

  it('periodically checks connection', async () => {
    const fetchServiceStatus = vi.fn().mockResolvedValue(undefined);
    mockStore({ fetchServiceStatus });

    renderHook(() => useOffline());

    await act(async () => {
      vi.advanceTimersByTime(5000);
    });

    expect(fetchServiceStatus).toHaveBeenCalledTimes(2);
  });

  it('does NOT re-run effect when mihomoRunning changes via ref', async () => {
    const fetchServiceStatus = vi.fn().mockResolvedValue(undefined);
    let currentState = { mihomoRunning: true, fetchServiceStatus };

    vi.mocked(serviceStore.useServiceStore).mockImplementation(
      (selector?: (s: typeof currentState) => unknown) => selector ? selector(currentState) : currentState,
    );

    const { rerender } = renderHook(() => useOffline());

    // Initial call + not triggered again by mihomoRunning change
    const callsBefore = fetchServiceStatus.mock.calls.length;

    // Change mihomoRunning (simulating backend event)
    currentState = { mihomoRunning: false, fetchServiceStatus };
    rerender();

    // Effect should NOT have re-run (no additional immediate checkConnection call)
    expect(fetchServiceStatus.mock.calls.length).toBe(callsBefore);
  });
});