import { describe, it, expect, beforeEach, vi } from 'vitest';
import { useServiceStore, initializeServiceStore } from '../service-store';
import { getServiceStatus } from '../../lib/tauri-ipc';
import { subscribeServiceStarted, subscribeServiceStopped, subscribeAutoRecoveryTriggered } from '../../lib/events';

vi.mock('../../lib/tauri-ipc', () => ({
  getServiceStatus: vi.fn(),
}));

vi.mock('../../lib/events', () => ({
  subscribeServiceStarted: vi.fn((_cb) => Promise.resolve(() => {})),
  subscribeServiceStopped: vi.fn((_cb) => Promise.resolve(() => {})),
  subscribeAutoRecoveryTriggered: vi.fn((_cb) => Promise.resolve(() => {})),
}));

describe('service-store', () => {
  beforeEach(() => {
    useServiceStore.getState().reset();
    vi.clearAllMocks();
  });

  it('initial state is correct', () => {
    const state = useServiceStore.getState();
    expect(state.mihomoRunning).toBe(false);
    expect(state.proxyGuardRestartCount).toBe(0);
    expect(state.isLoading).toBe(false);
    expect(state.error).toBeNull();
  });

  it('fetchServiceStatus updates state on success', async () => {
    vi.mocked(getServiceStatus).mockResolvedValue({
      mihomo_running: true,
      proxy_guard_restart_count: 3,
    });

    await useServiceStore.getState().fetchServiceStatus();

    const state = useServiceStore.getState();
    expect(state.mihomoRunning).toBe(true);
    expect(state.proxyGuardRestartCount).toBe(3);
    expect(state.isLoading).toBe(false);
    expect(state.error).toBeNull();
  });

  it('fetchServiceStatus handles error', async () => {
    vi.mocked(getServiceStatus).mockRejectedValue(new Error('Network error'));

    await useServiceStore.getState().fetchServiceStatus();

    const state = useServiceStore.getState();
    expect(state.isLoading).toBe(false);
    expect(state.error).toBe('Network error');
  });

  it('handleServiceStarted updates mihomoRunning', () => {
    useServiceStore.getState().handleServiceStarted({ mihomo_running: true });

    const state = useServiceStore.getState();
    expect(state.mihomoRunning).toBe(true);
  });

  it('handleServiceStopped sets mihomoRunning to false', () => {
    useServiceStore.getState().handleServiceStarted({ mihomo_running: true });
    useServiceStore.getState().handleServiceStopped({ reason: 'manual', recovery_triggered: false, non_target_verification: null });

    const state = useServiceStore.getState();
    expect(state.mihomoRunning).toBe(false);
  });

  it('handleAutoRecoveryTriggered updates restart count', () => {
    useServiceStore.getState().handleAutoRecoveryTriggered({
      restart_attempts: 2,
      max_attempts: 5,
    });

    const state = useServiceStore.getState();
    expect(state.proxyGuardRestartCount).toBe(2);
  });

  it('reset restores initial state', async () => {
    vi.mocked(getServiceStatus).mockResolvedValue({
      mihomo_running: true,
      proxy_guard_restart_count: 3,
    });

    await useServiceStore.getState().fetchServiceStatus();
    useServiceStore.getState().reset();

    const state = useServiceStore.getState();
    expect(state.mihomoRunning).toBe(false);
    expect(state.proxyGuardRestartCount).toBe(0);
  });

  it('initializeServiceStore subscribes to events', async () => {
    initializeServiceStore();

    expect(subscribeServiceStarted).toHaveBeenCalled();
    expect(subscribeServiceStopped).toHaveBeenCalled();
    expect(subscribeAutoRecoveryTriggered).toHaveBeenCalled();
  });
});